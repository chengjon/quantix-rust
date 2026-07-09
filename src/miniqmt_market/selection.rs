//! Artifact selection, local artifact resolution, controlled persistence policy.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::manifest::{MarketDatasetArtifact, MarketDatasetManifest};
use super::sampling::{
    file_uri_local_path_candidates, format_sha256_hash_like_expected, normalize_sha256_hash,
    sample_artifact_payload, sha256_file,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketArtifactSelector {
    artifact_type: String,
    schema_version: Option<String>,
}

impl MarketArtifactSelector {
    pub fn new(artifact_type: impl Into<String>) -> Self {
        Self {
            artifact_type: artifact_type.into(),
            schema_version: None,
        }
    }

    pub fn require_schema_version(mut self, schema_version: impl Into<String>) -> Self {
        self.schema_version = Some(schema_version.into());
        self
    }

    pub fn select<'a>(
        &self,
        manifest: &'a MarketDatasetManifest,
    ) -> Result<&'a MarketDatasetArtifact, String> {
        let matches: Vec<_> = manifest
            .artifacts
            .iter()
            .filter(|artifact| artifact.artifact_type == self.artifact_type)
            .filter(|artifact| match &self.schema_version {
                Some(schema_version) => artifact.schema_version == *schema_version,
                None => true,
            })
            .collect();

        match matches.as_slice() {
            [artifact] => Ok(*artifact),
            [] => Err(format!(
                "no matching artifact for type {} in dataset {}",
                self.artifact_type, manifest.dataset_version
            )),
            _ => Err(format!(
                "multiple matching artifacts for type {} in dataset {}",
                self.artifact_type, manifest.dataset_version
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedMarketArtifact {
    pub dataset_version: String,
    pub domain: String,
    pub lineage_id: String,
    pub payload_hash: String,
    pub maturity: String,
    pub quality_status: String,
    pub artifact_type: String,
    pub schema_version: String,
    pub uri: String,
    pub hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub computed_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub computed_row_count: Option<u64>,
    pub row_count: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sample_symbols: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sample_dates: Vec<String>,
}

impl ResolvedMarketArtifact {
    pub fn from_manifest_artifact(
        manifest: &MarketDatasetManifest,
        artifact: &MarketDatasetArtifact,
    ) -> Self {
        Self {
            dataset_version: manifest.dataset_version.clone(),
            domain: manifest.domain.clone(),
            lineage_id: manifest.lineage_id.clone(),
            payload_hash: manifest.payload_hash.clone(),
            maturity: manifest.maturity.clone(),
            quality_status: manifest.quality_status.clone(),
            artifact_type: artifact.artifact_type.clone(),
            schema_version: artifact.schema_version.clone(),
            uri: artifact.uri.clone(),
            hash: artifact.hash.clone(),
            rows_hash: artifact
                .rows_hash
                .clone()
                .or_else(|| manifest.rows_hash.clone()),
            computed_hash: None,
            computed_row_count: None,
            row_count: artifact.row_count,
            sample_symbols: Vec::new(),
            sample_dates: Vec::new(),
        }
    }

    pub fn to_pretty_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|err| format!("failed to serialize resolved market artifact: {err}"))
    }

    pub fn verify_artifact_file_hash(
        &mut self,
        manifest_path: impl AsRef<Path>,
    ) -> Result<(), String> {
        let artifact_path = self.resolve_local_artifact_path(manifest_path.as_ref())?;
        let computed_hash = sha256_file(&artifact_path)?;
        let expected_hash = normalize_sha256_hash(&self.hash);

        if !computed_hash.eq_ignore_ascii_case(expected_hash) {
            return Err(format!(
                "artifact file hash mismatch for {}: expected {}, got sha256:{}",
                self.uri, self.hash, computed_hash
            ));
        }

        self.computed_hash = Some(format_sha256_hash_like_expected(&self.hash, &computed_hash));
        if let Ok(sample) = sample_artifact_payload(&artifact_path, &self.artifact_type) {
            self.computed_row_count = sample.computed_row_count;
            self.sample_symbols = sample.sample_symbols;
            self.sample_dates = sample.sample_dates;
        }
        Ok(())
    }

    fn resolve_local_artifact_path(&self, manifest_path: &Path) -> Result<PathBuf, String> {
        if self.uri.starts_with("file://") {
            let candidates = file_uri_local_path_candidates(&self.uri)?;
            if let Some(existing_path) = candidates.iter().find(|path| path.exists()) {
                return Ok(existing_path.clone());
            }
            return candidates.into_iter().next().ok_or_else(|| {
                "artifact file uri did not produce local path candidates".to_string()
            });
        }

        if self.uri.contains("://") {
            return Err(format!(
                "unsupported artifact uri scheme for content hash verification: {}",
                self.uri
            ));
        }

        let path = PathBuf::from(&self.uri);
        if path.is_absolute() {
            return Ok(path);
        }

        let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
        Ok(manifest_dir.join(path))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlledPersistencePolicy {
    DryRunOnly,
    ClickHouseShadow { table: String },
    ClickHouseProduction { table: String },
}

impl ControlledPersistencePolicy {
    pub fn parse(database_target: &str) -> Result<Self, String> {
        if database_target == "dry-run-only" {
            return Ok(Self::DryRunOnly);
        }

        if let Some(table) = database_target.strip_prefix("clickhouse-shadow:") {
            let table = table.trim();
            if table.is_empty() {
                return Err("clickhouse_shadow_requires_table".to_string());
            }
            return Ok(Self::ClickHouseShadow {
                table: table.to_string(),
            });
        }

        if let Some(table) = database_target.strip_prefix("clickhouse-production:") {
            let table = table.trim();
            if table.is_empty() {
                return Err("clickhouse_production_requires_table".to_string());
            }
            return Ok(Self::ClickHouseProduction {
                table: table.to_string(),
            });
        }

        Err("unsupported_database_target".to_string())
    }

    pub fn validate_writes_performed(
        &self,
        writes_performed: bool,
    ) -> Result<&'static str, String> {
        match self {
            Self::DryRunOnly if writes_performed => Err("dry_run_only_must_not_write".to_string()),
            Self::DryRunOnly => Ok("dry_run_only_no_writes"),
            Self::ClickHouseShadow { .. } if writes_performed => {
                Ok("clickhouse_shadow_writes_explicit")
            }
            Self::ClickHouseShadow { .. } => {
                Err("clickhouse_shadow_requires_writes_performed".to_string())
            }
            Self::ClickHouseProduction { .. } => {
                Err("clickhouse_production_not_implemented".to_string())
            }
        }
    }
}
