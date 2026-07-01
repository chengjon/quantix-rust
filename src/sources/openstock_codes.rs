//! Parsers for OpenStock codes categories `STOCK_CODES` and `ALL_STOCKS`.
//!
//! `STOCK_CODES` returns the minimal `{code, name}` pair.
//! `ALL_STOCKS` is a richer shape that also carries `market` (sh/sz/bj)
//! and `listing_date`. Both tolerate extra provider fields via
//! `#[serde(flatten)] extra` catch-all (per the rule: do not hardcode
//! field names beyond the minimal parsing set).

use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

use crate::core::QuantixError;
use crate::sources::openstock::normalize_symbol;
use crate::sources::openstock_calendar::parse_calendar_date;
use crate::sources::openstock_envelope::OpenStockEnvelope;

/// Codes-parse error family.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StockCodeParseError {
    #[error("invalid OpenStock codes JSON: {0}")]
    InvalidJson(String),
    #[error("OpenStock codes payload contains no records")]
    EmptyRecords,
    #[error("missing required OpenStock codes field: {0}")]
    MissingField(&'static str),
    #[error("invalid OpenStock code `{value}`")]
    InvalidCode { value: String },
}

/// Bridge used by CLI handlers.
pub fn stock_code_error_into_quantix(error: StockCodeParseError) -> QuantixError {
    QuantixError::DataParse(error.to_string())
}

/// Raw record shape for `STOCK_CODES` payloads.
#[derive(Debug, Deserialize)]
pub struct StockCodeRecord {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Raw record shape for `ALL_STOCKS` payloads.
///
/// Runtime (baostock provider) returns `{code:"sh.000001",
/// code_name:"上证综合指数", tradeStatus:"1"}`. We use
/// `#[serde(rename_all = "camelCase")]` so `trade_status` matches the
/// runtime's `tradeStatus` automatically. `name` carries both
/// `code_name` (snake, identical in camel) and the legacy `name` field
/// via alias for back-compat with P0.9 fixtures. `market` and
/// `listing_date` remain `Option` — runtime does not populate them
/// today, but keeping the fields means the parser is forward-compatible
/// if baostock adds them.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockListRecord {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default, alias = "code_name")]
    pub name: Option<String>,
    #[serde(default)]
    pub market: Option<String>,
    // `rename_all = "camelCase"` renames this to `listingDate`; add an
    // explicit alias so the snake_case `listing_date` shape from
    // legacy fixtures and `STOCK_BASIC` still deserializes.
    #[serde(default, alias = "listing_date")]
    pub listing_date: Option<String>,
    /// Runtime emits `tradeStatus:"1"|"0"` (1=正常交易, 0=停牌/退市).
    /// Distinct from `listing_date` — never alias the two.
    #[serde(default)]
    pub trade_status: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Parsed entry from `STOCK_CODES`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockCode {
    pub code: String,
    pub name: Option<String>,
}

/// Parsed entry from `ALL_STOCKS`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockListEntry {
    pub code: String,
    pub name: Option<String>,
    pub market: Option<String>,
    pub listing_date: Option<NaiveDate>,
    pub trade_status: Option<String>,
}

/// Parse the `STOCK_CODES` category envelope.
pub fn parse_stock_codes(
    envelope: OpenStockEnvelope<StockCodeRecord>,
) -> Result<Vec<StockCode>, StockCodeParseError> {
    if envelope.is_empty() {
        return Err(StockCodeParseError::EmptyRecords);
    }
    envelope
        .data
        .into_iter()
        .map(|record| {
            let code = normalize_code(record.code)?;
            Ok(StockCode {
                code,
                name: record.name.filter(|text| !text.trim().is_empty()),
            })
        })
        .collect()
}

