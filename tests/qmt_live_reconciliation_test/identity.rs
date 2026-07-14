use super::*;

#[tokio::test]
async fn qmt_live_reconciliation_recovers_missing_identity_fields_from_completed_result() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    seed_qmt_live_order(
        &store,
        &run.run_id,
        "req-qmt-live-ack-recover",
        OrderStatus::PendingSubmit,
        serde_json::json!({
            "qmt_live": {
                "task_identity": {
                    "task_id": "task-1"
                },
                "operator_note": "keep-me"
            }
        }),
    )
    .await;

    let server = MockServer::start().await;
    mock_task_result_ack_with_external_order_id_for(&server, "req-qmt-live-ack-preserve").await;
    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 30_000).unwrap();
    let reconciliation = ReconciliationService::with_qmt_live_query_service(store.clone(), service);

    reconciliation.reconcile_all().await.unwrap();

    let saved = store
        .find_order_by_client_order_id("req-qmt-live-ack-recover")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::Accepted);
    assert_eq!(
        saved.payload_json["qmt_live"]["task_identity"]["task_id"],
        "task-1"
    );
    assert_eq!(
        saved.payload_json["qmt_live"]["task_identity"]["client_order_id"],
        "req-qmt-live-ack-recover"
    );
    assert_eq!(
        saved.payload_json["qmt_live"]["task_identity"]["local_submission_id"],
        "local-1"
    );
    assert_eq!(
        saved.payload_json["qmt_live"]["task_identity"]["external_order_id"],
        "broker-ack-1"
    );
    assert_eq!(saved.payload_json["qmt_live"]["operator_note"], "keep-me");
    assert_eq!(
        saved.payload_json["qmt_live"]["last_query"]["latest_status"],
        "accepted"
    );
    assert_eq!(
        saved.payload_json["qmt_live"]["reconciliation"]["last_action"],
        "state_updated"
    );
}

#[tokio::test]
async fn qmt_live_reconciliation_preserves_unrelated_payload_keys_when_persisting_query_result() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    seed_qmt_live_order(
        &store,
        &run.run_id,
        "req-qmt-live-ack-preserve",
        OrderStatus::PendingSubmit,
        serde_json::json!({
            "reason": "ma_cross_buy",
            "opaque_root": {
                "strategy_instance_id": "ma_fast_5_slow_20"
            },
            "qmt_live": {
                "task_identity": {
                    "task_id": "task-1",
                    "client_order_id": "req-qmt-live-ack-preserve",
                    "local_submission_id": "local-1",
                    "external_order_id": null
                },
                "operator_note": "keep-me"
            }
        }),
    )
    .await;

    let server = MockServer::start().await;
    mock_task_result_ack_with_external_order_id_for(&server, "req-qmt-live-ack-preserve").await;
    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 30_000).unwrap();
    let reconciliation = ReconciliationService::with_qmt_live_query_service(store.clone(), service);

    reconciliation.reconcile_all().await.unwrap();

    let saved = store
        .find_order_by_client_order_id("req-qmt-live-ack-preserve")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::Accepted);
    assert_eq!(saved.payload_json["reason"], "ma_cross_buy");
    assert_eq!(
        saved.payload_json["opaque_root"]["strategy_instance_id"],
        "ma_fast_5_slow_20"
    );
    assert_eq!(saved.payload_json["qmt_live"]["operator_note"], "keep-me");
    assert_eq!(
        saved.payload_json["qmt_live"]["task_identity"]["external_order_id"],
        "broker-ack-1"
    );
    assert_eq!(
        saved.payload_json["qmt_live"]["last_query"]["latest_status"],
        "accepted"
    );
    assert_eq!(
        saved.payload_json["qmt_live"]["reconciliation"]["last_action"],
        "state_updated"
    );
}

#[tokio::test]
async fn qmt_live_reconciliation_marks_manual_intervention_when_task_result_identity_mismatches() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    seed_qmt_live_order(
        &store,
        &run.run_id,
        "req-qmt-live-identity-local",
        OrderStatus::PendingSubmit,
        qmt_live_metadata(Some("task-1"), "req-qmt-live-identity-local"),
    )
    .await;

    let server = MockServer::start().await;
    mock_task_result_ack_with_external_order_id_for(&server, "req-qmt-live-identity-remote").await;
    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 30_000).unwrap();
    let reconciliation = ReconciliationService::with_qmt_live_query_service(store.clone(), service);

    reconciliation.reconcile_all().await.unwrap();

    let saved = store
        .find_order_by_client_order_id("req-qmt-live-identity-local")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::PendingSubmit);
    assert_eq!(
        saved.payload_json["qmt_live"]["reconciliation"]["last_action"],
        "manual_intervention"
    );
    let last_error = saved.payload_json["qmt_live"]["reconciliation"]["last_error"]
        .as_str()
        .unwrap();
    assert!(last_error.contains("client_order_id mismatch"));
    assert_eq!(
        saved.payload_json["qmt_live"]["task_identity"]["client_order_id"],
        "req-qmt-live-identity-local"
    );
    assert_eq!(
        saved.payload_json["qmt_live"]["task_identity"]["external_order_id"],
        serde_json::Value::Null
    );
}

#[tokio::test]
async fn qmt_live_reconciliation_marks_manual_intervention_when_task_id_missing() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    seed_qmt_live_order(
        &store,
        &run.run_id,
        "req-qmt-live-missing-task",
        OrderStatus::Unknown,
        qmt_live_metadata(None, "req-qmt-live-missing-task"),
    )
    .await;

    let server = MockServer::start().await;
    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 30_000).unwrap();
    let reconciliation = ReconciliationService::with_qmt_live_query_service(store.clone(), service);

    reconciliation.reconcile_all().await.unwrap();

    let saved = store
        .find_order_by_client_order_id("req-qmt-live-missing-task")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::Unknown);
    assert_eq!(
        saved.payload_json["qmt_live"]["reconciliation"]["last_action"],
        "manual_intervention"
    );
    assert!(
        saved.payload_json["qmt_live"]["reconciliation"]["last_error"]
            .as_str()
            .unwrap()
            .contains("task_id")
    );
}
