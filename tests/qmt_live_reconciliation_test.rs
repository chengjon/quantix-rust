use chrono::{TimeZone, Utc};
use quantix_cli::bridge::client::BridgeHttpClient;
use quantix_cli::execution::models::{
    OrderRecord, OrderSide, OrderStatus, OrderType, StrategyRunRecord, StrategyRunStatus,
};
use quantix_cli::execution::qmt_task_submit_service::QmtTaskSubmitService;
use quantix_cli::execution::reconciliation::ReconciliationService;
use quantix_cli::execution::runtime_store::StrategyRuntimeStore;
use rust_decimal_macros::dec;
use tempfile::tempdir;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 5, 3, 9, 30, 0).unwrap()
}

fn sample_run(symbol: &str, bar_end: chrono::DateTime<Utc>) -> StrategyRunRecord {
    StrategyRunRecord {
        run_id: uuid::Uuid::new_v4().to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "paper".to_string(),
        trigger: "once".to_string(),
        status: StrategyRunStatus::Running,
        symbol: symbol.to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        started_at: fixed_ts(),
        finished_at: None,
        metadata_json: serde_json::json!({"short": 5, "long": 20}),
    }
}

fn sample_client(server: &MockServer) -> BridgeHttpClient {
    BridgeHttpClient::new_with_contract(
        server.uri(),
        Some("legacy-key".to_string()),
        Some("bearer-123".to_string()),
        "miniqmt.v1".to_string(),
        30_000,
    )
    .unwrap()
}

async fn mock_task_result_pending(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .and(header("authorization", "Bearer bearer-123"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "pending",
            "bridge_contract_version": "miniqmt.v1",
            "result": null
        })))
        .mount(server)
        .await;
}

async fn mock_task_result_reject(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .and(header("authorization", "Bearer bearer-123"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "completed",
            "bridge_contract_version": "miniqmt.v1",
            "result": {
                "client_order_id": "req-qmt-live-reject",
                "local_submission_id": "local-1",
                "account_scope": "sim",
                "event_id": "evt-1",
                "occurred_at": "2026-05-03T09:31:00Z",
                "source_name": "miniqmt",
                "broker_event_type": "reject",
                "external_order_id": null,
                "reason_code": "live_bridge_invalid_result",
                "reason_detail": "price rejected",
                "evidence_ref": "evidence-1"
            }
        })))
        .mount(server)
        .await;
}

async fn mock_task_result_ack_with_external_order_id(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .and(header("authorization", "Bearer bearer-123"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "completed",
            "bridge_contract_version": "miniqmt.v1",
            "result": {
                "client_order_id": "req-qmt-live-ack",
                "local_submission_id": "local-1",
                "account_scope": "sim",
                "event_id": "evt-ack-1",
                "occurred_at": "2026-05-03T09:31:00Z",
                "source_name": "miniqmt",
                "broker_event_type": "acknowledgement",
                "external_order_id": "broker-ack-1",
                "reason_code": null,
                "reason_detail": null,
                "evidence_ref": "evidence-ack-1"
            }
        })))
        .mount(server)
        .await;
}

async fn mock_task_result_execution(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .and(header("authorization", "Bearer bearer-123"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "completed",
            "bridge_contract_version": "miniqmt.v1",
            "result": {
                "client_order_id": "req-qmt-live-execution",
                "local_submission_id": "local-1",
                "account_scope": "sim",
                "event_id": "evt-exec-1",
                "occurred_at": "2026-05-03T09:31:00Z",
                "source_name": "miniqmt",
                "broker_event_type": "execution",
                "external_order_id": "broker-fill-1",
                "reason_code": null,
                "reason_detail": null,
                "evidence_ref": "evidence-exec-1"
            }
        })))
        .mount(server)
        .await;
}

async fn mock_task_result_failed_timeout(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .and(header("authorization", "Bearer bearer-123"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "failed",
            "bridge_contract_version": "miniqmt.v1",
            "result": {
                "client_order_id": "req-qmt-live-timeout",
                "local_submission_id": "local-1",
                "account_scope": "sim",
                "event_id": "evt-timeout-1",
                "occurred_at": "2026-05-03T09:31:00Z",
                "source_name": "miniqmt",
                "reason_code": "live_bridge_timeout",
                "reason_detail": "deadline exceeded"
            }
        })))
        .mount(server)
        .await;
}

fn qmt_live_metadata(task_id: Option<&str>, client_order_id: &str) -> serde_json::Value {
    serde_json::json!({
        "qmt_live": {
            "task_identity": {
                "task_id": task_id,
                "client_order_id": client_order_id,
                "local_submission_id": "local-1",
                "external_order_id": null
            }
        }
    })
}

async fn seed_qmt_live_order(
    store: &StrategyRuntimeStore,
    run_id: &str,
    client_order_id: &str,
    status: OrderStatus,
    payload_json: serde_json::Value,
) {
    store
        .insert_order(&OrderRecord {
            order_id: client_order_id.to_string(),
            client_order_id: client_order_id.to_string(),
            run_id: run_id.to_string(),
            symbol: "000001".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            requested_quantity: 100,
            requested_price: dec!(10.50),
            filled_quantity: 0,
            remaining_quantity: 100,
            avg_fill_price: None,
            status,
            adapter: "qmt_live".to_string(),
            created_at: fixed_ts(),
            updated_at: fixed_ts(),
            last_transition_at: fixed_ts(),
            version: 0,
            payload_json,
        })
        .await
        .unwrap();
}

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
    mock_task_result_ack_with_external_order_id(&server).await;
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
