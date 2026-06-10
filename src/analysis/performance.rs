/// 性能指标计算
///
/// 从短线侠项目迁移 - 夏普比率、最大回撤、胜率等
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// 回测性能报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// 总收益率
    pub total_return: Decimal,
    /// 年化收益率
    pub annual_return: Decimal,
    /// 最大回撤
    pub max_drawdown: Decimal,
    /// 夏普比率
    pub sharpe_ratio: Decimal,
    /// 索提诺比率
    pub sortino_ratio: Decimal,
    /// 胜率
    pub win_rate: Decimal,
    /// 盈亏比
    pub profit_loss_ratio: Decimal,
    /// 总交易次数
    pub total_trades: usize,
    /// 盈利交易次数
    pub win_trades: usize,
    /// 亏损交易次数
    pub loss_trades: usize,
    /// 平均盈利
    pub avg_win: Decimal,
    /// 平均亏损
    pub avg_loss: Decimal,
    /// 最大盈利
    pub max_win: Decimal,
    /// 最大亏损
    pub max_loss: Decimal,
    /// 最大连续盈利次数
    pub max_consecutive_wins: usize,
    /// 最大连续亏损次数
    pub max_consecutive_losses: usize,
    /// 总手续费
    pub total_commission: Decimal,
    /// 卡玛比率
    pub calmar_ratio: Decimal,
}

impl Default for PerformanceReport {
    fn default() -> Self {
        Self {
            total_return: Decimal::ZERO,
            annual_return: Decimal::ZERO,
            max_drawdown: Decimal::ZERO,
            sharpe_ratio: Decimal::ZERO,
            sortino_ratio: Decimal::ZERO,
            win_rate: Decimal::ZERO,
            profit_loss_ratio: Decimal::ZERO,
            total_trades: 0,
            win_trades: 0,
            loss_trades: 0,
            avg_win: Decimal::ZERO,
            avg_loss: Decimal::ZERO,
            max_win: Decimal::ZERO,
            max_loss: Decimal::ZERO,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
            total_commission: Decimal::ZERO,
            calmar_ratio: Decimal::ZERO,
        }
    }
}

/// 账户价值记录
#[derive(Debug, Clone)]
pub struct EquityPoint {
    /// 日期
    pub date: NaiveDate,
    /// 账户价值
    pub equity: Decimal,
    /// 当日收益
    pub daily_return: Decimal,
}

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    /// 股票代码
    pub code: String,
    /// 交易方向
    pub side: TradeSide,
    /// 开仓日期
    pub open_date: NaiveDate,
    /// 平仓日期
    pub close_date: NaiveDate,
    /// 开仓价格
    pub open_price: Decimal,
    /// 平仓价格
    pub close_price: Decimal,
    /// 数量
    pub quantity: i64,
    /// 盈亏
    pub pnl: Decimal,
    /// 盈亏比例
    pub pnl_percent: Decimal,
    /// 手续费
    pub commission: Decimal,
}

/// 交易方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeSide {
    Long,
    Short,
}

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

        let final_equity = self.equity_curve.last().unwrap().equity;
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

        let start_date = self.equity_curve.first().unwrap().date;
        let end_date = self.equity_curve.last().unwrap().date;
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

/// 计算总收益率
pub fn calculate_total_return(equity_curve: &[Decimal]) -> Decimal {
    if equity_curve.is_empty() {
        return Decimal::ZERO;
    }
    let initial = equity_curve[0];
    let final_value = match equity_curve.last() {
        Some(value) => value,
        None => return Decimal::ZERO,
    };
    if initial > Decimal::ZERO {
        (final_value - initial) / initial
    } else {
        Decimal::ZERO
    }
}

/// 计算最大回撤
pub fn calculate_max_drawdown(equity_curve: &[Decimal]) -> Decimal {
    if equity_curve.len() < 2 {
        return Decimal::ZERO;
    }

    let mut peak = equity_curve[0];
    let mut max_dd = Decimal::ZERO;

    for &value in equity_curve.iter().skip(1) {
        if value > peak {
            peak = value;
        } else {
            let drawdown = (peak - value) / peak;
            if drawdown > max_dd {
                max_dd = drawdown;
            }
        }
    }

    max_dd
}

/// 计算夏普比率
pub fn calculate_sharpe_ratio(returns: &[Decimal], risk_free_rate: Decimal) -> Decimal {
    if returns.is_empty() {
        return Decimal::ZERO;
    }

    // 计算平均收益率
    let sum: Decimal = returns.iter().sum();
    let avg_return = sum / Decimal::from(returns.len() as i64);

    // 计算标准差
    let variance = returns
        .iter()
        .map(|r| {
            let diff = r - avg_return;
            diff * diff
        })
        .sum::<Decimal>()
        / Decimal::from(returns.len() as i64);

    let std_dev = variance.sqrt().unwrap_or(Decimal::ZERO);

    // 年化（假设252个交易日）
    let annualized_return = avg_return * Decimal::from(252);
    let annualized_std = std_dev * Decimal::from(252).sqrt().unwrap_or(Decimal::ZERO);

    if annualized_std > Decimal::ZERO {
        (annualized_return - risk_free_rate) / annualized_std
    } else {
        Decimal::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_performance_calculator() {
        let mut calc = PerformanceCalculator::new(dec!(100000), dec!(0.03));

        // 添加权益点
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        calc.add_equity_point(date, dec!(100000));

        let date2 = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
        calc.add_equity_point(date2, dec!(102000));

        let report = calc.calculate();
        assert_eq!(report.total_return, dec!(0.02));
    }

    #[test]
    fn test_max_drawdown() {
        let mut calc = PerformanceCalculator::new(dec!(100000), dec!(0.03));

        calc.add_equity_point(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), dec!(100000));
        calc.add_equity_point(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            dec!(110000), // 峰值
        );
        calc.add_equity_point(
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
            dec!(95000), // 回撤
        );

        let report = calc.calculate();
        // 最大回撤 = (110000 - 95000) / 110000 ≈ 0.136
        assert!(report.max_drawdown > dec!(0.13));
        assert!(report.max_drawdown < dec!(0.14));
    }
}
