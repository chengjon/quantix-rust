use super::*;
use crate::bridge::client::BridgeHttpClient;
use crate::bridge::error::BridgeError;
use crate::core::{CliRuntime, QuantixError, Result};
use crate::execution::adapter::{
    AdapterOrderRequest, ExecutionAdapter, ExecutionCancelSemantics, ExecutionCapabilities,
    ExecutionChannel, ExecutionFillSource, ExecutionStatusSource,
};
use crate::execution::config::JsonExecutionConfigStore;
use crate::execution::daemon::{
    ExecutionDaemonIterationSummary, consume_next_pending_request_with_components,
};
use crate::execution::models::{
    ExecutionRequestRecord, ExecutionRequestStatus, OrderRecord, OrderSide, OrderStatus, OrderType,
    QmtLiveRuntimeMetadata, QmtLiveTaskIdentity,
};
use crate::execution::qmt_bridge::QmtBridgePreviewAdapter;
use crate::execution::qmt_live_adapter::QmtLiveExecutionAdapter;
use crate::execution::qmt_live_gate::QmtLiveGateFailure;
use crate::execution::qmt_task_submit_service::QmtTaskSubmitService;
use crate::execution::request_diagnostics::{
    build_bridge_qmt_capability_check_failed_diagnostics,
    build_bridge_qmt_capability_disabled_diagnostics, build_bridge_qmt_mode_not_live_diagnostics,
    build_bridge_qmt_order_submit_capability_missing_diagnostics, build_completion_diagnostics,
    build_kill_switch_blocked_diagnostics, build_unclassified_execution_error_diagnostics,
};
use crate::execution::runtime_store::StrategyRuntimeStore;
use crate::safety::{
    JsonKillSwitchStore, KillSwitchState, build_kill_switch_payload,
    format_execution_kill_switch_block_message, load_blocking_kill_switch_state,
};
use chrono::Utc;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::Duration;

fn create_execution_config_store() -> JsonExecutionConfigStore {
    let runtime = CliRuntime::load();
    JsonExecutionConfigStore::new(runtime.execution_config_path)
}

pub fn create_bridge_client() -> Result<BridgeHttpClient> {
    let runtime = CliRuntime::load();
    BridgeHttpClient::new(runtime.bridge.base_url, runtime.bridge.api_key)
        .map_err(|err| QuantixError::Other(err.to_string()))
}

