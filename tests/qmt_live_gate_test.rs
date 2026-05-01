use quantix_cli::bridge::client::BridgeHttpClient;
use quantix_cli::execution::qmt_live_gate::ensure_bridge_qmt_live_mode;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn bridge_qmt_live_gate_rejects_preview_only_mode() {
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
    let err = ensure_bridge_qmt_live_mode(&client).await.unwrap_err();

    assert!(
        err.to_string().contains("preview_only"),
        "expected preview_only safety gate error, got: {err}"
    );
    assert!(
        err.to_string().contains("bridge qmt.mode=live"),
        "expected explicit qmt.mode=live requirement, got: {err}"
    );
}

#[tokio::test]
async fn bridge_qmt_live_gate_allows_live_mode() {
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

    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();

    ensure_bridge_qmt_live_mode(&client).await.unwrap();
}

#[tokio::test]
async fn bridge_qmt_live_gate_rejects_live_mode_without_order_submit_support() {
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
    let err = ensure_bridge_qmt_live_mode(&client).await.unwrap_err();

    assert!(
        err.to_string().contains("order_submit"),
        "expected order_submit safety gate error, got: {err}"
    );
}
