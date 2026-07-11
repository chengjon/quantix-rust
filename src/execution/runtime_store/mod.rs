//! Strategy runtime SQLite store module.
//!
//! This file is the module root: it only declares submodules and re-exports
//! the public store type plus the shared primitives that sibling impl files
//! (`orders.rs`, `requests.rs`, `signals.rs`, `schema.rs`) pull in via
//! `use super::*`. The struct and its base impl block live in [`store`];
//! table-specific impl blocks live in [`orders`], [`requests`], [`signals`],
//! and [`schema`]. Row codec helpers live in [`codec`] and the execution-
//! snapshot JSON builder lives in [`snapshot`].

#![allow(clippy::collapsible_if)]

mod codec;
mod orders;
mod requests;
mod schema;
mod signals;
mod snapshot;
mod store;

pub use self::store::StrategyRuntimeStore;

// Primitives re-exported for sibling files (`use super::*`).
pub(crate) use chrono::{DateTime, Utc};
pub(crate) use rust_decimal::Decimal;
pub(crate) use sqlx::Row;
pub(crate) use sqlx::sqlite::SqliteRow;
pub(crate) use uuid::Uuid;

pub(crate) use crate::core::{QuantixError, Result};
pub(crate) use crate::execution::models::{
    ApprovalStatus, ExecutionRequestRecord, ExecutionRequestStatus, MockLiveOrderState,
    OrderEventRecord, OrderRecord, OrderStatus, RunnerCheckpointRecord, SignalStatus,
    StrategyDaemonCheckpointRecord, StrategyRunRecord, StrategyRunStatus, StrategySignalRecord,
};

pub(crate) use self::codec::parse_timestamp;
pub(crate) use self::snapshot::build_execution_snapshot;
