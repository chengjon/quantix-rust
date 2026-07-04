//! Generic `OpenStockClient` skeleton — reqwest-based, fixture-tested.
//!
//! Knows the uniform `/data/fetch` envelope shape and `X-API-Key`
//! auth. No live HTTP in tests; fixture-only tests exercise
//! [`OpenStockResponse::from_envelope`] and the shared deserialization
//! paths.

use std::str::FromStr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use futures::stream::{self};
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

    /// Convenience: fetch `TICK_DATA` for a stock symbol on an optional
    /// trading date. Live-verified 2026-07-01 (see design.md D4.1).
    ///
    /// **Critical**: the runtime parameter is `symbol` (NOT `code`).
    /// Sending `code` returns HTTP 422 `"symbol is required for
    /// TICK_DATA"`. This differs from `fetch_index_klines` /
    /// `fetch_historical_klines`, which use `code`. The eltdx adapter
    /// requires the `symbol` name.
    ///
    /// `date` is `YYYYMMDD` (numeric, no dashes) per the eltdx adapter
    /// contract. When omitted, the runtime defaults to the latest
    /// trading day.
    ///
    /// Response shape: `data: [{ meta, ticks: [...] }]` — a one-element
    /// array wrapping a `{meta, ticks}` envelope-record (NOT a flat
    /// tick list). Use [`parse_tick_data`](crate::sources::openstock_ticks::parse_tick_data)
    /// to flatten.
    pub async fn fetch_tick_data(
        &self,
        symbol: &str,
        date: Option<&str>,
    ) -> Result<OpenStockResponse<crate::sources::openstock_ticks::TickEnvelopeRecord>> {
        let mut params = serde_json::json!({ "symbol": symbol });
        if let Some(date) = date {
            params["date"] = Value::String(date.to_string());
        }
        self.fetch("TICK_DATA", params).await
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

    /// Fetch daily OHLCV bars from OpenStock `/data/bars` endpoint.
    ///
    /// This is the preferred path for backtest/analysis K-line data.
    /// `/data/bars` natively supports symbol, period, date range and
    /// adjust type, and internally negotiates KLINES / ADJUSTED_KLINES
    /// / HISTORICAL_KLINES based on the `adjust` param.
    ///
    /// Returns `Vec<Kline>` on success. When the upstream provider is
    /// unavailable or returns empty data, returns an empty vec (caller
    /// should fall back to ClickHouse or another source).
    pub async fn fetch_daily_klines(
        &self,
        code: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Vec<crate::data::models::Kline>> {
        let endpoint = self
            .base_url
            .join("data/bars")
            .map_err(|e| QuantixError::Other(format!("url join failed: {}", e)))?;

        let mut body = serde_json::json!({
            "symbol": code,
            "period": "day",
        });
        if let Some(start) = start {
            body["start_date"] = Value::String(start.to_string());
        }
        if let Some(end) = end {
            body["end_date"] = Value::String(end.to_string());
        }

        let resp = self
            .http
            .post(endpoint)
            .header("X-API-Key", self.api_key.clone())
            .json(&body)
            .send()
            .await
            .map_err(|e| QuantixError::Network(format!("/data/bars request failed: {}", e)))?;

        let status = resp.status();
        let raw = resp
            .text()
            .await
            .map_err(|e| QuantixError::Network(format!("/data/bars body read failed: {}", e)))?;

        if !status.is_success() {
            return Err(QuantixError::Other(format!(
                "/data/bars returned {}: {}",
                status,
                raw.chars().take(200).collect::<String>()
            )));
        }

        #[derive(serde::Deserialize)]
        struct BarsResponse {
            data: Vec<BarRecord>,
        }

        #[derive(serde::Deserialize)]
        struct BarRecord {
            time: String,
            open: f64,
            high: f64,
            low: f64,
            close: f64,
            volume: f64,
            amount: f64,
        }

        let bars: BarsResponse = serde_json::from_str(&raw)
            .map_err(|e| QuantixError::Other(format!("/data/bars parse failed: {}", e)))?;

        let mut klines = Vec::with_capacity(bars.data.len());
        for bar in bars.data {
            // time format: "2026-06-01T15:00:00+08:00" → NaiveDate
            let date = chrono::NaiveDate::parse_from_str(&bar.time[..10], "%Y-%m-%d")
                .map_err(|e| QuantixError::DataParse(format!("解析 bars 日期失败: {}", e)))?;

            klines.push(crate::data::models::Kline {
                code: code.to_string(),
                date,
                open: rust_decimal::Decimal::from_str(&format!("{}", bar.open)).unwrap_or_default(),
                high: rust_decimal::Decimal::from_str(&format!("{}", bar.high)).unwrap_or_default(),
                low: rust_decimal::Decimal::from_str(&format!("{}", bar.low)).unwrap_or_default(),
                close: rust_decimal::Decimal::from_str(&format!("{}", bar.close))
                    .unwrap_or_default(),
                volume: bar.volume as i64,
                amount: Some(
                    rust_decimal::Decimal::from_str(&format!("{}", bar.amount)).unwrap_or_default(),
                ),
                adjust_type: crate::data::models::AdjustType::None,
            });
        }

        Ok(klines)
    }

    /// Fetch K-line bars from OpenStock `/data/bars` endpoint with explicit
    /// period (day/week/month) and adjust (none/qfq/hfq) parameters.
    ///
    /// This is the multi-period generalisation of [`Self::fetch_daily_klines`].
    /// Like that method, it uses a direct `reqwest` call (no retry, no circuit
    /// breaker) per the P0.10 design decision for `/data/bars`. The returned
    /// `Kline` records are stamped with the requested `adjust_type` (the
    /// runtime does not echo it back — decision D2 request-driven).
    ///
    /// `AdjustType::None` causes the `adjust` field to be omitted entirely
    /// from the request body (matches `fetch_daily_klines` wire shape).
    pub async fn fetch_klines(
        &self,
        code: &str,
        period: crate::data::models::BarPeriod,
        adjust: crate::data::models::AdjustType,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Vec<crate::data::models::Kline>> {
        use std::str::FromStr;

        let endpoint = self
            .base_url
            .join("data/bars")
            .map_err(|e| QuantixError::Other(format!("url join failed: {}", e)))?;

        let mut body = serde_json::json!({
            "symbol": code,
            "period": period.as_str(),
        });
        if let Some(adj) = adjust.as_openstock_param() {
            body["adjust"] = Value::String(adj.to_string());
        }
        if let Some(start) = start {
            body["start_date"] = Value::String(start.to_string());
        }
        if let Some(end) = end {
            body["end_date"] = Value::String(end.to_string());
        }

        let resp = self
            .http
            .post(endpoint)
            .header("X-API-Key", self.api_key.clone())
            .json(&body)
            .send()
            .await
            .map_err(|e| QuantixError::Network(format!("/data/bars request failed: {}", e)))?;

        let status = resp.status();
        let raw = resp
            .text()
            .await
            .map_err(|e| QuantixError::Network(format!("/data/bars body read failed: {}", e)))?;

        if !status.is_success() {
            return Err(QuantixError::Other(format!(
                "/data/bars returned {}: {}",
                status,
                raw.chars().take(200).collect::<String>()
            )));
        }

        #[derive(serde::Deserialize)]
        struct BarsResponse {
            data: Vec<BarRecord>,
        }

        #[derive(serde::Deserialize)]
        struct BarRecord {
            time: String,
            open: f64,
            high: f64,
            low: f64,
            close: f64,
            volume: f64,
            amount: f64,
        }

        let bars: BarsResponse = serde_json::from_str(&raw)
            .map_err(|e| QuantixError::Other(format!("/data/bars parse failed: {}", e)))?;

        let mut klines = Vec::with_capacity(bars.data.len());
        for bar in bars.data {
            // time format: "2026-06-01T15:00:00+08:00" → NaiveDate
            let date = chrono::NaiveDate::parse_from_str(&bar.time[..10], "%Y-%m-%d")
                .map_err(|e| QuantixError::DataParse(format!("解析 bars 日期失败: {}", e)))?;

            klines.push(crate::data::models::Kline {
                code: code.to_string(),
                date,
                open: rust_decimal::Decimal::from_str(&format!("{}", bar.open)).unwrap_or_default(),
                high: rust_decimal::Decimal::from_str(&format!("{}", bar.high)).unwrap_or_default(),
                low: rust_decimal::Decimal::from_str(&format!("{}", bar.low)).unwrap_or_default(),
                close: rust_decimal::Decimal::from_str(&format!("{}", bar.close))
                    .unwrap_or_default(),
                volume: bar.volume as i64,
                amount: Some(
                    rust_decimal::Decimal::from_str(&format!("{}", bar.amount)).unwrap_or_default(),
                ),
                adjust_type: adjust,
            });
        }

        Ok(klines)
    }

    /// Fetches minute-level OHLCV candles from `/data/bars` with `period`
    /// in `1m|5m|15m|30m|60m`. Mirrors `fetch_klines` shape (direct reqwest,
    /// no envelope, no retry, no circuit breaker). Returns `Vec<MinuteBar>`
    /// with `NaiveDateTime` timestamps (minute precision preserved from
    /// the wire ISO string).
    ///
    /// The returned `MinuteBar` records are stamped with the requested
    /// `adjust_type` (the runtime does not echo it back — decision D2
    /// request-driven, matching `fetch_klines`).
    ///
    /// `AdjustType::None` causes the `adjust` field to be omitted entirely
    /// from the request body (matches `fetch_klines` wire shape).
    ///
    /// `date_or_range` selects single-day (`Date` — wire body identical to
    /// P0.13b-1, INV-2A) or inclusive multi-day range (`Range` — wire body
    /// carries `start_date`/`end_date`, no `date` field, spec §3.2/§6 D4).
    pub async fn fetch_minute_klines(
        &self,
        code: &str,
        period: crate::data::models::MinutePeriod,
        date_or_range: crate::data::models::DateOrRange,
        adjust: crate::data::models::AdjustType,
    ) -> Result<Vec<crate::data::models::MinuteBar>> {
        use crate::data::models::DateOrRange;
        let (start, end) = match date_or_range {
            DateOrRange::Date(d) => (d, d),
            DateOrRange::Range { start, end } => (start, end),
        };
        self.fetch_minute_klines_range(code, period, start, end, adjust)
            .await
    }

    /// 流式拉取分钟 K 线（P0.13d D1/D2）。
    ///
    /// 把 `date_or_range` 解析为 `(start, end)`，调用 `chunk_range_weekly`
    /// 切成连续 ≤7 天段，每段一次 `fetch_minute_klines_range` 调用，yield
    /// 一个 `Vec<MinuteBar>`。
    ///
    /// - `Date(d)`：单段 `(d, d)`，一个 batch
    /// - `Range { start, end }`：从 start 起每 7 天一段；尾段可能短
    /// - 错误：首个 batch 失败即 yield `Err`，后续 `next()` 返回 `None`（D4）
    /// - 不经过 retry/circuit breaker（D5；与 batch klines 路径一致）
    /// - Wire shape 由 `fetch_minute_klines_range` 保证（INV-2A）
    pub fn fetch_minute_klines_stream<'a>(
        &'a self,
        code: &'a str,
        period: crate::data::models::MinutePeriod,
        date_or_range: crate::data::models::DateOrRange,
        adjust: crate::data::models::AdjustType,
    ) -> impl futures::Stream<
        Item = Result<Vec<crate::data::models::MinuteBar>>,
    > + 'a {
        use crate::data::models::DateOrRange;
        let (start, end) = match date_or_range {
            DateOrRange::Date(d) => (d, d),
            DateOrRange::Range { start, end } => (start, end),
        };
        let chunks = crate::data::models::chunk_range_weekly(start, end);
        // INV-5A: first Err yields, then stream terminates (next() returns None).
        // `unfold` ensures the next chunk is fetched only when the consumer asks
        // for the next item AND no prior chunk has errored. State carries the
        // iterator + errored flag so the closure body is FnMut-compatible.
        let state = (chunks.into_iter(), false);
        stream::unfold(state, move |(mut iter, errored)| async move {
            if errored {
                return None;
            }
            let (s, e) = iter.next()?;
            let res = self.fetch_minute_klines_range(code, period, s, e, adjust).await;
            let next_errored = res.is_err();
            Some((res, (iter, next_errored)))
        })
    }

    /// Private helper: fetch minute klines for an inclusive `[start..=end]` sub-range.
    ///
    /// Wire body field selection (INV-2A, preserving P0.13b-1 / P0.13c):
    ///   - `start == end` → body has only `date` (identical to P0.13b-1 single-day wire)
    ///   - `start != end` → body has only `start_date` + `end_date`
    ///
    /// This helper is shared by:
    ///   - `fetch_minute_klines` (batch API; dispatcher for `DateOrRange`)
    ///   - `fetch_minute_klines_stream` (streaming API; one call per weekly chunk)
    async fn fetch_minute_klines_range(
        &self,
        code: &str,
        period: crate::data::models::MinutePeriod,
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
        adjust: crate::data::models::AdjustType,
    ) -> Result<Vec<crate::data::models::MinuteBar>> {
        use std::str::FromStr;

        let endpoint = self
            .base_url
            .join("data/bars")
            .map_err(|e| QuantixError::Other(format!("url join failed: {}", e)))?;

        let mut body = serde_json::json!({
            "symbol": code,
            "period": period.as_str(),
        });
        if let Some(adj) = adjust.as_openstock_param() {
            body["adjust"] = serde_json::Value::String(adj.to_string());
        }
        if start == end {
            body["date"] = serde_json::Value::String(start.format("%Y-%m-%d").to_string());
        } else {
            body["start_date"] = serde_json::Value::String(start.format("%Y-%m-%d").to_string());
            body["end_date"] = serde_json::Value::String(end.format("%Y-%m-%d").to_string());
        }

        let resp = self
            .http
            .post(endpoint)
            .header("X-API-Key", self.api_key.clone())
            .json(&body)
            .send()
            .await
            .map_err(|e| QuantixError::Network(format!("/data/bars request failed: {}", e)))?;

        let status = resp.status();
        let raw = resp
            .text()
            .await
            .map_err(|e| QuantixError::Network(format!("/data/bars body read failed: {}", e)))?;

        if !status.is_success() {
            return Err(QuantixError::Other(format!(
                "/data/bars returned {}: {}",
                status,
                raw.chars().take(200).collect::<String>()
            )));
        }

        #[derive(serde::Deserialize)]
        struct BarsResponse {
            data: Vec<MinuteBarRecord>,
        }

        #[derive(serde::Deserialize)]
        struct MinuteBarRecord {
            time: String,
            open: f64,
            high: f64,
            low: f64,
            close: f64,
            volume: f64,
            amount: f64,
        }

        let bars: BarsResponse = serde_json::from_str(&raw)
            .map_err(|e| QuantixError::Other(format!("/data/bars parse failed: {}", e)))?;

        let mut out = Vec::with_capacity(bars.data.len());
        for bar in bars.data {
            // Wire time format: "2026-07-02T09:31:00+08:00" → take first 19 chars
            // "2026-07-02T09:31:00" → parse as NaiveDateTime (no timezone).
            let ts = chrono::NaiveDateTime::parse_from_str(&bar.time[..19], "%Y-%m-%dT%H:%M:%S")
                .map_err(|e| {
                    QuantixError::DataParse(format!("解析 minute bars 时间戳失败: {}", e))
                })?;

            out.push(crate::data::models::MinuteBar {
                code: code.to_string(),
                timestamp: ts,
                open: rust_decimal::Decimal::from_str(&format!("{}", bar.open)).unwrap_or_default(),
                high: rust_decimal::Decimal::from_str(&format!("{}", bar.high)).unwrap_or_default(),
                low: rust_decimal::Decimal::from_str(&format!("{}", bar.low)).unwrap_or_default(),
                close: rust_decimal::Decimal::from_str(&format!("{}", bar.close))
                    .unwrap_or_default(),
                volume: bar.volume as i64,
                amount: Some(
                    rust_decimal::Decimal::from_str(&format!("{}", bar.amount)).unwrap_or_default(),
                ),
                adjust_type: adjust,
            });
        }

        Ok(out)
    }

    /// 消费 MINUTE_DATA category（分时点序列 / 分时图 ticks）。
    ///
    /// 走 `/data/fetch` envelope 路径，复用 retry + circuit breaker
    /// （与 `fetch_stock_codes` / `fetch_trade_dates` 同路径）。
    ///
    /// **调用签名**（对齐 `fetch_stock_codes`）：`fetch<T>()` 接收
    /// `(category: &str, params: Value)` 双参数，内部拼装为
    /// `{data_category, params}` envelope。
    ///
    /// **category 无 period/adjust 维度** — params 仅 `{code, date}`。
    ///
    /// 解析：response.records 是 8 字段的数组，parse_minute_share 裁剪到
    /// 5 业务字段。单条记录关键字段缺失 → warn + skip（INV-2C）。
    pub async fn fetch_minute_share(
        &self,
        code: &str,
        date_or_range: crate::data::models::DateOrRange,
    ) -> Result<Vec<crate::data::models::MinuteShare>> {
        use crate::data::models::{DateOrRange, iter_dates_inclusive};

        match date_or_range {
            DateOrRange::Date(d) => self.fetch_minute_share_single(code, d).await,
            DateOrRange::Range { start, end } => {
                let mut all = Vec::new();
                for d in iter_dates_inclusive(start, end) {
                    let day_records = self.fetch_minute_share_single(code, d).await?;
                    all.extend(day_records);
                }
                Ok(all)
            }
        }
    }

    /// 流式拉取分时点序列（P0.13d D1/D3）。
    ///
    /// 每个自然日（含非交易日）yield 一个 `Vec<MinuteShare>`；非交易日 yield
    /// 空 Vec（D3；batch count == 日历天数，调用方可做完整性检查）。
    ///
    /// - 复用 P0.13c `fetch_minute_share_single`（带 retry + breaker）
    /// - 错误：首个 batch 失败即 yield `Err`，后续 `next()` 返回 `None`（D4）
    pub fn fetch_minute_share_stream<'a>(
        &'a self,
        code: &'a str,
        date_or_range: crate::data::models::DateOrRange,
    ) -> impl futures::Stream<
        Item = Result<Vec<crate::data::models::MinuteShare>>,
    > + 'a {
        use crate::data::models::{iter_dates_inclusive, DateOrRange};
        let (start, end) = match date_or_range {
            DateOrRange::Date(d) => (d, d),
            DateOrRange::Range { start, end } => (start, end),
        };
        let days: Vec<chrono::NaiveDate> = iter_dates_inclusive(start, end).collect();
        // INV-5A: first Err yields, then stream terminates (next() returns None).
        let state = (days.into_iter(), false);
        stream::unfold(state, move |(mut iter, errored)| async move {
            if errored {
                return None;
            }
            let d = iter.next()?;
            let res = self.fetch_minute_share_single(code, d).await;
            let next_errored = res.is_err();
            Some((res, (iter, next_errored)))
        })
    }

    /// Single-day helper for MINUTE_DATA fetches.
    ///
    /// Both `Date` and `Range` paths route here. The server wraps each result
    /// in a `{meta: {trading_date}, points: [...]}` envelope; we extract
    /// `trading_date` from the envelope (NOT from the request parameter) so
    /// that records are stamped with the actual trading day the server
    /// reported (INV-2C).
    async fn fetch_minute_share_single(
        &self,
        code: &str,
        date: chrono::NaiveDate,
    ) -> Result<Vec<crate::data::models::MinuteShare>> {
        let params = serde_json::json!({
            "code": code,
            "date": date.format("%Y-%m-%d").to_string(),
        });
        let resp = self
            .fetch::<MinuteShareEnvelope>("MINUTE_DATA", params)
            .await?;
        let mut out = Vec::new();
        for env in resp.records {
            let actual_date = env
                .meta
                .trading_date
                .as_deref()
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .unwrap_or(date);
            for raw in env.points {
                if let Some(share) = parse_minute_share(code, &raw, actual_date) {
                    out.push(share);
                } else {
                    tracing::warn!(
                        code = code,
                        requested_date = %date,
                        trading_date = %actual_date,
                        time_minutes = %raw.time_minutes,
                        "MINUTE_DATA record missing required field or invalid time, skipping"
                    );
                }
            }
        }
        Ok(out)
    }
}

