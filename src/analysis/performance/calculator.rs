//! The performance-calculator engine that turns an equity curve and trade
//! log into a [`PerformanceReport`].

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

use super::report::{EquityPoint, PerformanceReport, TradeRecord};

/// 性能计算器
pub struct PerformanceCalculator {
    /// 权益曲线
    equity_curve: Vec<EquityPoint>,
    /// 交易记录
    trades: Vec<TradeRecord>,
    /// 初始资金
    initial_capital: Decimal,
    /// 无风险利率（年化）
    risk_free_rate: Decimal,
}

impl PerformanceCalculator {
    /// 创建性能计算器
    pub fn new(initial_capital: Decimal, risk_free_rate: Decimal) -> Self {
        Self {
            equity_curve: Vec::new(),
            trades: Vec::new(),
            initial_capital,
            risk_free_rate,
        }
    }

    /// 添加权益点
    pub fn add_equity_point(&mut self, date: NaiveDate, equity: Decimal) {
        let daily_return = if let Some(last) = self.equity_curve.last() {
            if last.equity > Decimal::ZERO {
                (equity - last.equity) / last.equity
            } else {
                Decimal::ZERO
            }
        } else {
            Decimal::ZERO
        };

        self.equity_curve.push(EquityPoint {
            date,
            equity,
            daily_return,
        });
    }

    /// 添加交易记录
    pub fn add_trade(&mut self, trade: TradeRecord) {
        self.trades.push(trade);
    }

    /// 计算性能报告
    pub fn calculate(&self) -> PerformanceReport {
        if self.equity_curve.is_empty() {
            return PerformanceReport::default();
        }

        let final_equity = self
            .equity_curve
            .last()
            .map(|p| p.equity)
            .unwrap_or(self.initial_capital);
        let total_return = if self.initial_capital > Decimal::ZERO {
            (final_equity - self.initial_capital) / self.initial_capital
        } else {
            Decimal::ZERO
        };

        let annual_return = self.calculate_annual_return(total_return);
        let max_drawdown = self.calculate_max_drawdown();
        let sharpe_ratio = self.calculate_sharpe_ratio();
        let sortino_ratio = self.calculate_sortino_ratio();

        let (
            win_rate,
            profit_loss_ratio,
            avg_win,
            avg_loss,
            max_win,
            max_loss,
            max_consecutive_wins,
            max_consecutive_losses,
            win_trades,
            loss_trades,
        ) = self.calculate_trade_stats();

        let total_commission: Decimal = self.trades.iter().map(|t| t.commission).sum();
        let calmar_ratio = if max_drawdown != Decimal::ZERO {
            annual_return / max_drawdown.abs()
        } else {
            Decimal::ZERO
        };

        PerformanceReport {
            total_return,
            annual_return,
            max_drawdown,
            sharpe_ratio,
            sortino_ratio,
            win_rate,
            profit_loss_ratio,
            total_trades: self.trades.len(),
            win_trades,
            loss_trades,
            avg_win,
            avg_loss,
            max_win,
            max_loss,
            max_consecutive_wins,
            max_consecutive_losses,
            total_commission,
            calmar_ratio,
        }
    }

    /// 计算年化收益率
    fn calculate_annual_return(&self, total_return: Decimal) -> Decimal {
        if self.equity_curve.len() < 2 {
            return Decimal::ZERO;
        }

        let start_date = self
            .equity_curve
            .first()
            .map(|p| p.date)
            .unwrap_or_default();
        let end_date = self.equity_curve.last().map(|p| p.date).unwrap_or_default();
        let days = (end_date - start_date).num_days().max(1) as u64;

        // 年化收益率 = (1 + total_return)^(365/days) - 1
        let days_decimal = Decimal::from(days);
        let power = dec!(365) / days_decimal;

        if power <= dec!(50) {
            // 防止溢出
            let one_plus_return = Decimal::ONE + total_return;
            // 使用简化的幂计算，因为 Decimal::pow 不稳定
            let exp = power.to_u32().unwrap_or(1);
            let mut annual = Decimal::ONE;
            for _ in 0..exp {
                annual *= one_plus_return;
            }
            annual - Decimal::ONE
        } else {
            // 简化计算
            total_return * dec!(365) / days_decimal
        }
    }

