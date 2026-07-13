//! Strategy signal request tests, split by concern.
//!
//! Historical note: the original `strategy_requests.rs` was 716 lines and was
//! split into submodules per the file-size limits in CLAUDE.md:
//! - [`requests`] — list/approve/execute/cancel request-path tests,
//! - [`kill_switch`] — kill-switch rejection tests, plus the
//!   `seed_signal_for_kill_switch` and `enable_test_kill_switch` helpers.

pub(super) use super::*;

mod kill_switch;
mod requests;
