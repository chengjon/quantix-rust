use super::*;

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
