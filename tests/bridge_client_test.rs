#![allow(clippy::await_holding_lock)]

use quantix_cli::bridge::client::BridgeHttpClient;
use quantix_cli::bridge::error::BridgeError;
use quantix_cli::bridge::models::{
    BridgeFailureCode, BridgeTaskExecuteParams, BridgeTaskExecuteRequest,
    BridgeTaskLifecycleStatus, BridgeTaskResultResponse,
};
use quantix_cli::core::runtime::{BRIDGE_API_KEY_ENV, BRIDGE_BASE_URL_ENV, CliRuntime};
use std::sync::{Mutex, OnceLock};
use wiremock::matchers::{body_partial_json, header, method, path};
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

#[test]
fn bridge_task_execute_request_serializes_expected_shape() {
    let payload = sample_task_execute_request();

    let value = serde_json::to_value(payload).unwrap();
    assert_eq!(value["provider"], "qmt");
    assert_eq!(value["method"], "submit_order");
    assert_eq!(value["params"]["request_id"], "req-1");
    assert_eq!(value["params"]["client_order_id"], "cli-1");
    assert_eq!(value["params"]["local_submission_id"], "local-1");
}

#[test]
fn bridge_task_result_response_parses_failure_code() {
    let parsed: BridgeTaskResultResponse = serde_json::from_value(serde_json::json!({
        "task_id": "task-1",
        "status": "failed",
        "bridge_contract_version": "miniqmt.v1",
        "result": {
            "client_order_id": "cli-1",
            "local_submission_id": "local-1",
            "account_scope": "sim",
            "event_id": "evt-1",
            "occurred_at": "2026-04-30T00:00:00Z",
            "source_name": "miniqmt",
            "reason_code": "live_bridge_timeout",
            "reason_detail": "deadline exceeded"
        }
    }))
    .unwrap();

    assert_eq!(
        parsed.result.as_ref().unwrap().reason_code,
        Some(BridgeFailureCode::LiveBridgeTimeout)
    );
}

#[test]
fn bridge_task_result_response_allows_pending_without_result_payload() {
    let parsed: BridgeTaskResultResponse = serde_json::from_value(serde_json::json!({
        "task_id": "task-1",
        "status": "pending",
        "bridge_contract_version": "miniqmt.v1",
        "result": null
    }))
    .unwrap();

    assert_eq!(parsed.task_id, "task-1");
    assert_eq!(parsed.status, BridgeTaskLifecycleStatus::Pending);
    assert!(parsed.result.is_none());
}

#[tokio::test]
async fn bridge_client_submits_task_execute_with_bearer_and_contract_version() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/task/execute"))
        .and(header("authorization", "Bearer bearer-123"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .and(body_partial_json(serde_json::json!({
            "provider": "qmt",
            "method": "submit_order",
            "params": {
                "request_id": "req-1",
                "client_order_id": "cli-1",
                "local_submission_id": "local-1"
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "bridge_task_accepted",
            "receipt_timestamp": "2026-04-30T00:00:00Z",
            "bridge_contract_version": "miniqmt.v1",
            "source_name": "miniqmt"
        })))
        .mount(&server)
        .await;

    let client = BridgeHttpClient::new_with_contract(
        server.uri(),
        Some("legacy-key".to_string()),
        Some("bearer-123".to_string()),
        "miniqmt.v1".to_string(),
        30_000,
    )
    .unwrap();

    let receipt = client
        .task_execute_qmt_submit(&sample_task_execute_request())
        .await
        .unwrap();

    assert_eq!(receipt.task_id, "task-1");
}

#[tokio::test]
async fn bridge_client_task_result_falls_back_to_api_key_without_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .and(header("x-quantix-api-key", "legacy-key"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_task_result_json()))
        .mount(&server)
        .await;

    let client = BridgeHttpClient::new_with_contract(
        server.uri(),
        Some("legacy-key".to_string()),
        None,
        "miniqmt.v1".to_string(),
        30_000,
    )
    .unwrap();

    let result = client.task_result("task-1").await.unwrap();
    assert_eq!(result.task_id, "task-1");
    assert_eq!(result.result.as_ref().unwrap().client_order_id, "cli-1");
}

#[tokio::test]
async fn bridge_client_task_contract_requires_auth_when_no_bearer_or_api_key() {
    let client = BridgeHttpClient::new_with_contract(
        "http://127.0.0.1:17580".to_string(),
        None,
        None,
        "miniqmt.v1".to_string(),
        30_000,
    )
    .unwrap();

    let error = client
        .task_execute_qmt_submit(&sample_task_execute_request())
        .await
        .unwrap_err();

    assert!(matches!(error, BridgeError::Config(_)));
}

#[tokio::test]
async fn bridge_client_maps_unsupported_contract_version_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/task/execute"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "reason_code": "live_bridge_unsupported_contract_version",
            "reason_detail": "contract miniqmt.v0 is not supported"
        })))
        .mount(&server)
        .await;

    let client = BridgeHttpClient::new_with_contract(
        server.uri(),
        Some("legacy-key".to_string()),
        Some("bearer-123".to_string()),
        "miniqmt.v0".to_string(),
        30_000,
    )
    .unwrap();

    let error = client
        .task_execute_qmt_submit(&sample_task_execute_request())
        .await
        .unwrap_err();

    assert!(matches!(error, BridgeError::UnsupportedContractVersion(_)));
}

fn sample_task_execute_request() -> BridgeTaskExecuteRequest {
    BridgeTaskExecuteRequest {
        provider: "qmt".to_string(),
        method: "submit_order".to_string(),
        params: BridgeTaskExecuteParams {
            request_id: "req-1".to_string(),
            client_order_id: "cli-1".to_string(),
            local_submission_id: "local-1".to_string(),
            symbol: "600000.SH".to_string(),
            side: "buy".to_string(),
            quantity: 100,
            price: "10.50".to_string(),
            order_type: "limit".to_string(),
            strategy_name: Some("alpha".to_string()),
            order_remark: Some("manual".to_string()),
            snapshot_metadata: Some(serde_json::json!({
                "source": "unit-test"
            })),
        },
    }
}

fn sample_task_result_json() -> serde_json::Value {
    serde_json::json!({
        "task_id": "task-1",
        "status": "completed",
        "bridge_contract_version": "miniqmt.v1",
        "result": {
            "client_order_id": "cli-1",
            "local_submission_id": "local-1",
            "account_scope": "sim",
            "event_id": "evt-1",
            "occurred_at": "2026-04-30T00:00:00Z",
            "source_name": "miniqmt",
            "broker_event_type": "acknowledgement",
            "external_order_id": "broker-1",
            "reason_code": null,
            "reason_detail": null,
            "evidence_ref": "evidence-1"
        }
    })
}
