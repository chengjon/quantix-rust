//! Shared parsing primitives for both the daily-kline JSON lane and the
//! live shadow lane in [`super`].
//!
//! The functions here carry no I/O and no clock reads beyond what their
//! callers pass in; they turn raw strings/serde values into canonical
//! domain types, failing with [`OpenStockKlineParseError`].

use super::kline::OpenStockKlineParseError;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;

pub(crate) fn required_string(
    value: Option<String>,
    field: &'static str,
) -> Result<String, OpenStockKlineParseError> {
    value
        .filter(|text| !text.trim().is_empty())
        .ok_or(OpenStockKlineParseError::MissingField(field))
}

pub(crate) fn required_value(
    value: Option<serde_json::Value>,
    field: &'static str,
) -> Result<serde_json::Value, OpenStockKlineParseError> {
    value.ok_or(OpenStockKlineParseError::MissingField(field))
}

pub(crate) fn parse_date(value: &str) -> Result<NaiveDate, OpenStockKlineParseError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| {
        OpenStockKlineParseError::InvalidDate {
            value: value.to_string(),
            expected_format: "%Y-%m-%d",
        }
    })
}

pub(crate) fn parse_decimal(
    value: serde_json::Value,
    field: &'static str,
) -> Result<Decimal, OpenStockKlineParseError> {
    let text = match value {
        serde_json::Value::String(text) => text,
        serde_json::Value::Number(number) => number.to_string(),
        other => {
            return Err(OpenStockKlineParseError::InvalidDecimal {
                field,
                value: other.to_string(),
            });
        }
    };

    Decimal::from_str(&text)
        .map_err(|_| OpenStockKlineParseError::InvalidDecimal { field, value: text })
}

pub(crate) fn parse_volume(value: serde_json::Value) -> Result<i64, OpenStockKlineParseError> {
    match value {
        serde_json::Value::Number(number) => number
            .as_i64()
            .ok_or_else(|| OpenStockKlineParseError::InvalidVolume(number.to_string())),
        serde_json::Value::String(text) => text
            .parse::<i64>()
            .map_err(|_| OpenStockKlineParseError::InvalidVolume(text)),
        other => Err(OpenStockKlineParseError::InvalidVolume(other.to_string())),
    }
}

/// Strip a leading exchange prefix (`sh`/`sz`/`bj`) so that request-side
/// symbol `600000` and record-side symbol `sh600000` compare equal. The
/// numeric form is the canonical `Kline.code` shape used elsewhere in
/// the codebase, so the normalized value is what we store.
pub(crate) fn normalize_symbol(symbol: &str) -> String {
    let lower = symbol.to_ascii_lowercase();
    for prefix in ["sh", "sz", "bj"] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            return rest.to_string();
        }
    }
    symbol.to_string()
}

/// Parse the `time` field of a live `/data/bars` record. Real envelopes
/// send RFC3339 with a timezone offset (e.g. `2026-01-23T15:00:00+08:00`);
/// local fixtures use `YYYY-MM-DD`. Both forms are accepted, and the
/// resulting [`NaiveDate`] is the canonical `Kline.date` shape.
pub(crate) fn parse_live_time(value: &str) -> Result<NaiveDate, OpenStockKlineParseError> {
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Ok(date);
    }
    if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(value) {
        return Ok(timestamp.naive_local().date());
    }
    Err(OpenStockKlineParseError::InvalidDate {
        value: value.to_string(),
        expected_format: "%Y-%m-%d or RFC3339",
    })
}

/// Accept `daily` (fixture shape) and `day` (real `/data/bars` shape) as
/// equivalent. Anything else is unsupported for the daily shadow lane.
pub(crate) fn is_daily_period(period: &str) -> bool {
    matches!(period, "daily" | "day")
}
