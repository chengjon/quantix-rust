use std::str::FromStr;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use thiserror::Error;

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
