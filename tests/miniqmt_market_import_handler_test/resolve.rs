use super::*;
use quantix_cli::cli::handlers::{
    resolve_import_market_manifest_artifact, resolve_import_market_manifest_artifact_with_options,
};

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