pub(crate) async fn execute_execution_config_init() -> Result<()> {
    let config = create_execution_config_store().load_or_create()?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(crate) async fn execute_execution_config_show() -> Result<()> {
    let config = create_execution_config_store().load_or_create()?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(crate) async fn execute_execution_daemon_run(once: bool) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let config_store = JsonExecutionConfigStore::new(runtime.execution_config_path);
    let config = config_store.load_or_create()?;
    let trade_store = create_trade_store();
    let risk_store = create_risk_store();

    if once {
        let summary =
            consume_next_pending_request_with_components(&runtime_store, trade_store, risk_store)
                .await?;
        print_execution_daemon_summary(&summary);
        return Ok(());
    }

    loop {
        let summary = consume_next_pending_request_with_components(
            &runtime_store,
            trade_store.clone(),
            risk_store.clone(),
        )
        .await?;
        print_execution_daemon_summary(&summary);
        tokio::time::sleep(Duration::from_secs(config.poll_interval_secs)).await;
    }
}

pub(crate) fn format_qmt_promotion_checklist(
    capabilities: &crate::bridge::models::BridgeCapabilitiesResponse,
    qmt_live_capabilities: ExecutionCapabilities,
) -> String {
    let qmt_enabled = capabilities.qmt.enabled;
    let qmt_mode_live = capabilities.qmt.mode == "live";
    let order_submit_supported = capabilities
        .qmt
        .supports
        .iter()
        .any(|item| item == "order_submit");
    let status_mark = |ok: bool| if ok { "[ok]" } else { "[x]" };
    let qmt_live_channel = qmt_live_capabilities.channel == ExecutionChannel::QmtLive;
    let broker_status_source = qmt_live_capabilities.status_source == ExecutionStatusSource::Broker;
    let broker_fill_source = qmt_live_capabilities.fill_source == ExecutionFillSource::Broker;
    let broker_cancel_semantics =
        qmt_live_capabilities.cancel_semantics == ExecutionCancelSemantics::Broker;

    [
        "QMT promotion checklist".to_string(),
        format!("{} bridge qmt.enabled=true", status_mark(qmt_enabled)),
        format!("{} bridge qmt.mode=live", status_mark(qmt_mode_live)),
        format!(
            "{} bridge qmt.supports 包含 order_submit",
            status_mark(order_submit_supported)
        ),
        format!(
            "{} qmt_live adapter channel={}",
            status_mark(qmt_live_channel),
            qmt_live_capabilities.channel.as_str()
        ),
        format!(
            "{} qmt_live status_source={}",
            status_mark(broker_status_source),
            qmt_live_capabilities.status_source.as_str()
        ),
        format!(
            "{} qmt_live fill_source={}",
            status_mark(broker_fill_source),
            qmt_live_capabilities.fill_source.as_str()
        ),
        format!(
            "{} qmt_live cancel_semantics={}",
            status_mark(broker_cancel_semantics),
            qmt_live_capabilities.cancel_semantics.as_str()
        ),
        "[ ] request target_mode=qmt_live".to_string(),
        "[ ] 先在 paper 路径验证策略与风控".to_string(),
        "[ ] 再在 mock_live 路径验证非终态与收敛".to_string(),
        "[ ] 预览提交 payload: quantix execution qmt preview --request-id <ID>".to_string(),
        "[ ] 真实提交订单: quantix execution qmt live --request-id <ID> [--yes]".to_string(),
        "[ ] 查看 request 与收敛状态: quantix strategy request show <ID> --verbose".to_string(),
    ]
    .join("\n")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QmtLivePreflightFailureCategory {
    BridgeUnreachable,
    QmtCapabilityMissing,
    QmtDisabled,
    QmtModeNotLive,
    QmtOrderSubmitMissing,
    QmtLiveCapabilityMismatch,
    KillSwitchEnabled,
}

impl QmtLivePreflightFailureCategory {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::BridgeUnreachable => "bridge_unreachable",
            Self::QmtCapabilityMissing => "qmt_capability_missing",
            Self::QmtDisabled => "qmt_disabled",
            Self::QmtModeNotLive => "qmt_mode_not_live",
            Self::QmtOrderSubmitMissing => "qmt_order_submit_missing",
            Self::QmtLiveCapabilityMismatch => "qmt_live_capability_mismatch",
            Self::KillSwitchEnabled => "kill_switch_enabled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QmtLivePreflightReport {
    pub(crate) ready: bool,
    pub(crate) failure_category: Option<QmtLivePreflightFailureCategory>,
    pub(crate) bridge_reachable: bool,
    pub(crate) bridge_error: Option<String>,
    pub(crate) bridge_contract_version: String,
    pub(crate) capability_source: String,
    pub(crate) qmt_enabled: Option<bool>,
    pub(crate) qmt_mode: Option<String>,
    pub(crate) order_submit_supported: Option<bool>,
    pub(crate) qmt_live_channel: bool,
    pub(crate) broker_status_source: bool,
    pub(crate) broker_fill_source: bool,
    pub(crate) broker_cancel_semantics: bool,
    pub(crate) kill_switch_enabled: bool,
    pub(crate) kill_switch_reason: Option<String>,
}

pub(crate) fn build_qmt_live_preflight_report(
    capabilities: Option<&crate::bridge::models::BridgeCapabilitiesResponse>,
    bridge_error: Option<&str>,
    qmt_live_capabilities: ExecutionCapabilities,
    kill_switch_state: Option<&KillSwitchState>,
) -> QmtLivePreflightReport {
    let qmt_live_channel = qmt_live_capabilities.channel == ExecutionChannel::QmtLive;
    let broker_status_source = qmt_live_capabilities.status_source == ExecutionStatusSource::Broker;
    let broker_fill_source = qmt_live_capabilities.fill_source == ExecutionFillSource::Broker;
    let broker_cancel_semantics =
        qmt_live_capabilities.cancel_semantics == ExecutionCancelSemantics::Broker;
    let kill_switch_enabled = kill_switch_state
        .map(|state| state.enabled)
        .unwrap_or(false);

    let (qmt_enabled, qmt_mode, order_submit_supported) = capabilities
        .map(|capabilities| {
            (
                Some(capabilities.qmt.enabled),
                Some(capabilities.qmt.mode.clone()),
                Some(
                    capabilities
                        .qmt
                        .supports
                        .iter()
                        .any(|item| item == "order_submit"),
                ),
            )
        })
        .unwrap_or((None, None, None));

    let failure_category = if bridge_error.is_some() {
        Some(QmtLivePreflightFailureCategory::BridgeUnreachable)
    } else if capabilities.is_none() {
        Some(QmtLivePreflightFailureCategory::QmtCapabilityMissing)
    } else if qmt_enabled == Some(false) {
        Some(QmtLivePreflightFailureCategory::QmtDisabled)
    } else if qmt_mode.as_deref() != Some("live") {
        Some(QmtLivePreflightFailureCategory::QmtModeNotLive)
    } else if order_submit_supported != Some(true) {
        Some(QmtLivePreflightFailureCategory::QmtOrderSubmitMissing)
    } else if !(qmt_live_channel
        && broker_status_source
        && broker_fill_source
        && broker_cancel_semantics)
    {
        Some(QmtLivePreflightFailureCategory::QmtLiveCapabilityMismatch)
    } else if kill_switch_enabled {
        Some(QmtLivePreflightFailureCategory::KillSwitchEnabled)
    } else {
        None
    };

    QmtLivePreflightReport {
        ready: failure_category.is_none(),
        failure_category,
        bridge_reachable: bridge_error.is_none() && capabilities.is_some(),
        bridge_error: bridge_error.map(ToOwned::to_owned),
        bridge_contract_version: "unknown".to_string(),
        capability_source: "bridge:/api/v1/capabilities".to_string(),
        qmt_enabled,
        qmt_mode,
        order_submit_supported,
        qmt_live_channel,
        broker_status_source,
        broker_fill_source,
        broker_cancel_semantics,
        kill_switch_enabled,
        kill_switch_reason: kill_switch_state.and_then(|state| state.reason.clone()),
    }
}

fn qmt_live_preflight_report_json(report: &QmtLivePreflightReport) -> serde_json::Value {
    serde_json::json!({
        "ready": report.ready,
        "failure_category": report
            .failure_category
            .map(QmtLivePreflightFailureCategory::as_str),
        "bridge_reachable": report.bridge_reachable,
        "bridge_error": report.bridge_error,
        "bridge_contract_version": report.bridge_contract_version,
        "capability_source": report.capability_source,
        "qmt": {
            "enabled": report.qmt_enabled,
            "mode": report.qmt_mode,
            "order_submit_supported": report.order_submit_supported
        },
        "qmt_live_capabilities": {
            "channel": report.qmt_live_channel,
            "broker_status_source": report.broker_status_source,
            "broker_fill_source": report.broker_fill_source,
            "broker_cancel_semantics": report.broker_cancel_semantics
        },
        "kill_switch": {
            "enabled": report.kill_switch_enabled,
            "reason": report.kill_switch_reason
        }
    })
}

pub(crate) fn format_qmt_live_preflight_report(report: &QmtLivePreflightReport) -> String {
    let readiness = if report.ready { "ready" } else { "not_ready" };
    let failure_category = report
        .failure_category
        .map(QmtLivePreflightFailureCategory::as_str)
        .unwrap_or("none");
    let kill_switch = if report.kill_switch_enabled {
        "enabled"
    } else {
        "disabled"
    };

    [
        "QMT live preflight".to_string(),
        format!("readiness={readiness}"),
        format!("failure_category={failure_category}"),
        format!("bridge_reachable={}", report.bridge_reachable),
        format!("bridge_contract_version={}", report.bridge_contract_version),
        format!("capability_source={}", report.capability_source),
        format!("kill_switch={kill_switch}"),
    ]
    .join("\n")
}

pub(crate) async fn execute_execution_bridge_status(checklist: bool) -> Result<()> {
    let bridge_client = create_bridge_client()?;
    let qmt_live_capabilities = QmtLiveExecutionAdapter::new(bridge_client.clone()).capabilities();
    let capabilities_result = bridge_client.capabilities().await;
    let bridge_error = capabilities_result.as_ref().err().map(ToString::to_string);
    let kill_switch_state = if checklist {
        Some(JsonKillSwitchStore::with_default_path()?.load_or_default()?)
    } else {
        None
    };
    let preflight_report = build_qmt_live_preflight_report(
        capabilities_result.as_ref().ok(),
        bridge_error.as_deref(),
        qmt_live_capabilities,
        kill_switch_state.as_ref(),
    );

    let capabilities = match capabilities_result {
        Ok(capabilities) => capabilities,
        Err(err) if checklist => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "bridge_status_error": err.to_string(),
                    "qmt_live_preflight": qmt_live_preflight_report_json(&preflight_report)
                }))?
            );
            println!();
            println!("{}", format_qmt_live_preflight_report(&preflight_report));
            return Ok(());
        }
        Err(err) => {
            return Err(QuantixError::Other(format!(
                "bridge status 查询失败: {err}"
            )));
        }
    };

    let mut status_payload = serde_json::json!({
            "tdx": {
                "enabled": capabilities.tdx.enabled,
                "supports": capabilities.tdx.supports
            },
            "qmt": {
                "enabled": capabilities.qmt.enabled,
                "mode": capabilities.qmt.mode,
                "supports": capabilities.qmt.supports
            }
    });
    if checklist {
        status_payload["qmt_live_preflight"] = qmt_live_preflight_report_json(&preflight_report);
    }

    println!("{}", serde_json::to_string_pretty(&status_payload)?);
    if checklist {
        println!();
        println!(
            "{}",
            format_qmt_promotion_checklist(&capabilities, qmt_live_capabilities)
        );
        println!();
        println!("{}", format_qmt_live_preflight_report(&preflight_report));
    }
    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_preview(request_id: &str) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request = runtime_store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;
    if request.target_mode != "qmt_live" {
        return Err(QuantixError::Unsupported(format!(
            "execution bridge qmt-preview 只支持 target_mode=qmt_live 的 request；当前 request target_mode={}。如需预览 QMT 提交流程，请先创建 qmt_live request；通用 target_mode=live 仍未实现",
            request.target_mode
        )));
    }

    let adapter = QmtBridgePreviewAdapter::new(create_bridge_client()?);
    let preview = adapter.preview_request(&request).await?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "request_id": request_id,
            "adapter_order_id": preview.adapter_order_id,
            "latest_status": preview.latest_status.as_str(),
            "filled_quantity": preview.filled_quantity,
            "rejection_reason": preview.rejection_reason,
            "broker_payload": preview.broker_payload,
        }))?
    );
    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_live(
    request_id: &str,
    skip_confirm: bool,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let kill_switch_store = JsonKillSwitchStore::with_default_path()?;
    execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch(
        &runtime_store,
        &kill_switch_store,
        request_id,
        skip_confirm,
    )
    .await
}

