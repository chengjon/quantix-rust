//! Contract doc field sync tests, split by concern.
//!
//! Historical note: this file was once 1021 lines and was split into
//! thematic submodules per the file-size limits in CLAUDE.md. Shared
//! infrastructure (DOC/SCHEMA constants + parse/assert helpers) lives
//! in [`helpers`] and is re-exported to submodules via
//! `use super::helpers::*;`.

#[path = "contract_doc_field_sync_test/helpers.rs"]
mod helpers;

#[path = "contract_doc_field_sync_test/field_sync.rs"]
mod field_sync;

#[path = "contract_doc_field_sync_test/type_classes.rs"]
mod type_classes;
