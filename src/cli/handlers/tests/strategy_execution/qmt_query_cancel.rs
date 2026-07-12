use super::super::strategy_helpers::{fixed_ts, sample_run, sample_signal};
use super::*;

#[tokio::test]
async fn test_execute_execution_bridge_qmt_preview_remains_available_when_kill_switch_enabled() {
    let _lock = env_lock();
    let _guard = RuntimeEnvGuard::capture();
    let dir = tempdir().unwrap();
    let runtime_db_path = dir.path().join("runtime.db");
    let kill_switch_store =
        crate::safety::JsonKillSwitchStore::new(dir.path().join("kill_switch.json"));

    unsafe {
        std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, &runtime_db_path);
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    execute_safety_kill_switch_command_with_store_at(
        SafetyKillSwitchCommands::Enable {
            reason: "pre-canary drill".to_string(),
        },
        &kill_switch_store,
        fixed_ts(),
    )
    .unwrap();

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
    }

    Mock::given(method("POST"))
        .and(path("/api/v1/broker/qmt/orders/preview"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "adapter_order_id": "preview-1",
            "latest_status": "submitted",
            "filled_quantity": 0,
            "avg_fill_price": null,
            "fill_details": null,
            "rejection_reason": null,
            "broker_payload": {
                "dry_run": true
            }
        })))
        .mount(&server)
        .await;

    let runtime_store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(&run.run_id, "signal-qmt-preview-kill-switch", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-preview-kill-switch",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    execute_execution_bridge_qmt_preview(&request.request_id)
        .await
        .unwrap();

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Pending
    );
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_query_remains_available_when_kill_switch_enabled() {
    let _lock = env_lock();
    let _guard = RuntimeEnvGuard::capture();
    let dir = tempdir().unwrap();
    let runtime_db_path = dir.path().join("runtime.db");
    let kill_switch_store =
        crate::safety::JsonKillSwitchStore::new(dir.path().join("kill_switch.json"));

    unsafe {
        std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, &runtime_db_path);
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    execute_safety_kill_switch_command_with_store_at(
        SafetyKillSwitchCommands::Enable {
            reason: "incident response drill".to_string(),
        },
        &kill_switch_store,
        fixed_ts(),
    )
    .unwrap();

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
    }

    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-readonly-1"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-readonly-1",
            "status": "completed",
            "bridge_contract_version": "miniqmt.v1",
            "result": {
                "client_order_id": "req-readonly-1",
                "local_submission_id": "local-readonly-1",
                "account_scope": "default",
                "event_id": "evt-readonly-1",
                "occurred_at": "2026-05-03T09:32:00Z",
                "source_name": "miniqmt",
                "broker_event_type": "acknowledgement",
                "external_order_id": "broker-readonly-1",
                "reason_code": null,
                "reason_detail": null,
                "evidence_ref": null
            }
        })))
        .mount(&server)
        .await;

    execute_execution_bridge_qmt_query("task-readonly-1")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_build_execution_bridge_qmt_query_output_prefers_task_result_lookup() {
    let server = MockServer::start().await;
    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "completed",
            "bridge_contract_version": "miniqmt.v1",
            "result": {
                "client_order_id": "req-1",
                "local_submission_id": "local-1",
                "account_scope": "default",
                "event_id": "evt-1",
                "occurred_at": "2026-05-03T09:32:00Z",
                "source_name": "miniqmt",
                "broker_event_type": "acknowledgement",
                "external_order_id": "broker-1",
                "reason_code": null,
                "reason_detail": null,
                "evidence_ref": null
            }
        })))
        .mount(&server)
        .await;

    let output = build_execution_bridge_qmt_query_output(&client, "task-1")
        .await
        .unwrap();

    assert_eq!(output["query_mode"], "task_result");
    assert_eq!(output["adapter_order_id"], "task-1");
    assert_eq!(output["latest_status"], "accepted");
    assert_eq!(output["external_order_id"], "broker-1");
    assert_eq!(output["client_order_id"], "req-1");
    assert_eq!(output["local_submission_id"], "local-1");
    assert_eq!(output["source_name"], "miniqmt");
    assert_eq!(output["broker_event_type"], "Acknowledgement");
}

#[tokio::test]
async fn test_build_execution_bridge_qmt_query_output_falls_back_to_legacy_order_lookup() {
    let server = MockServer::start().await;
    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/broker-legacy-1"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({ "reason_detail": "task not found" })),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/broker/qmt/orders/broker-legacy-1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "adapter_order_id": "broker-legacy-1",
            "latest_status": "accepted",
            "filled_quantity": 100,
            "avg_fill_price": "10.50",
            "fill_details": null
        })))
        .mount(&server)
        .await;

    let output = build_execution_bridge_qmt_query_output(&client, "broker-legacy-1")
        .await
        .unwrap();

    assert_eq!(output["query_mode"], "legacy_order");
    assert_eq!(output["adapter_order_id"], "broker-legacy-1");
    assert_eq!(output["latest_status"], "accepted");
    assert_eq!(output["filled_quantity"], 100);
    assert_eq!(output["avg_fill_price"], "10.50");
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_cancel_with_client_resolves_task_id_to_external_order_id()
 {
    let server = MockServer::start().await;
    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-cancel-1"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-cancel-1",
            "status": "completed",
            "bridge_contract_version": "miniqmt.v1",
            "result": {
                "client_order_id": "req-cancel-1",
                "local_submission_id": "local-cancel-1",
                "account_scope": "default",
                "event_id": "evt-cancel-1",
                "occurred_at": "2026-05-03T09:40:00Z",
                "source_name": "miniqmt",
                "broker_event_type": "acknowledgement",
                "external_order_id": "broker-cancel-1",
                "reason_code": null,
                "reason_detail": null,
                "evidence_ref": null
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/broker/qmt/orders/broker-cancel-1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "order_id": "broker-cancel-1",
            "error_message": null
        })))
        .mount(&server)
        .await;

    let result = execute_execution_bridge_qmt_cancel_with_client(&client, "task-cancel-1")
        .await
        .unwrap();

    assert_eq!(result.requested_order_id, "task-cancel-1");
    assert_eq!(result.cancel_order_id, "broker-cancel-1");
    assert!(result.resolved_from_task_result);
    assert!(result.response.success);
    assert_eq!(result.response.order_id, "broker-cancel-1");
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_cancel_with_client_falls_back_to_direct_order_id() {
    let server = MockServer::start().await;
    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/broker-cancel-legacy-1"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({ "reason_detail": "task not found" })),
        )
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/broker/qmt/orders/broker-cancel-legacy-1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "order_id": "broker-cancel-legacy-1",
            "error_message": null
        })))
        .mount(&server)
        .await;

    let result = execute_execution_bridge_qmt_cancel_with_client(&client, "broker-cancel-legacy-1")
        .await
        .unwrap();

    assert_eq!(result.requested_order_id, "broker-cancel-legacy-1");
    assert_eq!(result.cancel_order_id, "broker-cancel-legacy-1");
    assert!(!result.resolved_from_task_result);
    assert!(result.response.success);
    assert_eq!(result.response.order_id, "broker-cancel-legacy-1");
}
