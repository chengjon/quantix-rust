/// 性能监控模块
///
/// 实时计算和追踪性能指标
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

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

/// 性能监控器
pub struct PerformanceMonitor {
    /// 配置
    config: PerformanceMonitorConfig,
    /// 初始资金
    initial_capital: Decimal,
    /// 权益历史
    equity_history: VecDeque<EquityPoint>,
    /// 最高权益
    peak_equity: Decimal,
    /// 当前指标
    current_metrics: RealtimeMetrics,
    /// 交易盈亏记录（用于计算夏普比率）
    trade_returns: VecDeque<Decimal>,
    /// 开始日期
    start_date: NaiveDate,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new(config: PerformanceMonitorConfig, initial_capital: Decimal) -> Self {
        let start_date = Utc::now().naive_utc().date();

        Self {
            config,
            initial_capital,
            equity_history: VecDeque::with_capacity(1000),
            peak_equity: initial_capital,
            current_metrics: RealtimeMetrics {
                current_equity: initial_capital,
                ..Default::default()
            },
            trade_returns: VecDeque::with_capacity(1000),
            start_date,
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults(initial_capital: Decimal) -> Self {
        Self::new(PerformanceMonitorConfig::default(), initial_capital)
    }

    /// 更新权益
    pub fn update_equity(
        &mut self,
        equity: Decimal,
        available_cash: Decimal,
        position_value: Decimal,
    ) {
        let now = Utc::now();
        let daily_return = if let Some(last_point) = self.equity_history.back() {
            if last_point.equity > Decimal::ZERO {
                (equity - last_point.equity) / last_point.equity
            } else {
                Decimal::ZERO
            }
        } else {
            Decimal::ZERO
        };

        let point = EquityPoint {
            timestamp: now,
            equity,
            available_cash,
            position_value,
            daily_return,
        };

        self.equity_history.push_back(point);

        // 限制历史大小
        if self.equity_history.len() > self.config.max_equity_history {
            self.equity_history.pop_front();
        }

        // 更新最高权益
        if equity > self.peak_equity {
            self.peak_equity = equity;
        }

        // 计算实时指标
        self.calculate_metrics();
    }

    /// 记录交易盈亏
    pub fn record_trade_pnl(&mut self, pnl: Decimal) {
        self.trade_returns.push_back(pnl);

        // 限制历史大小
        if self.trade_returns.len() > self.config.max_equity_history {
            self.trade_returns.pop_front();
        }

        // 更新胜率统计
        self.current_metrics.total_trades += 1;
        if pnl > Decimal::ZERO {
            self.current_metrics.win_trades += 1;
        } else if pnl < Decimal::ZERO {
            self.current_metrics.loss_trades += 1;
        }

        // 重新计算指标
        self.calculate_metrics();
    }

    /// 计算实时指标
    fn calculate_metrics(&mut self) {
        let current_equity = self
            .equity_history
            .back()
            .map(|p| p.equity)
            .unwrap_or(self.initial_capital);

        // 总收益率
        self.current_metrics.total_return = if self.initial_capital > Decimal::ZERO {
            (current_equity - self.initial_capital) / self.initial_capital
        } else {
            Decimal::ZERO
        };

        // 年化收益率
        let days_elapsed = (Utc::now().naive_utc().date() - self.start_date).num_days();
        self.current_metrics.annual_return =
            if days_elapsed > 0 && self.current_metrics.total_return != Decimal::ZERO {
                let daily_return = self.current_metrics.total_return / Decimal::from(days_elapsed);
                daily_return * Decimal::from(365)
            } else {
                Decimal::ZERO
            };

        // 当前回撤
        self.current_metrics.current_drawdown = if self.peak_equity > Decimal::ZERO {
            (self.peak_equity - current_equity) / self.peak_equity
        } else {
            Decimal::ZERO
        };

        // 最大回撤
        self.current_metrics.max_drawdown = self
            .equity_history
            .iter()
            .map(|p| {
                if p.equity > Decimal::ZERO {
                    (self.peak_equity - p.equity) / self.peak_equity
                } else {
                    Decimal::ZERO
                }
            })
            .max()
            .unwrap_or(Decimal::ZERO);

        // 权益偏离高点比例
        self.current_metrics.equity_from_peak_pct = self.current_metrics.current_drawdown;

        // 夏普比率和索提诺比率
        if self.config.enable_sharpe_ratio {
            self.current_metrics.sharpe_ratio = self.calculate_sharpe_ratio();
            self.current_metrics.sortino_ratio = self.calculate_sortino_ratio();
        }

        // 胜率
        if self.current_metrics.total_trades > 0 {
            self.current_metrics.win_rate = Decimal::from(self.current_metrics.win_trades)
                / Decimal::from(self.current_metrics.total_trades)
                * Decimal::from(100);
        }

        // 盈亏比
        self.current_metrics.profit_loss_ratio = self.calculate_profit_loss_ratio();

        self.current_metrics.current_equity = current_equity;
        self.current_metrics.peak_equity = self.peak_equity;
        self.current_metrics.last_updated = Utc::now();
    }

    /// 计算夏普比率
    fn calculate_sharpe_ratio(&self) -> Decimal {
        if self.trade_returns.is_empty() {
            return Decimal::ZERO;
        }

        let returns: Vec<_> = self.trade_returns.iter().collect();
        let mean_return =
            returns.iter().map(|&&r| r).sum::<Decimal>() / Decimal::from(returns.len());

        let variance = returns
            .iter()
            .map(|&&r| (r - mean_return) * (r - mean_return))
            .sum::<Decimal>()
            / Decimal::from(returns.len());

        let std_dev = if variance > Decimal::ZERO {
            variance.sqrt().unwrap_or(Decimal::ZERO)
        } else {
            Decimal::ZERO
        };

        if std_dev > Decimal::ZERO {
            let excess_return = mean_return - (self.config.risk_free_rate / Decimal::from(365));
            excess_return / std_dev * Decimal::from(365).sqrt().unwrap_or(Decimal::ONE)
        } else {
            Decimal::ZERO
        }
    }

    /// 计算索提诺比率
    fn calculate_sortino_ratio(&self) -> Decimal {
        if self.trade_returns.is_empty() {
            return Decimal::ZERO;
        }

        let returns: Vec<_> = self.trade_returns.iter().collect();
        let mean_return =
            returns.iter().map(|&&r| r).sum::<Decimal>() / Decimal::from(returns.len());

        // 计算下行偏差
        let negative_returns: Vec<_> = returns.iter().filter(|&&r| *r < Decimal::ZERO).collect();
        if negative_returns.is_empty() {
            return Decimal::from(100); // 无下行风险，返回高分
        }

        let downside_variance = negative_returns.iter().map(|&&r| r * r).sum::<Decimal>()
            / Decimal::from(negative_returns.len());

        let downside_deviation = if downside_variance > Decimal::ZERO {
            downside_variance.sqrt().unwrap_or(Decimal::ZERO)
        } else {
            Decimal::ZERO
        };

        if downside_deviation > Decimal::ZERO {
            let excess_return = mean_return - (self.config.risk_free_rate / Decimal::from(365));
            excess_return / downside_deviation * Decimal::from(365).sqrt().unwrap_or(Decimal::ONE)
        } else {
            Decimal::ZERO
        }
    }

    /// 计算盈亏比
    fn calculate_profit_loss_ratio(&self) -> Decimal {
        let total_profit: Decimal = self
            .trade_returns
            .iter()
            .filter(|&&r| r > Decimal::ZERO)
            .sum();

        let total_loss: Decimal = self
            .trade_returns
            .iter()
            .filter(|&&r| r < Decimal::ZERO)
            .map(|&r| r.abs())
            .sum();

        if total_loss > Decimal::ZERO {
            total_profit / total_loss
        } else {
            Decimal::ZERO
        }
    }

    /// 获取当前指标
    pub fn get_current_metrics(&self) -> &RealtimeMetrics {
        &self.current_metrics
    }

    /// 获取权益历史
    pub fn get_equity_history(&self) -> &VecDeque<EquityPoint> {
        &self.equity_history
    }

    /// 检查是否触发回撤告警
    pub fn check_drawdown_alert(&self) -> bool {
        if self.config.enable_drawdown_monitoring {
            self.current_metrics.current_drawdown >= self.config.drawdown_alert_threshold
        } else {
            false
        }
    }

    /// 获取回撤状态
    pub fn get_drawdown_status(&self) -> DrawdownStatus {
        let current = self.current_metrics.current_drawdown;
        let threshold = self.config.drawdown_alert_threshold;

        if current >= threshold * dec!(2) {
            DrawdownStatus::Critical
        } else if current >= threshold {
            DrawdownStatus::Warning
        } else if current >= threshold / dec!(2) {
            DrawdownStatus::Caution
        } else {
            DrawdownStatus::Normal
        }
    }

    /// 重置监控器
    pub fn reset(&mut self) {
        self.equity_history.clear();
        self.trade_returns.clear();
        self.peak_equity = self.initial_capital;
        self.current_metrics = RealtimeMetrics {
            current_equity: self.initial_capital,
            ..Default::default()
        };
        self.start_date = Utc::now().naive_utc().date();
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::with_defaults(dec!(1000000));
        assert_eq!(monitor.get_current_metrics().current_equity, dec!(1000000));
    }

    #[test]
    fn test_update_equity() {
        let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

        monitor.update_equity(dec!(1010000), dec!(10000), dec!(1000000));

        assert_eq!(monitor.get_current_metrics().current_equity, dec!(1010000));
        assert_eq!(monitor.get_current_metrics().total_return, dec!(0.01));
    }

    #[test]
    fn test_drawdown_calculation() {
        let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

        monitor.update_equity(dec!(1100000), dec!(0), dec!(1100000)); // 新高
        monitor.update_equity(dec!(1000000), dec!(0), dec!(1000000)); // 回撤

        assert!(monitor.get_current_metrics().current_drawdown > dec!(0.09));
        assert!(monitor.get_current_metrics().current_drawdown < dec!(0.091));
    }

    #[test]
    fn test_max_drawdown() {
        let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

        monitor.update_equity(dec!(1100000), dec!(0), dec!(1100000));
        monitor.update_equity(dec!(900000), dec!(0), dec!(900000));
        monitor.update_equity(dec!(950000), dec!(0), dec!(950000));

        assert!(monitor.get_current_metrics().max_drawdown > dec!(0.18));
    }

    #[test]
    fn test_record_trade_pnl() {
        let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

        monitor.record_trade_pnl(dec!(1000));
        monitor.record_trade_pnl(dec!(-500));

        assert_eq!(monitor.get_current_metrics().total_trades, 2);
        assert_eq!(monitor.get_current_metrics().win_trades, 1);
        assert_eq!(monitor.get_current_metrics().loss_trades, 1);
    }

    #[test]
    fn test_win_rate_calculation() {
        let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

        monitor.record_trade_pnl(dec!(1000));
        monitor.record_trade_pnl(dec!(500));
        monitor.record_trade_pnl(dec!(-500));

        let metrics = monitor.get_current_metrics();
        assert!(metrics.win_rate > dec!(66));
        assert!(metrics.win_rate < dec!(67));
    }

    #[test]
    fn test_check_drawdown_alert() {
        let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

        monitor.update_equity(dec!(1100000), dec!(0), dec!(1100000));
        monitor.update_equity(dec!(950000), dec!(0), dec!(950000)); // 13.6% 回撤

        assert!(monitor.check_drawdown_alert());
    }

    #[test]
    fn test_get_drawdown_status() {
        let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

        monitor.update_equity(dec!(1100000), dec!(0), dec!(1100000));
        monitor.update_equity(dec!(800000), dec!(0), dec!(800000)); // 27% 回撤

        assert_eq!(monitor.get_drawdown_status(), DrawdownStatus::Critical);
    }

    #[test]
    fn test_profit_loss_ratio() {
        let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

        monitor.record_trade_pnl(dec!(2000));
        monitor.record_trade_pnl(dec!(1000));
        monitor.record_trade_pnl(dec!(-1000));
        monitor.record_trade_pnl(dec!(-500));

        let metrics = monitor.get_current_metrics();
        assert_eq!(metrics.profit_loss_ratio, dec!(2));
    }

    #[test]
    fn test_reset() {
        let mut monitor = PerformanceMonitor::with_defaults(dec!(1000000));

        monitor.update_equity(dec!(1100000), dec!(0), dec!(1100000));
        monitor.record_trade_pnl(dec!(1000));

        monitor.reset();

        assert_eq!(monitor.get_current_metrics().current_equity, dec!(1000000));
        assert_eq!(monitor.get_current_metrics().total_trades, 0);
        assert_eq!(monitor.get_equity_history().len(), 0);
    }

    #[test]
    fn test_equity_history_limit() {
        let config = PerformanceMonitorConfig {
            max_equity_history: 5,
            ..Default::default()
        };
        let mut monitor = PerformanceMonitor::new(config, dec!(1000000));

        for i in 0..10 {
            let equity = Decimal::from(1_000_000 + i * 1_000);
            monitor.update_equity(equity, Decimal::ZERO, equity);
        }

        assert_eq!(monitor.get_equity_history().len(), 5);
    }
}
