//! Cross-cutting tests for the miniqmt_market facade.
//!
//! These tests exercise the composition of [`MarketArtifactRequest`] with the
//! manifest intake and artifact selector, plus regression report generation,
//! controlled persistence policy, and local artifact URI resolution. They live
//! in the parent module because they span multiple child modules.

use super::*;

fn published_manifest() -> MarketDatasetManifest {
    MarketDatasetManifest {
        dataset_version: "2026-05-24".to_string(),
        schema_version: "manifest.v1".to_string(),
        contract_profile: "market-data-platform-v1".to_string(),
        domain: "kline_daily".to_string(),
        maturity: "golden".to_string(),
        quality_status: "passed".to_string(),
        published: true,
        lineage_id: "lineage-20260524".to_string(),
        row_count: 2,
        payload_hash: "payload-hash".to_string(),
        rows_hash: Some("rows-hash".to_string()),
        sources: vec![ManifestSource {
            source_system: "miniqmt".to_string(),
            role: "source-of-truth".to_string(),
            source_version: Some("bridge.v1".to_string()),
        }],
        artifacts: vec![
            MarketDatasetArtifact {
                artifact_type: "parquet".to_string(),
                uri: "artifacts/kline_daily.parquet".to_string(),
                schema_version: "kline_daily.v1".to_string(),
                row_count: 2,
                hash: "sha256:abc123".to_string(),
                rows_hash: None,
            },
            MarketDatasetArtifact {
                artifact_type: "json-summary".to_string(),
                uri: "artifacts/kline_daily.summary.json".to_string(),
                schema_version: "summary.v1".to_string(),
                row_count: 1,
                hash: "sha256:def456".to_string(),
                rows_hash: Some("summary-rows-hash".to_string()),
            },
        ],
        quality: ManifestQuality {
            blocking_issues: Vec::new(),
            warnings: Vec::new(),
            gap_count: 0,
            conflict_count: 0,
        },
    }
}

#[test]
fn market_artifact_request_resolves_schema_and_hash_checked_artifact() {
    let manifest = published_manifest();
    let bytes = serde_json::to_vec(&manifest).unwrap();

    let resolved = MarketArtifactRequest::new("2026-05-24", "parquet")
        .require_schema_version("kline_daily.v1")
        .require_artifact_hash("sha256:abc123")
        .resolve_from_slice(&bytes)
        .unwrap();

    assert_eq!(resolved.dataset_version, "2026-05-24");
    assert_eq!(resolved.artifact_type, "parquet");
    assert_eq!(resolved.schema_version, "kline_daily.v1");
    assert_eq!(resolved.hash, "sha256:abc123");
    assert_eq!(resolved.rows_hash.as_deref(), Some("rows-hash"));
    assert_eq!(resolved.row_count, 2);
}

#[test]
fn manifest_validator_rejects_blocking_quality_before_artifact_selection() {
    let mut manifest = published_manifest();
    manifest.quality_status = "blocking".to_string();
    manifest
        .quality
        .blocking_issues
        .push("missing_trading_day".to_string());

    let err = ManifestValidator::new("2026-05-24")
        .validate(&manifest)
        .unwrap_err();

    assert!(err.contains("blocking quality issues"));
}

#[test]
fn artifact_selector_rejects_ambiguous_schema_matches() {
    let mut manifest = published_manifest();
    manifest.artifacts.push(manifest.artifacts[0].clone());

    let err = MarketArtifactSelector::new("parquet")
        .require_schema_version("kline_daily.v1")
        .select(&manifest)
        .unwrap_err();

    assert!(err.contains("multiple matching artifacts"));
}

#[test]
fn regression_report_records_dry_run_hash_and_payload_checks() {
    let manifest = published_manifest();
    let mut resolved =
        ResolvedMarketArtifact::from_manifest_artifact(&manifest, &manifest.artifacts[0]);
    resolved.computed_hash = Some("sha256:abc123".to_string());
    resolved.computed_row_count = Some(2);
    resolved.sample_symbols = vec!["000001.SZ".to_string()];
    resolved.sample_dates = vec!["2026-05-24".to_string()];

    let report = QuantixRegressionReport::from_resolved_artifact(
        &resolved,
        QuantixRegressionContext {
            source_command: "miniqmt export kline_daily".to_string(),
            run_at: "2026-05-24T08:00:00Z".to_string(),
            consumer_build_commit: "abc123".to_string(),
            database_target: "dry-run-only".to_string(),
            writes_performed: false,
        },
    );

    assert!(report.regression.passed);
    assert!(
        report
            .regression
            .checks
            .contains(&"artifact_file_hash_verified".to_string())
    );
    assert!(
        report
            .regression
            .checks
            .contains(&"artifact_payload_row_count_verified".to_string())
    );
    assert!(
        report
            .regression
            .checks
            .contains(&"dry_run_only_no_writes".to_string())
    );
    assert_eq!(
        report.regression.comparison_summary,
        "manifest_artifact_identity_and_payload_row_count"
    );
}

#[test]
fn controlled_persistence_policy_requires_explicit_shadow_writes() {
    let policy = ControlledPersistencePolicy::parse("clickhouse-shadow: market_shadow").unwrap();

    assert_eq!(
        policy.validate_writes_performed(false).unwrap_err(),
        "clickhouse_shadow_requires_writes_performed"
    );
    assert_eq!(
        policy.validate_writes_performed(true).unwrap(),
        "clickhouse_shadow_writes_explicit"
    );
}

#[test]
fn file_uri_local_path_candidates_decode_windows_drive_uri() {
    let candidates = super::sampling::file_uri_local_path_candidates(
        "file:///D:/MyCode3/miniQMT/bridge/logs/domain%3Dkline_daily/artifacts/format%3Dparquet/part-00000.parquet",
    )
    .unwrap();
    let rendered: Vec<String> = candidates
        .iter()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .collect();

    assert_eq!(
        rendered[0],
        "/mnt/d/MyCode3/miniQMT/bridge/logs/domain=kline_daily/artifacts/format=parquet/part-00000.parquet"
    );
    assert!(rendered.contains(
        &"/d/MyCode3/miniQMT/bridge/logs/domain=kline_daily/artifacts/format=parquet/part-00000.parquet"
            .to_string()
    ));
}
