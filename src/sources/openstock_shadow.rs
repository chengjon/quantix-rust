//! OpenStock live shadow persistence write path.
//!
//! Pure helpers and orchestration for writing captured live shadow
//! payloads into the `quantix_shadow.openstock_daily_kline_shadow`
//! namespace. This module never performs network I/O on its own; the
//! only side effects flow through the injected [`ClickHouseClient`]
//! handle, and only after both opt-in gates (`--apply` and
//! `QUANTIX_SHADOW_PERSIST_CONFIRM=yes`) have been satisfied.
//!
//! See `docs/reports/OPENSTOCK_DATA_CONSUMPTION_P0_8G_IMPL_SHADOW_PERSISTENCE_2026-06-29.md`
//! for the full design contract.

use std::collections::HashSet;

use chrono::Utc;
use rust_decimal::Decimal;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::data::models::AdjustType;
use crate::db::clickhouse::ClickHouseClient;
use crate::sources::openstock::LiveShadowReport;

/// SHA-256 hex digest of the raw payload bytes. Provides a stable
/// artifact identity for a given `raw: &str` regardless of when or
/// where it is hashed.
pub fn artifact_hash(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

/// Generate a fresh batch identifier. UUIDv4 gives us enough entropy
/// that two concurrent batches will not collide; the timestamp is
/// carried separately via `ingested_at` on each row.
pub fn new_batch_id() -> String {
    Uuid::new_v4().to_string()
}

/// ClickHouse LowCardinality(String) representation of [`AdjustType`].
/// Must match `db/schema/quantix_shadow_init.sql`.
fn adjust_type_token(adjust: AdjustType) -> &'static str {
    match adjust {
        AdjustType::None => "none",
        AdjustType::QFQ => "qfq",
        AdjustType::HFQ => "hfq",
    }
}

/// One canonical shadow row. The fields are intentionally identical
/// to the columns of `openstock_daily_kline_shadow` minus the batch
/// metadata, so the insert path can stay linear.
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowKlineRow {
    pub source: &'static str,
    pub period: String,
    pub code: String,
    pub date: chrono::NaiveDate,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: i64,
    pub amount: Option<Decimal>,
    pub adjust_type: &'static str,
    pub batch_id: String,
    pub artifact_hash: String,
    pub ingested_by: String,
    pub ingested_at: chrono::DateTime<Utc>,
}

/// Summary of a finished (or dry-run-previewed) shadow write.
#[derive(Debug, Clone)]
pub struct ShadowWriteReport {
    pub dry_run: bool,
    pub batch_id: String,
    pub artifact_hash: String,
    pub row_count: usize,
    pub duplicate_key_count: usize,
    pub applied: bool,
}

/// Reasons a shadow write may be refused before any DB touch.
#[derive(Debug, Clone, PartialEq)]
pub enum ShadowWriteError {
    /// `apply` flag was not set. The dry-run preview succeeded; pass
    /// `--apply` to materialize the rows.
    ApplyFlagRequired,
    /// `QUANTIX_SHADOW_PERSIST_CONFIRM=yes` was not observed. Even
    /// with `--apply`, the second gate must also fire.
    EnvConfirmRequired,
    /// The live validator flagged one or more records as
    /// unparseable. The shadow namespace must stay clean.
    FailClosedNotEmpty { count: usize },
    /// The live validator flagged drift against the request. The
    /// shadow namespace is for clean payloads only.
    DriftNotEmpty { count: usize },
    /// The validator returned zero mapped records. Nothing to write.
    EmptyPayload,
    /// `record_count != mapped_count`. The validator and the shadow
    /// builder disagree; refuse to write a partial picture.
    MappedCountMismatch {
        record_count: usize,
        mapped_count: usize,
    },
    /// The payload contains duplicate `(source, period, code, date,
    /// adjust_type)` keys. The unique key must hold inside a batch.
    DuplicateKeys { count: usize },
    /// The underlying ClickHouse client raised an error during an
    /// `--apply` write. Bubbled up verbatim so the operator can react.
    DbError(String),
}

