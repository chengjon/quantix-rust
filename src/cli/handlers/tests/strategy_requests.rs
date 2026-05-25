use super::strategy_helpers::{fixed_ts, sample_run};
use super::*;
use rust_decimal_macros::dec;

#[tokio::test]
async fn test_execute_strategy_signal_list_approve_reject_and_request_list() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-1".to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: run.run_id.clone(),
        metadata_json: json!({}),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
    };
    runtime_store.insert_signal(&signal).await.unwrap();

    let pending = execute_strategy_signal_list_with_store(&runtime_store, Some("pending"), None)
        .await
        .unwrap();
    assert_eq!(pending.len(), 1);

    let request =
        execute_strategy_signal_approve_with_store(&runtime_store, "signal-1", "paper", "default")
            .await
            .unwrap();
    assert_eq!(request.signal_id, "signal-1");

    let requests = execute_strategy_request_list_with_store(&runtime_store, Some("pending"))
        .await
        .unwrap();
    assert_eq!(requests.len(), 1);

    let second = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-2".to_string(),
        bar_end: fixed_ts() + chrono::Duration::days(1),
        ..signal
    };
    runtime_store.insert_signal(&second).await.unwrap();
    execute_strategy_signal_reject_with_store(&runtime_store, "signal-2", Some("manual"))
        .await
        .unwrap();

    let rejected = runtime_store.get_signal("signal-2").await.unwrap().unwrap();
    assert_eq!(
        rejected.approval_status,
        crate::execution::models::ApprovalStatus::Rejected
    );
}

#[tokio::test]
async fn test_execute_strategy_request_execute_rejects_live_target_mode_with_qmt_guidance() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_store = JsonRiskStore::new(dir.path().join("risk_state.json"));

    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-request-exec-live".to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: run.run_id.clone(),
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

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-request-exec-live",
            "live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_strategy_request_execute_with_components(
        &runtime_store,
        &request.request_id,
        trade_store,
        risk_store,
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Unsupported(_)));
    let message = err.to_string();
    assert!(message.contains("live 模式尚未实现"));
    assert!(message.contains("qmt_live"));
    assert!(message.contains("qmt.mode=live"));
    assert!(message.contains("execution bridge qmt-live"));

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Failed
    );
    assert!(
        saved.payload_json["execution_error"]["message"]
            .as_str()
            .unwrap()
            .contains("execution bridge qmt-live")
    );
}

#[tokio::test]
async fn test_execute_strategy_request_execute_and_cancel() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_store = JsonRiskStore::new(dir.path().join("risk_state.json"));

    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            crate::trade::InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();

    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-request-exec".to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: run.run_id.clone(),
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

    let request = execute_strategy_signal_approve_with_store(
        &runtime_store,
        "signal-request-exec",
        "mock_live",
        "default",
    )
    .await
    .unwrap();

    let completed = execute_strategy_request_execute_with_components(
        &runtime_store,
        &request.request_id,
        trade_store.clone(),
        risk_store.clone(),
    )
    .await
    .unwrap();
    assert_eq!(
        completed.request_status,
        crate::execution::models::ExecutionRequestStatus::Completed
    );
    assert_eq!(
        completed.payload_json["execution_result"]["order_status"],
        "accepted"
    );

    let second_signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-request-cancel".to_string(),
        bar_end: fixed_ts() + chrono::Duration::days(1),
        ..signal
    };
    runtime_store.insert_signal(&second_signal).await.unwrap();

    let cancel_request = execute_strategy_signal_approve_with_store(
        &runtime_store,
        "signal-request-cancel",
        "paper",
        "default",
    )
    .await
    .unwrap();

    let canceled = execute_strategy_request_cancel_with_store(
        &runtime_store,
        &cancel_request.request_id,
        Some("manual cancel"),
    )
    .await
    .unwrap();
    assert_eq!(
        canceled.request_status,
        crate::execution::models::ExecutionRequestStatus::Canceled
    );
    assert_eq!(
        canceled.payload_json["cancellation"]["reason"],
        "manual cancel"
    );
}

