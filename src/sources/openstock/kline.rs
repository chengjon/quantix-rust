//! Daily-kline JSON parsing lane for the OpenStock `/data/bars` fixture shape.
//!
//! [`parse_daily_kline_json`] deserializes a `{ period, adjust_type, records }`
//! payload into a canonical [`Kline`] vector, validating every field along
//! the way. Helper primitives live in [`super::parse`].

use super::parse::{parse_date, parse_decimal, parse_volume, required_string, required_value};

use crate::data::models::{AdjustType, Kline};
use serde::Deserialize;
use thiserror::Error;

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
        high: rust_decimal::Decimal,
        low: rust_decimal::Decimal,
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
