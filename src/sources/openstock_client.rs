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

mod klines;
mod minute;
mod reference;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests_core;
#[cfg(test)]
mod tests_klines;
#[cfg(test)]
mod tests_minute_klines;
#[cfg(test)]
mod tests_minute_share;
#[cfg(test)]
mod tests_minute_stream;
#[cfg(test)]
mod tests_reference;
