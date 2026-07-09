use super::*;
use crate::core::Result;
use crate::execution::models::OrderRecord;

impl ReconciliationService {
    /// Handle orders in Unknown state
    pub(crate) async fn handle_unknown_order(
        &self,
        order: &OrderRecord,
    ) -> Result<OrderReconciliationResult> {
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
}
