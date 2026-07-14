use super::*;
use quantix_cli::execution::models::ExecutionRequestStatus;

#[tokio::test]
async fn execute_request_by_id_rejects_mock_live_when_kill_switch_enabled() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_store = JsonRiskStore::new(dir.path().join("risk_state.json"));
    let kill_switch_store = JsonKillSwitchStore::new(dir.path().join("kill_switch.json"));

    kill_switch_store
        .save(&KillSwitchState {
            enabled: true,
            reason: Some("broker instability".to_string()),
            enabled_at: Some(fixed_ts()),
            disabled_at: None,
            updated_by: "cli".to_string(),
        })
        .unwrap();

    let run = sample_run(fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();
    let signal = sample_signal(
        &run.run_id,
        "signal-daemon-kill-switch-mock-live",
        fixed_ts(),
    );
    runtime_store.insert_signal(&signal).await.unwrap();
    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-daemon-kill-switch-mock-live",
            "mock_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_request_by_id_with_components_and_kill_switch(
        &runtime_store,
        &kill_switch_store,
        &request.request_id,
        trade_store,
        risk_store,
    )
    .await
    .unwrap_err();

    let message = err.to_string();
    assert!(message.contains("kill switch"));
    assert!(message.contains("mock_live"));
    assert!(message.contains("broker instability"));

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.request_status, ExecutionRequestStatus::Failed);
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "kill_switch_blocked"
    );
    assert_eq!(
        saved.payload_json["kill_switch"]["reason"],
        "broker instability"
    );
    assert_eq!(
        saved.payload_json["kill_switch"]["target_mode"],
        "mock_live"
    );
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

#[tokio::test]
async fn execute_request_by_id_rejects_qmt_live_when_kill_switch_enabled() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_store = JsonRiskStore::new(dir.path().join("risk_state.json"));
    let kill_switch_store = JsonKillSwitchStore::new(dir.path().join("kill_switch.json"));

    kill_switch_store
        .save(&KillSwitchState {
            enabled: true,
            reason: Some("broker instability".to_string()),
            enabled_at: Some(fixed_ts()),
            disabled_at: None,
            updated_by: "cli".to_string(),
        })
        .unwrap();

    let run = sample_run(fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();
    let signal = sample_signal(
        &run.run_id,
        "signal-daemon-kill-switch-qmt-live",
        fixed_ts(),
    );
    runtime_store.insert_signal(&signal).await.unwrap();
    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-daemon-kill-switch-qmt-live",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_request_by_id_with_components_and_kill_switch(
        &runtime_store,
        &kill_switch_store,
        &request.request_id,
        trade_store,
        risk_store,
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
    assert_eq!(saved.request_status, ExecutionRequestStatus::Failed);
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "kill_switch_blocked"
    );
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
