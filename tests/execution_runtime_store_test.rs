use chrono::{TimeZone, Utc};
use quantix_cli::execution::kernel::RecoverySummary;
use quantix_cli::execution::models::{
    ApprovalStatus, ExecutionRequestStatus, MockLiveOrderState, OrderEventRecord, OrderRecord,
    OrderSide, OrderStatus, OrderType, RunnerCheckpointRecord, SignalStatus,
    StrategyDaemonCheckpointRecord, StrategyRunRecord, StrategyRunStatus, StrategySignalRecord,
};
use quantix_cli::execution::runtime_store::StrategyRuntimeStore;
use rust_decimal_macros::dec;
use serde_json::json;
use tempfile::tempdir;
use uuid::Uuid;

fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 17, 9, 30, 0).unwrap()
}

fn sample_run(symbol: &str, bar_end: chrono::DateTime<Utc>) -> StrategyRunRecord {
    StrategyRunRecord {
        run_id: Uuid::new_v4().to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "paper".to_string(),
        trigger: "once".to_string(),
        status: StrategyRunStatus::Running,
        symbol: symbol.to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        started_at: fixed_ts(),
        finished_at: None,
        metadata_json: json!({"short": 5, "long": 20}),
    }
}

fn sample_order(run_id: &str, client_order_id: &str) -> OrderRecord {
    OrderRecord {
        order_id: Uuid::new_v4().to_string(),
        client_order_id: client_order_id.to_string(),
        run_id: run_id.to_string(),
        symbol: "000001".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        requested_quantity: 100,
        requested_price: dec!(12.34),
        filled_quantity: 0,
        remaining_quantity: 100,
        avg_fill_price: None,
        status: OrderStatus::PendingSubmit,
        adapter: "paper".to_string(),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        last_transition_at: fixed_ts(),
        version: 0,
        payload_json: json!({"reason": "ma_cross_buy"}),
    }
}

fn sample_signal(
    run_id: &str,
    signal_id: &str,
    bar_end: chrono::DateTime<Utc>,
) -> StrategySignalRecord {
    StrategySignalRecord {
        signal_id: signal_id.to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        signal_value: "buy".to_string(),
        signal_status: SignalStatus::New,
        approval_status: ApprovalStatus::Pending,
        run_id: run_id.to_string(),
        metadata_json: json!({"fast": 5, "slow": 20}),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
    }
}

fn sample_daemon_checkpoint(last_run_id: &str) -> StrategyDaemonCheckpointRecord {
    StrategyDaemonCheckpointRecord {
        checkpoint_id: Uuid::new_v4().to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        last_processed_bar: Some(fixed_ts()),
        last_run_id: Some(last_run_id.to_string()),
        state_json: json!({"bootstrap_policy": "latest_only"}),
        updated_at: fixed_ts(),
    }
}

#[tokio::test]
async fn bootstrap_creates_phase29a_schema() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");

    let store = StrategyRuntimeStore::new(&path).await.unwrap();

    assert!(store.has_table("strategy_runs").await.unwrap());
    assert!(store.has_table("signal_events").await.unwrap());
    assert!(store.has_table("orders").await.unwrap());
    assert!(store.has_table("order_events").await.unwrap());
    assert!(store.has_table("runner_checkpoints").await.unwrap());
    assert!(store.has_table("signals").await.unwrap());
    assert!(store.has_table("execution_requests").await.unwrap());
    assert!(
        store
            .has_table("strategy_daemon_checkpoints")
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn insert_run_rejects_duplicate_dedupe_key() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let bar_end = fixed_ts();

    let first = sample_run("000001", bar_end);
    let second = sample_run("000001", bar_end);

    store.insert_run(&first).await.unwrap();
    let err = store.insert_run(&second).await.unwrap_err();

    assert!(err.to_string().contains("strategy_runs"));
}

#[tokio::test]
async fn insert_order_rejects_duplicate_client_order_id() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let first = sample_order(&run.run_id, "run_000001_1");
    let second = sample_order(&run.run_id, "run_000001_1");

    store.insert_order(&first).await.unwrap();
    let err = store.insert_order(&second).await.unwrap_err();

    assert!(err.to_string().contains("client_order_id"));
}

#[tokio::test]
async fn checkpoint_upsert_overwrites_existing_row_for_same_stream() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();

    let first = RunnerCheckpointRecord {
        checkpoint_id: Uuid::new_v4().to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "paper".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        last_processed_bar: Some(fixed_ts()),
        last_run_id: Some("run-a".to_string()),
        state_json: json!({"step": 1}),
        updated_at: fixed_ts(),
    };
    let second = RunnerCheckpointRecord {
        checkpoint_id: Uuid::new_v4().to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "paper".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        last_processed_bar: Some(fixed_ts() + chrono::Duration::days(1)),
        last_run_id: Some("run-b".to_string()),
        state_json: json!({"step": 2}),
        updated_at: fixed_ts() + chrono::Duration::minutes(5),
    };

    store.upsert_checkpoint(&first).await.unwrap();
    store.upsert_checkpoint(&second).await.unwrap();

    let saved = store
        .load_checkpoint("ma_cross", "paper", "000001", "1d")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(saved.last_run_id.as_deref(), Some("run-b"));
    assert_eq!(saved.state_json, json!({"step": 2}));
}

