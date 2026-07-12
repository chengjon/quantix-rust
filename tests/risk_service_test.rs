//! Risk service integration tests, split by concern.
//!
//! Historical note: this file was once 882 lines and was split into
//! thematic submodules per the file-size limits in CLAUDE.md. Shared
//! infrastructure (FakeRiskStore, FakeIndustryResolver, service/snapshot
//! helpers) lives in [`helpers`] and is re-exported to submodules via
//! `use super::helpers::*;`.

#[path = "risk_service_test/helpers.rs"]
mod helpers;

#[path = "risk_service_test/set_rule_and_status.rs"]
mod set_rule_and_status;

#[path = "risk_service_test/locking_and_position.rs"]
mod locking_and_position;

#[path = "risk_service_test/release_and_log.rs"]
mod release_and_log;
