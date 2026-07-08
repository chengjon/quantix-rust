//! Import state tracking (P0.15b).
//!
//! Status rows are written to `quantix.import_state` to record per-code,
//! per-date, per-kind import outcomes. Latest-wins semantics: a rerun
//! queries `ORDER BY imported_at DESC LIMIT 1` to decide whether to skip.

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;

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

/// In-memory `ImportStateStoreTrait` for unit tests.
///
/// Keyed by `(code, date, kind)`. Each value is a Vec of `(Status,
/// batch_id)` pairs in insertion order; `get_status` returns the last
/// entry (matching the production "ORDER BY imported_at DESC LIMIT 1"
/// semantics).
#[cfg(test)]
type MockStateMap = HashMap<(String, NaiveDate, String), Vec<(Status, String)>>;

#[cfg(test)]
#[derive(Debug, Default)]
pub struct MockStateStore {
    inner: Mutex<MockStateMap>,
}

#[cfg(test)]
impl MockStateStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
#[async_trait]
impl ImportStateStoreTrait for MockStateStore {
    async fn get_status(&self, code: &str, date: NaiveDate, kind: &str) -> Result<Option<Status>> {
        let key = (code.to_string(), date, kind.to_string());
        let guard = self.inner.lock().expect("mock state mutex poisoned");
        Ok(guard
            .get(&key)
            .and_then(|history| history.last())
            .map(|(status, _)| status.clone()))
    }

    async fn record(
        &self,
        code: &str,
        date: NaiveDate,
        kind: &str,
        status: &Status,
        batch_id: &str,
    ) -> Result<()> {
        let key = (code.to_string(), date, kind.to_string());
        let mut guard = self.inner.lock().expect("mock state mutex poisoned");
        guard
            .entry(key)
            .or_default()
            .push((status.clone(), batch_id.to_string()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[tokio::test]
    async fn get_status_returns_none_when_no_record() {
        let store = MockStateStore::new();
        let got = store.get_status("sh600000", d(2026, 7, 8), "klines").await;
        assert!(got.unwrap().is_none());
    }

    #[tokio::test]
    async fn record_then_get_returns_latest() {
        let store = MockStateStore::new();
        let date = d(2026, 7, 8);
        store
            .record(
                "sh600000",
                date,
                "klines",
                &Status::Failed {
                    reason: "first attempt".into(),
                },
                "batch-1",
            )
            .await
            .unwrap();
        store
            .record("sh600000", date, "klines", &Status::Success, "batch-1")
            .await
            .unwrap();

        let got = store.get_status("sh600000", date, "klines").await.unwrap();
        assert_eq!(got, Some(Status::Success));
    }

    #[tokio::test]
    async fn different_kinds_are_independent() {
        let store = MockStateStore::new();
        let date = d(2026, 7, 8);
        store
            .record("sh600000", date, "klines", &Status::Success, "b1")
            .await
            .unwrap();
        let klines_status = store.get_status("sh600000", date, "klines").await.unwrap();
        let share_status = store.get_status("sh600000", date, "share").await.unwrap();
        assert_eq!(klines_status, Some(Status::Success));
        assert_eq!(share_status, None);
    }

    #[tokio::test]
    async fn failed_carries_reason() {
        let store = MockStateStore::new();
        let date = d(2026, 7, 8);
        store
            .record(
                "sh600000",
                date,
                "klines",
                &Status::Failed {
                    reason: "OpenStock 404".into(),
                },
                "b1",
            )
            .await
            .unwrap();
        let got = store.get_status("sh600000", date, "klines").await.unwrap();
        match got {
            Some(Status::Failed { reason }) => assert_eq!(reason, "OpenStock 404"),
            other => panic!("expected Failed, got {:?}", other),
        }
    }
}
