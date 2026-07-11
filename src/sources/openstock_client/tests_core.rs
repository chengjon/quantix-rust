//! Core tests for openstock_client: envelope parsing, from_settings,
//! retry loop, circuit breaker.

use super::test_support::{Rec, fast_test_cfg, success_body};
use super::*;
use serde_json::json;

#[test]
fn from_envelope_records_source_and_artifact_hash() {
    let raw = r#"{"data":[{"code":"600000"}],"source":"eltdx","received_at":"2026-06-30T10:00:00+08:00"}"#;
    let env: OpenStockEnvelope<Rec> = serde_json::from_str(raw).unwrap();
    let resp = OpenStockResponse::from_envelope(env, raw);
    assert_eq!(resp.records.len(), 1);
    assert_eq!(resp.records[0].code, "600000");
    assert_eq!(resp.source, "eltdx");
    assert_eq!(resp.artifact_hash.len(), 64);
    assert!(resp.received_at.is_some());
}

#[test]
fn from_envelope_defaults_missing_source() {
    let raw = r#"{"data":[]}"#;
    let env: OpenStockEnvelope<Rec> = serde_json::from_str(raw).unwrap();
    let resp = OpenStockResponse::from_envelope(env, raw);
    assert_eq!(resp.source, "");
    assert!(resp.records.is_empty());
}

#[test]
fn from_envelope_artifact_hash_stable_for_same_body() {
    let raw = r#"{"data":[{"code":"600000"}]}"#;
    let env: OpenStockEnvelope<Rec> = serde_json::from_str(raw).unwrap();
    let resp_a = OpenStockResponse::from_envelope(env.clone(), raw);
    let resp_b = OpenStockResponse::from_envelope(env, raw);
    assert_eq!(resp_a.artifact_hash, resp_b.artifact_hash);
}

// -----------------------------------------------------------------
// from_settings tests
// -----------------------------------------------------------------

#[test]
fn from_settings_builds_client_when_credentials_present() {
    use crate::core::runtime::OpenStockSettings;
    let settings = OpenStockSettings {
        base_url: Some("http://example.test:8040".to_string()),
        api_key: Some("sk-test".to_string()),
        timeout_secs: 5,
    };
    let client = OpenStockClient::from_settings(&settings).expect("client build");
    assert_eq!(client.config.timeout, Duration::from_secs(5));
    assert_eq!(client.config.max_retries, DEFAULT_MAX_RETRIES);
}

#[test]
fn from_settings_errors_when_base_url_missing() {
    use crate::core::runtime::OpenStockSettings;
    let settings = OpenStockSettings {
        base_url: None,
        api_key: Some("sk-test".to_string()),
        timeout_secs: 30,
    };
    let result = OpenStockClient::from_settings(&settings);
    assert!(matches!(result, Err(QuantixError::Config(_))));
}

#[test]
fn from_settings_errors_when_api_key_missing() {
    use crate::core::runtime::OpenStockSettings;
    let settings = OpenStockSettings {
        base_url: Some("http://example.test:8040".to_string()),
        api_key: None,
        timeout_secs: 30,
    };
    let result = OpenStockClient::from_settings(&settings);
    assert!(matches!(result, Err(QuantixError::Config(_))));
}

// -----------------------------------------------------------------
// Retry + circuit breaker tests (wiremock-based)
// -----------------------------------------------------------------

#[tokio::test]
async fn fetch_retries_on_5xx_then_succeeds() {
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .and(header("X-API-Key", "test-key"))
        .respond_with(ResponseTemplate::new(503).set_body_string("upstream down"))
        .up_to_n_times(2)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .and(header("X-API-Key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_string(success_body()))
        .mount(&server)
        .await;

    let resp: OpenStockResponse<Rec> = client
        .fetch("STOCK_CODES", json!({}))
        .await
        .expect("fetch ok");
    assert_eq!(resp.records.len(), 1);
    assert_eq!(resp.records[0].code, "600000");
}

#[tokio::test]
async fn fetch_does_not_retry_on_4xx() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_string(r#"{"code":"bad_request","message":"nope"}"#),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = client
        .fetch::<Rec>("STOCK_CODES", json!({}))
        .await
        .expect_err("should fail");
    let msg = format!("{:?}", err);
    assert!(msg.contains("bad_request"), "msg={}", msg);
}

#[tokio::test]
async fn fetch_does_not_retry_on_corrupt_2xx() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .expect(1)
        .mount(&server)
        .await;

    let err = client
        .fetch::<Rec>("STOCK_CODES", json!({}))
        .await
        .expect_err("should fail");
    let msg = format!("{:?}", err);
    assert!(msg.contains("cannot parse success envelope"), "msg={}", msg);
}

#[tokio::test]
async fn fetch_retries_on_network_error_then_exhausts() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let cfg = OpenStockClientConfig {
        base_url: server.uri(),
        api_key: "test-key".to_string(),
        timeout: Duration::from_millis(50), // tight timeout → send error
        max_retries: 1,
        retry_base_delay: Duration::from_millis(5),
        circuit_break_threshold: 0, // disable to isolate retry path
        circuit_break_cooldown: Duration::from_secs(60),
    };
    let client = OpenStockClient::new(cfg).expect("client build");

    // Slow response triggers client timeout (50ms) → reqwest send error.
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(500)))
        // expect 2 calls: 1 initial + 1 retry (max_retries=1)
        .expect(2)
        .mount(&server)
        .await;

    let _ = client.fetch::<Rec>("STOCK_CODES", json!({})).await;
    // assertions verified by wiremock `expect(2)` on drop
}

