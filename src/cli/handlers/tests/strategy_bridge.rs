use super::strategy_helpers::{fixed_ts, sample_run, sample_signal};
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

#[allow(dead_code)]
async fn test_execute_execution_bridge_qmt_live_rejects_preview_only_bridge_mode() {
    let _lock = env_lock();
    let _guard = RuntimeEnvGuard::capture();
    let dir = tempdir().unwrap();
    let runtime_db_path = dir.path().join("runtime.db");

    unsafe {
        std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, &runtime_db_path);
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
    }

    Mock::given(method("GET"))
        .and(path("/api/v1/capabilities"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tdx": {
                "enabled": true,
                "supports": ["quote", "batch_quotes", "kline"]
            },
            "qmt": {
                "enabled": true,
                "mode": "preview_only",
                "supports": ["account_status", "order_preview"]
            }
        })))
        .mount(&server)
        .await;

    let runtime_store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(&run.run_id, "signal-qmt-live-gate", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-live-gate",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_execution_bridge_qmt_live(&request.request_id, true)
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("preview_only"),
        "expected preview_only safety gate error, got: {err}"
    );

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Failed
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "bridge_qmt_mode_not_live"
    );
    assert!(
        saved.payload_json["execution_error"]["message"]
            .as_str()
            .unwrap()
            .contains("preview_only")
    );
}

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

#[tokio::test]
async fn test_execute_execution_bridge_qmt_preview_rejects_legacy_live_request_boundary() {
    let _lock = env_lock();
    let _guard = RuntimeEnvGuard::capture();
    let dir = tempdir().unwrap();
    let runtime_db_path = dir.path().join("runtime.db");

    unsafe {
        std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, &runtime_db_path);
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
    }

    let runtime_store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(&run.run_id, "signal-qmt-preview-live-boundary", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-preview-live-boundary",
            "live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_execution_bridge_qmt_preview(&request.request_id)
        .await
        .unwrap_err();

    assert!(matches!(err, QuantixError::Unsupported(_)));
    let message = err.to_string();
    assert!(message.contains("qmt-preview"));
    assert!(message.contains("target_mode=live"));
    assert!(message.contains("target_mode=qmt_live"));
    assert!(message.contains("qmt_live request"));

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Pending
    );
    assert!(saved.payload_json.get("execution_result").is_none());
    assert!(saved.payload_json.get("execution_error").is_none());
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_live_rejects_when_qmt_capability_is_disabled() {
    let _lock = env_lock();
    let _guard = RuntimeEnvGuard::capture();
    let dir = tempdir().unwrap();
    let runtime_db_path = dir.path().join("runtime.db");

    unsafe {
        std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, &runtime_db_path);
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
    }

    Mock::given(method("GET"))
        .and(path("/api/v1/capabilities"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tdx": {
                "enabled": true,
                "supports": ["quote", "batch_quotes", "kline"]
            },
            "qmt": {
                "enabled": false,
                "mode": "preview_only",
                "supports": ["account_status", "order_preview"]
            }
        })))
        .mount(&server)
        .await;

    let runtime_store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(
        &run.run_id,
        "signal-qmt-live-capability-disabled",
        fixed_ts(),
    );
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-live-capability-disabled",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_execution_bridge_qmt_live(&request.request_id, true)
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("capability 未启用"),
        "expected disabled capability error, got: {err}"
    );

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Failed
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "bridge_qmt_capability_disabled"
    );
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_live_rejects_when_capability_check_fails() {
    let _lock = env_lock();
    let _guard = RuntimeEnvGuard::capture();
    let dir = tempdir().unwrap();
    let runtime_db_path = dir.path().join("runtime.db");

    unsafe {
        std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, &runtime_db_path);
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
    }

    Mock::given(method("GET"))
        .and(path("/api/v1/capabilities"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(503).set_body_string("bridge unavailable"))
        .mount(&server)
        .await;

    let runtime_store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(
        &run.run_id,
        "signal-qmt-live-capability-check-failed",
        fixed_ts(),
    );
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-live-capability-check-failed",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_execution_bridge_qmt_live(&request.request_id, true)
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("能力检查失败"),
        "expected capability check failure error, got: {err}"
    );

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Failed
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "bridge_qmt_capability_check_failed"
    );
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_live_persists_task_identity_into_related_order() {
    let _lock = env_lock();
    let _guard = RuntimeEnvGuard::capture();
    let dir = tempdir().unwrap();
    let runtime_db_path = dir.path().join("runtime.db");

    unsafe {
        std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, &runtime_db_path);
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
    }

    Mock::given(method("GET"))
        .and(path("/api/v1/capabilities"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tdx": {
                "enabled": true,
                "supports": ["quote", "batch_quotes", "kline"]
            },
            "qmt": {
                "enabled": true,
                "mode": "live",
                "supports": ["account_status", "order_preview", "order_submit"]
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v1/task/execute"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "bridge_task_accepted",
            "receipt_timestamp": "2026-05-01T09:30:00Z",
            "bridge_contract_version": "miniqmt.v1",
            "source_name": "miniqmt"
        })))
        .mount(&server)
        .await;

    let runtime_store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(&run.run_id, "signal-qmt-live-success", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-live-success",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    execute_execution_bridge_qmt_live(&request.request_id, true)
        .await
        .unwrap();

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Completed
    );
    assert_eq!(
        saved.payload_json["execution_result"]["order_status"],
        "pending_submit"
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "request_completed_order_non_terminal"
    );
    assert_eq!(
        saved.payload_json["execution_result"]["adapter_order_id"],
        "task-1"
    );
    let saved_order = runtime_store
        .find_order_by_client_order_id(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_order.adapter, "qmt_live");
    assert_eq!(
        saved_order.status,
        crate::execution::models::OrderStatus::PendingSubmit
    );
    assert_eq!(
        saved_order.payload_json["qmt_live"]["task_identity"]["task_id"],
        "task-1"
    );
    assert_eq!(
        saved_order.payload_json["qmt_live"]["task_identity"]["client_order_id"],
        request.request_id
    );
    assert!(
        saved_order.payload_json["qmt_live"]["task_identity"]["local_submission_id"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_live_rejects_live_mode_without_order_submit_support() {
    let _lock = env_lock();
    let _guard = RuntimeEnvGuard::capture();
    let dir = tempdir().unwrap();
    let runtime_db_path = dir.path().join("runtime.db");

    unsafe {
        std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, &runtime_db_path);
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
    }

    Mock::given(method("GET"))
        .and(path("/api/v1/capabilities"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tdx": {
                "enabled": true,
                "supports": ["quote", "batch_quotes", "kline"]
            },
            "qmt": {
                "enabled": true,
                "mode": "live",
                "supports": ["account_status", "order_preview"]
            }
        })))
        .mount(&server)
        .await;

    let runtime_store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(&run.run_id, "signal-qmt-live-submit-capability", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-live-submit-capability",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_execution_bridge_qmt_live(&request.request_id, true)
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("order_submit"),
        "expected order_submit safety gate error, got: {err}"
    );

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Failed
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "bridge_qmt_order_submit_capability_missing"
    );
    assert_eq!(saved.payload_json["execution_error"]["adapter"], "qmt_live");
    assert!(
        saved.payload_json["execution_error"]["message"]
            .as_str()
            .unwrap()
            .contains("order_submit")
    );
    assert!(
        saved.payload_json["execution_error"]["failed_at"]
            .as_str()
            .is_some()
    );
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
