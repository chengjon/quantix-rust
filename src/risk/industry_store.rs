#![allow(clippy::too_many_arguments)]

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

/// 行业分类的 SQLite 持久化存储，承载行业参考表（current/history）与月度快照。
#[derive(Debug, Clone)]
pub struct SqliteIndustryStore {
    path: PathBuf,
    pool: SqlitePool,
}

impl SqliteIndustryStore {
    /// 打开（必要时创建）指定路径的 SQLite 库并建表，返回可用实例。
    pub async fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
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

    /// 由 risk_state 文件路径推导 industry_reference.db 位置并打开库。
    pub async fn from_risk_state_path(risk_state_path: impl AsRef<Path>) -> Result<Self> {
        Self::new(Self::default_db_path_from_risk_state(risk_state_path)).await
    }

    /// 由 risk_state 文件路径推导 industry_reference.db 的默认位置（同级目录）。
    pub fn default_db_path_from_risk_state(risk_state_path: impl AsRef<Path>) -> PathBuf {
        let risk_state_path = risk_state_path.as_ref();
        match risk_state_path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.join("industry_reference.db"),
            _ => risk_state_path.with_file_name("industry_reference.db"),
        }
    }

    /// 返回底层 SQLite 文件路径。
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

    /// 按 (standard, level, code) 查询当前行业分类；code 会被标准化（去交易所前缀）。
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

    /// 列出指定 (standard, level) 下的全部当前分类记录，按 code 升序。
    pub async fn list_current(
        &self,
        standard: ClassificationStandard,
        level: IndustryClassificationLevel,
    ) -> Result<Vec<IndustryReferenceRecord>> {
        let rows = sqlx::query(
            r#"
SELECT standard, level, code, industry_name, source
FROM industry_reference_current
WHERE standard = ? AND level = ?
ORDER BY code ASC
"#,
        )
        .bind(standard.as_str())
        .bind(level.as_str())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(map_reference_record).collect()
    }

    /// 按指定日期查询历史分类：取 `effective_from <= date` 且（`effective_to` 为空或 `>= date`）中最近一条。
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

    /// 按 (standard, level, snapshot_month, code) 精确查询月度快照记录。
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

    /// 查询指定 code 的最新一条月度快照（按 snapshot_month desc, captured_at desc）。
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

    /// 若 (standard, level, snapshot_month, code) 不存在则插入快照；已存在时静默跳过（INSERT OR IGNORE）。
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

    /// 批量 upsert 申万当前分类行；冲突时更新 industry_name/source/updated_at。
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

    /// 单事务内清空申万当前分类并重新写入全部行，失败回滚。
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

    /// 批量 upsert 申万历史分类行；冲突时更新 industry_name/effective_to/source/updated_at。
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

    /// 单事务内清空申万历史分类并重新写入全部行，失败回滚。
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
