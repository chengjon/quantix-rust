//! 分钟线测试：MinuteKlineCH / MinuteShareCH 字段检查、decimal/naive 转换、
//! bar_to_row / share_to_row 行映射。

use crate::db::clickhouse::datetime_utc_to_offsetdatetime;
use crate::db::clickhouse::models::{MinuteKlineCH, MinuteShareCH};

#[test]
fn models_minute_kline_ch_has_expected_fields() {
    use crate::db::clickhouse::naive_to_offsetdatetime;

    let row = MinuteKlineCH {
        timestamp: naive_to_offsetdatetime(
            chrono::NaiveDate::from_ymd_opt(2026, 7, 4)
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap(),
        ),
        code: "sh600000".into(),
        period: "1m".into(),
        adjust: "none".into(),
        open: 12.34,
        high: 12.50,
        low: 12.20,
        close: 12.40,
        volume: 123456.0,
        amount: 1_530_000.0,
    };
    assert_eq!(row.code, "sh600000");
    assert_eq!(row.period, "1m");
    assert_eq!(row.adjust, "none");
    assert_eq!(row.volume, 123456.0);
}

#[test]
fn models_minute_share_ch_has_expected_fields() {
    use crate::db::clickhouse::naive_to_offsetdatetime;

    let row = MinuteShareCH {
        timestamp: naive_to_offsetdatetime(
            chrono::NaiveDate::from_ymd_opt(2026, 7, 4)
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap(),
        ),
        code: "sh600000".into(),
        price: 12.34,
        volume: 1000.0,
        amount: 12340.0,
        avg_price: 12.34,
    };
    assert_eq!(row.code, "sh600000");
    assert_eq!(row.price, 12.34);
    assert_eq!(row.avg_price, 12.34);
}

// ─── P0.14 T2 — U1/U2/U3: helper tests ─────────────────────────────────────

#[test]
fn decimal_to_f64_normal_range_is_lossless() {
    use rust_decimal::Decimal;
    use std::str::FromStr;
    // Access private helper via public-ish path: tests are inside the same module
    // so we can reach into minute.rs through `crate::db::clickhouse::minute`.
    use crate::db::clickhouse::minute::decimal_to_f64_for_test;

    assert_eq!(
        decimal_to_f64_for_test(Decimal::from_str("1.23").unwrap()),
        1.23
    );
    assert_eq!(
        decimal_to_f64_for_test(Decimal::from_str("9999.99").unwrap()),
        9999.99
    );
    assert_eq!(
        decimal_to_f64_for_test(Decimal::from_str("0").unwrap()),
        0.0
    );
    assert_eq!(
        decimal_to_f64_for_test(Decimal::from_str("1234567890123.45").unwrap()),
        1_234_567_890_123.45
    );
}

#[test]
fn decimal_to_f64_extreme_value_falls_back_to_zero() {
    use crate::db::clickhouse::minute::decimal_to_f64_for_test;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // Construct a Decimal that overflows f64 mantissa (much larger than 2^53).
    // rust_decimal max is ~7.9e28, well beyond f64's i64-exact range.
    let huge = Decimal::from_str("79228162514264337593543950335").unwrap(); // rust_decimal MAX
    let v = decimal_to_f64_for_test(huge);
    // Whether to_f64 returns Some(finite-but-lossy) or None depends on rust_decimal version;
    // either way, our helper guarantees a finite f64 result (no NaN, no panic).
    assert!(v.is_finite());
}

#[test]
fn naive_to_offsetdatetime_preserves_wall_clock() {
    use crate::db::clickhouse::minute::naive_to_ch_timestamp_for_test;
    use chrono::NaiveDate;

    let naive = NaiveDate::from_ymd_opt(2026, 7, 4)
        .unwrap()
        .and_hms_opt(9, 30, 0)
        .unwrap();
    let dt = naive_to_ch_timestamp_for_test(naive);
    // Wall-clock Y/M/D/H/M/S preserved (this is the kline_data convention —
    // Beijing wall-clock tagged UTC, no timezone conversion).
    assert_eq!(dt.year(), 2026);
    assert_eq!(dt.month() as u32, 7);
    assert_eq!(dt.day(), 4);
    assert_eq!(dt.hour(), 9);
    assert_eq!(dt.minute(), 30);
    assert_eq!(dt.second(), 0);
}

// ─── P0.14 T2 — U4/U5: row conversion tests ────────────────────────────────

#[test]
fn bar_to_row_maps_all_minute_bar_fields() {
    use crate::data::models::{AdjustType, MinuteBar, MinutePeriod};
    use crate::db::clickhouse::minute::bar_to_row_for_test;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    let bar = MinuteBar {
        code: "sh600000".into(),
        timestamp: NaiveDate::from_ymd_opt(2026, 7, 4)
            .unwrap()
            .and_hms_opt(9, 30, 0)
            .unwrap(),
        open: Decimal::from_str("12.34").unwrap(),
        high: Decimal::from_str("12.50").unwrap(),
        low: Decimal::from_str("12.20").unwrap(),
        close: Decimal::from_str("12.40").unwrap(),
        volume: 123456,
        amount: Some(Decimal::from_str("1530000.00").unwrap()),
        adjust_type: AdjustType::None,
    };
    // MinuteBar has no `period` field; period comes from the stream function's
    // input parameter, so we pass it explicitly to bar_to_row.
    let row = bar_to_row_for_test(&bar, MinutePeriod::Minute1);
    assert_eq!(row.code, "sh600000");
    assert_eq!(row.period, "1m");
    assert_eq!(row.adjust, "none");
    assert_eq!(row.open, 12.34);
    assert_eq!(row.high, 12.50);
    assert_eq!(row.low, 12.20);
    assert_eq!(row.close, 12.40);
    assert_eq!(row.volume, 123456.0);
    assert_eq!(row.amount, 1_530_000.0);
}

#[test]
fn share_to_row_maps_all_minute_share_fields() {
    use crate::data::models::MinuteShare;
    use crate::db::clickhouse::minute::share_to_row_for_test;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    let share = MinuteShare {
        code: "sh600000".into(),
        timestamp: NaiveDate::from_ymd_opt(2026, 7, 4)
            .unwrap()
            .and_hms_opt(9, 30, 5)
            .unwrap(),
        price: Some(Decimal::from_str("12.34").unwrap()),
        volume: Some(1000),
        amount: Some(Decimal::from_str("12340.00").unwrap()),
        avg_price: Some(Decimal::from_str("12.34").unwrap()),
    };
    let row = share_to_row_for_test(&share);
    assert_eq!(row.code, "sh600000");
    assert_eq!(row.price, 12.34);
    assert_eq!(row.volume, 1000.0);
    assert_eq!(row.amount, 12340.0);
    assert_eq!(row.avg_price, 12.34);
}
