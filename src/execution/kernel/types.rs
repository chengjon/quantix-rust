use chrono::{DateTime, Utc};

use crate::execution::models::{ExecutionPolicy, OrderIntent, OrderStatus};
use crate::strategy::trait_def::Signal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskDecision {
    Allow,
    Reject { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRunRequest {
    pub run_id: String,
    pub strategy_name: String,
    pub mode: String,
    pub trigger: String,
    pub symbol: String,
    pub timeframe: String,
    pub bar_end: DateTime<Utc>,
    pub market_price: rust_decimal::Decimal,
    pub held_volume: Option<i64>,
    pub policy: ExecutionPolicy,
    pub client_order_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedExecutionRequest {
    pub run_id: String,
    pub strategy_name: String,
    pub mode: String,
    pub trigger: String,
    pub symbol: String,
    pub timeframe: String,
    pub bar_end: DateTime<Utc>,
    pub signal: Signal,
    pub signal_payload_json: serde_json::Value,
    pub intent: OrderIntent,
    pub client_order_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelExecutionResult {
    pub run_id: String,
    pub signal: Signal,
    pub order_status: Option<OrderStatus>,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverySummary {
    pub scanned: usize,
    pub recovered: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub skipped: usize,
}
