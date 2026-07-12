use super::helpers::*;

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
    assert_eq!(
        requests[0].payload_json["execution_snapshot"]["strategy_name"],
        "ma_cross"
    );
    assert_eq!(
        requests[0].payload_json["execution_snapshot"]["strategy_instance_id"],
        "ma_fast_5_slow_20"
    );
    assert_eq!(
        requests[0].payload_json["execution_snapshot"]["symbol"],
        "000001"
    );
    assert_eq!(
        requests[0].payload_json["execution_snapshot"]["signal_value"],
        "buy"
    );
    assert_eq!(
        requests[0].payload_json["execution_snapshot"]["market_price"],
        "12.34"
    );
    assert_eq!(
        requests[0].payload_json["execution_snapshot"]["execution_policy"]["fixed_cash_per_buy"],
        "10000"
    );
    assert_eq!(
        requests[0].payload_json["execution_snapshot"]["order_intent"]["side"],
        "buy"
    );
    assert_eq!(
        requests[0].payload_json["execution_snapshot"]["order_intent"]["requested_quantity"],
        800
    );
    assert_eq!(
        requests[0].payload_json["execution_snapshot"]["order_intent"]["requested_price"],
        "12.34"
    );
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
