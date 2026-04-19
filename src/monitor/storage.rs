#![allow(clippy::collapsible_if)]

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow};
use std::path::Path;

use crate::core::{QuantixError, Result};
use crate::monitor::{
    MonitorAlertStore, MonitorEventFilter, MonitorEventRow, MonitorEventType, MonitorRunMode,
    NewMonitorEvent, PriceAlert, PriceAlertKind,
};

const CREATE_PRICE_ALERTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS price_alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL,
    alert_type TEXT NOT NULL,
    target_price REAL NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    last_triggered_at TEXT
);
"#;

const CREATE_MONITOR_EVENTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS monitor_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_time TEXT NOT NULL,
    event_type TEXT NOT NULL,
    code TEXT NOT NULL,
    price REAL,
    message TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_key TEXT NOT NULL,
    observed_at TEXT,
    run_mode TEXT NOT NULL
);
"#;

const CREATE_MONITOR_TRIGGER_STATES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS monitor_trigger_states (
    source_type TEXT NOT NULL,
    source_key TEXT NOT NULL,
    is_triggered INTEGER NOT NULL,
    last_transition_at TEXT NOT NULL,
    PRIMARY KEY (source_type, source_key)
);
"#;

#[derive(Debug, Clone)]
pub struct SqliteMonitorAlertStore {
    pool: SqlitePool,
}

impl SqliteMonitorAlertStore {
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
        sqlx::query(CREATE_PRICE_ALERTS_TABLE_SQL)
            .execute(&self.pool)
            .await?;
        sqlx::query(CREATE_MONITOR_EVENTS_TABLE_SQL)
            .execute(&self.pool)
            .await?;
        sqlx::query(CREATE_MONITOR_TRIGGER_STATES_TABLE_SQL)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    fn row_to_alert(row: SqliteRow) -> Result<PriceAlert> {
        let alert_type: String = row.try_get("alert_type")?;
        let created_at: String = row.try_get("created_at")?;
        let last_triggered_at: Option<String> = row.try_get("last_triggered_at")?;

        Ok(PriceAlert {
            id: row.try_get("id")?,
            code: row.try_get("code")?,
            kind: parse_alert_kind(&alert_type)?,
            target_price: row.try_get("target_price")?,
            created_at: parse_timestamp(&created_at)?,
            last_triggered_at: last_triggered_at
                .as_deref()
                .map(parse_timestamp)
                .transpose()?,
        })
    }

    fn row_to_event(row: SqliteRow) -> Result<MonitorEventRow> {
        let event_time: String = row.try_get("event_time")?;
        let event_type: String = row.try_get("event_type")?;
        let observed_at: Option<String> = row.try_get("observed_at")?;
        let run_mode: String = row.try_get("run_mode")?;

        Ok(MonitorEventRow {
            id: row.try_get("id")?,
            event_time: parse_timestamp(&event_time)?,
            event_type: parse_event_type(&event_type)?,
            code: row.try_get("code")?,
            price: row.try_get("price")?,
            message: row.try_get("message")?,
            source_type: row.try_get("source_type")?,
            source_key: row.try_get("source_key")?,
            observed_at: observed_at.as_deref().map(parse_timestamp).transpose()?,
            run_mode: parse_run_mode(&run_mode)?,
        })
    }

