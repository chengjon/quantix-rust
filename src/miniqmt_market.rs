//! miniQMT market artifact contract boundary.
//!
//! This module is the public facade for miniQMT market dataset intake. Its
//! owned responsibilities are split across sibling modules:
//!
//! - [`manifest`] — manifest validation and intake;
//! - [`selection`] — artifact selection, local artifact resolution,
//!   controlled persistence policy;
//! - [`regression`] — Quantix regression report and evidence generation;
//! - [`sampling`] — local payload sampling and hash verification helpers.
//!
//! The facade [`MarketArtifactRequest`] stays here and composes the
//! manifest intake and artifact selector into a single public entry point.
//!
//! All types are re-exported here so callers can keep using
//! `crate::miniqmt_market::<Type>` paths unchanged.

pub mod manifest;
pub mod regression;
pub mod sampling;
pub mod selection;

#[cfg(test)]
mod tests;

pub use manifest::{
    ManifestIntake, ManifestQuality, ManifestSource, ManifestValidator, MarketDatasetArtifact,
    MarketDatasetManifest, load_manifest_from_path, load_manifest_from_slice,
};
pub use regression::{
    QuantixClickHouseReadOnlySummary, QuantixConsumerBuild, QuantixEvidenceEnvironment,
    QuantixEvidenceResultSummary, QuantixRegressionArtifact, QuantixRegressionComparison,
    QuantixRegressionContext, QuantixRegressionEvidence, QuantixRegressionReport,
    QuantixRegressionStatus, QuantixSourceOfTruthSummary, RawReportReference, raw_report_reference,
};
pub use sampling::ArtifactPayloadSample;
pub use selection::{ControlledPersistencePolicy, MarketArtifactSelector, ResolvedMarketArtifact};

use std::path::Path;

/// Public request facade composing manifest intake and artifact selection.
///
/// Callers describe the dataset version, artifact type, and optional schema
/// version / artifact hash constraints; this type wires those parameters into
/// the [`ManifestValidator`] / [`MarketArtifactSelector`] pair and resolves
/// a [`ResolvedMarketArtifact`] from either raw manifest bytes or a path.
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
