//! Parser for the OpenStock `INDEX_KLINES` category.
//!
//! Reuses the canonical [`Kline`] type and the `normalize_symbol` /
//! `parse_live_time` helpers widened to `pub(crate)` in `openstock.rs`.
//! Index klines always carry `AdjustType::None`.

use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use thiserror::Error;

use crate::core::QuantixError;
use crate::data::models::{AdjustType, Kline};
use crate::sources::openstock::parse_live_time;
use crate::sources::openstock_envelope::OpenStockEnvelope;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IndexKlineParseError {
    #[error("invalid OpenStock index kline JSON: {0}")]
    InvalidJson(String),
    #[error("OpenStock index kline payload contains no records")]
    EmptyRecords,
    #[error("missing required OpenStock index kline field: {0}")]
    MissingField(&'static str),
    #[error("invalid OpenStock index kline decimal field `{field}`: {value}")]
    InvalidDecimal { field: &'static str, value: String },
    #[error("invalid OpenStock index kline volume: {0}")]
    InvalidVolume(String),
    #[error("invalid OpenStock index kline date `{value}`, expected {expected}")]
    InvalidDate {
        value: String,
        expected: &'static str,
    },
    #[error("OpenStock index kline high is below low for {code} on {date}: high={high}, low={low}")]
    HighBelowLow {
        code: String,
        date: String,
        high: Decimal,
        low: Decimal,
    },
    #[error("OpenStock index kline payload mixes codes: expected {expected}, got {actual}")]
    MixedCode { expected: String, actual: String },
}

pub fn index_kline_error_into_quantix(error: IndexKlineParseError) -> QuantixError {
    QuantixError::DataParse(error.to_string())
}

#[derive(Debug, Deserialize)]
pub struct IndexKlineRecord {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub open: Option<serde_json::Value>,
    #[serde(default)]
    pub high: Option<serde_json::Value>,
    #[serde(default)]
    pub low: Option<serde_json::Value>,
    #[serde(default)]
    pub close: Option<serde_json::Value>,
    #[serde(default)]
    pub volume: Option<serde_json::Value>,
    #[serde(default)]
    pub amount: Option<serde_json::Value>,
}

pub fn parse_index_klines(
    envelope: OpenStockEnvelope<IndexKlineRecord>,
) -> Result<Vec<Kline>, IndexKlineParseError> {
    if envelope.is_empty() {
        return Err(IndexKlineParseError::EmptyRecords);
    }
    let adjust_type = AdjustType::None;
    let mut expected_code: Option<String> = None;
    let mut out = Vec::with_capacity(envelope.data.len());
    for record in envelope.data {
        let kline = map_index_record(record, adjust_type)?;
        match &expected_code {
            Some(expected) if expected != &kline.code => {
                return Err(IndexKlineParseError::MixedCode {
                    expected: expected.clone(),
                    actual: kline.code,
                });
            }
            Some(_) => {}
            None => expected_code = Some(kline.code.clone()),
        }
        out.push(kline);
    }
    Ok(out)
}

fn map_index_record(
    record: IndexKlineRecord,
    adjust_type: AdjustType,
) -> Result<Kline, IndexKlineParseError> {
    use crate::sources::openstock::normalize_symbol;
    let raw_symbol = record
        .symbol
        .filter(|text| !text.trim().is_empty())
        .ok_or(IndexKlineParseError::MissingField("symbol"))?;
    let code = normalize_symbol(&raw_symbol);

    let time_text = record
        .time
        .filter(|text| !text.trim().is_empty())
        .ok_or(IndexKlineParseError::MissingField("time"))?;
    let date = parse_live_time(&time_text).map_err(|_| IndexKlineParseError::InvalidDate {
        value: time_text.clone(),
        expected: "%Y-%m-%d or RFC3339",
    })?;

    let open = parse_decimal(record.open, "open")?;
    let high = parse_decimal(record.high, "high")?;
    let low = parse_decimal(record.low, "low")?;
    let close = parse_decimal(record.close, "close")?;
    let volume = parse_volume(record.volume)?;
    let amount = record
        .amount
        .map(|v| parse_decimal(Some(v), "amount"))
        .transpose()?;

    if high < low {
        return Err(IndexKlineParseError::HighBelowLow {
            code,
            date: time_text,
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

fn parse_decimal(
    value: Option<serde_json::Value>,
    field: &'static str,
) -> Result<Decimal, IndexKlineParseError> {
    let value = value.ok_or(IndexKlineParseError::MissingField(field))?;
    let text = match value {
        serde_json::Value::String(text) => text,
        serde_json::Value::Number(number) => number.to_string(),
        other => {
            return Err(IndexKlineParseError::InvalidDecimal {
                field,
                value: other.to_string(),
            });
        }
    };
    Decimal::from_str(&text)
        .map_err(|_| IndexKlineParseError::InvalidDecimal { field, value: text })
}

fn parse_volume(value: Option<serde_json::Value>) -> Result<i64, IndexKlineParseError> {
    let value = value.ok_or(IndexKlineParseError::MissingField("volume"))?;
    match value {
        serde_json::Value::Number(number) => number
            .as_i64()
            .ok_or_else(|| IndexKlineParseError::InvalidVolume(number.to_string())),
        serde_json::Value::String(text) => text
            .parse::<i64>()
            .map_err(|_| IndexKlineParseError::InvalidVolume(text)),
        other => Err(IndexKlineParseError::InvalidVolume(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_index_klines_happy() {
        let raw = r#"{
            "data": [
                {"symbol":"sh000001","time":"2026-01-02","open":"3000","high":"3050","low":"2990","close":"3040","volume":"100000000","amount":"1500000000"},
                {"symbol":"sh000001","time":"2026-01-03","open":"3040","high":"3080","low":"3030","close":"3070","volume":"90000000","amount":"1400000000"}
            ],
            "source":"baostock"
        }"#;
        let env: OpenStockEnvelope<IndexKlineRecord> = serde_json::from_str(raw).unwrap();
        let klines = parse_index_klines(env).unwrap();
        assert_eq!(klines.len(), 2);
        assert_eq!(klines[0].code, "000001");
        assert_eq!(klines[0].date.to_string(), "2026-01-02");
        assert!(matches!(klines[0].adjust_type, AdjustType::None));
    }

    #[test]
    fn parse_index_klines_empty_errors() {
        let raw = r#"{"data": []}"#;
        let env: OpenStockEnvelope<IndexKlineRecord> = serde_json::from_str(raw).unwrap();
        match parse_index_klines(env) {
            Err(IndexKlineParseError::EmptyRecords) => {}
            other => panic!("expected EmptyRecords, got {:?}", other),
        }
    }

    #[test]
    fn parse_index_klines_high_below_low_errors() {
        let raw = r#"{
            "data":[{"symbol":"sh000001","time":"2026-01-02","open":"10","high":"5","low":"20","close":"15","volume":"1","amount":"1"}]
        }"#;
        let env: OpenStockEnvelope<IndexKlineRecord> = serde_json::from_str(raw).unwrap();
        match parse_index_klines(env) {
            Err(IndexKlineParseError::HighBelowLow { code, .. }) => {
                assert_eq!(code, "000001");
            }
            other => panic!("expected HighBelowLow, got {:?}", other),
        }
    }

    #[test]
    fn parse_index_klines_mixed_codes_errors() {
        let raw = r#"{
            "data":[
                {"symbol":"sh000001","time":"2026-01-02","open":"1","high":"2","low":"1","close":"2","volume":"1","amount":"1"},
                {"symbol":"sh000300","time":"2026-01-02","open":"1","high":"2","low":"1","close":"2","volume":"1","amount":"1"}
            ]
        }"#;
        let env: OpenStockEnvelope<IndexKlineRecord> = serde_json::from_str(raw).unwrap();
        match parse_index_klines(env) {
            Err(IndexKlineParseError::MixedCode { expected, actual }) => {
                assert_eq!(expected, "000001");
                assert_eq!(actual, "000300");
            }
            other => panic!("expected MixedCode, got {:?}", other),
        }
    }
}
