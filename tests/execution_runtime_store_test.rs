//! Execution runtime store integration tests, split by concern.
//!
//! Historical note: this file was once 1017 lines and was split into
//! thematic submodules per the file-size limits in CLAUDE.md. Shared
//! infrastructure (use statements, fixture helpers) lives in [`helpers`]
//! and is re-exported to submodules via `use super::helpers::*;`.

#[path = "execution_runtime_store_test/helpers.rs"]
mod helpers;

#[path = "execution_runtime_store_test/daemon.rs"]
mod daemon;

#[path = "execution_runtime_store_test/orders.rs"]
mod orders;

#[path = "execution_runtime_store_test/qmt_live.rs"]
mod qmt_live;

#[path = "execution_runtime_store_test/recovery.rs"]
mod recovery;

#[path = "execution_runtime_store_test/schema.rs"]
mod schema;

#[path = "execution_runtime_store_test/signals.rs"]
mod signals;

#[path = "execution_runtime_store_test/versioning.rs"]
mod versioning;

#[path = "execution_runtime_store_test/workflow.rs"]
mod workflow;
