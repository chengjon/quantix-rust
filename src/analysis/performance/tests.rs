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