pub(crate) async fn execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch(
    runtime_store: &StrategyRuntimeStore,
    kill_switch_store: &JsonKillSwitchStore,
    request_id: &str,
    skip_confirm: bool,
) -> Result<()> {
    let request = runtime_store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;
    if request.request_status != ExecutionRequestStatus::Pending {
        return Err(QuantixError::Other(format!(
            "request 不是 pending: {request_id}"
        )));
    }
    if request.target_mode != "qmt_live" {
        return Err(QuantixError::Other(format!(
            "request target_mode 不是 qmt_live: {}",
            request.target_mode
        )));
    }

    if let Some(state) =
        load_blocking_kill_switch_state(kill_switch_store, request.target_mode.as_str())?
    {
        let blocked_at = Utc::now();
        let err = QuantixError::Other(format_execution_kill_switch_block_message(
            request.target_mode.as_str(),
            &state,
        ));
        let payload_json = merge_execution_request_payload(
            &request.payload_json,
            "execution_error",
            serde_json::json!({
                "failed_at": blocked_at.to_rfc3339(),
                "message": err.to_string(),
                "adapter": request.target_mode.as_str(),
            }),
        );
        let payload_json = merge_execution_request_payload(
            &payload_json,
            "kill_switch",
            build_kill_switch_payload(&state, request.target_mode.as_str(), blocked_at),
        );
        let payload_json = merge_execution_request_payload(
            &payload_json,
            "execution_diagnostics",
            build_kill_switch_blocked_diagnostics(request.target_mode.as_str()),
        );
        let updated = runtime_store
            .try_fail_pending_execution_request(&request.request_id, payload_json, blocked_at)
            .await?;
        if !updated {
            return Err(QuantixError::Other(format!(
                "request 状态已变化: {}",
                request.request_id
            )));
        }
        return Err(err);
    }

    // 从 payload_json 提取订单信息
    let snapshot = request
        .payload_json
        .get("execution_snapshot")
        .ok_or_else(|| QuantixError::Other("request 缺少 execution_snapshot".to_string()))?;
    let order_intent = snapshot
        .get("order_intent")
        .ok_or_else(|| QuantixError::Other("request 缺少 order_intent".to_string()))?;

    let symbol = snapshot
        .get("symbol")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let side = order_intent
        .get("side")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let quantity = order_intent
        .get("requested_quantity")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let price = order_intent
        .get("requested_price")
        .and_then(|v| v.as_str())
        .unwrap_or("0");
    let strategy_name = snapshot
        .get("strategy_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // 显示订单信息
    println!("═══════════════════════════════════════════════════════════════════");
    println!("⚠️  实盘下单确认");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    println!("  股票代码:    {}", symbol);
    println!("  买卖方向:    {}", side);
    println!("  数量:        {} 股", quantity);
    println!("  价格:        {}", price);
    println!("  策略名称:    {}", strategy_name);
    println!();

    // 确认提示
    if !skip_confirm {
        println!("⚠️  警告: 这将提交真实订单到券商账户!");
        println!("    输入 'YES' 确认下单，其他任意键取消:");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim() != "YES" {
            println!("已取消下单");
            return Ok(());
        }
    }

    // 构建订单请求
    let order_type = order_intent
        .get("order_type")
        .and_then(|v| v.as_str())
        .unwrap_or("limit");
    let side = OrderSide::from_str(side)
        .ok_or_else(|| QuantixError::Other(format!("request 包含无效 side: {side}")))?;
    let order_type = OrderType::from_str(order_type)
        .ok_or_else(|| QuantixError::Other(format!("request 包含无效 order_type: {order_type}")))?;
    let requested_price = Decimal::from_str(price).map_err(|err| {
        QuantixError::Other(format!("request 包含无效 requested_price: {price}, {err}"))
    })?;
    let signal = runtime_store
        .get_signal(&request.signal_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("signal 不存在: {}", request.signal_id)))?;

    let started_at = Utc::now();
    let start_payload = merge_execution_request_payload(
        &request.payload_json,
        "executor",
        serde_json::json!({
            "type": "manual_qmt_live",
            "started_at": started_at.to_rfc3339(),
        }),
    );
    let claimed = runtime_store
        .try_start_execution_request(&request.request_id, start_payload.clone(), started_at)
        .await?;
    if !claimed {
        return Err(QuantixError::Other(format!(
            "request 状态已变化: {}",
            request.request_id
        )));
    }

    // 提交订单
    let client = create_bridge_client()?;
    if let Err(gate_error) =
        crate::execution::qmt_live_gate::check_bridge_qmt_live_mode(&client).await
    {
        let failed_at = Utc::now();
        let error = gate_error.to_quantix_error();
        let diagnostics = match &gate_error {
            QmtLiveGateFailure::CapabilityCheckFailed(_) => {
                build_bridge_qmt_capability_check_failed_diagnostics(&error.to_string())
            }
            QmtLiveGateFailure::CapabilityDisabled => {
                build_bridge_qmt_capability_disabled_diagnostics()
            }
            QmtLiveGateFailure::ModeNotLive { observed_mode } => {
                build_bridge_qmt_mode_not_live_diagnostics(observed_mode)
            }
            QmtLiveGateFailure::MissingOrderSubmitSupport => {
                build_bridge_qmt_order_submit_capability_missing_diagnostics()
            }
        };
        let payload_json = merge_execution_request_payload(
            &start_payload,
            "execution_error",
            serde_json::json!({
                "failed_at": failed_at.to_rfc3339(),
                "message": error.to_string(),
                "adapter": "qmt_live",
            }),
        );
        let payload_json =
            merge_execution_request_payload(&payload_json, "execution_diagnostics", diagnostics);
        let updated = runtime_store
            .try_fail_execution_request(&request.request_id, payload_json, failed_at)
            .await?;
        if !updated {
            return Err(QuantixError::Other(format!(
                "request 状态已变化: {}",
                request.request_id
            )));
        }
        return Err(error);
    }

    let submit_service = QmtTaskSubmitService::new(client.clone(), 1, 30_000)
        .map_err(|err| QuantixError::Other(err.to_string()))?;
    let order_request = AdapterOrderRequest {
        client_order_id: request.request_id.clone(),
        symbol: normalize_symbol_for_bridge(symbol),
        side,
        quantity,
        price: requested_price,
    };

    let receipt = match submit_service.submit_order(&order_request).await {
        Ok(receipt) => receipt,
        Err(err) => {
            let failed_at = Utc::now();
            let payload_json = merge_execution_request_payload(
                &start_payload,
                "execution_error",
                serde_json::json!({
                    "failed_at": failed_at.to_rfc3339(),
                    "message": err.to_string(),
                    "adapter": "qmt_live",
                }),
            );
            let payload_json = merge_execution_request_payload(
                &payload_json,
                "execution_diagnostics",
                build_unclassified_execution_error_diagnostics(&err.to_string()),
            );
            let updated = runtime_store
                .try_fail_execution_request(&request.request_id, payload_json, failed_at)
                .await?;
            if !updated {
                return Err(QuantixError::Other(format!(
                    "request 状态已变化: {}",
                    request.request_id
                )));
            }
            return Err(QuantixError::Other(err.to_string()));
        }
    };

    let finished_at = Utc::now();
    let client_order_id = request.request_id.clone();
    let task_id = receipt.task_id.clone();
    let qmt_live_metadata = QmtLiveRuntimeMetadata {
        task_identity: Some(QmtLiveTaskIdentity {
            task_id: task_id.clone(),
            client_order_id: client_order_id.clone(),
            local_submission_id: receipt.local_submission_id.clone(),
            external_order_id: None,
        }),
        last_query: None,
        reconciliation: None,
    };
    match runtime_store
        .find_order_by_client_order_id(&request.request_id)
        .await?
    {
        Some(existing_order) => {
            let updated = runtime_store
                .try_update_order_qmt_live_metadata(
                    &existing_order,
                    &qmt_live_metadata,
                    finished_at,
                )
                .await?;
            if !updated {
                return Err(QuantixError::Other(format!(
                    "related order qmt_live metadata 更新失败: {}",
                    existing_order.order_id
                )));
            }
        }
        None => {
            let related_order = OrderRecord {
                order_id: client_order_id.clone(),
                client_order_id: client_order_id.clone(),
                run_id: signal.run_id,
                symbol: symbol.to_string(),
                side,
                order_type,
                requested_quantity: quantity,
                requested_price: order_request.price,
                filled_quantity: 0,
                remaining_quantity: quantity,
                avg_fill_price: None,
                status: OrderStatus::PendingSubmit,
                adapter: "qmt_live".to_string(),
                created_at: finished_at,
                updated_at: finished_at,
                last_transition_at: finished_at,
                version: 1,
                payload_json: serde_json::json!({
                    "qmt_live": qmt_live_metadata
                }),
            };
            runtime_store.insert_order(&related_order).await?;
        }
    }

    let payload_json = merge_execution_request_payload(
        &start_payload,
        "execution_result",
        serde_json::json!({
            "executed_at": finished_at.to_rfc3339(),
            "client_order_id": client_order_id,
            "order_status": OrderStatus::PendingSubmit.as_str(),
            "adapter": "qmt_live",
            "adapter_order_id": task_id.clone(),
            "filled_quantity": 0,
            "avg_fill_price": serde_json::Value::Null,
            "rejection_reason": serde_json::Value::Null,
        }),
    );
    let payload_json = merge_execution_request_payload(
        &payload_json,
        "execution_diagnostics",
        build_completion_diagnostics(Some(OrderStatus::PendingSubmit.as_str())),
    );
    let updated = runtime_store
        .try_complete_execution_request(&request.request_id, payload_json, finished_at)
        .await?;
    if !updated {
        return Err(QuantixError::Other(format!(
            "request 状态已变化: {}",
            request.request_id
        )));
    }

    println!();
    println!("✓ 订单提交任务已受理");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "request_id": request_id,
            "adapter_order_id": task_id,
            "latest_status": OrderStatus::PendingSubmit.as_str(),
            "local_submission_id": receipt.local_submission_id,
            "bridge_contract_version": receipt.bridge_contract_version,
            "source_name": receipt.source_name,
        }))?
    );

    println!();
    println!("提示: request 可能会显示为 completed。");
    println!("      这只表示执行层已完成提交，不代表订单已经终态。");
    println!("      订单初始状态通常仍为 pending_submit，请继续跟踪后续收敛。");
    println!();
    println!(
        "查看 request 与后续收敛状态: quantix strategy request show {} --verbose",
        request_id
    );

    Ok(())
}

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

