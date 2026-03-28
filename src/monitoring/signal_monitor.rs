/// 信号监控模块
///
/// 实时追踪策略交易信号
use chrono::{DateTime, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tokio::time::Instant;

use crate::strategy::trait_def::Signal;

/// 信号监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMonitorConfig {
    /// 历史信号保留数量
    pub max_history_size: usize,
    /// 统计窗口大小（秒）
    pub stats_window_secs: u64,
    /// 启用信号计数统计
    pub enable_count_stats: bool,
    /// 启用信号频率统计
    pub enable_frequency_stats: bool,
    /// 启用信号序列记录
    pub enable_sequence_tracking: bool,
}

impl Default for SignalMonitorConfig {
    fn default() -> Self {
        Self {
            max_history_size: 1000,
            stats_window_secs: 3600, // 1小时
            enable_count_stats: true,
            enable_frequency_stats: true,
            enable_sequence_tracking: true,
        }
    }
}

/// 信号事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEvent {
    /// 策略名称
    pub strategy_name: String,
    /// 股票代码
    pub code: String,
    /// 信号类型
    pub signal: Signal,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 价格
    pub price: Decimal,
    /// K线时间
    pub bar_time: NaiveDateTime,
    /// 额外信息
    pub metadata: HashMap<String, String>,
}

