use polars::prelude::*;

use crate::core::Result;
use crate::factor::loader::FactorDataLoader;
use crate::factor::types::FactorLoadRequest;

#[derive(Debug, Clone)]
pub struct FactorDataset {
    frame: DataFrame,
}

impl FactorDataset {
    pub async fn from_loader<L>(loader: &L, request: &FactorLoadRequest) -> Result<Self>
    where
        L: FactorDataLoader + ?Sized,
    {
        let frame = loader.load_bars(request).await?;
        let dataset = Self::new(frame)?;
        dataset.ensure_required_columns(&request.required_fields)?;
        Ok(dataset)
    }

    pub fn new(frame: DataFrame) -> Result<Self> {
        let frame = crate::factor::check::normalize_factor_frame(frame)?;
        let dataset = Self { frame };
        dataset.ensure_required_columns(&[])?;
        dataset.ensure_time_aligned()?;
        Ok(dataset)
    }

    pub fn frame(&self) -> &DataFrame {
        &self.frame
    }

    pub fn ensure_required_columns(&self, fields: &[String]) -> Result<()> {
        crate::factor::check::ensure_required_columns(&self.frame, fields)
    }

    pub fn ensure_time_aligned(&self) -> Result<()> {
        crate::factor::check::ensure_symbol_date_sorted(&self.frame)?;
        crate::factor::check::ensure_unique_symbol_date(&self.frame)
    }

    pub fn validate_no_lookahead_basic(&self) -> Result<()> {
        crate::factor::check::validate_no_lookahead_basic(&self.frame)
    }
}
