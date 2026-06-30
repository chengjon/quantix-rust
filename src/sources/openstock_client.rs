//! Generic `OpenStockClient` skeleton — reqwest-based, fixture-tested.
//!
//! Knows the uniform `/data/fetch` envelope shape and `X-API-Key`
//! auth. No live HTTP in tests; fixture-only tests exercise
//! [`OpenStockResponse::from_envelope`] and the shared deserialization
//! paths.

use std::time::Duration;

use reqwest::Url;
use reqwest::header::HeaderValue;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::core::{QuantixError, Result};
use crate::sources::openstock_envelope::{
    OpenStockEnvelope, OpenStockErrorEnvelope, artifact_hash,
};

const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Configuration for constructing an [`OpenStockClient`].
#[derive(Debug, Clone)]
pub struct OpenStockClientConfig {
    pub base_url: String,
    pub api_key: String,
    pub timeout: Duration,
}

impl Default for OpenStockClientConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            api_key: String::new(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }
}

/// Generic OpenStock client. Single backend (reqwest); no trait
/// abstraction — only one backend is expected for this slice.
pub struct OpenStockClient {
    base_url: Url,
    api_key: HeaderValue,
    http: reqwest::Client,
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
            cfg.api_key
        };
        let base_url_raw = if cfg.base_url.is_empty() {
            std::env::var("OPENSTOCK_BASE_URL").map_err(|_| {
                QuantixError::Config(
                    "OPENSTOCK_BASE_URL not set and no base_url in config".to_string(),
                )
            })?
        } else {
            cfg.base_url
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
        })
    }

    /// Convenience: build a client entirely from environment variables
    /// (`OPENSTOCK_BASE_URL` + `OPENSTOCK_API_KEY`), with default timeout.
    pub fn from_env() -> Result<Self> {
        Self::new(OpenStockClientConfig::default())
    }

    /// Generic envelope-aware fetch. POST `/data/fetch` with body
    /// `{"data_category": cat, "params": params}`; on 2xx deserialize
    /// into `OpenStockEnvelope<T>` and compose into
    /// [`OpenStockResponse<T>`]; on non-2xx deserialize into
    /// [`OpenStockErrorEnvelope`] and surface as `QuantixError::Other`.
    pub async fn fetch<T: DeserializeOwned>(
        &self,
        category: &str,
        params: Value,
    ) -> Result<OpenStockResponse<T>> {
        let endpoint = self
            .base_url
            .join("data/fetch")
            .map_err(|e| QuantixError::Other(format!("url join failed: {}", e)))?;
        let body = serde_json::json!({
            "data_category": category,
            "params": params,
        });
        let resp = self
            .http
            .post(endpoint)
            .header("X-API-Key", self.api_key.clone())
            .json(&body)
            .send()
            .await
            .map_err(|e| QuantixError::Network(format!("openstock request failed: {}", e)))?;
        let status = resp.status();
        let raw = resp
            .text()
            .await
            .map_err(|e| QuantixError::Network(format!("openstock body read failed: {}", e)))?;

        if !status.is_success() {
            // Try to parse the uniform error envelope; if that fails,
            // surface status + body snippet so the caller sees the
            // actual upstream error rather than a generic JSON failure.
            let summary = match serde_json::from_str::<OpenStockErrorEnvelope>(&raw) {
                Ok(env) => env.to_summary(),
                Err(_) => format!(
                    "openstock: HTTP {} | body: {}",
                    status,
                    raw.chars().take(200).collect::<String>()
                ),
            };
            return Err(QuantixError::Other(summary));
        }

        let env: OpenStockEnvelope<T> = serde_json::from_str(&raw).map_err(|e| {
            QuantixError::Other(format!(
                "openstock: cannot parse success envelope: {} | body: {}",
                e,
                raw.chars().take(200).collect::<String>()
            ))
        })?;
        Ok(OpenStockResponse::from_envelope(env, &raw))
    }

    /// Convenience: fetch `STOCK_CODES`.
    pub async fn fetch_stock_codes(
        &self,
    ) -> Result<OpenStockResponse<crate::sources::openstock_codes::StockCodeRecord>> {
        self.fetch("STOCK_CODES", serde_json::json!({})).await
    }

    /// Convenience: fetch `TRADE_DATES` for a year.
    pub async fn fetch_trade_dates(
        &self,
        year: u32,
    ) -> Result<OpenStockResponse<crate::sources::openstock_calendar::TradeDateRecord>> {
        self.fetch("TRADE_DATES", serde_json::json!({ "year": year }))
            .await
    }

    /// Convenience: fetch `INDEX_KLINES` for a symbol with optional date range.
    pub async fn fetch_index_klines(
        &self,
        code: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<OpenStockResponse<crate::sources::openstock_index::IndexKlineRecord>> {
        let mut params = serde_json::json!({ "code": code });
        if let Some(start) = start {
            params["start"] = Value::String(start.to_string());
        }
        if let Some(end) = end {
            params["end"] = Value::String(end.to_string());
        }
        self.fetch("INDEX_KLINES", params).await
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
}
