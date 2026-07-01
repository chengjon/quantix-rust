//! Generic `OpenStockClient` skeleton — reqwest-based, fixture-tested.
//!
//! Knows the uniform `/data/fetch` envelope shape and `X-API-Key`
//! auth. No live HTTP in tests; fixture-only tests exercise
//! [`OpenStockResponse::from_envelope`] and the shared deserialization
//! paths.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use reqwest::Url;
use reqwest::header::HeaderValue;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::core::runtime::OpenStockSettings;
use crate::core::{QuantixError, Result};
use crate::sources::openstock_envelope::{
    OpenStockEnvelope, OpenStockErrorEnvelope, artifact_hash,
};

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_RETRY_BASE_DELAY_MS: u64 = 500;
const DEFAULT_CIRCUIT_BREAK_THRESHOLD: u32 = 5;
const DEFAULT_CIRCUIT_BREAK_COOLDOWN_SECS: u64 = 30;

/// Configuration for constructing an [`OpenStockClient`].
#[derive(Debug, Clone)]
pub struct OpenStockClientConfig {
    pub base_url: String,
    pub api_key: String,
    pub timeout: Duration,

    /// 最大重试次数（不含首次）。
    ///
    /// 0 = 关闭 retry（仅尝试一次）。
    ///
    /// **注意**：`max_retries=0` 时 `circuit_break_threshold` 仍生效 ——
    /// "不重试" 不等于 "服务健康"，单次失败仍会被 circuit breaker 跟踪
    /// 避免对挂掉的 runtime 持续打。若想完全关闭保护，设
    /// `circuit_break_threshold = 0`。
    pub max_retries: u32,

    /// 指数退避基数。第 N 次重试等待 `base * 2^(N-1)`。
    pub retry_base_delay: Duration,

    /// 连续失败多少次后触发 circuit breaker。
    ///
    /// 0 = 关闭熔断（每次请求都跑完整 retry 循环）。
    ///
    /// **注意**：circuit breaker 在所有 category 之间**共享**。任一
    /// category 持续失败会阻塞全部 category 的请求。这是设计选择
    /// （runtime 是单实例，单点故障时全部 category 都不可用）。
    pub circuit_break_threshold: u32,

    /// 熔断后的冷却时间。冷却期内请求直接短路。
    pub circuit_break_cooldown: Duration,
}

impl Default for OpenStockClientConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            api_key: String::new(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            max_retries: DEFAULT_MAX_RETRIES,
            retry_base_delay: Duration::from_millis(DEFAULT_RETRY_BASE_DELAY_MS),
            circuit_break_threshold: DEFAULT_CIRCUIT_BREAK_THRESHOLD,
            circuit_break_cooldown: Duration::from_secs(DEFAULT_CIRCUIT_BREAK_COOLDOWN_SECS),
        }
    }
}

/// Circuit breaker 内部状态。所有 category 共享一个实例（见
/// [`OpenStockClientConfig::circuit_break_threshold`] doc）。
#[derive(Debug, Default)]
struct CircuitState {
    consecutive_failures: u32,
    tripped_until: Option<Instant>,
}

/// Generic OpenStock client. Single backend (reqwest); no trait
/// abstraction — only one backend is expected for this slice.
pub struct OpenStockClient {
    base_url: Url,
    api_key: HeaderValue,
    http: reqwest::Client,
    config: OpenStockClientConfig,
    circuit: Mutex<CircuitState>,
}

impl OpenStockClient {
    /// Build a client from explicit config. If `api_key` is empty,
    /// falls back to `OPENSTOCK_API_KEY` env var. If `base_url` is
    /// empty, falls back to `OPENSTOCK_BASE_URL` env var.
    pub fn new(cfg: OpenStockClientConfig) -> Result<Self> {
        let api_key_raw = if cfg.api_key.is_empty() {
            std::env::var("OPENSTOCK_API_KEY").map_err(|_| {
                QuantixError::Config(
                    "OPENSTOCK_API_KEY not set and no api_key in config".to_string(),
                )
            })?
        } else {
            cfg.api_key.clone()
        };
        let base_url_raw = if cfg.base_url.is_empty() {
            std::env::var("OPENSTOCK_BASE_URL").map_err(|_| {
                QuantixError::Config(
                    "OPENSTOCK_BASE_URL not set and no base_url in config".to_string(),
                )
            })?
        } else {
            cfg.base_url.clone()
        };
        let api_key = HeaderValue::from_str(&api_key_raw)
            .map_err(|e| QuantixError::Config(format!("invalid api_key header: {}", e)))?;
        let base_url = Url::parse(&base_url_raw).map_err(|e| {
            QuantixError::Config(format!("invalid base_url {}: {}", base_url_raw, e))
        })?;
        let http = reqwest::Client::builder()
            .timeout(cfg.timeout)
            .build()
            .map_err(|e| QuantixError::Other(format!("reqwest build failed: {}", e)))?;
        Ok(Self {
            base_url,
            api_key,
            http,
            config: cfg,
            circuit: Mutex::new(CircuitState::default()),
        })
    }

