use crate::core::Result;
use crate::execution::models::{ExecutionRequestRecord, OrderRecord};
use crate::execution::runtime_store::StrategyRuntimeStore;

pub(crate) async fn find_related_order_for_request(
    store: &StrategyRuntimeStore,
    request: &ExecutionRequestRecord,
) -> Result<Option<OrderRecord>> {
    let Some(client_order_id) = request
        .payload_json
        .get("execution_result")
        .and_then(|value| value.get("client_order_id"))
        .and_then(|value| value.as_str())
    else {
        return Ok(None);
    };

    store.find_order_by_client_order_id(client_order_id).await
}

pub(crate) fn qmt_live_compact_summary(order: &OrderRecord) -> Option<String> {
    if order.adapter != "qmt_live" {
        return None;
    }

    let qmt_live = order.payload_json.get("qmt_live");
    let task_id = qmt_live
        .and_then(|value| value.get("task_identity"))
        .and_then(|value| value.get("task_id"))
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty());
    let latest_status = qmt_live
        .and_then(|value| value.get("last_query"))
        .and_then(|value| value.get("latest_status"))
        .and_then(|value| value.as_str());
    let last_action = qmt_live
        .and_then(|value| value.get("reconciliation"))
        .and_then(|value| value.get("last_action"))
        .and_then(|value| value.as_str());

    let mut parts = Vec::new();
    match task_id {
        Some(task_id) => parts.push(format!("qmt_task_id={task_id}")),
        None => parts.push("qmt_recovery=unavailable".to_string()),
    }
    if let Some(latest_status) = latest_status {
        parts.push(format!("qmt_latest_status={latest_status}"));
    }
    if let Some(last_action) = last_action {
        parts.push(format!("qmt_last_action={last_action}"));
    }

    Some(parts.join(" "))
}

pub(crate) fn append_qmt_live_recovery_detail(
    lines: &mut Vec<String>,
    related_order: Option<&OrderRecord>,
) {
    let Some(order) = related_order else {
        return;
    };
    if order.adapter != "qmt_live" {
        return;
    }

    let qmt_live = order.payload_json.get("qmt_live");
    let task_identity = qmt_live.and_then(|value| value.get("task_identity"));
    let task_id = task_identity
        .and_then(|value| value.get("task_id"))
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty());
    let last_query = qmt_live.and_then(|value| value.get("last_query"));
    let reconciliation = qmt_live.and_then(|value| value.get("reconciliation"));

    lines.push(String::new());
    lines.push("=== QMT Live Recovery ===".to_string());
    match task_id {
        Some(task_id) => lines.push(format!("task_id: {task_id}")),
        None => lines.push("automatic_reconciliation: unavailable".to_string()),
    }
    if let Some(client_order_id) = task_identity
        .and_then(|value| value.get("client_order_id"))
        .and_then(|value| value.as_str())
    {
        lines.push(format!("client_order_id: {client_order_id}"));
    }
    if let Some(local_submission_id) = task_identity
        .and_then(|value| value.get("local_submission_id"))
        .and_then(|value| value.as_str())
    {
        lines.push(format!("local_submission_id: {local_submission_id}"));
    }
    if let Some(external_order_id) = task_identity
        .and_then(|value| value.get("external_order_id"))
        .and_then(|value| value.as_str())
    {
        lines.push(format!("external_order_id: {external_order_id}"));
    }
    if let Some(last_query) = last_query {
        if let Some(latest_status) = last_query
            .get("latest_status")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("latest_status: {latest_status}"));
        }
        if let Some(filled_quantity) = last_query
            .get("filled_quantity")
            .and_then(|value| value.as_i64())
        {
            lines.push(format!("filled_quantity: {filled_quantity}"));
        }
        if let Some(avg_fill_price) = last_query
            .get("avg_fill_price")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("avg_fill_price: {avg_fill_price}"));
        }
        if let Some(broker_event_type) = last_query
            .get("broker_event_type")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("broker_event_type: {broker_event_type}"));
        }
        if let Some(rejection_reason) = last_query
            .get("rejection_reason")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("rejection_reason: {rejection_reason}"));
        }
        if let Some(updated_at) = last_query
            .get("updated_at")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("last_query_updated_at: {updated_at}"));
        }
    }
    if let Some(reconciliation) = reconciliation {
        if let Some(last_action) = reconciliation
            .get("last_action")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("last_action: {last_action}"));
        }
        if let Some(last_error) = reconciliation
            .get("last_error")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("last_error: {last_error}"));
        }
        if let Some(last_attempt_at) = reconciliation
            .get("last_attempt_at")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("last_attempt_at: {last_attempt_at}"));
        }
    }
}
