use super::*;
use quantix_cli::miniqmt_market::QuantixRegressionContext;
use tempfile::tempdir;

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
    let dir = tempdir().unwrap();
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
    let dir = tempdir().unwrap();
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