fn create_qmt_task_submit_service(client: &BridgeHttpClient) -> Result<QmtTaskSubmitService> {
    QmtTaskSubmitService::new(client.clone(), 1, 30_000)
        .map_err(|err| QuantixError::Other(err.to_string()))
}

fn should_fallback_from_task_result_lookup(error: &BridgeError) -> bool {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum QmtAuditLookup {
    Request(String),
    Task(String),
    LocalSubmission(String),
}

impl QmtAuditLookup {
    fn from_cli(
        request_id: Option<String>,
        task_id: Option<String>,
        local_submission_id: Option<String>,
    ) -> Result<Self> {
        let request_id = normalize_qmt_audit_lookup_value(request_id);
        let task_id = normalize_qmt_audit_lookup_value(task_id);
        let local_submission_id = normalize_qmt_audit_lookup_value(local_submission_id);

        match (request_id, task_id, local_submission_id) {
            (Some(request_id), None, None) => Ok(Self::Request(request_id)),
            (None, Some(task_id), None) => Ok(Self::Task(task_id)),
            (None, None, Some(local_submission_id)) => Ok(Self::LocalSubmission(local_submission_id)),
            _ => Err(QuantixError::Other(
                "execution qmt audit requires exactly one of --request-id, --task-id, or --local-submission-id".to_string(),
            )),
        }
    }

    fn lookup_type(&self) -> &'static str {
        match self {
            Self::Request(_) => "request_id",
            Self::Task(_) => "task_id",
            Self::LocalSubmission(_) => "local_submission_id",
        }
    }

    fn value(&self) -> &str {
        match self {
            Self::Request(value) | Self::Task(value) | Self::LocalSubmission(value) => value,
        }
    }
}

