//! miniQMT market artifact contract boundary.
//!
//! This module is the public facade for miniQMT market dataset intake. Its
//! owned responsibilities are split across sibling modules:
//!
//! - [`manifest`] — manifest validation and intake;
//! - [`selection`] — artifact selection, local artifact resolution,
//!   controlled persistence policy;
//! - [`regression`] — Quantix regression report and evidence generation;
//! - [`sampling`] — local payload sampling and hash verification helpers;
//! - [`request`] (re-exported as [`MarketArtifactRequest`]) — public request
//!   facade.
//!
//! All types are re-exported here so callers can keep using
//! `crate::miniqmt_market::<Type>` paths unchanged.

pub mod manifest;
pub mod regression;
pub mod sampling;
pub mod selection;

#[cfg(test)]
mod tests_facade;
#[cfg(test)]
mod tests_manifest;
#[cfg(test)]
mod tests_regression;
#[cfg(test)]
mod tests_sampling;
#[cfg(test)]
mod tests_selection;

pub use manifest::{
    ManifestIntake, ManifestQuality, ManifestSource, ManifestValidator, MarketDatasetArtifact,
    MarketDatasetManifest,
};
pub use regression::{
    QuantixClickHouseReadOnlySummary, QuantixConsumerBuild, QuantixEvidenceEnvironment,
    QuantixEvidenceResultSummary, QuantixRegressionArtifact, QuantixRegressionComparison,
    QuantixRegressionContext, QuantixRegressionEvidence, QuantixRegressionReport,
    QuantixRegressionStatus, QuantixSourceOfTruthSummary, RawReportReference,
};
pub use sampling::ArtifactPayloadSample;
pub use selection::{ControlledPersistencePolicy, MarketArtifactSelector, ResolvedMarketArtifact};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantixRegressionContext {
    pub source_command: String,
    pub run_at: String,
    pub consumer_build_commit: String,
    pub database_target: String,
    pub writes_performed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantixRegressionReport {
    pub schema_version: String,
    pub source_command: String,
    pub run_at: String,
    pub consumer_system: String,
    pub dataset_version: String,
    pub lineage_id: String,
    pub payload_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows_hash: Option<String>,
    pub artifact: QuantixRegressionArtifact,
    pub row_count: u64,
    pub sample_symbols: Vec<String>,
    pub sample_dates: Vec<String>,
    pub regression: QuantixRegressionStatus,
    pub consumer_build: QuantixConsumerBuild,
    pub redaction_notes: Vec<String>,
    pub warnings: Vec<String>,
    pub generated_at: String,
}

impl QuantixRegressionReport {
    pub fn from_resolved_artifact(
        resolved: &ResolvedMarketArtifact,
        context: QuantixRegressionContext,
    ) -> Self {
        Self::from_resolved_artifact_with_comparison(resolved, context, None)
    }

    pub fn from_resolved_artifact_with_comparison(
        resolved: &ResolvedMarketArtifact,
        context: QuantixRegressionContext,
        comparison: Option<QuantixRegressionComparison>,
    ) -> Self {
        let mut checks = vec![
            "manifest_identity_validated".to_string(),
            "artifact_manifest_hash_matched".to_string(),
        ];
        let mut failed_checks = Vec::new();

        if resolved.computed_hash.is_some() {
            checks.push("artifact_file_hash_verified".to_string());
        } else {
            failed_checks.push("artifact_file_hash_not_verified".to_string());
        }

        match ControlledPersistencePolicy::parse(&context.database_target) {
            Ok(policy) => match policy.validate_writes_performed(context.writes_performed) {
                Ok(check) => checks.push(check.to_string()),
                Err(check) => failed_checks.push(check),
            },
            Err(check) => failed_checks.push(check),
        }

        let payload_sampled =
            !resolved.sample_symbols.is_empty() || !resolved.sample_dates.is_empty();
        if payload_sampled {
            checks.push("artifact_payload_sampled".to_string());
        }

        let payload_row_count_verified = match resolved.computed_row_count {
            Some(computed_row_count) if computed_row_count == resolved.row_count => {
                checks.push("artifact_payload_row_count_verified".to_string());
                true
            }
            Some(_) => {
                failed_checks.push("artifact_payload_row_count_mismatch".to_string());
                false
            }
            None => false,
        };

        let mut warnings = vec!["double_read_comparison_not_yet_implemented".to_string()];
        if !payload_sampled {
            warnings.push("artifact_payload_sampling_not_available".to_string());
        }
        let mut comparison_summary = if payload_row_count_verified {
            "manifest_artifact_identity_and_payload_row_count".to_string()
        } else {
            "manifest_artifact_identity_only".to_string()
        };
        if let Some(comparison) = &comparison {
            warnings.retain(|warning| warning != "double_read_comparison_not_yet_implemented");
            checks.push("double_read_comparison_performed".to_string());
            if comparison.row_count_matched {
                checks.push("double_read_row_count_matched".to_string());
            } else {
                failed_checks.push("double_read_row_count_mismatch".to_string());
            }
            if comparison.sample_symbols_matched {
                checks.push("double_read_sample_symbols_matched".to_string());
            } else {
                failed_checks.push("double_read_sample_symbols_mismatch".to_string());
            }
            if comparison.sample_dates_matched {
                checks.push("double_read_sample_dates_matched".to_string());
            } else {
                failed_checks.push("double_read_sample_dates_mismatch".to_string());
            }
            comparison_summary = if comparison.row_count_matched
                && comparison.sample_symbols_matched
                && comparison.sample_dates_matched
            {
                comparison_success_summary(&comparison.comparison_type).to_string()
            } else {
                comparison_failure_summary(&comparison.comparison_type).to_string()
            };
        }

        Self {
            schema_version: "quantix_regression_report.v1".to_string(),
            source_command: context.source_command,
            run_at: context.run_at.clone(),
            consumer_system: "quantix-rust".to_string(),
            dataset_version: resolved.dataset_version.clone(),
            lineage_id: resolved.lineage_id.clone(),
            payload_hash: resolved.payload_hash.clone(),
            rows_hash: resolved.rows_hash.clone(),
            artifact: QuantixRegressionArtifact {
                artifact_type: resolved.artifact_type.clone(),
                uri: resolved.uri.clone(),
                schema_version: resolved.schema_version.clone(),
                row_count: resolved.row_count,
                hash: resolved.hash.clone(),
                computed_hash: resolved.computed_hash.clone(),
                computed_row_count: resolved.computed_row_count,
                rows_hash: resolved.rows_hash.clone(),
            },
            row_count: resolved.row_count,
            sample_symbols: resolved.sample_symbols.clone(),
            sample_dates: resolved.sample_dates.clone(),
            regression: QuantixRegressionStatus {
                passed: failed_checks.is_empty(),
                failed_checks,
                checks,
                comparison_summary,
                comparison,
            },
            consumer_build: QuantixConsumerBuild {
                repo: "quantix-rust".to_string(),
                commit: context.consumer_build_commit,
                database_target: context.database_target,
                writes_performed: context.writes_performed,
            },
            redaction_notes: vec!["no_sensitive_payload_included".to_string()],
            warnings,
            generated_at: context.run_at,
        }
    }

    pub fn to_pretty_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|err| format!("failed to serialize quantix regression report: {err}"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantixRegressionArtifact {
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub uri: String,
    pub schema_version: String,
    pub row_count: u64,
    pub hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub computed_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub computed_row_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantixRegressionStatus {
    pub passed: bool,
    pub failed_checks: Vec<String>,
    pub checks: Vec<String>,
    pub comparison_summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison: Option<QuantixRegressionComparison>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantixRegressionComparison {
    pub comparison_type: String,
    pub reference_uri: String,
    pub reference_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_source_system: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_source_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_row_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_row_count: Option<u64>,
    pub row_count_matched: bool,
    pub sample_symbols_matched: bool,
    pub sample_dates_matched: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_sample_symbols: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_sample_dates: Vec<String>,
}

impl QuantixRegressionComparison {
    pub fn from_local_reference_artifact(
        resolved: &ResolvedMarketArtifact,
        reference_artifact: impl AsRef<str>,
    ) -> Result<Self, String> {
        let reference_artifact = reference_artifact.as_ref();
        let reference_path = resolve_reference_local_artifact_path(reference_artifact)?;
        let reference_hash = format!("sha256:{}", sha256_file(&reference_path)?);
        let reference_sample = sample_artifact_payload(&reference_path, &resolved.artifact_type)?;

        Ok(Self {
            comparison_type: "local_reference_artifact".to_string(),
            reference_uri: reference_artifact.to_string(),
            reference_hash,
            reference_source_system: None,
            reference_source_uri: None,
            target_row_count: resolved.computed_row_count,
            reference_row_count: reference_sample.computed_row_count,
            row_count_matched: resolved.computed_row_count == reference_sample.computed_row_count,
            sample_symbols_matched: resolved.sample_symbols == reference_sample.sample_symbols,
            sample_dates_matched: resolved.sample_dates == reference_sample.sample_dates,
            reference_sample_symbols: reference_sample.sample_symbols,
            reference_sample_dates: reference_sample.sample_dates,
        })
    }

    pub fn from_source_of_truth_summary(
        resolved: &ResolvedMarketArtifact,
        source_of_truth_summary: impl AsRef<str>,
    ) -> Result<Self, String> {
        let source_of_truth_summary = source_of_truth_summary.as_ref();
        let summary_path = resolve_source_of_truth_summary_path(source_of_truth_summary)?;
        let summary_bytes = fs::read(&summary_path).map_err(|err| {
            format!(
                "failed to read source-of-truth summary {}: {err}",
                summary_path.display()
            )
        })?;
        let summary: QuantixSourceOfTruthSummary =
            serde_json::from_slice(&summary_bytes).map_err(|err| {
                format!(
                    "invalid source-of-truth summary json {}: {err}",
                    summary_path.display()
                )
            })?;

        if summary.dataset_version != resolved.dataset_version {
            return Err(format!(
                "source_of_truth_dataset_version_mismatch: expected {}, got {}",
                resolved.dataset_version, summary.dataset_version
            ));
        }
        if let Some(lineage_id) = &summary.lineage_id
            && lineage_id != &resolved.lineage_id
        {
            return Err(format!(
                "source_of_truth_lineage_id_mismatch: expected {}, got {}",
                resolved.lineage_id, lineage_id
            ));
        }
        if let Some(payload_hash) = &summary.payload_hash
            && payload_hash != &resolved.payload_hash
        {
            return Err(format!(
                "source_of_truth_payload_hash_mismatch: expected {}, got {}",
                resolved.payload_hash, payload_hash
            ));
        }

        let target_row_count = resolved.computed_row_count.or(Some(resolved.row_count));
        let reference_hash = format!("sha256:{}", sha256_file(&summary_path)?);
        let row_count_matched = target_row_count == Some(summary.row_count);
        let sample_symbols_matched = !resolved.sample_symbols.is_empty()
            && resolved.sample_symbols == summary.sample_symbols;
        let sample_dates_matched =
            !resolved.sample_dates.is_empty() && resolved.sample_dates == summary.sample_dates;

        Ok(Self {
            comparison_type: "source_of_truth_summary".to_string(),
            reference_uri: source_of_truth_summary.to_string(),
            reference_hash,
            reference_source_system: Some(summary.source_system),
            reference_source_uri: Some(summary.source_uri),
            target_row_count,
            reference_row_count: Some(summary.row_count),
            row_count_matched,
            sample_symbols_matched,
            sample_dates_matched,
            reference_sample_symbols: summary.sample_symbols,
            reference_sample_dates: summary.sample_dates,
        })
    }

    pub fn from_clickhouse_read_only_summary(
        resolved: &ResolvedMarketArtifact,
        summary: QuantixClickHouseReadOnlySummary,
    ) -> Result<Self, String> {
        if summary.dataset_version != resolved.dataset_version {
            return Err(format!(
                "clickhouse_dataset_version_mismatch: expected {}, got {}",
                resolved.dataset_version, summary.dataset_version
            ));
        }

        let target_row_count = resolved.computed_row_count.or(Some(resolved.row_count));
        let row_count_matched = target_row_count == Some(summary.row_count);
        let sample_symbols_matched = !resolved.sample_symbols.is_empty()
            && resolved.sample_symbols == summary.sample_symbols;
        let sample_dates_matched =
            !resolved.sample_dates.is_empty() && resolved.sample_dates == summary.sample_dates;
        let reference_source_uri = summary.reference_source_uri();
        let reference_hash = summary.reference_hash()?;

        Ok(Self {
            comparison_type: "direct_clickhouse_read_only".to_string(),
            reference_uri: reference_source_uri.clone(),
            reference_hash,
            reference_source_system: Some("clickhouse".to_string()),
            reference_source_uri: Some(reference_source_uri),
            target_row_count,
            reference_row_count: Some(summary.row_count),
            row_count_matched,
            sample_symbols_matched,
            sample_dates_matched,
            reference_sample_symbols: summary.sample_symbols,
            reference_sample_dates: summary.sample_dates,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantixSourceOfTruthSummary {
    pub source_system: String,
    pub source_uri: String,
    pub dataset_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_hash: Option<String>,
    pub row_count: u64,
    #[serde(default)]
    pub sample_symbols: Vec<String>,
    #[serde(default)]
    pub sample_dates: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantixClickHouseReadOnlySummary {
    pub database: String,
    pub table: String,
    pub dataset_version: String,
    pub row_count: u64,
    #[serde(default)]
    pub sample_symbols: Vec<String>,
    #[serde(default)]
    pub sample_dates: Vec<String>,
}

impl QuantixClickHouseReadOnlySummary {
    pub fn reference_source_uri(&self) -> String {
        format!(
            "clickhouse://{}.{}?dataset_version={}",
            self.database, self.table, self.dataset_version
        )
    }

    fn reference_hash(&self) -> Result<String, String> {
        let bytes = serde_json::to_vec(self)
            .map_err(|err| format!("failed to serialize clickhouse summary: {err}"))?;
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Ok(format!("sha256:{:x}", hasher.finalize()))
    }
}

fn comparison_success_summary(comparison_type: &str) -> &'static str {
    match comparison_type {
        "direct_clickhouse_read_only" => "direct_clickhouse_read_only_matched",
        "source_of_truth_summary" => "source_of_truth_summary_matched",
        _ => "local_reference_artifact_matched",
    }
}

fn comparison_failure_summary(comparison_type: &str) -> &'static str {
    match comparison_type {
        "direct_clickhouse_read_only" => "direct_clickhouse_read_only_mismatch",
        "source_of_truth_summary" => "source_of_truth_summary_mismatch",
        _ => "local_reference_artifact_mismatch",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantixConsumerBuild {
    pub repo: String,
    pub commit: String,
    pub database_target: String,
    pub writes_performed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawReportReference {
    pub path: String,
    pub hash: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantixRegressionEvidence {
    pub schema_version: String,
    pub source_command: String,
    pub run_at: String,
    pub environment: QuantixEvidenceEnvironment,
    pub result_summary: QuantixEvidenceResultSummary,
}

impl QuantixRegressionEvidence {
    pub fn from_report(
        report: &QuantixRegressionReport,
        raw_report: RawReportReference,
        generated_at: impl Into<String>,
    ) -> Result<Self, String> {
        if !report.regression.passed || !report.regression.failed_checks.is_empty() {
            return Err(format!(
                "cannot generate passed controlled evidence from failed regression report: {:?}",
                report.regression.failed_checks
            ));
        }

        let generated_at = generated_at.into();
        Ok(Self {
            schema_version: "evidence.v1".to_string(),
            source_command: report.source_command.clone(),
            run_at: report.run_at.clone(),
            environment: QuantixEvidenceEnvironment {
                consumer_system: report.consumer_system.clone(),
                consumer_build: report.consumer_build.commit.clone(),
            },
            result_summary: QuantixEvidenceResultSummary {
                evidence_type: "promotion_consumer_regression".to_string(),
                consumer_system: report.consumer_system.clone(),
                dataset_version: report.dataset_version.clone(),
                lineage_id: report.lineage_id.clone(),
                payload_hash: report.payload_hash.clone(),
                rows_hash: report.rows_hash.clone(),
                artifact: report.artifact.clone(),
                regression: report.regression.clone(),
                row_count: report.row_count,
                sample_symbols: report.sample_symbols.clone(),
                sample_dates: report.sample_dates.clone(),
                consumer_build: report.consumer_build.clone(),
                raw_report,
                warnings: report.warnings.clone(),
                redaction_notes: report.redaction_notes.clone(),
                generated_at,
            },
        })
    }

    pub fn to_pretty_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|err| format!("failed to serialize quantix regression evidence: {err}"))
    }
}

pub fn raw_report_reference(path: impl AsRef<Path>) -> Result<RawReportReference, String> {
    let path = path.as_ref();
    let metadata = fs::metadata(path).map_err(|err| {
        format!(
            "failed to stat raw regression report {}: {err}",
            path.display()
        )
    })?;
    let hash = sha256_file(path)?;

    Ok(RawReportReference {
        path: path.to_string_lossy().to_string(),
        hash: format!("sha256:{hash}"),
        size_bytes: metadata.len(),
    })
}

fn resolve_reference_local_artifact_path(reference_artifact: &str) -> Result<PathBuf, String> {
    if reference_artifact.starts_with("file://") {
        let candidates = file_uri_local_path_candidates(reference_artifact)?;
        if let Some(existing_path) = candidates.iter().find(|path| path.exists()) {
            return Ok(existing_path.clone());
        }
        return candidates.into_iter().next().ok_or_else(|| {
            "reference artifact file uri did not produce local path candidates".to_string()
        });
    }

    if reference_artifact.contains("://") {
        return Err(format!(
            "unsupported reference artifact uri scheme: {reference_artifact}"
        ));
    }

    Ok(PathBuf::from(reference_artifact))
}

fn resolve_source_of_truth_summary_path(source_of_truth_summary: &str) -> Result<PathBuf, String> {
    if source_of_truth_summary.starts_with("file://") {
        let candidates = file_uri_local_path_candidates(source_of_truth_summary)?;
        if let Some(existing_path) = candidates.iter().find(|path| path.exists()) {
            return Ok(existing_path.clone());
        }
        return candidates.into_iter().next().ok_or_else(|| {
            "source-of-truth summary file uri did not produce local path candidates".to_string()
        });
    }

    if source_of_truth_summary.contains("://") {
        return Err(format!(
            "unsupported source-of-truth summary uri scheme: {source_of_truth_summary}"
        ));
    }

    Ok(PathBuf::from(source_of_truth_summary))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantixEvidenceEnvironment {
    pub consumer_system: String,
    pub consumer_build: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantixEvidenceResultSummary {
    pub evidence_type: String,
    pub consumer_system: String,
    pub dataset_version: String,
    pub lineage_id: String,
    pub payload_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows_hash: Option<String>,
    pub artifact: QuantixRegressionArtifact,
    pub regression: QuantixRegressionStatus,
    pub row_count: u64,
    pub sample_symbols: Vec<String>,
    pub sample_dates: Vec<String>,
    pub consumer_build: QuantixConsumerBuild,
    pub raw_report: RawReportReference,
    pub warnings: Vec<String>,
    pub redaction_notes: Vec<String>,
    pub generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ArtifactPayloadSample {
    computed_row_count: Option<u64>,
    sample_symbols: Vec<String>,
    sample_dates: Vec<String>,
}

const MAX_ARTIFACT_SAMPLE_VALUES: usize = 5;

fn sample_artifact_payload(
    path: &Path,
    artifact_type: &str,
) -> Result<ArtifactPayloadSample, String> {
    match artifact_type {
        "parquet" => sample_parquet_payload(path),
        other => Err(format!(
            "artifact payload sampling unsupported for type {other}"
        )),
    }
}

fn sample_parquet_payload(path: &Path) -> Result<ArtifactPayloadSample, String> {
    use arrow::array::{Array, LargeStringArray, PrimitiveArray, StringArray};
    use arrow::datatypes::Date32Type;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

    let file = fs::File::open(path)
        .map_err(|err| format!("failed to open parquet artifact {}: {err}", path.display()))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|err| format!("failed to create parquet reader {}: {err}", path.display()))?;
    let computed_row_count = Some(
        u64::try_from(builder.metadata().file_metadata().num_rows()).map_err(|err| {
            format!(
                "failed to read parquet row count from metadata {}: {err}",
                path.display()
            )
        })?,
    );
    let mut reader = builder
        .build()
        .map_err(|err| format!("failed to build parquet reader {}: {err}", path.display()))?;

    let mut sample = ArtifactPayloadSample {
        computed_row_count,
        ..ArtifactPayloadSample::default()
    };
    while !artifact_sample_is_full(&sample) {
        let Some(batch) = reader.next() else {
            break;
        };
        let batch = batch
            .map_err(|err| format!("failed to read parquet batch {}: {err}", path.display()))?;

        if let Some(symbols) = batch
            .column_by_name("symbol")
            .or_else(|| batch.column_by_name("code"))
            .or_else(|| batch.column_by_name("ts_code"))
            .or_else(|| batch.column_by_name("ticker"))
        {
            if let Some(values) = symbols.as_any().downcast_ref::<StringArray>() {
                collect_string_samples(values, &mut sample.sample_symbols);
            } else if let Some(values) = symbols.as_any().downcast_ref::<LargeStringArray>() {
                collect_large_string_samples(values, &mut sample.sample_symbols);
            }
        }

        if let Some(dates) = batch
            .column_by_name("date")
            .or_else(|| batch.column_by_name("trade_date"))
            .or_else(|| batch.column_by_name("datetime"))
            .or_else(|| batch.column_by_name("timestamp"))
        {
            if let Some(values) = dates.as_any().downcast_ref::<PrimitiveArray<Date32Type>>() {
                collect_date32_samples(values, &mut sample.sample_dates);
            } else if let Some(values) = dates.as_any().downcast_ref::<StringArray>() {
                collect_string_samples(values, &mut sample.sample_dates);
            } else if let Some(values) = dates.as_any().downcast_ref::<LargeStringArray>() {
                collect_large_string_samples(values, &mut sample.sample_dates);
            }
        }
    }

    Ok(sample)
}

fn artifact_sample_is_full(sample: &ArtifactPayloadSample) -> bool {
    sample.sample_symbols.len() >= MAX_ARTIFACT_SAMPLE_VALUES
        && sample.sample_dates.len() >= MAX_ARTIFACT_SAMPLE_VALUES
}

fn collect_string_samples(values: &arrow::array::StringArray, output: &mut Vec<String>) {
    use arrow::array::Array;

    for row in 0..values.len() {
        if values.is_null(row) {
            continue;
        }
        push_unique_sample(output, values.value(row));
        if output.len() >= MAX_ARTIFACT_SAMPLE_VALUES {
            break;
        }
    }
}

fn collect_large_string_samples(values: &arrow::array::LargeStringArray, output: &mut Vec<String>) {
    use arrow::array::Array;

    for row in 0..values.len() {
        if values.is_null(row) {
            continue;
        }
        push_unique_sample(output, values.value(row));
        if output.len() >= MAX_ARTIFACT_SAMPLE_VALUES {
            break;
        }
    }
}

fn collect_date32_samples(
    values: &arrow::array::PrimitiveArray<arrow::datatypes::Date32Type>,
    output: &mut Vec<String>,
) {
    use arrow::array::Array;

    let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).expect("1970-01-01 is a valid date");
    for row in 0..values.len() {
        if values.is_null(row) {
            continue;
        }
        if let Some(date) =
            epoch.checked_add_signed(chrono::Duration::days(values.value(row) as i64))
        {
            push_unique_sample(output, &date.format("%Y-%m-%d").to_string());
        }
        if output.len() >= MAX_ARTIFACT_SAMPLE_VALUES {
            break;
        }
    }
}

fn push_unique_sample(output: &mut Vec<String>, value: &str) {
    let value = value.trim();
    if value.is_empty()
        || output.len() >= MAX_ARTIFACT_SAMPLE_VALUES
        || output.iter().any(|existing| existing == value)
    {
        return;
    }
    output.push(value.to_string());
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|err| format!("failed to open artifact file {}: {err}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|err| format!("failed to read artifact file {}: {err}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn normalize_sha256_hash(hash: &str) -> &str {
    hash.strip_prefix("sha256:").unwrap_or(hash)
}

fn format_sha256_hash_like_expected(expected_hash: &str, computed_hash: &str) -> String {
    if expected_hash.starts_with("sha256:") {
        format!("sha256:{computed_hash}")
    } else {
        computed_hash.to_string()
    }
}

fn file_uri_local_path_candidates(uri: &str) -> Result<Vec<PathBuf>, String> {
    let raw_path = uri
        .strip_prefix("file://")
        .ok_or_else(|| format!("artifact uri is not a file uri: {uri}"))?;
    if raw_path.is_empty() {
        return Err("artifact file uri is empty".to_string());
    }

    let decoded = urlencoding::decode(raw_path)
        .map_err(|err| format!("failed to decode artifact file uri {uri}: {err}"))?
        .into_owned();

    if let Some((drive, tail)) = windows_drive_uri_tail(&decoded) {
        let drive = drive.to_ascii_lowercase();
        let tail = tail.trim_start_matches(['/', '\\']);
        return Ok(vec![
            PathBuf::from(format!("/mnt/{drive}/{tail}")),
            PathBuf::from(format!("/{drive}/{tail}")),
            PathBuf::from(decoded),
        ]);
    }

    Ok(vec![PathBuf::from(decoded)])
}

fn windows_drive_uri_tail(path: &str) -> Option<(char, &str)> {
    let normalized = path.strip_prefix('/').unwrap_or(path);
    let mut chars = normalized.chars();
    let drive = chars.next()?;
    if !drive.is_ascii_alphabetic() || chars.next()? != ':' {
        return None;
    }
    let separator = chars.next()?;
    if separator != '/' && separator != '\\' {
        return None;
    }
    Some((drive, chars.as_str()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketArtifactRequest {
    expected_dataset_version: String,
    artifact_type: String,
    schema_version: Option<String>,
    expected_artifact_hash: Option<String>,
}

impl MarketArtifactRequest {
    pub fn new(
        expected_dataset_version: impl Into<String>,
        artifact_type: impl Into<String>,
    ) -> Self {
        Self {
            expected_dataset_version: expected_dataset_version.into(),
            artifact_type: artifact_type.into(),
            schema_version: None,
            expected_artifact_hash: None,
        }
    }

    pub fn require_schema_version(mut self, schema_version: impl Into<String>) -> Self {
        self.schema_version = Some(schema_version.into());
        self
    }

    pub fn require_artifact_hash(mut self, expected_artifact_hash: impl Into<String>) -> Self {
        self.expected_artifact_hash = Some(expected_artifact_hash.into());
        self
    }

    pub fn resolve_from_slice(&self, bytes: &[u8]) -> Result<ResolvedMarketArtifact, String> {
        self.intake()
            .resolve_artifact_from_slice(bytes, &self.selector())
    }

    pub fn resolve_from_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<ResolvedMarketArtifact, String> {
        self.intake()
            .resolve_artifact_from_path(path, &self.selector())
    }

    fn intake(&self) -> ManifestIntake {
        let mut validator = ManifestValidator::new(self.expected_dataset_version.clone());
        if let Some(expected_hash) = &self.expected_artifact_hash {
            validator = validator.require_artifact_hash(expected_hash.clone());
        }
        ManifestIntake::new(validator)
    }

    fn selector(&self) -> MarketArtifactSelector {
        let mut selector = MarketArtifactSelector::new(self.artifact_type.clone());
        if let Some(schema_version) = &self.schema_version {
            selector = selector.require_schema_version(schema_version.clone());
        }
        selector
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

#[cfg(test)]
mod tests {
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
        let policy =
            ControlledPersistencePolicy::parse("clickhouse-shadow: market_shadow").unwrap();

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
        let candidates = file_uri_local_path_candidates(
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
}
