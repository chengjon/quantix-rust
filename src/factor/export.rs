use polars::prelude::{CsvWriter, ParquetWriter, SerWriter};
use std::fs::File;
use std::path::Path;

use crate::core::{QuantixError, Result};
use crate::factor::types::FactorComputeResult;

pub fn factor_result_to_csv_string(result: &FactorComputeResult) -> Result<String> {
    let mut bytes = Vec::new();
    CsvWriter::new(&mut bytes)
        .finish(&mut result.frame.clone())
        .map_err(|e| QuantixError::Other(format!("factor csv export failed: {}", e)))?;
    String::from_utf8(bytes)
        .map_err(|e| QuantixError::Other(format!("factor csv export produced invalid utf8: {}", e)))
}

pub fn factor_result_to_json_string(result: &FactorComputeResult) -> Result<String> {
    let rows = result.frame.height();
    Ok(format!(
        "{{\"factor_id\":\"{}\",\"rows\":{},\"columns\":{:?}}}",
        result.factor_id,
        rows,
        result.frame.get_column_names()
    ))
}

pub fn factor_result_to_parquet_file(
    result: &FactorComputeResult,
    path: impl AsRef<Path>,
) -> Result<()> {
    let file = File::create(path.as_ref())
        .map_err(|e| QuantixError::Other(format!("factor parquet create failed: {}", e)))?;
    let mut frame = result.frame.clone();
    ParquetWriter::new(file)
        .finish(&mut frame)
        .map_err(|e| QuantixError::Other(format!("factor parquet export failed: {}", e)))?;
    Ok(())
}
