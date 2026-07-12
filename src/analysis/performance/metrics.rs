//! Free-standing metric helpers backed by an equity curve / return series.
//!
//! Kept alongside the [`super::calculator`] engine so the math stays in one
//! place without bloating the calculator file.

use rust_decimal::prelude::*;

/// 计算总收益率
#[allow(dead_code)]
pub(crate) fn calculate_total_return(equity_curve: &[Decimal]) -> Decimal {
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
#[allow(dead_code)]
pub(crate) fn calculate_max_drawdown(equity_curve: &[Decimal]) -> Decimal {
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
#[allow(dead_code)]
pub(crate) fn calculate_sharpe_ratio(returns: &[Decimal], risk_free_rate: Decimal) -> Decimal {
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
    use crate::analysis::performance::calculator::PerformanceCalculator;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
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
