use async_trait::async_trait;

use crate::core::Result;
use crate::execution::models::{FillDeltaContext, FillDeltaResult};

use super::FillDeltaApplier;

#[derive(Debug, Clone, Copy, Default)]
pub struct NoopFillDeltaApplier;

#[async_trait]
impl FillDeltaApplier for NoopFillDeltaApplier {
    async fn apply_fill_delta(&self, ctx: FillDeltaContext) -> Result<FillDeltaResult> {
        let delta_quantity = (ctx.new_filled_quantity - ctx.old_filled_quantity).max(0);
        Ok(FillDeltaResult {
            applied: delta_quantity > 0,
            delta_quantity,
            trade_record_id: None,
        })
    }
}
