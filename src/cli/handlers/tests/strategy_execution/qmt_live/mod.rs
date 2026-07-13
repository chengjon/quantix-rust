//! QMT live execution + audit report tests, split by concern.
//!
//! Historical note: the original `qmt_live.rs` was 768 lines and was split
//! into submodules per the file-size limits in CLAUDE.md:
//! - [`execution`] — safety-gate + success-path tests for the
//!   `execute_execution_bridge_qmt_live` flow,
//! - [`audit`] — audit output + manual-intervention report tests, plus the
//!   `QmtManualCaseSpec` helper and `insert_qmt_manual_intervention_case`.

// Re-export the parent test harness namespace so submodules can write
// `use super::*;` and pick up everything `strategy_execution.rs` already imports.
pub(super) use super::*;

mod audit;
mod execution;
