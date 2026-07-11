//! K-line family fetch methods for [`crate::sources::openstock_client::OpenStockClient`].
//!
//! Five methods covering daily/weekly/monthly bars, historical K-lines,
//! index K-lines, and tick data. `fetch_daily_klines` and `fetch_klines`
//! use a direct `reqwest` call to `/data/bars` (no retry, no circuit
//! breaker) per the P0.10 design decision; the other three route through
//! the shared `self.fetch(...)` envelope.

use std::str::FromStr;

use serde_json::Value;

use super::OpenStockResponse;
use crate::core::{QuantixError, Result};

impl super::OpenStockClient {
    /// Convenience: fetch `INDEX_KLINES` for a symbol with optional date range.
    ///
    /// Runtime contract (`baostock._fetch_index_klines`): accepts
    /// `start_date` / `end_date` as `YYYY-MM-DD` strings — same as
    /// [`fetch_trade_dates`](Self::fetch_trade_dates). The legacy
    /// `start` / `end` parameter names are ignored by the runtime
    /// (verified 2026-07-01 against live `http://192.168.123.104:8040`).
    pub async fn fetch_index_klines(
        &self,
        code: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<OpenStockResponse<crate::sources::openstock_index::IndexKlineRecord>> {
        let mut params = serde_json::json!({ "code": code });
        if let Some(start) = start {
            params["start_date"] = Value::String(start.to_string());
        }
        if let Some(end) = end {
            params["end_date"] = Value::String(end.to_string());
        }
        self.fetch("INDEX_KLINES", params).await
    }

    /// Convenience: fetch `HISTORICAL_KLINES` for an A-share stock code
    /// with optional date range.
    ///
    /// Runtime contract: `HISTORICAL_KLINES` is documented in
    /// `DATA_CAPABILITY_SCOPE.md` §"Long-history A-share K-lines" but
    /// has not yet been live-verified from quantix-rust (P0.11a defers
    /// the live smoke to fixture-driven parser tests; field shape may
    /// differ from `INDEX_KLINES`). Parameters mirror
    /// [`fetch_index_klines`](Self::fetch_index_klines): `code` plus
    /// optional `start_date` / `end_date` as `YYYY-MM-DD` strings.
    /// Record type reuses `IndexKlineRecord` since its fields are all
    /// `Option<serde_json::Value>`, absorbing shape drift via the parser
    /// layer (per P0.11 design.md D4).
    pub async fn fetch_historical_klines(
        &self,
        code: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<OpenStockResponse<crate::sources::openstock_index::IndexKlineRecord>> {
        let mut params = serde_json::json!({ "code": code });
        if let Some(start) = start {
            params["start_date"] = Value::String(start.to_string());
        }
        if let Some(end) = end {
            params["end_date"] = Value::String(end.to_string());
        }
        self.fetch("HISTORICAL_KLINES", params).await
    }

    /// Convenience: fetch `TICK_DATA` for a stock symbol on an optional
    /// trading date. Live-verified 2026-07-01 (see design.md D4.1).
    ///
    /// **Critical**: the runtime parameter is `symbol` (NOT `code`).
    /// Sending `code` returns HTTP 422 `"symbol is required for
    /// TICK_DATA"`. This differs from `fetch_index_klines` /
    /// `fetch_historical_klines`, which use `code`. The eltdx adapter
    /// requires the `symbol` name.
    ///
    /// `date` is `YYYYMMDD` (numeric, no dashes) per the eltdx adapter
    /// contract. When omitted, the runtime defaults to the latest
    /// trading day.
    ///
    /// Response shape: `data: [{ meta, ticks: [...] }]` — a one-element
    /// array wrapping a `{meta, ticks}` envelope-record (NOT a flat
    /// tick list). Use [`parse_tick_data`](crate::sources::openstock_ticks::parse_tick_data)
    /// to flatten.
    pub async fn fetch_tick_data(
        &self,
        symbol: &str,
        date: Option<&str>,
    ) -> Result<OpenStockResponse<crate::sources::openstock_ticks::TickEnvelopeRecord>> {
        let mut params = serde_json::json!({ "symbol": symbol });
        if let Some(date) = date {
            params["date"] = Value::String(date.to_string());
        }
        self.fetch("TICK_DATA", params).await
    }

    /// Fetch daily OHLCV bars from OpenStock `/data/bars` endpoint.
    ///
    /// This is the preferred path for backtest/analysis K-line data.
    /// `/data/bars` natively supports symbol, period, date range and
    /// adjust type, and internally negotiates KLINES / ADJUSTED_KLINES
    /// / HISTORICAL_KLINES based on the `adjust` param.
    ///
    /// Returns `Vec<Kline>` on success. When the upstream provider is
    /// unavailable or returns empty data, returns an empty vec (caller
    /// should fall back to ClickHouse or another source).
    pub async fn fetch_daily_klines(
        &self,
        code: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Vec<crate::data::models::Kline>> {
        let endpoint = self
            .base_url
            .join("data/bars")
            .map_err(|e| QuantixError::Other(format!("url join failed: {}", e)))?;

        let mut body = serde_json::json!({
            "symbol": code,
            "period": "day",
        });
        if let Some(start) = start {
            body["start_date"] = Value::String(start.to_string());
        }
        if let Some(end) = end {
            body["end_date"] = Value::String(end.to_string());
        }

        let resp = self
            .http
            .post(endpoint)
            .header("X-API-Key", self.api_key.clone())
            .json(&body)
            .send()
            .await
            .map_err(|e| QuantixError::Network(format!("/data/bars request failed: {}", e)))?;

        let status = resp.status();
        let raw = resp
            .text()
            .await
            .map_err(|e| QuantixError::Network(format!("/data/bars body read failed: {}", e)))?;

        if !status.is_success() {
            return Err(QuantixError::Other(format!(
                "/data/bars returned {}: {}",
                status,
                raw.chars().take(200).collect::<String>()
            )));
        }

        #[derive(serde::Deserialize)]
        struct BarsResponse {
            data: Vec<BarRecord>,
        }

        #[derive(serde::Deserialize)]
        struct BarRecord {
            time: String,
            open: f64,
            high: f64,
            low: f64,
            close: f64,
            volume: f64,
            amount: f64,
        }

        let bars: BarsResponse = serde_json::from_str(&raw)
            .map_err(|e| QuantixError::Other(format!("/data/bars parse failed: {}", e)))?;

        let mut klines = Vec::with_capacity(bars.data.len());
        for bar in bars.data {
            // time format: "2026-06-01T15:00:00+08:00" → NaiveDate
            let date = chrono::NaiveDate::parse_from_str(&bar.time[..10], "%Y-%m-%d")
                .map_err(|e| QuantixError::DataParse(format!("解析 bars 日期失败: {}", e)))?;

            klines.push(crate::data::models::Kline {
                code: code.to_string(),
                date,
                open: rust_decimal::Decimal::from_str(&format!("{}", bar.open)).unwrap_or_default(),
                high: rust_decimal::Decimal::from_str(&format!("{}", bar.high)).unwrap_or_default(),
                low: rust_decimal::Decimal::from_str(&format!("{}", bar.low)).unwrap_or_default(),
                close: rust_decimal::Decimal::from_str(&format!("{}", bar.close))
                    .unwrap_or_default(),
                volume: bar.volume as i64,
                amount: Some(
                    rust_decimal::Decimal::from_str(&format!("{}", bar.amount)).unwrap_or_default(),
                ),
                adjust_type: crate::data::models::AdjustType::None,
            });
        }

        Ok(klines)
    }

    /// Fetch K-line bars from OpenStock `/data/bars` endpoint with explicit
    /// period (day/week/month) and adjust (none/qfq/hfq) parameters.
    ///
    /// This is the multi-period generalisation of [`Self::fetch_daily_klines`].
    /// Like that method, it uses a direct `reqwest` call (no retry, no circuit
    /// breaker) per the P0.10 design decision for `/data/bars`. The returned
    /// `Kline` records are stamped with the requested `adjust_type` (the
    /// runtime does not echo it back — decision D2 request-driven).
    ///
    /// `AdjustType::None` causes the `adjust` field to be omitted entirely
    /// from the request body (matches `fetch_daily_klines` wire shape).
    pub async fn fetch_klines(
        &self,
        code: &str,
        period: crate::data::models::BarPeriod,
        adjust: crate::data::models::AdjustType,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Vec<crate::data::models::Kline>> {
        use std::str::FromStr;

        let endpoint = self
            .base_url
            .join("data/bars")
            .map_err(|e| QuantixError::Other(format!("url join failed: {}", e)))?;

        let mut body = serde_json::json!({
            "symbol": code,
            "period": period.as_str(),
        });
        if let Some(adj) = adjust.as_openstock_param() {
            body["adjust"] = Value::String(adj.to_string());
        }
        if let Some(start) = start {
            body["start_date"] = Value::String(start.to_string());
        }
        if let Some(end) = end {
            body["end_date"] = Value::String(end.to_string());
        }

        let resp = self
            .http
            .post(endpoint)
            .header("X-API-Key", self.api_key.clone())
            .json(&body)
            .send()
            .await
            .map_err(|e| QuantixError::Network(format!("/data/bars request failed: {}", e)))?;

        let status = resp.status();
        let raw = resp
            .text()
            .await
            .map_err(|e| QuantixError::Network(format!("/data/bars body read failed: {}", e)))?;

        if !status.is_success() {
            return Err(QuantixError::Other(format!(
                "/data/bars returned {}: {}",
                status,
                raw.chars().take(200).collect::<String>()
            )));
        }

        #[derive(serde::Deserialize)]
        struct BarsResponse {
            data: Vec<BarRecord>,
        }

        #[derive(serde::Deserialize)]
        struct BarRecord {
            time: String,
            open: f64,
            high: f64,
            low: f64,
            close: f64,
            volume: f64,
            amount: f64,
        }

        let bars: BarsResponse = serde_json::from_str(&raw)
            .map_err(|e| QuantixError::Other(format!("/data/bars parse failed: {}", e)))?;

        let mut klines = Vec::with_capacity(bars.data.len());
        for bar in bars.data {
            // time format: "2026-06-01T15:00:00+08:00" → NaiveDate
            let date = chrono::NaiveDate::parse_from_str(&bar.time[..10], "%Y-%m-%d")
                .map_err(|e| QuantixError::DataParse(format!("解析 bars 日期失败: {}", e)))?;

            klines.push(crate::data::models::Kline {
                code: code.to_string(),
                date,
                open: rust_decimal::Decimal::from_str(&format!("{}", bar.open)).unwrap_or_default(),
                high: rust_decimal::Decimal::from_str(&format!("{}", bar.high)).unwrap_or_default(),
                low: rust_decimal::Decimal::from_str(&format!("{}", bar.low)).unwrap_or_default(),
                close: rust_decimal::Decimal::from_str(&format!("{}", bar.close))
                    .unwrap_or_default(),
                volume: bar.volume as i64,
                amount: Some(
                    rust_decimal::Decimal::from_str(&format!("{}", bar.amount)).unwrap_or_default(),
                ),
                adjust_type: adjust,
            });
        }

        Ok(klines)
    }
}
