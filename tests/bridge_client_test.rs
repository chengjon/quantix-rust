use quantix_cli::bridge::client::BridgeHttpClient;
use quantix_cli::core::runtime::{BRIDGE_API_KEY_ENV, BRIDGE_BASE_URL_ENV, CliRuntime};
use std::sync::{Mutex, OnceLock};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}

#[test]
fn runtime_loads_bridge_settings_from_env() {
    let _lock = env_lock();
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, "http://127.0.0.1:17580");
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    let runtime = CliRuntime::load();

    assert_eq!(runtime.bridge.base_url, "http://127.0.0.1:17580");
    assert_eq!(runtime.bridge.api_key.as_deref(), Some("bridge-test-key"));

    unsafe {
        std::env::remove_var(BRIDGE_BASE_URL_ENV);
        std::env::remove_var(BRIDGE_API_KEY_ENV);
    }
}

#[tokio::test]
async fn bridge_client_fetches_capabilities_with_api_key() {
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
    let capabilities = client.capabilities().await.unwrap();

    assert!(capabilities.tdx.enabled);
    assert_eq!(capabilities.qmt.mode, "preview_only");
}
