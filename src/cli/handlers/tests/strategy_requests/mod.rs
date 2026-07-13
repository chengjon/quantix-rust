//! Strategy signal request tests, split by concern.
//!
//! Historical note: the original `strategy_requests.rs` was 716 lines and was
//! split into submodules per the file-size limits in CLAUDE.md. The first
//! split left `requests.rs` at >500L, so it was split again:
//! - [`list_approve`] — signal list/approve/reject + request-list read-path
//!   tests, plus the live-target-mode early-rejection cases,
//! - [`execute_cancel`] — request execute + cancel write-path tests,
//! - [`kill_switch`] — kill-switch rejection tests, plus the
//!   `seed_signal_for_kill_switch` and `enable_test_kill_switch` helpers,
//! - [`source_contract`] — a plain `#[test]` that asserts the manual
//!   qmt_live handler source retains YES-confirmation + request-guidance
//!   strings and drops the legacy qmt-query hint.

pub(super) use super::*;

mod execute_cancel;
mod kill_switch;
mod list_approve;
mod source_contract;
