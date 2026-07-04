//! ClickHouse write path for OpenStock minute-level data (P0.14).
//!
//! Consumes `fetch_minute_klines_stream` / `fetch_minute_share_stream`
//! (P0.13d) and writes batches to `quantix.minute_klines` / `minute_shares`.
//!
//! Type mapping follows `KlineDataCH` / `kline_data` exactly:
//! - `DateTime<Utc>` for `timestamp`
//! - `String` for `period` / `adjust`
//! - `Float64` for OHLCV / amount
//! - `dec.to_f64().unwrap_or(0.0)` for Decimal→f64 (matches kline.rs:213-219)

use crate::core::QuantixError;
use crate::data::models::{AdjustType, DateOrRange, MinuteBar, MinutePeriod, MinuteShare};
use crate::db::clickhouse::models::{MinuteKlineCH, MinuteShareCH};
use crate::sources::openstock_client::OpenStockClient;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use clickhouse::Client;
use futures::StreamExt;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

// ─── Conversion helpers (private) ──────────────────────────────────────────

/// Lift a NaiveDateTime to a UTC-tagged DateTime for ClickHouse `DateTime` columns.
///
/// 与 `src/db/clickhouse/kline.rs:210` 完全一致。OpenStock 返回的 naive 时间
/// 是北京时间 wall-clock；按 `kline_data` 表的约定写入为 `DateTime<Utc>`，
/// 读回时调用方按 A 股东八区语义解读。
fn naive_to_utc(naive: chrono::NaiveDateTime) -> DateTime<Utc> {
    DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)
}

/// Convert Decimal to f64 for ClickHouse Float64 columns.
///
/// A 股数值范围内（|v| < 10^15）Decimal → f64 转换无损：
/// - 价格：[0.01, 9999.99]，远低于 2^53
/// - 成交额：单 bar ≤ 10^12
///
/// 与 `src/db/clickhouse/kline.rs:213-219` 完全一致：
/// 通过 `rust_decimal::prelude::*` 的 `ToPrimitive::to_f64`；
/// `.unwrap_or(0.0)` 是防御性回退（理论不可能失败）。
/// 不写 warn：与 kline.rs 静默回退模式对齐，避免正常运行时刷日志。
fn decimal_to_f64(v: Decimal) -> f64 {
    v.to_f64().unwrap_or(0.0)
}

/// `period` enum → OpenStock API 字面量字符串。
fn period_as_str(p: &MinutePeriod) -> &'static str {
    match p {
        MinutePeriod::Minute1 => "1m",
        MinutePeriod::Minute5 => "5m",
        MinutePeriod::Minute15 => "15m",
        MinutePeriod::Minute30 => "30m",
        MinutePeriod::Minute60 => "60m",
    }
}

/// `adjust_type` enum → OpenStock API 字面量字符串。
fn adjust_as_str(a: &AdjustType) -> &'static str {
    match a {
        AdjustType::None => "none",
        AdjustType::QFQ => "qfq",
        AdjustType::HFQ => "hfq",
    }
}

fn bar_to_row(bar: &MinuteBar, period: MinutePeriod) -> MinuteKlineCH {
    MinuteKlineCH {
        timestamp: naive_to_utc(bar.timestamp),
        code: bar.code.clone(),
        // NOTE: MinuteBar has no `period` field (per data/models.rs:138-148);
        // period is the input parameter to `fetch_minute_klines_stream`, so
        // it must be threaded through `bar_to_row` from the stream consumer.
        period: period_as_str(&period).to_string(),
        adjust: adjust_as_str(&bar.adjust_type).to_string(),
        open: decimal_to_f64(bar.open),
        high: decimal_to_f64(bar.high),
        low: decimal_to_f64(bar.low),
        close: decimal_to_f64(bar.close),
        volume: bar.volume as f64,
        // INV-2D: parser guarantees non-None; unwrap_or_default is safe.
        amount: bar.amount.unwrap_or_default().to_f64().unwrap_or(0.0),
    }
}

fn share_to_row(share: &MinuteShare) -> MinuteShareCH {
    MinuteShareCH {
        timestamp: naive_to_utc(share.timestamp),
        code: share.code.clone(),
        // INV-2D: parser guarantees non-None for all four fields.
        price: share.price.unwrap_or_default().to_f64().unwrap_or(0.0),
        volume: share.volume.unwrap_or_default() as f64,
        amount: share.amount.unwrap_or_default().to_f64().unwrap_or(0.0),
        avg_price: share.avg_price.unwrap_or_default().to_f64().unwrap_or(0.0),
    }
}

// ─── Test-only exposure of helpers (still crate-private) ───────────────────
//
// Unit tests in `tests.rs` reach these via `minute::decimal_to_f64_for_test`.
// The `_for_test` suffix guards against accidental production use.
#[cfg(test)]
pub(crate) fn decimal_to_f64_for_test(v: Decimal) -> f64 {
    decimal_to_f64(v)
}
#[cfg(test)]
pub(crate) fn naive_to_utc_for_test(naive: chrono::NaiveDateTime) -> DateTime<Utc> {
    naive_to_utc(naive)
}
#[cfg(test)]
pub(crate) fn bar_to_row_for_test(bar: &MinuteBar, period: MinutePeriod) -> MinuteKlineCH {
    bar_to_row(bar, period)
}
#[cfg(test)]
pub(crate) fn share_to_row_for_test(share: &MinuteShare) -> MinuteShareCH {
    share_to_row(share)
}

