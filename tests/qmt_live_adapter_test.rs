use quantix_cli::bridge::client::BridgeHttpClient;
use quantix_cli::execution::adapter::{AdapterOrderRequest, ExecutionAdapter};
use quantix_cli::execution::models::{OrderSide, OrderStatus};
use quantix_cli::execution::qmt_live_adapter::QmtLiveExecutionAdapter;
use rust_decimal_macros::dec;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn qmt_live_adapter_submit_returns_pending_submit_task_receipt() {
    let server = MockServer::start().await;
    mock_live_capabilities(&server).await;
    mock_task_execute_accepted(&server).await;

    let adapter = sample_qmt_live_adapter(&server);
    let response = adapter.submit_order(sample_request("cli-1")).await.unwrap();

    assert_eq!(response.adapter_order_id, "task-1");
    assert_eq!(response.latest_status, OrderStatus::PendingSubmit);
    assert_eq!(response.filled_quantity, 0);
    assert!(response.avg_fill_price.is_none());
    assert!(response.fill_details.is_none());
}

#[tokio::test]
async fn qmt_live_adapter_query_maps_pending_task_result_to_pending_submit() {
    let server = MockServer::start().await;
    mock_task_result_pending(&server).await;

    let adapter = sample_qmt_live_adapter(&server);
    let response = adapter.query_order("task-1").await.unwrap();

    assert_eq!(response.adapter_order_id, "task-1");
    assert_eq!(response.latest_status, OrderStatus::PendingSubmit);
    assert_eq!(response.filled_quantity, 0);
}

#[tokio::test]
async fn qmt_live_adapter_query_maps_acknowledgement_to_accepted() {
    let server = MockServer::start().await;
    mock_task_result_ack(&server).await;

    let adapter = sample_qmt_live_adapter(&server);
    let response = adapter.query_order("task-1").await.unwrap();

    assert_eq!(response.adapter_order_id, "task-1");
    assert_eq!(response.latest_status, OrderStatus::Accepted);
    assert!(response.rejection_reason.is_none());
}

#[tokio::test]
async fn qmt_live_adapter_query_maps_reject_to_rejection_reason() {
    let server = MockServer::start().await;
    mock_task_result_reject(&server).await;

    let adapter = sample_qmt_live_adapter(&server);
    let response = adapter.query_order("task-1").await.unwrap();

    assert_eq!(response.latest_status, OrderStatus::Rejected);
    assert_eq!(response.rejection_reason.as_deref(), Some("price rejected"));
}

#[tokio::test]
async fn qmt_live_adapter_rejects_submit_when_bridge_is_preview_only() {
    let server = MockServer::start().await;
    mock_preview_only_capabilities(&server).await;

    let adapter = sample_qmt_live_adapter(&server);
    let error = adapter.submit_order(sample_request("cli-preview")).await.unwrap_err();

    assert!(error.to_string().contains("bridge qmt.mode=preview_only"));
}

#[tokio::test]
async fn qmt_live_adapter_rejects_submit_when_order_submit_capability_missing() {
    let server = MockServer::start().await;
    mock_live_capabilities_without_order_submit(&server).await;

    let adapter = sample_qmt_live_adapter(&server);
    let error = adapter.submit_order(sample_request("cli-missing-cap")).await.unwrap_err();

    assert!(error.to_string().contains("order_submit"));
}

fn sample_qmt_live_adapter(server: &MockServer) -> QmtLiveExecutionAdapter {
    let client = BridgeHttpClient::new_with_contract(
        server.uri(),
        Some("legacy-key".to_string()),
        Some("bearer-123".to_string()),
        "miniqmt.v1".to_string(),
        30_000,
    )
    .unwrap();

    QmtLiveExecutionAdapter::with_polling(client, 1, 10)
}

fn sample_request(client_order_id: &str) -> AdapterOrderRequest {
    AdapterOrderRequest {
        client_order_id: client_order_id.to_string(),
        symbol: "600000.SH".to_string(),
        side: OrderSide::Buy,
        quantity: 100,
        price: dec!(10.50),
    }
}

async fn mock_live_capabilities(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/capabilities"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tdx": {
                "enabled": true,
                "supports": ["quote", "batch_quotes", "kline"]
            },
            "qmt": {
                "enabled": true,
                "mode": "live",
                "supports": ["order_submit", "account_status"]
            }
        })))
        .mount(server)
        .await;
}

async fn mock_preview_only_capabilities(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/capabilities"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tdx": {
                "enabled": true,
                "supports": ["quote", "batch_quotes", "kline"]
            },
            "qmt": {
                "enabled": true,
                "mode": "preview_only",
                "supports": ["order_submit", "account_status"]
            }
        })))
        .mount(server)
        .await;
}

async fn mock_live_capabilities_without_order_submit(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/capabilities"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tdx": {
                "enabled": true,
                "supports": ["quote", "batch_quotes", "kline"]
            },
            "qmt": {
                "enabled": true,
                "mode": "live",
                "supports": ["account_status"]
            }
        })))
        .mount(server)
        .await;
}

async fn mock_task_execute_accepted(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/api/v1/task/execute"))
        .and(header("authorization", "Bearer bearer-123"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .and(body_partial_json(serde_json::json!({
            "provider": "qmt",
            "method": "submit_order",
            "params": {
                "client_order_id": "cli-1",
                "symbol": "600000.SH",
                "side": "buy"
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "bridge_task_accepted",
            "receipt_timestamp": "2026-05-01T09:30:00Z",
            "bridge_contract_version": "miniqmt.v1",
            "source_name": "miniqmt"
        })))
        .mount(server)
        .await;
}

async fn mock_task_result_pending(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "pending",
            "bridge_contract_version": "miniqmt.v1",
            "result": null
        })))
        .mount(server)
        .await;
}

async fn mock_task_result_ack(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "completed",
            "bridge_contract_version": "miniqmt.v1",
            "result": {
                "client_order_id": "cli-1",
                "local_submission_id": "local-1",
                "account_scope": "sim",
                "event_id": "evt-1",
                "occurred_at": "2026-05-01T09:31:00Z",
                "source_name": "miniqmt",
                "broker_event_type": "acknowledgement",
                "external_order_id": "broker-1",
                "reason_code": null,
                "reason_detail": null,
                "evidence_ref": "evidence-1"
            }
        })))
        .mount(server)
        .await;
}

async fn mock_task_result_reject(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "completed",
            "bridge_contract_version": "miniqmt.v1",
            "result": {
                "client_order_id": "cli-1",
                "local_submission_id": "local-1",
                "account_scope": "sim",
                "event_id": "evt-1",
                "occurred_at": "2026-05-01T09:31:00Z",
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
