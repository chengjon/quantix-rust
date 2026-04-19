#![allow(clippy::collapsible_if)]

use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow};
use std::path::Path;
use uuid::Uuid;

use crate::core::{QuantixError, Result};
use crate::risk::{
    LiveImportBatchSummary, LiveImportConflict, LiveImportMirrorAccount, LiveImportRecord,
};

mod parsing;

use parsing::{parse_decimal, parse_timestamp, row_to_mirror_position};

const CREATE_LIVE_IMPORT_BATCHES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS live_import_batches (
    batch_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    input_name TEXT NOT NULL,
    imported_at TEXT NOT NULL,
    total_rows INTEGER NOT NULL,
    inserted_rows INTEGER NOT NULL,
    skipped_duplicates INTEGER NOT NULL,
    conflict_rows INTEGER NOT NULL
);
"#;

const CREATE_LIVE_IMPORT_RECORDS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS live_import_records (
    account_id TEXT NOT NULL,
    external_id TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    imported_at TEXT NOT NULL,
    batch_id TEXT NOT NULL,
    PRIMARY KEY (account_id, external_id)
);
"#;

const CREATE_LIVE_IMPORT_CONFLICTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS live_import_conflicts (
    id TEXT PRIMARY KEY,
    batch_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    external_id TEXT NOT NULL,
    existing_record_json TEXT NOT NULL,
    incoming_record_json TEXT NOT NULL,
    detail TEXT NOT NULL,
    created_at TEXT NOT NULL
);
"#;

const CREATE_LIVE_IMPORT_MIRROR_ACCOUNTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS live_import_mirror_accounts (
    account_id TEXT PRIMARY KEY,
    trading_date TEXT NOT NULL,
    as_of TEXT NOT NULL,
    starting_total_assets TEXT NOT NULL,
    current_total_assets TEXT NOT NULL,
    cash_balance TEXT NOT NULL,
    realized_pnl TEXT NOT NULL,
    total_fees TEXT NOT NULL,
    last_rebuild_at TEXT NOT NULL
);
"#;

const CREATE_LIVE_IMPORT_MIRROR_POSITIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS live_import_mirror_positions (
    account_id TEXT NOT NULL,
    code TEXT NOT NULL,
    volume INTEGER NOT NULL,
    avg_cost TEXT NOT NULL,
    last_trade_at TEXT NOT NULL,
    PRIMARY KEY (account_id, code)
);
"#;

const CREATE_LIVE_IMPORT_REBUILDS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS live_import_rebuilds (
    rebuild_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    status TEXT NOT NULL,
    detail TEXT,
    rebuilt_at TEXT NOT NULL
);
"#;

#[derive(Debug, Clone)]
pub struct SqliteLiveImportStore {
    pool: SqlitePool,
}

