#![allow(clippy::too_many_arguments)]

//! Order table operations: CRUD, optimistic-lock updates, listing, mock-live
//! state, and the runner-checkpoint helpers.
//!
//! Each child module holds its own `impl StrategyRuntimeStore` block; callers
//! reach the methods on the store struct. The parent (`runtime_store/mod.rs`)
//! re-exports the primitives for the sibling impl files; we re-export them here
//! as well so the children can keep using `use super::*` unchanged.

mod lifecycle;
mod mock;
mod optimistic;

pub(crate) use super::store::StrategyRuntimeStore;
pub(crate) use crate::core::Result;
pub(crate) use crate::execution::models::{
    MockLiveOrderState, OrderEventRecord, OrderRecord, OrderStatus, RunnerCheckpointRecord,
};
pub(crate) use chrono::{DateTime, Utc};
pub(crate) use rust_decimal::Decimal;
pub(crate) use sqlx::Row;
