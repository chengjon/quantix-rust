#![allow(clippy::should_implement_trait)]

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::signal::Signal;
use crate::core::{QuantixError, Result};

mod mock_live;
pub use mock_live::{MockLiveFaultInjection, MockLiveFillStep, MockLiveOrderState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyRunStatus {
    Running,
    Success,
    Failed,
}

impl StrategyRunStatus {
    /// 返回标识串，与 from_str 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Success => "success",
            Self::Failed => "failed",
        }
    }

    /// 从字符串解析，未匹配返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "running" => Some(Self::Running),
            "success" => Some(Self::Success),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    PendingSubmit,
    Submitted,
    Accepted,
    PartiallyFilled,
    PendingCancel,
    Filled,
    Canceled,
    Rejected,
    Unknown,
}

impl OrderStatus {
    /// 返回标识串，与 from_str 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PendingSubmit => "pending_submit",
            Self::Submitted => "submitted",
            Self::Accepted => "accepted",
            Self::PartiallyFilled => "partially_filled",
            Self::PendingCancel => "pending_cancel",
            Self::Filled => "filled",
            Self::Canceled => "canceled",
            Self::Rejected => "rejected",
            Self::Unknown => "unknown",
        }
    }

    /// 从字符串解析，未匹配返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "pending_submit" => Some(Self::PendingSubmit),
            "submitted" => Some(Self::Submitted),
            "accepted" => Some(Self::Accepted),
            "partially_filled" => Some(Self::PartiallyFilled),
            "pending_cancel" => Some(Self::PendingCancel),
            "filled" => Some(Self::Filled),
            "canceled" => Some(Self::Canceled),
            "rejected" => Some(Self::Rejected),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalStatus {
    New,
    Superseded,
    Expired,
}

impl SignalStatus {
    /// 返回标识串，与 from_str 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Superseded => "superseded",
            Self::Expired => "expired",
        }
    }

    /// 从字符串解析，未匹配返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "new" => Some(Self::New),
            "superseded" => Some(Self::Superseded),
            "expired" => Some(Self::Expired),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

impl ApprovalStatus {
    /// 返回标识串，与 from_str 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }

    /// 从字符串解析，未匹配返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionRequestStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Canceled,
}

impl ExecutionRequestStatus {
    /// 返回标识串，与 from_str 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
        }
    }

    /// 从字符串解析，未匹配返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "in_progress" => Some(Self::InProgress),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "canceled" => Some(Self::Canceled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    /// 返回标识串，与 from_str 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
        }
    }

    /// 从字符串解析，未匹配返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "buy" => Some(Self::Buy),
            "sell" => Some(Self::Sell),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
}

