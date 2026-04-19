#![allow(clippy::bool_comparison)]

use super::*;

impl<A, F, R> ExecutionKernel<A, F, R>
where
    A: ExecutionAdapter,
    F: FillDeltaApplier,
    R: RiskEvaluator,
{
    pub async fn recover_pending_orders(&self) -> Result<RecoverySummary> {
        let mut summary = RecoverySummary {
            scanned: 0,
            recovered: 0,
            unchanged: 0,
            failed: 0,
            skipped: 0,
        };

        let orders = self.store.list_recoverable_mock_live_orders().await?;
        summary.scanned = orders.len();

        for order in orders {
            let before_state = self
                .store
                .get_mock_live_order_state(&order.order_id)
                .await?;
            let response = match self.adapter.query_order(&order.client_order_id).await {
                Ok(response) => response,
                Err(_) => {
                    summary.failed += 1;
                    continue;
                }
            };
            let after_state = self
                .store
                .get_mock_live_order_state(&order.order_id)
                .await?;

            if response.filled_quantity > order.filled_quantity {
                let updated_at = Utc::now();
                let fill_result = match self
                    .fill_delta
                    .apply_fill_delta(crate::execution::models::FillDeltaContext {
                        order_id: order.order_id.clone(),
                        client_order_id: order.client_order_id.clone(),
                        symbol: order.symbol.clone(),
                        side: order.side,
                        requested_price: order.requested_price,
                        old_filled_quantity: order.filled_quantity,
                        new_filled_quantity: response.filled_quantity,
                        fill_details: response.fill_details.clone(),
                        event_time: updated_at,
                    })
                    .await
                {
                    Ok(result) => result,
                    Err(err) => {
                        self.store
                            .insert_order_event(&OrderEventRecord {
                                event_id: Uuid::new_v4().to_string(),
                                order_id: order.order_id.clone(),
                                client_order_id: order.client_order_id.clone(),
                                event_type: "fill_apply_failed".to_string(),
                                event_time: updated_at,
                                details_json: serde_json::json!({
                                    "error": err.to_string(),
                                    "proposed_status": response.latest_status.as_str(),
                                    "proposed_filled_quantity": response.filled_quantity,
                                    "avg_fill_price": response.avg_fill_price,
                                    "fill_details": fill_details_json(response.fill_details.as_ref()),
                                }),
                            })
                            .await?;
                        summary.failed += 1;
                        continue;
                    }
                };

                if !fill_result.applied {
                    summary.skipped += 1;
                    continue;
                }

                let remaining_quantity =
                    (order.requested_quantity - response.filled_quantity).max(0);
                let updated = self
                    .store
                    .try_update_order_with_version(
                        &order.order_id,
                        order.version,
                        response.latest_status,
                        response.filled_quantity,
                        remaining_quantity,
                        response.avg_fill_price,
                        updated_at,
                    )
                    .await?;

                if updated {
                    self.store
                        .insert_order_event(&OrderEventRecord {
                            event_id: Uuid::new_v4().to_string(),
                            order_id: order.order_id.clone(),
                            client_order_id: order.client_order_id.clone(),
                            event_type: response.latest_status.as_str().to_string(),
                            event_time: updated_at,
                            details_json: serde_json::json!({
                                "filled_quantity": response.filled_quantity,
                                "avg_fill_price": response.avg_fill_price,
                            }),
                        })
                        .await?;
                    self.store
                        .insert_order_event(&OrderEventRecord {
                            event_id: Uuid::new_v4().to_string(),
                            order_id: order.order_id.clone(),
                            client_order_id: order.client_order_id.clone(),
                            event_type: "fill_applied".to_string(),
                            event_time: updated_at,
                            details_json: serde_json::json!({
                                "delta_quantity": fill_result.delta_quantity,
                                "trade_record_id": fill_result.trade_record_id,
                                "fill_details": fill_details_json(response.fill_details.as_ref()),
                            }),
                        })
                        .await?;
                    self.mark_mock_live_fill_applied(
                        &order.order_id,
                        &order.client_order_id,
                        response.fill_details.as_ref(),
                    )
                    .await?;
                    self.risk.sync_after_fill().await?;

                    summary.recovered += 1;
                    continue;
                }

                let latest = self
                    .store
                    .find_order_by_client_order_id(&order.client_order_id)
                    .await?;
                if let Some(latest) = latest {
                    if latest.filled_quantity >= response.filled_quantity
                        && latest.status == response.latest_status
                    {
                        summary.skipped += 1;
                        continue;
                    }

                    let retry_updated = self
                        .store
                        .try_update_order_with_version(
                            &latest.order_id,
                            latest.version,
                            response.latest_status,
                            response.filled_quantity,
                            (latest.requested_quantity - response.filled_quantity).max(0),
                            response.avg_fill_price,
                            updated_at,
                        )
                        .await?;
                    if retry_updated {
                        self.store
                            .insert_order_event(&OrderEventRecord {
                                event_id: Uuid::new_v4().to_string(),
                                order_id: latest.order_id.clone(),
                                client_order_id: latest.client_order_id.clone(),
                                event_type: response.latest_status.as_str().to_string(),
                                event_time: updated_at,
                                details_json: serde_json::json!({
                                    "filled_quantity": response.filled_quantity,
                                    "avg_fill_price": response.avg_fill_price,
                                }),
                            })
                            .await?;
                        self.store
                            .insert_order_event(&OrderEventRecord {
                                event_id: Uuid::new_v4().to_string(),
                                order_id: latest.order_id.clone(),
                                client_order_id: latest.client_order_id.clone(),
                                event_type: "fill_applied".to_string(),
                                event_time: updated_at,
                                details_json: serde_json::json!({
                                    "delta_quantity": fill_result.delta_quantity,
                                    "trade_record_id": fill_result.trade_record_id,
                                    "fill_details": fill_details_json(response.fill_details.as_ref()),
                                }),
                            })
                            .await?;
                        self.mark_mock_live_fill_applied(
                            &latest.order_id,
                            &latest.client_order_id,
                            response.fill_details.as_ref(),
                        )
                        .await?;
                        self.risk.sync_after_fill().await?;

                        summary.recovered += 1;
                    } else {
                        summary.skipped += 1;
                    }
                } else {
                    summary.skipped += 1;
                }
                continue;
            }

            if response.latest_status != order.status
                || response.filled_quantity != order.filled_quantity
            {
                let remaining_quantity =
                    (order.requested_quantity - response.filled_quantity).max(0);
                let updated_at = Utc::now();
                let updated = self
                    .store
                    .try_update_order_with_version(
                        &order.order_id,
                        order.version,
                        response.latest_status,
                        response.filled_quantity,
                        remaining_quantity,
                        response.avg_fill_price,
                        updated_at,
                    )
                    .await?;

                if updated {
                    self.store
                        .insert_order_event(&OrderEventRecord {
                            event_id: Uuid::new_v4().to_string(),
                            order_id: order.order_id.clone(),
                            client_order_id: order.client_order_id.clone(),
                            event_type: response.latest_status.as_str().to_string(),
                            event_time: updated_at,
                            details_json: serde_json::json!({
                                "filled_quantity": response.filled_quantity,
                                "avg_fill_price": response.avg_fill_price,
                            }),
                        })
                        .await?;
                    summary.recovered += 1;
                    continue;
                }

                let latest = self
                    .store
                    .find_order_by_client_order_id(&order.client_order_id)
                    .await?;
                if let Some(latest) = latest {
                    let retry_updated = self
                        .store
                        .try_update_order_with_version(
                            &latest.order_id,
                            latest.version,
                            response.latest_status,
                            response.filled_quantity,
                            (latest.requested_quantity - response.filled_quantity).max(0),
                            response.avg_fill_price,
                            updated_at,
                        )
                        .await?;
                    if retry_updated {
                        self.store
                            .insert_order_event(&OrderEventRecord {
                                event_id: Uuid::new_v4().to_string(),
                                order_id: latest.order_id.clone(),
                                client_order_id: latest.client_order_id.clone(),
                                event_type: response.latest_status.as_str().to_string(),
                                event_time: updated_at,
                                details_json: serde_json::json!({
                                    "filled_quantity": response.filled_quantity,
                                    "avg_fill_price": response.avg_fill_price,
                                }),
                            })
                            .await?;
                        summary.recovered += 1;
                    } else {
                        summary.skipped += 1;
                    }
                } else {
                    summary.skipped += 1;
                }
                continue;
            }

            let exhaustion_transition = before_state
                .as_ref()
                .map(|state| state.recovery_exhausted)
                .unwrap_or(false)
                == false
                && after_state
                    .as_ref()
                    .map(|state| state.recovery_exhausted)
                    .unwrap_or(false);

            if exhaustion_transition {
                let after_state = after_state.unwrap();
                self.store
                    .insert_order_event(&OrderEventRecord {
                        event_id: Uuid::new_v4().to_string(),
                        order_id: order.order_id.clone(),
                        client_order_id: order.client_order_id.clone(),
                        event_type: "recovery_exhausted".to_string(),
                        event_time: Utc::now(),
                        details_json: serde_json::json!({
                            "unknown_retries": after_state.unknown_retries,
                            "reason": after_state.exhausted_reason,
                        }),
                    })
                    .await?;
                summary.recovered += 1;
            } else {
                summary.unchanged += 1;
            }
        }

        Ok(summary)
    }
}
