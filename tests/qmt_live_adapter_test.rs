use quantix_cli::bridge::client::BridgeHttpClient;
use quantix_cli::execution::adapter::{AdapterOrderRequest, ExecutionAdapter};
use quantix_cli::execution::models::OrderSide;
use quantix_cli::execution::qmt_live_adapter::QmtLiveExecutionAdapter;
use rust_decimal_macros::dec;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_request() -> AdapterOrderRequest {
    AdapterOrderRequest {
        client_order_id: "live-order-1".to_string(),
        symbol: "000001.SZ".to_string(),
        side: OrderSide::Buy,
        quantity: 100,
        price: dec!(15.50),
    }
}

#[tokio::test]
async fn qmt_live_adapter_rejects_submit_when_bridge_is_preview_only() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/capabilities"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tdx": {
                "enabled": true,
                "supports": ["quote", "batch_quotes", "kline"]
            },
            "qmt": {
                "enabled": true,
                "mode": "preview_only",
                "supports": ["account_status", "order_preview"]
            }
        })))
        .mount(&server)
        .await;

    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();
    let adapter = QmtLiveExecutionAdapter::new(client);

    let err = adapter.submit_order(sample_request()).await.unwrap_err();

    assert!(
        err.to_string().contains("preview_only"),
        "expected preview_only safety gate error, got: {err}"
    );
}

#[tokio::test]
async fn qmt_live_adapter_submits_when_bridge_is_live() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/capabilities"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tdx": {
                "enabled": true,
                "supports": ["quote", "batch_quotes", "kline"]
            },
            "qmt": {
                "enabled": true,
                "mode": "live",
                "supports": ["account_status", "order_preview", "order_submit"]
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v1/broker/qmt/orders"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "adapter_order_id": "qmt-order-1",
            "latest_status": "submitted",
            "filled_quantity": 0,
            "avg_fill_price": null,
            "fill_details": null,
            "rejection_reason": null,
            "broker_payload": null
        })))
        .mount(&server)
        .await;

    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();
    let adapter = QmtLiveExecutionAdapter::new(client);

    let response = adapter.submit_order(sample_request()).await.unwrap();

    assert_eq!(response.adapter_order_id, "qmt-order-1");
    assert_eq!(response.latest_status.as_str(), "submitted");
    assert_eq!(response.filled_quantity, 0);
    assert_eq!(response.rejection_reason, None);
}

#[tokio::test]
async fn qmt_live_adapter_rejects_submit_when_bridge_lacks_order_submit_support() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/capabilities"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tdx": {
                "enabled": true,
                "supports": ["quote", "batch_quotes", "kline"]
            },
            "qmt": {
                "enabled": true,
                "mode": "live",
                "supports": ["account_status", "order_preview"]
            }
        })))
        .mount(&server)
        .await;

    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();
    let adapter = QmtLiveExecutionAdapter::new(client);

    let err = adapter.submit_order(sample_request()).await.unwrap_err();

    assert!(
        err.to_string().contains("order_submit"),
        "expected order_submit safety gate error, got: {err}"
    );
}
