use polars::prelude::{CsvWriter, SerWriter};

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
