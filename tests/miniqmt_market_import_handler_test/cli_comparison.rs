use super::*;

#[test]
fn market_manifest_cli_writes_raw_report_and_controlled_evidence() {
    use std::process::Command;

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
    use std::process::Command;

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
    use serde_json::json;
    use std::process::Command;

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
    use std::process::Command;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
