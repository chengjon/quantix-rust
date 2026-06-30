//! Parsers for OpenStock calendar categories `TRADE_DATES` and `WORKDAYS`.
//!
//! Both categories share a uniform envelope (`OpenStockEnvelope<T>`)
//! and a uniform date string shape. Records carry a `#[serde(flatten)]
//! extra` catch-all so provider-specific fields beyond the minimal
//! parsing set are tolerated without failing the parse (per the rule:
//! do not hardcode field names beyond the minimal parsing set).

use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

use crate::core::QuantixError;
use crate::sources::openstock_envelope::OpenStockEnvelope;

/// Calendar-parse error family. Mirrors the `OpenStockKlineParseError`
/// shape but scoped to the calendar categories (no decimal/volume/high-low
/// concerns here).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CalendarParseError {
    #[error("invalid OpenStock calendar JSON: {0}")]
    InvalidJson(String),
    #[error("OpenStock calendar payload contains no records")]
    EmptyRecords,
    #[error("missing required OpenStock calendar field: {0}")]
    MissingField(&'static str),
    #[error("invalid OpenStock calendar date `{value}`, expected {expected}")]
    InvalidDate {
        value: String,
        expected: &'static str,
    },
}

/// Bridge used by CLI handlers to convert parse errors into the
/// project's canonical error type without losing the original message.
pub fn calendar_error_into_quantix(error: CalendarParseError) -> QuantixError {
    QuantixError::DataParse(error.to_string())
}

