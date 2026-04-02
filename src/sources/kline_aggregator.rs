use crate::sources::tdx::StockQuote;
use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::time::{Duration, interval};
use tracing::info;

/// K线周期
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KlinePeriod {
    /// 1分钟
    OneMinute,
    /// 5分钟
    FiveMinutes,
    /// 15分钟
    FifteenMinutes,
    /// 30分钟
    ThirtyMinutes,
    /// 60分钟
    OneHour,
    /// 日线
    Daily,
}

impl KlinePeriod {
    /// 转换为字符串标识符
    pub fn as_str(&self) -> &'static str {
        match self {
            KlinePeriod::OneMinute => "1m",
            KlinePeriod::FiveMinutes => "5m",
            KlinePeriod::FifteenMinutes => "15m",
            KlinePeriod::ThirtyMinutes => "30m",
            KlinePeriod::OneHour => "60m",
            KlinePeriod::Daily => "1d",
        }
    }

    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "1m" => Some(KlinePeriod::OneMinute),
            "5m" => Some(KlinePeriod::FiveMinutes),
            "15m" => Some(KlinePeriod::FifteenMinutes),
            "30m" => Some(KlinePeriod::ThirtyMinutes),
            "60m" => Some(KlinePeriod::OneHour),
            "1d" => Some(KlinePeriod::Daily),
            _ => None,
        }
    }

    /// 获取周期分钟数
    pub fn minutes(&self) -> u64 {
        match self {
            KlinePeriod::OneMinute => 1,
            KlinePeriod::FiveMinutes => 5,
            KlinePeriod::FifteenMinutes => 15,
            KlinePeriod::ThirtyMinutes => 30,
            KlinePeriod::OneHour => 60,
            KlinePeriod::Daily => 240, // 4小时交易时间
        }
    }
}

/// K线数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineData {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 股票代码
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 周期
    pub period: KlinePeriod,
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 收盘价
    pub close: f64,
    /// 成交量（手）
    pub volume: f64,
    /// 成交额（元）
    pub amount: f64,
    /// 成交笔数
    pub trade_count: u32,
    /// 数据源
    pub source: String,
}

/// K线聚合窗口（内存中状态）
#[derive(Debug, Clone)]
pub struct KlineWindow {
    /// 股票代码
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 周期
    pub period: KlinePeriod,
    /// 开盘价
    pub open: Option<f64>,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 收盘价
    pub close: f64,
    /// 成交量（手）
    pub volume: f64,
    /// 成交额（元）
    pub amount: f64,
    /// 成交笔数
    pub trade_count: u32,
    /// 窗口开始时间
    pub start_time: DateTime<Utc>,
    /// 最后更新时间
    pub last_update: DateTime<Utc>,
}

impl KlineWindow {
    /// 创建新窗口
    pub fn new(code: String, name: String, period: KlinePeriod, start_time: DateTime<Utc>) -> Self {
        Self {
            code,
            name,
            period,
            open: None,
            high: f64::MIN,
            low: f64::MAX,
            close: 0.0,
            volume: 0.0,
            amount: 0.0,
            trade_count: 0,
            start_time,
            last_update: start_time,
        }
    }

    /// 更新窗口数据（从实时行情）
    pub fn update(&mut self, quote: &StockQuote) {
        // 首笔价格作为开盘价
        if self.open.is_none() {
            self.open = Some(quote.price);
        }

        // 更新最高价和最低价
        self.high = self.high.max(quote.price);
        self.low = self.low.min(quote.price);

        // 更新收盘价（最新价格）
        self.close = quote.price;

        // 累加成交量和成交额
        self.volume += quote.volume;
        self.amount += quote.amount;
        self.trade_count += 1;

        // 更新最后更新时间（从 u64 timestamp 转换为 DateTime<Utc>）
        self.last_update =
            super::kline_aggregator_support::quote_timestamp_or(quote.timestamp, Utc::now());
    }

    /// 判断窗口是否应该关闭（时间窗口结束）
    pub fn should_close(&self, current_time: DateTime<Utc>) -> bool {
        let elapsed = current_time
            .signed_duration_since(self.start_time)
            .num_seconds()
            .abs() as u64;
        elapsed >= self.period.minutes() * 60
    }

    /// 转换为KlineData
    pub fn to_kline_data(&self, source: &str) -> Option<KlineData> {
        let open = self.open?;
        let (high, low) =
            super::kline_aggregator_support::resolved_high_low(open, self.high, self.low);
        Some(KlineData {
            timestamp: self.start_time,
            code: self.code.clone(),
            name: self.name.clone(),
            period: self.period,
            open,
            high,
            low,
            close: self.close,
            volume: self.volume,
            amount: self.amount,
            trade_count: self.trade_count,
            source: source.to_string(),
        })
    }
}

/// K线聚合器
pub struct KlineAggregator {
    /// 窗口映射：key = "code:period:date"
    windows: Arc<Mutex<HashMap<String, KlineWindow>>>,
    /// 完成的 K线数据发送器
    kline_sender: mpsc::UnboundedSender<KlineData>,
}

