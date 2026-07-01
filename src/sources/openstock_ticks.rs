//! Parser for the OpenStock `TICK_DATA` category.
//!
//! Field shape live-verified 2026-07-01 against
//! `http://192.168.123.104:8040` (see design.md D4.1):
//!
//! ```text
//! data: [{
//!   meta: { symbol, trading_date, returned_count, price_base, has_more, ... },
//!   ticks: [
//!     { trade_datetime, price, volume, amount, side, order_count, status, ... }
//!   ]
//! }]
//! ```
//!
//! The outer `data` array wraps a single envelope-record (per source).
//! `parse_tick_data` flattens the inner `ticks` into a `Vec<Tick>`,
//! dropping unknown fields and the `status` byte (semantics unknown).

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use thiserror::Error;

use crate::core::QuantixError;
use crate::data::models::{Tick, TradeDirection};
use crate::sources::openstock::normalize_symbol;
use crate::sources::openstock_envelope::OpenStockEnvelope;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TickParseError {
    #[error("invalid OpenStock tick JSON: {0}")]
    InvalidJson(String),
    #[error("OpenStock tick payload contains no envelope-records")]
    EmptyRecords,
    #[error("OpenStock tick envelope-record has no meta")]
    MissingMeta,
    #[error("missing required OpenStock tick field: {0}")]
    MissingField(&'static str),
    #[error("invalid OpenStock tick decimal field `{field}`: {value}")]
    InvalidDecimal { field: &'static str, value: String },
    #[error("invalid OpenStock tick volume: {0}")]
    InvalidVolume(String),
    #[error("invalid OpenStock tick datetime `{value}`, expected {expected}")]
    InvalidDatetime {
        value: String,
        expected: &'static str,
    },
    #[error("OpenStock tick payload mixes symbols: expected {expected}, got {actual}")]
    MixedSymbol { expected: String, actual: String },
}

pub fn tick_error_into_quantix(error: TickParseError) -> QuantixError {
    QuantixError::DataParse(error.to_string())
}

/// One envelope-record from the `data` array. The runtime returns
/// exactly one such record per request; we accept more but require
/// every `meta.symbol` to agree.
#[derive(Debug, Deserialize)]
pub struct TickEnvelopeRecord {
    #[serde(default)]
    pub meta: Option<TickMeta>,
    #[serde(default)]
    pub ticks: Vec<TickEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TickMeta {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub trading_date: Option<String>,
    #[serde(default)]
    pub returned_count: Option<serde_json::Value>,
    #[serde(default)]
    pub price_base: Option<serde_json::Value>,
    #[serde(default)]
    pub has_more: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TickEntry {
    #[serde(default)]
    pub trade_datetime: Option<String>,
    #[serde(default)]
    pub price: Option<serde_json::Value>,
    #[serde(default)]
    pub volume: Option<serde_json::Value>,
    #[serde(default)]
    pub amount: Option<serde_json::Value>,
    #[serde(default)]
    pub side: Option<String>,
}

/// Parse a TICK_DATA envelope into a flat tick list paired with the
/// canonical stock code (prefix-stripped) and the reported trading date.
pub fn parse_tick_data(
    envelope: OpenStockEnvelope<TickEnvelopeRecord>,
) -> Result<(TickMeta, Vec<Tick>), TickParseError> {
    if envelope.is_empty() {
        return Err(TickParseError::EmptyRecords);
    }
    let mut canonical_code: Option<String> = None;
    let mut canonical_meta: Option<TickMeta> = None;
    let mut out: Vec<Tick> = Vec::new();
    for record in envelope.data {
        let meta = record.meta.ok_or(TickParseError::MissingMeta)?;
        let raw_symbol = meta
            .symbol
            .as_ref()
            .filter(|text| !text.trim().is_empty())
            .ok_or(TickParseError::MissingField("meta.symbol"))?;
        let code = normalize_symbol(raw_symbol);
        match &canonical_code {
            Some(expected) if expected != &code => {
                return Err(TickParseError::MixedSymbol {
                    expected: expected.clone(),
                    actual: code,
                });
            }
            Some(_) => {}
            None => {
                canonical_code = Some(code.clone());
                canonical_meta = Some(meta.clone());
            }
        }
        for entry in record.ticks {
            let tick = map_tick_entry(entry, &code)?;
            out.push(tick);
        }
    }
    Ok((canonical_meta.ok_or(TickParseError::MissingMeta)?, out))
}

fn map_tick_entry(entry: TickEntry, code: &str) -> Result<Tick, TickParseError> {
    let dt_text = entry
        .trade_datetime
        .as_ref()
        .filter(|text| !text.trim().is_empty())
        .ok_or(TickParseError::MissingField("ticks[].trade_datetime"))?;
    let timestamp = parse_tick_datetime(dt_text).map_err(|_| TickParseError::InvalidDatetime {
        value: dt_text.clone(),
        expected: "%Y-%m-%dT%H:%M:%S or RFC3339",
    })?;

    let price = parse_decimal(entry.price, "ticks[].price")?;
    let volume = parse_volume(entry.volume)?;
    let amount = parse_decimal(entry.amount, "ticks[].amount")?;
    let direction = match entry.side.as_deref() {
        Some("buy") => TradeDirection::Buy,
        Some("sell") => TradeDirection::Sell,
        _ => TradeDirection::Neutral,
    };

    Ok(Tick {
        code: code.to_string(),
        timestamp,
        price,
        volume,
        amount,
        direction,
    })
}

fn parse_tick_datetime(text: &str) -> Result<NaiveDateTime, ()> {
    NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f"))
        .map_err(|_| ())
}

fn parse_decimal(
    value: Option<serde_json::Value>,
    field: &'static str,
) -> Result<Decimal, TickParseError> {
    let value = value.ok_or(TickParseError::MissingField(field))?;
    let text = match value {
        serde_json::Value::String(text) => text,
        serde_json::Value::Number(number) => number.to_string(),
        other => {
            return Err(TickParseError::InvalidDecimal {
                field,
                value: other.to_string(),
            });
        }
    };
    Decimal::from_str(&text).map_err(|_| TickParseError::InvalidDecimal { field, value: text })
}

fn parse_volume(value: Option<serde_json::Value>) -> Result<i64, TickParseError> {
    let value = value.ok_or(TickParseError::MissingField("ticks[].volume"))?;
    match value {
        serde_json::Value::Number(number) => number
            .as_i64()
            .ok_or_else(|| TickParseError::InvalidVolume(number.to_string())),
        serde_json::Value::String(text) => text
            .parse::<i64>()
            .map_err(|_| TickParseError::InvalidVolume(text)),
        other => Err(TickParseError::InvalidVolume(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HAPPY_BODY: &str = r#"{
        "data": [
            {
                "meta": {
                    "symbol": "sh600000",
                    "trading_date": "2026-06-30",
                    "returned_count": 2,
                    "price_base": 10.0,
                    "has_more": false
                },
                "ticks": [
                    {"trade_datetime":"2026-06-30T09:30:00","price":10.00,"volume":100,"amount":1000.0,"side":"buy"},
                    {"trade_datetime":"2026-06-30T09:30:03","price":10.01,"volume":200,"amount":2002.0,"side":"sell"}
                ]
            }
        ],
        "source":"eltdx"
    }"#;

    #[test]
    fn parse_tick_data_happy() {
        let env: OpenStockEnvelope<TickEnvelopeRecord> = serde_json::from_str(HAPPY_BODY).unwrap();
        let (meta, ticks) = parse_tick_data(env).unwrap();
        assert_eq!(meta.symbol.as_deref(), Some("sh600000"));
        assert_eq!(meta.trading_date.as_deref(), Some("2026-06-30"));
        assert_eq!(ticks.len(), 2);
        assert_eq!(ticks[0].code, "600000");
        assert_eq!(ticks[0].timestamp.to_string(), "2026-06-30 09:30:00");
        assert_eq!(ticks[0].price, Decimal::new(1000, 2));
        assert_eq!(ticks[0].volume, 100);
        assert!(matches!(ticks[0].direction, TradeDirection::Buy));
        assert!(matches!(ticks[1].direction, TradeDirection::Sell));
    }

    #[test]
    fn parse_tick_data_empty_errors() {
        let raw = r#"{"data": []}"#;
        let env: OpenStockEnvelope<TickEnvelopeRecord> = serde_json::from_str(raw).unwrap();
        match parse_tick_data(env) {
            Err(TickParseError::EmptyRecords) => {}
            other => panic!("expected EmptyRecords, got {:?}", other),
        }
    }

    #[test]
    fn parse_tick_data_missing_meta_errors() {
        let raw = r#"{"data": [ { "ticks": [] } ]}"#;
        let env: OpenStockEnvelope<TickEnvelopeRecord> = serde_json::from_str(raw).unwrap();
        match parse_tick_data(env) {
            Err(TickParseError::MissingMeta) => {}
            other => panic!("expected MissingMeta, got {:?}", other),
        }
    }

    #[test]
    fn parse_tick_data_missing_trade_datetime_errors() {
        let raw = r#"{
            "data": [{
                "meta": {"symbol":"sh600000"},
                "ticks": [ {"price":"10","volume":"1","amount":"10","side":"buy"} ]
            }]
        }"#;
        let env: OpenStockEnvelope<TickEnvelopeRecord> = serde_json::from_str(raw).unwrap();
        match parse_tick_data(env) {
            Err(TickParseError::MissingField("ticks[].trade_datetime")) => {}
            other => panic!("expected MissingField trade_datetime, got {:?}", other),
        }
    }

    #[test]
    fn parse_tick_data_string_numerics_ok() {
        // Mirrors the IndexKlineRecord shape-drift lesson: numbers may arrive as strings.
        let raw = r#"{
            "data": [{
                "meta": {"symbol":"sh600000"},
                "ticks": [
                    {"trade_datetime":"2026-06-30T09:30:00","price":"10.50","volume":"150","amount":"1575.00","side":"buy"}
                ]
            }]
        }"#;
        let env: OpenStockEnvelope<TickEnvelopeRecord> = serde_json::from_str(raw).unwrap();
        let (_meta, ticks) = parse_tick_data(env).unwrap();
        assert_eq!(ticks.len(), 1);
        assert_eq!(ticks[0].price, Decimal::new(1050, 2));
        assert_eq!(ticks[0].volume, 150);
    }

    #[test]
    fn parse_tick_data_invalid_decimal_errors() {
        let raw = r#"{
            "data": [{
                "meta": {"symbol":"sh600000"},
                "ticks": [
                    {"trade_datetime":"2026-06-30T09:30:00","price":"not-a-number","volume":1,"amount":1,"side":"buy"}
                ]
            }]
        }"#;
        let env: OpenStockEnvelope<TickEnvelopeRecord> = serde_json::from_str(raw).unwrap();
        match parse_tick_data(env) {
            Err(TickParseError::InvalidDecimal { field, .. }) => {
                assert_eq!(field, "ticks[].price");
            }
            other => panic!("expected InvalidDecimal price, got {:?}", other),
        }
    }

    #[test]
    fn parse_tick_data_unknown_side_defaults_neutral() {
        let raw = r#"{
            "data": [{
                "meta": {"symbol":"sh600000"},
                "ticks": [
                    {"trade_datetime":"2026-06-30T09:30:00","price":10.0,"volume":1,"amount":10.0,"side":"unknown"}
                ]
            }]
        }"#;
        let env: OpenStockEnvelope<TickEnvelopeRecord> = serde_json::from_str(raw).unwrap();
        let (_meta, ticks) = parse_tick_data(env).unwrap();
        assert!(matches!(ticks[0].direction, TradeDirection::Neutral));
    }

    #[test]
    fn parse_tick_data_mixed_symbol_errors() {
        let raw = r#"{
            "data": [
                {"meta":{"symbol":"sh600000"},"ticks":[]},
                {"meta":{"symbol":"sh600001"},"ticks":[]}
            ]
        }"#;
        let env: OpenStockEnvelope<TickEnvelopeRecord> = serde_json::from_str(raw).unwrap();
        match parse_tick_data(env) {
            Err(TickParseError::MixedSymbol { expected, actual }) => {
                assert_eq!(expected, "600000");
                assert_eq!(actual, "600001");
            }
            other => panic!("expected MixedSymbol, got {:?}", other),
        }
    }
}
