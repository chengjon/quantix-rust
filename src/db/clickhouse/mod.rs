//! ClickHouse client module.
//!
//! This file is the module root: it only declares submodules and re-exports
//! their public symbols. The shared client lives in [`client`], and the
//! chrono/time bridge helpers live in [`chrono_helpers`].
//!
//! The `pub(crate) use` block below re-exports crate-level primitives that
//! sibling files (`kline.rs`, `fundamentals.rs`, etc.) pull in via
//! `use super::*;`. Keeping these re-exports here lets the table-specific
//! files stay focused on their `impl ClickHouseClient` blocks without each
//! one restating the same imports.

mod chrono_helpers;
mod client;
mod fundamentals;
mod gbbq;
mod kline;
mod minute;
mod models;
mod schema;
mod shadow_kline;

#[cfg(test)]
mod tests;

pub use self::client::ClickHouseClient;
pub use self::minute::{
    StreamStats, stream_minute_klines_to_clickhouse, stream_minute_shares_to_clickhouse,
};
// Re-exported pub(crate) so P0.15a handlers (Task 3/4) can construct the sinks.
// Without this, the `mod minute` privacy barrier blocks external naming even though
// the structs themselves are `pub(crate)`. (Task 4 will consume ClickHouseMinuteShareSink.)
#[allow(unused_imports)]
pub(crate) use self::minute::{ClickHouseMinuteKlineSink, ClickHouseMinuteShareSink};
pub use self::models::{
    GbbqEventCH, KlineDataCH, LimitUpEventCH, MarketFundamentalSnapshotCH, MarketSentimentDailyCH,
    MinuteKlineCH, MinuteShareCH, NorthFlowDailyCH, SectorDailyCH, StockInfoCH, StockQuoteCH,
};

// Chrono ↔ time bridge: keep old `crate::db::clickhouse::naive_to_offsetdatetime`
// path stable for sibling files (kline.rs, minute.rs, tests.rs).
pub(crate) use self::chrono_helpers::{
    datetime_utc_to_offsetdatetime, naive_to_offsetdatetime, offsetdatetime_to_naivedate,
};

// Primitives re-exported for sibling files (`use super::*`).
pub(crate) use crate::core::{QuantixError, Result};
pub(crate) use ::serde::Deserialize;
pub(crate) use rust_decimal::Decimal;
pub(crate) use rust_decimal::prelude::*;
pub(crate) use tracing::debug;