impl KlineAggregator {
    /// 创建新的聚合器
    pub fn new(buffer_size: usize) -> (Self, mpsc::UnboundedReceiver<KlineData>) {
        info!("初始化K线实时聚合器，buffer_size={}", buffer_size);

        let (kline_sender, kline_receiver) = mpsc::unbounded_channel();

        let aggregator = Self {
            windows: Arc::new(Mutex::new(HashMap::new())),
            kline_sender,
        };

        // 启动过期窗口清理任务
        let windows_clone = Arc::clone(&aggregator.windows);
        tokio::spawn(async move {
            Self::cleanup_expired_windows_task(windows_clone).await;
        });

        (aggregator, kline_receiver)
    }

    /// 生成窗口Key
    fn make_window_key(code: &str, period: KlinePeriod, time: &DateTime<Utc>) -> String {
        super::kline_aggregator_support::make_window_key(code, period, time)
    }

    /// 计算窗口开始时间
    fn calculate_window_start(time: DateTime<Utc>, period: KlinePeriod) -> DateTime<Utc> {
        super::kline_aggregator_support::calculate_window_start(time, period)
    }

    /// 处理单条行情数据
    pub async fn process_quote(&self, quote: &StockQuote) -> Vec<KlineData> {
        let mut completed_klines = Vec::new();

        // 处理1分钟窗口
        if let Some(kline) = self.update_window(quote, KlinePeriod::OneMinute).await {
            completed_klines.push(kline);
        }

        // 处理5分钟窗口
        if let Some(kline) = self.update_window(quote, KlinePeriod::FiveMinutes).await {
            completed_klines.push(kline);
        }

        // 处理30分钟窗口
        if let Some(kline) = self.update_window(quote, KlinePeriod::ThirtyMinutes).await {
            completed_klines.push(kline);
        }

        completed_klines
    }

    /// 更新或创建窗口
    async fn update_window(&self, quote: &StockQuote, period: KlinePeriod) -> Option<KlineData> {
        // 从 u64 timestamp 转换为 DateTime<Utc>
        let current_time =
            super::kline_aggregator_support::quote_timestamp_or(quote.timestamp, Utc::now());
        let window_key = Self::make_window_key(&quote.code, period, &current_time);

        let mut windows = self.windows.lock().await;

        // 检查是否已存在窗口
        if let Some(window) = windows.get_mut(&window_key) {
            // 更新现有窗口
            window.update(quote);

            // 检查窗口是否应该关闭
            if window.should_close(current_time) {
                let window = windows.remove(&window_key)?;
                return window.to_kline_data("realtime");
            }
        } else {
            // 创建新窗口
            let window_start = Self::calculate_window_start(current_time, period);
            let mut window =
                KlineWindow::new(quote.code.clone(), quote.name.clone(), period, window_start);
            window.update(quote);
            windows.insert(window_key, window);
        }

        None
    }

    /// 清理过期窗口任务（定期运行）
    async fn cleanup_expired_windows_task(windows: Arc<Mutex<HashMap<String, KlineWindow>>>) {
        let mut cleanup_interval = interval(Duration::from_secs(300)); // 每5分钟清理一次
        cleanup_interval.tick().await; // 跳过第一次立即触发

        loop {
            cleanup_interval.tick().await;
            Self::cleanup_expired_windows_inner(Arc::clone(&windows)).await;
        }
    }

    /// 清理过期窗口（内部实现）
    async fn cleanup_expired_windows_inner(windows: Arc<Mutex<HashMap<String, KlineWindow>>>) {
        let current_time = Utc::now();
        let mut windows = windows.lock().await;
        let initial_count = windows.len();

        // 清理超过2小时未更新的窗口
        windows.retain(|_key, window| {
            super::kline_aggregator_support::should_retain_window(
                current_time,
                window.last_update,
            )
        });

        let removed_count = initial_count - windows.len();
        if removed_count > 0 {
            info!(
                "清理过期窗口：移除 {} 个窗口，剩余 {} 个窗口",
                removed_count,
                windows.len()
            );
        }
    }

    /// 清理过期窗口（供外部调用）
    pub async fn cleanup_expired_windows(&self) {
        Self::cleanup_expired_windows_inner(Arc::clone(&self.windows)).await;
    }

    /// 获取当前窗口数量（用于监控）
    pub async fn window_count(&self) -> usize {
        self.windows.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kline_period_as_str() {
        assert_eq!(KlinePeriod::OneMinute.as_str(), "1m");
        assert_eq!(KlinePeriod::FiveMinutes.as_str(), "5m");
        assert_eq!(KlinePeriod::Daily.as_str(), "1d");
    }

    #[test]
    fn test_kline_period_minutes() {
        assert_eq!(KlinePeriod::OneMinute.minutes(), 1);
        assert_eq!(KlinePeriod::FiveMinutes.minutes(), 5);
        assert_eq!(KlinePeriod::OneHour.minutes(), 60);
    }

    #[test]
    fn test_calculate_window_start_1m() {
        let time = DateTime::parse_from_rfc3339("2026-01-02T10:30:45Z")
            .unwrap()
            .with_timezone(&Utc);
        let start = KlineAggregator::calculate_window_start(time, KlinePeriod::OneMinute);
        assert_eq!(start.second(), 0);
        assert_eq!(start.nanosecond(), 0);
    }

    #[test]
    fn test_calculate_window_start_5m() {
        let time = DateTime::parse_from_rfc3339("2026-01-02T10:33:45Z")
            .unwrap()
            .with_timezone(&Utc);
        let start = KlineAggregator::calculate_window_start(time, KlinePeriod::FiveMinutes);
        assert_eq!(start.minute(), 30); // 应该对齐到10:30
        assert_eq!(start.second(), 0);
    }
}
