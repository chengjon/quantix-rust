use super::strategy_helpers::{fixed_ts, sample_run};
use super::*;

async fn seed_signal_for_kill_switch(runtime_store: &StrategyRuntimeStore, signal_id: &str) {
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = crate::execution::models::StrategySignalRecord {
        signal_id: signal_id.to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: run.run_id,
        metadata_json: json!({
            "market_price": "12.34",
            "signal_value": "buy",
            "execution_policy": {
                "fixed_cash_per_buy": "10000",
                "slippage_bps": 0
            },
            "bar_source_id": "test-primary",
            "bar_source_fallback": false
        }),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
    };
    runtime_store.insert_signal(&signal).await.unwrap();
}

fn enable_test_kill_switch(store: &crate::safety::JsonKillSwitchStore) {
    execute_safety_kill_switch_command_with_store_at(
        SafetyKillSwitchCommands::Enable {
            reason: "broker instability".to_string(),
        },
        store,
        fixed_ts(),
    )
    .unwrap();
}

#[tokio::test]
async fn test_execute_strategy_signal_approve_rejects_mock_live_when_kill_switch_enabled() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let kill_switch_store =
        crate::safety::JsonKillSwitchStore::new(dir.path().join("kill_switch.json"));
    enable_test_kill_switch(&kill_switch_store);
    seed_signal_for_kill_switch(&runtime_store, "signal-mock-live-kill-switch").await;

    let err = execute_strategy_signal_approve_with_store_and_kill_switch(
        &runtime_store,
        &kill_switch_store,
        "signal-mock-live-kill-switch",
        "mock_live",
        "default",
    )
    .await
    .unwrap_err();

    let message = err.to_string();
    assert!(message.contains("kill switch"));
    assert!(message.contains("mock_live"));
    assert!(message.contains("broker instability"));

    let saved_signal = runtime_store
        .get_signal("signal-mock-live-kill-switch")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved_signal.approval_status,
        crate::execution::models::ApprovalStatus::Pending
    );
    let requests = execute_strategy_request_list_with_store(&runtime_store, None)
        .await
        .unwrap();
    assert!(requests.is_empty());
}

#[tokio::test]
async fn test_execute_strategy_signal_approve_rejects_qmt_live_when_kill_switch_enabled() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let kill_switch_store =
        crate::safety::JsonKillSwitchStore::new(dir.path().join("kill_switch.json"));
    enable_test_kill_switch(&kill_switch_store);
    seed_signal_for_kill_switch(&runtime_store, "signal-qmt-live-kill-switch").await;

    let err = execute_strategy_signal_approve_with_store_and_kill_switch(
        &runtime_store,
        &kill_switch_store,
        "signal-qmt-live-kill-switch",
        "qmt_live",
        "default",
    )
    .await
    .unwrap_err();

    let message = err.to_string();
    assert!(message.contains("kill switch"));
    assert!(message.contains("qmt_live"));
    assert!(message.contains("broker instability"));

    let saved_signal = runtime_store
        .get_signal("signal-qmt-live-kill-switch")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved_signal.approval_status,
        crate::execution::models::ApprovalStatus::Pending
    );
    let requests = execute_strategy_request_list_with_store(&runtime_store, None)
        .await
        .unwrap();
    assert!(requests.is_empty());
}

#[tokio::test]
async fn test_execute_strategy_signal_approve_allows_paper_when_kill_switch_enabled() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let kill_switch_store =
        crate::safety::JsonKillSwitchStore::new(dir.path().join("kill_switch.json"));
    enable_test_kill_switch(&kill_switch_store);
    seed_signal_for_kill_switch(&runtime_store, "signal-paper-kill-switch").await;

    let request = execute_strategy_signal_approve_with_store_and_kill_switch(
        &runtime_store,
        &kill_switch_store,
        "signal-paper-kill-switch",
        "paper",
        "default",
    )
    .await
    .unwrap();

    assert_eq!(request.target_mode, "paper");
    assert_eq!(
        request.request_status,
        crate::execution::models::ExecutionRequestStatus::Pending
    );
}
