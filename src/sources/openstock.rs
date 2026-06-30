use std::fmt;
use std::str::FromStr;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use thiserror::Error;

use crate::core::QuantixError;
use crate::data::models::{AdjustType, Kline};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OpenStockKlineParseError {
    #[error("invalid OpenStock daily kline JSON: {0}")]
    InvalidJson(String),
    #[error("unsupported OpenStock period: {0}")]
    UnsupportedPeriod(String),
    #[error("unsupported OpenStock adjust_type: {0}")]
    UnsupportedAdjustType(String),
    #[error("OpenStock daily kline payload contains no records")]
    EmptyRecords,
    #[error("missing required OpenStock field: {0}")]
    MissingField(&'static str),
    #[error("invalid OpenStock date `{value}`, expected {expected_format}")]
    InvalidDate {
        value: String,
        expected_format: &'static str,
    },
    #[error("invalid OpenStock decimal field `{field}`: {value}")]
    InvalidDecimal { field: &'static str, value: String },
    #[error("invalid OpenStock volume: {0}")]
    InvalidVolume(String),
    #[error("OpenStock high is below low for {code} on {date}: high={high}, low={low}")]
    HighBelowLow {
        code: String,
        date: String,
        high: Decimal,
        low: Decimal,
    },
    #[error("OpenStock payload mixes codes: expected {expected}, got {actual}")]
    MixedCode { expected: String, actual: String },
}

#[derive(Debug, Deserialize)]
struct OpenStockDailyKlinePayload {
    period: String,
    adjust_type: Option<String>,
    records: Vec<OpenStockDailyKlineRecord>,
}

#[derive(Debug, Deserialize)]
struct OpenStockDailyKlineRecord {
    code: Option<String>,
    date: Option<String>,
    open: Option<serde_json::Value>,
    high: Option<serde_json::Value>,
    low: Option<serde_json::Value>,
    close: Option<serde_json::Value>,
    volume: Option<serde_json::Value>,
    amount: Option<serde_json::Value>,
}

pub fn parse_daily_kline_json(input: &str) -> Result<Vec<Kline>, OpenStockKlineParseError> {
    let payload: OpenStockDailyKlinePayload = serde_json::from_str(input)
        .map_err(|error| OpenStockKlineParseError::InvalidJson(error.to_string()))?;

    if payload.period != "daily" {
        return Err(OpenStockKlineParseError::UnsupportedPeriod(payload.period));
    }

    let adjust_type = parse_adjust_type(payload.adjust_type.as_deref())?;
    if payload.records.is_empty() {
        return Err(OpenStockKlineParseError::EmptyRecords);
    }

    let mut expected_code: Option<String> = None;
    let mut klines = Vec::with_capacity(payload.records.len());

    for record in payload.records {
        let kline = record.into_kline(adjust_type)?;

        match &expected_code {
            Some(expected) if expected != &kline.code => {
                return Err(OpenStockKlineParseError::MixedCode {
                    expected: expected.clone(),
                    actual: kline.code,
                });
            }
            Some(_) => {}
            None => expected_code = Some(kline.code.clone()),
        }

        klines.push(kline);
    }

    Ok(klines)
}

impl OpenStockDailyKlineRecord {
    fn into_kline(self, adjust_type: AdjustType) -> Result<Kline, OpenStockKlineParseError> {
        let code = required_string(self.code, "code")?;
        let date_text = required_string(self.date, "date")?;
        let date = parse_date(&date_text)?;
        let open = parse_decimal(required_value(self.open, "open")?, "open")?;
        let high = parse_decimal(required_value(self.high, "high")?, "high")?;
        let low = parse_decimal(required_value(self.low, "low")?, "low")?;
        let close = parse_decimal(required_value(self.close, "close")?, "close")?;
        let volume = parse_volume(required_value(self.volume, "volume")?)?;
        let amount = self
            .amount
            .map(|value| parse_decimal(value, "amount"))
            .transpose()?;

        if high < low {
            return Err(OpenStockKlineParseError::HighBelowLow {
                code,
                date: date_text,
                high,
                low,
            });
        }

        Ok(Kline {
            code,
            date,
            open,
            high,
            low,
            close,
            volume,
            amount,
            adjust_type,
        })
    }
}

fn parse_adjust_type(value: Option<&str>) -> Result<AdjustType, OpenStockKlineParseError> {
    match value.unwrap_or("none") {
        "none" => Ok(AdjustType::None),
        "qfq" => Ok(AdjustType::QFQ),
        "hfq" => Ok(AdjustType::HFQ),
        other => Err(OpenStockKlineParseError::UnsupportedAdjustType(
            other.to_string(),
        )),
    }
}

fn required_string(
    value: Option<String>,
    field: &'static str,
) -> Result<String, OpenStockKlineParseError> {
    value
        .filter(|text| !text.trim().is_empty())
        .ok_or(OpenStockKlineParseError::MissingField(field))
}

fn required_value(
    value: Option<serde_json::Value>,
    field: &'static str,
) -> Result<serde_json::Value, OpenStockKlineParseError> {
    value.ok_or(OpenStockKlineParseError::MissingField(field))
}

fn parse_date(value: &str) -> Result<NaiveDate, OpenStockKlineParseError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| {
        OpenStockKlineParseError::InvalidDate {
            value: value.to_string(),
            expected_format: "%Y-%m-%d",
        }
    })
}

