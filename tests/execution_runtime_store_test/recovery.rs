use super::helpers::*;

#[tokio::test]
async fn list_recoverable_mock_live_orders_returns_only_non_terminal_rows_with_state() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let mut recoverable = sample_order(&run.run_id, "run_000001_recoverable");
    recoverable.status = OrderStatus::Accepted;
    store.insert_order(&recoverable).await.unwrap();
    store
        .insert_mock_live_order_state(
            &recoverable.order_id,
            Some("adapter-r"),
            &MockLiveOrderState::default(),
        )
        .await
        .unwrap();

    let mut terminal = sample_order(&run.run_id, "run_000001_terminal");
    terminal.status = OrderStatus::Filled;
    terminal.filled_quantity = 100;
    terminal.remaining_quantity = 0;
    store.insert_order(&terminal).await.unwrap();
    store
        .insert_mock_live_order_state(
            &terminal.order_id,
            Some("adapter-t"),
            &MockLiveOrderState::default(),
        )
        .await
        .unwrap();

    let rows = store.list_recoverable_mock_live_orders().await.unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].client_order_id, "run_000001_recoverable");
}

#[test]
fn phase29b_signal_and_request_enums_use_stable_string_values() {
    assert_eq!(OrderStatus::PendingCancel.as_str(), "pending_cancel");
    assert_eq!(
        OrderStatus::from_str("pending_cancel"),
        Some(OrderStatus::PendingCancel)
    );
    assert_eq!(SignalStatus::New.as_str(), "new");
    assert_eq!(
        SignalStatus::from_str("superseded"),
        Some(SignalStatus::Superseded)
    );
    assert_eq!(ApprovalStatus::Approved.as_str(), "approved");
    assert_eq!(
        ApprovalStatus::from_str("rejected"),
        Some(ApprovalStatus::Rejected)
    );
    assert_eq!(ExecutionRequestStatus::Pending.as_str(), "pending");
    assert_eq!(ExecutionRequestStatus::InProgress.as_str(), "in_progress");
    assert_eq!(
        ExecutionRequestStatus::from_str("canceled"),
        Some(ExecutionRequestStatus::Canceled)
    );
    assert_eq!(
        ExecutionRequestStatus::from_str("in_progress"),
        Some(ExecutionRequestStatus::InProgress)
    );
}

#[test]
fn phase29c_mock_live_state_round_trips_through_serde_defaults() {
    let state = MockLiveOrderState::default();
    let json = serde_json::to_string(&state).unwrap();
    let parsed: MockLiveOrderState = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.unknown_retries, 0);
    assert!(!parsed.recovery_exhausted);
    assert!(!parsed.cancel_requested);
}

#[test]
fn phase29c_recovery_summary_exposes_extended_counters() {
    let summary = RecoverySummary {
        scanned: 4,
        recovered: 1,
        unchanged: 2,
        failed: 0,
        skipped: 1,
    };

    assert_eq!(summary.scanned, 4);
    assert_eq!(summary.recovered, 1);
    assert_eq!(summary.unchanged, 2);
    assert_eq!(summary.failed, 0);
    assert_eq!(summary.skipped, 1);
}
