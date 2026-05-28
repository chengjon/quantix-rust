use crate::core::Result;
/// 智能采集调度器
///
/// 从短线侠项目迁移，根据交易时段动态调整采集频率
use crate::core::trading_calendar::{TradingCalendar, TradingSession};
use crate::sources::quote_collector::{QuoteCollector, StockInfo as QuoteStockInfo};
use crate::sources::tdx::StockQuote;
use chrono::{DateTime, Duration, Timelike, Utc};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 调度器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerState {
    /// 活跃状态 - 正在采集数据
    Active,
    /// 非活跃状态 - 非交易时间
    Inactive,
    /// 盘前状态 - 接近开盘
    PreMarket,
    /// 盘后状态 - 收盘后清理
    PostMarket,
}

impl SchedulerState {
    pub fn display_name(&self) -> &'static str {
        match self {
            SchedulerState::Active => "活跃",
            SchedulerState::Inactive => "非活跃",
            SchedulerState::PreMarket => "盘前",
            SchedulerState::PostMarket => "盘后",
        }
    }
}

/// 调度器配置
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// 强制模式（用于测试）
    pub force_mode: bool,
    /// 盘前检查时间（分钟）
    pub pre_market_minutes: u64,
    /// 盘后检查时间（分钟）
    pub post_market_minutes: u64,
    /// 非交易时间检查间隔（秒）
    pub inactive_check_interval: u64,
    /// 交易时段采集间隔（秒）
    pub active_check_interval: u64,
    /// 竞价时段采集间隔（秒）
    pub auction_check_interval: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        // 支持通过环境变量配置
        let force_mode = std::env::var("FORCE_MODE").is_ok();

        Self {
            force_mode,
            pre_market_minutes: 30,
            post_market_minutes: 30,
            inactive_check_interval: 300, // 5分钟
            active_check_interval: 60,    // 1分钟
            auction_check_interval: 30,   // 30秒
        }
    }
}

/// 采集任务回调类型
pub type CollectCallback = Arc<dyn Fn(Vec<StockQuote>) + Send + Sync + 'static>;

/// 智能采集调度器
pub struct CollectScheduler {
    /// 交易日历
    calendar: TradingCalendar,
    /// 配置
    config: SchedulerConfig,
    /// 行情采集器
    collector: Arc<QuoteCollector>,
    /// 股票列表
    stock_list: Arc<RwLock<Vec<QuoteStockInfo>>>,
    /// 采集回调
    on_quotes_collected: Arc<RwLock<Option<CollectCallback>>>,
    /// 运行状态
    running: Arc<RwLock<bool>>,
}

