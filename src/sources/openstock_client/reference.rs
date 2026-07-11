//! Reference-data family fetch methods (codes, all_stocks, trade_dates, workdays)
//! for [`crate::sources::openstock_client::OpenStockClient`].

use serde_json::Value;

use super::OpenStockResponse;
use crate::core::Result;

impl super::OpenStockClient {
    /// Convenience: fetch `STOCK_CODES`.
    pub async fn fetch_stock_codes(
        &self,
    ) -> Result<OpenStockResponse<crate::sources::openstock_codes::StockCodeRecord>> {
        self.fetch("STOCK_CODES", serde_json::json!({})).await
    }

    /// Convenience: fetch `TRADE_DATES` for an optional date range.
    ///
    /// Runtime contract (`baostock._fetch_trade_dates`): accepts
    /// `start_date` / `end_date` as `YYYY-MM-DD` strings. When both are
    /// `None`, baostock returns the full history (which the runtime
    /// truncates). The legacy `year` parameter is **ignored** by the
    /// runtime — callers should pass `start`/`end` instead.
    pub async fn fetch_trade_dates(
        &self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<OpenStockResponse<crate::sources::openstock_calendar::TradeDateRecord>> {
        let mut params = serde_json::json!({});
        if let Some(start) = start {
            params["start_date"] = Value::String(start.to_string());
        }
        if let Some(end) = end {
            params["end_date"] = Value::String(end.to_string());
        }
        self.fetch("TRADE_DATES", params).await
    }

    /// Convenience: fetch `ALL_STOCKS` (baostock full-market snapshot).
    /// `day` is optional (`YYYY-MM-DD`); when omitted, the server falls
    /// back to the most recent trading day and reports it via
    /// `quality_flags.fallback_day`.
    pub async fn fetch_all_stocks(
        &self,
        day: Option<&str>,
    ) -> Result<OpenStockResponse<crate::sources::openstock_codes::StockListRecord>> {
        let mut params = serde_json::json!({});
        if let Some(day) = day {
            params["day"] = Value::String(day.to_string());
        }
        self.fetch("ALL_STOCKS", params).await
    }

    /// Convenience: fetch `WORKDAYS` (eltdx action-driven calendar).
    /// `action` is one of `today` / `today_is_workday` / `is_workday` /
    /// `range` / `next_workday` / `previous_workday`. `date` is required
    /// for `is_workday`/`next_workday`/`previous_workday`; `start`+`end`
    /// are required for `range`. Other actions ignore the date params.
    pub async fn fetch_workdays(
        &self,
        action: &str,
        date: Option<&str>,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<OpenStockResponse<crate::sources::openstock_calendar::WorkdayRecord>> {
        let mut params = serde_json::json!({ "action": action });
        if let Some(date) = date {
            params["date"] = Value::String(date.to_string());
        }
        if let Some(start) = start {
            params["start"] = Value::String(start.to_string());
        }
        if let Some(end) = end {
            params["end"] = Value::String(end.to_string());
        }
        self.fetch("WORKDAYS", params).await
    }
}