    pub async fn record_event_edge(
        &self,
        source_type: &str,
        source_key: &str,
        is_triggered: bool,
        new_event: Option<NewMonitorEvent>,
        max_event_history: usize,
    ) -> Result<bool> {
        let existing_state: Option<i64> = sqlx::query_scalar(
            "SELECT is_triggered FROM monitor_trigger_states WHERE source_type = ? AND source_key = ?",
        )
        .bind(source_type)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await?;
        let was_triggered = existing_state.unwrap_or(0) != 0;

        if is_triggered {
            let event = new_event.ok_or_else(|| {
                QuantixError::Other(
                    "record_event_edge requires event payload when triggered".into(),
                )
            })?;

            if was_triggered {
                return Ok(false);
            }

            sqlx::query(
                "INSERT INTO monitor_events (event_time, event_type, code, price, message, source_type, source_key, observed_at, run_mode) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(event.event_time.to_rfc3339())
            .bind(format_event_type(event.event_type))
            .bind(&event.code)
            .bind(event.price)
            .bind(&event.message)
            .bind(&event.source_type)
            .bind(&event.source_key)
            .bind(event.observed_at.map(|value| value.to_rfc3339()))
            .bind(format_run_mode(event.run_mode))
            .execute(&self.pool)
            .await?;

            sqlx::query(
                "INSERT INTO monitor_trigger_states (source_type, source_key, is_triggered, last_transition_at) VALUES (?, ?, 1, ?)
                 ON CONFLICT(source_type, source_key) DO UPDATE SET is_triggered = excluded.is_triggered, last_transition_at = excluded.last_transition_at",
            )
            .bind(source_type)
            .bind(source_key)
            .bind(event.event_time.to_rfc3339())
            .execute(&self.pool)
            .await?;

            self.trim_event_history(max_event_history).await?;
            Ok(true)
        } else {
            sqlx::query(
                "INSERT INTO monitor_trigger_states (source_type, source_key, is_triggered, last_transition_at) VALUES (?, ?, 0, ?)
                 ON CONFLICT(source_type, source_key) DO UPDATE SET is_triggered = excluded.is_triggered, last_transition_at = excluded.last_transition_at",
            )
            .bind(source_type)
            .bind(source_key)
            .bind(Utc::now().to_rfc3339())
            .execute(&self.pool)
            .await?;

            Ok(false)
        }
    }

    pub async fn list_events(&self, filter: &MonitorEventFilter) -> Result<Vec<MonitorEventRow>> {
        let rows = sqlx::query(
            "SELECT id, event_time, event_type, code, price, message, source_type, source_key, observed_at, run_mode
             FROM monitor_events
             WHERE (?1 IS NULL OR code = ?1)
               AND (?2 IS NULL OR event_type = ?2)
             ORDER BY event_time DESC, id DESC
             LIMIT ?3",
        )
        .bind(filter.code.as_deref())
        .bind(filter.event_type.map(format_event_type))
        .bind(i64::try_from(filter.limit).map_err(|_| {
            QuantixError::Other(format!("event list limit out of range: {}", filter.limit))
        })?)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(Self::row_to_event).collect()
    }

    async fn trim_event_history(&self, max_event_history: usize) -> Result<()> {
        let max_rows = i64::try_from(max_event_history).map_err(|_| {
            QuantixError::Other(format!(
                "max_event_history out of range: {}",
                max_event_history
            ))
        })?;
        sqlx::query(
            "DELETE FROM monitor_events
             WHERE id IN (
                 SELECT id
                 FROM monitor_events
                 ORDER BY event_time DESC, id DESC
                 LIMIT -1 OFFSET ?
             )",
        )
        .bind(max_rows)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[async_trait]
impl MonitorAlertStore for SqliteMonitorAlertStore {
    async fn add_alert(
        &self,
        code: &str,
        kind: PriceAlertKind,
        target_price: f64,
        now: DateTime<Utc>,
    ) -> Result<PriceAlert> {
        let result = sqlx::query(
            "INSERT INTO price_alerts (code, alert_type, target_price, is_active, created_at) VALUES (?, ?, ?, 1, ?)",
        )
        .bind(code)
        .bind(format_alert_kind(kind))
        .bind(target_price)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(PriceAlert {
            id: result.last_insert_rowid(),
            code: code.to_string(),
            kind,
            target_price,
            created_at: now,
            last_triggered_at: None,
        })
    }

    async fn list_alerts(&self) -> Result<Vec<PriceAlert>> {
        let rows = sqlx::query(
            "SELECT id, code, alert_type, target_price, created_at, last_triggered_at FROM price_alerts WHERE is_active = 1 ORDER BY id ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(Self::row_to_alert).collect()
    }

    async fn remove_alert(&self, id: i64) -> Result<bool> {
        let result =
            sqlx::query("UPDATE price_alerts SET is_active = 0 WHERE id = ? AND is_active = 1")
                .bind(id)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn mark_triggered(&self, id: i64, triggered_at: DateTime<Utc>) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE price_alerts SET last_triggered_at = ? WHERE id = ? AND is_active = 1",
        )
        .bind(triggered_at.to_rfc3339())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

fn format_alert_kind(kind: PriceAlertKind) -> &'static str {
    match kind {
        PriceAlertKind::Above => "above",
        PriceAlertKind::Below => "below",
    }
}

fn parse_alert_kind(value: &str) -> Result<PriceAlertKind> {
    match value {
        "above" => Ok(PriceAlertKind::Above),
        "below" => Ok(PriceAlertKind::Below),
        other => Err(QuantixError::DataParse(format!(
            "unknown alert type in sqlite store: {}",
            other
        ))),
    }
}

fn format_event_type(event_type: MonitorEventType) -> &'static str {
    match event_type {
        MonitorEventType::PriceAlert => "price-alert",
        MonitorEventType::StopLoss => "stop-loss",
        MonitorEventType::StopProfit => "stop-profit",
        MonitorEventType::TrailingStop => "trailing-stop",
    }
}

fn parse_event_type(value: &str) -> Result<MonitorEventType> {
    match value {
        "price-alert" => Ok(MonitorEventType::PriceAlert),
        "stop-loss" => Ok(MonitorEventType::StopLoss),
        "stop-profit" => Ok(MonitorEventType::StopProfit),
        "trailing-stop" => Ok(MonitorEventType::TrailingStop),
        other => Err(QuantixError::DataParse(format!(
            "unknown monitor event type in sqlite store: {}",
            other
        ))),
    }
}

fn format_run_mode(run_mode: MonitorRunMode) -> &'static str {
    match run_mode {
        MonitorRunMode::Foreground => "foreground",
        MonitorRunMode::Daemon => "daemon",
    }
}

fn parse_run_mode(value: &str) -> Result<MonitorRunMode> {
    match value {
        "foreground" => Ok(MonitorRunMode::Foreground),
        "daemon" => Ok(MonitorRunMode::Daemon),
        other => Err(QuantixError::DataParse(format!(
            "unknown monitor run mode in sqlite store: {}",
            other
        ))),
    }
}

fn parse_timestamp(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)
        .map_err(|err| QuantixError::DataParse(format!("invalid stored timestamp: {}", err)))?
        .with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::SqliteMonitorAlertStore;
    use crate::monitor::{MonitorAlertStore, PriceAlertKind};
    use chrono::{TimeZone, Utc};
    use tempfile::tempdir;

    fn sample_time() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 11, 10, 0, 0).unwrap()
    }

    #[tokio::test]
    async fn monitor_db_storage_creates_schema_automatically() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("alerts.db");

        let store = SqliteMonitorAlertStore::new(&db_path).await.unwrap();
        let alerts = store.list_alerts().await.unwrap();

        assert!(alerts.is_empty());
    }

    #[tokio::test]
    async fn monitor_db_storage_add_list_remove_round_trips_alert() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("alerts.db");
        let store = SqliteMonitorAlertStore::new(&db_path).await.unwrap();

        let created = store
            .add_alert("000001", PriceAlertKind::Above, 16.0, sample_time())
            .await
            .unwrap();
        assert!(created.id > 0);

        let alerts = store.list_alerts().await.unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].code, "000001");
        assert_eq!(alerts[0].kind, PriceAlertKind::Above);
        assert_eq!(alerts[0].target_price, 16.0);
        assert_eq!(alerts[0].last_triggered_at, None);

        let removed = store.remove_alert(created.id).await.unwrap();
        assert!(removed);
        assert!(store.list_alerts().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn monitor_db_storage_updates_last_triggered_at() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("alerts.db");
        let store = SqliteMonitorAlertStore::new(&db_path).await.unwrap();
        let created = store
            .add_alert("000001", PriceAlertKind::Below, 15.0, sample_time())
            .await
            .unwrap();
        let triggered_at = Utc.with_ymd_and_hms(2026, 3, 11, 10, 5, 0).unwrap();

        let updated = store
            .mark_triggered(created.id, triggered_at)
            .await
            .unwrap();

        assert!(updated);
        let alerts = store.list_alerts().await.unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].last_triggered_at, Some(triggered_at));
    }

    #[tokio::test]
    async fn monitor_db_storage_trait_boundary_supports_mark_triggered() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("alerts.db");
        let store = SqliteMonitorAlertStore::new(&db_path).await.unwrap();
        let created = store
            .add_alert("000001", PriceAlertKind::Above, 16.0, sample_time())
            .await
            .unwrap();
        let triggered_at = Utc.with_ymd_and_hms(2026, 3, 11, 10, 6, 0).unwrap();
        let trait_store: &dyn MonitorAlertStore = &store;

        let updated = trait_store
            .mark_triggered(created.id, triggered_at)
            .await
            .unwrap();

        assert!(updated);
        let alerts = trait_store.list_alerts().await.unwrap();
        assert_eq!(alerts[0].last_triggered_at, Some(triggered_at));
    }

    #[tokio::test]
    async fn monitor_db_storage_persists_alerts_across_reopen() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("alerts.db");
        let triggered_at = Utc.with_ymd_and_hms(2026, 3, 11, 10, 7, 0).unwrap();

        let created_id = {
            let store = SqliteMonitorAlertStore::new(&db_path).await.unwrap();
            let created = store
                .add_alert("000001", PriceAlertKind::Below, 15.0, sample_time())
                .await
                .unwrap();
            let updated = store
                .mark_triggered(created.id, triggered_at)
                .await
                .unwrap();

            assert!(updated);
            created.id
        };

        let reopened = SqliteMonitorAlertStore::new(&db_path).await.unwrap();
        let alerts = reopened.list_alerts().await.unwrap();

        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].id, created_id);
        assert_eq!(alerts[0].code, "000001");
        assert_eq!(alerts[0].kind, PriceAlertKind::Below);
        assert_eq!(alerts[0].target_price, 15.0);
        assert_eq!(alerts[0].last_triggered_at, Some(triggered_at));
    }
}
