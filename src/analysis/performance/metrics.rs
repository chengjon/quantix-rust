use super::*;

/// 计算总收益率
pub fn calculate_total_return(equity_curve: &[Decimal]) -> Decimal {
    if equity_curve.is_empty() {
        return Decimal::ZERO;
    }
    let initial = equity_curve[0];
    let final_value = equity_curve.last().unwrap();
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
