pub mod models;
pub mod service;
pub mod storage;

pub use models::{
    StopAnchorSource, StopEvalState, StopEvaluationResult, StopHistoryEvent, StopHistoryEventType,
    StopHistoryFilter, StopHistoryTriggerKind, StopRule, StopRuleUpdate, StopStatusRow,
    StopTriggerKind, TriggeredStop,
};
pub use service::{StopRuleStore, StopService};
pub use storage::SqliteStopRuleStore;

/// 兼容别名：SQLite 存储层使用的 StopHistoryEvent 类型名（与 StopHistoryEvent 同型，保留以避免下游 rename）。
pub type SqliteStopHistoryEvent = StopHistoryEvent;
/// 兼容别名：SQLite 存储层使用的 StopHistoryFilter 类型名。
pub type SqliteStopHistoryFilter = StopHistoryFilter;
/// 兼容别名：SQLite 存储层使用的 StopHistoryTriggerKind 类型名。
pub type SqliteStopHistoryTriggerKind = StopHistoryTriggerKind;
