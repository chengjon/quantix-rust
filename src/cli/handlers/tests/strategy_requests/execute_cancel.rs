use super::strategy_helpers::{fixed_ts, sample_run};
use super::*;

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
