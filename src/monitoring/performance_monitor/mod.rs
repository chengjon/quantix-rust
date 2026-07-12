//! 性能监控模块
//!
//! 实时计算和追踪性能指标. The data types live in [`types`]; this module
//! owns the [`PerformanceMonitor`] engine that folds equity updates and
//! trade PnL into [`RealtimeMetrics`].

#[cfg(test)]
mod tests;

mod types;

pub use types::{PerformanceMonitorConfig, RealtimeMetrics};

use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::collections::VecDeque;

use types::{DrawdownStatus, EquityPoint};

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