    /// 计算最大回撤
    fn calculate_max_drawdown(&self) -> Decimal {
        let mut max_drawdown = Decimal::ZERO;
        let mut peak = self.initial_capital;

        for point in &self.equity_curve {
            if point.equity > peak {
                peak = point.equity;
            }

            let drawdown = if peak > Decimal::ZERO {
                (peak - point.equity) / peak
            } else {
                Decimal::ZERO
            };

            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        max_drawdown
    }

    /// 计算夏普比率
    fn calculate_sharpe_ratio(&self) -> Decimal {
        if self.equity_curve.len() < 2 {
            return Decimal::ZERO;
        }

        // 计算日收益率序列
        let returns: Vec<Decimal> = self
            .equity_curve
            .iter()
            .skip(1)
            .map(|p| p.daily_return)
            .collect();

        if returns.is_empty() {
            return Decimal::ZERO;
        }

        // 平均日收益率
        let avg_daily_return: Decimal =
            returns.iter().sum::<Decimal>() / Decimal::from(returns.len());

        // 标准差
        let variance = returns
            .iter()
            .map(|r| (r - avg_daily_return) * (r - avg_daily_return))
            .sum::<Decimal>()
            / Decimal::from(returns.len());

        let std_dev = variance.sqrt().unwrap_or(Decimal::ZERO);

        if std_dev == Decimal::ZERO {
            return Decimal::ZERO;
        }

        // 年化
        let annual_return = avg_daily_return * dec!(252); // 252 个交易日
        let annual_std = std_dev * (dec!(252).sqrt().unwrap_or(Decimal::ZERO));

        // 调整无风险利率
        let excess_return = annual_return - self.risk_free_rate;

        if annual_std > Decimal::ZERO {
            excess_return / annual_std
        } else {
            Decimal::ZERO
        }
    }

    /// 计算索提诺比率
    fn calculate_sortino_ratio(&self) -> Decimal {
        if self.equity_curve.len() < 2 {
            return Decimal::ZERO;
        }

        let returns: Vec<Decimal> = self
            .equity_curve
            .iter()
            .skip(1)
            .map(|p| p.daily_return)
            .collect();

        let avg_return: Decimal = returns.iter().sum::<Decimal>() / Decimal::from(returns.len());

        // 下行偏差（只考虑负收益）
        let downside_variance = returns
            .iter()
            .filter(|r| **r < Decimal::ZERO)
            .map(|r| (r - Decimal::ZERO) * (r - Decimal::ZERO))
            .sum::<Decimal>()
            / Decimal::from(returns.len());

        let downside_dev = downside_variance.sqrt().unwrap_or(Decimal::ZERO);

        if downside_dev == Decimal::ZERO {
            return Decimal::ZERO;
        }

        let annual_return = avg_return * dec!(252);
        let annual_downside = downside_dev * (dec!(252).sqrt().unwrap_or(Decimal::ZERO));
        let excess_return = annual_return - self.risk_free_rate;

        if annual_downside > Decimal::ZERO {
            excess_return / annual_downside
        } else {
            Decimal::ZERO
        }
    }

    /// 计算交易统计
    fn calculate_trade_stats(
        &self,
    ) -> (
        Decimal,
        Decimal,
        Decimal,
        Decimal,
        Decimal,
        Decimal,
        usize,
        usize,
        usize,
        usize,
    ) {
        if self.trades.is_empty() {
            return (
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO,
                0,
                0,
                0,
                0,
            );
        }

        let wins: Vec<&TradeRecord> = self
            .trades
            .iter()
            .filter(|t| t.pnl > Decimal::ZERO)
            .collect();
        let losses: Vec<&TradeRecord> = self
            .trades
            .iter()
            .filter(|t| t.pnl < Decimal::ZERO)
            .collect();

        let win_trades = wins.len();
        let loss_trades = losses.len();
        let total_trades = self.trades.len();

        let win_rate = if total_trades > 0 {
            Decimal::from(win_trades) / Decimal::from(total_trades) * dec!(100)
        } else {
            Decimal::ZERO
        };

        let avg_win = if win_trades > 0 {
            wins.iter().map(|t| t.pnl).sum::<Decimal>() / Decimal::from(win_trades)
        } else {
            Decimal::ZERO
        };

        let avg_loss = if loss_trades > 0 {
            losses.iter().map(|t| t.pnl.abs()).sum::<Decimal>() / Decimal::from(loss_trades)
        } else {
            Decimal::ZERO
        };

        let profit_loss_ratio = if avg_loss != Decimal::ZERO {
            avg_win / avg_loss
        } else {
            Decimal::ZERO
        };

        let max_win = wins.iter().map(|t| t.pnl).max().unwrap_or(Decimal::ZERO);
        let max_loss = losses.iter().map(|t| t.pnl).min().unwrap_or(Decimal::ZERO);

        // 计算最大连续盈亏
        let (max_consecutive_wins, max_consecutive_losses) = self.calculate_consecutive();

        (
            win_rate,
            profit_loss_ratio,
            avg_win,
            avg_loss,
            max_win,
            max_loss,
            max_consecutive_wins,
            max_consecutive_losses,
            win_trades,
            loss_trades,
        )
    }

    /// 计算最大连续盈亏
    fn calculate_consecutive(&self) -> (usize, usize) {
        let mut max_consecutive_wins = 0;
        let mut max_consecutive_losses = 0;
        let mut current_wins = 0;
        let mut current_losses = 0;

        for trade in &self.trades {
            if trade.pnl > Decimal::ZERO {
                current_wins += 1;
                current_losses = 0;
                max_consecutive_wins = max_consecutive_wins.max(current_wins);
            } else if trade.pnl < Decimal::ZERO {
                current_losses += 1;
                current_wins = 0;
                max_consecutive_losses = max_consecutive_losses.max(current_losses);
            }
        }

        (max_consecutive_wins, max_consecutive_losses)
    }

    /// 获取权益曲线
    pub fn equity_curve(&self) -> &[EquityPoint] {
        &self.equity_curve
    }

    /// 获取交易记录
    pub fn trades(&self) -> &[TradeRecord] {
        &self.trades
    }
}