fn parse_decimal(
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

fn parse_volume(value: serde_json::Value) -> Result<i64, OpenStockKlineParseError> {
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

// ============================================================
// P0.8f — live shadow validation
//
// Read-only validator for raw OpenStock `/data/bars` POST payloads
// captured out-of-band. NEVER performs network I/O, NEVER writes
// ClickHouse, NEVER replaces production data-source routes. Used by
// the `quantix data openstock validate-live` CLI to produce a dry-run
// report describing what *would* be persisted if the live payload
// were ingested.
// ============================================================

/// Drift rule tags emitted by the live shadow validator.
pub const DRIFT_RULE_LIMIT: &str = "received_count_exceeds_limit";
pub const DRIFT_RULE_OUT_OF_WINDOW: &str = "out_of_requested_window";

/// Request-side parameters captured alongside the live payload. These
/// mirror what the operator sent to OpenStock and let the validator
/// detect service-side anomalies (e.g. start/end/limit not honored).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveShadowRequest {
    pub symbol: String,
    pub period: String,
    pub start_date: String,
    pub end_date: String,
    pub limit: Option<u32>,
}

/// One drift observation. Multiple drifts can coexist on the same payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveShadowDrift {
    pub rule: &'static str,
    pub detail: String,
}

/// Status the report ends in. `Ok` = safe to (hypothetically) persist,
/// `Drift` = service behavior diverged from the request but every record
/// still mapped cleanly, `FailClosed` = at least one record could not be
/// mapped to a canonical `Kline` and must not be persisted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveShadowStatus {
    Ok,
    Drift,
    FailClosed,
}

impl fmt::Display for LiveShadowStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok => f.write_str("ok"),
            Self::Drift => f.write_str("drift"),
            Self::FailClosed => f.write_str("fail_closed"),
        }
    }
}

/// Result of validating one captured live payload. Carries enough
/// information for a dry-run report without retaining the raw bytes.
#[derive(Debug, Clone)]
pub struct LiveShadowReport {
    pub dry_run: bool,
    pub source: &'static str,
    pub status: LiveShadowStatus,
    pub record_count: usize,
    pub mapped_count: usize,
    pub symbol: Option<String>,
    pub period: Option<String>,
    pub received_date_range: Option<(NaiveDate, NaiveDate)>,
    pub drifts: Vec<LiveShadowDrift>,
    pub fail_closed_errors: Vec<OpenStockKlineParseError>,
    pub klines: Vec<Kline>,
}

