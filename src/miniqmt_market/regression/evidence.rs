use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::report::{
    QuantixConsumerBuild, QuantixRegressionArtifact, QuantixRegressionReport,
    QuantixRegressionStatus,
};
use crate::miniqmt_market::sampling::{file_uri_local_path_candidates, sha256_file};

/// 原始报告文件引用：path 路径、hash sha256、size_bytes 字节数，由 raw_report_reference() 收集并嵌入 evidence。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawReportReference {
    pub path: String,
    pub hash: String,
    pub size_bytes: u64,
}

/// 受控 evidence（schema=evidence.v1）：仅能从 passed=true 的 regression report 构造，作为 dataset 升级 / 消费方回归通过的不可篡改证据。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantixRegressionEvidence {
    pub schema_version: String,
    pub source_command: String,
    pub run_at: String,
    pub environment: QuantixEvidenceEnvironment,
    pub result_summary: QuantixEvidenceResultSummary,
}

impl QuantixRegressionEvidence {
    /// 从已通过（passed=true 且 failed_checks 为空）的 regression report 构造受控 evidence；传入失败 report 时返回带 failed_checks 详情的 Err。
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

    /// 把 evidence 序列化为格式化 JSON 字符串；序列化失败返回带原因的 Err。
    pub fn to_pretty_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|err| format!("failed to serialize quantix regression evidence: {err}"))
    }
}

/// 收集指定 report 文件的 path/sha256/size 作为 RawReportReference；stat 或 hash 失败返回带路径信息的 Err。
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

pub(crate) fn resolve_reference_local_artifact_path(
    reference_artifact: &str,
) -> Result<PathBuf, String> {
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

pub(crate) fn resolve_source_of_truth_summary_path(
    source_of_truth_summary: &str,
) -> Result<PathBuf, String> {
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

/// Evidence 运行环境：consumer_system 消费方系统名、consumer_build 消费方提交 SHA。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantixEvidenceEnvironment {
    pub consumer_system: String,
    pub consumer_build: String,
}

/// Evidence 结果摘要：evidence_type 固定 "promotion_consumer_regression"、消费方/dataset/lineage/payload_hash/rows_hash/artifact/regression/row_count/samples/consumer_build/raw_report/warnings/redaction_notes/generated_at 全量快照。
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
