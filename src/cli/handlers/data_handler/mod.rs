//! Data-source configuration, K-line query/import/export commands.
//!
//! Each child module owns one coherent group of handlers plus any private
//! helpers it needs; the sibling-only test module lives in [`tests`]. The
//! parent (`handlers/mod.rs`) reaches the handlers through the
//! `pub(crate) use` globs below, so callers keep working unchanged.

mod export;
mod query;
mod sources;
#[cfg(test)]
mod tests;

pub(crate) use export::*;
pub(crate) use query::*;
pub(crate) use sources::*;
