use super::strategy_helpers::{fixed_ts, sample_run};
use super::*;

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

    let pending = execute_strategy_signal_list_with_store(
        &runtime_store,
        StrategySignalListFilters {
            approval_status: Some("pending"),
            ..Default::default()
        },
    )
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
async fn test_execute_strategy_signal_list_applies_declared_filters_and_limit() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let matching_signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-match".to_string(),
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
    let same_filter_later_signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-match-later".to_string(),
        bar_end: fixed_ts() - chrono::Duration::days(1),
        ..matching_signal.clone()
    };
    let wrong_code_signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-wrong-code".to_string(),
        symbol: "000002".to_string(),
        bar_end: fixed_ts() + chrono::Duration::days(2),
        ..matching_signal.clone()
    };
    let wrong_strategy_signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-wrong-strategy".to_string(),
        strategy_instance_id: "breakout_000001".to_string(),
        strategy_name: "breakout".to_string(),
        bar_end: fixed_ts() + chrono::Duration::days(3),
        ..matching_signal.clone()
    };

    for signal in [
        &wrong_code_signal,
        &wrong_strategy_signal,
        &same_filter_later_signal,
        &matching_signal,
    ] {
        runtime_store.insert_signal(signal).await.unwrap();
    }

    let rows = execute_strategy_signal_list_with_store(
        &runtime_store,
        StrategySignalListFilters {
            strategy_instance: Some("ma_fast_5_slow_20"),
            strategy: Some("ma_cross"),
            code: Some("000001"),
            approval_status: Some("pending"),
            signal_status: Some("new"),
            limit: Some(1),
        },
    )
    .await
    .unwrap();

    let signal_ids = rows
        .iter()
        .map(|row| row.signal_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(signal_ids.len(), 1);
    assert!(matches!(
        signal_ids.as_slice(),
        ["signal-match"] | ["signal-match-later"]
    ));

    let all_matching_rows = execute_strategy_signal_list_with_store(
        &runtime_store,
        StrategySignalListFilters {
            strategy_instance: Some("ma_fast_5_slow_20"),
            strategy: Some("ma_cross"),
            code: Some("000001"),
            approval_status: Some("pending"),
            signal_status: Some("new"),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let all_matching_ids = all_matching_rows
        .iter()
        .map(|row| row.signal_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(all_matching_ids.len(), 2);
    assert!(all_matching_ids.contains(&"signal-match"));
    assert!(all_matching_ids.contains(&"signal-match-later"));
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
