//! Data types for the real-time performance monitor: configuration, equity
//! points, computed metrics, and the drawdown-level enum.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// 性能监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitorConfig {
    /// 权益历史保留数量
    pub max_equity_history: usize,
    /// 启用回撤监控
    pub enable_drawdown_monitoring: bool,
    /// 启用收益率监控
    pub enable_return_monitoring: bool,
    /// 回撤告警阈值（0-1）
    pub drawdown_alert_threshold: Decimal,
    /// 启用夏普比率实时计算
    pub enable_sharpe_ratio: bool,
    /// 无风险利率（年化）
    pub risk_free_rate: Decimal,
}

impl Default for PerformanceMonitorConfig {
    fn default() -> Self {
        Self {
            max_equity_history: 1000,
            enable_drawdown_monitoring: true,
            enable_return_monitoring: true,
            drawdown_alert_threshold: dec!(0.1), // 10%
            enable_sharpe_ratio: true,
            risk_free_rate: dec!(0.03), // 3%
        }
    }
}

/// 权益点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 权益值
    pub equity: Decimal,
    /// 可用资金
    pub available_cash: Decimal,
    /// 持仓市值
    pub position_value: Decimal,
    /// 当日收益率
    pub daily_return: Decimal,
}

/// 实时性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeMetrics {
    /// 总收益率
    pub total_return: Decimal,
    /// 年化收益率
    pub annual_return: Decimal,
    /// 当前回撤
    pub current_drawdown: Decimal,
    /// 最大回撤
    pub max_drawdown: Decimal,
    /// 夏普比率（实时估算）
    pub sharpe_ratio: Decimal,
    /// 索提诺比率（实时估算）
    pub sortino_ratio: Decimal,
    /// 胜率
    pub win_rate: Decimal,
    /// 盈亏比
    pub profit_loss_ratio: Decimal,
    /// 当前权益
    pub current_equity: Decimal,
    /// 最高权益
    pub peak_equity: Decimal,
    /// 权益偏离高点比例
    pub equity_from_peak_pct: Decimal,
    /// 总交易次数
    pub total_trades: usize,
    /// 盈利交易次数
    pub win_trades: usize,
    /// 亏损交易次数
    pub loss_trades: usize,
    /// 最近更新时间
    pub last_updated: DateTime<Utc>,
}

impl Default for RealtimeMetrics {
    fn default() -> Self {
        Self {
            total_return: Decimal::ZERO,
            annual_return: Decimal::ZERO,
            current_drawdown: Decimal::ZERO,
            max_drawdown: Decimal::ZERO,
            sharpe_ratio: Decimal::ZERO,
            sortino_ratio: Decimal::ZERO,
            win_rate: Decimal::ZERO,
            profit_loss_ratio: Decimal::ZERO,
            current_equity: Decimal::ZERO,
            peak_equity: Decimal::ZERO,
            equity_from_peak_pct: Decimal::ZERO,
            total_trades: 0,
            win_trades: 0,
            loss_trades: 0,
            last_updated: Utc::now(),
        }
    }
}

/// 回撤状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawdownStatus {
    Normal,
    Caution,
    Warning,
    Critical,
}
