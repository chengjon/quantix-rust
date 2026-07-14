//! MiniQMT market-import handler integration tests, split by concern.
//!
//! Historical note: `miniqmt_market_import_handler_test.rs` was 588 lines and was split:
//! - [`resolve`] — the four `resolve_import_market_manifest_artifact_*` cases
//!   (summary, hash verify, hash mismatch, handler-level mismatch),
//! - [`cli_comparison`] — the five `market_manifest_cli_*` end-to-end CLI cases
//!   (regression report, local reference comparison, source-of-truth summary,
//!   direct clickhouse read-only, ...).
//!
//! Shared file helpers (`days_since_epoch`, `write_sample_parquet`,
//! `sha256_file_for_test`, `write_manifest_with_artifact`, `write_manifest`) live here.

use arrow::array::{Float64Array, PrimitiveArray, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Date32Type, Field, Schema};
use parquet::arrow::arrow_writer::ArrowWriter;
use sha2::{Digest, Sha256};
use std::fs;
use std::sync::Arc;

pub(super) fn days_since_epoch(year: i32, month: u32, day: u32) -> i32 {
    let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let date = chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap();
    date.signed_duration_since(epoch).num_days() as i32
}

pub(super) fn write_sample_parquet(path: &std::path::Path) {
    let schema = Arc::new(Schema::new(vec![
        Field::new("symbol", DataType::Utf8, false),
        Field::new("date", DataType::Date32, false),
        Field::new("open", DataType::Float64, false),
        Field::new("close", DataType::Float64, false),
    ]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(vec![
                "000001.SZ",
                "600000.SH",
                "000001.SZ",
            ])),
            Arc::new(PrimitiveArray::<Date32Type>::from(vec![
                days_since_epoch(2026, 5, 12),
                days_since_epoch(2026, 5, 13),
                days_since_epoch(2026, 5, 12),
            ])),
            Arc::new(Float64Array::from(vec![10.0, 20.0, 10.5])),
            Arc::new(Float64Array::from(vec![10.2, 20.5, 10.8])),
        ],
    )
    .unwrap();
    let file = fs::File::create(path).unwrap();
    let mut writer = ArrowWriter::try_new(file, schema, None).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();
}

pub(super) fn sha256_file_for_test(path: &std::path::Path) -> String {
    let bytes = fs::read(path).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

pub(super) fn write_manifest_with_artifact(
    artifact_uri: String,
    artifact_hash: &str,
) -> (tempfile::TempDir, std::path::PathBuf) {
    use serde_json::json;

    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("manifest.json");
    fs::write(
        &manifest_path,
        json!({
            "contract_profile": "market-data-platform-v1",
            "dataset_version": "cn-a-share-daily@2026-05-12",
            "schema_version": "manifest-v1",
            "domain": "kline.daily",
            "maturity": "validated",
            "quality_status": "validated",
            "published": true,
            "lineage_id": "lineage-20260512",
            "row_count": 3,
            "payload_hash": "sha256:payload",
            "sources": [
                {
                    "source_system": "xtdata",
                    "role": "primary",
                    "source_version": "xtdata-daily-20260512"
                }
            ],
            "artifacts": [
                {
                    "type": "parquet",
                    "uri": artifact_uri,
                    "schema_version": "kline-daily-v1",
                    "row_count": 3,
                    "hash": artifact_hash,
                }
            ],
            "quality": {
                "blocking_issues": [],
                "warnings": [],
                "gap_count": 0,
                "conflict_count": 0
            }
        })
        .to_string(),
    )
    .unwrap();

    (dir, manifest_path)
}

pub(super) fn write_manifest() -> (tempfile::TempDir, std::path::PathBuf) {
    write_manifest_with_artifact(
        "file:///exports/cn-a-share-daily.parquet".to_string(),
        "sha256:artifact",
    )
}

mod cli_comparison;
mod resolve;
