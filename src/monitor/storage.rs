use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow};
use std::path::Path;

use crate::core::{QuantixError, Result};
use crate::monitor::{MonitorAlertStore, PriceAlert, PriceAlertKind};

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
        Ok(())
    }

    pub async fn mark_triggered(&self, id: i64, triggered_at: DateTime<Utc>) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE price_alerts SET last_triggered_at = ? WHERE id = ? AND is_active = 1",
        )
        .bind(triggered_at.to_rfc3339())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
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
}
