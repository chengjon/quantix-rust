use polars::prelude::*;

use crate::core::{QuantixError, Result};

const BASE_COLUMNS: &[&str] = &["date", "symbol", "open", "high", "low", "close", "volume"];

pub fn normalize_factor_frame(mut frame: DataFrame) -> Result<DataFrame> {
    ensure_base_columns(&frame)?;
    cast_required_types(&mut frame)?;
    frame
        .sort(["symbol", "date"], Default::default())
        .map_err(|e| QuantixError::DataParse(format!("factor dataset sort failed: {}", e)))
}

pub fn ensure_required_columns(frame: &DataFrame, fields: &[String]) -> Result<()> {
    ensure_base_columns(frame)?;
    for field in fields {
        if frame.column(field).is_err() {
            return Err(QuantixError::DataParse(format!(
                "factor dataset missing required field `{}`",
                field
            )));
        }
    }
    Ok(())
}

pub fn ensure_unique_symbol_date(frame: &DataFrame) -> Result<()> {
    let subset = vec!["symbol".to_string(), "date".to_string()];
    let unique = frame
        .unique_stable(Some(&subset), UniqueKeepStrategy::First, None)
        .map_err(|e| QuantixError::DataParse(format!("factor uniqueness check failed: {}", e)))?;
    if unique.height() != frame.height() {
        return Err(QuantixError::DataParse(
            "factor dataset contains duplicate (symbol, date) rows".to_string(),
        ));
    }
    Ok(())
}

pub fn ensure_symbol_date_sorted(frame: &DataFrame) -> Result<()> {
    let symbols = frame.column("symbol").map_err(|e| {
        QuantixError::DataParse(format!("factor symbol column check failed: {}", e))
    })?;
    let dates = frame
        .column("date")
        .map_err(|e| QuantixError::DataParse(format!("factor date column check failed: {}", e)))?;

    let mut previous: Option<(String, String)> = None;
    for idx in 0..frame.height() {
        let current = (
            symbols
                .get(idx)
                .map_err(|e| QuantixError::DataParse(format!("factor symbol read failed: {}", e)))?
                .to_string(),
            dates
                .get(idx)
                .map_err(|e| QuantixError::DataParse(format!("factor date read failed: {}", e)))?
                .to_string(),
        );
        if let Some(prev) = &previous {
            if current < *prev {
                return Err(QuantixError::DataParse(
                    "factor dataset must be sorted by symbol,date ascending".to_string(),
                ));
            }
        }
        previous = Some(current);
    }
    Ok(())
}

pub fn validate_no_lookahead_basic(frame: &DataFrame) -> Result<()> {
    ensure_symbol_date_sorted(frame)?;
    ensure_unique_symbol_date(frame)
}

fn ensure_base_columns(frame: &DataFrame) -> Result<()> {
    for column in BASE_COLUMNS {
        if frame.column(column).is_err() {
            return Err(QuantixError::DataParse(format!(
                "factor dataset missing base column `{}`",
                column
            )));
        }
    }
    Ok(())
}

fn cast_required_types(frame: &mut DataFrame) -> Result<()> {
    cast_column(frame, "date", &DataType::Date)?;
    cast_column(frame, "symbol", &DataType::String)?;
    for column in ["open", "high", "low", "close"] {
        cast_column(frame, column, &DataType::Float64)?;
    }
    cast_column(frame, "volume", &DataType::Int64)?;
    if frame.column("amount").is_ok() {
        cast_column(frame, "amount", &DataType::Float64)?;
    }
    Ok(())
}

fn cast_column(frame: &mut DataFrame, column: &str, dtype: &DataType) -> Result<()> {
    let casted = frame
        .column(column)
        .and_then(|s| s.cast(dtype))
        .map_err(|e| {
            QuantixError::DataParse(format!("factor column `{}` cast failed: {}", column, e))
        })?;
    frame.replace(column, casted).map_err(|e| {
        QuantixError::DataParse(format!("factor column `{}` replace failed: {}", column, e))
    })?;
    Ok(())
}