fn normalize_qmt_audit_lookup_value(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn value_as_audit_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) if !value.is_empty() => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn string_path(value: &serde_json::Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    value_as_audit_string(current)
}

fn i64_path(value: &serde_json::Value, path: &[&str]) -> Option<i64> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    current.as_i64()
}

fn qmt_order_identity_field<'a>(order: &'a OrderRecord, field: &str) -> Option<&'a str> {
    order
        .payload_json
        .get("qmt_live")?
        .get("task_identity")?
        .get(field)?
        .as_str()
        .filter(|value| !value.is_empty())
}

fn qmt_order_metadata_field(order: &OrderRecord, path: &[&str]) -> Option<String> {
    let qmt_live = order.payload_json.get("qmt_live")?;
    string_path(qmt_live, path)
}

fn redact_account_label(value: &str) -> String {
    let trimmed = value.trim();
    let char_count = trimmed.chars().count();
    let looks_raw_account =
        char_count >= 8 && trimmed.chars().all(|character| character.is_ascii_digit());

    if !looks_raw_account {
        return trimmed.to_string();
    }

    let tail_len = 4.min(char_count);
    let tail_start = char_count.saturating_sub(tail_len);
    let tail: String = trimmed.chars().skip(tail_start).collect();
    let mask_len = tail_start.min(14);
    format!("{}{}", "*".repeat(mask_len), tail)
}

