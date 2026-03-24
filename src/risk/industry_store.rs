use chrono::{DateTime, NaiveDate, Utc};
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::{Path, PathBuf};

use crate::core::{QuantixError, Result};
use crate::risk::industry::{
    ClassificationStandard, IndustryClassificationLevel, IndustryReferenceRecord,
    IndustrySnapshotRecord, ShenwanCurrentSeedRow, ShenwanHistoricalSeedRow,
    normalize_security_code,
};

const CREATE_INDUSTRY_REFERENCE_CURRENT_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS industry_reference_current (
    standard TEXT NOT NULL,
    level TEXT NOT NULL,
    code TEXT NOT NULL,
    industry_name TEXT NOT NULL,
    source TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (standard, level, code)
);
"#;

const CREATE_INDUSTRY_REFERENCE_HISTORY_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS industry_reference_history (
    standard TEXT NOT NULL,
    level TEXT NOT NULL,
    code TEXT NOT NULL,
    industry_name TEXT NOT NULL,
    effective_from TEXT NOT NULL,
    effective_to TEXT,
    source TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (standard, level, code, effective_from)
);
"#;

const CREATE_RISK_INDUSTRY_SNAPSHOTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS risk_industry_snapshots (
    standard TEXT NOT NULL,
    level TEXT NOT NULL,
    snapshot_month TEXT NOT NULL,
    code TEXT NOT NULL,
    industry_name TEXT NOT NULL,
    source TEXT NOT NULL,
    captured_at TEXT NOT NULL,
    PRIMARY KEY (standard, level, snapshot_month, code)
);
"#;

#[derive(Debug, Clone)]
pub struct SqliteIndustryStore {
    path: PathBuf,
    pool: SqlitePool,
}

