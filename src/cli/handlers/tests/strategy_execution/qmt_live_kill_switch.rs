use super::super::strategy_helpers::{fixed_ts, sample_run, sample_signal};
use super::*;

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
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["diagnostic_source"],
        "qmt_live_gate"
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["qmt_live_failure_category"],
        "missing_required_capability"
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["compatibility_requirement"],
        "bridge qmt.supports includes order_submit"
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

#[tokio::test]
async fn test_execute_execution_bridge_qmt_live_rejects_when_kill_switch_enabled() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let kill_switch_store =
        crate::safety::JsonKillSwitchStore::new(dir.path().join("kill_switch.json"));

    execute_safety_kill_switch_command_with_store_at(
        SafetyKillSwitchCommands::Enable {
            reason: "broker instability".to_string(),
        },
        &kill_switch_store,
        fixed_ts(),
    )
    .unwrap();

    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(&run.run_id, "signal-qmt-live-kill-switch", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-live-kill-switch",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch(
        &runtime_store,
        &kill_switch_store,
        &request.request_id,
        true,
    )
    .await
    .unwrap_err();

    let message = err.to_string();
    assert!(message.contains("kill switch"));
    assert!(message.contains("qmt_live"));
    assert!(message.contains("broker instability"));

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
        "kill_switch_blocked"
    );
    assert_eq!(saved.payload_json["execution_error"]["adapter"], "qmt_live");
    assert_eq!(
        saved.payload_json["kill_switch"]["reason"],
        "broker instability"
    );
    assert_eq!(saved.payload_json["kill_switch"]["target_mode"], "qmt_live");
    assert_eq!(
        saved.payload_json["kill_switch"]["enabled_at"],
        fixed_ts().to_rfc3339()
    );
    assert!(
        saved.payload_json["kill_switch"]["blocked_at"]
            .as_str()
            .is_some()
    );
}
