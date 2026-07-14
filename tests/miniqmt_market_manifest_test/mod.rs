//! MiniQMT market-dataset manifest integration tests, split by concern.
//!
//! Historical note: `miniqmt_market_manifest_test.rs` was 662 lines and was split:
//! - [`validation`] — `ManifestValidator` accept/reject cases plus the JSON
//!   `load_manifest_from_slice` / `load_manifest_from_path` round-trips,
//! - [`artifact`] — `MarketArtifactSelector`, `ResolvedMarketArtifact`, and the
//!   `MarketArtifactRequest` resolution / dry-run pretty-json cases,
//! - [`regression`] — `QuantixRegressionReport`, `QuantixRegressionEvidence`, and
//!   `ControlledPersistencePolicy` cases.
//!
//! The shared manifest sample (`valid_manifest`) and the three file helpers
//! (`days_since_epoch`, `write_sample_parquet`, `sha256_file_for_test`) live here.

use chrono::NaiveDate;
use quantix_cli::miniqmt_market::{
    ControlledPersistencePolicy, ManifestIntake, ManifestQuality, ManifestSource,
    ManifestValidator, MarketArtifactRequest, MarketArtifactSelector, MarketDatasetArtifact,
    MarketDatasetManifest, QuantixRegressionEvidence, QuantixRegressionReport, RawReportReference,
    ResolvedMarketArtifact, load_manifest_from_path, load_manifest_from_slice,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::sync::Arc;

pub(super) fn valid_manifest() -> MarketDatasetManifest {
    MarketDatasetManifest {
        dataset_version: "kline_daily_20260512_v1".to_string(),
        schema_version: "v1".to_string(),
        contract_profile: "market-data-platform-v1".to_string(),
        domain: "kline_daily".to_string(),
        maturity: "validated".to_string(),
        quality_status: "validated".to_string(),
        published: true,
        lineage_id: "lineage-20260512".to_string(),
        row_count: 123_456,
        payload_hash: "dataset-hash".to_string(),
        rows_hash: Some("dataset-rows-hash".to_string()),
        sources: vec![
            ManifestSource {
                source_system: "xtdata".to_string(),
                role: "primary".to_string(),
                source_version: Some("xtdata_daily_20260512_v1".to_string()),
            },
            ManifestSource {
                source_system: "baostock".to_string(),
                role: "validation".to_string(),
                source_version: Some("baostock_daily_20260512_v1".to_string()),
            },
        ],
        artifacts: vec![MarketDatasetArtifact {
            artifact_type: "parquet".to_string(),
            uri: "nas://market-data/kline_daily/dataset_version=kline_daily_20260512_v1/"
                .to_string(),
            schema_version: "v1".to_string(),
            row_count: 123_456,
            hash: "artifact-hash".to_string(),
            rows_hash: Some("artifact-rows-hash".to_string()),
        }],
        quality: ManifestQuality {
            blocking_issues: vec![],
            warnings: vec![],
            gap_count: 0,
            conflict_count: 0,
        },
    }
}

pub(super) fn days_since_epoch(year: i32, month: u32, day: u32) -> i32 {
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
    date.signed_duration_since(epoch).num_days() as i32
}

pub(super) fn write_sample_parquet(path: &std::path::Path) {
    use arrow::array::{Float64Array, PrimitiveArray, RecordBatch, StringArray};
    use arrow::datatypes::{DataType, Date32Type, Field, Schema};
    use parquet::arrow::arrow_writer::ArrowWriter;

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

mod artifact;
mod regression;
mod validation;