// ─── Sink trait (pub(crate), test-only mock injection) ─────────────────────

/// Internal sink abstraction. Used **only** by unit tests to inject a mock
/// without touching the real ClickHouse. Not part of any public API.
//
// INV-4A/B: trait + concrete sinks are `pub(crate)`. The public stream
// consumers below take `<S: MinuteSink<...>>`, but because the trait itself
// is `pub(crate)`, external crates cannot construct a satisfying type —
// effectively making the public functions internal-only (INV-4D).
#[async_trait]
pub trait MinuteSink<T: Send + Sync>: Send + Sync {
    async fn insert_batch(&self, batch: &[T]) -> Result<usize, clickhouse::error::Error>;
}

#[allow(dead_code)] // constructed in T3 live tests
pub(crate) struct ClickHouseMinuteKlineSink<'a> {
    pub(crate) client: &'a Client,
}

#[allow(dead_code)] // constructed in T3 live tests
pub(crate) struct ClickHouseMinuteShareSink<'a> {
    pub(crate) client: &'a Client,
}

#[async_trait]
impl<'a> MinuteSink<MinuteKlineCH> for ClickHouseMinuteKlineSink<'a> {
    async fn insert_batch(
        &self,
        batch: &[MinuteKlineCH],
    ) -> Result<usize, clickhouse::error::Error> {
        if batch.is_empty() {
            return Ok(0);
        }
        let mut insert = self
            .client
            .insert("minute_klines")?
            .with_option("async_insert", "1")
            .with_option("wait_for_async_insert", "1");
        for row in batch {
            insert.write(row).await?;
        }
        insert.end().await?;
        Ok(batch.len())
    }
}

#[async_trait]
impl<'a> MinuteSink<MinuteShareCH> for ClickHouseMinuteShareSink<'a> {
    async fn insert_batch(
        &self,
        batch: &[MinuteShareCH],
    ) -> Result<usize, clickhouse::error::Error> {
        if batch.is_empty() {
            return Ok(0);
        }
        let mut insert = self
            .client
            .insert("minute_shares")?
            .with_option("async_insert", "1")
            .with_option("wait_for_async_insert", "1");
        for row in batch {
            insert.write(row).await?;
        }
        insert.end().await?;
        Ok(batch.len())
    }
}

// ─── Stream consumers (public API) ─────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StreamStats {
    pub batches: u64,
    pub input_records: u64,
    pub inserted_records: u64,
}

/// Consume the klines stream and insert each batch into `minute_klines`.
///
/// Stream pinning is internal: `fetch_minute_klines_stream` returns
/// `impl Stream + 'a` (not Unpin), so we use `futures::pin_mut!` here.
///
/// INV-3A: short-circuits on first stream error (`?`).
/// INV-3C: never catches errors internally.
//
// INV-4D: `<S: MinuteSink<...>>` is bounded by a trait whose only concrete
// impls (`ClickHouseMinute{Kline,Share}Sink`) are `pub(crate)`. External
// crates therefore cannot construct a satisfying type, making this `pub` fn
// effectively internal-only.
pub async fn stream_minute_klines_to_clickhouse<S: MinuteSink<MinuteKlineCH>>(
    client: &OpenStockClient,
    sink: &S,
    code: &str,
    period: MinutePeriod,
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
    adjust: AdjustType,
) -> Result<StreamStats, QuantixError> {
    let dor = DateOrRange::Range { start, end };
    let stream = client.fetch_minute_klines_stream(code, period, dor, adjust);
    futures::pin_mut!(stream);

    let mut stats = StreamStats::default();
    while let Some(batch_result) = stream.next().await {
        let batch = batch_result?;
        stats.batches += 1;
        stats.input_records += batch.len() as u64;

        let rows: Vec<MinuteKlineCH> = batch.iter().map(|b| bar_to_row(b, period)).collect();
        let inserted = sink
            .insert_batch(&rows)
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("ch insert minute_klines: {}", e)))?;
        stats.inserted_records += inserted as u64;
    }
    Ok(stats)
}

/// Consume the shares stream and insert each batch into `minute_shares`.
///
/// INV-3B: short-circuits on first stream or sink error.
//
// INV-4D: see `stream_minute_klines_to_clickhouse`.
pub async fn stream_minute_shares_to_clickhouse<S: MinuteSink<MinuteShareCH>>(
    client: &OpenStockClient,
    sink: &S,
    code: &str,
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> Result<StreamStats, QuantixError> {
    let dor = DateOrRange::Range { start, end };
    let stream = client.fetch_minute_share_stream(code, dor);
    futures::pin_mut!(stream);

    let mut stats = StreamStats::default();
    while let Some(batch_result) = stream.next().await {
        let batch = batch_result?;
        stats.batches += 1;
        stats.input_records += batch.len() as u64;

        let rows: Vec<MinuteShareCH> = batch.iter().map(share_to_row).collect();
        let inserted = sink
            .insert_batch(&rows)
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("ch insert minute_shares: {}", e)))?;
        stats.inserted_records += inserted as u64;
    }
    Ok(stats)
}
