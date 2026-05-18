use std::time::Duration;

use rust_decimal::Decimal;
use tokio::time::{Instant, sleep};

use crate::bridge::client::BridgeHttpClient;
use crate::bridge::error::BridgeError;
use crate::bridge::models::{
    BridgeBrokerEventType, BridgeFailureCode, BridgeTaskExecuteParams, BridgeTaskExecuteRequest,
    BridgeTaskLifecycleStatus, BridgeTaskResultPayload,
};
use crate::execution::adapter::AdapterOrderRequest;
use crate::execution::models::{OrderSide, OrderStatus};

#[derive(Debug, Clone)]
pub struct QmtTaskSubmitService {
    client: BridgeHttpClient,
    poll_interval: Duration,
    poll_timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QmtTaskSubmitReceipt {
    pub task_id: String,
    pub client_order_id: String,
    pub local_submission_id: String,
    pub bridge_contract_version: String,
    pub source_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QmtTaskResolvedResult {
    pub adapter_order_id: String,
    pub latest_status: OrderStatus,
    pub filled_quantity: i64,
    pub avg_fill_price: Option<Decimal>,
    pub rejection_reason: Option<String>,
    pub broker_event_type: Option<BridgeBrokerEventType>,
    pub external_order_id: Option<String>,
    pub client_order_id: Option<String>,
    pub local_submission_id: Option<String>,
    pub source_name: Option<String>,
}

impl QmtTaskSubmitService {
    pub fn new(
        client: BridgeHttpClient,
        poll_interval_ms: u64,
        poll_timeout_ms: u64,
    ) -> Result<Self, BridgeError> {
        if poll_interval_ms == 0 {
            return Err(BridgeError::Config(
                "qmt task poll interval must be greater than zero".to_string(),
            ));
        }

        if poll_timeout_ms == 0 {
            return Err(BridgeError::Config(
                "qmt task poll timeout must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            client,
            poll_interval: Duration::from_millis(poll_interval_ms),
            poll_timeout: Duration::from_millis(poll_timeout_ms),
        })
    }

    pub async fn submit_order(
        &self,
        request: &AdapterOrderRequest,
    ) -> Result<QmtTaskSubmitReceipt, BridgeError> {
        let local_submission_id = uuid::Uuid::new_v4().to_string();
        let payload = BridgeTaskExecuteRequest {
            provider: "qmt".to_string(),
            method: "submit_order".to_string(),
            params: BridgeTaskExecuteParams {
                request_id: uuid::Uuid::new_v4().to_string(),
                client_order_id: request.client_order_id.clone(),
                local_submission_id: local_submission_id.clone(),
                symbol: request.symbol.clone(),
                side: side_to_bridge(&request.side).to_string(),
                quantity: request.quantity,
                price: request.price.to_string(),
                order_type: order_type_to_bridge(&request.price).to_string(),
                strategy_name: None,
                order_remark: None,
                snapshot_metadata: None,
            },
        };

        let receipt = self.client.task_execute_qmt_submit(&payload).await?;
        if receipt.status != BridgeTaskLifecycleStatus::BridgeTaskAccepted {
            return Err(BridgeError::Protocol(format!(
                "unexpected task receipt status: {:?}",
                receipt.status
            )));
        }

        Ok(QmtTaskSubmitReceipt {
            task_id: receipt.task_id,
            client_order_id: request.client_order_id.clone(),
            local_submission_id,
            bridge_contract_version: receipt.bridge_contract_version,
            source_name: receipt.source_name,
        })
    }

    pub async fn query_task_result_once(
        &self,
        task_id: &str,
        client_order_id: &str,
        local_submission_id: &str,
    ) -> Result<QmtTaskResolvedResult, BridgeError> {
        self.query_task_result_internal(task_id, Some((client_order_id, local_submission_id)))
            .await
    }

    pub async fn query_task_result_by_task_id(
        &self,
        task_id: &str,
    ) -> Result<QmtTaskResolvedResult, BridgeError> {
        self.query_task_result_internal(task_id, None).await
    }

    pub async fn poll_task_result_until_terminal(
        &self,
        task_id: &str,
        client_order_id: &str,
        local_submission_id: &str,
    ) -> Result<QmtTaskResolvedResult, BridgeError> {
        let deadline = Instant::now() + self.poll_timeout;

        loop {
            let result = self
                .query_task_result_once(task_id, client_order_id, local_submission_id)
                .await?;

            if result.latest_status != OrderStatus::PendingSubmit {
                return Ok(result);
            }

            if Instant::now() >= deadline {
                return Err(BridgeError::Timeout(format!(
                    "task result polling timed out for task_id={task_id}"
                )));
            }

            sleep(self.poll_interval).await;
        }
    }

    async fn query_task_result_internal(
        &self,
        task_id: &str,
        expected_identity: Option<(&str, &str)>,
    ) -> Result<QmtTaskResolvedResult, BridgeError> {
        let response = self.client.task_result(task_id).await?;

        match response.status {
            BridgeTaskLifecycleStatus::Pending | BridgeTaskLifecycleStatus::BridgeTaskAccepted => {
                Ok(QmtTaskResolvedResult {
                    adapter_order_id: task_id.to_string(),
                    latest_status: OrderStatus::PendingSubmit,
                    filled_quantity: 0,
                    avg_fill_price: None,
                    rejection_reason: None,
                    broker_event_type: None,
                    external_order_id: None,
                    client_order_id: None,
                    local_submission_id: None,
                    source_name: None,
                })
            }
            BridgeTaskLifecycleStatus::Completed => {
                let payload = response.result.ok_or_else(|| {
                    BridgeError::Protocol("completed task result missing payload".to_string())
                })?;
                validate_identity(&payload, expected_identity)?;
                Ok(map_completed_result(task_id, payload))
            }
            BridgeTaskLifecycleStatus::Failed => {
                let payload = response.result.ok_or_else(|| {
                    BridgeError::Protocol("failed task result missing payload".to_string())
                })?;
                validate_identity(&payload, expected_identity)?;
                Err(map_failure_payload(payload))
            }
        }
    }
}

fn validate_identity(
    payload: &BridgeTaskResultPayload,
    expected_identity: Option<(&str, &str)>,
) -> Result<(), BridgeError> {
    let Some((expected_client_order_id, expected_local_submission_id)) = expected_identity else {
        return Ok(());
    };

    if payload.client_order_id != expected_client_order_id {
        return Err(BridgeError::InvalidResult(format!(
            "task result client_order_id mismatch: expected={}, actual={}",
            expected_client_order_id, payload.client_order_id
        )));
    }

    if payload.local_submission_id != expected_local_submission_id {
        return Err(BridgeError::InvalidResult(format!(
            "task result local_submission_id mismatch: expected={}, actual={}",
            expected_local_submission_id, payload.local_submission_id
        )));
    }

    Ok(())
}

fn map_completed_result(task_id: &str, payload: BridgeTaskResultPayload) -> QmtTaskResolvedResult {
    let rejection_reason = payload
        .reason_detail
        .clone()
        .or_else(|| payload.reason_code.map(failure_code_name));

    let latest_status = match payload.broker_event_type {
        Some(BridgeBrokerEventType::Acknowledgement) => OrderStatus::Accepted,
        Some(BridgeBrokerEventType::Reject) => OrderStatus::Rejected,
        Some(BridgeBrokerEventType::Execution) => OrderStatus::Filled,
        None => OrderStatus::Unknown,
    };

    QmtTaskResolvedResult {
        adapter_order_id: task_id.to_string(),
        latest_status,
        filled_quantity: 0,
        avg_fill_price: None,
        rejection_reason,
        broker_event_type: payload.broker_event_type,
        external_order_id: payload.external_order_id,
        client_order_id: Some(payload.client_order_id),
        local_submission_id: Some(payload.local_submission_id),
        source_name: Some(payload.source_name),
    }
}

fn map_failure_payload(payload: BridgeTaskResultPayload) -> BridgeError {
    let message = payload
        .reason_detail
        .or_else(|| payload.reason_code.map(failure_code_name))
        .unwrap_or_else(|| "bridge task failed without reason".to_string());

    match payload.reason_code {
        Some(BridgeFailureCode::LiveBridgeTimeout) => BridgeError::Timeout(message),
        Some(BridgeFailureCode::LiveBridgeUnavailable) => BridgeError::Unavailable(message),
        Some(BridgeFailureCode::LiveBridgeAuthFailed) => BridgeError::Unauthorized(message),
        Some(BridgeFailureCode::LiveBridgeUnsupportedContractVersion) => {
            BridgeError::UnsupportedContractVersion(message)
        }
        Some(BridgeFailureCode::LiveBridgeUnsupportedMethod) => {
            BridgeError::UnsupportedMethod(message)
        }
        Some(BridgeFailureCode::LiveBridgeInvalidResult)
        | Some(BridgeFailureCode::LiveBridgeIdentityMismatch) => {
            BridgeError::InvalidResult(message)
        }
        None => BridgeError::Protocol(message),
    }
}

fn failure_code_name(code: BridgeFailureCode) -> String {
    match code {
        BridgeFailureCode::LiveBridgeTimeout => "live_bridge_timeout".to_string(),
        BridgeFailureCode::LiveBridgeUnavailable => "live_bridge_unavailable".to_string(),
        BridgeFailureCode::LiveBridgeAuthFailed => "live_bridge_auth_failed".to_string(),
        BridgeFailureCode::LiveBridgeUnsupportedContractVersion => {
            "live_bridge_unsupported_contract_version".to_string()
        }
        BridgeFailureCode::LiveBridgeUnsupportedMethod => {
            "live_bridge_unsupported_method".to_string()
        }
        BridgeFailureCode::LiveBridgeInvalidResult => "live_bridge_invalid_result".to_string(),
        BridgeFailureCode::LiveBridgeIdentityMismatch => {
            "live_bridge_identity_mismatch".to_string()
        }
    }
}

fn side_to_bridge(side: &OrderSide) -> &'static str {
    match side {
        OrderSide::Buy => "buy",
        OrderSide::Sell => "sell",
    }
}

fn order_type_to_bridge(price: &Decimal) -> &'static str {
    if price.is_zero() { "market" } else { "limit" }
}
