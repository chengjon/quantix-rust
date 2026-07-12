//! Strategy execution tests, split by concern.
//!
//! Historical note: this file was once 2084 lines and was split into
//! submodules per the file-size limits in CLAUDE.md. Shared helpers
//! (`make_kline`, `fixed_ts`, `sample_run`, `sample_signal`, `FakeLoader`)
//! live in [`super::strategy_helpers`].

// Re-export the parent test harness namespace so submodules can write
// `use super::*;` and pick up everything `tests/mod.rs` already imports.
pub(super) use super::*;

mod config_service;
mod modes;
mod qmt_live;
mod qmt_live_kill_switch;
mod qmt_query_cancel;
mod risk_bridge;