async fn resolve_qmt_audit_record(
    runtime_store: &StrategyRuntimeStore,
    lookup: &QmtAuditLookup,
) -> Result<(ExecutionRequestRecord, Option<OrderRecord>)> {
    match lookup {
        QmtAuditLookup::Request(request_id) => {
            let request = runtime_store
                .get_execution_request(request_id)
                .await?
                .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;
            let order = runtime_store
                .find_order_by_client_order_id(&request.request_id)
                .await?;
            Ok((request, order))
        }
        QmtAuditLookup::Task(task_id) => {
            let orders = runtime_store.list_orders().await?;
            if let Some(order) = orders
                .into_iter()
                .find(|order| qmt_order_identity_field(order, "task_id") == Some(task_id.as_str()))
            {
                let request = runtime_store
                    .get_execution_request(&order.client_order_id)
                    .await?
                    .ok_or_else(|| {
                        QuantixError::Other(format!(
                            "qmt_live task_id={task_id} 对应 request 不存在: {}",
                            order.client_order_id
                        ))
                    })?;
                return Ok((request, Some(order)));
            }

            let requests = runtime_store.list_execution_requests(None).await?;
            let request = requests
                .into_iter()
                .find(|request| {
                    string_path(
                        &request.payload_json,
                        &["execution_result", "adapter_order_id"],
                    )
                    .as_deref()
                        == Some(task_id.as_str())
                })
                .ok_or_else(|| {
                    QuantixError::Other(format!("qmt_live task_id 不存在: {task_id}"))
                })?;
            let order = runtime_store
                .find_order_by_client_order_id(&request.request_id)
                .await?;
            Ok((request, order))
        }
        QmtAuditLookup::LocalSubmission(local_submission_id) => {
            let orders = runtime_store.list_orders().await?;
            let order = orders
                .into_iter()
                .find(|order| {
                    qmt_order_identity_field(order, "local_submission_id")
                        == Some(local_submission_id.as_str())
                })
                .ok_or_else(|| {
                    QuantixError::Other(format!(
                        "qmt_live local_submission_id 不存在: {local_submission_id}"
                    ))
                })?;
            let request = runtime_store
                .get_execution_request(&order.client_order_id)
                .await?
                .ok_or_else(|| {
                    QuantixError::Other(format!(
                        "qmt_live local_submission_id={local_submission_id} 对应 request 不存在: {}",
                        order.client_order_id
                    ))
                })?;
            Ok((request, Some(order)))
        }
    }
}

