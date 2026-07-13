use super::super::strategy_helpers::{fixed_ts, sample_run, sample_signal};
use super::*;

#[tokio::test]
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
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["diagnostic_source"],
        "qmt_live_gate"
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["qmt_live_failure_category"],
        "capability_check_failed"
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["compatibility_requirement"],
        "bridge /api/v1/capabilities returns qmt capability metadata"
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
