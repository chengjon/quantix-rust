use chrono::NaiveDate;
use quantix_cli::data::models::{AdjustType, Kline};
use quantix_cli::screener::{
    PresetInvocation, PresetKind, evaluate_preset, parse_preset_invocation, required_lookback,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::BTreeMap;

fn make_kline(day: u32, close: Decimal, volume: i64) -> Kline {
    Kline {
        code: "000001".to_string(),
        date: NaiveDate::from_ymd_opt(2024, 1, day).unwrap(),
        open: close,
        high: close + dec!(1),
        low: close - dec!(1),
        close,
        volume,
        amount: None,
        adjust_type: AdjustType::None,
    }
}

#[test]
fn evaluates_close_above_ma() {
    let invocation = parse_preset_invocation("close_above_ma:period=3").unwrap();
    let klines = vec![
        make_kline(1, dec!(10), 100),
        make_kline(2, dec!(10), 100),
        make_kline(3, dec!(10), 100),
        make_kline(4, dec!(11), 100),
        make_kline(5, dec!(12), 100),
    ];

    let detail = evaluate_preset(&invocation, &klines).unwrap();

    assert_eq!(required_lookback(&invocation).unwrap(), 3);
    assert!(detail.matched);
    assert_eq!(detail.actual_value, Some(dec!(12)));
    assert_eq!(detail.threshold_value, Some(dec!(11)));
}

#[test]
fn evaluates_close_below_ma() {
    let invocation = parse_preset_invocation("close_below_ma:period=3").unwrap();
    let klines = vec![
        make_kline(1, dec!(12), 100),
        make_kline(2, dec!(11), 100),
        make_kline(3, dec!(10), 100),
        make_kline(4, dec!(9), 100),
        make_kline(5, dec!(8), 100),
    ];

    let detail = evaluate_preset(&invocation, &klines).unwrap();

    assert!(detail.matched);
    assert_eq!(detail.actual_value, Some(dec!(8)));
    assert_eq!(detail.threshold_value, Some(dec!(9)));
}

#[test]
fn evaluates_rsi_gte() {
    let invocation = parse_preset_invocation("rsi_gte:period=3,value=55").unwrap();
    let klines = vec![
        make_kline(1, dec!(10), 100),
        make_kline(2, dec!(11), 100),
        make_kline(3, dec!(12), 100),
        make_kline(4, dec!(13), 100),
        make_kline(5, dec!(14), 100),
    ];

    let detail = evaluate_preset(&invocation, &klines).unwrap();

    assert_eq!(required_lookback(&invocation).unwrap(), 4);
    assert!(detail.matched);
    assert!(detail.actual_value.unwrap() >= dec!(55));
    assert_eq!(detail.threshold_value, Some(dec!(55)));
}

#[test]
fn evaluates_rsi_lte() {
    let invocation = parse_preset_invocation("rsi_lte:period=3,value=45").unwrap();
    let klines = vec![
        make_kline(1, dec!(14), 100),
        make_kline(2, dec!(13), 100),
        make_kline(3, dec!(12), 100),
        make_kline(4, dec!(11), 100),
        make_kline(5, dec!(10), 100),
    ];

    let detail = evaluate_preset(&invocation, &klines).unwrap();

    assert!(detail.matched);
    assert!(detail.actual_value.unwrap() <= dec!(45));
    assert_eq!(detail.threshold_value, Some(dec!(45)));
}

#[test]
fn evaluates_volume_ratio_gte() {
    let invocation = parse_preset_invocation("volume_ratio_gte:window=5,value=1.5").unwrap();
    let klines = vec![
        make_kline(1, dec!(10), 100),
        make_kline(2, dec!(10), 100),
        make_kline(3, dec!(10), 100),
        make_kline(4, dec!(10), 100),
        make_kline(5, dec!(10), 300),
    ];

    let detail = evaluate_preset(&invocation, &klines).unwrap();

    assert_eq!(required_lookback(&invocation).unwrap(), 5);
    assert!(detail.matched);
    assert!(detail.actual_value.unwrap() >= dec!(1.5));
    assert_eq!(detail.threshold_value, Some(dec!(1.5)));
}

#[test]
fn rejects_volume_ratio_zero_window_without_panicking() {
    let invocation = PresetInvocation {
        kind: PresetKind::VolumeRatioGte,
        params: BTreeMap::from([
            ("window".to_string(), "0".to_string()),
            ("value".to_string(), "1.5".to_string()),
        ]),
    };
    let klines = vec![make_kline(1, dec!(10), 100)];

    let lookback_err = required_lookback(&invocation).unwrap_err();
    let err = evaluate_preset(&invocation, &klines).unwrap_err();

    assert!(lookback_err.to_string().contains("window"));
    assert!(err.to_string().contains("window"));
}

#[test]
fn returns_non_match_with_reason_when_kline_window_is_too_short() {
    let invocation = parse_preset_invocation("close_above_ma:period=5").unwrap();
    let klines = vec![
        make_kline(1, dec!(10), 100),
        make_kline(2, dec!(11), 100),
        make_kline(3, dec!(12), 100),
    ];

    let detail = evaluate_preset(&invocation, &klines).unwrap();

    assert!(!detail.matched);
    assert!(detail.reason.unwrap().contains("数据不足"));
    assert_eq!(detail.actual_value, None);
}
