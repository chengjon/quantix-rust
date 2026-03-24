use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;

use crate::core::{QuantixError, Result};
use crate::db::ClickHouseClient;
use crate::risk::industry_store::{IndustrySnapshotRecord, SqliteIndustrySnapshotStore};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatestIndustryRecord {
    pub code: String,
    pub industry_name: String,
    pub source: String,
    pub captured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndustrySourceTier {
    Latest,
    SnapshotMonth,
    SnapshotFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedIndustry {
    pub code: String,
    pub industry_name: String,
    pub source: String,
    pub captured_at: DateTime<Utc>,
    pub snapshot_month: String,
    pub source_tier: IndustrySourceTier,
}

#[async_trait]
pub trait LatestIndustryReader: Send + Sync {
    async fn load_latest_industry(&self, code: &str) -> Result<Option<LatestIndustryRecord>>;
}

#[derive(Debug, Clone)]
pub struct IndustryResolver<R> {
    snapshot_store: SqliteIndustrySnapshotStore,
    latest_reader: R,
}

impl<R> IndustryResolver<R>
where
    R: LatestIndustryReader,
{
    pub fn new(snapshot_store: SqliteIndustrySnapshotStore, latest_reader: R) -> Self {
        Self {
            snapshot_store,
            latest_reader,
        }
    }

    pub async fn resolve(&self, code: &str, query_date: NaiveDate) -> Result<ResolvedIndustry> {
        let snapshot_month = snapshot_month_for(query_date);

        match self.latest_reader.load_latest_industry(code).await {
            Ok(Some(latest)) => {
                let snapshot = IndustrySnapshotRecord::from_latest(&snapshot_month, latest.clone());
                self.snapshot_store.insert_if_missing(&snapshot).await?;
                return Ok(ResolvedIndustry {
                    code: latest.code,
                    industry_name: latest.industry_name,
                    source: latest.source,
                    captured_at: latest.captured_at,
                    snapshot_month,
                    source_tier: IndustrySourceTier::Latest,
                });
            }
            Ok(None) | Err(_) => {}
        }

        if let Some(snapshot) = self
            .snapshot_store
            .find_month_snapshot(&snapshot_month, code)
            .await?
        {
            return Ok(ResolvedIndustry::from_snapshot(
                snapshot,
                IndustrySourceTier::SnapshotMonth,
            ));
        }

        if let Some(snapshot) = self.snapshot_store.find_latest_snapshot(code).await? {
            return Ok(ResolvedIndustry::from_snapshot(
                snapshot,
                IndustrySourceTier::SnapshotFallback,
            ));
        }

        Err(QuantixError::Other(format!(
            "risk industry resolution failed: code={} tiers=latest/monthly/fallback",
            code
        )))
    }
}

#[derive(Clone)]
pub struct ClickHouseLatestIndustryReader {
    client: Arc<ClickHouseClient>,
}

impl ClickHouseLatestIndustryReader {
    pub fn new(client: Arc<ClickHouseClient>) -> Self {
        Self { client }
    }
}

#[derive(Debug, Deserialize)]
struct LatestIndustryRow {
    code: String,
    industry_name: String,
    source: String,
    captured_at: String,
}

#[async_trait]
impl LatestIndustryReader for ClickHouseLatestIndustryReader {
    async fn load_latest_industry(&self, code: &str) -> Result<Option<LatestIndustryRecord>> {
        let escaped_code = escape_sql_literal(code);
        let sql = format!(
            r#"
            SELECT
                '{code}' AS code,
                sector_name AS industry_name,
                'sector_daily' AS source,
                toString(updated_at) AS captured_at
            FROM sector_daily
            WHERE sector_type = 'industry'
              AND (leader_code = '{code}' OR sector_code = '{code}')
            ORDER BY trade_date DESC, updated_at DESC, rank ASC, sector_code ASC
            LIMIT 1
            "#,
            code = escaped_code,
        );

        let row = self
            .client
            .query_json::<LatestIndustryRow>(&sql)
            .await
            .map_err(|err| {
                QuantixError::DatabaseQuery(format!(
                    "risk industry latest lookup failed for code={}: {}",
                    code, err
                ))
            })?
            .into_iter()
            .next();

        row.map(TryInto::try_into).transpose()
    }
}

impl TryFrom<LatestIndustryRow> for LatestIndustryRecord {
    type Error = QuantixError;

    fn try_from(value: LatestIndustryRow) -> Result<Self> {
        Ok(Self {
            code: value.code,
            industry_name: value.industry_name,
            source: value.source,
            captured_at: parse_clickhouse_timestamp(&value.captured_at)?,
        })
    }
}

impl ResolvedIndustry {
    fn from_snapshot(snapshot: IndustrySnapshotRecord, source_tier: IndustrySourceTier) -> Self {
        Self {
            code: snapshot.code,
            industry_name: snapshot.industry_name,
            source: snapshot.source,
            captured_at: snapshot.captured_at,
            snapshot_month: snapshot.snapshot_month,
            source_tier,
        }
    }
}

impl IndustrySnapshotRecord {
    pub fn from_latest(snapshot_month: &str, latest: LatestIndustryRecord) -> Self {
        Self {
            snapshot_month: snapshot_month.to_string(),
            code: latest.code,
            industry_name: latest.industry_name,
            source: latest.source,
            captured_at: latest.captured_at,
        }
    }
}

pub fn snapshot_month_for(query_date: NaiveDate) -> String {
    query_date.format("%Y-%m").to_string()
}

fn parse_clickhouse_timestamp(value: &str) -> Result<DateTime<Utc>> {
    if let Ok(ts) = DateTime::parse_from_rfc3339(value) {
        return Ok(ts.with_timezone(&Utc));
    }

    let ts = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").map_err(|err| {
        QuantixError::DataParse(format!("invalid ClickHouse timestamp {value}: {err}"))
    })?;
    Ok(ts.and_utc())
}

fn escape_sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}
