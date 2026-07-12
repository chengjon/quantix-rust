//! QMT query output builders and the cancel command result type.

use crate::bridge::client::BridgeHttpClient;
use crate::bridge::error::BridgeError;
use crate::core::{QuantixError, Result};
use crate::execution::qmt_task_submit_service::QmtTaskSubmitService;

pub(crate) fn normalize_symbol_for_bridge(symbol: &str) -> String {
    if symbol.contains('.') {
        return symbol.to_string();
    }
    if symbol.starts_with('6') {
        format!("{symbol}.SH")
    } else {
        format!("{symbol}.SZ")
    }
}

#[derive(Debug, Clone)]
pub(crate) struct QmtCancelCommandResult {
    pub requested_order_id: String,
    pub cancel_order_id: String,
    pub resolved_from_task_result: bool,
    pub response: crate::bridge::models::BridgeQmtCancelResponse,
}

pub(crate) fn create_qmt_task_submit_service(
    client: &BridgeHttpClient,
) -> Result<QmtTaskSubmitService> {
    QmtTaskSubmitService::new(client.clone(), 1, 30_000)
        .map_err(|err| QuantixError::Other(err.to_string()))
}

pub(crate) fn should_fallback_from_task_result_lookup(error: &BridgeError) -> bool {
    matches!(
        error,
        BridgeError::Http(_) | BridgeError::UnsupportedMethod(_)
    )
}

pub(crate) async fn build_execution_bridge_qmt_query_output(
    client: &BridgeHttpClient,
    order_id: &str,
) -> Result<serde_json::Value> {
    let submit_service = create_qmt_task_submit_service(client)?;

    match submit_service.query_task_result_by_task_id(order_id).await {
        Ok(result) => Ok(serde_json::json!({
            "query_mode": "task_result",
            "adapter_order_id": result.adapter_order_id,
            "latest_status": result.latest_status.as_str(),
            "filled_quantity": result.filled_quantity,
            "avg_fill_price": result.avg_fill_price.map(|value| value.to_string()),
            "rejection_reason": result.rejection_reason,
            "broker_event_type": result.broker_event_type.map(|value| format!("{value:?}")),
            "external_order_id": result.external_order_id,
            "client_order_id": result.client_order_id,
            "local_submission_id": result.local_submission_id,
            "source_name": result.source_name,
        })),
        Err(error) if should_fallback_from_task_result_lookup(&error) => {
            let response = client
                .qmt_query_order(order_id)
                .await
                .map_err(|err| QuantixError::Other(err.to_string()))?;
            Ok(serde_json::json!({
                "query_mode": "legacy_order",
                "adapter_order_id": response.adapter_order_id,
                "latest_status": response.latest_status,
                "filled_quantity": response.filled_quantity,
                "avg_fill_price": response.avg_fill_price,
            }))
        }
        Err(error) => Err(QuantixError::Other(error.to_string())),
    }
}