impl SqliteLiveImportStore {
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
        for statement in [
            CREATE_LIVE_IMPORT_BATCHES_TABLE_SQL,
            CREATE_LIVE_IMPORT_RECORDS_TABLE_SQL,
            CREATE_LIVE_IMPORT_CONFLICTS_TABLE_SQL,
            CREATE_LIVE_IMPORT_MIRROR_ACCOUNTS_TABLE_SQL,
            CREATE_LIVE_IMPORT_MIRROR_POSITIONS_TABLE_SQL,
            CREATE_LIVE_IMPORT_REBUILDS_TABLE_SQL,
        ] {
            sqlx::query(statement).execute(&self.pool).await?;
        }
        Ok(())
    }

    pub async fn import_records(
        &self,
        account_id: &str,
        input_name: &str,
        records: &[LiveImportRecord],
        imported_at: DateTime<Utc>,
    ) -> Result<LiveImportBatchSummary> {
        let batch_id = Uuid::new_v4().to_string();
        let mut inserted = 0usize;
        let mut skipped_duplicates = 0usize;
        let mut conflicts = 0usize;

        for record in records {
            if record.account_id != account_id {
                return Err(QuantixError::Other(format!(
                    "risk import record account_id {} 与命令 account {} 不一致",
                    record.account_id, account_id
                )));
            }

            let existing = sqlx::query(
                "SELECT payload_json FROM live_import_records WHERE account_id = ? AND external_id = ?",
            )
            .bind(&record.account_id)
            .bind(&record.external_id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some(row) = existing {
                let existing_json: String = row.try_get("payload_json")?;
                let existing_record: LiveImportRecord = serde_json::from_str(&existing_json)?;
                if existing_record == *record {
                    skipped_duplicates += 1;
                } else {
                    conflicts += 1;
                    self.insert_conflict(
                        &batch_id,
                        &existing_record,
                        record,
                        "duplicate external_id with different payload",
                        imported_at,
                    )
                    .await?;
                }
                continue;
            }

            sqlx::query(
                r#"
INSERT INTO live_import_records (
    account_id,
    external_id,
    payload_json,
    imported_at,
    batch_id
) VALUES (?, ?, ?, ?, ?)
"#,
            )
            .bind(&record.account_id)
            .bind(&record.external_id)
            .bind(serde_json::to_string(record)?)
            .bind(imported_at.to_rfc3339())
            .bind(&batch_id)
            .execute(&self.pool)
            .await?;
            inserted += 1;
        }

        sqlx::query(
            r#"
INSERT INTO live_import_batches (
    batch_id,
    account_id,
    input_name,
    imported_at,
    total_rows,
    inserted_rows,
    skipped_duplicates,
    conflict_rows
) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&batch_id)
        .bind(account_id)
        .bind(input_name)
        .bind(imported_at.to_rfc3339())
        .bind(records.len() as i64)
        .bind(inserted as i64)
        .bind(skipped_duplicates as i64)
        .bind(conflicts as i64)
        .execute(&self.pool)
        .await?;

        Ok(LiveImportBatchSummary {
            batch_id,
            account_id: account_id.to_string(),
            total_rows: records.len(),
            inserted,
            skipped_duplicates,
            conflicts,
        })
    }

    pub async fn list_records(&self, account_id: &str) -> Result<Vec<LiveImportRecord>> {
        let rows = sqlx::query(
            r#"
SELECT payload_json
FROM live_import_records
WHERE account_id = ?
ORDER BY external_id ASC
"#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let payload_json: String = row.try_get("payload_json")?;
                serde_json::from_str(&payload_json).map_err(QuantixError::from)
            })
            .collect()
    }

    pub async fn list_conflicts(&self, batch_id: &str) -> Result<Vec<LiveImportConflict>> {
        let rows = sqlx::query(
            r#"
SELECT
    id,
    batch_id,
    account_id,
    external_id,
    existing_record_json,
    incoming_record_json,
    detail,
    created_at
FROM live_import_conflicts
WHERE batch_id = ?
ORDER BY created_at ASC
"#,
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_conflict).collect()
    }

    pub async fn replace_mirror_account(&self, mirror: &LiveImportMirrorAccount) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
