//! Strategy request formatting tests, split by concern.
//!
//! Historical note: the original `strategy_request_formatting.rs` was 590
//! lines and was split into submodules per the file-size limits in
//! CLAUDE.md:
//! - [`daemon_summary`] — `format_execution_daemon_summary` tests (idle,
//!   non-terminal order semantics, gate-failure diagnostics, failure
//!   timestamp + reason cases),
//! - [`request_formatting`] — approval/rejection/request-row/request-detail
//!   format tests, qmt_live recovery context, execution diagnostics section,
//!   timestamp + cancellation diagnostics, and the signal-row source
//!   metadata case.

pub(super) use super::*;

mod daemon_summary;
mod request_formatting;
