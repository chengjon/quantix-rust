//! OpenStock daily minute import scheduler (P0.15b).
//!
//! Iterates the full A-share code list, calls the P0.15a minute import
//! logic per code, tracks success/failure in PostgreSQL
//! (`quantix.import_state`), and continues on per-code errors.

pub mod engine;
pub mod fetcher;
pub mod scheduler;
pub mod state;

pub use engine::{CodeResult, ImportEngine};
#[cfg(test)]
pub use fetcher::MockFetcher;
pub use fetcher::{StockListFetchTrait, StockListFetcher};
pub use scheduler::{BatchScheduler, BatchSummary, FailureEntry, KlineShareCount};
#[cfg(test)]
pub use state::MockStateStore;
pub use state::{ImportStateStore, ImportStateStoreTrait, Status};
