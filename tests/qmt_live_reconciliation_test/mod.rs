//! QMT live-order reconciliation integration tests, split by concern.
//!
//! Historical note: `qmt_live_reconciliation_test.rs` was 628 lines and was split:
//! - [`state_transitions`] — the five state-transition cases (pending kept,
//!   rejected, external-order-id persisted, filled, manual-intervention on
//!   query failure),
//! - [`identity`] — the four identity/preservation cases (recover missing
//!   fields, preserve unrelated payload keys, client_order_id mismatch,
//!   missing task_id).
//!
//! Shared helpers (`fixed_ts`, `sample_run`, `sample_client`, the five
//! `mock_task_result_*` fns, `qmt_live_metadata`, `seed_qmt_live_order`) live here.

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

pub(super) fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 5, 3, 9, 30, 0).unwrap()
}

pub(super) fn sample_run(symbol: &str, bar_end: chrono::DateTime<Utc>) -> StrategyRunRecord {
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

pub(super) fn sample_client(server: &MockServer) -> BridgeHttpClient {
    BridgeHttpClient::new_with_contract(
        server.uri(),
        Some("legacy-key".to_string()),
        Some("bearer-123".to_string()),
        "miniqmt.v1".to_string(),
        30_000,
    )
    .unwrap()
}

pub(super) async fn mock_task_result_pending(server: &MockServer) {
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

pub(super) async fn mock_task_result_reject(server: &MockServer) {
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

pub(super) async fn mock_task_result_ack_with_external_order_id(server: &MockServer) {
    mock_task_result_ack_with_external_order_id_for(server, "req-qmt-live-ack").await;
}

async fn mock_task_result_ack_with_external_order_id_for(
    server: &MockServer,
    client_order_id: &str,
) {
    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .and(header("authorization", "Bearer bearer-123"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "completed",
            "bridge_contract_version": "miniqmt.v1",
            "result": {
                "client_order_id": client_order_id,
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

pub(super) async fn mock_task_result_execution(server: &MockServer) {
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

pub(super) async fn mock_task_result_failed_timeout(server: &MockServer) {
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

pub(super) fn qmt_live_metadata(task_id: Option<&str>, client_order_id: &str) -> serde_json::Value {
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

pub(super) async fn seed_qmt_live_order(
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

mod identity;
mod state_transitions;