impl SqliteIndustryStore {
    pub async fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let options = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        let store = Self { path, pool };
        store.ensure_schema().await?;
        Ok(store)
    }

    pub async fn from_risk_state_path(risk_state_path: impl AsRef<Path>) -> Result<Self> {
        Self::new(Self::default_db_path_from_risk_state(risk_state_path)).await
    }

    pub fn default_db_path_from_risk_state(risk_state_path: impl AsRef<Path>) -> PathBuf {
        let risk_state_path = risk_state_path.as_ref();
        match risk_state_path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.join("industry_reference.db"),
            _ => risk_state_path.with_file_name("industry_reference.db"),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    async fn ensure_schema(&self) -> Result<()> {
        for statement in [
            CREATE_INDUSTRY_REFERENCE_CURRENT_TABLE_SQL,
            CREATE_INDUSTRY_REFERENCE_HISTORY_TABLE_SQL,
            CREATE_RISK_INDUSTRY_SNAPSHOTS_TABLE_SQL,
        ] {
            sqlx::query(statement).execute(&self.pool).await?;
        }
        Ok(())
    }

    pub async fn lookup_current(
        &self,
        standard: ClassificationStandard,
        level: IndustryClassificationLevel,
        code: &str,
    ) -> Result<Option<IndustryReferenceRecord>> {
        let normalized_code = normalize_security_code(code);
        let row = sqlx::query(
            r#"
SELECT standard, level, code, industry_name, source
FROM industry_reference_current
WHERE standard = ? AND level = ? AND code = ?
"#,
        )
        .bind(standard.as_str())
        .bind(level.as_str())
        .bind(normalized_code)
        .fetch_optional(&self.pool)
        .await?;

        row.map(map_reference_record).transpose()
    }

    pub async fn lookup_historical(
        &self,
        standard: ClassificationStandard,
        level: IndustryClassificationLevel,
        code: &str,
        query_date: NaiveDate,
    ) -> Result<Option<IndustryReferenceRecord>> {
        let normalized_code = normalize_security_code(code);
        let row = sqlx::query(
            r#"
SELECT standard, level, code, industry_name, source
FROM industry_reference_history
WHERE standard = ?
  AND level = ?
  AND code = ?
  AND effective_from <= ?
  AND (effective_to IS NULL OR effective_to >= ?)
ORDER BY effective_from DESC
LIMIT 1
"#,
        )
        .bind(standard.as_str())
        .bind(level.as_str())
        .bind(normalized_code)
        .bind(query_date.to_string())
        .bind(query_date.to_string())
        .fetch_optional(&self.pool)
        .await?;

        row.map(map_reference_record).transpose()
    }

    pub async fn lookup_snapshot_month(
        &self,
        standard: ClassificationStandard,
        level: IndustryClassificationLevel,
        code: &str,
        snapshot_month: &str,
    ) -> Result<Option<IndustrySnapshotRecord>> {
        let normalized_code = normalize_security_code(code);
        let row = sqlx::query(
            r#"
SELECT standard, level, snapshot_month, code, industry_name, source, captured_at
FROM risk_industry_snapshots
WHERE standard = ? AND level = ? AND snapshot_month = ? AND code = ?
"#,
        )
        .bind(standard.as_str())
        .bind(level.as_str())
        .bind(snapshot_month)
        .bind(normalized_code)
        .fetch_optional(&self.pool)
        .await?;

        row.map(map_snapshot_record).transpose()
    }

    pub async fn lookup_latest_snapshot(
        &self,
        standard: ClassificationStandard,
        level: IndustryClassificationLevel,
        code: &str,
    ) -> Result<Option<IndustrySnapshotRecord>> {
        let normalized_code = normalize_security_code(code);
        let row = sqlx::query(
            r#"
SELECT standard, level, snapshot_month, code, industry_name, source, captured_at
FROM risk_industry_snapshots
WHERE standard = ? AND level = ? AND code = ?
ORDER BY snapshot_month DESC, captured_at DESC
LIMIT 1
"#,
        )
        .bind(standard.as_str())
        .bind(level.as_str())
        .bind(normalized_code)
        .fetch_optional(&self.pool)
        .await?;

        row.map(map_snapshot_record).transpose()
    }

    pub async fn insert_snapshot_if_missing(
        &self,
        standard: ClassificationStandard,
        level: IndustryClassificationLevel,
        snapshot_month: &str,
        code: &str,
        industry_name: &str,
        source: &str,
        captured_at: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            r#"
INSERT OR IGNORE INTO risk_industry_snapshots (
    standard,
    level,
    snapshot_month,
    code,
    industry_name,
    source,
    captured_at
) VALUES (?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(standard.as_str())
        .bind(level.as_str())
        .bind(snapshot_month)
        .bind(normalize_security_code(code))
        .bind(industry_name)
        .bind(source)
        .bind(captured_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert_shenwan_current_rows(
        &self,
        rows: &[ShenwanCurrentSeedRow],
        imported_at: DateTime<Utc>,
    ) -> Result<()> {
        for row in rows {
            sqlx::query(
                r#"
INSERT INTO industry_reference_current (
    standard,
    level,
    code,
    industry_name,
    source,
    updated_at
) VALUES (?, ?, ?, ?, ?, ?)
ON CONFLICT(standard, level, code) DO UPDATE SET
    industry_name = excluded.industry_name,
    source = excluded.source,
    updated_at = excluded.updated_at
"#,
            )
            .bind(ClassificationStandard::Shenwan.as_str())
            .bind(IndustryClassificationLevel::FirstLevel.as_str())
            .bind(normalize_security_code(&row.security_code))
            .bind(&row.industry_name)
            .bind(&row.source)
            .bind(imported_at.to_rfc3339())
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn refresh_shenwan_current_rows(
        &self,
        rows: &[ShenwanCurrentSeedRow],
        imported_at: DateTime<Utc>,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
DELETE FROM industry_reference_current
WHERE standard = ? AND level = ?
"#,
        )
        .bind(ClassificationStandard::Shenwan.as_str())
        .bind(IndustryClassificationLevel::FirstLevel.as_str())
        .execute(&mut *tx)
        .await?;

        for row in rows {
            sqlx::query(
                r#"
INSERT INTO industry_reference_current (
    standard,
    level,
    code,
    industry_name,
    source,
    updated_at
) VALUES (?, ?, ?, ?, ?, ?)
"#,
            )
            .bind(ClassificationStandard::Shenwan.as_str())
            .bind(IndustryClassificationLevel::FirstLevel.as_str())
            .bind(normalize_security_code(&row.security_code))
            .bind(&row.industry_name)
            .bind(&row.source)
            .bind(imported_at.to_rfc3339())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn upsert_shenwan_history_rows(
        &self,
        rows: &[ShenwanHistoricalSeedRow],
        imported_at: DateTime<Utc>,
    ) -> Result<()> {
        for row in rows {
            sqlx::query(
                r#"
INSERT INTO industry_reference_history (
    standard,
    level,
    code,
    industry_name,
    effective_from,
    effective_to,
    source,
    updated_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(standard, level, code, effective_from) DO UPDATE SET
    industry_name = excluded.industry_name,
    effective_to = excluded.effective_to,
    source = excluded.source,
    updated_at = excluded.updated_at
"#,
            )
            .bind(ClassificationStandard::Shenwan.as_str())
            .bind(IndustryClassificationLevel::FirstLevel.as_str())
            .bind(normalize_security_code(&row.security_code))
            .bind(&row.industry_name)
            .bind(row.effective_from.to_string())
            .bind(row.effective_to.map(|value| value.to_string()))
            .bind(&row.source)
            .bind(imported_at.to_rfc3339())
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn refresh_shenwan_history_rows(
        &self,
        rows: &[ShenwanHistoricalSeedRow],
        imported_at: DateTime<Utc>,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
DELETE FROM industry_reference_history
WHERE standard = ? AND level = ?
"#,
        )
        .bind(ClassificationStandard::Shenwan.as_str())
        .bind(IndustryClassificationLevel::FirstLevel.as_str())
        .execute(&mut *tx)
        .await?;

        for row in rows {
            sqlx::query(
                r#"
INSERT INTO industry_reference_history (
    standard,
    level,
    code,
    industry_name,
    effective_from,
    effective_to,
    source,
    updated_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
"#,
            )
            .bind(ClassificationStandard::Shenwan.as_str())
            .bind(IndustryClassificationLevel::FirstLevel.as_str())
            .bind(normalize_security_code(&row.security_code))
            .bind(&row.industry_name)
            .bind(row.effective_from.to_string())
            .bind(row.effective_to.map(|value| value.to_string()))
            .bind(&row.source)
            .bind(imported_at.to_rfc3339())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

fn map_reference_record(row: sqlx::sqlite::SqliteRow) -> Result<IndustryReferenceRecord> {
    Ok(IndustryReferenceRecord {
        code: row.try_get("code")?,
        industry_name: row.try_get("industry_name")?,
        standard: ClassificationStandard::parse(&row.try_get::<String, _>("standard")?)?,
        level: IndustryClassificationLevel::parse(&row.try_get::<String, _>("level")?)?,
        source: row.try_get("source")?,
    })
}

fn map_snapshot_record(row: sqlx::sqlite::SqliteRow) -> Result<IndustrySnapshotRecord> {
    let captured_at_raw: String = row.try_get("captured_at")?;
    let captured_at = DateTime::parse_from_rfc3339(&captured_at_raw)
        .map_err(|err| QuantixError::Other(format!("invalid snapshot captured_at: {err}")))?
        .with_timezone(&Utc);

    Ok(IndustrySnapshotRecord {
        code: row.try_get("code")?,
        industry_name: row.try_get("industry_name")?,
        standard: ClassificationStandard::parse(&row.try_get::<String, _>("standard")?)?,
        level: IndustryClassificationLevel::parse(&row.try_get::<String, _>("level")?)?,
        snapshot_month: row.try_get("snapshot_month")?,
        source: row.try_get("source")?,
        captured_at,
    })
}
