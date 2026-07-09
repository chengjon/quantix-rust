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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QmtLiveCapabilityValue {
    Known(String),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QmtLiveCapabilityReadiness {
    Ready,
    Disabled,
    NonLiveMode,
    MissingOrderSubmit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QmtLiveCompatibilityDescriptor {
    pub readiness: QmtLiveCapabilityReadiness,
    pub missing_required_supports: Vec<String>,
}

impl QmtLiveCompatibilityDescriptor {
    fn from_bridge_capabilities(
        qmt_enabled: bool,
        qmt_mode: &str,
        qmt_supports: &[String],
    ) -> Self {
        let order_submit_supported = qmt_supports
            .iter()
            .any(|supported| supported == "order_submit");
        let missing_required_supports = if order_submit_supported {
            Vec::new()
        } else {
            vec!["order_submit".to_string()]
        };
        let readiness = if !qmt_enabled {
            QmtLiveCapabilityReadiness::Disabled
        } else if qmt_mode != "live" {
            QmtLiveCapabilityReadiness::NonLiveMode
        } else if !order_submit_supported {
            QmtLiveCapabilityReadiness::MissingOrderSubmit
        } else {
            QmtLiveCapabilityReadiness::Ready
        };

        Self {
            readiness,
            missing_required_supports,
        }
    }

    /// 判断当前 bridge 的 qmt.live 能力是否已就绪（Enabled + live 模式 + 支持 order_submit 三者全满足）。
    pub fn is_ready(&self) -> bool {
        self.readiness == QmtLiveCapabilityReadiness::Ready
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QmtLiveCapabilitySnapshot {
    pub qmt_enabled: bool,
    pub qmt_mode: String,
    pub qmt_supports: Vec<String>,
    pub compatibility: QmtLiveCompatibilityDescriptor,
    pub bridge_contract_version: QmtLiveCapabilityValue,
    pub miniqmt_version: QmtLiveCapabilityValue,
}

impl QmtLiveCapabilitySnapshot {
    /// 判断 snapshot 的 qmt.supports 列表中是否包含指定能力（精确字符串匹配）。
    pub fn supports(&self, capability: &str) -> bool {
        self.qmt_supports
            .iter()
            .any(|supported| supported == capability)
    }

    /// 判断当前 snapshot 是否满足 qmt_live 提交条件（委托给 compatibility.is_ready）。
    pub fn is_live_order_submit_ready(&self) -> bool {
        self.compatibility.is_ready()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QmtLiveErrorCategory {
    LocalValidationRejected,
    LocalRiskGateRejected,
    BridgeFailure,
    BridgeTimeout,
    BridgeUnavailable,
    BridgeAuthFailed,
    BridgeUnsupportedContractVersion,
    BridgeUnsupportedMethod,
    BridgeProtocolViolation,
    BridgeHttpFailure,
    BridgeInvalidResult,
    BrokerRejected,
    BrokerUnknownState,
    TaskIdentityMismatch,
    ManualInterventionRequired,
}

impl QmtLiveErrorCategory {
    /// 返回该错误类别的稳定字符串标识，用于写入 diagnostics 与日志。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalValidationRejected => "local_validation_rejected",
            Self::LocalRiskGateRejected => "local_risk_gate_rejected",
            Self::BridgeFailure => "bridge_failure",
            Self::BridgeTimeout => "bridge_timeout",
            Self::BridgeUnavailable => "bridge_unavailable",
            Self::BridgeAuthFailed => "bridge_auth_failed",
            Self::BridgeUnsupportedContractVersion => "bridge_unsupported_contract_version",
            Self::BridgeUnsupportedMethod => "bridge_unsupported_method",
            Self::BridgeProtocolViolation => "bridge_protocol_violation",
            Self::BridgeHttpFailure => "bridge_http_failure",
            Self::BridgeInvalidResult => "bridge_invalid_result",
            Self::BrokerRejected => "broker_rejected",
            Self::BrokerUnknownState => "broker_unknown_state",
            Self::TaskIdentityMismatch => "task_identity_mismatch",
            Self::ManualInterventionRequired => "manual_intervention_required",
        }
    }

    /// 把 BridgeError 映射到本地 QmtLiveErrorCategory；InvalidResult 中含 identity mismatch 关键字时归类为 TaskIdentityMismatch。
    pub fn from_bridge_error(error: &BridgeError) -> Self {
        match error {
            BridgeError::Config(_) => Self::LocalValidationRejected,
            BridgeError::Timeout(_) => Self::BridgeTimeout,
            BridgeError::Unavailable(_) => Self::BridgeUnavailable,
            BridgeError::Unauthorized(_) => Self::BridgeAuthFailed,
            BridgeError::UnsupportedContractVersion(_) => Self::BridgeUnsupportedContractVersion,
            BridgeError::UnsupportedMethod(_) => Self::BridgeUnsupportedMethod,
            BridgeError::Protocol(_) => Self::BridgeProtocolViolation,
            BridgeError::Http(_) => Self::BridgeHttpFailure,
            BridgeError::InvalidResult(message) if is_identity_mismatch_error(message) => {
                Self::TaskIdentityMismatch
            }
            BridgeError::InvalidResult(_) => Self::BridgeInvalidResult,
        }
    }

    /// 根据 task 终态结果推导错误类别：Rejected → BrokerRejected，Unknown → BrokerUnknownState，其余返回 `None`。
    pub fn from_task_result(result: &QmtTaskResolvedResult) -> Option<Self> {
        match result.latest_status {
            OrderStatus::Rejected => Some(Self::BrokerRejected),
            OrderStatus::Unknown => Some(Self::BrokerUnknownState),
            _ => None,
        }
    }
}

fn is_identity_mismatch_error(message: &str) -> bool {
    message.contains("client_order_id mismatch")
        || message.contains("local_submission_id mismatch")
        || message.contains("live_bridge_identity_mismatch")
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
    /// 构造 QMT 任务提交服务；poll_interval_ms / poll_timeout_ms 为 0 时返回 BridgeError::Config。
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

    /// 拉取 bridge /api/v1/capabilities 并聚合为 QmtLiveCapabilitySnapshot；bridge_contract_version / miniqmt_version 字段暂以 Unknown 占位。
    pub async fn qmt_live_capability_snapshot(
        &self,
    ) -> Result<QmtLiveCapabilitySnapshot, BridgeError> {
        let capabilities = self.client.capabilities().await?;
        let qmt_enabled = capabilities.qmt.enabled;
        let qmt_mode = capabilities.qmt.mode;
        let qmt_supports = capabilities.qmt.supports;
        let compatibility = QmtLiveCompatibilityDescriptor::from_bridge_capabilities(
            qmt_enabled,
            &qmt_mode,
            &qmt_supports,
        );

        Ok(QmtLiveCapabilitySnapshot {
            qmt_enabled,
            qmt_mode,
            qmt_supports,
            compatibility,
            bridge_contract_version: QmtLiveCapabilityValue::Unknown,
            miniqmt_version: QmtLiveCapabilityValue::Unknown,
        })
    }

    /// 通过 bridge task_execute qmt submit_order 提交订单，返回任务回执；bridge 返回非 BridgeTaskAccepted 时以 BridgeError::Protocol 失败。
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

    /// 单次查询指定 task_id 的结果，并在返回 Completed/Failed 时校验 (client_order_id, local_submission_id) 一致性。
    pub async fn query_task_result_once(
        &self,
        task_id: &str,
        client_order_id: &str,
        local_submission_id: &str,
    ) -> Result<QmtTaskResolvedResult, BridgeError> {
        self.query_task_result_internal(task_id, Some((client_order_id, local_submission_id)))
            .await
    }

    /// 仅按 task_id 单次查询结果（不做 identity 校验），适合从异常恢复路径中按 adapter_order_id 反查。
    pub async fn query_task_result_by_task_id(
        &self,
        task_id: &str,
    ) -> Result<QmtTaskResolvedResult, BridgeError> {
        self.query_task_result_internal(task_id, None).await
    }

    /// 周期性轮询 task 结果直到脱离 PendingSubmit 或超过 poll_timeout；超时返回 BridgeError::Timeout。
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
