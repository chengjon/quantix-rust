use super::strategy_helpers::{fixed_ts, sample_run};
use super::*;
use crate::bridge::models::{
    BridgeCapabilitiesResponse, BridgeCapabilitySection, BridgeQmtCapabilitySection,
};
use crate::execution::adapter::{
    ExecutionCancelSemantics, ExecutionCapabilities, ExecutionChannel, ExecutionFillSource,
    ExecutionStatusSource,
};
use crate::execution::mode_semantics::{QMT_LIVE_RISK_NOTICE, QMT_LIVE_STORAGE_NAMESPACE};
use crate::safety::KillSwitchState;

#[test]
fn test_format_qmt_promotion_checklist_surfaces_live_readiness_and_next_steps() {
    let checklist = format_qmt_promotion_checklist(
        &BridgeCapabilitiesResponse {
            tdx: BridgeCapabilitySection {
                enabled: true,
                supports: vec!["quote".to_string()],
            },
            qmt: BridgeQmtCapabilitySection {
                enabled: false,
                mode: "preview_only".to_string(),
                supports: vec!["order_preview".to_string()],
            },
        },
        ExecutionCapabilities {
            channel: ExecutionChannel::QmtLive,
            status_source: ExecutionStatusSource::Broker,
            fill_source: ExecutionFillSource::Broker,
            relies_on_broker_api: true,
            supports_pending_order_lifecycle: true,
            supports_partial_fill: true,
            cancel_semantics: ExecutionCancelSemantics::Broker,
        },
    );

    assert!(checklist.contains("QMT promotion checklist"));
    assert!(checklist.contains("[x] bridge qmt.enabled=true"));
    assert!(checklist.contains("[x] bridge qmt.mode=live"));
    assert!(checklist.contains("[x] bridge qmt.supports 包含 order_submit"));
    assert!(checklist.contains("[ok] qmt_live adapter channel=qmt_live"));
    assert!(checklist.contains("[ok] qmt_live status_source=broker"));
    assert!(checklist.contains("[ok] qmt_live fill_source=broker"));
    assert!(checklist.contains("[ok] qmt_live cancel_semantics=broker"));
    assert!(checklist.contains(&format!("[ok] qmt_live risk_notice={QMT_LIVE_RISK_NOTICE}")));
    assert!(checklist.contains(&format!(
        "[ok] qmt_live storage_namespace={QMT_LIVE_STORAGE_NAMESPACE}"
    )));
    assert!(checklist.contains("[ ] request target_mode=qmt_live"));
    assert!(checklist.contains("[ ] 先在 paper 路径验证策略与风控"));
    assert!(checklist.contains("[ ] 再在 mock_live 路径验证非终态与收敛"));
    assert!(checklist.contains("quantix execution qmt preview --request-id <ID>"));
    assert!(checklist.contains("quantix execution qmt live --request-id <ID> [--yes]"));
    assert!(checklist.contains("quantix strategy request show <ID> --verbose"));
}

fn qmt_live_broker_capabilities() -> ExecutionCapabilities {
    ExecutionCapabilities {
        channel: ExecutionChannel::QmtLive,
        status_source: ExecutionStatusSource::Broker,
        fill_source: ExecutionFillSource::Broker,
        relies_on_broker_api: true,
        supports_pending_order_lifecycle: true,
        supports_partial_fill: true,
        cancel_semantics: ExecutionCancelSemantics::Broker,
    }
}

fn bridge_capabilities(
    qmt_enabled: bool,
    qmt_mode: &str,
    qmt_supports: &[&str],
) -> BridgeCapabilitiesResponse {
    BridgeCapabilitiesResponse {
        tdx: BridgeCapabilitySection {
            enabled: true,
            supports: vec!["quote".to_string()],
        },
        qmt: BridgeQmtCapabilitySection {
            enabled: qmt_enabled,
            mode: qmt_mode.to_string(),
            supports: qmt_supports.iter().map(|item| item.to_string()).collect(),
        },
    }
}

