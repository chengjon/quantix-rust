//! Order reconciliation module for execution system
//!
//! Provides open-order scanning, account reconciliation, and state repair capabilities.
#![allow(clippy::should_implement_trait)]
//!
//! # Features
//!
//! - **Open Order Scanner**: Scan and query open orders across all adapters
//! - **Account Reconciliation**: Compare local state with broker state
//! - **State Repair**: Handle inconsistencies between local and broker states
//! - **Recovery**: Recover from Unknown order states

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::bridge::error::BridgeError;
use crate::bridge::models::BridgeBrokerEventType;
use crate::core::QuantixError;
use crate::execution::models::{
    OrderRecord, OrderStatus, QmtLiveLastQuerySummary, QmtLiveReconciliationState,
    QmtLiveRuntimeMetadata, QmtLiveTaskIdentity,
};
use crate::execution::qmt_task_submit_service::{QmtTaskResolvedResult, QmtTaskSubmitService};
use crate::execution::runtime_store::StrategyRuntimeStore;

mod order_state;
mod qmt_live;
mod scanner;
mod service_core;

#[allow(unused_imports)]
pub use order_state::*;
#[allow(unused_imports)]
pub use qmt_live::*;
#[allow(unused_imports)]
pub use scanner::*;
#[allow(unused_imports)]
pub use service_core::*;

/// Reconciliation summary for a single reconciliation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationSummary {
    /// When this reconciliation was performed
    pub reconciled_at: DateTime<Utc>,
    /// Total open orders scanned
    pub total_open_orders: usize,
    /// Orders that matched broker state
    pub matched_orders: usize,
    /// Orders with state discrepancies
    pub mismatched_orders: usize,
    /// Orders recovered from Unknown state
    pub recovered_orders: usize,
    /// Orders that failed to reconcile
    pub failed_orders: usize,
    /// Time taken in milliseconds
    pub duration_ms: u64,
}

/// Details of a single order reconciliation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderReconciliationResult {
    /// Order ID
    pub order_id: String,
    /// Client order ID
    pub client_order_id: String,
    /// Symbol
    pub symbol: String,
    /// Local status before reconciliation
    pub local_status: OrderStatus,
    /// Broker status (if available)
    pub broker_status: Option<OrderStatus>,
    /// Action taken
    pub action: ReconciliationAction,
    /// Whether reconciliation succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Actions that can be taken during reconciliation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconciliationAction {
    /// No action needed - states match
    NoAction,
    /// Updated local state to match broker
    StateUpdated,
    /// Order was in Unknown state and recovered
    Recovered,
    /// Order was marked as failed due to timeout
    MarkedFailed,
    /// Order was cancelled due to discrepancy
    Cancelled,
    /// Manual intervention required
    ManualIntervention,
}

impl ReconciliationAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoAction => "no_action",
            Self::StateUpdated => "state_updated",
            Self::Recovered => "recovered",
            Self::MarkedFailed => "marked_failed",
            Self::Cancelled => "cancelled",
            Self::ManualIntervention => "manual_intervention",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "no_action" => Some(Self::NoAction),
            "state_updated" => Some(Self::StateUpdated),
            "recovered" => Some(Self::Recovered),
            "marked_failed" => Some(Self::MarkedFailed),
            "cancelled" => Some(Self::Cancelled),
            "manual_intervention" => Some(Self::ManualIntervention),
            _ => None,
        }
    }
}

/// Open order scanner for finding orders that need attention
pub struct OpenOrderScanner {
    store: StrategyRuntimeStore,
    /// Maximum age in seconds for an order to be considered "stale"
    stale_order_threshold_seconds: i64,
    /// Maximum age in seconds for an Unknown order before marking failed
    unknown_timeout_seconds: i64,
}

pub struct OpenOrderSummary {
    /// Total number of open orders
    pub total_open: usize,
    /// Count by status
    pub by_status: HashMap<String, usize>,
    /// Number of stale orders
    pub stale_count: usize,
    /// Number of orders in Unknown state
    pub unknown_count: usize,
    /// Stale threshold in seconds
    pub stale_threshold_seconds: i64,
    /// Unknown timeout in seconds
    pub unknown_timeout_seconds: i64,
}

/// Reconciliation service for comparing and fixing order states
pub struct ReconciliationService {
    store: StrategyRuntimeStore,
    scanner: OpenOrderScanner,
    qmt_submit_service: Option<QmtTaskSubmitService>,
}

pub struct ReconciliationReport {
    /// Summary statistics
    pub summary: ReconciliationSummary,
    /// Individual order results
    pub results: Vec<OrderReconciliationResult>,
}

#[cfg(test)]
mod tests;
