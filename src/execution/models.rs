use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde_json::Value;

use crate::core::{QuantixError, Result};
use crate::strategy::trait_def::Signal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyRunStatus {
    Running,
    Success,
    Failed,
}

impl StrategyRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Success => "success",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "running" => Some(Self::Running),
            "success" => Some(Self::Success),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    PendingSubmit,
    Submitted,
    Accepted,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Unknown,
}

impl OrderStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PendingSubmit => "pending_submit",
            Self::Submitted => "submitted",
            Self::Accepted => "accepted",
            Self::PartiallyFilled => "partially_filled",
            Self::Filled => "filled",
            Self::Canceled => "canceled",
            Self::Rejected => "rejected",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "pending_submit" => Some(Self::PendingSubmit),
            "submitted" => Some(Self::Submitted),
            "accepted" => Some(Self::Accepted),
            "partially_filled" => Some(Self::PartiallyFilled),
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
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Superseded => "superseded",
            Self::Expired => "expired",
        }
    }

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
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }

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
    Completed,
    Failed,
    Canceled,
}

impl ExecutionRequestStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
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
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
        }
    }

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
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Market => "market",
            Self::Limit => "limit",
        }
    }

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
    pub avg_fill_price: Option<Decimal>,
    pub status: OrderStatus,
    pub adapter: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub payload_json: Value,
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
