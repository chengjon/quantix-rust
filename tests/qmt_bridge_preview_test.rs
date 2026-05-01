use chrono::{TimeZone, Utc};
use quantix_cli::bridge::client::BridgeHttpClient;
use quantix_cli::execution::models::ExecutionRequestRecord;
use quantix_cli::execution::qmt_bridge::QmtBridgePreviewAdapter;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_request() -> ExecutionRequestRecord {
    ExecutionRequestRecord {
        request_id: "req-1".to_string(),
        signal_id: "sig-1".to_string(),
        target_mode: "qmt_live".to_string(),
        target_account: "default".to_string(),
        request_status: quantix_cli::execution::models::ExecutionRequestStatus::Pending,
        approved_by: Some("cli".to_string()),
        created_at: Utc.with_ymd_and_hms(2026, 3, 26, 9, 30, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2026, 3, 26, 9, 30, 0).unwrap(),
        payload_json: serde_json::json!({
            "execution_snapshot": {
                "symbol": "000001",
                "order_intent": {
                    "side": "buy",
                    "requested_quantity": 100,
                    "requested_price": "15.50",
                    "order_type": "limit"
                }
            }
        }),
    }
}

#[tokio::test]
async fn qmt_bridge_preview_adapter_uses_frozen_snapshot_payload() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/broker/qmt/orders/preview"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .and(body_partial_json(serde_json::json!({
            "request_id": "req-1",
            "client_order_id": "req-1",
            "symbol": "000001.SZ",
            "side": "buy",
            "quantity": 100,
            "price": "15.50",
            "order_type": "limit"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "adapter_order_id": "preview-req-1",
            "latest_status": "accepted",
            "filled_quantity": 0,
            "avg_fill_price": null,
            "fill_details": null,
            "rejection_reason": null,
            "broker_payload": {
                "market": "SZ",
                "qmt_order_type": "limit"
            }
        })))
        .mount(&server)
        .await;

    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();
    let adapter = QmtBridgePreviewAdapter::new(client);

    let response = adapter.preview_request(&sample_request()).await.unwrap();

    assert_eq!(response.latest_status.as_str(), "accepted");
    assert_eq!(response.filled_quantity, 0);
    assert!(response.fill_details.is_none());
}