/// MINUTE_DATA 原始记录（8 字段，未裁剪）。
///
/// OpenStock envelope `records` 数组元素的反序列化目标。
/// 字段名对应 eLtdx MINUTE_DATA 输出：
///   - time_minutes: "0930" 或 "09:30" 格式
///   - price/volume/amount/avg_price: 业务字段（保留）
///   - index/time/price_milli: 冗余字段（serde default 容忍缺失）
///
/// **数值字段直接用 `Decimal`**（rust_decimal + serde 自动反序列化 JSON number）。
/// 若 live 测试发现字符串格式数值漂移，切换到 `serde_json::Value` + parse_decimal。
#[derive(Debug, serde::Deserialize)]
struct RawMinuteRecord {
    time_minutes: String,
    price: Option<rust_decimal::Decimal>,
    volume: Option<i64>,
    amount: Option<rust_decimal::Decimal>,
    avg_price: Option<rust_decimal::Decimal>,
}

/// OpenStock MINUTE_DATA envelope wrapper.
///
/// Each element of the `/data/fetch MINUTE_DATA` response `data` array is a
/// `{meta, points}` wrapper (not a flat record). `trading_date` lives in
/// `meta`, not inside each point — see spec §3.2 R1 evidence and
/// `_eltdx_timeseries.py:181-208`.
#[derive(Debug, serde::Deserialize)]
struct MinuteShareEnvelope {
    #[serde(default)]
    meta: MinuteShareMeta,
    #[serde(default)]
    points: Vec<RawMinuteRecord>,
}

