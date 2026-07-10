//! Quantix regression report and evidence generation.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

use super::sampling::{file_uri_local_path_candidates, sample_artifact_payload, sha256_file};
use super::selection::{ControlledPersistencePolicy, ResolvedMarketArtifact};

/// 回归运行上下文（由调用方提供）：source_command 触发命令、run_at 运行时间、consumer_build_commit 提交、database_target 目标库、writes_performed 是否实际写库。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantixRegressionContext {
    pub source_command: String,
    pub run_at: String,
    pub consumer_build_commit: String,
    pub database_target: String,
    pub writes_performed: bool,
}

/// Quantix 回归报告根（schema=quantix_regression_report.v1）：来源/运行时间/消费方系统、dataset_version/lineage_id/payload_hash/rows_hash、artifact 元数据、行数、样本 symbols/dates、回归状态、消费方构建信息、redaction_notes/warnings。由 QuantixRegressionReport::from_resolved_artifact_with_comparison 构造。
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
    /// 以 resolved artifact + context 构造回归报告（不带 double-read comparison）；等价于 comparison=None 调用 from_resolved_artifact_with_comparison。
    pub fn from_resolved_artifact(
        resolved: &ResolvedMarketArtifact,
        context: QuantixRegressionContext,
    ) -> Self {
        Self::from_resolved_artifact_with_comparison(resolved, context, None)
    }

    /// 构造完整回归报告：聚合 manifest/artifact 校验项、ControlledPersistencePolicy 校验、payload 采样与行数验证；可选 comparison 触发 double-read 对比项，决定最终 comparison_summary。
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

    /// 把 report 序列化为格式化 JSON 字符串；序列化失败返回带原因的 Err。
    pub fn to_pretty_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|err| format!("failed to serialize quantix regression report: {err}"))
    }
}

/// 回归报告内嵌的 artifact 元数据：type/uri/schema_version/row_count/hash 四个 manifest 字段 + computed_hash/computed_row_count/rows_hash 三个本地复算字段，对比二者构成校验项。
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

/// 回归校验状态：passed 是否全部通过、failed_checks 失败项列表、checks 全部检查项（含通过）、comparison_summary 对比结论摘要、可选 comparison 详情。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantixRegressionStatus {
    pub passed: bool,
    pub failed_checks: Vec<String>,
    pub checks: Vec<String>,
    pub comparison_summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison: Option<QuantixRegressionComparison>,
}

/// Double-read 对比详情：comparison_type 对比类型（local_reference_artifact/source_of_truth_summary/direct_clickhouse_read_only）、reference_uri/reference_hash 参考基准、行数与样本（symbols/dates）是否匹配、参考样本详情。
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
    /// 以本地参考 artifact 文件为基准构造 comparison：解析路径、计算 sha256、采样 payload 后逐项（行数/symbols/dates）比对。
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

    /// 以 source-of-truth summary JSON 为基准构造 comparison：解析路径→读取→反序列化→校验 dataset_version/lineage_id/payload_hash 一致性→比对行数与样本。
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

    /// 以 ClickHouse 只读 summary 为基准构造 comparison：先校验 dataset_version 一致，再用 summary 行数/样本对比 resolved 的对应字段；reference_hash 由 summary 内容序列化后 sha256 得到。
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

/// Source-of-truth summary JSON 反序列化结构：source_system/source_uri 来源、dataset_version/lineage_id/payload_hash 三元一致性键、row_count 行数、sample_symbols/sample_dates 样本、generated_at 生成时间。
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

/// ClickHouse 只读查询返回的 summary：database/table 来源、dataset_version、row_count、sample_symbols/sample_dates。用于 from_clickhouse_read_only_summary 对比基准。
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
    /// 返回该 summary 的稳定引用 URI（clickhouse://{database}.{table}?dataset_version=...），用于 evidence/comparison 记录。
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

/// 消费方构建信息：repo 仓库名、commit 提交 SHA、database_target 目标库（用于 ControlledPersistencePolicy 校验）、writes_performed 是否实际写库。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantixConsumerBuild {
    pub repo: String,
    pub commit: String,
    pub database_target: String,
    pub writes_performed: bool,
}

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
