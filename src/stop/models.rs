use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopRule {
    pub code: String,
    pub stop_loss_price: Option<f64>,
    pub take_profit_price: Option<f64>,
    pub trailing_pct: Option<f64>,
    pub highest_price: Option<f64>,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopTriggerKind {
    Loss,
    Profit,
    TrailingLoss,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggeredStop {
    pub code: String,
    pub kind: StopTriggerKind,
    pub current_price: f64,
    pub threshold_price: f64,
    pub highest_price: Option<f64>,
    pub triggered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopEvaluationResult {
    pub updated_rule: StopRule,
    pub triggered_stop: Option<TriggeredStop>,
}
