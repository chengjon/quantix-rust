//! Live shadow validation.
//!
//! Read-only validator for raw OpenStock `/data/bars` POST payloads
//! captured out-of-band. NEVER performs network I/O, NEVER writes
//! ClickHouse, NEVER replaces production data-source routes. Used by
//! the `quantix data openstock validate-live` CLI to produce a dry-run
//! report describing what *would* be persisted if the live payload
//! were ingested.

use super::kline::OpenStockKlineParseError;
use super::parse::{
    is_daily_period, normalize_symbol, parse_date, parse_decimal, parse_live_time, parse_volume,
    required_string, required_value,
};

use crate::core::QuantixError;
use crate::data::models::{AdjustType, Kline};
use chrono::NaiveDate;
use serde::Deserialize;
use std::fmt;

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
/// still mapped cleanly, `FailClosed` = at least one record could not
/// be mapped to a canonical `Kline` and must not be persisted.
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

/// Bridge used by the CLI handler to convert parse errors into the
/// project's canonical error type without losing the original message.
pub fn live_shadow_error_into_quantix(error: OpenStockKlineParseError) -> QuantixError {
    QuantixError::DataParse(error.to_string())
}
