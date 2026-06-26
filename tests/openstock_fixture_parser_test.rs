use quantix_cli::data::models::AdjustType;
use quantix_cli::sources::{OpenStockKlineParseError, parse_daily_kline_json};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

const DAILY_KLINE_FIXTURE: &str = include_str!("fixtures/openstock/daily_kline.json");

#[test]
fn parses_openstock_daily_kline_fixture_into_canonical_klines() {
    let klines = parse_daily_kline_json(DAILY_KLINE_FIXTURE).expect("fixture should parse");

    assert_eq!(klines.len(), 2);

    let first = &klines[0];
    assert_eq!(first.code, "600000");
    assert_eq!(first.date.to_string(), "2026-06-22");
    assert_eq!(first.open, dec!(9.80));
    assert_eq!(first.high, dec!(10.15));
    assert_eq!(first.low, dec!(9.70));
    assert_eq!(first.close, dec!(10.05));
    assert_eq!(first.volume, 1_234_567);
    assert_eq!(first.amount, Some(dec!(12345678.90)));
    assert!(matches!(first.adjust_type, AdjustType::None));
}

#[test]
fn rejects_empty_daily_kline_payload() {
    let err = parse_daily_kline_json(
        r#"{"provider":"openstock","period":"daily","adjust_type":"none","records":[]}"#,
    )
    .expect_err("empty records must be rejected");

    assert_eq!(err, OpenStockKlineParseError::EmptyRecords);
}

#[test]
fn rejects_missing_code() {
    let err = parse_daily_kline_json(
        r#"{"provider":"openstock","period":"daily","adjust_type":"none","records":[{"date":"2026-06-22","open":"9.80","high":"10.15","low":"9.70","close":"10.05","volume":1234567}]}"#,
    )
    .expect_err("missing code must be rejected");

    assert_eq!(err, OpenStockKlineParseError::MissingField("code"));
}

#[test]
fn rejects_unparseable_date() {
    let err = parse_daily_kline_json(
        r#"{"provider":"openstock","period":"daily","adjust_type":"none","records":[{"code":"600000","date":"20260622","open":"9.80","high":"10.15","low":"9.70","close":"10.05","volume":1234567}]}"#,
    )
    .expect_err("bad date must be rejected");

    assert!(matches!(
        err,
        OpenStockKlineParseError::InvalidDate {
            value,
            expected_format: "%Y-%m-%d"
        } if value == "20260622"
    ));
}

#[test]
fn rejects_non_finite_numeric_values() {
    let err = parse_daily_kline_json(
        r#"{"provider":"openstock","period":"daily","adjust_type":"none","records":[{"code":"600000","date":"2026-06-22","open":"NaN","high":"10.15","low":"9.70","close":"10.05","volume":1234567}]}"#,
    )
    .expect_err("non-finite decimal must be rejected");

    assert_eq!(
        err,
        OpenStockKlineParseError::InvalidDecimal {
            field: "open",
            value: "NaN".to_string()
        }
    );
}

#[test]
fn rejects_high_below_low() {
    let err = parse_daily_kline_json(
        r#"{"provider":"openstock","period":"daily","adjust_type":"none","records":[{"code":"600000","date":"2026-06-22","open":"9.80","high":"9.60","low":"9.70","close":"10.05","volume":1234567}]}"#,
    )
    .expect_err("high below low must be rejected");

    assert_eq!(
        err,
        OpenStockKlineParseError::HighBelowLow {
            code: "600000".to_string(),
            date: "2026-06-22".to_string(),
            high: dec!(9.60),
            low: dec!(9.70)
        }
    );
}

#[test]
fn rejects_unsupported_period() {
    let err = parse_daily_kline_json(
        r#"{"provider":"openstock","period":"minute","adjust_type":"none","records":[{"code":"600000","date":"2026-06-22","open":"9.80","high":"10.15","low":"9.70","close":"10.05","volume":1234567}]}"#,
    )
    .expect_err("non-daily period must be rejected");

    assert_eq!(
        err,
        OpenStockKlineParseError::UnsupportedPeriod("minute".to_string())
    );
}

#[test]
fn rejects_mixed_record_code() {
    let err = parse_daily_kline_json(
        r#"{"provider":"openstock","period":"daily","adjust_type":"none","records":[{"code":"600000","date":"2026-06-22","open":"9.80","high":"10.15","low":"9.70","close":"10.05","volume":1234567},{"code":"000001","date":"2026-06-23","open":"10.05","high":"10.30","low":"9.95","close":"10.20","volume":2345678}]}"#,
    )
    .expect_err("mixed code payload must be rejected");

    assert_eq!(
        err,
        OpenStockKlineParseError::MixedCode {
            expected: "600000".to_string(),
            actual: "000001".to_string()
        }
    );
}

#[test]
fn accepts_numeric_json_values_without_losing_decimal_contract() {
    let klines = parse_daily_kline_json(
        r#"{"provider":"openstock","period":"daily","adjust_type":"none","records":[{"code":"600000","date":"2026-06-22","open":9.8,"high":10.15,"low":9.70,"close":10.05,"volume":1234567,"amount":12345678.90}]}"#,
    )
    .expect("numeric JSON values should parse");

    assert_eq!(klines[0].open, Decimal::from_str_exact("9.8").unwrap());
    assert_eq!(
        klines[0].amount,
        Some(Decimal::from_str_exact("12345678.9").unwrap())
    );
}