/// Parse the `ALL_STOCKS` category envelope.
pub fn parse_all_stocks(
    envelope: OpenStockEnvelope<StockListRecord>,
) -> Result<Vec<StockListEntry>, StockCodeParseError> {
    if envelope.is_empty() {
        return Err(StockCodeParseError::EmptyRecords);
    }
    envelope
        .data
        .into_iter()
        .map(|record| {
            let code = normalize_code(record.code)?;
            let listing_date = match record.listing_date {
                Some(text) if !text.trim().is_empty() => Some(parse_listing_date(&text)?),
                _ => None,
            };
            Ok(StockListEntry {
                code,
                name: record.name.filter(|text| !text.trim().is_empty()),
                market: record.market.filter(|text| !text.trim().is_empty()),
                listing_date,
                trade_status: record.trade_status.filter(|text| !text.trim().is_empty()),
            })
        })
        .collect()
}

/// Validate and normalize a stock code.
///
/// Accepts both bare (`"600000"`, `"000001"`) and prefixed
/// (`"sh.000001"`, `"sz000001"`, `"bj920193"`) shapes. The prefix
/// variants are what baostock's `ALL_STOCKS` actually returns; the
/// bare variants are what eltdx's `STOCK_CODES` returns. Prefixes
/// (`sh`/`sz`/`bj`) and the separator (`.`, or none) are stripped
/// before the digit-only + length check.
fn normalize_code(raw: Option<String>) -> Result<String, StockCodeParseError> {
    let text = raw
        .filter(|text| !text.trim().is_empty())
        .ok_or(StockCodeParseError::MissingField("code"))?;
    let trimmed = text.trim();
    let stripped = normalize_symbol(trimmed);
    // baostock emits `sh.000001` — normalize_symbol strips `sh` but
    // keeps the `.`, so trim a single leading separator here.
    let stripped = stripped
        .strip_prefix('.')
        .map(|s| s.to_string())
        .unwrap_or(stripped);
    if !stripped.chars().all(|c| c.is_ascii_digit()) {
        return Err(StockCodeParseError::InvalidCode {
            value: text.clone(),
        });
    }
    if !(4..=8).contains(&stripped.len()) {
        return Err(StockCodeParseError::InvalidCode {
            value: text.clone(),
        });
    }
    Ok(stripped.to_string())
}

