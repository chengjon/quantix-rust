#![allow(clippy::collapsible_if)]

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::{QueryBuilder, Row, Sqlite};
use std::path::Path;

use crate::core::{QuantixError, Result};
use crate::stop::{
    StopHistoryEvent, StopHistoryEventType, StopHistoryFilter, StopHistoryTriggerKind, StopRule,
    StopRuleStore,
};

const CREATE_STOP_RULES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS stop_rules (
    code TEXT PRIMARY KEY,
    stop_loss_price REAL,
    take_profit_price REAL,
    stop_loss_pct REAL,
    take_profit_pct REAL,
    trailing_pct REAL,
    highest_price REAL,
    reference_price REAL,
    last_triggered_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
"#;

const CREATE_STOP_HISTORY_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS stop_history (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL,
    event_type TEXT NOT NULL,
    trigger_kind TEXT,
    trigger_price REAL,
    anchor_price REAL,
    anchor_source TEXT,
    snapshot_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);
"#;

const CREATE_STOP_HISTORY_CODE_CREATED_AT_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_stop_history_code_created_at
ON stop_history(code, created_at)
"#;

const CREATE_STOP_HISTORY_EVENT_CREATED_AT_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_stop_history_event_created_at
ON stop_history(event_type, created_at)
"#;

/// SQLite 后端 StopRuleStore 实现：持有 SqlitePool，维护 stop_rules/stop_history 表与索引；构造时自动跑 migrations。
#[derive(Debug, Clone)]
pub struct SqliteStopRuleStore {
    pool: SqlitePool,
}

impl SqliteStopRuleStore {
    /// 打开（必要时创建父目录并初始化 stop_rules/stop_history 表）SQLite 止损库；传入路径后自动跑 migrations 并返回 SqliteStopRuleStore。文件创建、目录创建或 migration 失败透传。
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
        sqlx::query(CREATE_STOP_HISTORY_TABLE_SQL)
            .execute(&self.pool)
            .await?;
        sqlx::query(CREATE_STOP_HISTORY_CODE_CREATED_AT_INDEX_SQL)
            .execute(&self.pool)
            .await?;
        sqlx::query(CREATE_STOP_HISTORY_EVENT_CREATED_AT_INDEX_SQL)
            .execute(&self.pool)
            .await?;
        self.ensure_stop_rule_schema_extensions().await?;
        Ok(())
    }

    async fn ensure_stop_rule_schema_extensions(&self) -> Result<()> {
        self.ensure_column_exists(
            "stop_rules",
            "stop_loss_pct",
            "ALTER TABLE stop_rules ADD COLUMN stop_loss_pct REAL",
        )
        .await?;
        self.ensure_column_exists(
            "stop_rules",
            "take_profit_pct",
            "ALTER TABLE stop_rules ADD COLUMN take_profit_pct REAL",
        )
        .await?;
        self.ensure_column_exists(
            "stop_rules",
            "reference_price",
            "ALTER TABLE stop_rules ADD COLUMN reference_price REAL",
        )
        .await?;
        Ok(())
    }

    async fn ensure_column_exists(
        &self,
        table_name: &str,
        column_name: &str,
        alter_sql: &str,
    ) -> Result<()> {
        let pragma_sql = format!("PRAGMA table_info({table_name})");
        let rows = sqlx::query(&pragma_sql).fetch_all(&self.pool).await?;
        let has_column = rows.iter().any(|row| {
            row.try_get::<String, _>("name")
                .map(|name| name == column_name)
                .unwrap_or(false)
        });
        if !has_column {
            sqlx::query(alter_sql).execute(&self.pool).await?;
        }
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
            stop_loss_pct: row.try_get("stop_loss_pct")?,
            take_profit_pct: row.try_get("take_profit_pct")?,
            trailing_pct: row.try_get("trailing_pct")?,
            highest_price: row.try_get("highest_price")?,
            reference_price: row.try_get("reference_price")?,
            last_triggered_at: last_triggered_at
                .as_deref()
                .map(parse_timestamp)
                .transpose()?,
            created_at: parse_timestamp(&created_at)?,
            updated_at: parse_timestamp(&updated_at)?,
        })
    }

    fn row_to_history_event(row: SqliteRow) -> Result<StopHistoryEvent> {
        let created_at: String = row.try_get("created_at")?;
        let event_type: String = row.try_get("event_type")?;
        let trigger_kind: Option<String> = row.try_get("trigger_kind")?;
        let snapshot_json: String = row.try_get("snapshot_json")?;
        let parsed_trigger_kind = match trigger_kind.as_deref() {
            Some(value) => Some(StopHistoryTriggerKind::from_str(value).ok_or_else(|| {
                QuantixError::DataParse(format!("invalid stop_history trigger_kind: {value}"))
            })?),
            None => None,
        };

        Ok(StopHistoryEvent {
            id: row.try_get("id")?,
            code: row.try_get("code")?,
            event_type: StopHistoryEventType::from_str(&event_type).ok_or_else(|| {
                QuantixError::DataParse(format!("invalid stop_history event_type: {event_type}"))
            })?,
            trigger_kind: parsed_trigger_kind,
            trigger_price: row.try_get("trigger_price")?,
            anchor_price: row.try_get("anchor_price")?,
            anchor_source: row.try_get("anchor_source")?,
            snapshot_json: serde_json::from_str(&snapshot_json)?,
            created_at: parse_timestamp(&created_at)?,
        })
    }

    /// 按 code 查询单条 stop_rule；未命中返回 Ok(None)，命中时反序列化所有价格/百分比/时间戳字段，任一列读取或类型转换失败返回 DataParse。
    pub async fn get_rule(&self, code: &str) -> Result<Option<StopRule>> {
        let row = sqlx::query(
            r#"
SELECT
    code,
    stop_loss_price,
    take_profit_price,
    stop_loss_pct,
    take_profit_pct,
    trailing_pct,
    highest_price,
    reference_price,
    last_triggered_at,
    created_at,
    updated_at
FROM stop_rules
WHERE code = ?
"#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::row_to_rule).transpose()
    }

    /// 向 stop_history 追加一条事件（主键 id），写入 code/event_type/trigger_kind/trigger_price/anchor_price/anchor_source/snapshot_json/created_at；SQL 执行失败透传。
    pub async fn append_history(&self, event: StopHistoryEvent) -> Result<()> {
        sqlx::query(
            r#"
INSERT INTO stop_history (
    id,
    code,
    event_type,
    trigger_kind,
    trigger_price,
    anchor_price,
    anchor_source,
    snapshot_json,
    created_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&event.id)
        .bind(&event.code)
        .bind(event.event_type.as_str())
        .bind(event.trigger_kind.map(|value| value.as_str()))
        .bind(event.trigger_price)
        .bind(event.anchor_price)
        .bind(event.anchor_source.as_deref())
        .bind(serde_json::to_string(&event.snapshot_json)?)
        .bind(event.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 按 filter 查询 stop_history：可选 code 精确匹配、date 精确匹配、event_type 过滤、limit 截断；默认按 created_at DESC 排序。逐行反序列化为 StopHistoryEvent，任一行失败聚合返回错误。
    pub async fn list_history(&self, filter: StopHistoryFilter) -> Result<Vec<StopHistoryEvent>> {
        let mut builder = QueryBuilder::<Sqlite>::new(
            r#"
SELECT
    id,
    code,
    event_type,
    trigger_kind,
    trigger_price,
    anchor_price,
    anchor_source,
    snapshot_json,
    created_at
FROM stop_history
"#,
        );

        let mut has_where = false;
        if filter.code.is_some() || filter.date.is_some() || filter.event_type.is_some() {
            builder.push(" WHERE ");
        }
        if let Some(code) = filter.code.as_deref() {
            builder.push("code = ").push_bind(code);
            has_where = true;
        }
        if let Some(date) = filter.date {
            if has_where {
                builder.push(" AND ");
            }
            builder
                .push("date(created_at) = ")
                .push_bind(date.format("%Y-%m-%d").to_string());
            has_where = true;
        }
        if let Some(event_type) = filter.event_type {
            if has_where {
                builder.push(" AND ");
            }
            builder.push("event_type = ").push_bind(event_type.as_str());
        }
        builder.push(" ORDER BY created_at DESC");
        if let Some(limit) = filter.limit {
            builder.push(" LIMIT ").push_bind(limit as i64);
        }

        let rows = builder.build().fetch_all(&self.pool).await?;
        rows.into_iter().map(Self::row_to_history_event).collect()
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
    stop_loss_pct,
    take_profit_pct,
    trailing_pct,
    highest_price,
    reference_price,
    last_triggered_at,
    created_at,
    updated_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(code) DO UPDATE SET
    stop_loss_price = excluded.stop_loss_price,
    take_profit_price = excluded.take_profit_price,
    stop_loss_pct = excluded.stop_loss_pct,
    take_profit_pct = excluded.take_profit_pct,
    trailing_pct = excluded.trailing_pct,
    highest_price = excluded.highest_price,
    reference_price = excluded.reference_price,
    last_triggered_at = excluded.last_triggered_at,
    created_at = excluded.created_at,
    updated_at = excluded.updated_at
"#,
        )
        .bind(&rule.code)
        .bind(rule.stop_loss_price)
        .bind(rule.take_profit_price)
        .bind(rule.stop_loss_pct)
        .bind(rule.take_profit_pct)
        .bind(rule.trailing_pct)
        .bind(rule.highest_price)
        .bind(rule.reference_price)
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
    stop_loss_pct,
    take_profit_pct,
    trailing_pct,
    highest_price,
    reference_price,
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

    async fn get_rule(&self, code: &str) -> Result<Option<StopRule>> {
        Self::get_rule(self, code).await
    }

    async fn append_history(&self, event: StopHistoryEvent) -> Result<()> {
        Self::append_history(self, event).await
    }

    async fn list_history(&self, filter: StopHistoryFilter) -> Result<Vec<StopHistoryEvent>> {
        Self::list_history(self, filter).await
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
