use async_trait::async_trait;
use polars::prelude::DataFrame;

use crate::core::Result;
use crate::factor::types::FactorLoadRequest;

#[async_trait]
pub trait FactorDataLoader: Send + Sync {
    async fn load_bars(&self, request: &FactorLoadRequest) -> Result<DataFrame>;
}
