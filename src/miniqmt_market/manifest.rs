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
    pub fn new(expected_dataset_version: impl Into<String>) -> Self {
        Self {
            expected_dataset_version: expected_dataset_version.into(),
            expected_artifact_hash: None,
        }
    }

    pub fn require_artifact_hash(mut self, expected_artifact_hash: impl Into<String>) -> Self {
        self.expected_artifact_hash = Some(expected_artifact_hash.into());
        self
    }

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
    pub fn new(validator: ManifestValidator) -> Self {
        Self { validator }
    }

    pub fn load_from_slice(&self, bytes: &[u8]) -> Result<MarketDatasetManifest, String> {
        load_manifest_from_slice(bytes, &self.validator)
    }

    pub fn load_from_path(&self, path: impl AsRef<Path>) -> Result<MarketDatasetManifest, String> {
        load_manifest_from_path(path, &self.validator)
    }

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

pub fn load_manifest_from_slice(
    bytes: &[u8],
    validator: &ManifestValidator,
) -> Result<MarketDatasetManifest, String> {
    let manifest: MarketDatasetManifest =
        serde_json::from_slice(bytes).map_err(|err| format!("invalid manifest json: {err}"))?;

    validator.validate(&manifest)?;

    Ok(manifest)
}

pub fn load_manifest_from_path(
    path: impl AsRef<Path>,
    validator: &ManifestValidator,
) -> Result<MarketDatasetManifest, String> {
    let path = path.as_ref();
    let bytes = fs::read(path)
        .map_err(|err| format!("failed to read manifest {}: {err}", path.display()))?;

    load_manifest_from_slice(&bytes, validator)
}
