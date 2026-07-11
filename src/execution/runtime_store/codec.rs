use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::core::signal::Signal;
use crate::core::{QuantixError, Result};

pub(crate) fn parse_signal_value(value: &str) -> Option<Signal> {
    match value {
        "buy" => Some(Signal::Buy),
        "sell" => Some(Signal::Sell),
        "hold" => Some(Signal::Hold),
        _ => None,
    }
}

pub(crate) fn parse_timestamp(value: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|ts| ts.with_timezone(&Utc))
        .map_err(|err| QuantixError::DataParse(format!("invalid RFC3339 timestamp {value}: {err}")))
}

pub(crate) fn parse_decimal(value: &str) -> Result<Decimal> {
    Decimal::from_str_exact(value)
        .map_err(|err| QuantixError::DataParse(format!("invalid decimal {value}: {err}")))
}