impl CollectScheduler {
    /// 创建新的调度器
    pub async fn new(collector: QuoteCollector) -> Result<Self> {
        let calendar = TradingCalendar::new().await?;
        let config = SchedulerConfig::default();

        info!("智能采集调度器初始化完成");

        Ok(Self {
            calendar,
            config,
            collector: Arc::new(collector),
            stock_list: Arc::new(RwLock::new(Vec::new())),
            on_quotes_collected: Arc::new(RwLock::new(None)),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// 使用自定义配置创建调度器
    pub async fn with_config(collector: QuoteCollector, config: SchedulerConfig) -> Result<Self> {
        let calendar = TradingCalendar::new().await?;

        info!("智能采集调度器初始化完成（自定义配置）");

        Ok(Self {
            calendar,
            config,
            collector: Arc::new(collector),
            stock_list: Arc::new(RwLock::new(Vec::new())),
            on_quotes_collected: Arc::new(RwLock::new(None)),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// 设置股票列表
    pub async fn set_stock_list(&self, stocks: Vec<QuoteStockInfo>) {
        let mut list = self.stock_list.write().await;
        *list = stocks;
        info!("股票列表已更新：{} 只股票", list.len());
    }

    /// 设置采集回调
    pub async fn set_callback<F>(&self, callback: F)
    where
        F: Fn(Vec<StockQuote>) + Send + Sync + 'static,
    {
        let mut cb = self.on_quotes_collected.write().await;
        *cb = Some(Arc::new(callback));
    }

    /// 检查当前状态和下次检查时间
    pub async fn check_status(&self) -> Result<(SchedulerState, DateTime<Utc>, StdDuration)> {
        // 强制模式：始终返回 Active 状态
        if self.config.force_mode {
            info!("FORCE_MODE 启用 - 调度器始终活跃");
            let next_check = Utc::now() + Duration::seconds(60);
            return Ok((
                SchedulerState::Active,
                next_check,
                StdDuration::from_secs(60),
            ));
        }

        let status = self.calendar.get_current_status().await;
        let now = Utc::now();

        debug!(
            "当前交易状态: is_trading_day={}, session={:?}",
            status.is_trading_day, status.current_session
        );

        // 如果不是交易日，返回 Inactive 状态
        if !status.is_trading_day {
            info!("非交易日 - 调度器休眠");
            let next_check = now + Duration::seconds(self.config.inactive_check_interval as i64);
            return Ok((
                SchedulerState::Inactive,
                next_check,
                StdDuration::from_secs(self.config.inactive_check_interval),
            ));
        }

        // 根据交易时段确定状态
        let (state, interval) = match status.current_session {
            TradingSession::Morning => {
                info!("上午交易时段 - 调度器活跃");
                (SchedulerState::Active, self.config.active_check_interval)
            }
            TradingSession::Afternoon => {
                info!("下午交易时段 - 调度器活跃");
                (SchedulerState::Active, self.config.active_check_interval)
            }
            TradingSession::Auction => {
                info!("竞价时段 - 调度器活跃");
                (SchedulerState::Active, self.config.auction_check_interval)
            }
            TradingSession::Closed => {
                // 判断是否在盘前或盘后时段
                let state = self.determine_market_state(&now);
                let interval = match state {
                    SchedulerState::PreMarket => 300,
                    SchedulerState::PostMarket => 300,
                    SchedulerState::Inactive => self.config.inactive_check_interval,
                    _ => self.config.inactive_check_interval,
                };
                (state, interval)
            }
        };

        let next_check = now + Duration::seconds(interval as i64);
        Ok((state, next_check, StdDuration::from_secs(interval)))
    }

    /// 判断市场状态（盘前/盘后/非活跃）
    fn determine_market_state(&self, now: &DateTime<Utc>) -> SchedulerState {
        // 转换为北京时间（UTC+8）
        let beijing_time = *now + Duration::hours(8);
        let hour = beijing_time.hour() as u64;
        let minute = beijing_time.minute() as u64;
        let time_in_minutes = hour * 60 + minute;

        // 早上 9:00-9:30 为盘前
        let pre_market_start = 9 * 60; // 9:00
        let pre_market_end = 9 * 60 + 30; // 9:30

        // 下午 15:00-15:30 为盘后
        let post_market_start = 15 * 60; // 15:00
        let post_market_end = 15 * 60 + 30; // 15:30

        if time_in_minutes >= pre_market_start && time_in_minutes < pre_market_end {
            info!("盘前时段检测");
            SchedulerState::PreMarket
        } else if time_in_minutes >= post_market_start && time_in_minutes < post_market_end {
            info!("盘后时段检测");
            SchedulerState::PostMarket
        } else if time_in_minutes < post_market_end {
            // 盘后之前都认为是非活跃状态
            info!("非交易时段 - 调度器非活跃");
            SchedulerState::Inactive
        } else {
            // 盘后之后
            info!("盘后时段结束 - 调度器非活跃");
            SchedulerState::Inactive
        }
    }

    /// 执行一次采集
    pub async fn collect_once(&self) -> Result<Vec<StockQuote>> {
        let stocks = self.stock_list.read().await;

        if stocks.is_empty() {
            warn!("股票列表为空，跳过采集");
            return Ok(Vec::new());
        }

        info!("开始采集 {} 只股票的实时行情", stocks.len());

        let quotes = self.collector.collect_all(&stocks).await?;

        info!("采集完成：获取 {} 只股票的行情数据", quotes.len());

        // 调用回调
        let cb = self.on_quotes_collected.read().await;
        if let Some(ref callback) = *cb {
            callback(quotes.clone());
        }

        Ok(quotes)
    }

    /// 启动调度器（阻塞运行）
    pub async fn run(&self) -> Result<()> {
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        info!("智能采集调度器启动");

        loop {
            // 检查是否应该停止
            {
                let running = self.running.read().await;
                if !*running {
                    info!("调度器已停止");
                    break;
                }
            }

            // 检查状态
            let (state, _next_check, interval) = self.check_status().await?;

            match state {
                SchedulerState::Active => {
                    // 执行采集
                    if let Err(e) = self.collect_once().await {
                        warn!("采集失败: {}", e);
                    }
                }
                SchedulerState::PreMarket => {
                    debug!("盘前时段，等待开盘...");
                }
                SchedulerState::PostMarket => {
                    debug!("盘后时段，等待收盘...");
                }
                SchedulerState::Inactive => {
                    debug!("非交易时段，调度器休眠");
                }
            }

            // 等待下次检查
            tokio::time::sleep(interval).await;
        }

        Ok(())
    }

    /// 停止调度器
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("调度器停止信号已发送");
    }

    /// 获取建议的采集间隔（秒）
    pub async fn get_recommended_interval(&self) -> u64 {
        self.calendar.get_recommended_interval().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::tdx::TdxSource;

    #[tokio::test]
    async fn test_scheduler_config_default() {
        let config = SchedulerConfig::default();
        assert_eq!(config.active_check_interval, 60);
        assert_eq!(config.auction_check_interval, 30);
    }

    #[test]
    fn test_scheduler_state_display() {
        assert_eq!(SchedulerState::Active.display_name(), "活跃");
        assert_eq!(SchedulerState::Inactive.display_name(), "非活跃");
    }

    #[tokio::test]
    async fn test_scheduler_creation() {
        let tdx_source = TdxSource::new(1, vec![], 7709, 10).unwrap();
        let collector = QuoteCollector::new(tdx_source, 800, 10);
        let scheduler = CollectScheduler::new(collector).await;
        assert!(scheduler.is_ok());
    }
}
