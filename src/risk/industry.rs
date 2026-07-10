use chrono::{DateTime, Datelike, NaiveDate, Utc};

use crate::core::{QuantixError, Result};

/// 当前生效的行业分类标准：申万（Shenwan）。改动此项需要同步迁移 industry_* 表数据。
pub const ACTIVE_CLASSIFICATION_STANDARD: ClassificationStandard = ClassificationStandard::Shenwan;
/// 当前生效的行业分类层级：一级行业。集中度与黑名单均按此层级判定。
pub const ACTIVE_INDUSTRY_LEVEL: IndustryClassificationLevel =
    IndustryClassificationLevel::FirstLevel;

/// 行业分类标准：Shenwan 申万、Csrc 证监会。当前 ACTIVE_CLASSIFICATION_STANDARD 为 Shenwan。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassificationStandard {
    Shenwan,
    Csrc,
}

impl ClassificationStandard {
    /// 返回该分类标准的稳定字符串标识（"shenwan" / "csrc"），用于入库与序列化。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shenwan => "shenwan",
            Self::Csrc => "csrc",
        }
    }

    /// 反向解析 "shenwan"/"csrc"；未知值返回错误。
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "shenwan" => Ok(Self::Shenwan),
            "csrc" => Ok(Self::Csrc),
            other => Err(QuantixError::Other(format!(
                "unknown classification standard: {other}"
            ))),
        }
    }
}

/// 行业分类层级：当前仅 FirstLevel（一级行业），保留枚举供后续扩展二/三级行业。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndustryClassificationLevel {
    FirstLevel,
}

impl IndustryClassificationLevel {
    /// 返回该分类层级的稳定字符串标识（当前仅 "first_level"）。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FirstLevel => "first_level",
        }
    }

    /// 反向解析 "first_level"；未知值返回错误。
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "first_level" => Ok(Self::FirstLevel),
            other => Err(QuantixError::Other(format!(
                "unknown industry classification level: {other}"
            ))),
        }
    }
}

/// 行业来源层级（优先级从高到低）：CurrentActive 当前生效、SnapshotMonth 指定月份快照、Historical 历史回溯、LatestSnapshot 最近可用快照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndustrySourceTier {
    CurrentActive,
    SnapshotMonth,
    Historical,
    LatestSnapshot,
}

/// 已解析出的行业归属：code 标的、industry_name 行业名、standard 分类标准、level 层级、source_tier 来源层级、query_month 查询月份（YYYY-MM）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedIndustry {
    pub code: String,
    pub industry_name: String,
    pub standard: ClassificationStandard,
    pub level: IndustryClassificationLevel,
    pub source_tier: IndustrySourceTier,
    pub query_month: String,
}

/// 行业参照表记录（reference 表行）：code、industry_name、standard、level、source 来源标识。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndustryReferenceRecord {
    pub code: String,
    pub industry_name: String,
    pub standard: ClassificationStandard,
    pub level: IndustryClassificationLevel,
    pub source: String,
}

/// 行业快照表记录（snapshot 表行）：code、industry_name、standard、level、snapshot_month 月份键、source 来源、captured_at 抓取时间。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndustrySnapshotRecord {
    pub code: String,
    pub industry_name: String,
    pub standard: ClassificationStandard,
    pub level: IndustryClassificationLevel,
    pub snapshot_month: String,
    pub source: String,
    pub captured_at: DateTime<Utc>,
}

/// 申万当前生效种子行：security_code 证券代码、industry_name 行业名、source 来源标识。用于回填 reference/snapshot 表。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShenwanCurrentSeedRow {
    pub security_code: String,
    pub industry_name: String,
    pub source: String,
}

/// 申万历史种子行：security_code、industry_name、effective_from 生效起、effective_to 可选生效止（None 表示至今）、source 来源。用于历史回溯快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShenwanHistoricalSeedRow {
    pub security_code: String,
    pub industry_name: String,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub source: String,
}

/// 标准化证券代码：去掉前后空白、去掉 `.SH`/`.SZ` 等后缀（取 `.` 之前部分），转大写。
pub fn normalize_security_code(code: &str) -> String {
    code.trim()
        .split('.')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_uppercase()
}

/// 把 NaiveDate 转成 `YYYY-MM` 字符串，作为 snapshot_month 的查询键。
pub fn snapshot_month(query_date: NaiveDate) -> String {
    format!("{:04}-{:02}", query_date.year(), query_date.month())
}
