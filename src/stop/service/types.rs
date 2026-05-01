use crate::core::Result;
use crate::stop::models::{
    StopAnchorSource, StopEvalState, StopHistoryEvent, StopHistoryFilter, StopRule, TriggeredStop,
};
use async_trait::async_trait;

#[async_trait]
pub trait StopRuleStore: Send + Sync {
    async fn upsert_rule(&self, rule: StopRule) -> Result<StopRule>;

    async fn list_rules(&self) -> Result<Vec<StopRule>>;

    async fn get_rule(&self, code: &str) -> Result<Option<StopRule>>;

    async fn append_history(&self, event: StopHistoryEvent) -> Result<()>;

    async fn list_history(&self, filter: StopHistoryFilter) -> Result<Vec<StopHistoryEvent>>;

    async fn remove_rule(&self, code: &str) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub struct StopService<RS> {
    pub(super) store: RS,
}

#[derive(Debug, Clone)]
pub(super) struct EvaluatedRuleState {
    pub(super) updated_rule: StopRule,
    pub(super) triggered_stop: Option<TriggeredStop>,
    pub(super) anchor_price: Option<f64>,
    pub(super) anchor_source: Option<StopAnchorSource>,
    pub(super) loss_threshold: Option<f64>,
    pub(super) profit_threshold: Option<f64>,
    pub(super) eval_state: StopEvalState,
}
