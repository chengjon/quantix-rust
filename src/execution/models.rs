use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value;

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
