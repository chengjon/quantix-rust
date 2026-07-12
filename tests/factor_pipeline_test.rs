//! Factor pipeline integration tests, split by concern.
//!
//! Historical note: this file was once 917 lines and was split into
//! thematic submodules per the file-size limits in CLAUDE.md. Shared
//! infrastructure (use statements + MockFactorLoader + mock frame
//! builders) lives in [`helpers`] and is re-exported to submodules
//! via `use super::helpers::*;`.

#[path = "factor_pipeline_test/helpers.rs"]
mod helpers;

#[path = "factor_pipeline_test/core_and_operators.rs"]
mod core_and_operators;

#[path = "factor_pipeline_test/catalog.rs"]
mod catalog;

#[path = "factor_pipeline_test/export_and_pipeline.rs"]
mod export_and_pipeline;

#[path = "factor_pipeline_test/evaluation.rs"]
mod evaluation;

#[path = "factor_pipeline_test/neutralization_and_backtest.rs"]
mod neutralization_and_backtest;