/// Pure construction of the canonical shadow rows from a live shadow
/// report. Performs all dry-run gate checks (drift, fail-closed,
/// empties, duplicates, count consistency) without ever touching a
/// ClickHouse client. Used both by the dry-run preview and as the
/// pre-flight check immediately before an `--apply` write.
pub fn build_shadow_rows_from_report(
    report: &LiveShadowReport,
    raw_payload: &str,
    batch_id: &str,
    ingested_by: &str,
) -> Result<Vec<ShadowKlineRow>, ShadowWriteError> {
    if !report.fail_closed_errors.is_empty() {
        return Err(ShadowWriteError::FailClosedNotEmpty {
            count: report.fail_closed_errors.len(),
        });
    }
    if !report.drifts.is_empty() {
        return Err(ShadowWriteError::DriftNotEmpty {
            count: report.drifts.len(),
        });
    }
    if report.klines.is_empty() {
        return Err(ShadowWriteError::EmptyPayload);
    }
    if report.record_count != report.mapped_count {
        return Err(ShadowWriteError::MappedCountMismatch {
            record_count: report.record_count,
            mapped_count: report.mapped_count,
        });
    }

    let artifact = artifact_hash(raw_payload);
    let ingested_at = Utc::now();
    let source: &'static str = "openstock_live_shadow";

    let mut seen: HashSet<(&str, &str, chrono::NaiveDate, &str)> = HashSet::new();
    let mut duplicates = 0usize;

    let mut rows: Vec<ShadowKlineRow> = Vec::with_capacity(report.klines.len());
    for kline in &report.klines {
        let adjust = adjust_type_token(kline.adjust_type);
        let key = (
            source,
            report.period.as_deref().unwrap_or("daily"),
            kline.date,
            adjust,
        );
        if !seen.insert(key) {
            duplicates += 1;
        }
        rows.push(ShadowKlineRow {
            source,
            period: report.period.clone().unwrap_or_else(|| "daily".to_string()),
            code: kline.code.clone(),
            date: kline.date,
            open: kline.open,
            high: kline.high,
            low: kline.low,
            close: kline.close,
            volume: kline.volume,
            amount: kline.amount,
            adjust_type: adjust,
            batch_id: batch_id.to_string(),
            artifact_hash: artifact.clone(),
            ingested_by: ingested_by.to_string(),
            ingested_at,
        });
    }

    if duplicates > 0 {
        return Err(ShadowWriteError::DuplicateKeys { count: duplicates });
    }

    Ok(rows)
}

/// Top-level orchestration: build rows (dry-run gate), enforce the
/// double-gate opt-in, then hand off to the ClickHouse client.
///
/// When `apply == false` or `env_confirmed == false`, returns a
/// [`ShadowWriteReport`] with `dry_run: true` and `applied: false`
/// and never calls into the client. When both gates fire, calls
/// `insert_shadow_klines` and returns `dry_run: false,
/// applied: true`.
pub async fn write_shadow_klines(
    client: &ClickHouseClient,
    report: &LiveShadowReport,
    raw_payload: &str,
    batch_id: &str,
    ingested_by: &str,
    apply: bool,
    env_confirmed: bool,
) -> Result<ShadowWriteReport, ShadowWriteError> {
    let rows = build_shadow_rows_from_report(report, raw_payload, batch_id, ingested_by)?;
    let row_count = rows.len();
    let artifact = artifact_hash(raw_payload);

    if !apply {
        return Ok(ShadowWriteReport {
            dry_run: true,
            batch_id: batch_id.to_string(),
            artifact_hash: artifact,
            row_count,
            duplicate_key_count: 0,
            applied: false,
        });
    }
    if !env_confirmed {
        return Err(ShadowWriteError::EnvConfirmRequired);
    }

    client
        .insert_shadow_klines(&rows)
        .await
        .map_err(|e| ShadowWriteError::DbError(e.to_string()))?;

    Ok(ShadowWriteReport {
        dry_run: false,
        batch_id: batch_id.to_string(),
        artifact_hash: artifact,
        row_count,
        duplicate_key_count: 0,
        applied: true,
    })
}

/// Idempotent rollback. Removes every row matching `batch_id`. Safe
/// to call repeatedly: a second invocation reports zero rows
/// affected.
pub async fn rollback_shadow_batch(
    client: &ClickHouseClient,
    batch_id: &str,
) -> Result<usize, ShadowWriteError> {
    client
        .delete_shadow_batch(batch_id)
        .await
        .map_err(|e| ShadowWriteError::DbError(e.to_string()))
}

