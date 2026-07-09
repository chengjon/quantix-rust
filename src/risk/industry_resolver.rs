use chrono::{DateTime, NaiveDate, Utc};

use crate::core::{QuantixError, Result};
use crate::risk::industry::{
    ACTIVE_CLASSIFICATION_STANDARD, ACTIVE_INDUSTRY_LEVEL, ClassificationStandard,
    IndustryClassificationLevel, IndustrySourceTier, ResolvedIndustry, ShenwanCurrentSeedRow,
    ShenwanHistoricalSeedRow, normalize_security_code, snapshot_month,
};
use crate::risk::industry_store::SqliteIndustryStore;

#[derive(Debug, Clone)]
pub struct IndustryResolver {
    store: SqliteIndustryStore,
}

impl IndustryResolver {
    /// 用底层 SqliteIndustryStore 构造 resolver。
    pub fn new(store: SqliteIndustryStore) -> Self {
        Self { store }
    }

    /// 返回当前激活的分类标准（项目级常量 ACTIVE_CLASSIFICATION_STANDARD，目前为 Shenwan）。
    pub fn active_standard(&self) -> ClassificationStandard {
        ACTIVE_CLASSIFICATION_STANDARD
    }

    /// 返回当前激活的分类层级（项目级常量 ACTIVE_INDUSTRY_LEVEL，目前为 FirstLevel）。
    pub fn active_level(&self) -> IndustryClassificationLevel {
        ACTIVE_INDUSTRY_LEVEL
    }

    /// 单事务刷新 shenwan current 表（refresh_shenwan_current_rows 的代理调用）。
    pub async fn sync_shenwan_current_rows(
        &self,
        rows: &[ShenwanCurrentSeedRow],
        imported_at: DateTime<Utc>,
    ) -> Result<()> {
        self.store
            .refresh_shenwan_current_rows(rows, imported_at)
            .await
    }

    /// 单事务刷新 shenwan history 表（refresh_shenwan_history_rows 的代理调用）。
    pub async fn sync_shenwan_history_rows(
        &self,
        rows: &[ShenwanHistoricalSeedRow],
        imported_at: DateTime<Utc>,
    ) -> Result<()> {
        self.store
            .refresh_shenwan_history_rows(rows, imported_at)
            .await
    }

    /// 按 current → snapshot_month → historical → latest_snapshot 顺序解析行业归属；任一命中即返回并标注 source_tier，全部未命中返回错误。
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
