use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow};
use std::path::{Path, PathBuf};

use crate::core::{QuantixError, Result};

const CREATE_RISK_INDUSTRY_SNAPSHOTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS risk_industry_snapshots (
    snapshot_month TEXT NOT NULL,
    code TEXT NOT NULL,
    industry_name TEXT NOT NULL,
    source TEXT NOT NULL,
    captured_at TEXT NOT NULL,
    PRIMARY KEY (snapshot_month, code)
);
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndustrySnapshotRecord {
    pub snapshot_month: String,
    pub code: String,
    pub industry_name: String,
    pub source: String,
    pub captured_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SqliteIndustrySnapshotStore {
    path: PathBuf,
    pool: SqlitePool,
}

impl SqliteIndustrySnapshotStore {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        let store = Self {
            path: path.to_path_buf(),
            pool,
        };
        store.ensure_schema().await?;
        Ok(store)
    }

    pub async fn from_risk_path<P: AsRef<Path>>(risk_path: P) -> Result<Self> {
        Self::new(Self::path_for_risk_path(risk_path.as_ref())).await
    }

    pub fn path_for_risk_path(risk_path: &Path) -> PathBuf {
        risk_path.with_file_name("industry_snapshots.db")
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    async fn ensure_schema(&self) -> Result<()> {
        sqlx::query(CREATE_RISK_INDUSTRY_SNAPSHOTS_TABLE_SQL)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn find_month_snapshot(
        &self,
        snapshot_month: &str,
        code: &str,
    ) -> Result<Option<IndustrySnapshotRecord>> {
        let row = sqlx::query(
            r#"
SELECT snapshot_month, code, industry_name, source, captured_at
FROM risk_industry_snapshots
WHERE snapshot_month = ? AND code = ?
LIMIT 1
"#,
        )
        .bind(snapshot_month)
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::row_to_record).transpose()
    }

    pub async fn find_latest_snapshot(&self, code: &str) -> Result<Option<IndustrySnapshotRecord>> {
        let row = sqlx::query(
            r#"
SELECT snapshot_month, code, industry_name, source, captured_at
FROM risk_industry_snapshots
WHERE code = ?
ORDER BY snapshot_month DESC, captured_at DESC
LIMIT 1
"#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::row_to_record).transpose()
    }

    pub async fn insert_if_missing(&self, record: &IndustrySnapshotRecord) -> Result<bool> {
        let result = sqlx::query(
            r#"
INSERT INTO risk_industry_snapshots (
    snapshot_month,
    code,
    industry_name,
    source,
    captured_at
) VALUES (?, ?, ?, ?, ?)
ON CONFLICT(snapshot_month, code) DO NOTHING
"#,
        )
        .bind(&record.snapshot_month)
        .bind(&record.code)
        .bind(&record.industry_name)
        .bind(&record.source)
        .bind(record.captured_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    fn row_to_record(row: SqliteRow) -> Result<IndustrySnapshotRecord> {
        let captured_at: String = row.try_get("captured_at")?;
        let captured_at = DateTime::parse_from_rfc3339(&captured_at)
            .map_err(|err| {
                QuantixError::DataParse(format!(
                    "invalid industry snapshot timestamp {}: {}",
                    captured_at, err
                ))
            })?
            .with_timezone(&Utc);

        Ok(IndustrySnapshotRecord {
            snapshot_month: row.try_get("snapshot_month")?,
            code: row.try_get("code")?,
            industry_name: row.try_get("industry_name")?,
            source: row.try_get("source")?,
            captured_at,
        })
    }
}
