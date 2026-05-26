use arrow::array::{Float64Array, PrimitiveArray, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Date32Type, Field, Schema};
use parquet::arrow::arrow_writer::ArrowWriter;
use quantix_cli::miniqmt_market::{
    ControlledPersistencePolicy, ManifestIntake, ManifestQuality, ManifestSource,
    ManifestValidator, MarketArtifactRequest, MarketArtifactSelector, MarketDatasetArtifact,
    MarketDatasetManifest, QuantixRegressionContext, QuantixRegressionEvidence,
    QuantixRegressionReport, RawReportReference, ResolvedMarketArtifact, load_manifest_from_path,
    load_manifest_from_slice,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::sync::Arc;

fn valid_manifest() -> MarketDatasetManifest {
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

#[test]
fn manifest_validation_accepts_published_release_dataset() {
    let manifest = valid_manifest();
    let validator =
        ManifestValidator::new("kline_daily_20260512_v1").require_artifact_hash("artifact-hash");

    validator.validate(&manifest).unwrap();
}

#[test]
fn manifest_validation_rejects_unpublished_candidate_dataset() {
    let mut manifest = valid_manifest();
    manifest.published = false;

    let err = ManifestValidator::new("kline_daily_20260512_v1")
        .validate(&manifest)
        .unwrap_err();

    assert!(err.contains("not published"));
}

#[test]
fn manifest_validation_rejects_dataset_version_mismatch() {
    let manifest = valid_manifest();

    let err = ManifestValidator::new("other_dataset_version")
        .validate(&manifest)
        .unwrap_err();

    assert!(err.contains("dataset_version mismatch"));
}

#[test]
fn manifest_validation_rejects_blocking_quality_issues() {
    let mut manifest = valid_manifest();
    manifest
        .quality
        .blocking_issues
        .push("missing lineage for 600000.SH".to_string());

    let err = ManifestValidator::new("kline_daily_20260512_v1")
        .validate(&manifest)
        .unwrap_err();

    assert!(err.contains("blocking quality issues"));
}

#[test]
fn manifest_validation_rejects_artifact_hash_mismatch() {
    let manifest = valid_manifest();

    let err = ManifestValidator::new("kline_daily_20260512_v1")
        .require_artifact_hash("expected-hash")
        .validate(&manifest)
        .unwrap_err();

    assert!(err.contains("artifact hash mismatch"));
}

#[test]
fn load_manifest_from_slice_parses_and_validates_manifest_json() {
    let manifest_json = serde_json::to_vec(&valid_manifest()).unwrap();
    let validator =
        ManifestValidator::new("kline_daily_20260512_v1").require_artifact_hash("artifact-hash");

    let manifest = load_manifest_from_slice(&manifest_json, &validator).unwrap();

    assert_eq!(manifest.dataset_version, "kline_daily_20260512_v1");
    assert_eq!(manifest.artifacts[0].hash, "artifact-hash");
}

#[test]
fn load_manifest_from_slice_rejects_invalid_json() {
    let validator = ManifestValidator::new("kline_daily_20260512_v1");

    let err = load_manifest_from_slice(b"{invalid-json", &validator).unwrap_err();

    assert!(err.contains("invalid manifest json"));
}

#[test]
fn load_manifest_from_path_reads_and_validates_manifest_file() {
    let manifest_path = std::env::temp_dir().join(format!(
        "quantix-miniqmt-manifest-{}.json",
        std::process::id()
    ));
    let manifest_json = serde_json::to_vec(&valid_manifest()).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    let validator =
        ManifestValidator::new("kline_daily_20260512_v1").require_artifact_hash("artifact-hash");
    let manifest = load_manifest_from_path(&manifest_path, &validator).unwrap();

    fs::remove_file(&manifest_path).unwrap();
    assert_eq!(manifest.dataset_version, "kline_daily_20260512_v1");
}

#[test]
fn load_manifest_from_path_reports_missing_file() {
    let manifest_path = std::env::temp_dir().join(format!(
        "quantix-miniqmt-missing-manifest-{}.json",
        std::process::id()
    ));
    let validator = ManifestValidator::new("kline_daily_20260512_v1");

    let err = load_manifest_from_path(&manifest_path, &validator).unwrap_err();

    assert!(err.contains("failed to read manifest"));
}

#[test]
fn manifest_intake_loads_from_path_with_configured_validator() {
    let manifest_path = std::env::temp_dir().join(format!(
        "quantix-miniqmt-intake-manifest-{}.json",
        std::process::id()
    ));
    let manifest_json = serde_json::to_vec(&valid_manifest()).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    let intake = ManifestIntake::new(
        ManifestValidator::new("kline_daily_20260512_v1").require_artifact_hash("artifact-hash"),
    );
    let manifest = intake.load_from_path(&manifest_path).unwrap();

    fs::remove_file(&manifest_path).unwrap();
    assert_eq!(manifest.lineage_id, "lineage-20260512");
}

#[test]
fn manifest_intake_rejects_payload_with_configured_validator() {
    let manifest_json = serde_json::to_vec(&valid_manifest()).unwrap();
    let intake = ManifestIntake::new(ManifestValidator::new("other_dataset_version"));

    let err = intake.load_from_slice(&manifest_json).unwrap_err();

    assert!(err.contains("dataset_version mismatch"));
}

#[test]
fn artifact_selector_selects_unique_artifact_by_type_and_schema_version() {
    let manifest = valid_manifest();
    let selector = MarketArtifactSelector::new("parquet").require_schema_version("v1");

    let artifact = selector.select(&manifest).unwrap();

    assert_eq!(
        artifact.uri,
        "nas://market-data/kline_daily/dataset_version=kline_daily_20260512_v1/"
    );
    assert_eq!(artifact.hash, "artifact-hash");
}

#[test]
fn artifact_selector_rejects_missing_schema_version() {
    let manifest = valid_manifest();
    let selector = MarketArtifactSelector::new("parquet").require_schema_version("v2");

    let err = selector.select(&manifest).unwrap_err();

    assert!(err.contains("no matching artifact"));
}

#[test]
fn artifact_selector_rejects_ambiguous_artifacts() {
    let mut manifest = valid_manifest();
    manifest.artifacts.push(MarketDatasetArtifact {
        artifact_type: "parquet".to_string(),
        uri: "nas://market-data/kline_daily/alternate/".to_string(),
        schema_version: "v1".to_string(),
        row_count: 123_456,
        hash: "alternate-artifact-hash".to_string(),
        rows_hash: Some("alternate-artifact-rows-hash".to_string()),
    });
    let selector = MarketArtifactSelector::new("parquet").require_schema_version("v1");

    let err = selector.select(&manifest).unwrap_err();

    assert!(err.contains("multiple matching artifacts"));
}

#[test]
fn resolved_market_artifact_captures_stable_consumer_fields() {
    let manifest = valid_manifest();
    let selector = MarketArtifactSelector::new("parquet").require_schema_version("v1");
    let artifact = selector.select(&manifest).unwrap();

    let resolved = ResolvedMarketArtifact::from_manifest_artifact(&manifest, artifact);

    assert_eq!(resolved.dataset_version, "kline_daily_20260512_v1");
    assert_eq!(resolved.domain, "kline_daily");
    assert_eq!(resolved.lineage_id, "lineage-20260512");
    assert_eq!(resolved.payload_hash, "dataset-hash");
    assert_eq!(resolved.maturity, "validated");
    assert_eq!(resolved.quality_status, "validated");
    assert_eq!(resolved.artifact_type, "parquet");
    assert_eq!(resolved.schema_version, "v1");
    assert_eq!(
        resolved.uri,
        "nas://market-data/kline_daily/dataset_version=kline_daily_20260512_v1/"
    );
    assert_eq!(resolved.hash, "artifact-hash");
    assert_eq!(resolved.rows_hash.as_deref(), Some("artifact-rows-hash"));
    assert_eq!(resolved.row_count, 123_456);
}

#[test]
fn manifest_intake_resolves_artifact_from_path() {
    let manifest_path = std::env::temp_dir().join(format!(
        "quantix-miniqmt-resolve-manifest-{}.json",
        std::process::id()
    ));
    let manifest_json = serde_json::to_vec(&valid_manifest()).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    let intake = ManifestIntake::new(
        ManifestValidator::new("kline_daily_20260512_v1").require_artifact_hash("artifact-hash"),
    );
    let selector = MarketArtifactSelector::new("parquet").require_schema_version("v1");

    let resolved = intake
        .resolve_artifact_from_path(&manifest_path, &selector)
        .unwrap();

    fs::remove_file(&manifest_path).unwrap();
    assert_eq!(resolved.dataset_version, "kline_daily_20260512_v1");
    assert_eq!(
        resolved.uri,
        "nas://market-data/kline_daily/dataset_version=kline_daily_20260512_v1/"
    );
}

#[test]
fn manifest_intake_resolve_artifact_from_slice_rejects_missing_artifact() {
    let manifest_json = serde_json::to_vec(&valid_manifest()).unwrap();
    let intake = ManifestIntake::new(ManifestValidator::new("kline_daily_20260512_v1"));
    let selector = MarketArtifactSelector::new("json").require_schema_version("v1");

    let err = intake
        .resolve_artifact_from_slice(&manifest_json, &selector)
        .unwrap_err();

    assert!(err.contains("no matching artifact"));
}

#[test]
fn resolved_market_artifact_formats_pretty_json_for_dry_run_output() {
    let manifest = valid_manifest();
    let selector = MarketArtifactSelector::new("parquet").require_schema_version("v1");
    let artifact = selector.select(&manifest).unwrap();
    let resolved = ResolvedMarketArtifact::from_manifest_artifact(&manifest, artifact);

    let json = resolved.to_pretty_json().unwrap();

    assert!(json.contains("\"dataset_version\": \"kline_daily_20260512_v1\""));
    assert!(json.contains("\"domain\": \"kline_daily\""));
    assert!(json.contains("\"artifact_type\": \"parquet\""));
    assert!(json.contains("\"hash\": \"artifact-hash\""));
}

#[test]
fn market_artifact_request_resolves_artifact_from_slice() {
    let manifest_json = serde_json::to_vec(&valid_manifest()).unwrap();
    let request = MarketArtifactRequest::new("kline_daily_20260512_v1", "parquet")
        .require_schema_version("v1")
        .require_artifact_hash("artifact-hash");

    let resolved = request.resolve_from_slice(&manifest_json).unwrap();

    assert_eq!(resolved.dataset_version, "kline_daily_20260512_v1");
    assert_eq!(resolved.artifact_type, "parquet");
    assert_eq!(resolved.schema_version, "v1");
    assert_eq!(resolved.hash, "artifact-hash");
}

#[test]
fn market_artifact_request_rejects_artifact_hash_mismatch() {
    let manifest_json = serde_json::to_vec(&valid_manifest()).unwrap();
    let request = MarketArtifactRequest::new("kline_daily_20260512_v1", "parquet")
        .require_schema_version("v1")
        .require_artifact_hash("unexpected-hash");

    let err = request.resolve_from_slice(&manifest_json).unwrap_err();

    assert!(err.contains("artifact hash mismatch"));
}

#[test]
fn quantix_regression_report_records_dry_run_identity_and_build_context() {
    let manifest = valid_manifest();
    let selector = MarketArtifactSelector::new("parquet").require_schema_version("v1");
    let artifact = selector.select(&manifest).unwrap();
    let mut resolved = ResolvedMarketArtifact::from_manifest_artifact(&manifest, artifact);
    resolved.computed_hash = Some("sha256:artifact-hash".to_string());

    let report = QuantixRegressionReport::from_resolved_artifact(
        &resolved,
        QuantixRegressionContext {
            source_command: "quantix import market-manifest --verify-artifact-file".to_string(),
            run_at: "2026-05-18T00:00:00Z".to_string(),
            consumer_build_commit: "abc123".to_string(),
            database_target: "dry-run-only".to_string(),
            writes_performed: false,
        },
    );

    assert_eq!(report.schema_version, "quantix_regression_report.v1");
    assert_eq!(report.consumer_system, "quantix-rust");
    assert_eq!(report.dataset_version, "kline_daily_20260512_v1");
    assert_eq!(report.lineage_id, "lineage-20260512");
    assert_eq!(report.payload_hash, "dataset-hash");
    assert_eq!(report.artifact.hash, "artifact-hash");
    assert_eq!(
        report.artifact.computed_hash.as_deref(),
        Some("sha256:artifact-hash")
    );
    assert!(report.regression.passed);
    assert!(report.regression.failed_checks.is_empty());
    assert_eq!(report.consumer_build.repo, "quantix-rust");
    assert_eq!(report.consumer_build.commit, "abc123");
    assert_eq!(report.consumer_build.database_target, "dry-run-only");
    assert!(!report.consumer_build.writes_performed);
}

#[test]
fn quantix_regression_report_samples_symbols_and_dates_from_verified_parquet_payload() {
    let dir = tempfile::tempdir().unwrap();
    let artifact_path = dir.path().join("part-00000.parquet");
    write_sample_parquet(&artifact_path);
    let artifact_hash = sha256_file_for_test(&artifact_path);

    let mut manifest = valid_manifest();
    manifest.artifacts[0].uri = format!("file://{}", artifact_path.display());
    manifest.artifacts[0].hash = artifact_hash.clone();
    manifest.artifacts[0].row_count = 3;
    let selector = MarketArtifactSelector::new("parquet").require_schema_version("v1");
    let artifact = selector.select(&manifest).unwrap();
    let mut resolved = ResolvedMarketArtifact::from_manifest_artifact(&manifest, artifact);

    resolved
        .verify_artifact_file_hash(dir.path().join("manifest.json"))
        .unwrap();

    assert_eq!(resolved.sample_symbols, vec!["000001.SZ", "600000.SH"]);
    assert_eq!(resolved.sample_dates, vec!["2026-05-12", "2026-05-13"]);
    assert_eq!(resolved.computed_row_count, Some(3));

    let report = QuantixRegressionReport::from_resolved_artifact(
        &resolved,
        QuantixRegressionContext {
            source_command: "quantix import market-manifest --verify-artifact-file".to_string(),
            run_at: "2026-05-18T00:00:00Z".to_string(),
            consumer_build_commit: "abc123".to_string(),
            database_target: "dry-run-only".to_string(),
            writes_performed: false,
        },
    );

    assert_eq!(report.sample_symbols, vec!["000001.SZ", "600000.SH"]);
    assert_eq!(report.sample_dates, vec!["2026-05-12", "2026-05-13"]);
    assert_eq!(report.artifact.computed_row_count, Some(3));
    assert!(
        report
            .regression
            .checks
            .contains(&"artifact_payload_sampled".to_string())
    );
    assert!(
        report
            .regression
            .checks
            .contains(&"artifact_payload_row_count_verified".to_string())
    );
    assert!(
        !report
            .warnings
            .contains(&"artifact_payload_sampling_not_available".to_string())
    );
}

#[test]
fn quantix_regression_report_fails_closed_when_parquet_payload_row_count_differs() {
    let dir = tempfile::tempdir().unwrap();
    let artifact_path = dir.path().join("part-00000.parquet");
    write_sample_parquet(&artifact_path);
    let artifact_hash = sha256_file_for_test(&artifact_path);

    let mut manifest = valid_manifest();
    manifest.artifacts[0].uri = format!("file://{}", artifact_path.display());
    manifest.artifacts[0].hash = artifact_hash;
    manifest.artifacts[0].row_count = 4;
    let selector = MarketArtifactSelector::new("parquet").require_schema_version("v1");
    let artifact = selector.select(&manifest).unwrap();
    let mut resolved = ResolvedMarketArtifact::from_manifest_artifact(&manifest, artifact);

    resolved
        .verify_artifact_file_hash(dir.path().join("manifest.json"))
        .unwrap();

    let report = QuantixRegressionReport::from_resolved_artifact(
        &resolved,
        QuantixRegressionContext {
            source_command: "quantix import market-manifest --verify-artifact-file".to_string(),
            run_at: "2026-05-18T00:00:00Z".to_string(),
            consumer_build_commit: "abc123".to_string(),
            database_target: "dry-run-only".to_string(),
            writes_performed: false,
        },
    );

    assert_eq!(resolved.computed_row_count, Some(3));
    assert!(!report.regression.passed);
    assert_eq!(
        report.regression.failed_checks,
        vec!["artifact_payload_row_count_mismatch".to_string()]
    );
}

#[test]
fn quantix_regression_report_fails_closed_without_computed_artifact_hash() {
    let manifest = valid_manifest();
    let selector = MarketArtifactSelector::new("parquet").require_schema_version("v1");
    let artifact = selector.select(&manifest).unwrap();
    let resolved = ResolvedMarketArtifact::from_manifest_artifact(&manifest, artifact);

    let report = QuantixRegressionReport::from_resolved_artifact(
        &resolved,
        QuantixRegressionContext {
            source_command: "quantix import market-manifest".to_string(),
            run_at: "2026-05-18T00:00:00Z".to_string(),
            consumer_build_commit: "abc123".to_string(),
            database_target: "dry-run-only".to_string(),
            writes_performed: false,
        },
    );

    assert!(!report.regression.passed);
    assert_eq!(
        report.regression.failed_checks,
        vec!["artifact_file_hash_not_verified".to_string()]
    );
}

#[test]
fn controlled_persistence_policy_enforces_registered_phases() {
    let dry_run = ControlledPersistencePolicy::parse("dry-run-only").unwrap();
    assert_eq!(
        dry_run.validate_writes_performed(false).unwrap(),
        "dry_run_only_no_writes"
    );
    assert_eq!(
        dry_run.validate_writes_performed(true).unwrap_err(),
        "dry_run_only_must_not_write"
    );

    let shadow = ControlledPersistencePolicy::parse("clickhouse-shadow:miniqmt_stage").unwrap();
    assert_eq!(
        shadow.validate_writes_performed(true).unwrap(),
        "clickhouse_shadow_writes_explicit"
    );
    assert_eq!(
        shadow.validate_writes_performed(false).unwrap_err(),
        "clickhouse_shadow_requires_writes_performed"
    );

    let production =
        ControlledPersistencePolicy::parse("clickhouse-production:market_data").unwrap();
    assert_eq!(
        production.validate_writes_performed(true).unwrap_err(),
        "clickhouse_production_not_implemented"
    );

    assert_eq!(
        ControlledPersistencePolicy::parse("clickhouse-shadow:").unwrap_err(),
        "clickhouse_shadow_requires_table"
    );
    assert_eq!(
        ControlledPersistencePolicy::parse("warehouse:market_data").unwrap_err(),
        "unsupported_database_target"
    );
}

#[test]
fn quantix_regression_report_fails_closed_for_shadow_target_without_write_path() {
    let manifest = valid_manifest();
    let selector = MarketArtifactSelector::new("parquet").require_schema_version("v1");
    let artifact = selector.select(&manifest).unwrap();
    let mut resolved = ResolvedMarketArtifact::from_manifest_artifact(&manifest, artifact);
    resolved.computed_hash = Some("sha256:artifact-hash".to_string());

    let report = QuantixRegressionReport::from_resolved_artifact(
        &resolved,
        QuantixRegressionContext {
            source_command:
                "quantix import market-manifest --database-target clickhouse-shadow:miniqmt_stage"
                    .to_string(),
            run_at: "2026-05-18T00:00:00Z".to_string(),
            consumer_build_commit: "abc123".to_string(),
            database_target: "clickhouse-shadow:miniqmt_stage".to_string(),
            writes_performed: false,
        },
    );

    assert!(!report.regression.passed);
    assert_eq!(
        report.regression.failed_checks,
        vec!["clickhouse_shadow_requires_writes_performed".to_string()]
    );
    assert_eq!(
        report.consumer_build.database_target,
        "clickhouse-shadow:miniqmt_stage"
    );
    assert!(!report.consumer_build.writes_performed);
}

#[test]
fn quantix_regression_evidence_wraps_report_with_raw_report_reference() {
    let manifest = valid_manifest();
    let selector = MarketArtifactSelector::new("parquet").require_schema_version("v1");
    let artifact = selector.select(&manifest).unwrap();
    let mut resolved = ResolvedMarketArtifact::from_manifest_artifact(&manifest, artifact);
    resolved.computed_hash = Some("sha256:artifact-hash".to_string());
    let report = QuantixRegressionReport::from_resolved_artifact(
        &resolved,
        QuantixRegressionContext {
            source_command: "quantix import market-manifest --verify-artifact-file".to_string(),
            run_at: "2026-05-18T00:00:00Z".to_string(),
            consumer_build_commit: "abc123".to_string(),
            database_target: "dry-run-only".to_string(),
            writes_performed: false,
        },
    );

    let evidence = QuantixRegressionEvidence::from_report(
        &report,
        RawReportReference {
            path: "reports/quantix-regression.json".to_string(),
            hash: "sha256:raw-report-hash".to_string(),
            size_bytes: 2048,
        },
        "2026-05-18T00:00:01Z",
    )
    .unwrap();

    assert_eq!(evidence.schema_version, "evidence.v1");
    assert_eq!(evidence.environment.consumer_system, "quantix-rust");
    assert_eq!(evidence.environment.consumer_build, "abc123");
    assert_eq!(
        evidence.result_summary.evidence_type,
        "promotion_consumer_regression"
    );
    assert_eq!(
        evidence.result_summary.dataset_version,
        "kline_daily_20260512_v1"
    );
    assert_eq!(
        evidence.result_summary.raw_report.hash,
        "sha256:raw-report-hash"
    );
    assert!(evidence.result_summary.regression.passed);
    assert!(evidence.result_summary.regression.failed_checks.is_empty());
}
