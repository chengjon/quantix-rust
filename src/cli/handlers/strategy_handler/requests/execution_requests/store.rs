use crate::core::{QuantixError, Result};
use crate::execution::models::{ExecutionRequestRecord, ExecutionRequestStatus};
use crate::execution::runtime_store::StrategyRuntimeStore;

pub(crate) async fn execute_strategy_request_list_with_store(
    store: &StrategyRuntimeStore,
    status: Option<&str>,
) -> Result<Vec<ExecutionRequestRecord>> {
    let status_filter = status.map(parse_execution_request_status).transpose()?;
    store.list_execution_requests(status_filter).await
}

pub(crate) fn parse_execution_request_status(value: &str) -> Result<ExecutionRequestStatus> {
    ExecutionRequestStatus::from_str(value)
        .ok_or_else(|| QuantixError::Unsupported(format!("未知 request_status: {value}")))
}

pub(crate) fn merge_execution_request_payload(
    original: &serde_json::Value,
    key: &str,
    value: serde_json::Value,
) -> serde_json::Value {
    let mut payload = match original {
        serde_json::Value::Object(map) => serde_json::Value::Object(map.clone()),
        _ => serde_json::json!({}),
    };
    payload[key] = value;
    payload
}
