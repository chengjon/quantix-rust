use arrow::array::{Float64Array, PrimitiveArray, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Date32Type, Field, Schema};
use parquet::arrow::arrow_writer::ArrowWriter;
use quantix_cli::cli::handlers::{
    resolve_import_market_manifest_artifact, resolve_import_market_manifest_artifact_with_options,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::process::Command;
use std::sync::Arc;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn days_since_epoch(year: i32, month: u32, day: u32) -> i32 {
    let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let date = chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap();
    date.signed_duration_since(epoch).num_days() as i32
}

fn write_sample_parquet(path: &std::path::Path) {
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

fn sha256_file_for_test(path: &std::path::Path) -> String {
    let bytes = fs::read(path).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

fn write_manifest_with_artifact(
    artifact_uri: String,
    artifact_hash: &str,
) -> (tempfile::TempDir, std::path::PathBuf) {
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

fn write_manifest() -> (tempfile::TempDir, std::path::PathBuf) {
    write_manifest_with_artifact(
        "file:///exports/cn-a-share-daily.parquet".to_string(),
        "sha256:artifact",
    )
}

#[test]
fn resolve_import_market_manifest_artifact_returns_stable_artifact_summary() {
    let (_dir, manifest_path) = write_manifest();

    let resolved = resolve_import_market_manifest_artifact(
        manifest_path.to_str().unwrap(),
        "cn-a-share-daily@2026-05-12",
        "parquet",
        Some("kline-daily-v1"),
        Some("sha256:artifact"),
    )
    .unwrap();

    assert_eq!(resolved.dataset_version, "cn-a-share-daily@2026-05-12");
    assert_eq!(resolved.domain, "kline.daily");
    assert_eq!(resolved.artifact_type, "parquet");
    assert_eq!(resolved.schema_version, "kline-daily-v1");
    assert_eq!(resolved.uri, "file:///exports/cn-a-share-daily.parquet");
    assert_eq!(resolved.hash, "sha256:artifact");
    assert_eq!(resolved.row_count, 3);
}

#[test]
fn resolve_import_market_manifest_artifact_verifies_local_artifact_hash() {
    let dir = tempfile::tempdir().unwrap();
    let artifact_path = dir.path().join("cn-a-share-daily.parquet");
    fs::write(&artifact_path, "market rows\n").unwrap();
    let (_manifest_dir, manifest_path) = write_manifest_with_artifact(
        format!("file://{}", artifact_path.display()),
        "sha256:a873694304815e968ca9ca7f50689c80d2454aa131973a3c8bbe059ea9f2c83d",
    );

    let resolved = resolve_import_market_manifest_artifact_with_options(
        manifest_path.to_str().unwrap(),
        "cn-a-share-daily@2026-05-12",
        "parquet",
        Some("kline-daily-v1"),
        Some("sha256:a873694304815e968ca9ca7f50689c80d2454aa131973a3c8bbe059ea9f2c83d"),
        true,
    )
    .unwrap();

    assert_eq!(
        resolved.computed_hash.as_deref(),
        Some("sha256:a873694304815e968ca9ca7f50689c80d2454aa131973a3c8bbe059ea9f2c83d")
    );
}

#[test]
fn resolve_import_market_manifest_artifact_rejects_local_artifact_hash_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    let artifact_path = dir.path().join("cn-a-share-daily.parquet");
    fs::write(&artifact_path, "market rows\n").unwrap();
    let (_manifest_dir, manifest_path) = write_manifest_with_artifact(
        format!("file://{}", artifact_path.display()),
        "sha256:wrong-artifact",
    );

    let err = resolve_import_market_manifest_artifact_with_options(
        manifest_path.to_str().unwrap(),
        "cn-a-share-daily@2026-05-12",
        "parquet",
        Some("kline-daily-v1"),
        Some("sha256:wrong-artifact"),
        true,
    )
    .unwrap_err();

    let message = err.to_string();
    assert!(message.contains("数据解析错误"));
    assert!(message.contains("artifact file hash mismatch"));
}

#[test]
fn market_manifest_cli_writes_raw_report_and_controlled_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let artifact_path = dir.path().join("cn-a-share-daily.parquet");
    let report_path = dir.path().join("quantix-regression.json");
    let evidence_path = dir.path().join("quantix-regression.evidence.json");
    fs::write(&artifact_path, "market rows\n").unwrap();
    let (_manifest_dir, manifest_path) = write_manifest_with_artifact(
        format!("file://{}", artifact_path.display()),
        "sha256:a873694304815e968ca9ca7f50689c80d2454aa131973a3c8bbe059ea9f2c83d",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_quantix"))
        .args([
            "import",
            "market-manifest",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--dataset-version",
            "cn-a-share-daily@2026-05-12",
            "--artifact-type",
            "parquet",
            "--schema-version",
            "kline-daily-v1",
            "--artifact-hash",
            "sha256:a873694304815e968ca9ca7f50689c80d2454aa131973a3c8bbe059ea9f2c83d",
            "--verify-artifact-file",
            "--regression-report-output",
            report_path.to_str().unwrap(),
            "--evidence-output",
            evidence_path.to_str().unwrap(),
            "--consumer-build-commit",
            "abc123",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report_path).unwrap()).unwrap();
    assert_eq!(report["schema_version"], "quantix_regression_report.v1");
    assert_eq!(report["dataset_version"], "cn-a-share-daily@2026-05-12");
    assert_eq!(
        report["artifact"]["computed_hash"],
        "sha256:a873694304815e968ca9ca7f50689c80d2454aa131973a3c8bbe059ea9f2c83d"
    );
    assert_eq!(report["regression"]["passed"], true);
    assert_eq!(report["consumer_build"]["database_target"], "dry-run-only");
    assert_eq!(report["consumer_build"]["writes_performed"], false);

    let evidence: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&evidence_path).unwrap()).unwrap();
    assert_eq!(evidence["schema_version"], "evidence.v1");
    assert_eq!(
        evidence["result_summary"]["evidence_type"],
        "promotion_consumer_regression"
    );
    assert_eq!(
        evidence["result_summary"]["dataset_version"],
        "cn-a-share-daily@2026-05-12"
    );
    assert_eq!(evidence["result_summary"]["regression"]["passed"], true);
    assert_eq!(
        evidence["result_summary"]["raw_report"]["path"].as_str(),
        Some(report_path.to_string_lossy().as_ref())
    );
    assert!(
        evidence["result_summary"]["raw_report"]["hash"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
}

#[test]
fn market_manifest_cli_records_local_reference_double_read_comparison() {
    let dir = tempfile::tempdir().unwrap();
    let artifact_path = dir.path().join("cn-a-share-daily.parquet");
    let reference_path = dir.path().join("quantix-reference.parquet");
    let report_path = dir.path().join("quantix-regression.json");
    let evidence_path = dir.path().join("quantix-regression.evidence.json");
    write_sample_parquet(&artifact_path);
    write_sample_parquet(&reference_path);
    let artifact_hash = sha256_file_for_test(&artifact_path);
    let reference_hash = sha256_file_for_test(&reference_path);
    let (_manifest_dir, manifest_path) = write_manifest_with_artifact(
        format!("file://{}", artifact_path.display()),
        &artifact_hash,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_quantix"))
        .args([
            "import",
            "market-manifest",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--dataset-version",
            "cn-a-share-daily@2026-05-12",
            "--artifact-type",
            "parquet",
            "--schema-version",
            "kline-daily-v1",
            "--artifact-hash",
            &artifact_hash,
            "--verify-artifact-file",
            "--comparison-reference-artifact",
            reference_path.to_str().unwrap(),
            "--regression-report-output",
            report_path.to_str().unwrap(),
            "--evidence-output",
            evidence_path.to_str().unwrap(),
            "--consumer-build-commit",
            "abc123",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report_path).unwrap()).unwrap();
    assert_eq!(report["regression"]["passed"], true);
    assert_eq!(
        report["regression"]["comparison_summary"],
        "local_reference_artifact_matched"
    );
    assert_eq!(
        report["regression"]["comparison"]["reference_hash"],
        reference_hash
    );
    assert_eq!(report["regression"]["comparison"]["reference_row_count"], 3);
    assert!(
        report["regression"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check == "double_read_comparison_performed")
    );
    assert!(
        !report["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning == "double_read_comparison_not_yet_implemented")
    );

    let evidence: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&evidence_path).unwrap()).unwrap();
    assert_eq!(evidence["result_summary"]["regression"]["passed"], true);
    assert_eq!(
        evidence["result_summary"]["regression"]["comparison_summary"],
        "local_reference_artifact_matched"
    );
}

#[test]
fn market_manifest_cli_records_source_of_truth_summary_comparison() {
    let dir = tempfile::tempdir().unwrap();
    let artifact_path = dir.path().join("cn-a-share-daily.parquet");
    let summary_path = dir.path().join("clickhouse-source-of-truth-summary.json");
    let report_path = dir.path().join("quantix-regression.json");
    let evidence_path = dir.path().join("quantix-regression.evidence.json");
    write_sample_parquet(&artifact_path);
    let artifact_hash = sha256_file_for_test(&artifact_path);
    fs::write(
        &summary_path,
        json!({
            "source_system": "clickhouse-shadow",
            "source_uri": "clickhouse://quantix.miniqmt_shadow_kline_daily?dataset_version=cn-a-share-daily@2026-05-12",
            "dataset_version": "cn-a-share-daily@2026-05-12",
            "lineage_id": "lineage-20260512",
            "payload_hash": "sha256:payload",
            "row_count": 3,
            "sample_symbols": ["000001.SZ", "600000.SH"],
            "sample_dates": ["2026-05-12", "2026-05-13"]
        })
        .to_string(),
    )
    .unwrap();
    let summary_hash = sha256_file_for_test(&summary_path);
    let (_manifest_dir, manifest_path) = write_manifest_with_artifact(
        format!("file://{}", artifact_path.display()),
        &artifact_hash,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_quantix"))
        .args([
            "import",
            "market-manifest",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--dataset-version",
            "cn-a-share-daily@2026-05-12",
            "--artifact-type",
            "parquet",
            "--schema-version",
            "kline-daily-v1",
            "--artifact-hash",
            &artifact_hash,
            "--verify-artifact-file",
            "--comparison-source-of-truth-summary",
            summary_path.to_str().unwrap(),
            "--regression-report-output",
            report_path.to_str().unwrap(),
            "--evidence-output",
            evidence_path.to_str().unwrap(),
            "--consumer-build-commit",
            "abc123",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report_path).unwrap()).unwrap();
    assert_eq!(report["regression"]["passed"], true);
    assert_eq!(
        report["regression"]["comparison_summary"],
        "source_of_truth_summary_matched"
    );
    assert_eq!(
        report["regression"]["comparison"]["comparison_type"],
        "source_of_truth_summary"
    );
    assert_eq!(
        report["regression"]["comparison"]["reference_hash"],
        summary_hash
    );
    assert_eq!(
        report["regression"]["comparison"]["reference_source_system"],
        "clickhouse-shadow"
    );
    assert_eq!(
        report["regression"]["comparison"]["reference_source_uri"],
        "clickhouse://quantix.miniqmt_shadow_kline_daily?dataset_version=cn-a-share-daily@2026-05-12"
    );
    assert!(
        !report["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning == "double_read_comparison_not_yet_implemented")
    );

    let evidence: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&evidence_path).unwrap()).unwrap();
    assert_eq!(evidence["result_summary"]["regression"]["passed"], true);
    assert_eq!(
        evidence["result_summary"]["regression"]["comparison_summary"],
        "source_of_truth_summary_matched"
    );
}

#[tokio::test]
async fn market_manifest_cli_records_direct_clickhouse_read_only_comparison() {
    let clickhouse = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("count() AS row_count"))
        .and(body_string_contains("FROM miniqmt_shadow_kline_daily"))
        .and(body_string_contains(
            "dataset_version = 'cn-a-share-daily@2026-05-12'",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_string("{\"row_count\":3}\n"))
        .mount(&clickhouse)
        .await;
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("DISTINCT symbol AS value"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("{\"value\":\"000001.SZ\"}\n{\"value\":\"600000.SH\"}\n"),
        )
        .mount(&clickhouse)
        .await;
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("DISTINCT date AS value"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("{\"value\":\"2026-05-12\"}\n{\"value\":\"2026-05-13\"}\n"),
        )
        .mount(&clickhouse)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let artifact_path = dir.path().join("cn-a-share-daily.parquet");
    let report_path = dir.path().join("quantix-regression.json");
    let evidence_path = dir.path().join("quantix-regression.evidence.json");
    write_sample_parquet(&artifact_path);
    let artifact_hash = sha256_file_for_test(&artifact_path);
    let (_manifest_dir, manifest_path) = write_manifest_with_artifact(
        format!("file://{}", artifact_path.display()),
        &artifact_hash,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_quantix"))
        .args([
            "import",
            "market-manifest",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--dataset-version",
            "cn-a-share-daily@2026-05-12",
            "--artifact-type",
            "parquet",
            "--schema-version",
            "kline-daily-v1",
            "--artifact-hash",
            &artifact_hash,
            "--verify-artifact-file",
            "--comparison-clickhouse-url",
            &clickhouse.uri(),
            "--comparison-clickhouse-database",
            "quantix",
            "--comparison-clickhouse-table",
            "miniqmt_shadow_kline_daily",
            "--comparison-clickhouse-dataset-version-column",
            "dataset_version",
            "--comparison-clickhouse-symbol-column",
            "symbol",
            "--comparison-clickhouse-date-column",
            "date",
            "--regression-report-output",
            report_path.to_str().unwrap(),
            "--evidence-output",
            evidence_path.to_str().unwrap(),
            "--consumer-build-commit",
            "abc123",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report_path).unwrap()).unwrap();
    assert_eq!(report["regression"]["passed"], true);
    assert_eq!(
        report["regression"]["comparison_summary"],
        "direct_clickhouse_read_only_matched"
    );
    assert_eq!(
        report["regression"]["comparison"]["comparison_type"],
        "direct_clickhouse_read_only"
    );
    assert_eq!(
        report["regression"]["comparison"]["reference_source_system"],
        "clickhouse"
    );
    assert_eq!(
        report["regression"]["comparison"]["reference_source_uri"],
        "clickhouse://quantix.miniqmt_shadow_kline_daily?dataset_version=cn-a-share-daily@2026-05-12"
    );
    assert_eq!(report["regression"]["comparison"]["reference_row_count"], 3);
    assert_eq!(report["consumer_build"]["writes_performed"], false);

    let evidence: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&evidence_path).unwrap()).unwrap();
    assert_eq!(evidence["result_summary"]["regression"]["passed"], true);
    assert_eq!(
        evidence["result_summary"]["regression"]["comparison_summary"],
        "direct_clickhouse_read_only_matched"
    );
}

#[test]
fn resolve_import_market_manifest_artifact_rejects_artifact_hash_mismatch() {
    let (_dir, manifest_path) = write_manifest();

    let err = resolve_import_market_manifest_artifact(
        manifest_path.to_str().unwrap(),
        "cn-a-share-daily@2026-05-12",
        "parquet",
        Some("kline-daily-v1"),
        Some("sha256:wrong-artifact"),
    )
    .unwrap_err();

    let message = err.to_string();
    assert!(message.contains("数据解析错误"));
    assert!(message.contains("artifact hash mismatch"));
}
