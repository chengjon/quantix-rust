//! Uniform envelope types for OpenStock `/data/fetch` responses.
//!
//! Real OpenStock runtime exposes a uniform `POST /data/fetch` endpoint
//! with `data_category` routing. Both success and error responses use
//! fixed envelope shapes (see `/opt/claude/openstock/docs/CONNECTION_GUIDE.md`):
//!
//! - Success: `{ data, source, data_category?, request_id?, route_decision_id?,
//!   quality_flags, cache_state, circuit_state, latency_ms, received_at }`
//! - Error:   `{ code, message, request_id?, details? }`
//!
//! `artifact_hash` is re-exported from `openstock_shadow` so the consumer
//! side has a single SHA-256 source of truth (per `CONNECTION_GUIDE.md
//! §migration` — OpenStock does not push `artifact_hash`, the consumer
//! computes it from the raw body).

use serde::Deserialize;

pub use crate::sources::openstock_shadow::artifact_hash;

/// Raw serde target for a successful `/data/fetch` response.
///
/// `data` is the only required field; every metadata field is optional
/// because providers may omit any of them. Callers deserializing a
/// response body into this type get a 1:1 view of the JSON shape; use
/// [`crate::sources::openstock_client::OpenStockResponse`] for the
/// flattened public view consumed by downstream code.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenStockEnvelope<T> {
    /// Canonical records array. Always a JSON array for the 5 P0
    /// categories in scope (`STOCK_CODES`, `ALL_STOCKS`, `TRADE_DATES`,
    /// `WORKDAYS`, `INDEX_KLINES`).
    pub data: Vec<T>,

    /// Provider that served the request (e.g. `eltdx`, `baostock`).
    #[serde(default)]
    pub source: Option<String>,

    /// Category that was requested (e.g. `STOCK_CODES`).
    #[serde(default)]
    pub data_category: Option<String>,

    /// Server-assigned request identifier.
    #[serde(default)]
    pub request_id: Option<String>,

    /// Routing decision identifier (provider selection trace).
    #[serde(default)]
    pub route_decision_id: Option<String>,

    /// Quality flags raised by the provider (e.g. `partial`, `stale`).
    #[serde(default)]
    pub quality_flags: Vec<String>,

    /// Cache state at the provider (e.g. `hit`, `miss`, `bypass`).
    #[serde(default)]
    pub cache_state: Option<String>,

    /// Circuit breaker state at the provider.
    #[serde(default)]
    pub circuit_state: Option<String>,

    /// Server-side latency in milliseconds (sub-ms precision as float).
    #[serde(default)]
    pub latency_ms: Option<f64>,

    /// Server-side receive timestamp (RFC3339 or `%Y-%m-%dT%H:%M:%S`).
    #[serde(default)]
    pub received_at: Option<String>,
}

impl<T> OpenStockEnvelope<T> {
    /// Number of records in the `data` array.
    pub fn record_count(&self) -> usize {
        self.data.len()
    }

    /// True if the `data` array is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Raw serde target for a non-2xx `/data/fetch` error response.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenStockErrorEnvelope {
    /// Stable upstream error code (e.g. `provider_unavailable`).
    #[serde(default)]
    pub code: Option<String>,

    /// Human-readable error message.
    #[serde(default)]
    pub message: Option<String>,

    /// Optional request identifier for tracing.
    #[serde(default)]
    pub request_id: Option<String>,

    /// Optional structured details (free-form).
    #[serde(default)]
    pub details: Option<serde_json::Value>,
}

impl OpenStockErrorEnvelope {
    /// Render the error envelope into a single-line summary suitable
    /// for embedding in a `QuantixError::Other` message.
    pub fn to_summary(&self) -> String {
        let code = self.code.as_deref().unwrap_or("unknown");
        let message = self.message.as_deref().unwrap_or("");
        match (&self.request_id, &self.details) {
            (Some(req_id), Some(details)) => {
                format!(
                    "openstock error [{}] {}: req_id={} details={}",
                    code, message, req_id, details
                )
            }
            (Some(req_id), None) => {
                format!("openstock error [{}] {}: req_id={}", code, message, req_id)
            }
            (None, Some(details)) => {
                format!(
                    "openstock error [{}] {}: details={}",
                    code, message, details
                )
            }
            (None, None) => format!("openstock error [{}] {}", code, message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Record {
        code: String,
    }

    #[test]
    fn envelope_parses_full_metadata() {
        let raw = r#"{
            "data": [{"code":"600000"},{"code":"000001"}],
            "source": "eltdx",
            "data_category": "STOCK_CODES",
            "request_id": "req-abc",
            "route_decision_id": "rd-1",
            "quality_flags": ["partial"],
            "cache_state": "hit",
            "circuit_state": "closed",
            "latency_ms": 12.5,
            "received_at": "2026-06-30T10:00:00+08:00"
        }"#;
        let env: OpenStockEnvelope<Record> = serde_json::from_str(raw).unwrap();
        assert_eq!(env.record_count(), 2);
        assert!(!env.is_empty());
        assert_eq!(env.source.as_deref(), Some("eltdx"));
        assert_eq!(env.data_category.as_deref(), Some("STOCK_CODES"));
        assert_eq!(env.request_id.as_deref(), Some("req-abc"));
        assert_eq!(env.route_decision_id.as_deref(), Some("rd-1"));
        assert_eq!(env.quality_flags, vec!["partial".to_string()]);
        assert_eq!(env.cache_state.as_deref(), Some("hit"));
        assert_eq!(env.circuit_state.as_deref(), Some("closed"));
        assert_eq!(env.latency_ms, Some(12.5));
        assert!(env.received_at.is_some());
        assert_eq!(env.data[0].code, "600000");
    }

    #[test]
    fn envelope_defaults_missing_metadata() {
        let raw = r#"{"data": []}"#;
        let env: OpenStockEnvelope<Record> = serde_json::from_str(raw).unwrap();
        assert!(env.is_empty());
        assert_eq!(env.source, None);
        assert_eq!(env.data_category, None);
        assert_eq!(env.quality_flags, Vec::<String>::new());
        assert_eq!(env.latency_ms, None);
    }

    #[test]
    fn error_envelope_summary_with_all_fields() {
        let raw = r#"{
            "code": "provider_unavailable",
            "message": "baostock offline",
            "request_id": "req-xyz",
            "details": {"retry_after_ms": 5000}
        }"#;
        let env: OpenStockErrorEnvelope = serde_json::from_str(raw).unwrap();
        let summary = env.to_summary();
        assert!(summary.contains("provider_unavailable"));
        assert!(summary.contains("baostock offline"));
        assert!(summary.contains("req-xyz"));
        assert!(summary.contains("retry_after_ms"));
    }

    #[test]
    fn error_envelope_summary_with_minimal_fields() {
        let raw = r#"{"code": "rate_limited"}"#;
        let env: OpenStockErrorEnvelope = serde_json::from_str(raw).unwrap();
        assert_eq!(env.to_summary(), "openstock error [rate_limited] ");
    }
}
