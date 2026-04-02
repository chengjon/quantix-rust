/// 竞价数据采集器
///
/// 从短线侠项目迁移，采集集合竞价时段（9:15-9:25）的股票数据
use crate::core::Result;
use crate::core::trading_calendar::TradingCalendar;
use chrono::{Local, Timelike};
use rustdx_complete::tcp::stock::SecurityQuotes;
use rustdx_complete::tcp::{Tcp, Tdx};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

/// 竞价数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionQuote {
    /// 股票代码
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 采集时间
    pub time: String,
    /// 当前价
    pub price: f64,
    /// 昨收价
    pub pre_close: f64,
    /// 成交量（手）
    pub volume: u64,
    /// 成交额（元）
    pub amount: f64,
    /// 买一价
    pub buy1_price: f64,
    /// 买一量（手）
    pub buy1_volume: u64,
    /// 卖一价
    pub sell1_price: f64,
    /// 卖一量（手）
    pub sell1_volume: u64,
    /// 涨跌幅(%)
    pub change_percent: f64,
    /// 买封金额（元）
    pub sealed_amount_buy: f64,
    /// 卖封金额（元）
    pub sealed_amount_sell: f64,
    /// 抢筹强度评分 (0-100)
    pub strength_score: f32,
}

/// 自选股列表
#[derive(Debug, Clone)]
pub struct WatchlistStock {
    pub code: String,
    pub name: String,
    pub market: u16, // 0=深圳, 1=上海
}

/// 竞价数据采集器
pub struct AuctionCollector {
    /// TDX TCP 连接
    tcp: Tcp,
    /// 自选股列表
    watchlist: Vec<WatchlistStock>,
    /// 交易日历
    calendar: TradingCalendar,
}

impl AuctionCollector {
    /// 创建新的竞价采集器
    pub async fn new() -> Result<Self> {
        let tcp = Tcp::new()
            .map_err(|e| crate::core::QuantixError::DataSource(format!("TDX 连接失败: {}", e)))?;

        let calendar = TradingCalendar::new().await?;

        // 默认自选股列表（可从配置文件加载）
        let watchlist = Self::default_watchlist();

        info!("竞价采集器初始化成功，自选股数量: {}", watchlist.len());

        Ok(Self {
            tcp,
            watchlist,
            calendar,
        })
    }

    /// 使用自定义自选股列表创建
    pub async fn with_watchlist(watchlist: Vec<WatchlistStock>) -> Result<Self> {
        let tcp = Tcp::new()
            .map_err(|e| crate::core::QuantixError::DataSource(format!("TDX 连接失败: {}", e)))?;

        let calendar = TradingCalendar::new().await?;

        info!("竞价采集器初始化成功，自选股数量: {}", watchlist.len());

        Ok(Self {
            tcp,
            watchlist,
            calendar,
        })
    }

    /// 默认自选股列表
    fn default_watchlist() -> Vec<WatchlistStock> {
        vec![
            WatchlistStock {
                code: "000001".to_string(),
                name: "平安银行".to_string(),
                market: 0,
            },
            WatchlistStock {
                code: "000002".to_string(),
                name: "万科A".to_string(),
                market: 0,
            },
            WatchlistStock {
                code: "600000".to_string(),
                name: "浦发银行".to_string(),
                market: 1,
            },
            WatchlistStock {
                code: "600036".to_string(),
                name: "招商银行".to_string(),
                market: 1,
            },
            WatchlistStock {
                code: "600519".to_string(),
                name: "贵州茅台".to_string(),
                market: 1,
            },
        ]
    }

    /// 设置自选股列表
    pub fn set_watchlist(&mut self, watchlist: Vec<WatchlistStock>) {
        self.watchlist = watchlist;
        info!("自选股列表已更新，数量: {}", self.watchlist.len());
    }

    /// 检查当前是否在竞价时段（9:15-9:25）
    pub async fn is_auction_time(&self) -> bool {
        // 检查是否为交易日
        let now = Local::now();
        let date = now.date_naive();

        if !self.calendar.is_trading_day(date).await {
            return false;
        }

        // 检查是否在竞价时段
        let hour = now.hour();
        let minute = now.minute();

        // 竞价时段：9:15-9:25
        super::auction_collector_support::is_within_auction_window(hour, minute)
    }

    /// 计算抢筹强度评分（0-100分）
    ///
    /// 评分算法：
    /// - 涨幅权重 40%
    /// - 买盘占比权重 30%
    /// - 成交量权重 30%
    /// 采集单只股票的竞价数据
    pub fn fetch_auction_quote(&mut self, stock: &WatchlistStock) -> Result<AuctionQuote> {
        let mut quotes = SecurityQuotes::new(vec![(stock.market, &stock.code)]);

        quotes.recv_parsed(&mut self.tcp).map_err(|e| {
            crate::core::QuantixError::DataSource(format!("获取竞价数据失败: {}", e))
        })?;

        if let Some(quote) = quotes.result().first() {
            Ok(super::auction_collector_support::build_auction_quote(
                stock,
                Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                quote.price,
                quote.preclose,
                quote.vol as u64,
                quote.amount,
                quote.bid1,
                quote.bid1_vol as u64,
                quote.ask1,
                quote.ask1_vol as u64,
            ))
        } else {
            Err(crate::core::QuantixError::DataSource(format!(
                "获取竞价数据失败: {}",
                stock.code
            )))
        }
    }

    /// 采集所有自选股的竞价数据
    pub async fn collect_all(&mut self) -> Result<Vec<AuctionQuote>> {
        let watchlist = self.watchlist.clone();
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failed_codes = Vec::new();

        for stock in &watchlist {
            match self.fetch_auction_quote(stock) {
                Ok(quote) => {
                    debug!(
                        "竞价数据: {} {} 价格:{:.2} 涨跌:{:.2}% 买封:{:.0}元 评分:{:.0}",
                        quote.code,
                        quote.name,
                        quote.price,
                        quote.change_percent,
                        quote.sealed_amount_buy,
                        quote.strength_score
                    );
                    results.push(quote);
                    success_count += 1;
                }
                Err(e) => {
                    warn!("采集失败 [{}]: {}", stock.code, e);
                    failed_codes.push(stock.code.clone());
                }
            }
        }

        info!(
            "竞价采集完成: 成功 {} 失败 {}",
            success_count,
            failed_codes.len()
        );

        Ok(results)
    }

    /// 运行竞价采集循环
    pub async fn run(&mut self) -> Result<()> {
        info!("竞价采集服务启动");

        loop {
            // 时序检查：只在竞价时段运行
            if !self.is_auction_time().await {
                debug!("非竞价时段，等待中...");
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }

            match self.collect_all().await {
                Ok(quotes) => {
                    info!("竞价采集完成，获取 {} 条数据", quotes.len());
                }
                Err(e) => {
                    warn!("竞价采集失败: {}", e);
                }
            }

            // 每 1 秒采集一次
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sealed_amount() {
        let (buy, sell) =
            super::super::auction_collector_support::calculate_sealed_amount(10.0, 1000, 10.5, 500);
        assert_eq!(buy, 10000.0);
        assert_eq!(sell, 5250.0);
    }

    #[tokio::test]
    async fn test_auction_collector_creation() {
        let collector = AuctionCollector::new().await;
        assert!(collector.is_ok());
    }
}
