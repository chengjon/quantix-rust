//! Integration tests for `openstock_calendar` parsers (fixture-driven).

use quantix_cli::sources::openstock_calendar::{
    CalendarParseError, parse_trade_dates, parse_workdays,
};
use quantix_cli::sources::openstock_envelope::OpenStockEnvelope;

const TRADE_DATES_FIXTURE: &str = include_str!("fixtures/openstock/trade_dates.json");
const TRADE_DATES_EMPTY_FIXTURE: &str = include_str!("fixtures/openstock/trade_dates_empty.json");
const WORKDAYS_FIXTURE: &str = include_str!("fixtures/openstock/workdays.json");

#[test]
fn parse_trade_dates_fixture() {
    let env: OpenStockEnvelope<_> = serde_json::from_str(TRADE_DATES_FIXTURE).unwrap();
    let dates = parse_trade_dates(env).unwrap();
    assert_eq!(dates.len(), 5);
    assert_eq!(dates[0].date.to_string(), "2026-01-02");
    assert!(dates[0].is_trading_day);
    assert!(!dates[1].is_trading_day);
    assert!(dates[3].is_trading_day);
}

#[test]
fn parse_trade_dates_empty_errors() {
    let env: OpenStockEnvelope<_> = serde_json::from_str(TRADE_DATES_EMPTY_FIXTURE).unwrap();
    assert_eq!(
        parse_trade_dates(env),
        Err(CalendarParseError::EmptyRecords)
    );
}

#[test]
fn parse_workdays_fixture() {
    let env: OpenStockEnvelope<_> = serde_json::from_str(WORKDAYS_FIXTURE).unwrap();
    let workdays = parse_workdays(env).unwrap();
    assert_eq!(workdays.len(), 4);
    assert_eq!(workdays[0].action.as_deref(), Some("today"));
    assert_eq!(workdays[0].date.unwrap().to_string(), "2026-06-30");
    assert_eq!(workdays[1].today_is_workday, Some(true));
    assert_eq!(workdays[2].is_workday, Some(true));
    assert_eq!(workdays[3].action.as_deref(), Some("range"));
}
