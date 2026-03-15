use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow};
use std::path::Path;

use crate::core::{QuantixError, Result};
use crate::stop::{StopRule, StopRuleStore};

const CREATE_STOP_RULES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS stop_rules (
    code TEXT PRIMARY KEY,
    stop_loss_price REAL,
    take_profit_price REAL,
    trailing_pct REAL,
    highest_price REAL,
    last_triggered_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
"#;

#[derive(Debug, Clone)]
pub struct SqliteStopRuleStore {
    pool: SqlitePool,
}

impl SqliteStopRuleStore {
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
        let store = Self { pool };
        store.ensure_schema().await?;
        Ok(store)
    }

    async fn ensure_schema(&self) -> Result<()> {
        sqlx::query(CREATE_STOP_RULES_TABLE_SQL)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    fn row_to_rule(row: SqliteRow) -> Result<StopRule> {
        let created_at: String = row.try_get("created_at")?;
        let updated_at: String = row.try_get("updated_at")?;
        let last_triggered_at: Option<String> = row.try_get("last_triggered_at")?;

        Ok(StopRule {
            code: row.try_get("code")?,
            stop_loss_price: row.try_get("stop_loss_price")?,
            take_profit_price: row.try_get("take_profit_price")?,
            trailing_pct: row.try_get("trailing_pct")?,
            highest_price: row.try_get("highest_price")?,
            last_triggered_at: last_triggered_at
                .as_deref()
                .map(parse_timestamp)
                .transpose()?,
            created_at: parse_timestamp(&created_at)?,
            updated_at: parse_timestamp(&updated_at)?,
        })
    }
}

#[async_trait]
impl StopRuleStore for SqliteStopRuleStore {
    async fn upsert_rule(&self, rule: StopRule) -> Result<StopRule> {
        sqlx::query(
            r#"
INSERT INTO stop_rules (
    code,
    stop_loss_price,
    take_profit_price,
    trailing_pct,
    highest_price,
    last_triggered_at,
    created_at,
    updated_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(code) DO UPDATE SET
    stop_loss_price = excluded.stop_loss_price,
    take_profit_price = excluded.take_profit_price,
    trailing_pct = excluded.trailing_pct,
    highest_price = excluded.highest_price,
    last_triggered_at = excluded.last_triggered_at,
    created_at = excluded.created_at,
    updated_at = excluded.updated_at
"#,
        )
        .bind(&rule.code)
        .bind(rule.stop_loss_price)
        .bind(rule.take_profit_price)
        .bind(rule.trailing_pct)
        .bind(rule.highest_price)
        .bind(rule.last_triggered_at.map(|value| value.to_rfc3339()))
        .bind(rule.created_at.to_rfc3339())
        .bind(rule.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(rule)
    }

    async fn list_rules(&self) -> Result<Vec<StopRule>> {
        let rows = sqlx::query(
            r#"
SELECT
    code,
    stop_loss_price,
    take_profit_price,
    trailing_pct,
    highest_price,
    last_triggered_at,
    created_at,
    updated_at
FROM stop_rules
ORDER BY code ASC
"#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(Self::row_to_rule).collect()
    }

    async fn remove_rule(&self, code: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM stop_rules WHERE code = ?")
            .bind(code)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

fn parse_timestamp(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)
        .map_err(|err| QuantixError::DataParse(format!("invalid stored timestamp: {}", err)))?
        .with_timezone(&Utc))
}
