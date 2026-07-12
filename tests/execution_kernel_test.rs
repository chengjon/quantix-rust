//! Execution kernel integration tests, split by concern.
//!
//! Historical note: this file was once 1417 lines and was split into
//! thematic submodules per the file-size limits in CLAUDE.md. Shared
//! infrastructure (use statements, fixture helpers, mock adapters,
//! test stores) lives in [`helpers`] and is re-exported to submodules
//! via `use super::helpers::*;`.

#[path = "execution_kernel_test/helpers.rs"]
mod helpers;

#[path = "execution_kernel_test/kernel.rs"]
mod kernel;

#[path = "execution_kernel_test/fill_delta.rs"]
mod fill_delta;

#[path = "execution_kernel_test/paper_adapter.rs"]
mod paper_adapter;

#[path = "execution_kernel_test/request_prepared.rs"]
mod request_prepared;

#[path = "execution_kernel_test/signals.rs"]
mod signals;
