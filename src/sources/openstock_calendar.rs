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
///
/// Runtime (baostock provider) returns `{calendar_date, is_trading_day}`
/// where `is_trading_day` is a string `"0"`/`"1"`. We accept both
/// `calendar_date` (runtime) and `date` (legacy fixtures) via serde alias,
/// and tolerate `is_trading_day` as either string or bool.
#[derive(Debug, Deserialize)]
pub struct TradeDateRecord {
    #[serde(default, alias = "calendar_date")]
    pub date: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_loose")]
    pub is_trading_day: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Raw record shape for `WORKDAYS` payloads. `WORKDAYS` is action-driven
/// (eltdx provider); runtime returns `{action, date, ...}` where the
/// extra fields depend on the action value:
/// - `today` -> `{action, date}`
/// - `today_is_workday` -> `{action, today_is_workday: bool}`
/// - `is_workday` (date param) -> `{action, date, is_workday: bool}`
/// - `range` (start/end) -> `{action, date}` × N
/// - `next_workday`/`previous_workday` (date param) -> `{action, date, next_workday|previous_workday}`
///
/// We tolerate every optional field; consumers branch on `action`.
#[derive(Debug, Deserialize)]
pub struct WorkdayRecord {
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default, alias = "calendar_date")]
    pub date: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_loose")]
    pub is_workday: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_bool_loose")]
    pub today_is_workday: Option<bool>,
    #[serde(default)]
    pub next_workday: Option<String>,
    #[serde(default)]
    pub previous_workday: Option<String>,
    /// Legacy field kept for back-compat with P0.9 fixtures that used
    /// the union-calendar shape. Runtime does not populate this today.
    #[serde(default, deserialize_with = "deserialize_bool_loose")]
    pub is_trading_day: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Parsed trade-date entry. The canonical calendar primitive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeDate {
    pub date: NaiveDate,
    pub is_trading_day: bool,
}

/// Parsed workday entry. Reflects runtime's action-driven shape —
/// `is_workday`/`today_is_workday` carry the boolean signal when the
/// action provides one; `date`/`next_workday`/`previous_workday` carry
/// the date payload when present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workday {
    pub action: Option<String>,
    pub date: Option<NaiveDate>,
    pub is_workday: Option<bool>,
    pub today_is_workday: Option<bool>,
    pub next_workday: Option<NaiveDate>,
    pub previous_workday: Option<NaiveDate>,
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
            Ok(TradeDate {
                date,
                is_trading_day: record.is_trading_day.unwrap_or(false),
            })
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
            let date = record
                .date
                .filter(|text| !text.trim().is_empty())
                .map(|raw| parse_calendar_date(&raw))
                .transpose()?;
            let next_workday = record
                .next_workday
                .filter(|text| !text.trim().is_empty())
                .map(|raw| parse_calendar_date(&raw))
                .transpose()?;
            let previous_workday = record
                .previous_workday
                .filter(|text| !text.trim().is_empty())
                .map(|raw| parse_calendar_date(&raw))
                .transpose()?;
            Ok(Workday {
                action: record.action,
                date,
                is_workday: record.is_workday,
                today_is_workday: record.today_is_workday,
                next_workday,
                previous_workday,
            })
        })
        .collect()
}

