//! fin-primitives 集成金丝雀：确定性输入 + 手算期望。
//! 上游升级若打破任何断言 = 行为漂移，须人工核对（配合 Cargo 精确锁版本）。

use quantix_cli::data::models::{AdjustType, Kline};
use quantix_cli::fin;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;

fn kline(day: u32, close: &str) -> Kline {
    let c = Decimal::from_str(close).unwrap();
    Kline {
        code: "600000".into(),
        date: NaiveDate::from_ymd_opt(2026, 1, day).unwrap(),
        open: c,
        high: c,
        low: c,
        close: c,
        volume: 1000,
        amount: None,
        adjust_type: AdjustType::None,
    }
}

#[test]
fn canary_indicator_chain() {
    let klines: Vec<Kline> = (1..=10u32).map(|i| kline(i, &i.to_string())).collect();
    let sma = fin::indicators::sma(&klines, 3).unwrap();
    assert_eq!(sma.values[9], Some(Decimal::from_str("9").unwrap()));
    let rising: Vec<Kline> = (1..=30u32)
        .map(|i| kline(i.min(28).max(1), &i.to_string()))
        .collect();
    let rsi = fin::indicators::rsi(&rising, 14).unwrap();
    assert_eq!(rsi.values[29], Some(Decimal::from(100)));
}

#[test]
fn canary_drawdown_chain() {
    let report =
        fin::risk::drawdown_report(&[Decimal::from(100), Decimal::from(110), Decimal::from(99)])
            .unwrap();
    assert_eq!(report.worst_pct, Decimal::from_str("10").unwrap());
}

#[test]
fn canary_backtest_chain() {
    let mut closes: Vec<i64> = vec![10; 4];
    closes.extend(vec![12; 3]);
    closes.extend(vec![9; 3]);
    let klines: Vec<Kline> = closes
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let c = Decimal::from(*c);
            Kline {
                code: "600000".into(),
                date: NaiveDate::from_ymd_opt(2026, 1, (i + 1) as u32).unwrap(),
                open: c,
                high: c,
                low: c,
                close: c,
                volume: 1000,
                amount: None,
                adjust_type: AdjustType::None,
            }
        })
        .collect();
    let out = fin::backtest::run_fin_backtest(
        &klines,
        2,
        3,
        Decimal::from(10000),
        Decimal::ZERO,
        Decimal::from(100),
    )
    .unwrap();
    assert_eq!(out.final_equity, Decimal::from_str("9700").unwrap());
    assert_eq!(out.trade_count, 2);
}