    /// Convenience: build a client entirely from environment variables
    /// (`OPENSTOCK_BASE_URL` + `OPENSTOCK_API_KEY`), with default timeout.
    pub fn from_env() -> Result<Self> {
        Self::new(OpenStockClientConfig::default())
    }

    /// Build a client from [`OpenStockSettings`] loaded into `CliRuntime`.
    ///
    /// Preferred over [`from_env`](Self::from_env) when a `CliRuntime` is
    /// already constructed — keeps env-var reads at the runtime boundary
    /// (matching the `BridgeRuntimeSettings` pattern at
    /// `src/core/runtime/settings.rs:60`).
    ///
    /// Returns `QuantixError::Config` if `base_url` or `api_key` are
    /// missing (env vars not set).
    pub fn from_settings(settings: &OpenStockSettings) -> Result<Self> {
        let base_url = settings
            .base_url
            .clone()
            .ok_or_else(|| QuantixError::Config("OPENSTOCK_BASE_URL not set".to_string()))?;
        let api_key = settings
            .api_key
            .clone()
            .ok_or_else(|| QuantixError::Config("OPENSTOCK_API_KEY not set".to_string()))?;
        Self::new(OpenStockClientConfig {
            base_url,
            api_key,
            timeout: Duration::from_secs(settings.timeout_secs),
            ..OpenStockClientConfig::default()
        })
    }

    /// Generic envelope-aware fetch. POST `/data/fetch` with body
    /// `{"data_category": cat, "params": params}`; on 2xx deserialize
    /// into `OpenStockEnvelope<T>` and compose into
    /// [`OpenStockResponse<T>`]; on non-2xx deserialize into
    /// [`OpenStockErrorEnvelope`] and surface as `QuantixError::Other`.
    ///
    /// Retry & circuit breaker semantics:
    /// - 网络错误 / HTTP 5xx → retry（指数退避，`max_retries` 次）
    /// - HTTP 4xx → fail-fast，不重试，不计入 circuit breaker
    /// - 2xx 但 envelope 解析失败 → fail-fast，不重试
    /// - 重试耗尽 → 计入 circuit breaker；连续 `circuit_break_threshold`
    ///   次触发熔断，`circuit_break_cooldown` 内的请求直接短路
    /// - 任意成功 → 重置 circuit breaker
    pub async fn fetch<T: DeserializeOwned>(
        &self,
        category: &str,
        params: Value,
    ) -> Result<OpenStockResponse<T>> {
        // 1. Circuit breaker check (read-only)
        {
            let guard = self
                .circuit
                .lock()
                .map_err(|e| QuantixError::Other(format!("circuit mutex poisoned: {}", e)))?;
            if let Some(until) = guard.tripped_until
                && Instant::now() < until
            {
                return Err(QuantixError::Other(format!(
                    "openstock circuit breaker open until {:?} (category={})",
                    until, category
                )));
            }
        }

        let endpoint = self
            .base_url
            .join("data/fetch")
            .map_err(|e| QuantixError::Other(format!("url join failed: {}", e)))?;
        let body = serde_json::json!({
            "data_category": category,
            "params": params,
        });

        let mut last_err: Option<QuantixError> = None;
        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let delay = self.config.retry_base_delay * 2u32.pow(attempt - 1);
                tracing::warn!(
                    attempt,
                    max_retries = self.config.max_retries,
                    category,
                    delay_ms = delay.as_millis(),
                    "openstock retry"
                );
                tokio::time::sleep(delay).await;
            }

            let send_result = self
                .http
                .post(endpoint.clone())
                .header("X-API-Key", self.api_key.clone())
                .json(&body)
                .send()
                .await;