/// Meta block of the MINUTE_DATA envelope.
#[derive(Debug, Default, serde::Deserialize)]
struct MinuteShareMeta {
    /// Server-reported trading day for the records in this envelope
    /// (`YYYY-MM-DD`). Per INV-2C, per-record timestamps are stamped from
    /// this value, not from the client-supplied request date.
    #[serde(default)]
    trading_date: Option<String>,
}

/// 解析 MINUTE_DATA 单条记录为 `MinuteShare`。
///
/// 丢弃字段：`index`（内部序号）、`time`（ISO 冗余）、`price_milli`（毫表示）。
/// 保留字段：`time_minutes, price, volume, amount, avg_price`。
///
/// 返回 `Option<MinuteShare>`：当 4 个关键字段（price/volume/amount/avg_price）
/// 任一为 None，或 `time_minutes` 解析失败时返回 None，调用方 warn + skip（INV-2C）。
fn parse_minute_share(
    code: &str,
    raw: &RawMinuteRecord,
    date: chrono::NaiveDate,
) -> Option<crate::data::models::MinuteShare> {
    let price = raw.price?;
    let volume = raw.volume?;
    let amount = raw.amount?;
    let avg_price = raw.avg_price?;
    let (hh, mm) = parse_time_minutes(&raw.time_minutes)?;
    let timestamp = date.and_hms_opt(hh, mm, 0)?;
    Some(crate::data::models::MinuteShare {
        code: code.to_string(),
        timestamp,
        price: Some(price),
        volume: Some(volume),
        amount: Some(amount),
        avg_price: Some(avg_price),
    })
}

