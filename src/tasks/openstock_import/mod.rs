//! OpenStock daily minute import scheduler (P0.15b).
//!
//! Iterates the full A-share code list, calls the P0.15a minute import
//! logic per code, tracks success/failure in PostgreSQL
//! (`quantix.import_state`), and continues on per-code errors.

pub mod engine;
pub mod fetcher;
pub mod scheduler;
pub mod state;

#[cfg(test)]
pub use fetcher::MockFetcher;
pub use fetcher::{StockListFetchTrait, StockListFetcher};
pub use state::{ImportStateStore, ImportStateStoreTrait, Status};
