use rust_decimal::Decimal;
use std::str::FromStr;

use crate::data::models::{AdjustType, BarPeriod, MinutePeriod, MinuteShare};

#[test]
fn bar_period_as_str_round_trip() {
    assert_eq!(BarPeriod::Day.as_str(), "day");
    assert_eq!(BarPeriod::Week.as_str(), "week");
    assert_eq!(BarPeriod::Month.as_str(), "month");
}

#[test]
fn bar_period_from_str_accepts_canonical_case_insensitive() {
    assert!(matches!(BarPeriod::from_str("day"), Ok(BarPeriod::Day)));
    assert!(matches!(BarPeriod::from_str("WEEK"), Ok(BarPeriod::Week)));
    assert!(matches!(BarPeriod::from_str("Month"), Ok(BarPeriod::Month)));
}

#[test]
fn bar_period_from_str_rejects_aliases() {
    // D6: strict — reject daily/weekly/monthly/minute* aliases
    assert!(BarPeriod::from_str("daily").is_err());
    assert!(BarPeriod::from_str("weekly").is_err());
    assert!(BarPeriod::from_str("monthly").is_err());
    assert!(BarPeriod::from_str("1m").is_err());
    assert!(BarPeriod::from_str("minute").is_err());
    assert!(BarPeriod::from_str("").is_err());
}

#[test]
fn adjust_type_as_openstock_param() {
    assert_eq!(AdjustType::None.as_openstock_param(), None);
    assert_eq!(AdjustType::QFQ.as_openstock_param(), Some("qfq"));
    assert_eq!(AdjustType::HFQ.as_openstock_param(), Some("hfq"));
}

#[test]
fn adjust_type_from_str_case_insensitive() {
    assert!(matches!(AdjustType::from_str("none"), Ok(AdjustType::None)));
    assert!(matches!(AdjustType::from_str("QFQ"), Ok(AdjustType::QFQ)));
    assert!(matches!(AdjustType::from_str("Hfq"), Ok(AdjustType::HFQ)));
    assert!(AdjustType::from_str("front").is_err());
    assert!(AdjustType::from_str("").is_err());
}

#[test]
fn minute_period_as_str_round_trip() {
    assert_eq!(MinutePeriod::Minute1.as_str(), "1m");
    assert_eq!(MinutePeriod::Minute5.as_str(), "5m");
    assert_eq!(MinutePeriod::Minute15.as_str(), "15m");
    assert_eq!(MinutePeriod::Minute30.as_str(), "30m");
    assert_eq!(MinutePeriod::Minute60.as_str(), "60m");
}

#[test]
fn minute_period_from_str_accepts_canonical_case_insensitive() {
    assert!(matches!(
        MinutePeriod::from_str("1m"),
        Ok(MinutePeriod::Minute1)
    ));
    assert!(matches!(
        MinutePeriod::from_str("5M"),
        Ok(MinutePeriod::Minute5)
    ));
    assert!(matches!(
        MinutePeriod::from_str("15m"),
        Ok(MinutePeriod::Minute15)
    ));
    assert!(matches!(
        MinutePeriod::from_str("30M"),
        Ok(MinutePeriod::Minute30)
    ));
    assert!(matches!(
        MinutePeriod::from_str("60m"),
        Ok(MinutePeriod::Minute60)
    ));
}

#[test]
fn minute_period_from_str_rejects_aliases() {
    // D4 strict — reject 1min/minute/5min/1h/hour and any day* value
    assert!(MinutePeriod::from_str("1min").is_err());
    assert!(MinutePeriod::from_str("minute").is_err());
    assert!(MinutePeriod::from_str("5min").is_err());
    assert!(MinutePeriod::from_str("1h").is_err());
    assert!(MinutePeriod::from_str("hour").is_err());
    assert!(MinutePeriod::from_str("day").is_err());
    assert!(MinutePeriod::from_str("").is_err());
}

#[test]
fn minute_share_round_trip_serde() {
    use chrono::NaiveDate;
    let share = MinuteShare {
        code: "sh600000".to_string(),
        timestamp: NaiveDate::from_ymd_opt(2026, 7, 1)
            .unwrap()
            .and_hms_opt(9, 30, 0)
            .unwrap(),
        price: Some(Decimal::from_str("10.50").unwrap()),
        volume: Some(123_456),
        amount: Some(Decimal::from_str("1296288.00").unwrap()),
        avg_price: Some(Decimal::from_str("10.4975").unwrap()),
    };
    let json = serde_json::to_string(&share).expect("serialize");
    let back: MinuteShare = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.code, "sh600000");
    assert_eq!(back.volume, Some(123_456));
    assert_eq!(back.avg_price, Some(Decimal::from_str("10.4975").unwrap()));
}

#[test]
fn minute_share_allows_missing_optional_fields() {
    // Missing price/volume/amount/avg_price fields → all None (INV-2C foundation)
    let json = r#"{"code":"sh600000","timestamp":"2026-07-01T09:30:00"}"#;
    let share: MinuteShare =
        serde_json::from_str(json).expect("deserialize with missing optionals");
    assert_eq!(share.code, "sh600000");
    assert_eq!(share.price, None);
    assert_eq!(share.volume, None);
    assert_eq!(share.amount, None);
    assert_eq!(share.avg_price, None);
}
