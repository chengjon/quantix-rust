#![allow(clippy::should_implement_trait)]

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopRule {
    pub code: String,
    pub stop_loss_price: Option<f64>,
    pub take_profit_price: Option<f64>,
    pub stop_loss_pct: Option<f64>,
    pub take_profit_pct: Option<f64>,
    pub trailing_pct: Option<f64>,
    pub highest_price: Option<f64>,
    pub reference_price: Option<f64>,
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
    pub anchor_price: Option<f64>,
    pub anchor_source: Option<StopAnchorSource>,
    pub triggered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopEvaluationResult {
    pub updated_rule: StopRule,
    pub triggered_stop: Option<TriggeredStop>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopHistoryEventType {
    Set,
    Update,
    Remove,
    Trigger,
}

impl StopHistoryEventType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Set => "set",
            Self::Update => "update",
            Self::Remove => "remove",
            Self::Trigger => "trigger",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "set" => Some(Self::Set),
            "update" => Some(Self::Update),
            "remove" => Some(Self::Remove),
            "trigger" => Some(Self::Trigger),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopHistoryTriggerKind {
    Loss,
    Profit,
    Trailing,
}

impl StopHistoryTriggerKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Loss => "loss",
            Self::Profit => "profit",
            Self::Trailing => "trailing",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "loss" => Some(Self::Loss),
            "profit" => Some(Self::Profit),
            "trailing" => Some(Self::Trailing),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopHistoryEvent {
    pub id: String,
    pub code: String,
    pub event_type: StopHistoryEventType,
    pub trigger_kind: Option<StopHistoryTriggerKind>,
    pub trigger_price: Option<f64>,
    pub anchor_price: Option<f64>,
    pub anchor_source: Option<String>,
    pub snapshot_json: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopHistoryFilter {
    pub code: Option<String>,
    pub date: Option<NaiveDate>,
    pub event_type: Option<StopHistoryEventType>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StopRuleUpdate {
    pub stop_loss_price: Option<Option<f64>>,
    pub take_profit_price: Option<Option<f64>>,
    pub stop_loss_pct: Option<Option<f64>>,
    pub take_profit_pct: Option<Option<f64>>,
    pub trailing_pct: Option<Option<f64>>,
    pub reference_price: Option<Option<f64>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopAnchorSource {
    PositionCost,
    ReferencePrice,
}

impl StopAnchorSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PositionCost => "position_cost",
            Self::ReferencePrice => "reference_price",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopEvalState {
    Armed,
    Triggered,
    AnchorMissing,
    QuoteMissing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopStatusRow {
    pub code: String,
    pub last_price: Option<f64>,
    pub anchor_price: Option<f64>,
    pub anchor_source: Option<StopAnchorSource>,
    pub loss_threshold: Option<f64>,
    pub profit_threshold: Option<f64>,
    pub trailing_pct: Option<f64>,
    pub highest_price: Option<f64>,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub eval_state: StopEvalState,
}