/// Deserialize a boolean that may arrive as `true`/`false` or as the
/// string `"1"`/`"0"` (the latter is what baostock emits for
/// `is_trading_day`). Returns `None` for empty/unrecognized strings.
fn deserialize_bool_loose<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    Ok(opt.and_then(|v| match v {
        serde_json::Value::Bool(b) => Some(b),
        serde_json::Value::String(s) => match s.trim() {
            "1" | "true" => Some(true),
            "0" | "false" => Some(false),
            _ => None,
        },
        serde_json::Value::Number(n) if n.is_i64() => Some(n.as_i64() != Some(0)),
        _ => None,
    }))
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
        // Missing is_trading_day defaults to false.
        assert!(!dates[0].is_trading_day);
    }

    #[test]
    fn parse_trade_dates_accepts_runtime_calendar_date_alias() {
        // Real runtime (baostock) returns `calendar_date` + `is_trading_day`
        // as the string "0"/"1".
        let raw = r#"{
            "data": [
                {"calendar_date": "2015-01-01", "is_trading_day": "0"},
                {"calendar_date": "2015-01-05", "is_trading_day": "1"}
            ],
            "source": "baostock"
        }"#;
        let env: OpenStockEnvelope<TradeDateRecord> = serde_json::from_str(raw).unwrap();
        let dates = parse_trade_dates(env).unwrap();
        assert_eq!(dates.len(), 2);
        assert_eq!(dates[0].date.to_string(), "2015-01-01");
        assert!(!dates[0].is_trading_day);
        assert!(dates[1].is_trading_day);
    }

    #[test]
    fn parse_trade_dates_tolerates_string_and_bool_flags() {
        let raw = r#"{
            "data": [
                {"date": "2026-01-02", "is_trading_day": true},
                {"date": "2026-01-03", "is_trading_day": "0"},
                {"date": "2026-01-06", "is_trading_day": "1"}
            ]
        }"#;
        let env: OpenStockEnvelope<TradeDateRecord> = serde_json::from_str(raw).unwrap();
        let dates = parse_trade_dates(env).unwrap();
        assert!(dates[0].is_trading_day);
        assert!(!dates[1].is_trading_day);
        assert!(dates[2].is_trading_day);
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
    fn parse_workdays_today_action_matches_runtime_shape() {
        // Real runtime (eltdx) returns action="today" with just a date.
        let raw = r#"{
            "data": [{"action": "today", "date": "2026-06-30"}],
            "source": "eltdx"
        }"#;
        let env: OpenStockEnvelope<WorkdayRecord> = serde_json::from_str(raw).unwrap();
        let workdays = parse_workdays(env).unwrap();
        assert_eq!(workdays.len(), 1);
        assert_eq!(workdays[0].action.as_deref(), Some("today"));
        assert_eq!(workdays[0].date.unwrap().to_string(), "2026-06-30");
        assert!(workdays[0].is_workday.is_none());
    }

    #[test]
    fn parse_workdays_today_is_workday_action() {
        let raw = r#"{
            "data": [{"action": "today_is_workday", "today_is_workday": true}],
            "source": "eltdx"
        }"#;
        let env: OpenStockEnvelope<WorkdayRecord> = serde_json::from_str(raw).unwrap();
        let workdays = parse_workdays(env).unwrap();
        assert_eq!(workdays[0].today_is_workday, Some(true));
    }

    #[test]
    fn parse_workdays_is_workday_action() {
        let raw = r#"{
            "data": [{"action": "is_workday", "date": "2026-06-30", "is_workday": true}],
            "source": "eltdx"
        }"#;
        let env: OpenStockEnvelope<WorkdayRecord> = serde_json::from_str(raw).unwrap();
        let workdays = parse_workdays(env).unwrap();
        assert_eq!(workdays[0].is_workday, Some(true));
        assert_eq!(workdays[0].date.unwrap().to_string(), "2026-06-30");
    }

    #[test]
    fn parse_workdays_next_and_previous_actions() {
        let raw = r#"{
            "data": [
                {"action": "next_workday", "date": "2026-06-30", "next_workday": "2026-07-01"},
                {"action": "previous_workday", "date": "2026-06-30", "previous_workday": "2026-06-27"}
            ]
        }"#;
        let env: OpenStockEnvelope<WorkdayRecord> = serde_json::from_str(raw).unwrap();
        let workdays = parse_workdays(env).unwrap();
        assert_eq!(workdays[0].next_workday.unwrap().to_string(), "2026-07-01");
        assert_eq!(
            workdays[1].previous_workday.unwrap().to_string(),
            "2026-06-27"
        );
    }

    #[test]
    fn parse_workdays_range_action_emits_multiple_dates() {
        let raw = r#"{
            "data": [
                {"action": "range", "date": "2026-06-30"},
                {"action": "range", "date": "2026-07-01"}
            ]
        }"#;
        let env: OpenStockEnvelope<WorkdayRecord> = serde_json::from_str(raw).unwrap();
        let workdays = parse_workdays(env).unwrap();
        assert_eq!(workdays.len(), 2);
        assert_eq!(workdays[0].action.as_deref(), Some("range"));
    }

    #[test]
    fn parse_workdays_legacy_union_calendar_shape_still_works() {
        // P0.9 fixtures used {date, is_trading_day: bool} — keep tolerating.
        let raw = r#"{
            "data": [
                {"date": "2026-01-02", "is_trading_day": true},
                {"date": "2026-01-03", "is_trading_day": false}
            ]
        }"#;
        let env: OpenStockEnvelope<WorkdayRecord> = serde_json::from_str(raw).unwrap();
        let workdays = parse_workdays(env).unwrap();
        assert_eq!(workdays.len(), 2);
        assert!(workdays[0].date.is_some());
    }

    #[test]
    fn parse_workdays_defaults_missing_action_to_none() {
        let raw = r#"{"data": [{"date": "2026-01-02"}]}"#;
        let env: OpenStockEnvelope<WorkdayRecord> = serde_json::from_str(raw).unwrap();
        let workdays = parse_workdays(env).unwrap();
        assert_eq!(workdays.len(), 1);
        assert!(workdays[0].action.is_none());
    }

    #[test]
    fn parse_workdays_tolerates_extra_fields() {
        let raw = r#"{
            "data": [
                {"action": "today", "date": "2026-01-02", "note": "holiday-eve", "provider_meta": 42}
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
