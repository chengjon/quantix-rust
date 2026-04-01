pub mod models;
pub mod service;
pub mod storage;
mod service_eval;

pub use models::{
    StopAnchorSource, StopEvaluationResult, StopEvalState, StopHistoryEvent,
    StopHistoryEventType, StopHistoryFilter, StopHistoryTriggerKind, StopRule, StopRuleUpdate,
    StopStatusRow, StopTriggerKind, TriggeredStop,
};
pub use service::{StopRuleStore, StopService};
pub use storage::SqliteStopRuleStore;

pub type SqliteStopHistoryEvent = StopHistoryEvent;
pub type SqliteStopHistoryFilter = StopHistoryFilter;
pub type SqliteStopHistoryTriggerKind = StopHistoryTriggerKind;