impl SignalEvent {
    /// 创建新的信号事件
    pub fn new(
        strategy_name: String,
        code: String,
        signal: Signal,
        price: Decimal,
        bar_time: NaiveDateTime,
    ) -> Self {
        Self {
            strategy_name,
            code,
            signal,
            timestamp: Utc::now(),
            price,
            bar_time,
            metadata: HashMap::new(),
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// 信号统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalStats {
    /// 买入信号次数
    pub buy_count: usize,
    /// 卖出信号次数
    pub sell_count: usize,
    /// 观望信号次数
    pub hold_count: usize,
    /// 总信号次数
    pub total_count: usize,
    /// 买入信号频率（每分钟）
    pub buy_frequency: f64,
    /// 卖出信号频率（每分钟）
    pub sell_frequency: f64,
    /// 最后信号时间
    pub last_signal_time: Option<DateTime<Utc>>,
    /// 最后N个信号序列
    pub recent_signals: Vec<Signal>,
}

impl Default for SignalStats {
    fn default() -> Self {
        Self {
            buy_count: 0,
            sell_count: 0,
            hold_count: 0,
            total_count: 0,
            buy_frequency: 0.0,
            sell_frequency: 0.0,
            last_signal_time: None,
            recent_signals: Vec::new(),
        }
    }
}

/// 信号监控器
pub struct SignalMonitor {
    /// 配置
    config: SignalMonitorConfig,
    /// 信号历史记录
    signal_history: VecDeque<SignalEvent>,
    /// 按策略分组的统计
    strategy_stats: HashMap<String, SignalStats>,
    /// 按股票代码分组的统计
    code_stats: HashMap<String, SignalStats>,
    /// 窗口开始时间
    window_start: Instant,
}

impl SignalMonitor {
    /// 创建新的信号监控器
    pub fn new(config: SignalMonitorConfig) -> Self {
        Self {
            config,
            signal_history: VecDeque::with_capacity(1000),
            strategy_stats: HashMap::new(),
            code_stats: HashMap::new(),
            window_start: Instant::now(),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(SignalMonitorConfig::default())
    }

    /// 记录信号
    pub fn record_signal(&mut self, event: SignalEvent) {
        // 添加到历史记录
        self.signal_history.push_back(event.clone());

        // 限制历史记录大小
        if self.signal_history.len() > self.config.max_history_size {
            self.signal_history.pop_front();
        }

        // 更新策略统计
        self.update_strategy_stats(&event);

        // 更新股票统计
        self.update_code_stats(&event);

        // 定期重置窗口统计
        if self.window_start.elapsed() >= Duration::from_secs(self.config.stats_window_secs) {
            self.reset_window_stats();
        }
    }

    /// 更新策略统计
    fn update_strategy_stats(&mut self, event: &SignalEvent) {
        let signal = event.signal;
        let timestamp = event.timestamp;
        let enable_sequence_tracking = self.config.enable_sequence_tracking;
        let elapsed_min = if self.config.enable_frequency_stats {
            Some(self.window_start.elapsed().as_secs_f64() / 60.0)
        } else {
            None
        };

        let stats = self
            .strategy_stats
            .entry(event.strategy_name.clone())
            .or_insert_with(SignalStats::default);

        // Update counts
        match signal {
            Signal::Buy => {
                stats.buy_count += 1;
            }
            Signal::Sell => {
                stats.sell_count += 1;
            }
            Signal::Hold => {
                stats.hold_count += 1;
            }
        }

        stats.total_count += 1;
        stats.last_signal_time = Some(timestamp);

        if enable_sequence_tracking {
            stats.recent_signals.push(signal);
            if stats.recent_signals.len() > 10 {
                stats.recent_signals.remove(0);
            }
        }

        if let Some(elapsed) = elapsed_min {
            if elapsed > 0.0 {
                stats.buy_frequency = stats.buy_count as f64 / elapsed;
                stats.sell_frequency = stats.sell_count as f64 / elapsed;
            }
        }
    }

    /// 更新股票统计
    fn update_code_stats(&mut self, event: &SignalEvent) {
        let signal = event.signal;
        let timestamp = event.timestamp;
        let enable_sequence_tracking = self.config.enable_sequence_tracking;
        let elapsed_min = if self.config.enable_frequency_stats {
            Some(self.window_start.elapsed().as_secs_f64() / 60.0)
        } else {
            None
        };

        let stats = self
            .code_stats
            .entry(event.code.clone())
            .or_insert_with(SignalStats::default);

        // Update counts
        match signal {
            Signal::Buy => {
                stats.buy_count += 1;
            }
            Signal::Sell => {
                stats.sell_count += 1;
            }
            Signal::Hold => {
                stats.hold_count += 1;
            }
        }

        stats.total_count += 1;
        stats.last_signal_time = Some(timestamp);

        if enable_sequence_tracking {
            stats.recent_signals.push(signal);
            if stats.recent_signals.len() > 10 {
                stats.recent_signals.remove(0);
            }
        }

        if let Some(elapsed) = elapsed_min {
            if elapsed > 0.0 {
                stats.buy_frequency = stats.buy_count as f64 / elapsed;
                stats.sell_frequency = stats.sell_count as f64 / elapsed;
            }
        }
    }

    /// 重置窗口统计
    fn reset_window_stats(&mut self) {
        self.window_start = Instant::now();
        for stats in self.strategy_stats.values_mut() {
            stats.buy_count = 0;
            stats.sell_count = 0;
            stats.hold_count = 0;
            stats.total_count = 0;
            stats.buy_frequency = 0.0;
            stats.sell_frequency = 0.0;
        }
        for stats in self.code_stats.values_mut() {
            stats.buy_count = 0;
            stats.sell_count = 0;
            stats.hold_count = 0;
            stats.total_count = 0;
            stats.buy_frequency = 0.0;
            stats.sell_frequency = 0.0;
        }
    }

    /// 获取策略统计
    pub fn get_strategy_stats(&self, strategy_name: &str) -> Option<&SignalStats> {
        self.strategy_stats.get(strategy_name)
    }

    /// 获取股票统计
    pub fn get_code_stats(&self, code: &str) -> Option<&SignalStats> {
        self.code_stats.get(code)
    }

    /// 获取所有策略统计
    pub fn get_all_strategy_stats(&self) -> &HashMap<String, SignalStats> {
        &self.strategy_stats
    }

    /// 获取所有股票统计
    pub fn get_all_code_stats(&self) -> &HashMap<String, SignalStats> {
        &self.code_stats
    }

    /// 获取最近的信号
    pub fn get_recent_signals(&self, count: usize) -> Vec<&SignalEvent> {
        self.signal_history.iter().rev().take(count).collect()
    }

    /// 获取指定策略的最近信号
    pub fn get_strategy_recent_signals(
        &self,
        strategy_name: &str,
        count: usize,
    ) -> Vec<&SignalEvent> {
        self.signal_history
            .iter()
            .rev()
            .filter(|e| e.strategy_name == strategy_name)
            .take(count)
            .collect()
    }

    /// 获取指定股票的最近信号
    pub fn get_code_recent_signals(&self, code: &str, count: usize) -> Vec<&SignalEvent> {
        self.signal_history
            .iter()
            .rev()
            .filter(|e| e.code == code)
            .take(count)
            .collect()
    }

    /// 获取信号历史大小
    pub fn history_size(&self) -> usize {
        self.signal_history.len()
    }

    /// 清空历史记录
    pub fn clear_history(&mut self) {
        self.signal_history.clear();
    }

    /// 重置所有统计
    pub fn reset_stats(&mut self) {
        self.strategy_stats.clear();
        self.code_stats.clear();
        self.window_start = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use rust_decimal_macros::dec;

    fn create_test_signal(strategy: &str, code: &str, signal: Signal, price: f64) -> SignalEvent {
        SignalEvent::new(
            strategy.to_string(),
            code.to_string(),
            signal,
            Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO),
            NaiveDateTime::from_timestamp_opt(1640995200, 0).unwrap(),
        )
    }

    #[test]
    fn test_signal_monitor_creation() {
        let monitor = SignalMonitor::with_defaults();
        assert_eq!(monitor.history_size(), 0);
    }

    #[test]
    fn test_record_signal() {
        let mut monitor = SignalMonitor::with_defaults();

        let event = create_test_signal("MA_Cross", "000001", Signal::Buy, 100.0);
        monitor.record_signal(event);

        assert_eq!(monitor.history_size(), 1);

        let stats = monitor.get_strategy_stats("MA_Cross").unwrap();
        assert_eq!(stats.buy_count, 1);
        assert_eq!(stats.total_count, 1);
    }

    #[test]
    fn test_multiple_signals() {
        let mut monitor = SignalMonitor::with_defaults();

        monitor.record_signal(create_test_signal("MA_Cross", "000001", Signal::Buy, 100.0));
        monitor.record_signal(create_test_signal(
            "MA_Cross",
            "000001",
            Signal::Sell,
            105.0,
        ));
        monitor.record_signal(create_test_signal("MA_Cross", "000001", Signal::Buy, 98.0));

        let stats = monitor.get_strategy_stats("MA_Cross").unwrap();
        assert_eq!(stats.buy_count, 2);
        assert_eq!(stats.sell_count, 1);
        assert_eq!(stats.total_count, 3);
    }

    #[test]
    fn test_history_limit() {
        let config = SignalMonitorConfig {
            max_history_size: 5,
            ..Default::default()
        };
        let mut monitor = SignalMonitor::new(config);

        for i in 0..10 {
            monitor.record_signal(create_test_signal(
                "MA_Cross",
                "000001",
                Signal::Buy,
                100.0 + i as f64,
            ));
        }

        assert_eq!(monitor.history_size(), 5);
    }

    #[test]
    fn test_get_recent_signals() {
        let mut monitor = SignalMonitor::with_defaults();

        monitor.record_signal(create_test_signal("MA_Cross", "000001", Signal::Buy, 100.0));
        monitor.record_signal(create_test_signal("MA_Cross", "000002", Signal::Buy, 101.0));
        monitor.record_signal(create_test_signal("MA_Cross", "000003", Signal::Buy, 102.0));

        let recent = monitor.get_recent_signals(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].code, "000003");
        assert_eq!(recent[1].code, "000002");
    }

    #[test]
    fn test_code_stats() {
        let mut monitor = SignalMonitor::with_defaults();

        monitor.record_signal(create_test_signal("MA_Cross", "000001", Signal::Buy, 100.0));
        monitor.record_signal(create_test_signal(
            "MA_Cross",
            "000001",
            Signal::Sell,
            105.0,
        ));
        monitor.record_signal(create_test_signal("MA_Cross", "000002", Signal::Buy, 50.0));

        let stats1 = monitor.get_code_stats("000001").unwrap();
        assert_eq!(stats1.buy_count, 1);
        assert_eq!(stats1.sell_count, 1);

        let stats2 = monitor.get_code_stats("000002").unwrap();
        assert_eq!(stats2.buy_count, 1);
        assert_eq!(stats2.sell_count, 0);
    }

    #[test]
    fn test_clear_history() {
        let mut monitor = SignalMonitor::with_defaults();

        monitor.record_signal(create_test_signal("MA_Cross", "000001", Signal::Buy, 100.0));
        monitor.record_signal(create_test_signal("MA_Cross", "000002", Signal::Buy, 101.0));

        assert_eq!(monitor.history_size(), 2);

        monitor.clear_history();
        assert_eq!(monitor.history_size(), 0);
    }

    #[test]
    fn test_signal_event_with_metadata() {
        let event = create_test_signal("MA_Cross", "000001", Signal::Buy, 100.0)
            .with_metadata("rsi".to_string(), "30".to_string())
            .with_metadata("volume_ratio".to_string(), "2.5".to_string());

        assert_eq!(event.metadata.len(), 2);
        assert_eq!(event.metadata.get("rsi"), Some(&"30".to_string()));
    }

    #[test]
    fn test_reset_stats() {
        let mut monitor = SignalMonitor::with_defaults();

        monitor.record_signal(create_test_signal("MA_Cross", "000001", Signal::Buy, 100.0));
        monitor.record_signal(create_test_signal(
            "MA_Cross",
            "000001",
            Signal::Sell,
            105.0,
        ));

        let stats = monitor.get_strategy_stats("MA_Cross").unwrap();
        assert_eq!(stats.total_count, 2);

        monitor.reset_stats();

        let stats = monitor.get_strategy_stats("MA_Cross");
        assert!(stats.is_none());
    }
}