#[test]
fn test_execute_execution_bridge_qmt_live_source_keeps_confirmation_and_request_guidance() {
    let source = std::fs::read_to_string(
        repo_root()
            .join("src")
            .join("cli")
            .join("handlers")
            .join("execution_handler.rs"),
    )
    .expect("expected src/cli/handlers/execution_handler.rs");

    assert!(
        source.contains("输入 'YES' 确认下单"),
        "expected manual qmt_live handler to keep explicit YES confirmation"
    );
    assert!(
        source.contains("查看 request 与后续收敛状态: quantix strategy request show"),
        "expected manual qmt_live handler to guide operators to request/reconciliation visibility"
    );
    assert!(
        source.contains("request 可能会显示为 completed"),
        "expected manual qmt_live handler to explain completed request visibility after submission"
    );
    assert!(
        source.contains("这只表示执行层已完成提交"),
        "expected manual qmt_live handler to explain completed request semantics"
    );
    assert!(
        source.contains("订单初始状态通常仍为 pending_submit"),
        "expected manual qmt_live handler to explain non-terminal initial order status"
    );
    assert!(
        !source.contains("查询订单状态: quantix execution bridge qmt-query --order-id"),
        "expected manual qmt_live handler to drop legacy qmt-query post-submit guidance"
    );
}

#[tokio::test]
async fn test_execute_strategy_signal_approve_rejects_live_target_mode_early() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();

    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-live-target".to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: run.run_id.clone(),
        metadata_json: json!({}),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
    };
    runtime_store.insert_signal(&signal).await.unwrap();

    let err = execute_strategy_signal_approve_with_store(
        &runtime_store,
        "signal-live-target",
        "live",
        "default",
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Unsupported(_)));
    let message = err.to_string();
    assert!(message.contains("target_mode=live"));
    assert!(message.contains("target_mode=qmt_live"));
    assert!(message.contains("execution bridge qmt-live"));
    assert!(message.contains("qmt.mode=live"));

    let saved_signal = runtime_store
        .get_signal("signal-live-target")
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
async fn test_execute_strategy_signal_approve_allows_qmt_live_target_mode() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();

    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-qmt-live-target".to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: run.run_id.clone(),
        metadata_json: json!({}),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
    };
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = execute_strategy_signal_approve_with_store(
        &runtime_store,
        "signal-qmt-live-target",
        "qmt_live",
        "default",
    )
    .await
    .unwrap();

    assert_eq!(request.target_mode, "qmt_live");
    assert_eq!(
        request.request_status,
        crate::execution::models::ExecutionRequestStatus::Pending
    );
}

#[tokio::test]
async fn test_execute_strategy_request_execute_rejects_qmt_live_with_manual_bridge_guidance() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_store = JsonRiskStore::new(dir.path().join("risk_state.json"));

    let run = crate::execution::models::StrategyRunRecord {
        run_id: uuid::Uuid::new_v4().to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "signal".to_string(),
        trigger: "daemon".to_string(),
        status: crate::execution::models::StrategyRunStatus::Running,
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        started_at: fixed_ts(),
        finished_at: None,
        metadata_json: serde_json::json!({}),
    };
    runtime_store.insert_run(&run).await.unwrap();

    let signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-request-qmt-live".to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: run.run_id.clone(),
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

    let request = execute_strategy_signal_approve_with_store(
        &runtime_store,
        "signal-request-qmt-live",
        "qmt_live",
        "default",
    )
    .await
    .unwrap();

    let err = execute_strategy_request_execute_with_components(
        &runtime_store,
        &request.request_id,
        trade_store,
        risk_store,
    )
    .await
    .unwrap_err();

    let message = err.to_string();
    assert!(message.contains("qmt_live"));
    assert!(message.contains("execution bridge qmt-live"));
}

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