/// 解析 `time_minutes` 字段为 (HH, MM)。
///
/// 接受两种格式（D4 双格式容错，防御 R2 格式歧义）：
///   - "0930"     → (9, 30)
///   - "09:30"    → (9, 30)
///
/// 长度不匹配或字符非数字 → None（触发 INV-2C skip）。
fn parse_time_minutes(s: &str) -> Option<(u32, u32)> {
    let cleaned: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.len() != 4 {
        return None;
    }
    let hh: u32 = cleaned[..2].parse().ok()?;
    let mm: u32 = cleaned[2..].parse().ok()?;
    if hh >= 24 || mm >= 60 {
        return None;
    }
    Some((hh, mm))
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

    // -----------------------------------------------------------------
    // fetch_klines tests (wiremock-based, P0.13a Task 1.2)
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn fetch_klines_day_none_sends_period_day_and_omits_adjust() {
        use crate::data::models::{AdjustType, BarPeriod};
        use wiremock::matchers::{body_partial_json, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        let body = serde_json::json!({
            "data": [
                {
                    "time": "2026-06-01T15:00:00+08:00",
                    "open": 10.5,
                    "high": 11.0,
                    "low": 10.2,
                    "close": 10.8,
                    "volume": 1000000.0,
                    "amount": 10800000.0,
                }
            ]
        });
        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .and(body_partial_json(serde_json::json!({
                "symbol": "600000",
                "period": "day",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let klines = client
            .fetch_klines("600000", BarPeriod::Day, AdjustType::None, None, None)
            .await
            .expect("fetch_klines ok");
        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].code, "600000");
        assert_eq!(klines[0].adjust_type, AdjustType::None);
    }

    #[tokio::test]
    async fn fetch_klines_qfq_sends_adjust_qfq_and_stamps_records() {
        use crate::data::models::{AdjustType, BarPeriod};
        use wiremock::matchers::{body_partial_json, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        let body = serde_json::json!({
            "data": [
                {
                    "time": "2026-06-02T15:00:00+08:00",
                    "open": 5.0,
                    "high": 5.5,
                    "low": 4.9,
                    "close": 5.2,
                    "volume": 2000000.0,
                    "amount": 10400000.0,
                }
            ]
        });
        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .and(body_partial_json(serde_json::json!({
                "symbol": "000001",
                "period": "week",
                "adjust": "qfq",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let klines = client
            .fetch_klines(
                "000001",
                BarPeriod::Week,
                AdjustType::QFQ,
                Some("2026-01-01"),
                Some("2026-06-30"),
            )
            .await
            .expect("fetch_klines ok");
        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].code, "000001");
        assert_eq!(klines[0].adjust_type, AdjustType::QFQ);
    }

    #[tokio::test]
    async fn fetch_klines_propagates_4xx() {
        use crate::data::models::{AdjustType, BarPeriod};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_string(r#"{"code":"bad_request","message":"nope"}"#),
            )
            .expect(1) // no retry on 4xx — matches fetch_daily_klines
            .mount(&server)
            .await;

        let err = client
            .fetch_klines("600000", BarPeriod::Month, AdjustType::HFQ, None, None)
            .await
            .expect_err("should fail");
        let msg = format!("{:?}", err);
        assert!(msg.contains("/data/bars returned 400"), "msg={}", msg);
    }

    // -----------------------------------------------------------------
    // fetch_minute_klines tests (wiremock-based, P0.13b-1 Task 2)
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn fetch_minute_klines_1m_none_sends_period_1m_and_date() {
        use crate::data::models::{AdjustType, MinutePeriod};
        use wiremock::matchers::{body_partial_json, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        let body = serde_json::json!({
            "data": [
                {"time": "2026-07-02T09:31:00+08:00", "open": 10.0, "high": 10.5, "low": 9.9, "close": 10.2, "volume": 1000.0, "amount": 10200.0},
                {"time": "2026-07-02T09:32:00+08:00", "open": 10.2, "high": 10.4, "low": 10.1, "close": 10.3, "volume": 800.0, "amount": 8240.0},
            ]
        });
        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .and(body_partial_json(serde_json::json!({
                "symbol": "sh600000",
                "period": "1m",
                "date": "2026-07-02"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .expect(1)
            .mount(&server)
            .await;

        let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
        let bars = client
            .fetch_minute_klines(
                "sh600000",
                MinutePeriod::Minute1,
                crate::data::models::DateOrRange::Date(date),
                AdjustType::None,
            )
            .await
            .expect("fetch_minute_klines ok");

        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].code, "sh600000");
        assert_eq!(
            bars[0].timestamp,
            chrono::NaiveDateTime::parse_from_str("2026-07-02T09:31:00", "%Y-%m-%dT%H:%M:%S")
                .unwrap()
        );
        assert_eq!(bars[0].adjust_type, AdjustType::None);
        assert_eq!(bars[1].volume, 800);

        // W2: Date path wire body must NOT contain range fields (INV-2A backward compat).
        let received = server.received_requests().await.expect("at least one");
        let req_body: serde_json::Value =
            serde_json::from_slice(&received[0].body).expect("body is json");
        assert!(
            req_body.get("start_date").is_none(),
            "Date body must not include start_date, got: {:?}",
            req_body
        );
        assert!(
            req_body.get("end_date").is_none(),
            "Date body must not include end_date, got: {:?}",
            req_body
        );
    }

    #[tokio::test]
    async fn fetch_minute_klines_5m_qfq_sends_adjust_and_stamps_records() {
        use crate::data::models::{AdjustType, MinutePeriod};
        use wiremock::matchers::{body_partial_json, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .and(body_partial_json(serde_json::json!({
                "symbol": "sh600000",
                "period": "5m",
                "date": "2026-07-02",
                "adjust": "qfq"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"time": "2026-07-02T09:35:00+08:00", "open": 11.0, "high": 11.2, "low": 10.9, "close": 11.1, "volume": 500.0, "amount": 5550.0}
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
        let bars = client
            .fetch_minute_klines(
                "sh600000",
                MinutePeriod::Minute5,
                crate::data::models::DateOrRange::Date(date),
                AdjustType::QFQ,
            )
            .await
            .expect("fetch_minute_klines ok");

        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0].adjust_type, AdjustType::QFQ);
    }

    #[tokio::test]
    async fn fetch_minute_klines_propagates_4xx() {
        use crate::data::models::{AdjustType, MinutePeriod};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .respond_with(ResponseTemplate::new(400).set_body_string("bad period"))
            .expect(1) // no retry on 4xx — matches fetch_klines
            .mount(&server)
            .await;

        let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
        let result = client
            .fetch_minute_klines(
                "sh600000",
                MinutePeriod::Minute15,
                crate::data::models::DateOrRange::Date(date),
                AdjustType::None,
            )
            .await;

        let err = result.expect_err("expected error on 400");
        let msg = format!("{:?}", err);
        assert!(
            msg.contains("/data/bars returned 400"),
            "expected '/data/bars returned 400' in error, got: {}",
            msg
        );
    }

    #[tokio::test]
    async fn fetch_minute_klines_range_sends_start_date_end_date_body() {
        // W1: Range mode sends start_date + end_date (NOT date) — spec §3.2 row, §6 D4.
        use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let body = serde_json::json!({
            "data": [
                {"time": "2026-06-01T09:31:00+08:00", "open": 10.0, "high": 10.5, "low": 9.9, "close": 10.2, "volume": 1000.0, "amount": 10200.0},
                {"time": "2026-06-30T15:00:00+08:00", "open": 11.0, "high": 11.2, "low": 10.8, "close": 11.1, "volume": 500.0, "amount": 5550.0}
            ]
        });
        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .expect(1)
            .mount(&server)
            .await;

        let cfg = fast_test_cfg(server.uri());
        let client = OpenStockClient::new(cfg).expect("client build");
        let start = chrono::NaiveDate::parse_from_str("2026-06-01", "%Y-%m-%d").unwrap();
        let end = chrono::NaiveDate::parse_from_str("2026-06-30", "%Y-%m-%d").unwrap();
        let bars = client
            .fetch_minute_klines(
                "sh600000",
                MinutePeriod::Minute1,
                DateOrRange::Range { start, end },
                AdjustType::None,
            )
            .await
            .expect("fetch ok");

        assert_eq!(bars.len(), 2);

        let received = server.received_requests().await.expect("at least one");
        assert_eq!(received.len(), 1);
        let req_body: serde_json::Value =
            serde_json::from_slice(&received[0].body).expect("body is json");
        assert_eq!(req_body["start_date"], "2026-06-01");
        assert_eq!(req_body["end_date"], "2026-06-30");
        assert!(
            req_body.get("date").is_none(),
            "Range body must not include 'date', got: {:?}",
            req_body
        );
        assert_eq!(req_body["symbol"], "sh600000");
        assert_eq!(req_body["period"], "1m");
    }

    // -----------------------------------------------------------------
    // fetch_minute_share (P0.13b-2)
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn fetch_minute_share_sends_minute_data_category_and_date() {
        use rust_decimal_macros::dec;
        use wiremock::matchers::{body_partial_json, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/data/fetch"))
            .and(body_partial_json(serde_json::json!({
                "data_category": "MINUTE_DATA",
                "params": { "code": "sh600000", "date": "2026-07-01" }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "source": "eltdx",
                "artifact_hash": "abc123",
                "latency_ms": 42,
                "data": [
                    {
                        "meta": { "trading_date": "2026-07-01" },
                        "points": [
                            { "time_minutes": "09:30", "price": 10.50, "volume": 12300, "amount": 129150.0, "avg_price": 10.50, "index": 0, "time": "2026-07-01T09:30:00", "price_milli": 10500 },
                            { "time_minutes": "09:31", "price": 10.51, "volume": 8800, "amount": 92488.0, "avg_price": 10.505, "index": 1, "time": "2026-07-01T09:31:00", "price_milli": 10510 }
                        ]
                    }
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");
        let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let shares = client
            .fetch_minute_share("sh600000", crate::data::models::DateOrRange::Date(date))
            .await
            .expect("fetch ok");
        assert_eq!(shares.len(), 2);
        assert_eq!(shares[0].code, "sh600000");
        assert_eq!(shares[0].timestamp, date.and_hms_opt(9, 30, 0).unwrap());
        assert_eq!(shares[0].price, Some(dec!(10.50)));
        assert_eq!(shares[0].volume, Some(12300));
        assert_eq!(shares[1].timestamp, date.and_hms_opt(9, 31, 0).unwrap());
    }

    #[tokio::test]
    async fn fetch_minute_share_skips_records_with_missing_required_field() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/data/fetch"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "source": "eltdx",
                "data": [
                    {
                        "meta": { "trading_date": "2026-07-01" },
                        "points": [
                            { "time_minutes": "09:30", "price": 10.50, "volume": 100, "amount": 1050.0, "avg_price": 10.50 },
                            { "time_minutes": "09:31", "price": 10.51, "volume": 200, "amount": 2102.0 },
                            { "time_minutes": "09:32", "price": 10.52, "amount": 526.0, "avg_price": 10.52 },
                            { "time_minutes": "99:99", "price": 10.53, "volume": 300, "amount": 3159.0, "avg_price": 10.53 },
                            { "time_minutes": "1130", "price": 10.54, "volume": 400, "amount": 4216.0, "avg_price": 10.54 }
                        ]
                    }
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");
        let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let shares = client
            .fetch_minute_share("sh600000", crate::data::models::DateOrRange::Date(date))
            .await
            .expect("fetch ok");
        assert_eq!(
            shares.len(),
            2,
            "expected 2 valid records, got {:?}",
            shares
        );
        assert_eq!(shares[0].timestamp, date.and_hms_opt(9, 30, 0).unwrap());
        assert_eq!(shares[1].timestamp, date.and_hms_opt(11, 30, 0).unwrap());
    }

    #[tokio::test]
    async fn fetch_minute_share_propagates_4xx() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/data/fetch"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "error": { "code": "NOT_FOUND", "message": "unknown code" }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");
        let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let err = client
            .fetch_minute_share("invalid_code", crate::data::models::DateOrRange::Date(date))
            .await
            .expect_err("must error");
        let msg = format!("{err}");
        assert!(
            msg.contains("404") || msg.contains("NOT_FOUND") || msg.contains("unknown"),
            "expected error to mention status/error, got: {msg}"
        );
    }

    #[tokio::test]
    async fn fetch_minute_share_range_loops_per_day() {
        // W3: Range triggers N single-day requests, each yielding records
        // stamped with meta.trading_date (NOT request date — INV-2C).
        use crate::data::models::{DateOrRange, iter_dates_inclusive};
        use wiremock::matchers::{body_partial_json, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // Single Mock with closure responder: each call inspects params.date
        // and returns a record stamped with the requested trading_date.
        // wiremock 0.6 supports closure responders.
        Mock::given(method("POST"))
            .and(path("/data/fetch"))
            .and(body_partial_json(
                serde_json::json!({ "data_category": "MINUTE_DATA" }),
            ))
            .respond_with(|request: &wiremock::Request| {
                let body: serde_json::Value =
                    serde_json::from_slice(&request.body).unwrap_or_default();
                let req_date = body["params"]["date"].as_str().unwrap_or("");
                let resp = serde_json::json!({
                    "status": "ok",
                    "source": "eltdx",
                    "artifact_hash": format!("hash-{}", req_date),
                    "data": [{
                        "meta": { "trading_date": req_date },
                        "points": [{
                            "time_minutes": "0931",
                            "price": 10.0,
                            "volume": 100,
                            "amount": 1000.0,
                            "avg_price": 10.0
                        }]
                    }]
                });
                ResponseTemplate::new(200).set_body_json(resp)
            })
            .expect(3)
            .mount(&server)
            .await;

        let cfg = fast_test_cfg(server.uri());
        let client = OpenStockClient::new(cfg).expect("client build");
        let start = chrono::NaiveDate::from_ymd_opt(2026, 6, 28).unwrap();
        let end = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
        let shares = client
            .fetch_minute_share("sh600000", DateOrRange::Range { start, end })
            .await
            .expect("fetch ok");

        assert_eq!(shares.len(), 3, "one record per day × 3 days");
        let dates: Vec<chrono::NaiveDate> = shares.iter().map(|s| s.timestamp.date()).collect();
        // Verify all days present
        for d in iter_dates_inclusive(start, end) {
            assert!(dates.contains(&d), "expected day {} in results", d);
        }
        // Verify ascending order
        let mut sorted = dates.clone();
        sorted.sort();
        assert_eq!(dates, sorted, "results must be in ascending date order");
    }

    #[tokio::test]
    async fn fetch_minute_share_range_skips_non_trading_days() {
        // W5: Range iterates all days client-side; non-trading days return
        // empty points arrays → no records contributed for that day.
        use crate::data::models::DateOrRange;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/data/fetch"))
            .respond_with(|request: &wiremock::Request| {
                let body: serde_json::Value =
                    serde_json::from_slice(&request.body).unwrap_or_default();
                let req_date = body["params"]["date"].as_str().unwrap_or("");
                // For "2026-06-28" (Sunday) return empty points
                let points: Vec<serde_json::Value> = if req_date == "2026-06-28" {
                    vec![]
                } else {
                    vec![serde_json::json!({
                        "time_minutes": "1000",
                        "price": 10.0, "volume": 100,
                        "amount": 1000.0, "avg_price": 10.0,
                    })]
                };
                let resp = serde_json::json!({
                    "status": "ok",
                    "source": "eltdx",
                    "artifact_hash": "x",
                    "data": [{
                        "meta": { "trading_date": req_date },
                        "points": points
                    }]
                });
                ResponseTemplate::new(200).set_body_json(resp)
            })
            .expect(3)
            .mount(&server)
            .await;

        let cfg = fast_test_cfg(server.uri());
        let client = OpenStockClient::new(cfg).expect("client build");
        let start = chrono::NaiveDate::from_ymd_opt(2026, 6, 28).unwrap();
        let end = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
        let shares = client
            .fetch_minute_share("sh600000", DateOrRange::Range { start, end })
            .await
            .expect("fetch ok");

        // Sunday returns empty, so only 2 trading days × 1 record = 2 records
        assert_eq!(shares.len(), 2, "non-trading day must contribute 0 records");
        let dates: Vec<chrono::NaiveDate> = shares.iter().map(|s| s.timestamp.date()).collect();
        assert!(
            !dates.contains(&start),
            "non-trading day must not appear in results"
        );
    }

    #[test]
    fn parse_time_minutes_accepts_compact_format() {
        assert_eq!(parse_time_minutes("0930"), Some((9, 30)));
        assert_eq!(parse_time_minutes("1130"), Some((11, 30)));
        assert_eq!(parse_time_minutes("1500"), Some((15, 0)));
    }

    #[test]
    fn parse_time_minutes_accepts_colon_format() {
        assert_eq!(parse_time_minutes("09:30"), Some((9, 30)));
        assert_eq!(parse_time_minutes("11:30"), Some((11, 30)));
    }

    #[test]
    fn parse_time_minutes_rejects_invalid() {
        assert_eq!(parse_time_minutes("99:99"), None);
        assert_eq!(parse_time_minutes("25:00"), None);
        assert_eq!(parse_time_minutes("12:60"), None);
        assert_eq!(parse_time_minutes("abc"), None);
        assert_eq!(parse_time_minutes("123"), None);
        assert_eq!(parse_time_minutes("12345"), None);
    }

    // -----------------------------------------------------------------
    // Stream API unit tests (P0.13d Task 4: INV-1A / INV-5A / INV-5B)
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn fetch_minute_klines_stream_collects_same_as_batch_per_chunk() {
        // S5 / INV-1A: stream yields the same records as N batch calls, one per
        // weekly chunk. 14-day range => 2 weekly chunks => 2 /data/bars calls.
        use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
        use chrono::NaiveDate;
        use futures::StreamExt;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"time":"2026-06-01T09:31:00+08:00","open":1.0,"high":2.0,"low":0.5,"close":1.5,"volume":100.0,"amount":150.0},
                    {"time":"2026-06-01T09:32:00+08:00","open":1.5,"high":2.5,"low":1.0,"close":2.0,"volume":200.0,"amount":400.0},
                ]
            })))
            .expect(2) // 14 days / 7 = 2 chunks
            .mount(&server)
            .await;

        let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        let s = client.fetch_minute_klines_stream(
            "sh600000",
            MinutePeriod::Minute1,
            DateOrRange::Range { start, end },
            AdjustType::None,
        );
        futures::pin_mut!(s);

        let mut total = 0usize;
        while let Some(batch) = s.next().await {
            let batch = batch.expect("batch ok");
            total += batch.len();
        }
        assert_eq!(total, 4, "2 chunks × 2 records = 4"); // INV-1A
    }

    #[tokio::test]
    async fn fetch_minute_klines_stream_terminates_on_first_batch_error() {
        // S6 / INV-5A: first Err yields, subsequent next() returns None.
        // Stream must not retry / advance to the next chunk after an Err.
        use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
        use chrono::NaiveDate;
        use futures::StreamExt;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        // 14-day range => 2 chunks; mock fails on every call but we expect
        // only the first chunk's call to fire (stream short-circuits).
        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .respond_with(ResponseTemplate::new(500).set_body_string("simulated server error"))
            .expect(1)
            .mount(&server)
            .await;

        let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        let s = client.fetch_minute_klines_stream(
            "sh600000",
            MinutePeriod::Minute1,
            DateOrRange::Range { start, end },
            AdjustType::None,
        );
        futures::pin_mut!(s);

        let first = s.next().await.expect("first item exists");
        assert!(first.is_err(), "first batch must be Err");

        // Stream must terminate after the error (no second chunk polled).
        let next = s.next().await;
        assert!(next.is_none(), "stream must return None after first Err");
    }

    #[tokio::test]
    async fn fetch_minute_share_stream_yields_empty_vec_for_non_trading_days() {
        // S7 / INV-5B: server returns no records for non-trading days; stream
        // still yields an empty Vec for each day (not skipped).
        // batch count == calendar day count.
        use crate::data::models::DateOrRange;
        use chrono::NaiveDate;
        use futures::StreamExt;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        // Every MINUTE_DATA request returns empty points array.
        Mock::given(method("POST"))
            .and(path("/data/fetch"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"meta": {"trading_date": "2026-06-01"}, "points": []}
                ]
            })))
            .expect(3) // one per calendar day
            .mount(&server)
            .await;

        let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 6, 3).unwrap();
        let s = client.fetch_minute_share_stream("sh600000", DateOrRange::Range { start, end });
        futures::pin_mut!(s);

        let mut batch_count = 0usize;
        let mut total_records = 0usize;
        while let Some(batch) = s.next().await {
            let batch = batch.expect("batch ok");
            batch_count += 1;
            total_records += batch.len();
        }
        assert_eq!(batch_count, 3, "INV-5B: one batch per calendar day");
        assert_eq!(total_records, 0, "no records for non-trading days");
    }

    // -----------------------------------------------------------------
    // Stream API wiremock tests (P0.13d Task 5: INV-2A / INV-2B wire shape)
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn fetch_minute_klines_stream_emits_per_chunk_subrange_body() {
        // W1 / INV-2A: each chunk request body uses start_date/end_date of that
        // chunk (no `date` field) for a multi-week Range input.
        use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
        use chrono::NaiveDate;
        use futures::StreamExt;
        use wiremock::matchers::{body_partial_json, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        // 14-day range => chunk 1: 06-01..=06-07, chunk 2: 06-08..=06-14
        let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();

        // Chunk 1 body assertion
        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .and(body_partial_json(serde_json::json!({
                "start_date": "2026-06-01",
                "end_date": "2026-06-07",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
            .expect(1)
            .mount(&server)
            .await;

        // Chunk 2 body assertion
        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .and(body_partial_json(serde_json::json!({
                "start_date": "2026-06-08",
                "end_date": "2026-06-14",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
            .expect(1)
            .mount(&server)
            .await;

        let s = client.fetch_minute_klines_stream(
            "sh600000",
            MinutePeriod::Minute1,
            DateOrRange::Range { start, end },
            AdjustType::None,
        );
        futures::pin_mut!(s);
        while let Some(b) = s.next().await {
            b.expect("ok");
        }
    }

    #[tokio::test]
    async fn fetch_minute_share_stream_emits_one_request_per_calendar_day() {
        // W2 / INV-2B: each calendar day emits one /data/fetch MINUTE_DATA request.
        use crate::data::models::DateOrRange;
        use chrono::NaiveDate;
        use futures::StreamExt;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        // 5-day range
        let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 6, 5).unwrap();

        Mock::given(method("POST"))
            .and(path("/data/fetch"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"meta": {"trading_date": "2026-06-01"}, "points": []}]
            })))
            .expect(5) // one per calendar day
            .mount(&server)
            .await;

        let s = client.fetch_minute_share_stream("sh600000", DateOrRange::Range { start, end });
        futures::pin_mut!(s);
        while let Some(b) = s.next().await {
            b.expect("ok");
        }
    }

    #[tokio::test]
    async fn fetch_minute_klines_stream_date_mode_emits_single_batch_with_date_field() {
        // W3 / INV-2A Date path: Date(d) -> 1 chunk (d,d) -> body has `date` only.
        use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
        use chrono::NaiveDate;
        use futures::StreamExt;
        use wiremock::matchers::{body_partial_json, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        let d = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();

        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .and(body_partial_json(serde_json::json!({"date": "2026-06-01"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
            .expect(1)
            .mount(&server)
            .await;

        let s = client.fetch_minute_klines_stream(
            "sh600000",
            MinutePeriod::Minute1,
            DateOrRange::Date(d),
            AdjustType::None,
        );
        futures::pin_mut!(s);

        let mut batches = 0usize;
        while let Some(b) = s.next().await {
            b.expect("ok");
            batches += 1;
        }
        assert_eq!(batches, 1, "Date(d) must produce exactly 1 batch");
    }
}
