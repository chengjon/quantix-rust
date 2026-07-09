use super::*;
use crate::core::Result;
use crate::execution::models::OrderRecord;

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

    /// Get the scanner for direct access
    pub fn scanner(&self) -> &OpenOrderScanner {
        &self.scanner
    }
}
