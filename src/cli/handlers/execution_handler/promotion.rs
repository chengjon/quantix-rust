//! QMT promotion checklist + live preflight report builders.

use super::*;
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
    let qmt_risk_notice = crate::execution::mode_semantics::risk_notice_for_execution_channel(
        qmt_live_capabilities.channel,
    );
    let qmt_storage_namespace =
        crate::execution::mode_semantics::storage_namespace_for_execution_channel(
            qmt_live_capabilities.channel,
        );

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
        format!(
            "{} qmt_live risk_notice={}",
            status_mark(qmt_risk_notice.is_some()),
            qmt_risk_notice.unwrap_or("unregistered")
        ),
        format!(
            "{} qmt_live storage_namespace={}",
            status_mark(qmt_storage_namespace.is_some()),
            qmt_storage_namespace.unwrap_or("unregistered")
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

pub(crate) fn qmt_live_preflight_report_json(report: &QmtLivePreflightReport) -> serde_json::Value {
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
    let qmt_risk_notice = crate::execution::mode_semantics::risk_notice_for_execution_channel(
        ExecutionChannel::QmtLive,
    );
    let qmt_storage_namespace =
        crate::execution::mode_semantics::storage_namespace_for_execution_channel(
            ExecutionChannel::QmtLive,
        );

    [
        "QMT live preflight".to_string(),
        format!("readiness={readiness}"),
        format!("failure_category={failure_category}"),
        format!("bridge_reachable={}", report.bridge_reachable),
        format!("bridge_contract_version={}", report.bridge_contract_version),
        format!("capability_source={}", report.capability_source),
        format!("risk_notice={}", qmt_risk_notice.unwrap_or("unregistered")),
        format!(
            "storage_namespace={}",
            qmt_storage_namespace.unwrap_or("unregistered")
        ),
        format!("kill_switch={kill_switch}"),
    ]
    .join("\n")
}
