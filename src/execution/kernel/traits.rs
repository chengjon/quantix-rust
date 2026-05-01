use async_trait::async_trait;

use crate::core::Result;
use crate::execution::models::{FillDeltaContext, FillDeltaResult, OrderIntent};

use super::RiskDecision;

#[async_trait]
pub trait RiskEvaluator: Send + Sync {
    async fn evaluate(&self, intent: OrderIntent) -> Result<RiskDecision>;

    async fn sync_after_fill(&self) -> Result<()>;
}

#[async_trait]
pub trait FillDeltaApplier: Send + Sync {
    async fn apply_fill_delta(&self, ctx: FillDeltaContext) -> Result<FillDeltaResult>;
}
