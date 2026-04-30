use quantix_cli::bridge::client::BridgeHttpClient;
use quantix_cli::bridge::error::BridgeError;
use quantix_cli::execution::adapter::AdapterOrderRequest;
use quantix_cli::execution::models::OrderSide;
use quantix_cli::execution::qmt_task_submit_service::QmtTaskSubmitService;
use rust_decimal_macros::dec;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn qmt_task_submit_service_returns_receipt_with_local_submission_id() {
    let server = MockServer::start().await;
    mock_task_execute_accepted(&server).await;

    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 10).unwrap();
    let receipt = service
        .submit_order(&sample_adapter_request("cli-1"))
        .await
        .unwrap();

    assert_eq!(receipt.task_id, "task-1");
    assert_eq!(receipt.client_order_id, "cli-1");
    assert_eq!(receipt.bridge_contract_version, "miniqmt.v1");
    assert!(!receipt.local_submission_id.is_empty());
}

#[tokio::test]
async fn qmt_task_submit_service_rejects_identity_mismatch() {
    let server = MockServer::start().await;
    mock_task_result_identity_mismatch(&server).await;

    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 10).unwrap();
    let error = service
        .query_task_result_once("task-1", "cli-1", "local-1")
        .await
        .unwrap_err();

    assert!(matches!(error, BridgeError::InvalidResult(_)));
}

#[tokio::test]
async fn qmt_task_submit_service_poll_until_terminal_times_out_on_repeated_pending() {
    let server = MockServer::start().await;
    mock_task_result_pending(&server).await;

    let service = QmtTaskSubmitService::new(sample_client(&server), 1, 5).unwrap();
    let error = service
        .poll_task_result_until_terminal("task-1", "cli-1", "local-1")
        .await
        .unwrap_err();

    assert!(matches!(error, BridgeError::Timeout(_)));
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

fn sample_adapter_request(client_order_id: &str) -> AdapterOrderRequest {
    AdapterOrderRequest {
        client_order_id: client_order_id.to_string(),
        symbol: "600000.SH".to_string(),
        side: OrderSide::Buy,
        quantity: 100,
        price: dec!(10.50),
    }
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

async fn mock_task_result_identity_mismatch(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .and(header("authorization", "Bearer bearer-123"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "completed",
            "bridge_contract_version": "miniqmt.v1",
            "result": {
                "client_order_id": "other-client",
                "local_submission_id": "other-local",
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
