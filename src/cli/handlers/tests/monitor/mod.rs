//! Monitor command tests, split by concern.
//!
//! Historical note: the original `monitor.rs` was 638 lines and was split
//! into submodules per the file-size limits in CLAUDE.md:
//! - [`alerts`] — alert add/list/remove CRUD tests, the invalid-threshold
//!   validation case, the two persist-triggered-alerts timestamp tests, and
//!   the monitor-config show/set round-trip tests,
//! - [`watchlist`] — watchlist once/repeat/stop-rule trigger tests
//!   (fixed-loss, fixed-profit, trailing-update, trailing-drawdown, and the
//!   missing-price no-trigger case).
//!
//! The shared monitor fakes live at the `tests` module level in
//! `monitor_helpers.rs`, so each submodule brings them in via
//! `use super::monitor_helpers::*;`.

pub(super) use super::*;

mod alerts;
mod watchlist;
