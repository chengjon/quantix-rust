//! Import state tracking (P0.15b).
//!
//! Status rows are written to `quantix.import_state` to record per-code,
//! per-date, per-kind import outcomes. Latest-wins semantics: a rerun
//! queries `ORDER BY imported_at DESC LIMIT 1` to decide whether to skip.

use chrono::NaiveDate;

use async_trait::async_trait;

use crate::core::error::Result;

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

// Below: real Postgres implementation and in-memory test impl.
// These will be filled in by Task 2 (real) and Task 3 (mock).

// Placeholder comment — implementations land in Task 2 and Task 3:
// pub struct ImportStateStore { ... }   // Task 2
// pub struct MockStateStore { ... }     // Task 3
