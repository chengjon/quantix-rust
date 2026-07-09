use super::*;
use crate::core::Result;
use crate::execution::models::OrderRecord;

impl ReconciliationService {
    pub(crate) async fn reconcile_qmt_live_order(
        &self,
        order: &OrderRecord,
    ) -> Result<OrderReconciliationResult> {
        let task_identity = order
            .payload_json
            .get("qmt_live")
            .and_then(|value| value.get("task_identity"));

        let Some(task_id) = task_identity
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

        let local_submission_id = task_identity
            .and_then(|value| value.get("local_submission_id"))
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty());

        let query_result = if let Some(local_submission_id) = local_submission_id {
            service
                .query_task_result_once(task_id, &order.client_order_id, local_submission_id)
                .await
        } else {
            service.query_task_result_by_task_id(task_id).await
        };

        match query_result {
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
                let payload_json =
                    self.qmt_live_payload_json(order, Some(&result), action, None, Utc::now())?;

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

        let root = payload_json.as_object_mut().ok_or_else(|| {
            QuantixError::Other("order payload_json is not an object".to_string())
        })?;
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
            if result.client_order_id.is_some()
                || result.local_submission_id.is_some()
                || result.external_order_id.is_some()
            {
                let current_metadata = serde_json::from_value::<QmtLiveRuntimeMetadata>(
                    serde_json::Value::Object(qmt_live.clone()),
                )
                .unwrap_or_default();

                let recovered_task_identity = current_metadata
                    .recover_task_identity(
                        &result.adapter_order_id,
                        &order.client_order_id,
                        result.local_submission_id.as_deref(),
                        result.external_order_id.as_deref(),
                    )
                    .task_identity
                    .unwrap_or_else(|| QmtLiveTaskIdentity {
                        task_id: result.adapter_order_id.clone(),
                        client_order_id: order.client_order_id.clone(),
                        local_submission_id: result
                            .local_submission_id
                            .as_deref()
                            .unwrap_or_default()
                            .to_string(),
                        external_order_id: result.external_order_id.clone(),
                    });

                let task_identity = qmt_live
                    .entry("task_identity".to_string())
                    .or_insert_with(|| serde_json::json!({}));
                if !task_identity.is_object() {
                    *task_identity = serde_json::json!({});
                }
                let task_identity = task_identity.as_object_mut().ok_or_else(|| {
                    QuantixError::Other(
                        "qmt_live task_identity payload is not an object".to_string(),
                    )
                })?;

                task_identity.insert(
                    "task_id".to_string(),
                    serde_json::Value::String(recovered_task_identity.task_id),
                );
                task_identity.insert(
                    "client_order_id".to_string(),
                    serde_json::Value::String(recovered_task_identity.client_order_id),
                );
                task_identity.insert(
                    "local_submission_id".to_string(),
                    serde_json::Value::String(recovered_task_identity.local_submission_id),
                );
                if let Some(external_order_id) = recovered_task_identity.external_order_id {
                    task_identity.insert(
                        "external_order_id".to_string(),
                        serde_json::Value::String(external_order_id),
                    );
                }
            }

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
}

fn qmt_live_broker_event_type_name(event_type: BridgeBrokerEventType) -> String {
    match event_type {
        BridgeBrokerEventType::Acknowledgement => "acknowledgement".to_string(),
        BridgeBrokerEventType::Reject => "reject".to_string(),
        BridgeBrokerEventType::Execution => "execution".to_string(),
    }
}
