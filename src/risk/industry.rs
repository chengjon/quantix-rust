use chrono::{DateTime, Datelike, NaiveDate, Utc};

use crate::core::{QuantixError, Result};

pub const ACTIVE_CLASSIFICATION_STANDARD: ClassificationStandard = ClassificationStandard::Shenwan;
pub const ACTIVE_INDUSTRY_LEVEL: IndustryClassificationLevel =
    IndustryClassificationLevel::FirstLevel;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndustrySourceTier {
    CurrentActive,
    SnapshotMonth,
    Historical,
    LatestSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedIndustry {
    pub code: String,
    pub industry_name: String,
    pub standard: ClassificationStandard,
    pub level: IndustryClassificationLevel,
    pub source_tier: IndustrySourceTier,
    pub query_month: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndustryReferenceRecord {
    pub code: String,
    pub industry_name: String,
    pub standard: ClassificationStandard,
    pub level: IndustryClassificationLevel,
    pub source: String,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShenwanCurrentSeedRow {
    pub security_code: String,
    pub industry_name: String,
    pub source: String,
}

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