impl OrderType {
    /// 返回标识串，与 from_str 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Market => "market",
            Self::Limit => "limit",
        }
    }

    /// 从字符串解析，未匹配返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "market" => Some(Self::Market),
            "limit" => Some(Self::Limit),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrategyRunRecord {
    pub run_id: String,
    pub strategy_name: String,
    pub mode: String,
    pub trigger: String,
    pub status: StrategyRunStatus,
    pub symbol: String,
    pub timeframe: String,
    pub bar_end: DateTime<Utc>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub metadata_json: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SignalEventRecord {
    pub event_id: String,
    pub run_id: String,
    pub strategy_name: String,
    pub symbol: String,
    pub signal: String,
    pub ts: DateTime<Utc>,
    pub payload_json: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SignalEnvelope {
    pub signal: Signal,
    pub metadata_json: Value,
}

impl SignalEnvelope {
    /// 用信号构造 envelope，`metadata_json` 初始化为空对象。
    pub fn new(signal: Signal) -> Self {
        Self {
            signal,
            metadata_json: Value::Object(Default::default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPolicy {
    pub fixed_cash_per_buy: Decimal,
    pub slippage_bps: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderIntent {
    pub symbol: String,
    pub side: OrderSide,
    pub requested_quantity: i64,
    pub requested_price: Decimal,
    pub order_type: OrderType,
    pub reason: String,
    pub policy_snapshot_json: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderRecord {
    pub order_id: String,
    pub client_order_id: String,
    pub run_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub requested_quantity: i64,
    pub requested_price: Decimal,
    pub filled_quantity: i64,
    pub remaining_quantity: i64,
    pub avg_fill_price: Option<Decimal>,
    pub status: OrderStatus,
    pub adapter: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_transition_at: DateTime<Utc>,
    pub version: i64,
    pub payload_json: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct QmtLiveTaskIdentity {
    #[serde(default)]
    pub task_id: String,
    #[serde(default)]
    pub client_order_id: String,
    #[serde(default)]
    pub local_submission_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_order_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct QmtLiveLastQuerySummary {
    pub latest_status: String,
    #[serde(default)]
    pub filled_quantity: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_fill_price: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub broker_event_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejection_reason: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct QmtLiveReconciliationState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_attempt_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct QmtLiveRuntimeMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_identity: Option<QmtLiveTaskIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_query: Option<QmtLiveLastQuerySummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reconciliation: Option<QmtLiveReconciliationState>,
}

impl QmtLiveTaskIdentity {
    /// 用入参回填缺失字段（空串视为缺失），返回新的 identity 实例。
    pub fn recover_missing_fields(
        &self,
        task_id: &str,
        client_order_id: &str,
        local_submission_id: Option<&str>,
        external_order_id: Option<&str>,
    ) -> Self {
        Self {
            task_id: if self.task_id.trim().is_empty() {
                task_id.to_string()
            } else {
                self.task_id.clone()
            },
            client_order_id: if self.client_order_id.trim().is_empty() {
                client_order_id.to_string()
            } else {
                self.client_order_id.clone()
            },
            local_submission_id: if self.local_submission_id.trim().is_empty() {
                local_submission_id.unwrap_or_default().to_string()
            } else {
                self.local_submission_id.clone()
            },
            external_order_id: self
                .external_order_id
                .clone()
                .or_else(|| external_order_id.map(|value| value.to_string())),
        }
    }
}

impl QmtLiveRuntimeMetadata {
    /// 从单字段推断完整 identity（用于崩溃恢复 / 重启场景）。
    pub fn recover_task_identity(
        &self,
        task_id: &str,
        client_order_id: &str,
        local_submission_id: Option<&str>,
        external_order_id: Option<&str>,
    ) -> Self {
        Self {
            task_identity: self.task_identity.as_ref().map(|task_identity| {
                task_identity.recover_missing_fields(
                    task_id,
                    client_order_id,
                    local_submission_id,
                    external_order_id,
                )
            }),
            last_query: self.last_query.clone(),
            reconciliation: self.reconciliation.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FillDetails {
    pub fill_id: u64,
    pub fill_quantity: i64,
    pub fill_price: Decimal,
    /// Last fill price (for incremental fills)
    #[serde(default)]
    pub last_fill_price: Decimal,
    /// Last fill quantity (for incremental fills)
    #[serde(default)]
    pub last_fill_quantity: i64,
    /// Total number of fills
    #[serde(default)]
    pub total_fills: i64,
    /// Commission amount
    #[serde(default)]
    pub commission: Decimal,
    /// Other fees
    #[serde(default)]
    pub fees: Decimal,
    /// Execution venue
    #[serde(default)]
    pub venue: String,
    /// Broker's fill ID
    #[serde(default)]
    pub broker_fill_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FillDeltaContext {
    pub order_id: String,
    pub client_order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub requested_price: Decimal,
    pub old_filled_quantity: i64,
    pub new_filled_quantity: i64,
    pub fill_details: Option<FillDetails>,
    pub event_time: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillDeltaResult {
    pub applied: bool,
    pub delta_quantity: i64,
    pub trade_record_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderEventRecord {
    pub event_id: String,
    pub order_id: String,
    pub client_order_id: String,
    pub event_type: String,
    pub event_time: DateTime<Utc>,
    pub details_json: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunnerCheckpointRecord {
    pub checkpoint_id: String,
    pub strategy_name: String,
    pub mode: String,
    pub symbol: String,
    pub timeframe: String,
    pub last_processed_bar: Option<DateTime<Utc>>,
    pub last_run_id: Option<String>,
    pub state_json: Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrategySignalRecord {
    pub signal_id: String,
    pub strategy_instance_id: String,
    pub strategy_name: String,
    pub symbol: String,
    pub timeframe: String,
    pub bar_end: DateTime<Utc>,
    pub signal_value: String,
    pub signal_status: SignalStatus,
    pub approval_status: ApprovalStatus,
    pub run_id: String,
    pub metadata_json: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionRequestRecord {
    pub request_id: String,
    pub signal_id: String,
    pub target_mode: String,
    pub target_account: String,
    pub request_status: ExecutionRequestStatus,
    pub approved_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub payload_json: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrategyDaemonCheckpointRecord {
    pub checkpoint_id: String,
    pub strategy_instance_id: String,
    pub strategy_name: String,
    pub symbol: String,
    pub timeframe: String,
    pub last_processed_bar: Option<DateTime<Utc>>,
    pub last_run_id: Option<String>,
    pub state_json: Value,
    pub updated_at: DateTime<Utc>,
}

/// 将策略信号 envelope 翻译为可执行的交易意图（含价格 / 数量 / 方向 / 时间戳）。
pub fn translate_signal(
    envelope: &SignalEnvelope,
    symbol: &str,
    market_price: Decimal,
    held_volume: Option<i64>,
    policy: &ExecutionPolicy,
) -> Result<Option<OrderIntent>> {
    match envelope.signal {
        Signal::Hold => Ok(None),
        Signal::Buy => {
            let requested_quantity = board_lot_quantity(policy.fixed_cash_per_buy, market_price)?;
            if requested_quantity <= 0 {
                return Err(QuantixError::Other(
                    "strategy paper buy 可用固定金额不足以下整手单".to_string(),
                ));
            }

            Ok(Some(OrderIntent {
                symbol: symbol.to_string(),
                side: OrderSide::Buy,
                requested_quantity,
                requested_price: apply_slippage(market_price, policy.slippage_bps, true)?,
                order_type: OrderType::Market,
                reason: "signal_buy".to_string(),
                policy_snapshot_json: serde_json::json!({
                    "fixed_cash_per_buy": policy.fixed_cash_per_buy,
                    "slippage_bps": policy.slippage_bps,
                }),
            }))
        }
        Signal::Sell => {
            let requested_quantity = held_volume.unwrap_or(0);
            if requested_quantity <= 0 {
                return Err(QuantixError::Other(
                    "strategy paper sell 当前无可卖持仓".to_string(),
                ));
            }

            Ok(Some(OrderIntent {
                symbol: symbol.to_string(),
                side: OrderSide::Sell,
                requested_quantity,
                requested_price: apply_slippage(market_price, policy.slippage_bps, false)?,
                order_type: OrderType::Market,
                reason: "signal_sell".to_string(),
                policy_snapshot_json: serde_json::json!({
                    "sell_mode": "sell_all",
                    "slippage_bps": policy.slippage_bps,
                }),
            }))
        }
    }
}

fn board_lot_quantity(cash: Decimal, price: Decimal) -> Result<i64> {
    if price <= Decimal::ZERO {
        return Err(QuantixError::Other(
            "strategy paper 市价必须大于 0".to_string(),
        ));
    }

    let raw_shares = (cash / price).floor();
    let lot_count = (raw_shares / Decimal::from(100)).floor();
    lot_count
        .to_i64()
        .map(|lots| lots * 100)
        .ok_or_else(|| QuantixError::Other("strategy paper 下单数量超出支持范围".to_string()))
}

fn apply_slippage(price: Decimal, slippage_bps: u32, is_buy: bool) -> Result<Decimal> {
    if price <= Decimal::ZERO {
        return Err(QuantixError::Other(
            "strategy paper 市价必须大于 0".to_string(),
        ));
    }

    let bps = Decimal::from(slippage_bps) / Decimal::from(10_000);
    let factor = if is_buy {
        Decimal::ONE + bps
    } else {
        Decimal::ONE - bps
    };

    Ok(price * factor)
}
