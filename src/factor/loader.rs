use async_trait::async_trait;
use polars::prelude::*;
use std::path::{Path, PathBuf};

use crate::core::{QuantixError, Result};
use crate::factor::types::FactorLoadRequest;

#[async_trait]
pub trait FactorDataLoader: Send + Sync {
    async fn load_bars(&self, request: &FactorLoadRequest) -> Result<DataFrame>;
}

#[derive(Debug, Clone)]
pub struct CsvFactorDataLoader {
    path: PathBuf,
}

impl CsvFactorDataLoader {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

#[async_trait]
impl FactorDataLoader for CsvFactorDataLoader {
    async fn load_bars(&self, request: &FactorLoadRequest) -> Result<DataFrame> {
        let start = request.start.format("%Y-%m-%d").to_string();
        let end = request.end.format("%Y-%m-%d").to_string();
        let symbol_filter = request
            .symbols
            .iter()
            .map(|symbol| col("symbol").eq(lit(symbol.clone())))
            .reduce(|acc, expr| acc.or(expr))
            .ok_or_else(|| {
                QuantixError::DataParse("factor csv load requires at least one symbol".to_string())
            })?;

        CsvReadOptions::default()
            .with_has_header(true)
            .try_into_reader_with_file_path(Some(self.path.clone()))
            .and_then(|reader| reader.finish())
            .and_then(|frame| {
                frame
                    .lazy()
                    .filter(
                        symbol_filter
                            .and(col("date").gt_eq(lit(start)))
                            .and(col("date").lt_eq(lit(end))),
                    )
                    .collect()
            })
            .map_err(|e| {
                QuantixError::DataParse(format!(
                    "factor csv load failed `{}`: {}",
                    self.path.display(),
                    e
                ))
            })
    }
}
