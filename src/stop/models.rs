#![allow(clippy::should_implement_trait)]

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 单只标的的止损止盈规则：code 标的、可选 stop_loss_price/take_profit_price 绝对价位、可选 stop_loss_pct/take_profit_pct 百分比阈值、可选 trailing_pct 移动止损百分比、highest_price 跟踪最高价、reference_price 参考价、last_triggered_at 最近触发时间、created_at/updated_at 时间戳。
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

/// 止损止盈触发类型：Loss 止损、Profit 止盈、TrailingLoss 移动止损。用于 TriggeredStop.kind 与对外展示。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopTriggerKind {
    Loss,
    Profit,
    TrailingLoss,
}

/// 已触发的止损止盈快照：code、kind 触发类型、current_price 当前价、threshold_price 阈值价、highest_price 跟踪最高价（移动止损）、anchor_price/anchor_source 锚定价与来源、triggered_at 触发时间。
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

/// 单次止损止盈评估结果：updated_rule 更新后的规则（如跟踪最高价）、triggered_stop 可选触发快照（未触发时为 None）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopEvaluationResult {
    pub updated_rule: StopRule,
    pub triggered_stop: Option<TriggeredStop>,
}

/// 止损止盈历史事件类型：Set 首次设置、Update 更新、Remove 删除、Trigger 触发。入库为 snake_case 字符串，as_str/from_str 互逆。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopHistoryEventType {
    Set,
    Update,
    Remove,
    Trigger,
}

impl StopHistoryEventType {
    /// 返回事件类型的稳定字符串标识（"set"/"update"/"remove"/"trigger"），用于入库与日志。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Set => "set",
            Self::Update => "update",
            Self::Remove => "remove",
            Self::Trigger => "trigger",
        }
    }

    /// 反向解析事件类型字符串；未知值返回 None（调用方需处理）。
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

/// 止损止盈历史触发类型（与 StopTriggerKind 区别：移动止损在此处统一记为 Trailing）：Loss 止损、Profit 止盈、Trailing 移动止损。入库为 snake_case 字符串。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopHistoryTriggerKind {
    Loss,
    Profit,
    Trailing,
}

impl StopHistoryTriggerKind {
    /// 返回触发类型的稳定字符串标识（"loss"/"profit"/"trailing"），用于入库与日志。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Loss => "loss",
            Self::Profit => "profit",
            Self::Trailing => "trailing",
        }
    }

    /// 反向解析触发类型字符串；未知值返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "loss" => Some(Self::Loss),
            "profit" => Some(Self::Profit),
            "trailing" => Some(Self::Trailing),
            _ => None,
        }
    }
}

/// 止损止盈历史事件入库记录：id 主键、code 标的、event_type 事件类型、trigger_kind 可选触发类型、trigger_price/anchor_price/anchor_source 触发与锚定上下文、snapshot_json 规则全量快照、created_at 时间。
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

/// 历史查询过滤器：code 可选标的过滤、date 可选日期过滤、event_type 可选事件类型过滤、limit 可选行数上限。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopHistoryFilter {
    pub code: Option<String>,
    pub date: Option<NaiveDate>,
    pub event_type: Option<StopHistoryEventType>,
    pub limit: Option<usize>,
}

/// 止损止盈规则 patch 更新：每字段为 Option<Option<f64>>——外层 None 表示不改、内层 None 表示清空。Default 全 None（不修改任何字段）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StopRuleUpdate {
    pub stop_loss_price: Option<Option<f64>>,
    pub take_profit_price: Option<Option<f64>>,
    pub stop_loss_pct: Option<Option<f64>>,
    pub take_profit_pct: Option<Option<f64>>,
    pub trailing_pct: Option<Option<f64>>,
    pub reference_price: Option<Option<f64>>,
}

/// 止损止盈锚定价来源：PositionCost 持仓成本价、ReferencePrice 用户指定的参考价。入库为 snake_case 字符串，as_str 提供 stable 标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopAnchorSource {
    PositionCost,
    ReferencePrice,
}

impl StopAnchorSource {
    /// 返回锚定来源的稳定字符串标识（"position_cost"/"reference_price"），用于入库与日志。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PositionCost => "position_cost",
            Self::ReferencePrice => "reference_price",
        }
    }
}

/// 止损止盈评估状态：Armed 已武装（待触发）、Triggered 已触发、AnchorMissing 缺锚定价无法评估、QuoteMissing 缺行情无法评估。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopEvalState {
    Armed,
    Triggered,
    AnchorMissing,
    QuoteMissing,
}

/// 止损止盈状态展示行：code 标的、last_price 最新价、anchor_price/anchor_source 锚定价与来源、loss_threshold/profit_threshold/trailing_pct 各类阈值、highest_price 跟踪最高价、last_triggered_at 最近触发时间、eval_state 当前评估状态。
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
