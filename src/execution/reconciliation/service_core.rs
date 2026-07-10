use super::*;
use crate::core::Result;
use crate::execution::models::OrderRecord;

impl ReconciliationService {
    /// 构造对账服务：仅注入 store，不挂 QMT 恢复能力。
    pub fn new(store: StrategyRuntimeStore) -> Self {
        let scanner = OpenOrderScanner::new(store.clone());
        Self {
            store,
            scanner,
            qmt_submit_service: None,
        }
    }

    /// 构造对账服务并注入 QmtTaskSubmitService，用于 qmt_live 订单的恢复。
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

    /// 对全部挂单执行对账：扫描挂单 → 逐单比对 adapter/本地状态 → 修复不一致 → Unknown 超时转失败；返回带 summary 统计与逐单明细的报告。
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

    /// 对单笔订单执行对账：qmt_live 可恢复走 QMT 路径；Unknown 走超时处理；其余无动作返回 NoAction。
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

    /// 返回内部扫描器引用，便于直接复用其查询能力。
    pub fn scanner(&self) -> &OpenOrderScanner {
        &self.scanner
    }
}