#[tokio::test]
async fn order_events_round_trip_against_existing_order() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let order = sample_order(&run.run_id, "run_000001_1");
    store.insert_order(&order).await.unwrap();

    let event = OrderEventRecord {
        event_id: Uuid::new_v4().to_string(),
        order_id: order.order_id.clone(),
        client_order_id: order.client_order_id.clone(),
        event_type: "submitted".to_string(),
        event_time: fixed_ts(),
        details_json: json!({"status": "submitted"}),
    };

    store.insert_order_event(&event).await.unwrap();

    let events = store.list_order_events(&order.order_id).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "submitted");
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
    assert_eq!(
        ExecutionRequestStatus::from_str("canceled"),
        Some(ExecutionRequestStatus::Canceled)
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

#[tokio::test]
async fn insert_signal_rejects_duplicate_stream_bar_key() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let first = sample_signal(&run.run_id, "signal-1", fixed_ts());
    let second = sample_signal(&run.run_id, "signal-2", fixed_ts());

    store.insert_signal(&first).await.unwrap();
    let err = store.insert_signal(&second).await.unwrap_err();

    assert!(err.to_string().contains("signals"));
}

#[tokio::test]
async fn approve_signal_creates_exactly_one_pending_execution_request() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    let signal = sample_signal(&run.run_id, "signal-approve", fixed_ts());
    store.insert_signal(&signal).await.unwrap();

    let request = store
        .approve_signal_and_create_request("signal-approve", "paper", "default", Some("cli-user"))
        .await
        .unwrap();

    assert_eq!(request.request_status, ExecutionRequestStatus::Pending);
    assert_eq!(request.signal_id, "signal-approve");

    let saved_signal = store.get_signal("signal-approve").await.unwrap().unwrap();
    assert_eq!(saved_signal.approval_status, ApprovalStatus::Approved);

    let requests = store.list_execution_requests(None).await.unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].signal_id, "signal-approve");
}

#[tokio::test]
async fn reject_signal_updates_approval_state_without_creating_request() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    let signal = sample_signal(&run.run_id, "signal-reject", fixed_ts());
    store.insert_signal(&signal).await.unwrap();

    store
        .reject_signal("signal-reject", Some("manual reject"))
        .await
        .unwrap();

    let saved_signal = store.get_signal("signal-reject").await.unwrap().unwrap();
    assert_eq!(saved_signal.approval_status, ApprovalStatus::Rejected);
    assert_eq!(
        saved_signal.metadata_json["rejection_reason"],
        "manual reject"
    );
    assert!(
        store
            .list_execution_requests(None)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn superseding_signal_cancels_pending_execution_request() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let old_signal = sample_signal(&run.run_id, "signal-old", fixed_ts());
    store.insert_signal(&old_signal).await.unwrap();
    store
        .approve_signal_and_create_request("signal-old", "paper", "default", Some("cli-user"))
        .await
        .unwrap();

    let new_signal = sample_signal(
        &run.run_id,
        "signal-new",
        fixed_ts() + chrono::Duration::days(1),
    );
    store.insert_signal(&new_signal).await.unwrap();

    let superseded = store
        .supersede_previous_signals_and_cancel_pending_requests(
            "ma_fast_5_slow_20",
            "000001",
            "1d",
            "signal-new",
            fixed_ts() + chrono::Duration::days(1),
        )
        .await
        .unwrap();

    assert_eq!(superseded, 1);

    let saved_old_signal = store.get_signal("signal-old").await.unwrap().unwrap();
    assert_eq!(saved_old_signal.signal_status, SignalStatus::Superseded);

    let request = store
        .get_execution_request_by_signal_id("signal-old")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(request.request_status, ExecutionRequestStatus::Canceled);
}

#[tokio::test]
async fn daemon_checkpoint_upsert_overwrites_existing_row_for_same_stream() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();

    let first = sample_daemon_checkpoint("run-a");
    let mut second = sample_daemon_checkpoint("run-b");
    second.last_processed_bar = Some(fixed_ts() + chrono::Duration::days(1));
    second.updated_at = fixed_ts() + chrono::Duration::minutes(10);

    store.upsert_daemon_checkpoint(&first).await.unwrap();
    store.upsert_daemon_checkpoint(&second).await.unwrap();

    let saved = store
        .find_daemon_checkpoint("ma_fast_5_slow_20", "000001", "1d")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(saved.last_run_id.as_deref(), Some("run-b"));
    assert_eq!(saved.last_processed_bar, second.last_processed_bar);
}
