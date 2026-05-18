use chrono::{DateTime, Datelike, NaiveDate, Utc};

use crate::core::{QuantixError, Result};
use crate::risk::industry_store::SqliteIndustryStore;

pub const ACTIVE_CLASSIFICATION_STANDARD: ClassificationStandard = ClassificationStandard::Shenwan;
pub const ACTIVE_INDUSTRY_LEVEL: IndustryClassificationLevel =
    IndustryClassificationLevel::FirstLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassificationStandard {
    Shenwan,
    Csrc,
}

impl ClassificationStandard {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shenwan => "shenwan",
            Self::Csrc => "csrc",
        }
    }

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
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FirstLevel => "first_level",
        }
    }

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

#[derive(Debug, Clone)]
pub struct IndustryResolver {
    store: SqliteIndustryStore,
}

impl IndustryResolver {
    pub fn new(store: SqliteIndustryStore) -> Self {
        Self { store }
    }

    pub fn active_standard(&self) -> ClassificationStandard {
        ACTIVE_CLASSIFICATION_STANDARD
    }

    pub fn active_level(&self) -> IndustryClassificationLevel {
        ACTIVE_INDUSTRY_LEVEL
    }

    pub async fn sync_shenwan_current_rows(
        &self,
        rows: &[ShenwanCurrentSeedRow],
        imported_at: DateTime<Utc>,
    ) -> Result<()> {
        self.store
            .refresh_shenwan_current_rows(rows, imported_at)
            .await
    }

    pub async fn sync_shenwan_history_rows(
        &self,
        rows: &[ShenwanHistoricalSeedRow],
        imported_at: DateTime<Utc>,
    ) -> Result<()> {
        self.store
            .refresh_shenwan_history_rows(rows, imported_at)
            .await
    }

    pub async fn resolve(
        &self,
        code: &str,
        query_date: NaiveDate,
        captured_at: DateTime<Utc>,
    ) -> Result<ResolvedIndustry> {
        let normalized_code = normalize_security_code(code);
        let standard = self.active_standard();
        let level = self.active_level();
        let query_month = snapshot_month(query_date);

        if let Some(current) = self
            .store
            .lookup_current(standard, level, &normalized_code)
            .await?
        {
            self.store
                .insert_snapshot_if_missing(
                    standard,
                    level,
                    &query_month,
                    &normalized_code,
                    &current.industry_name,
                    &current.source,
                    captured_at,
                )
                .await?;

            return Ok(ResolvedIndustry {
                code: normalized_code,
                industry_name: current.industry_name,
                standard,
                level,
                source_tier: IndustrySourceTier::CurrentActive,
                query_month,
            });
        }

        if let Some(snapshot) = self
            .store
            .lookup_snapshot_month(standard, level, &normalized_code, &query_month)
            .await?
        {
            return Ok(ResolvedIndustry {
                code: normalized_code,
                industry_name: snapshot.industry_name,
                standard,
                level,
                source_tier: IndustrySourceTier::SnapshotMonth,
                query_month,
            });
        }

        if let Some(history) = self
            .store
            .lookup_historical(standard, level, &normalized_code, query_date)
            .await?
        {
            return Ok(ResolvedIndustry {
                code: normalized_code,
                industry_name: history.industry_name,
                standard,
                level,
                source_tier: IndustrySourceTier::Historical,
                query_month,
            });
        }

        if let Some(snapshot) = self
            .store
            .lookup_latest_snapshot(standard, level, &normalized_code)
            .await?
        {
            return Ok(ResolvedIndustry {
                code: normalized_code,
                industry_name: snapshot.industry_name,
                standard,
                level,
                source_tier: IndustrySourceTier::LatestSnapshot,
                query_month,
            });
        }

        Err(QuantixError::Other(format!(
            "industry resolution failed for code {} on {} across current/monthly/history/fallback",
            normalized_code, query_date
        )))
    }
}

pub fn normalize_security_code(code: &str) -> String {
    code.trim()
        .split('.')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_uppercase()
}

pub fn snapshot_month(query_date: NaiveDate) -> String {
    format!("{:04}-{:02}", query_date.year(), query_date.month())
}