/// Post-write verification. Returns the row count currently
/// attributable to `batch_id` in the shadow table.
pub async fn verify_shadow_batch(
    client: &ClickHouseClient,
    batch_id: &str,
) -> Result<u64, ShadowWriteError> {
    client
        .count_shadow_batch(batch_id)
        .await
        .map_err(|e| ShadowWriteError::DbError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::openstock::{LiveShadowRequest, validate_live_shadow_payload};

    fn sample_payload() -> &'static str {
        // Two daily records for code 600000, simple rise pattern.
        r#"{"data":[
            {"symbol":"600000","time":"2026-06-01","open":"10.00","high":"10.20","low":"9.95","close":"10.10","volume":1000,"amount":"10100.00","period":"daily"},
            {"symbol":"600000","time":"2026-06-02","open":"10.10","high":"10.30","low":"10.05","close":"10.25","volume":1100,"amount":"11275.00","period":"daily"}
        ]}"#
    }

    fn request() -> LiveShadowRequest {
        LiveShadowRequest {
            symbol: "600000".to_string(),
            period: "daily".to_string(),
            start_date: "2026-06-01".to_string(),
            end_date: "2026-06-02".to_string(),
            limit: Some(100),
        }
    }

    #[test]
    fn artifact_hash_is_deterministic() {
        let raw = sample_payload();
        let a = artifact_hash(raw);
        let b = artifact_hash(raw);
        assert_eq!(a, b, "same input must hash identically");
        assert_eq!(a.len(), 64, "SHA-256 hex is 64 chars");
    }

    #[test]
    fn artifact_hash_distinguishes_inputs() {
        let a = artifact_hash(sample_payload());
        let b = artifact_hash(&sample_payload().replace("600000", "600001"));
        assert_ne!(a, b, "different inputs must hash differently");
    }

    #[test]
    fn build_rows_succeeds_for_clean_payload() {
        let report = validate_live_shadow_payload(sample_payload(), &request()).unwrap();
        let batch = new_batch_id();
        let rows = build_shadow_rows_from_report(&report, sample_payload(), &batch, "ci").unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|r| r.batch_id == batch));
        assert!(rows.iter().all(|r| r.source == "openstock_live_shadow"));
        assert!(rows.iter().all(|r| r.ingested_by == "ci"));
    }

    #[test]
    fn build_rows_rejects_drift_payload() {
        // limit=1 forces a limit drift on a 2-record payload.
        let mut req = request();
        req.limit = Some(1);
        let report = validate_live_shadow_payload(sample_payload(), &req).unwrap();
        let batch = new_batch_id();
        let err = build_shadow_rows_from_report(&report, sample_payload(), &batch, "ci")
            .expect_err("drift payload must be rejected");
        assert_eq!(err, ShadowWriteError::DriftNotEmpty { count: 1 });
    }

    #[test]
    fn build_rows_rejects_empty_payload() {
        let raw = r#"{"data":[]}"#;
        let _ = validate_live_shadow_payload(raw, &request())
            .expect_err("empty envelope rejected by validator upstream");
        // Validator already short-circuits empty envelopes, so reach
        // the gate via an explicit empty Kline vector.
        let report = LiveShadowReport {
            dry_run: true,
            source: "openstock_live_shadow",
            status: crate::sources::openstock::LiveShadowStatus::Ok,
            record_count: 0,
            mapped_count: 0,
            symbol: None,
            period: Some("daily".to_string()),
            received_date_range: None,
            drifts: Vec::new(),
            fail_closed_errors: Vec::new(),
            klines: Vec::new(),
        };
        let err = build_shadow_rows_from_report(&report, "raw", "batch", "ci")
            .expect_err("empty payload must be rejected");
        assert_eq!(err, ShadowWriteError::EmptyPayload);
    }

    #[test]
    fn build_rows_rejects_duplicate_keys() {
        use crate::data::models::{AdjustType, Kline};
        use rust_decimal::Decimal;
        let dup = Kline {
            code: "600000".to_string(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            open: Decimal::from(10),
            high: Decimal::from(10),
            low: Decimal::from(10),
            close: Decimal::from(10),
            volume: 1,
            amount: None,
            adjust_type: AdjustType::None,
        };
        let report = LiveShadowReport {
            dry_run: true,
            source: "openstock_live_shadow",
            status: crate::sources::openstock::LiveShadowStatus::Ok,
            record_count: 2,
            mapped_count: 2,
            symbol: Some("600000".to_string()),
            period: Some("daily".to_string()),
            received_date_range: None,
            drifts: Vec::new(),
            fail_closed_errors: Vec::new(),
            klines: vec![dup.clone(), dup],
        };
        let err = build_shadow_rows_from_report(&report, "raw", "batch", "ci")
            .expect_err("duplicate keys must be rejected");
        assert_eq!(err, ShadowWriteError::DuplicateKeys { count: 1 });
    }
}