            match send_result {
                Err(e) => {
                    last_err = Some(QuantixError::Network(format!(
                        "openstock request failed: {}",
                        e
                    )));
                    continue; // retry
                }
                Ok(resp) => {
                    let status = resp.status();
                    let raw = resp.text().await.map_err(|e| {
                        QuantixError::Network(format!("openstock body read failed: {}", e))
                    })?;

                    if !status.is_success() {
                        let summary = match serde_json::from_str::<OpenStockErrorEnvelope>(&raw) {
                            Ok(env) => env.to_summary(),
                            Err(_) => format!(
                                "openstock: HTTP {} | body: {}",
                                status,
                                raw.chars().take(200).collect::<String>()
                            ),
                        };
                        if status.is_server_error() {
                            last_err = Some(QuantixError::Other(summary));
                            continue; // retry 5xx
                        }
                        // 4xx: fail-fast, don't retry, don't trip circuit (client bug)
                        return Err(QuantixError::Other(summary));
                    }

                    // 2xx — try parse
                    match serde_json::from_str::<OpenStockEnvelope<T>>(&raw) {
                        Ok(env) => {
                            self.reset_circuit()?;
                            return Ok(OpenStockResponse::from_envelope(env, &raw));
                        }
                        Err(e) => {
                            // fail-fast, do NOT retry (corrupted response)
                            return Err(QuantixError::Other(format!(
                                "openstock: cannot parse success envelope: {} | body: {}",
                                e,
                                raw.chars().take(200).collect::<String>()
                            )));
                        }
                    }
                }
            }
        }

        // All retries exhausted — record circuit failure
        self.record_circuit_failure(category)?;
        Err(last_err
            .unwrap_or_else(|| QuantixError::Other("openstock retry exhausted".to_string())))
    }

    fn reset_circuit(&self) -> Result<()> {
        let mut guard = self
            .circuit
            .lock()
            .map_err(|e| QuantixError::Other(format!("circuit mutex poisoned: {}", e)))?;
        guard.consecutive_failures = 0;
        guard.tripped_until = None;
        Ok(())
    }

    fn record_circuit_failure(&self, category: &str) -> Result<()> {
        if self.config.circuit_break_threshold == 0 {
            return Ok(()); // breaker disabled
        }
        let mut guard = self
            .circuit
            .lock()
            .map_err(|e| QuantixError::Other(format!("circuit mutex poisoned: {}", e)))?;
        guard.consecutive_failures += 1;
        if guard.consecutive_failures >= self.config.circuit_break_threshold {
            let cooldown = self.config.circuit_break_cooldown;
            guard.tripped_until = Some(Instant::now() + cooldown);
            tracing::error!(
                consecutive_failures = guard.consecutive_failures,
                threshold = self.config.circuit_break_threshold,
                cooldown_ms = cooldown.as_millis(),
                category,
                "openstock circuit breaker tripped"
            );
        }
        Ok(())
    }

    /// Convenience: fetch `STOCK_CODES`.
    pub async fn fetch_stock_codes(
        &self,
    ) -> Result<OpenStockResponse<crate::sources::openstock_codes::StockCodeRecord>> {
        self.fetch("STOCK_CODES", serde_json::json!({})).await
    }

    /// Convenience: fetch `TRADE_DATES` for an optional date range.
    ///
    /// Runtime contract (`baostock._fetch_trade_dates`): accepts
    /// `start_date` / `end_date` as `YYYY-MM-DD` strings. When both are
    /// `None`, baostock returns the full history (which the runtime
    /// truncates). The legacy `year` parameter is **ignored** by the
    /// runtime — callers should pass `start`/`end` instead.
    pub async fn fetch_trade_dates(
        &self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<OpenStockResponse<crate::sources::openstock_calendar::TradeDateRecord>> {
        let mut params = serde_json::json!({});
        if let Some(start) = start {
            params["start_date"] = Value::String(start.to_string());
        }
        if let Some(end) = end {
            params["end_date"] = Value::String(end.to_string());
        }
        self.fetch("TRADE_DATES", params).await
    }

    /// Convenience: fetch `INDEX_KLINES` for a symbol with optional date range.
    ///
    /// Runtime contract (`baostock._fetch_index_klines`): accepts
    /// `start_date` / `end_date` as `YYYY-MM-DD` strings — same as
    /// [`fetch_trade_dates`](Self::fetch_trade_dates). The legacy
    /// `start` / `end` parameter names are ignored by the runtime
    /// (verified 2026-07-01 against live `http://192.168.123.104:8040`).
    pub async fn fetch_index_klines(
        &self,
        code: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<OpenStockResponse<crate::sources::openstock_index::IndexKlineRecord>> {
        let mut params = serde_json::json!({ "code": code });
        if let Some(start) = start {
            params["start_date"] = Value::String(start.to_string());
        }
        if let Some(end) = end {
            params["end_date"] = Value::String(end.to_string());
        }
        self.fetch("INDEX_KLINES", params).await
    }

    /// Convenience: fetch `HISTORICAL_KLINES` for an A-share stock code
    /// with optional date range.
    ///
    /// Runtime contract: `HISTORICAL_KLINES` is documented in
    /// `DATA_CAPABILITY_SCOPE.md` §"Long-history A-share K-lines" but
    /// has not yet been live-verified from quantix-rust (P0.11a defers
    /// the live smoke to fixture-driven parser tests; field shape may
    /// differ from `INDEX_KLINES`). Parameters mirror
    /// [`fetch_index_klines`](Self::fetch_index_klines): `code` plus
    /// optional `start_date` / `end_date` as `YYYY-MM-DD` strings.
    /// Record type reuses `IndexKlineRecord` since its fields are all
    /// `Option<serde_json::Value>`, absorbing shape drift via the parser
    /// layer (per P0.11 design.md D4).
    pub async fn fetch_historical_klines(
        &self,
        code: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<OpenStockResponse<crate::sources::openstock_index::IndexKlineRecord>> {
        let mut params = serde_json::json!({ "code": code });
        if let Some(start) = start {
            params["start_date"] = Value::String(start.to_string());
        }
        if let Some(end) = end {
            params["end_date"] = Value::String(end.to_string());
        }
        self.fetch("HISTORICAL_KLINES", params).await
    }

    /// Convenience: fetch `ALL_STOCKS` (baostock full-market snapshot).
    /// `day` is optional (`YYYY-MM-DD`); when omitted, the server falls
    /// back to the most recent trading day and reports it via
    /// `quality_flags.fallback_day`.
    pub async fn fetch_all_stocks(
        &self,
        day: Option<&str>,
    ) -> Result<OpenStockResponse<crate::sources::openstock_codes::StockListRecord>> {
        let mut params = serde_json::json!({});
        if let Some(day) = day {
            params["day"] = Value::String(day.to_string());
        }
        self.fetch("ALL_STOCKS", params).await
    }

    /// Convenience: fetch `WORKDAYS` (eltdx action-driven calendar).
    /// `action` is one of `today` / `today_is_workday` / `is_workday` /
    /// `range` / `next_workday` / `previous_workday`. `date` is required
    /// for `is_workday`/`next_workday`/`previous_workday`; `start`+`end`
    /// are required for `range`. Other actions ignore the date params.
    pub async fn fetch_workdays(
        &self,
        action: &str,
        date: Option<&str>,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<OpenStockResponse<crate::sources::openstock_calendar::WorkdayRecord>> {
        let mut params = serde_json::json!({ "action": action });
        if let Some(date) = date {
            params["date"] = Value::String(date.to_string());
        }
        if let Some(start) = start {
            params["start"] = Value::String(start.to_string());
        }
        if let Some(end) = end {
            params["end"] = Value::String(end.to_string());
        }
        self.fetch("WORKDAYS", params).await
    }
}

/// Public post-parse view of a `/data/fetch` success response.
/// Flattened to `(records, source, artifact_hash, received_at, latency_ms)`.
#[derive(Debug, Clone)]
pub struct OpenStockResponse<T> {
    pub records: Vec<T>,
    pub source: String,
    pub artifact_hash: String,
    pub received_at: Option<String>,
    pub latency_ms: Option<f64>,
}

impl<T> OpenStockResponse<T> {
    /// Compose the public view from a raw envelope and the raw response
    /// body string. The `artifact_hash` is SHA-256 of the raw body
    /// (computed via the canonical `openstock_shadow::artifact_hash`).
    pub fn from_envelope(env: OpenStockEnvelope<T>, raw_body: &str) -> Self {
        Self {
            records: env.data,
            source: env.source.unwrap_or_default(),
            artifact_hash: artifact_hash(raw_body),
            received_at: env.received_at,
            latency_ms: env.latency_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(Debug, Clone, serde::Deserialize, PartialEq)]
    struct Rec {
        code: String,
    }

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

    fn fast_test_cfg(base_url: String) -> OpenStockClientConfig {
        OpenStockClientConfig {
            base_url,
            api_key: "test-key".to_string(),
            timeout: Duration::from_secs(1),
            max_retries: 2,
            retry_base_delay: Duration::from_millis(5),
            circuit_break_threshold: 5,
            circuit_break_cooldown: Duration::from_millis(50),
        }
    }

    fn success_body() -> &'static str {
        r#"{"data":[{"code":"600000"}],"source":"eltdx"}"#
    }

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
}
