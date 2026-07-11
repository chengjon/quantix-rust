use std::fs;

use super::openstock_shadow_handler::read_payload;
use super::shared_support::decimal_to_f64;
use crate::core::runtime::OpenStockSettings;
use crate::core::{QuantixError, Result};
use crate::sources::openstock::{
    LiveShadowRequest, live_shadow_error_into_quantix, validate_live_shadow_payload,
};
use crate::sources::openstock_calendar::{
    TradeDateRecord, WorkdayRecord, calendar_error_into_quantix, parse_trade_dates, parse_workdays,
};
use crate::sources::openstock_client::OpenStockClient;
use crate::sources::openstock_codes::{
    StockCodeRecord, StockListRecord, parse_all_stocks, parse_stock_codes,
    stock_code_error_into_quantix,
};
use crate::sources::openstock_envelope::OpenStockEnvelope;
use crate::sources::openstock_index::{
    IndexKlineRecord, index_kline_error_into_quantix, parse_index_klines,
};
use crate::sources::parse_daily_kline_json;

mod fetch;
mod import;
mod validate;

#[allow(unused_imports)]
pub use fetch::*;
#[allow(unused_imports)]
pub use import::*;
#[allow(unused_imports)]
pub use validate::*;

/// P0.15a double-key gate env-var name.
///
/// Writes to ClickHouse `minute_klines` / `minute_shares` occur iff
/// `--apply == true` AND this env var is `"yes"` (verbatim).
/// Mirrors `QUANTIX_OPENSTOCK_KLINE_APPLY` semantics (openstock_handler.rs:1055).
pub(crate) const MINUTE_APPLY_ENV: &str = "QUANTIX_OPENSTOCK_MINUTE_APPLY";

/// Compute whether to actually write to ClickHouse.
///
/// Returns `true` iff `apply` (from `--apply` CLI flag) AND the env var
/// `QUANTIX_OPENSTOCK_MINUTE_APPLY` is `"yes"` (verbatim). Anything else
/// returns `false` (dry-run).
///
/// Reading the env internally (rather than passing `env: Option<&str>`)
/// forces tests U2/U3 to set the real env-var name, exercising the contract.
pub(crate) fn compute_apply(apply: bool) -> bool {
    apply && std::env::var(MINUTE_APPLY_ENV).ok().as_deref() == Some("yes")
}