/// Parse `ALL_STOCKS` `listing_date` field. Mirrors the calendar date
/// helper but reports errors in the codes-parse error family.
fn parse_listing_date(value: &str) -> Result<NaiveDate, StockCodeParseError> {
    parse_calendar_date(value).map_err(|_| StockCodeParseError::InvalidCode {
        value: value.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stock_codes_happy() {
        let raw = r#"{
            "data": [
                {"code": "600000", "name": "浦发银行"},
                {"code": "000001", "name": "平安银行"}
            ],
            "source": "eltdx"
        }"#;
        let env: OpenStockEnvelope<StockCodeRecord> = serde_json::from_str(raw).unwrap();
        let codes = parse_stock_codes(env).unwrap();
        assert_eq!(codes.len(), 2);
        assert_eq!(codes[0].code, "600000");
        assert_eq!(codes[0].name.as_deref(), Some("浦发银行"));
    }

    #[test]
    fn parse_stock_codes_empty_errors() {
        let raw = r#"{"data": []}"#;
        let env: OpenStockEnvelope<StockCodeRecord> = serde_json::from_str(raw).unwrap();
        assert_eq!(
            parse_stock_codes(env),
            Err(StockCodeParseError::EmptyRecords)
        );
    }

    #[test]
    fn parse_stock_codes_missing_code_errors() {
        let raw = r#"{"data": [{"name": "missing-code"}]}"#;
        let env: OpenStockEnvelope<StockCodeRecord> = serde_json::from_str(raw).unwrap();
        assert_eq!(
            parse_stock_codes(env),
            Err(StockCodeParseError::MissingField("code"))
        );
    }

    #[test]
    fn parse_stock_codes_non_numeric_code_errors() {
        // "XYZ" has no recognized exchange prefix → normalize_symbol
        // returns it unchanged → fails the digit-only check.
        let raw = r#"{"data": [{"code": "XYZ", "name": "x"}]}"#;
        let env: OpenStockEnvelope<StockCodeRecord> = serde_json::from_str(raw).unwrap();
        match parse_stock_codes(env) {
            Err(StockCodeParseError::InvalidCode { value }) => {
                assert_eq!(value, "XYZ");
            }
            other => panic!("expected InvalidCode, got {:?}", other),
        }
    }

    #[test]
    fn parse_stock_codes_strips_exchange_prefix() {
        // eltdx STOCK_CODES returns prefixed shapes like "sh689009".
        let raw = r#"{
            "data": [
                {"code": "sh689009", "symbol": "sh689009", "market": "a_share"},
                {"code": "bj920193", "symbol": "bj920193", "market": "a_share"}
            ],
            "source": "eltdx"
        }"#;
        let env: OpenStockEnvelope<StockCodeRecord> = serde_json::from_str(raw).unwrap();
        let codes = parse_stock_codes(env).unwrap();
        assert_eq!(codes.len(), 2);
        assert_eq!(codes[0].code, "689009");
        assert_eq!(codes[1].code, "920193");
    }

    #[test]
    fn parse_all_stocks_happy_with_market_and_listing_date() {
        let raw = r#"{
            "data": [
                {"code": "600000", "name": "浦发银行", "market": "sh", "listing_date": "1999-11-10"},
                {"code": "000001", "name": "平安银行", "market": "sz", "listing_date": "19910403"}
            ],
            "source": "eltdx"
        }"#;
        let env: OpenStockEnvelope<StockListRecord> = serde_json::from_str(raw).unwrap();
        let entries = parse_all_stocks(env).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].market.as_deref(), Some("sh"));
        assert_eq!(entries[0].listing_date.unwrap().to_string(), "1999-11-10");
        assert_eq!(entries[1].listing_date.unwrap().to_string(), "1991-04-03");
    }

    #[test]
    fn parse_all_stocks_tolerates_optional_market_and_listing_date() {
        let raw = r#"{"data": [{"code": "600000", "name": "浦发银行"}]}"#;
        let env: OpenStockEnvelope<StockListRecord> = serde_json::from_str(raw).unwrap();
        let entries = parse_all_stocks(env).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].market.is_none());
        assert!(entries[0].listing_date.is_none());
    }

    #[test]
    fn parse_all_stocks_tolerates_extra_fields() {
        let raw = r#"{
            "data": [{"code": "600000", "name": "浦发银行", "delisted": false, "vendor_id": 42}]
        }"#;
        let env: OpenStockEnvelope<StockListRecord> = serde_json::from_str(raw).unwrap();
        let entries = parse_all_stocks(env).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].code, "600000");
    }

    /// Real runtime (baostock) shape, captured during live integration on
    /// 2026-07-01. Records use prefixed codes (`"sh.000001"`),
    /// `code_name` (snake, not camelCase), and `tradeStatus` (camelCase).
    /// Verifies the `#[serde(rename_all = "camelCase")]` + `alias =
    /// "code_name"` + `normalize_symbol` chain handles this end-to-end.
    #[test]
    fn parse_all_stocks_runtime_shape_from_baostock() {
        let raw = r#"{
            "data": [
                {"code": "sh.000001", "tradeStatus": "1", "code_name": "上证综合指数"},
                {"code": "sh.600000", "tradeStatus": "1", "code_name": "浦发银行"}
            ],
            "source": "baostock",
            "quality_flags": ["fallback_day:2021-05-14"]
        }"#;
        let env: OpenStockEnvelope<StockListRecord> = serde_json::from_str(raw).unwrap();
        let entries = parse_all_stocks(env).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].code, "000001");
        assert_eq!(entries[0].name.as_deref(), Some("上证综合指数"));
        assert_eq!(entries[0].trade_status.as_deref(), Some("1"));
        assert!(
            entries[0].market.is_none(),
            "runtime does not populate market today; field stays Option for forward-compat"
        );
        assert!(
            entries[0].listing_date.is_none(),
            "tradeStatus must NOT be aliased into listing_date (semantic mismatch)"
        );
        assert_eq!(entries[1].code, "600000");
    }

    #[test]
    fn stock_code_error_into_quantix_preserves_message() {
        let error = StockCodeParseError::EmptyRecords;
        let quantix = stock_code_error_into_quantix(error);
        let text = quantix.to_string();
        assert!(text.contains("no records"));
    }
}
