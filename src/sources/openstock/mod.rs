//! OpenStock data-source parsers.
//!
//! Two lanes share a parse-primitives + error-enum core:
//! see [`kline`] (daily-kline JSON fixtures) and [`live_shadow`] (read-only
//! validation of captured live `/data/bars` payloads).

mod kline;
mod live_shadow;
mod parse;

pub use kline::{OpenStockKlineParseError, parse_daily_kline_json};
pub use live_shadow::{
    LiveShadowDrift, LiveShadowReport, LiveShadowRequest, LiveShadowStatus,
    live_shadow_error_into_quantix, validate_live_shadow_payload,
};

// Shared helpers the sibling openstock_* submodules reach via the
// `crate::sources::openstock::*` glob. Kept pub(crate); re-exported without
// naming each item to avoid E0364 on pub(crate)-visibility items.
pub(crate) use parse::*;
