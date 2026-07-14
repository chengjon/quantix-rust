use super::*;

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
