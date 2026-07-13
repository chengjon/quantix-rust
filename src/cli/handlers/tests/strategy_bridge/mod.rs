//! Strategy bridge execution + preflight tests, split by concern.
//!
//! Historical note: the original `strategy_bridge.rs` was 687 lines and was
//! split into submodules per the file-size limits in CLAUDE.md:
//! - [`execution`] — qmt_live/qmt_preview execution-bridge write-path tests,
//! - [`preflight`] — preflight-report + promotion-checklist classification
//!   tests, plus the `qmt_live_broker_capabilities` and `bridge_capabilities`
//!   helpers.

pub(super) use super::*;

mod execution;
mod preflight;