pub(crate) async fn build_execution_bridge_qmt_audit_output(
    runtime_store: &StrategyRuntimeStore,
    lookup: QmtAuditLookup,
) -> Result<serde_json::Value> {
    let (request, order) = resolve_qmt_audit_record(runtime_store, &lookup).await?;
    let order = order.as_ref();
    let request_payload = &request.payload_json;

    let symbol = order
        .map(|order| order.symbol.clone())
        .or_else(|| string_path(request_payload, &["execution_snapshot", "symbol"]));
    let side = order
        .map(|order| order.side.as_str().to_string())
        .or_else(|| {
            string_path(
                request_payload,
                &["execution_snapshot", "order_intent", "side"],
            )
        });
    let quantity = order.map(|order| order.requested_quantity).or_else(|| {
        i64_path(
            request_payload,
            &["execution_snapshot", "order_intent", "requested_quantity"],
        )
    });
    let order_type = order
        .map(|order| order.order_type.as_str().to_string())
        .or_else(|| {
            string_path(
                request_payload,
                &["execution_snapshot", "order_intent", "order_type"],
            )
        });
    let price_intent = order
        .map(|order| order.requested_price.to_string())
        .or_else(|| {
            string_path(
                request_payload,
                &["execution_snapshot", "order_intent", "requested_price"],
            )
        });

    let local_submission_id = order
        .and_then(|order| qmt_order_identity_field(order, "local_submission_id"))
        .map(str::to_string);
    let client_order_id = order
        .map(|order| order.client_order_id.clone())
        .or_else(|| string_path(request_payload, &["execution_result", "client_order_id"]));
    let task_id = order
        .and_then(|order| qmt_order_identity_field(order, "task_id"))
        .map(str::to_string)
        .or_else(|| string_path(request_payload, &["execution_result", "adapter_order_id"]));
    let external_order_id = order
        .and_then(|order| qmt_order_identity_field(order, "external_order_id"))
        .map(str::to_string)
        .or_else(|| string_path(request_payload, &["execution_result", "external_order_id"]));
    let bridge_contract_version = order
        .and_then(|order| qmt_order_metadata_field(order, &["bridge_contract_version"]))
        .or_else(|| {
            string_path(
                request_payload,
                &["execution_result", "bridge_contract_version"],
            )
        });
    let qmt_live_error_category = string_path(
        request_payload,
        &["execution_diagnostics", "qmt_live_failure_category"],
    );
    let reconciliation_decision =
        order.and_then(|order| qmt_order_metadata_field(order, &["reconciliation", "last_action"]));
    let manual_intervention_marker =
        reconciliation_decision.as_deref() == Some("manual_intervention");

    Ok(serde_json::json!({
        "lookup": {
            "type": lookup.lookup_type(),
            "value": lookup.value(),
        },
        "request": {
            "request_id": request.request_id.as_str(),
            "target_mode": request.target_mode.as_str(),
            "redacted_account_label": redact_account_label(&request.target_account),
            "target_account_raw": serde_json::Value::Null,
        },
        "order": {
            "symbol": symbol,
            "side": side,
            "quantity": quantity,
            "order_type": order_type,
            "price_intent": price_intent,
        },
        "qmt_live": {
            "local_submission_id": local_submission_id,
            "client_order_id": client_order_id,
            "task_id": task_id,
            "external_order_id": external_order_id,
            "bridge_contract_version": bridge_contract_version,
            "qmt_live_error_category": qmt_live_error_category,
            "reconciliation_decision": reconciliation_decision,
            "manual_intervention_marker": manual_intervention_marker,
        },
    }))
}

