use polars::prelude::*;

use crate::core::{QuantixError, Result};

const BASE_COLUMNS: &[&str] = &["date", "symbol", "open", "high", "low", "close", "volume"];

/// 规范化因子数据集：校验 base 列存在（date/symbol/open/high/low/close/volume），将 date→Date、symbol→String、ohl c→Float64、volume→Int64（amount 存在时→Float64），最后按 (symbol, date) 升序排序返回新 frame；列缺失、cast 或 sort 失败返回 DataParse。
pub fn normalize_factor_frame(mut frame: DataFrame) -> Result<DataFrame> {
    ensure_base_columns(&frame)?;
    cast_required_types(&mut frame)?;
    frame
        .sort(["symbol", "date"], Default::default())
        .map_err(|e| QuantixError::DataParse(format!("factor dataset sort failed: {}", e)))
}

/// 校验 frame 含 base 列以及额外指定的 fields 列；任一缺失返回带列名的 DataParse 错误。
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

/// 校验 (symbol, date) 在 frame 中唯一：通过 unique_stable 比对行数，若存在重复返回 DataParse 错误；unique 操作本身失败也透传。
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

/// 校验 frame 按 (symbol, date) 字典序升序排列：逐行与前一行比较，发现逆序或读取失败返回 DataParse 错误，用于防止时序算子产生错位。
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
        if let Some(prev) = &previous
            && current < *prev
        {
            return Err(QuantixError::DataParse(
                "factor dataset must be sorted by symbol,date ascending".to_string(),
            ));
        }
        previous = Some(current);
    }
    Ok(())
}

/// 基础无未来信息校验：先确保 (symbol, date) 升序、再确保 (symbol, date) 唯一；任一失败返回 DataParse 错误。该检查只覆盖排序与唯一性，不校验因子计算本身是否引用了未来行。
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