/// Raw record shape for `TRADE_DATES` payloads.
#[derive(Debug, Deserialize)]
pub struct TradeDateRecord {
    #[serde(default)]
    pub date: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Raw record shape for `WORKDAYS` payloads. `WORKDAYS` is union-shaped:
/// it carries both trading and non-trading days, with `is_trading_day`
/// indicating which.
#[derive(Debug, Deserialize)]
pub struct WorkdayRecord {
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub is_trading_day: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Parsed trade-date entry. The canonical calendar primitive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeDate {
    pub date: NaiveDate,
}

/// Parsed workday entry — a date plus its trading-day flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workday {
    pub date: NaiveDate,
    pub is_trading_day: bool,
}

/// Parse the `TRADE_DATES` category envelope into typed trade dates.
pub fn parse_trade_dates(
    envelope: OpenStockEnvelope<TradeDateRecord>,
) -> Result<Vec<TradeDate>, CalendarParseError> {
    if envelope.is_empty() {
        return Err(CalendarParseError::EmptyRecords);
    }
    envelope
        .data
        .into_iter()
        .map(|record| {
            let raw = record
                .date
                .filter(|text| !text.trim().is_empty())
                .ok_or(CalendarParseError::MissingField("date"))?;
            let date = parse_calendar_date(&raw)?;
            Ok(TradeDate { date })
        })
        .collect()
}

/// Parse the `WORKDAYS` category envelope into typed workday entries.
pub fn parse_workdays(
    envelope: OpenStockEnvelope<WorkdayRecord>,
) -> Result<Vec<Workday>, CalendarParseError> {
    if envelope.is_empty() {
        return Err(CalendarParseError::EmptyRecords);
    }
    envelope
        .data
        .into_iter()
        .map(|record| {
            let raw = record
                .date
                .filter(|text| !text.trim().is_empty())
                .ok_or(CalendarParseError::MissingField("date"))?;
            let date = parse_calendar_date(&raw)?;
            let is_trading_day = record.is_trading_day.unwrap_or(false);
            Ok(Workday {
                date,
                is_trading_day,
            })
        })
        .collect()
}

/// Parse the calendar date string. Accepts both `%Y-%m-%d` (e.g.
/// `2026-01-02`) and `%Y%m%d` (e.g. `20260102`) — the two shapes that
/// appear across the eltdx/baostock providers.
pub fn parse_calendar_date(value: &str) -> Result<NaiveDate, CalendarParseError> {
    let trimmed = value.trim();
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Ok(date);
    }
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y%m%d") {
        return Ok(date);
    }
    Err(CalendarParseError::InvalidDate {
        value: value.to_string(),
        expected: "%Y-%m-%d or %Y%m%d",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_trade_dates_happy() {
        let raw = r#"{
            "data": [
                {"date": "2026-01-02"},
                {"date": "2026-01-03"},
                {"date": "20260106"}
            ],
            "source": "eltdx"
        }"#;
        let env: OpenStockEnvelope<TradeDateRecord> = serde_json::from_str(raw).unwrap();
        let dates = parse_trade_dates(env).unwrap();
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[0].date.to_string(), "2026-01-02");
        assert_eq!(dates[2].date.to_string(), "2026-01-06");
    }

    #[test]
    fn parse_trade_dates_empty_errors() {
        let raw = r#"{"data": [], "source": "eltdx"}"#;
        let env: OpenStockEnvelope<TradeDateRecord> = serde_json::from_str(raw).unwrap();
        assert_eq!(
            parse_trade_dates(env),
            Err(CalendarParseError::EmptyRecords)
        );
    }

    #[test]
    fn parse_trade_dates_missing_field_errors() {
        let raw = r#"{"data": [{}]}"#;
        let env: OpenStockEnvelope<TradeDateRecord> = serde_json::from_str(raw).unwrap();
        assert_eq!(
            parse_trade_dates(env),
            Err(CalendarParseError::MissingField("date"))
        );
    }

    #[test]
    fn parse_trade_dates_invalid_format_errors() {
        let raw = r#"{"data": [{"date": "02/01/2026"}]}"#;
        let env: OpenStockEnvelope<TradeDateRecord> = serde_json::from_str(raw).unwrap();
        match parse_trade_dates(env) {
            Err(CalendarParseError::InvalidDate { value, .. }) => {
                assert_eq!(value, "02/01/2026");
            }
            other => panic!("expected InvalidDate, got {:?}", other),
        }
    }

    #[test]
    fn parse_workdays_happy() {
        let raw = r#"{
            "data": [
                {"date": "2026-01-02", "is_trading_day": true},
                {"date": "2026-01-03", "is_trading_day": false},
                {"date": "2026-01-04", "is_trading_day": true}
            ],
            "source": "eltdx"
        }"#;
        let env: OpenStockEnvelope<WorkdayRecord> = serde_json::from_str(raw).unwrap();
        let workdays = parse_workdays(env).unwrap();
        assert_eq!(workdays.len(), 3);
        assert!(workdays[0].is_trading_day);
        assert!(!workdays[1].is_trading_day);
    }

    #[test]
    fn parse_workdays_defaults_missing_flag_to_false() {
        let raw = r#"{"data": [{"date": "2026-01-02"}]}"#;
        let env: OpenStockEnvelope<WorkdayRecord> = serde_json::from_str(raw).unwrap();
        let workdays = parse_workdays(env).unwrap();
        assert_eq!(workdays.len(), 1);
        assert!(!workdays[0].is_trading_day);
    }

    #[test]
    fn parse_workdays_tolerates_extra_fields() {
        let raw = r#"{
            "data": [
                {"date": "2026-01-02", "is_trading_day": true, "note": "holiday-eve", "provider_meta": 42}
            ]
        }"#;
        let env: OpenStockEnvelope<WorkdayRecord> = serde_json::from_str(raw).unwrap();
        let workdays = parse_workdays(env).unwrap();
        assert_eq!(workdays.len(), 1);
        // Extra fields are captured in `extra` and do not break the parse.
    }

    #[test]
    fn parse_calendar_date_accepts_both_formats() {
        assert_eq!(
            parse_calendar_date("2026-01-02").unwrap().to_string(),
            "2026-01-02"
        );
        assert_eq!(
            parse_calendar_date("20260102").unwrap().to_string(),
            "2026-01-02"
        );
    }

    #[test]
    fn calendar_error_into_quantix_preserves_message() {
        let error = CalendarParseError::EmptyRecords;
        let quantix = calendar_error_into_quantix(error);
        let text = quantix.to_string();
        assert!(text.contains("no records"));
    }
}
