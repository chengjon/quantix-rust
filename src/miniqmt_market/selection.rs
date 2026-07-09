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
    /// 以目标 artifact_type 构造选择器；schema_version 约束默认未设置，可通过 require_schema_version 链式补充。
    pub fn new(artifact_type: impl Into<String>) -> Self {
        Self {
            artifact_type: artifact_type.into(),
            schema_version: None,
        }
    }

    /// Builder：要求 artifact 的 schema_version 必须精确匹配给定值；返回 self 便于链式构造。
    pub fn require_schema_version(mut self, schema_version: impl Into<String>) -> Self {
        self.schema_version = Some(schema_version.into());
        self
    }

    /// 在 manifest.artifacts 中按 type（+可选 schema_version）过滤；恰好一个匹配返回 Ok，0 个或多个匹配均返回带 dataset_version 上下文的 Err。
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
    /// 从 manifest 元数据 + manifest 内的 artifact 条目组装 ResolvedMarketArtifact；computed_hash / computed_row_count / sample_* 字段初始为空，待 verify_artifact_file_hash 填充。
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

    /// 把 resolved artifact 序列化为格式化 JSON 字符串；序列化失败返回带原因的 Err。
    pub fn to_pretty_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|err| format!("failed to serialize resolved market artifact: {err}"))
    }

    /// 解析 artifact 本地路径并校验文件 sha256 与 manifest.hash 一致（大小写不敏感）；匹配成功时填充 computed_hash/computed_row_count/sample_symbols/sample_dates，失配或路径解析失败返回带 uri 上下文的 Err。
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
    /// 把 database_target 字符串解析为策略枚举：dry-run-only / clickhouse-shadow:<table> / clickhouse-production:<table>；table 为空或前缀未识别均返回稳定错误码字符串。
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

    /// 校验当前策略与 writes_performed 是否自洽（dry-run 不得写、shadow 必须显式 writes、production 暂未实现）；不一致时返回稳定错误码字符串，一致时返回对应的 check 标识。
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
