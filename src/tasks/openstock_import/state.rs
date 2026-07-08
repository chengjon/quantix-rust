//! Import state tracking (P0.15b).
//!
//! Status rows are written to `quantix.import_state` to record per-code,
//! per-date, per-kind import outcomes. Latest-wins semantics: a rerun
//! queries `ORDER BY imported_at DESC LIMIT 1` to decide whether to skip.

use chrono::NaiveDate;

use async_trait::async_trait;

use crate::core::error::Result;
use crate::db::PostgresClient;

/// Outcome of importing one (code, date, kind) tuple.
///
/// `Success` means the inner import function returned `Ok(stats)`.
/// `Failed` carries the error message; the batch continues to the next
/// code regardless.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Success,
    Failed { reason: String },
}

impl Status {
    /// True if this status should cause a rerun to skip the code.
    pub fn is_success(&self) -> bool {
        matches!(self, Status::Success)
    }

    /// String representation persisted to `import_state.status`.
    /// Used by ImportStateStore::record; do not change without a migration.
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Status::Success => "success",
            Status::Failed { .. } => "failed",
        }
    }
}

/// Read/write interface for `quantix.import_state`.
///
/// The trait abstraction exists so `ImportEngine` and `BatchScheduler`
/// can be unit-tested with an in-memory implementation. The production
/// implementation is `ImportStateStore`; tests use `MockStateStore`.
#[async_trait]
pub trait ImportStateStoreTrait: Send + Sync {
    /// Latest status for (code, date, kind), or `None` if no record.
    /// Implementations must return the row with the newest `imported_at`.
    async fn get_status(&self, code: &str, date: NaiveDate, kind: &str) -> Result<Option<Status>>;

    /// Append a status row. Multiple writes for the same (code, date, kind)
    /// are kept as history; consumers query with `get_status` to find the
    /// latest.
    async fn record(
        &self,
        code: &str,
        date: NaiveDate,
        kind: &str,
        status: &Status,
        batch_id: &str,
    ) -> Result<()>;
}

/// Production `ImportStateStoreTrait` impl backed by PostgreSQL.
///
/// Schema (assumed shipped by P0.15b-pre):
/// ```sql
/// CREATE TABLE quantix.import_state (
///     code         VARCHAR(16) NOT NULL,
///     trade_date   DATE NOT NULL,
///     kind         VARCHAR(8) NOT NULL CHECK (kind IN ('klines', 'share')),
///     status       VARCHAR(8) NOT NULL CHECK (status IN ('success', 'failed')),
///     reason       TEXT,
///     batch_id     VARCHAR(40) NOT NULL,
///     imported_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
///     PRIMARY KEY (code, trade_date, kind, imported_at)
/// );
/// ```
pub struct ImportStateStore<'a> {
    pg: &'a PostgresClient,
}

impl<'a> ImportStateStore<'a> {
    pub fn new(pg: &'a PostgresClient) -> Self {
        Self { pg }
    }
}

#[async_trait]
impl<'a> ImportStateStoreTrait for ImportStateStore<'a> {
    async fn get_status(&self, code: &str, date: NaiveDate, kind: &str) -> Result<Option<Status>> {
        let row: Option<(String, Option<String>)> = sqlx::query_as(
            "SELECT status, reason FROM quantix.import_state \
             WHERE code = $1 AND trade_date = $2 AND kind = $3 \
             ORDER BY imported_at DESC LIMIT 1",
        )
        .bind(code)
        .bind(date)
        .bind(kind)
        .fetch_optional(self.pg.pool())
        .await
        .map_err(|e| crate::core::error::QuantixError::DatabaseQuery(e.to_string()))?;

        Ok(row.map(|(status, reason)| {
            if status == "success" {
                Status::Success
            } else {
                Status::Failed {
                    reason: reason.unwrap_or_else(|| "unknown".to_string()),
                }
            }
        }))
    }

    async fn record(
        &self,
        code: &str,
        date: NaiveDate,
        kind: &str,
        status: &Status,
        batch_id: &str,
    ) -> Result<()> {
        let (status_str, reason_str): (&str, Option<&str>) = match status {
            Status::Success => ("success", None),
            Status::Failed { reason } => ("failed", Some(reason.as_str())),
        };
        sqlx::query(
            "INSERT INTO quantix.import_state \
             (code, trade_date, kind, status, reason, batch_id) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(code)
        .bind(date)
        .bind(kind)
        .bind(status_str)
        .bind(reason_str)
        .bind(batch_id)
        .execute(self.pg.pool())
        .await
        .map_err(|e| crate::core::error::QuantixError::DatabaseQuery(e.to_string()))?;
        Ok(())
    }
}
