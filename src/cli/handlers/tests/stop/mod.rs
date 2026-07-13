//! Stop-rule command tests, split by concern.
//!
//! Historical note: the original `stop.rs` was 547 lines and was split into
//! submodules per the file-size limits in CLAUDE.md:
//! - [`query`] — stop list + remove tests, and the stop status/history row
//!   evaluation test,
//! - [`set_update`] — stop set tests (fixed-loss, fixed-profit, trailing),
//!   the invalid-condition rejection case, the watchlist-enforcement case,
//!   the rule-shape overwrite case, the loss-pct resolution case, and the
//!   update-patch + clear-flags case.
//!
//! The shared helpers (`stop_rule`, `stop_sample_time`, `stop_watchlist_storage`)
//! are `pub(super)` and used by both submodules, so they live here at the
//! `stop` module level.

pub(super) use super::*;

fn stop_sample_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 12, 0, 0).unwrap()
}

pub(super) fn stop_rule(code: &str) -> StopRule {
    StopRule {
        code: code.to_string(),
        stop_loss_price: Some(14.5),
        take_profit_price: None,
        stop_loss_pct: None,
        take_profit_pct: None,
        trailing_pct: None,
        highest_price: None,
        reference_price: None,
        last_triggered_at: None,
        created_at: stop_sample_time(),
        updated_at: stop_sample_time(),
    }
}

pub(super) fn stop_watchlist_storage(codes: &[&str]) -> (tempfile::TempDir, WatchlistStorage) {
    let dir = tempfile::tempdir().unwrap();
    let storage = WatchlistStorage::new(dir.path().join("watchlist.json"));
    let service = WatchlistService::default();
    let mut store = storage.load_or_create().unwrap();
    for code in codes {
        service.add(&mut store, code, None, Utc::now()).unwrap();
    }
    storage.save(&store).unwrap();
    (dir, storage)
}

mod query;
mod set_update;