#[test]
fn test_qmt_live_preflight_report_marks_ready_and_surfaces_kill_switch_state() {
    let capabilities = bridge_capabilities(true, "live", &["order_submit"]);
    let kill_switch = KillSwitchState::default();
    let report = build_qmt_live_preflight_report(
        Some(&capabilities),
        None,
        qmt_live_broker_capabilities(),
        Some(&kill_switch),
    );

    assert!(report.ready);
    assert_eq!(report.failure_category, None);
    assert_eq!(report.bridge_contract_version, "unknown");
    assert!(!report.kill_switch_enabled);

    let formatted = format_qmt_live_preflight_report(&report);
    assert!(formatted.contains("QMT live preflight"));
    assert!(formatted.contains("readiness=ready"));
    assert!(formatted.contains("failure_category=none"));
    assert!(formatted.contains(&format!("risk_notice={QMT_LIVE_RISK_NOTICE}")));
    assert!(formatted.contains(&format!("storage_namespace={QMT_LIVE_STORAGE_NAMESPACE}")));
    assert!(formatted.contains("kill_switch=disabled"));
}

#[test]
fn test_qmt_live_preflight_report_classifies_fail_closed_categories() {
    let live_ready = bridge_capabilities(true, "live", &["order_submit"]);
    let disabled = bridge_capabilities(false, "live", &["order_submit"]);
    let non_live = bridge_capabilities(true, "preview_only", &["order_submit"]);
    let missing_submit = bridge_capabilities(true, "live", &["order_preview"]);
    let local_mismatch = ExecutionCapabilities {
        channel: ExecutionChannel::PaperImmediate,
        status_source: ExecutionStatusSource::LocalImmediateAccounting,
        fill_source: ExecutionFillSource::LocalImmediateAccounting,
        relies_on_broker_api: false,
        supports_pending_order_lifecycle: false,
        supports_partial_fill: false,
        cancel_semantics: ExecutionCancelSemantics::AlreadyFilledOnly,
    };
    let enabled_kill_switch = KillSwitchState {
        enabled: true,
        reason: Some("operator stop".to_string()),
        ..KillSwitchState::default()
    };

    let cases = vec![
        (
            build_qmt_live_preflight_report(
                None,
                Some("connection refused"),
                qmt_live_broker_capabilities(),
                None,
            ),
            QmtLivePreflightFailureCategory::BridgeUnreachable,
        ),
        (
            build_qmt_live_preflight_report(None, None, qmt_live_broker_capabilities(), None),
            QmtLivePreflightFailureCategory::QmtCapabilityMissing,
        ),
        (
            build_qmt_live_preflight_report(
                Some(&disabled),
                None,
                qmt_live_broker_capabilities(),
                None,
            ),
            QmtLivePreflightFailureCategory::QmtDisabled,
        ),
        (
            build_qmt_live_preflight_report(
                Some(&non_live),
                None,
                qmt_live_broker_capabilities(),
                None,
            ),
            QmtLivePreflightFailureCategory::QmtModeNotLive,
        ),
        (
            build_qmt_live_preflight_report(
                Some(&missing_submit),
                None,
                qmt_live_broker_capabilities(),
                None,
            ),
            QmtLivePreflightFailureCategory::QmtOrderSubmitMissing,
        ),
        (
            build_qmt_live_preflight_report(Some(&live_ready), None, local_mismatch, None),
            QmtLivePreflightFailureCategory::QmtLiveCapabilityMismatch,
        ),
        (
            build_qmt_live_preflight_report(
                Some(&live_ready),
                None,
                qmt_live_broker_capabilities(),
                Some(&enabled_kill_switch),
            ),
            QmtLivePreflightFailureCategory::KillSwitchEnabled,
        ),
    ];

    for (report, category) in cases {
        assert!(
            !report.ready,
            "expected {category:?} report to be not ready"
        );
        assert_eq!(report.failure_category, Some(category));
        assert!(format_qmt_live_preflight_report(&report).contains(category.as_str()));
    }
}