impl fmt::Display for LiveShadowReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "OpenStock live shadow validation")?;
        writeln!(f, "  dry_run: {}", self.dry_run)?;
        writeln!(f, "  source: {}", self.source)?;
        writeln!(f, "  status: {}", self.status)?;
        writeln!(f, "  records: {}", self.record_count)?;
        writeln!(f, "  mapped: {}", self.mapped_count)?;
        if let Some(symbol) = self.symbol.as_deref() {
            writeln!(f, "  symbol: {}", symbol)?;
        } else {
            writeln!(f, "  symbol: <unknown>")?;
        }
        if let Some(period) = self.period.as_deref() {
            writeln!(f, "  period: {}", period)?;
        }
        if let Some((start, end)) = self.received_date_range {
            writeln!(f, "  received_date_range: {}..{}", start, end)?;
        }
        if !self.drifts.is_empty() {
            writeln!(f, "  drifts:")?;
            for drift in &self.drifts {
                writeln!(f, "    - {}: {}", drift.rule, drift.detail)?;
            }
        }
        if !self.fail_closed_errors.is_empty() {
            writeln!(f, "  fail_closed_errors:")?;
            for error in &self.fail_closed_errors {
                writeln!(f, "    - {}", error)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct LiveShadowEnvelope {
    /// Records field. Real `/data/bars` envelopes use `data`; the
    /// `records` alias is retained for symmetry with the local fixture
    /// shape used by `parse_daily_kline_json`.
    #[serde(default, alias = "data")]
    records: Vec<LiveShadowRecord>,
}

#[derive(Debug, Deserialize)]
struct LiveShadowRecord {
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    time: Option<String>,
    #[serde(default)]
    open: Option<serde_json::Value>,
    #[serde(default)]
    high: Option<serde_json::Value>,
    #[serde(default)]
    low: Option<serde_json::Value>,
    #[serde(default)]
    close: Option<serde_json::Value>,
    #[serde(default)]
    volume: Option<serde_json::Value>,
    #[serde(default)]
    amount: Option<serde_json::Value>,
    #[serde(default)]
    period: Option<String>,
}

/// Validate a captured OpenStock `/data/bars` POST response payload.
///
/// Never performs network I/O and never mutates external state. The
/// returned [`LiveShadowReport`] is a pure data description of what
/// would be persisted if the live payload were ingested. Records that
/// fail to map to a canonical [`Kline`] are recorded as fail-closed
/// errors rather than aborting the whole report.
pub fn validate_live_shadow_payload(
    raw: &str,
    request: &LiveShadowRequest,
) -> std::result::Result<LiveShadowReport, OpenStockKlineParseError> {
    let envelope: LiveShadowEnvelope = serde_json::from_str(raw)
        .map_err(|error| OpenStockKlineParseError::InvalidJson(error.to_string()))?;

    if envelope.records.is_empty() {
        return Err(OpenStockKlineParseError::EmptyRecords);
    }

    let envelope_period = envelope
        .records
        .first()
        .and_then(|r| r.period.clone())
        .unwrap_or_else(|| request.period.clone());
    let adjust_type = AdjustType::None;
    let requested_start = parse_date(&request.start_date).ok();
    let requested_end = parse_date(&request.end_date).ok();

    let mut mapped: Vec<Kline> = Vec::with_capacity(envelope.records.len());
    let mut fail_closed_errors: Vec<OpenStockKlineParseError> = Vec::new();
    let mut symbol_lock: Option<String> = None;
    let mut min_date: Option<NaiveDate> = None;
    let mut max_date: Option<NaiveDate> = None;

    for record in envelope.records {
        match map_live_record(record, &envelope_period, adjust_type, &request.symbol) {
            Ok(kline) => {
                if symbol_lock.is_none() {
                    symbol_lock = Some(kline.code.clone());
                }
                min_date = Some(match min_date {
                    Some(current) if current < kline.date => current,
                    _ => kline.date,
                });
                max_date = Some(match max_date {
                    Some(current) if current > kline.date => current,
                    _ => kline.date,
                });
                mapped.push(kline);
            }
            Err(error) => fail_closed_errors.push(error),
        }
    }

    let record_count = mapped.len() + fail_closed_errors.len();
    let received_date_range = match (min_date, max_date) {
        (Some(start), Some(end)) => Some((start, end)),
        _ => None,
    };

    let mut drifts = Vec::new();
    if let Some(limit) = request.limit {
        let limit_usize = usize::try_from(limit).unwrap_or(usize::MAX);
        if record_count > limit_usize {
            drifts.push(LiveShadowDrift {
                rule: DRIFT_RULE_LIMIT,
                detail: format!(
                    "service returned {} records despite requested limit {}",
                    record_count, limit
                ),
            });
        }
    }
    if let (Some(start), Some(end), Some((recv_start, recv_end))) =
        (requested_start, requested_end, received_date_range)
        && (recv_start < start || recv_end > end)
    {
        drifts.push(LiveShadowDrift {
            rule: DRIFT_RULE_OUT_OF_WINDOW,
            detail: format!(
                "received {}..{} falls outside requested {}..{}",
                recv_start, recv_end, start, end
            ),
        });
    }

    let status = if !fail_closed_errors.is_empty() {
        LiveShadowStatus::FailClosed
    } else if !drifts.is_empty() {
        LiveShadowStatus::Drift
    } else {
        LiveShadowStatus::Ok
    };

    Ok(LiveShadowReport {
        dry_run: true,
        source: "openstock_live_shadow",
        status,
        record_count,
        mapped_count: mapped.len(),
        symbol: symbol_lock,
        period: Some(envelope_period),
        received_date_range,
        drifts,
        fail_closed_errors,
        klines: mapped,
    })
}

fn map_live_record(
    record: LiveShadowRecord,
    envelope_period: &str,
    adjust_type: AdjustType,
    requested_symbol: &str,
) -> std::result::Result<Kline, OpenStockKlineParseError> {
    let record_period = record.period.as_deref().unwrap_or(envelope_period);
    if !is_daily_period(record_period) {
        return Err(OpenStockKlineParseError::UnsupportedPeriod(
            record_period.to_string(),
        ));
    }

    let raw_symbol = required_string(record.symbol, "symbol")?;
    let normalized_symbol = normalize_symbol(&raw_symbol);
    if normalized_symbol != normalize_symbol(requested_symbol) {
        return Err(OpenStockKlineParseError::MixedCode {
            expected: requested_symbol.to_string(),
            actual: raw_symbol.clone(),
        });
    }

    let time_text = required_string(record.time, "time")?;
    let date = parse_live_time(&time_text)?;

    let open = parse_decimal(required_value(record.open, "open")?, "open")?;
    let high = parse_decimal(required_value(record.high, "high")?, "high")?;
    let low = parse_decimal(required_value(record.low, "low")?, "low")?;
    let close = parse_decimal(required_value(record.close, "close")?, "close")?;
    let volume = parse_volume(required_value(record.volume, "volume")?)?;
    let amount = record
        .amount
        .map(|value| parse_decimal(value, "amount"))
        .transpose()?;

    if high < low {
        return Err(OpenStockKlineParseError::HighBelowLow {
            code: normalized_symbol.clone(),
            date: time_text,
            high,
            low,
        });
    }

    Ok(Kline {
        code: normalized_symbol,
        date,
        open,
        high,
        low,
        close,
        volume,
        amount,
        adjust_type,
    })
}

/// Accept `daily` (fixture shape) and `day` (real `/data/bars` shape)
/// as equivalent. Anything else is unsupported for the daily shadow lane.
fn is_daily_period(period: &str) -> bool {
    matches!(period, "daily" | "day")
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
pub(crate) fn parse_live_time(
    value: &str,
) -> std::result::Result<NaiveDate, OpenStockKlineParseError> {
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

/// Bridge used by the CLI handler to convert parse errors into the
/// project's canonical error type without losing the original message.
pub fn live_shadow_error_into_quantix(error: OpenStockKlineParseError) -> QuantixError {
    QuantixError::DataParse(error.to_string())
}
