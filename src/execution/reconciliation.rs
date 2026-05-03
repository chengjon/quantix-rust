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
use crate::core::{QuantixError, Result};
use crate::execution::models::{
    OrderRecord, OrderStatus, QmtLiveLastQuerySummary, QmtLiveReconciliationState,
};
use crate::execution::qmt_task_submit_service::{QmtTaskResolvedResult, QmtTaskSubmitService};
use crate::execution::runtime_store::StrategyRuntimeStore;

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

impl OpenOrderScanner {
    /// Create a new open order scanner
    pub fn new(store: StrategyRuntimeStore) -> Self {
        Self {
            store,
            stale_order_threshold_seconds: 3600, // 1 hour
            unknown_timeout_seconds: 300,        // 5 minutes
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(
        store: StrategyRuntimeStore,
        stale_threshold_seconds: i64,
        unknown_timeout_seconds: i64,
    ) -> Self {
        Self {
            store,
            stale_order_threshold_seconds: stale_threshold_seconds,
            unknown_timeout_seconds,
        }
    }

    /// List all open orders (orders that are not in terminal state)
    pub async fn list_open_orders(&self) -> Result<Vec<OrderRecord>> {
        self.store.list_open_orders().await
    }

    /// List orders in Unknown state that may need recovery
    pub async fn list_unknown_orders(&self) -> Result<Vec<OrderRecord>> {
        let open_orders = self.list_open_orders().await?;
        Ok(open_orders
            .into_iter()
            .filter(|o| o.status == OrderStatus::Unknown)
            .collect())
    }

    /// List stale orders (open orders older than threshold)
    pub async fn list_stale_orders(&self) -> Result<Vec<OrderRecord>> {
        let open_orders = self.list_open_orders().await?;
        let now = Utc::now();
        let threshold = chrono::Duration::seconds(self.stale_order_threshold_seconds);

        Ok(open_orders
            .into_iter()
            .filter(|o| {
                let age = now - o.created_at;
                age > threshold
            })
            .collect())
    }

    /// Get summary of open orders by status
    pub async fn get_open_order_summary(&self) -> Result<OpenOrderSummary> {
        let open_orders = self.list_open_orders().await?;
        let now = Utc::now();
        let stale_threshold = chrono::Duration::seconds(self.stale_order_threshold_seconds);

        let mut by_status: HashMap<String, usize> = HashMap::new();
        let mut stale_count = 0;
        let mut unknown_count = 0;

        for order in &open_orders {
            *by_status
                .entry(order.status.as_str().to_string())
                .or_insert(0) += 1;

            if order.status == OrderStatus::Unknown {
                unknown_count += 1;
            }

            let age = now - order.created_at;
            if age > stale_threshold {
                stale_count += 1;
            }
        }

        Ok(OpenOrderSummary {
            total_open: open_orders.len(),
            by_status,
            stale_count,
            unknown_count,
            stale_threshold_seconds: self.stale_order_threshold_seconds,
            unknown_timeout_seconds: self.unknown_timeout_seconds,
        })
    }
}

/// Summary of open orders
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl ReconciliationService {
    /// Create a new reconciliation service
    pub fn new(store: StrategyRuntimeStore) -> Self {
        let scanner = OpenOrderScanner::new(store.clone());
        Self {
            store,
            scanner,
            qmt_submit_service: None,
        }
    }

    pub fn with_qmt_live_query_service(
        store: StrategyRuntimeStore,
        qmt_submit_service: QmtTaskSubmitService,
    ) -> Self {
        let scanner = OpenOrderScanner::new(store.clone());
        Self {
            store,
            scanner,
            qmt_submit_service: Some(qmt_submit_service),
        }
    }

    /// Run reconciliation on all open orders
    ///
    /// This will:
    /// 1. Scan all open orders
    /// 2. Check each order against adapter state (if available)
    /// 3. Update local state if discrepancies found
    /// 4. Handle Unknown orders with timeout recovery
    pub async fn reconcile_all(&self) -> Result<ReconciliationReport> {
        let start = std::time::Instant::now();
        let open_orders = self.scanner.list_open_orders().await?;
        let mut results = Vec::new();

        for order in open_orders {
            let result = self.reconcile_order(&order).await?;
            results.push(result);
        }

        let matched = results
            .iter()
            .filter(|r| r.action == ReconciliationAction::NoAction)
            .count();
        let mismatched = results
            .iter()
            .filter(|r| r.action == ReconciliationAction::StateUpdated)
            .count();
        let recovered = results
            .iter()
            .filter(|r| r.action == ReconciliationAction::Recovered)
            .count();
        let failed = results
            .iter()
            .filter(|r| {
                matches!(
                    r.action,
                    ReconciliationAction::MarkedFailed
                        | ReconciliationAction::Cancelled
                        | ReconciliationAction::ManualIntervention
                ) || !r.success
            })
            .count();

        Ok(ReconciliationReport {
            summary: ReconciliationSummary {
                reconciled_at: Utc::now(),
                total_open_orders: results.len(),
                matched_orders: matched,
                mismatched_orders: mismatched,
                recovered_orders: recovered,
                failed_orders: failed,
                duration_ms: start.elapsed().as_millis() as u64,
            },
            results,
        })
    }

    /// Reconcile a single order
    pub async fn reconcile_order(&self, order: &OrderRecord) -> Result<OrderReconciliationResult> {
        if self.is_qmt_live_recoverable(order) {
            return self.reconcile_qmt_live_order(order).await;
        }

        // Check for Unknown state timeout
        if order.status == OrderStatus::Unknown {
            return self.handle_unknown_order(order).await;
        }

        // For mock_live orders, we can query the adapter state
        // For now, return no action needed for non-Unknown orders
        Ok(OrderReconciliationResult {
            order_id: order.order_id.clone(),
            client_order_id: order.client_order_id.clone(),
            symbol: order.symbol.clone(),
            local_status: order.status,
            broker_status: Some(order.status),
            action: ReconciliationAction::NoAction,
            success: true,
            error: None,
        })
    }

    fn is_qmt_live_recoverable(&self, order: &OrderRecord) -> bool {
        order.adapter == "qmt_live"
            && matches!(
                order.status,
                OrderStatus::PendingSubmit
                    | OrderStatus::Submitted
                    | OrderStatus::Accepted
                    | OrderStatus::Unknown
            )
    }

    async fn reconcile_qmt_live_order(&self, order: &OrderRecord) -> Result<OrderReconciliationResult> {
        let Some(task_id) = order
            .payload_json
            .get("qmt_live")
            .and_then(|value| value.get("task_identity"))
            .and_then(|value| value.get("task_id"))
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty())
        else {
            return self
                .persist_qmt_live_manual_intervention(
                    order,
                    None,
                    "task-id-based recovery is unavailable because task_id is missing",
                )
                .await;
        };

        let Some(service) = self.qmt_submit_service.as_ref() else {
            return self
                .persist_qmt_live_manual_intervention(
                    order,
                    None,
                    "qmt_live reconciliation service missing",
                )
                .await;
        };

        match service.query_task_result_by_task_id(task_id).await {
            Ok(result) => self.apply_qmt_live_result(order, result).await,
            Err(err) => self.persist_qmt_live_query_failure(order, err).await,
        }
    }

    async fn apply_qmt_live_result(
        &self,
        order: &OrderRecord,
        result: QmtTaskResolvedResult,
    ) -> Result<OrderReconciliationResult> {
        match result.latest_status {
            OrderStatus::PendingSubmit => {
                let payload_json = self.qmt_live_payload_json(
                    order,
                    Some(&result),
                    ReconciliationAction::NoAction,
                    None,
                    Utc::now(),
                )?;
                self.persist_qmt_live_payload_only(
                    order,
                    payload_json,
                    ReconciliationAction::NoAction,
                    Some(OrderStatus::PendingSubmit),
                    None,
                )
                .await
            }
            OrderStatus::Accepted | OrderStatus::Rejected | OrderStatus::Filled => {
                let action = if order.status == result.latest_status
                    && order.filled_quantity == result.filled_quantity
                    && order.avg_fill_price == result.avg_fill_price
                {
                    ReconciliationAction::NoAction
                } else {
                    ReconciliationAction::StateUpdated
                };
                let payload_json = self.qmt_live_payload_json(
                    order,
                    Some(&result),
                    action,
                    None,
                    Utc::now(),
                )?;

                if action == ReconciliationAction::NoAction {
                    return self
                        .persist_qmt_live_payload_only(
                            order,
                            payload_json,
                            action,
                            Some(result.latest_status),
                            None,
                        )
                        .await;
                }

                self.persist_qmt_live_state_and_payload(
                    order,
                    result.latest_status,
                    result.filled_quantity,
                    result.avg_fill_price,
                    payload_json,
                    action,
                )
                .await
            }
            _ => {
                self.persist_qmt_live_manual_intervention(
                    order,
                    Some(&result),
                    "completed qmt_live task missing terminal broker status",
                )
                .await
            }
        }
    }

    async fn persist_qmt_live_query_failure(
        &self,
        order: &OrderRecord,
        err: BridgeError,
    ) -> Result<OrderReconciliationResult> {
        let message = err.to_string();
        self.persist_qmt_live_manual_intervention(order, None, &message)
            .await
    }

    async fn persist_qmt_live_manual_intervention(
        &self,
        order: &OrderRecord,
        result: Option<&QmtTaskResolvedResult>,
        message: &str,
    ) -> Result<OrderReconciliationResult> {
        let payload_json = self.qmt_live_payload_json(
            order,
            result,
            ReconciliationAction::ManualIntervention,
            Some(message),
            Utc::now(),
        )?;
        self.persist_qmt_live_payload_only(
            order,
            payload_json,
            ReconciliationAction::ManualIntervention,
            result.map(|value| value.latest_status),
            Some(message.to_string()),
        )
        .await
    }

    async fn persist_qmt_live_payload_only(
        &self,
        order: &OrderRecord,
        payload_json: serde_json::Value,
        action: ReconciliationAction,
        broker_status: Option<OrderStatus>,
        error: Option<String>,
    ) -> Result<OrderReconciliationResult> {
        let updated_at = Utc::now();
        let updated = self
            .store
            .try_update_order_payload_with_version(
                &order.order_id,
                order.version,
                payload_json,
                updated_at,
            )
            .await?;
        if !updated {
            return Err(QuantixError::Other(format!(
                "qmt_live reconciliation payload update lost optimistic lock: {}",
                order.order_id
            )));
        }

        Ok(OrderReconciliationResult {
            order_id: order.order_id.clone(),
            client_order_id: order.client_order_id.clone(),
            symbol: order.symbol.clone(),
            local_status: order.status,
            broker_status,
            action,
            success: true,
            error,
        })
    }

    async fn persist_qmt_live_state_and_payload(
        &self,
        order: &OrderRecord,
        status: OrderStatus,
        filled_quantity: i64,
        avg_fill_price: Option<Decimal>,
        payload_json: serde_json::Value,
        action: ReconciliationAction,
    ) -> Result<OrderReconciliationResult> {
        let updated_at = Utc::now();
        let remaining_quantity = (order.requested_quantity - filled_quantity).max(0);
        let updated = self
            .store
            .try_update_order_state_and_payload_with_version(
                &order.order_id,
                order.version,
                status,
                filled_quantity,
                remaining_quantity,
                avg_fill_price,
                payload_json,
                updated_at,
            )
            .await?;
        if !updated {
            return Err(QuantixError::Other(format!(
                "qmt_live reconciliation state update lost optimistic lock: {}",
                order.order_id
            )));
        }

        Ok(OrderReconciliationResult {
            order_id: order.order_id.clone(),
            client_order_id: order.client_order_id.clone(),
            symbol: order.symbol.clone(),
            local_status: order.status,
            broker_status: Some(status),
            action,
            success: true,
            error: None,
        })
    }

    fn qmt_live_payload_json(
        &self,
        order: &OrderRecord,
        result: Option<&QmtTaskResolvedResult>,
        action: ReconciliationAction,
        error: Option<&str>,
        updated_at: DateTime<Utc>,
    ) -> Result<serde_json::Value> {
        let mut payload_json = order.payload_json.clone();
        if !payload_json.is_object() {
            payload_json = serde_json::json!({});
        }

        let root = payload_json
            .as_object_mut()
            .ok_or_else(|| QuantixError::Other("order payload_json is not an object".to_string()))?;
        let qmt_live = root
            .entry("qmt_live".to_string())
            .or_insert_with(|| serde_json::json!({}));
        if !qmt_live.is_object() {
            *qmt_live = serde_json::json!({});
        }

        let qmt_live = qmt_live
            .as_object_mut()
            .ok_or_else(|| QuantixError::Other("qmt_live payload is not an object".to_string()))?;

        if let Some(result) = result {
            qmt_live.insert(
                "last_query".to_string(),
                serde_json::to_value(QmtLiveLastQuerySummary {
                    latest_status: result.latest_status.as_str().to_string(),
                    filled_quantity: result.filled_quantity,
                    avg_fill_price: result.avg_fill_price.map(|value| value.to_string()),
                    broker_event_type: result
                        .broker_event_type
                        .map(qmt_live_broker_event_type_name),
                    rejection_reason: result.rejection_reason.clone(),
                    updated_at: updated_at.to_rfc3339(),
                })?,
            );
        }

        qmt_live.insert(
            "reconciliation".to_string(),
            serde_json::to_value(QmtLiveReconciliationState {
                last_action: Some(action.as_str().to_string()),
                last_error: error.map(|value| value.to_string()),
                last_attempt_at: Some(updated_at.to_rfc3339()),
            })?,
        );

        Ok(payload_json)
    }

    /// Handle orders in Unknown state
    async fn handle_unknown_order(&self, order: &OrderRecord) -> Result<OrderReconciliationResult> {
        let now = Utc::now();
        let timeout = chrono::Duration::seconds(self.scanner.unknown_timeout_seconds);
        let age = now - order.updated_at;

        // If order has been in Unknown state too long, mark as failed
        if age > timeout {
            // Check if there's a mock_live state we can recover from
            if let Ok(Some(mock_state)) =
                self.store.get_mock_live_order_state(&order.order_id).await
            {
                let filled_qty = mock_state
                    .fill_plan
                    .iter()
                    .take(mock_state.next_step_index)
                    .map(|step| step.quantity)
                    .sum::<i64>();
                let recovered_fill_price = mock_state
                    .simulated_fill_price
                    .or(order.avg_fill_price)
                    .or(Some(order.requested_price));

                // If recovery exhausted, mark as failed
                if mock_state.recovery_exhausted {
                    return self
                        .mark_order_failed(order, "Unknown state recovery exhausted")
                        .await;
                }

                if filled_qty >= order.requested_quantity {
                    return self
                        .mark_order_filled(
                            order,
                            filled_qty.min(order.requested_quantity),
                            recovered_fill_price,
                        )
                        .await;
                } else if filled_qty > 0 {
                    return self
                        .mark_order_partial_fill(order, filled_qty, recovered_fill_price)
                        .await;
                }
            }

            // No recovery possible, mark as failed
            return self.mark_order_failed(order, "Unknown state timeout").await;
        }

        // Still within timeout window, no action yet
        Ok(OrderReconciliationResult {
            order_id: order.order_id.clone(),
            client_order_id: order.client_order_id.clone(),
            symbol: order.symbol.clone(),
            local_status: OrderStatus::Unknown,
            broker_status: None,
            action: ReconciliationAction::NoAction,
            success: true,
            error: None,
        })
    }

    /// Mark an order as filled
    async fn mark_order_filled(
        &self,
        order: &OrderRecord,
        filled_quantity: i64,
        avg_fill_price: Option<Decimal>,
    ) -> Result<OrderReconciliationResult> {
        let now = Utc::now();
        self.store
            .update_order(
                &order.order_id,
                OrderStatus::Filled,
                filled_quantity,
                avg_fill_price,
                now,
            )
            .await?;

        Ok(OrderReconciliationResult {
            order_id: order.order_id.clone(),
            client_order_id: order.client_order_id.clone(),
            symbol: order.symbol.clone(),
            local_status: OrderStatus::Unknown,
            broker_status: Some(OrderStatus::Filled),
            action: ReconciliationAction::Recovered,
            success: true,
            error: None,
        })
    }

    /// Mark an order as partially filled
    async fn mark_order_partial_fill(
        &self,
        order: &OrderRecord,
        filled_quantity: i64,
        avg_fill_price: Option<Decimal>,
    ) -> Result<OrderReconciliationResult> {
        let now = Utc::now();
        self.store
            .update_order(
                &order.order_id,
                OrderStatus::PartiallyFilled,
                filled_quantity,
                avg_fill_price,
                now,
            )
            .await?;

        Ok(OrderReconciliationResult {
            order_id: order.order_id.clone(),
            client_order_id: order.client_order_id.clone(),
            symbol: order.symbol.clone(),
            local_status: OrderStatus::Unknown,
            broker_status: Some(OrderStatus::PartiallyFilled),
            action: ReconciliationAction::Recovered,
            success: true,
            error: None,
        })
    }

    /// Mark an order as failed
    async fn mark_order_failed(
        &self,
        order: &OrderRecord,
        reason: &str,
    ) -> Result<OrderReconciliationResult> {
        let now = Utc::now();
        self.store
            .update_order(
                &order.order_id,
                OrderStatus::Rejected,
                order.filled_quantity,
                order.avg_fill_price,
                now,
            )
            .await?;

        Ok(OrderReconciliationResult {
            order_id: order.order_id.clone(),
            client_order_id: order.client_order_id.clone(),
            symbol: order.symbol.clone(),
            local_status: OrderStatus::Unknown,
            broker_status: Some(OrderStatus::Rejected),
            action: ReconciliationAction::MarkedFailed,
            success: true,
            error: Some(reason.to_string()),
        })
    }

    /// Get the scanner for direct access
    pub fn scanner(&self) -> &OpenOrderScanner {
        &self.scanner
    }
}

/// Full reconciliation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationReport {
    /// Summary statistics
    pub summary: ReconciliationSummary,
    /// Individual order results
    pub results: Vec<OrderReconciliationResult>,
}

fn qmt_live_broker_event_type_name(event_type: BridgeBrokerEventType) -> String {
    match event_type {
        BridgeBrokerEventType::Acknowledgement => "acknowledgement".to_string(),
        BridgeBrokerEventType::Reject => "reject".to_string(),
        BridgeBrokerEventType::Execution => "execution".to_string(),
    }
}

#[cfg(test)]
mod tests;