pub(crate) async fn execute_execution_bridge_qmt_audit(lookup: QmtAuditLookup) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let output = build_execution_bridge_qmt_audit_output(&runtime_store, lookup).await?;

    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_query(order_id: &str) -> Result<()> {
    let client = create_bridge_client()?;
    let output = build_execution_bridge_qmt_query_output(&client, order_id).await?;

    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_cancel_with_client(
    client: &BridgeHttpClient,
    order_id: &str,
) -> Result<QmtCancelCommandResult> {
    let submit_service = create_qmt_task_submit_service(client)?;
    let (cancel_order_id, resolved_from_task_result) =
        match submit_service.query_task_result_by_task_id(order_id).await {
            Ok(result) => match result.external_order_id {
                Some(external_order_id) => (external_order_id, true),
                None => (order_id.to_string(), false),
            },
            Err(error) if should_fallback_from_task_result_lookup(&error) => {
                (order_id.to_string(), false)
            }
            Err(error) => return Err(QuantixError::Other(error.to_string())),
        };

    let response = client
        .qmt_cancel_order(&cancel_order_id)
        .await
        .map_err(|err| QuantixError::Other(err.to_string()))?;

    Ok(QmtCancelCommandResult {
        requested_order_id: order_id.to_string(),
        cancel_order_id,
        resolved_from_task_result,
        response,
    })
}

pub(crate) async fn execute_execution_bridge_qmt_cancel(order_id: &str) -> Result<()> {
    println!("⚠️  确认撤销订单: {}", order_id);
    println!("    输入 'YES' 确认撤单，其他任意键取消:");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim() != "YES" {
        println!("已取消撤单");
        return Ok(());
    }

    let client = create_bridge_client()?;
    let result = execute_execution_bridge_qmt_cancel_with_client(&client, order_id).await?;

    if result.response.success {
        if result.resolved_from_task_result && result.cancel_order_id != result.requested_order_id {
            println!(
                "✓ 撤单成功: {} (from task_id {})",
                result.response.order_id, result.requested_order_id
            );
        } else {
            println!("✓ 撤单成功: {}", result.response.order_id);
        }
    } else {
        println!(
            "✗ 撤单失败: {}",
            result
                .response
                .error_message
                .unwrap_or_else(|| "未知错误".to_string())
        );
    }

    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_account() -> Result<()> {
    let client = create_bridge_client()?;
    let response = client
        .qmt_account_status()
        .await
        .map_err(|e| QuantixError::Other(e.to_string()))?;

    println!("账户状态");
    println!("─────────────────────────────────────");
    println!("  适配器:      {}", response.adapter);
    println!("  模式:        {}", response.mode);
    println!("  SDK 可用:   {}", response.sdk_available);
    println!(
        "  连接状态:    {}",
        if response.connected {
            "已连接"
        } else {
            "未连接"
        }
    );
    if let Some(account) = response.account_masked {
        println!("  账户:        {}", account);
    }

    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_positions() -> Result<()> {
    let client = create_bridge_client()?;
    let positions = client
        .qmt_positions()
        .await
        .map_err(|e| QuantixError::Other(e.to_string()))?;

    if positions.is_empty() {
        println!("当前无持仓");
        return Ok(());
    }

    println!("持仓列表");
    println!("─────────────────────────────────────────────────────────────────────────────────");
    println!(
        "{:<12} {:<10} {:<12} {:<12} {:<12}",
        "股票代码", "持仓", "可用", "成本价", "市值"
    );
    println!("{}", "-".repeat(76));

    for pos in positions {
        println!(
            "{:<12} {:<10} {:<12} {:<12} {:<12}",
            pos.symbol,
            pos.volume,
            pos.available,
            pos.cost_price.as_deref().unwrap_or("-"),
            pos.market_value.as_deref().unwrap_or("-")
        );
    }

    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_asset() -> Result<()> {
    let client = create_bridge_client()?;
    let asset = client
        .qmt_asset()
        .await
        .map_err(|e| QuantixError::Other(e.to_string()))?;

    println!("资产信息");
    println!("─────────────────────────────────────");
    println!("  总资产:      {}", asset.total_asset);
    println!("  可用现金:    {}", asset.cash);
    println!("  持仓市值:    {}", asset.market_value);
    println!("  账户 ID:    {}", asset.account_id);

    Ok(())
}

pub(crate) fn print_execution_daemon_summary(summary: &ExecutionDaemonIterationSummary) {
    println!("{}", format_execution_daemon_summary(summary));
}

pub(crate) async fn execute_execution_command(cmd: ExecutionCommands) -> Result<()> {
    match cmd {
        ExecutionCommands::Config(subcommand) => match subcommand {
            ExecutionConfigCommands::Init => {
                execute_execution_config_init().await?;
            }
            ExecutionConfigCommands::Show => {
                execute_execution_config_show().await?;
            }
        },
        ExecutionCommands::Daemon(subcommand) => match subcommand {
            ExecutionDaemonCommands::Run { once } => {
                execute_execution_daemon_run(once).await?;
            }
        },
        ExecutionCommands::Bridge(subcommand) => match subcommand {
            ExecutionBridgeCommands::Status { checklist } => {
                execute_execution_bridge_status(checklist).await?;
            }
            ExecutionBridgeCommands::QmtPreview { request_id } => {
                execute_execution_bridge_qmt_preview(&request_id).await?;
            }
            ExecutionBridgeCommands::QmtLive { request_id, yes } => {
                execute_execution_bridge_qmt_live(&request_id, yes).await?;
            }
            ExecutionBridgeCommands::QmtQuery { order_id } => {
                execute_execution_bridge_qmt_query(&order_id).await?;
            }
            ExecutionBridgeCommands::QmtAudit {
                request_id,
                task_id,
                local_submission_id,
            } => {
                let lookup = QmtAuditLookup::from_cli(request_id, task_id, local_submission_id)?;
                execute_execution_bridge_qmt_audit(lookup).await?;
            }
            ExecutionBridgeCommands::QmtCancel { order_id } => {
                execute_execution_bridge_qmt_cancel(&order_id).await?;
            }
            ExecutionBridgeCommands::QmtAccount => {
                execute_execution_bridge_qmt_account().await?;
            }
            ExecutionBridgeCommands::QmtPositions => {
                execute_execution_bridge_qmt_positions().await?;
            }
            ExecutionBridgeCommands::QmtAsset => {
                execute_execution_bridge_qmt_asset().await?;
            }
        },
        ExecutionCommands::Qmt(subcommand) => match subcommand {
            ExecutionQmtCommands::Status { checklist } => {
                execute_execution_bridge_status(checklist).await?;
            }
            ExecutionQmtCommands::Preview { request_id } => {
                execute_execution_bridge_qmt_preview(&request_id).await?;
            }
            ExecutionQmtCommands::Live { request_id, yes } => {
                execute_execution_bridge_qmt_live(&request_id, yes).await?;
            }
            ExecutionQmtCommands::Query { order_id } => {
                execute_execution_bridge_qmt_query(&order_id).await?;
            }
            ExecutionQmtCommands::Audit {
                request_id,
                task_id,
                local_submission_id,
            } => {
                let lookup = QmtAuditLookup::from_cli(request_id, task_id, local_submission_id)?;
                execute_execution_bridge_qmt_audit(lookup).await?;
            }
            ExecutionQmtCommands::Cancel { order_id } => {
                execute_execution_bridge_qmt_cancel(&order_id).await?;
            }
            ExecutionQmtCommands::Account => {
                execute_execution_bridge_qmt_account().await?;
            }
            ExecutionQmtCommands::Positions => {
                execute_execution_bridge_qmt_positions().await?;
            }
            ExecutionQmtCommands::Asset => {
                execute_execution_bridge_qmt_asset().await?;
            }
        },
    }

    Ok(())
}