#[tokio::test]
async fn circuit_breaker_trips_after_threshold() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let cfg = OpenStockClientConfig {
        base_url: server.uri(),
        api_key: "test-key".to_string(),
        timeout: Duration::from_secs(1),
        max_retries: 0, // no retry → each fetch = 1 call
        retry_base_delay: Duration::from_millis(5),
        circuit_break_threshold: 2,
        circuit_break_cooldown: Duration::from_millis(50),
    };
    let client = OpenStockClient::new(cfg).expect("client build");

    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(500).set_body_string("down"))
        .expect(2)
        .mount(&server)
        .await;

    // 1st failure: consecutive_failures=1, not tripped yet
    let _ = client.fetch::<Rec>("STOCK_CODES", json!({})).await;
    // 2nd failure: consecutive_failures=2 → trips
    let _ = client.fetch::<Rec>("STOCK_CODES", json!({})).await;
    // 3rd call: circuit open, should be short-circuited (no HTTP)
    let err = client
        .fetch::<Rec>("STOCK_CODES", json!({}))
        .await
        .expect_err("should be short-circuited");
    let msg = format!("{:?}", err);
    assert!(msg.contains("circuit breaker open"), "msg={}", msg);
}

#[tokio::test]
async fn circuit_breaker_resets_after_cooldown() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let cfg = OpenStockClientConfig {
        base_url: server.uri(),
        api_key: "test-key".to_string(),
        timeout: Duration::from_secs(1),
        max_retries: 0,
        retry_base_delay: Duration::from_millis(5),
        circuit_break_threshold: 1, // trips after just 1 failure
        circuit_break_cooldown: Duration::from_millis(30),
    };
    let client = OpenStockClient::new(cfg).expect("client build");

    // Phase 1: fail once → trips
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(500).set_body_string("down"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    let _ = client.fetch::<Rec>("STOCK_CODES", json!({})).await;

    // Phase 2: cooldown elapses → next request should be served
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(200).set_body_string(success_body()))
        .expect(1)
        .mount(&server)
        .await;

    tokio::time::sleep(Duration::from_millis(50)).await;
    let resp: OpenStockResponse<Rec> = client
        .fetch("STOCK_CODES", json!({}))
        .await
        .expect("fetch ok after cooldown");
    assert_eq!(resp.records.len(), 1);
}

#[tokio::test]
async fn circuit_breaker_disabled_when_threshold_zero() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let cfg = OpenStockClientConfig {
        base_url: server.uri(),
        api_key: "test-key".to_string(),
        timeout: Duration::from_secs(1),
        max_retries: 0,
        retry_base_delay: Duration::from_millis(5),
        circuit_break_threshold: 0, // disabled
        circuit_break_cooldown: Duration::from_secs(60),
    };
    let client = OpenStockClient::new(cfg).expect("client build");

    // Each call hits the server; no short-circuit even after many failures.
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(500).set_body_string("down"))
        .expect(3)
        .mount(&server)
        .await;

    for _ in 0..3 {
        let _ = client.fetch::<Rec>("STOCK_CODES", json!({})).await;
    }
    // Verified by `expect(3)` on drop.
}

#[tokio::test]
async fn success_resets_circuit() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let cfg = OpenStockClientConfig {
        base_url: server.uri(),
        api_key: "test-key".to_string(),
        timeout: Duration::from_secs(1),
        max_retries: 0,
        retry_base_delay: Duration::from_millis(5),
        circuit_break_threshold: 2,
        circuit_break_cooldown: Duration::from_millis(50),
    };
    let client = OpenStockClient::new(cfg).expect("client build");

    // 1st failure → consecutive_failures=1
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(500).set_body_string("down"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    let _ = client.fetch::<Rec>("STOCK_CODES", json!({})).await;

    // 2nd call: success → resets consecutive_failures to 0
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(200).set_body_string(success_body()))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    let _: OpenStockResponse<Rec> = client
        .fetch("STOCK_CODES", json!({}))
        .await
        .expect("fetch ok");

    // 3rd call: failure again → consecutive_failures should be 1 (not 2)
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(500).set_body_string("down"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    let _ = client.fetch::<Rec>("STOCK_CODES", json!({})).await;

    // 4th call: should NOT be short-circuited (consecutive_failures=1 < threshold=2)
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(500).set_body_string("down"))
        .expect(1)
        .mount(&server)
        .await;
    let _ = client.fetch::<Rec>("STOCK_CODES", json!({})).await;
    // Verified by `expect(1)` on drop — circuit did NOT open.
}
