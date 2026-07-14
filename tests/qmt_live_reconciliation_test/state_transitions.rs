use super::*;

#[tokio::test]
async fn qmt_live_reconciliation_keeps_pending_submit_when_task_result_is_pending() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    seed_qmt_live_order(
        &store,
        &run.run_id,
        "req-qmt-live-1",
        OrderStatus::PendingSubmit,
        qmt_live_metadata(Some("task-1"), "req-qmt-live-1"),
    )
    .await;

    let server = MockServer::start().await;
    mock_task_result_pending(&server).await;
    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 30_000).unwrap();
    let reconciliation = ReconciliationService::with_qmt_live_query_service(store.clone(), service);

    reconciliation.reconcile_all().await.unwrap();

    let saved = store
        .find_order_by_client_order_id("req-qmt-live-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::PendingSubmit);
    assert_eq!(
        saved.payload_json["qmt_live"]["last_query"]["latest_status"],
        "pending_submit"
    );
    assert_eq!(
        saved.payload_json["qmt_live"]["reconciliation"]["last_action"],
        "no_action"
    );
}

#[tokio::test]
async fn qmt_live_reconciliation_marks_rejected_and_persists_reason() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    seed_qmt_live_order(
        &store,
        &run.run_id,
        "req-qmt-live-reject",
        OrderStatus::Accepted,
        qmt_live_metadata(Some("task-1"), "req-qmt-live-reject"),
    )
    .await;

    let server = MockServer::start().await;
    mock_task_result_reject(&server).await;
    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 30_000).unwrap();
    let reconciliation = ReconciliationService::with_qmt_live_query_service(store.clone(), service);

    reconciliation.reconcile_all().await.unwrap();

    let saved = store
        .find_order_by_client_order_id("req-qmt-live-reject")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::Rejected);
    assert_eq!(
        saved.payload_json["qmt_live"]["last_query"]["latest_status"],
        "rejected"
    );
    assert_eq!(
        saved.payload_json["qmt_live"]["last_query"]["rejection_reason"],
        "price rejected"
    );
    assert_eq!(
        saved.payload_json["qmt_live"]["reconciliation"]["last_action"],
        "state_updated"
    );
}

#[tokio::test]
async fn qmt_live_reconciliation_persists_external_order_id_from_task_result() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    seed_qmt_live_order(
        &store,
        &run.run_id,
        "req-qmt-live-ack",
        OrderStatus::PendingSubmit,
        qmt_live_metadata(Some("task-1"), "req-qmt-live-ack"),
    )
    .await;

    let server = MockServer::start().await;
    mock_task_result_ack_with_external_order_id(&server).await;
    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 30_000).unwrap();
    let reconciliation = ReconciliationService::with_qmt_live_query_service(store.clone(), service);

    reconciliation.reconcile_all().await.unwrap();

    let saved = store
        .find_order_by_client_order_id("req-qmt-live-ack")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::Accepted);
    assert_eq!(
        saved.payload_json["qmt_live"]["task_identity"]["external_order_id"],
        "broker-ack-1"
    );
}

#[tokio::test]
async fn qmt_live_reconciliation_marks_filled_when_task_result_reports_execution() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    seed_qmt_live_order(
        &store,
        &run.run_id,
        "req-qmt-live-execution",
        OrderStatus::Accepted,
        qmt_live_metadata(Some("task-1"), "req-qmt-live-execution"),
    )
    .await;

    let server = MockServer::start().await;
    mock_task_result_execution(&server).await;
    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 30_000).unwrap();
    let reconciliation = ReconciliationService::with_qmt_live_query_service(store.clone(), service);

    reconciliation.reconcile_all().await.unwrap();

    let saved = store
        .find_order_by_client_order_id("req-qmt-live-execution")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::Filled);
    assert_eq!(
        saved.payload_json["qmt_live"]["last_query"]["latest_status"],
        "filled"
    );
    assert_eq!(
        saved.payload_json["qmt_live"]["last_query"]["broker_event_type"],
        "execution"
    );
    assert_eq!(
        saved.payload_json["qmt_live"]["reconciliation"]["last_action"],
        "state_updated"
    );
}

#[tokio::test]
async fn qmt_live_reconciliation_preserves_local_state_when_task_result_query_fails() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    seed_qmt_live_order(
        &store,
        &run.run_id,
        "req-qmt-live-timeout",
        OrderStatus::Accepted,
        qmt_live_metadata(Some("task-1"), "req-qmt-live-timeout"),
    )
    .await;

    let server = MockServer::start().await;
    mock_task_result_failed_timeout(&server).await;
    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 30_000).unwrap();
    let reconciliation = ReconciliationService::with_qmt_live_query_service(store.clone(), service);

    reconciliation.reconcile_all().await.unwrap();

    let saved = store
        .find_order_by_client_order_id("req-qmt-live-timeout")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::Accepted);
    assert_eq!(
        saved.payload_json["qmt_live"]["reconciliation"]["last_action"],
        "manual_intervention"
    );
    let last_error = saved.payload_json["qmt_live"]["reconciliation"]["last_error"]
        .as_str()
        .unwrap();
    assert!(last_error.contains("timed out"));
    assert!(last_error.contains("deadline exceeded"));
}
