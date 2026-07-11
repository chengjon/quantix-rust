//! Manifest validation and intake for miniQMT market datasets.

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::selection::{MarketArtifactSelector, ResolvedMarketArtifact};
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketDatasetManifest {
    pub dataset_version: String,
    pub schema_version: String,
    pub contract_profile: String,
    pub domain: String,
    pub maturity: String,
    pub quality_status: String,
    pub published: bool,
    pub lineage_id: String,
    pub row_count: u64,
    pub payload_hash: String,
    #[serde(default)]
    pub rows_hash: Option<String>,
    #[serde(default)]
    pub sources: Vec<ManifestSource>,
    #[serde(default)]
    pub artifacts: Vec<MarketDatasetArtifact>,
    pub quality: ManifestQuality,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestSource {
    pub source_system: String,
    pub role: String,
    pub source_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketDatasetArtifact {
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub uri: String,
    pub schema_version: String,
    pub row_count: u64,
    pub hash: String,
    #[serde(default)]
    pub rows_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestQuality {
    #[serde(default)]
    pub blocking_issues: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub gap_count: u64,
    pub conflict_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestValidator {
    expected_dataset_version: String,
    expected_artifact_hash: Option<String>,
}

impl ManifestValidator {
    /// 构造一个期望指定 dataset_version 的校验器；artifact hash 期望未设置时通过 require_artifact_hash 链式补充。
    pub fn new(expected_dataset_version: impl Into<String>) -> Self {
        Self {
            expected_dataset_version: expected_dataset_version.into(),
            expected_artifact_hash: None,
        }
    }

    /// Builder：要求 manifest 的 artifacts 列表中至少存在一个 hash 等于该值的 artifact；返回 self 便于链式构造。
    pub fn require_artifact_hash(mut self, expected_artifact_hash: impl Into<String>) -> Self {
        self.expected_artifact_hash = Some(expected_artifact_hash.into());
        self
    }

    /// 按合同/版本/published/lineage/payload_hash/quality/artifact hash 顺序逐项校验 manifest；任一不满足返回带具体原因的 Err。
    pub fn validate(&self, manifest: &MarketDatasetManifest) -> Result<(), String> {
        if manifest.contract_profile != "market-data-platform-v1" {
            return Err(format!(
                "unsupported contract_profile: {}",
                manifest.contract_profile
            ));
        }

        if manifest.dataset_version != self.expected_dataset_version {
            return Err(format!(
                "dataset_version mismatch: expected {}, got {}",
                self.expected_dataset_version, manifest.dataset_version
            ));
        }

        if !manifest.published {
            return Err(format!(
                "dataset {} is not published",
                manifest.dataset_version
            ));
        }

        if manifest.lineage_id.trim().is_empty() {
            return Err("lineage_id is required".to_string());
        }

        if manifest.payload_hash.trim().is_empty() {
            return Err("payload_hash is required".to_string());
        }

        if manifest.quality_status == "blocking" || !manifest.quality.blocking_issues.is_empty() {
            return Err(format!(
                "blocking quality issues for dataset {}",
                manifest.dataset_version
            ));
        }

        if manifest.artifacts.is_empty() {
            return Err(format!(
                "dataset {} has no published artifacts",
                manifest.dataset_version
            ));
        }

        if let Some(expected_hash) = &self.expected_artifact_hash {
            let matches_artifact = manifest
                .artifacts
                .iter()
                .any(|artifact| artifact.hash == *expected_hash);

            if !matches_artifact {
                return Err(format!(
                    "artifact hash mismatch for dataset {}",
                    manifest.dataset_version
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestIntake {
    validator: ManifestValidator,
}

impl ManifestIntake {
    /// 以给定校验器构造 intake；后续 load_*/resolve_artifact_* 都会复用该校验器。
    pub fn new(validator: ManifestValidator) -> Self {
        Self { validator }
    }

    /// 从内存 bytes 解析 JSON manifest 并执行 validator.validate；JSON 解析失败或校验失败均返回带原因的 Err。
    pub fn load_from_slice(&self, bytes: &[u8]) -> Result<MarketDatasetManifest, String> {
        load_manifest_from_slice(bytes, &self.validator)
    }

    /// 从磁盘读取并解析 manifest 文件，再委托给 load_from_slice 完成校验；读取或解析失败均返回带路径信息的 Err。
    pub fn load_from_path(&self, path: impl AsRef<Path>) -> Result<MarketDatasetManifest, String> {
        load_manifest_from_path(path, &self.validator)
    }

    /// 在 slice manifest 上应用 selector.select 解析目标 artifact，并组装为 ResolvedMarketArtifact；selector 失败或 load 失败均向上透传。
    pub fn resolve_artifact_from_slice(
        &self,
        bytes: &[u8],
        selector: &MarketArtifactSelector,
    ) -> Result<ResolvedMarketArtifact, String> {
        let manifest = self.load_from_slice(bytes)?;
        let artifact = selector.select(&manifest)?;

        Ok(ResolvedMarketArtifact::from_manifest_artifact(
            &manifest, artifact,
        ))
    }

    /// 等价于 resolve_artifact_from_slice，但 manifest 从给定文件路径加载；路径读取、manifest 校验、selector 哪一步失败都会返回 Err。
    pub fn resolve_artifact_from_path(
        &self,
        path: impl AsRef<Path>,
        selector: &MarketArtifactSelector,
    ) -> Result<ResolvedMarketArtifact, String> {
        let manifest = self.load_from_path(path)?;
        let artifact = selector.select(&manifest)?;

        Ok(ResolvedMarketArtifact::from_manifest_artifact(
            &manifest, artifact,
        ))
    }
}

/// 从内存 bytes 反序列化 manifest 并用 validator 校验；JSON 解析失败或校验失败均返回带原因的 Err（不写入磁盘）。
pub fn load_manifest_from_slice(
    bytes: &[u8],
    validator: &ManifestValidator,
) -> Result<MarketDatasetManifest, String> {
    let manifest: MarketDatasetManifest =
        serde_json::from_slice(bytes).map_err(|err| format!("invalid manifest json: {err}"))?;

    validator.validate(&manifest)?;

    Ok(manifest)
}

/// 从磁盘读取 manifest 文件，再委托给 load_manifest_from_slice；读取失败带路径信息透传。
pub fn load_manifest_from_path(
    path: impl AsRef<Path>,
    validator: &ManifestValidator,
) -> Result<MarketDatasetManifest, String> {
    let path = path.as_ref();
    let bytes = fs::read(path)
        .map_err(|err| format!("failed to read manifest {}: {err}", path.display()))?;

    load_manifest_from_slice(&bytes, validator)
}