INSERT INTO live_import_mirror_accounts (
    account_id,
    trading_date,
    as_of,
    starting_total_assets,
    current_total_assets,
    cash_balance,
    realized_pnl,
    total_fees,
    last_rebuild_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(account_id) DO UPDATE SET
    trading_date = excluded.trading_date,
    as_of = excluded.as_of,
    starting_total_assets = excluded.starting_total_assets,
    current_total_assets = excluded.current_total_assets,
    cash_balance = excluded.cash_balance,
    realized_pnl = excluded.realized_pnl,
    total_fees = excluded.total_fees,
    last_rebuild_at = excluded.last_rebuild_at
"#,
        )
        .bind(&mirror.account_id)
        .bind(mirror.trading_date.to_string())
        .bind(mirror.as_of.to_rfc3339())
        .bind(mirror.starting_total_assets.to_string())
        .bind(mirror.current_total_assets.to_string())
        .bind(mirror.cash_balance.to_string())
        .bind(mirror.realized_pnl.to_string())
        .bind(mirror.total_fees.to_string())
        .bind(mirror.last_rebuild_at.to_rfc3339())
        .execute(&mut *tx)
        .await?;

        sqlx::query("DELETE FROM live_import_mirror_positions WHERE account_id = ?")
            .bind(&mirror.account_id)
            .execute(&mut *tx)
            .await?;

        for position in &mirror.positions {
            sqlx::query(
                r#"
INSERT INTO live_import_mirror_positions (
    account_id,
    code,
    volume,
    avg_cost,
    last_trade_at
) VALUES (?, ?, ?, ?, ?)
"#,
            )
            .bind(&mirror.account_id)
            .bind(&position.code)
            .bind(position.volume)
            .bind(position.avg_cost.to_string())
            .bind(position.last_trade_at.to_rfc3339())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_latest_mirror_account(
        &self,
        account_id: &str,
    ) -> Result<Option<LiveImportMirrorAccount>> {
        let row = sqlx::query(
            r#"
SELECT
    account_id,
    trading_date,
    as_of,
    starting_total_assets,
    current_total_assets,
    cash_balance,
    realized_pnl,
    total_fees,
    last_rebuild_at
FROM live_import_mirror_accounts
WHERE account_id = ?
"#,
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let position_rows = sqlx::query(
            r#"
SELECT code, volume, avg_cost, last_trade_at
FROM live_import_mirror_positions
WHERE account_id = ?
ORDER BY code ASC
"#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(LiveImportMirrorAccount {
            account_id: row.try_get("account_id")?,
            trading_date: chrono::NaiveDate::parse_from_str(
                &row.try_get::<String, _>("trading_date")?,
                "%Y-%m-%d",
            )
            .map_err(|err| QuantixError::DataParse(format!("invalid trading_date: {err}")))?,
            as_of: parse_timestamp(&row.try_get::<String, _>("as_of")?)?,
            starting_total_assets: parse_decimal(
                &row.try_get::<String, _>("starting_total_assets")?,
                "starting_total_assets",
            )?,
            current_total_assets: parse_decimal(
                &row.try_get::<String, _>("current_total_assets")?,
                "current_total_assets",
            )?,
            cash_balance: parse_decimal(
                &row.try_get::<String, _>("cash_balance")?,
                "cash_balance",
            )?,
            realized_pnl: parse_decimal(
                &row.try_get::<String, _>("realized_pnl")?,
                "realized_pnl",
            )?,
            total_fees: parse_decimal(&row.try_get::<String, _>("total_fees")?, "total_fees")?,
            last_rebuild_at: parse_timestamp(&row.try_get::<String, _>("last_rebuild_at")?)?,
            positions: position_rows
                .into_iter()
                .map(row_to_mirror_position)
                .collect::<Result<Vec<_>>>()?,
        }))
    }

    pub async fn append_rebuild_audit(
        &self,
        account_id: &str,
        status: &str,
        detail: Option<&str>,
        rebuilt_at: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            r#"
INSERT INTO live_import_rebuilds (
    rebuild_id,
    account_id,
    status,
    detail,
    rebuilt_at
) VALUES (?, ?, ?, ?, ?)
"#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(account_id)
        .bind(status)
        .bind(detail)
        .bind(rebuilt_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn insert_conflict(
        &self,
        batch_id: &str,
        existing_record: &LiveImportRecord,
        incoming_record: &LiveImportRecord,
        detail: &str,
        created_at: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            r#"
INSERT INTO live_import_conflicts (
    id,
    batch_id,
    account_id,
    external_id,
    existing_record_json,
    incoming_record_json,
    detail,
    created_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(batch_id)
        .bind(&incoming_record.account_id)
        .bind(&incoming_record.external_id)
        .bind(serde_json::to_string(existing_record)?)
        .bind(serde_json::to_string(incoming_record)?)
        .bind(detail)
        .bind(created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

fn row_to_conflict(row: SqliteRow) -> Result<LiveImportConflict> {
    let created_at: String = row.try_get("created_at")?;
    let existing_record_json: String = row.try_get("existing_record_json")?;
    let incoming_record_json: String = row.try_get("incoming_record_json")?;

    Ok(LiveImportConflict {
        id: row.try_get("id")?,
        batch_id: row.try_get("batch_id")?,
        account_id: row.try_get("account_id")?,
        external_id: row.try_get("external_id")?,
        existing_record_json: serde_json::from_str(&existing_record_json)?,
        incoming_record_json: serde_json::from_str(&incoming_record_json)?,
        detail: row.try_get("detail")?,
        created_at: DateTime::parse_from_rfc3339(&created_at)
            .map_err(|err| QuantixError::DataParse(format!("invalid stored timestamp: {err}")))?
            .with_timezone(&Utc),
    })
}
