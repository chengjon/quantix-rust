//! Minute-data family fetch methods (klines-by-period, minute share)
//! for [`crate::sources::openstock_client::OpenStockClient`].

use futures::stream;

use crate::core::{QuantixError, Result};

impl super::OpenStockClient {
    /// Fetches minute-level OHLCV candles from `/data/bars` with `period`
    /// in `1m|5m|15m|30m|60m`. Mirrors `fetch_klines` shape (direct reqwest,
    /// no envelope, no retry, no circuit breaker). Returns `Vec<MinuteBar>`
    /// with `NaiveDateTime` timestamps (minute precision preserved from
    /// the wire ISO string).
    ///
    /// The returned `MinuteBar` records are stamped with the requested
    /// `adjust_type` (the runtime does not echo it back — decision D2
    /// request-driven, matching `fetch_klines`).
    ///
    /// `AdjustType::None` causes the `adjust` field to be omitted entirely
    /// from the request body (matches `fetch_klines` wire shape).
    ///
    /// `date_or_range` selects single-day (`Date` — wire body identical to
    /// P0.13b-1, INV-2A) or inclusive multi-day range (`Range` — wire body
    /// carries `start_date`/`end_date`, no `date` field, spec §3.2/§6 D4).
    pub async fn fetch_minute_klines(
        &self,
        code: &str,
        period: crate::data::models::MinutePeriod,
        date_or_range: crate::data::models::DateOrRange,
        adjust: crate::data::models::AdjustType,
    ) -> Result<Vec<crate::data::models::MinuteBar>> {
        use crate::data::models::DateOrRange;
        let (start, end) = match date_or_range {
            DateOrRange::Date(d) => (d, d),
            DateOrRange::Range { start, end } => (start, end),
        };
        self.fetch_minute_klines_range(code, period, start, end, adjust)
            .await
    }

    /// 流式拉取分钟 K 线（P0.13d D1/D2）。
    ///
    /// 把 `date_or_range` 解析为 `(start, end)`，调用 `chunk_range_weekly`
    /// 切成连续 ≤7 天段，每段一次 `fetch_minute_klines_range` 调用，yield
    /// 一个 `Vec<MinuteBar>`。
    ///
    /// - `Date(d)`：单段 `(d, d)`，一个 batch
    /// - `Range { start, end }`：从 start 起每 7 天一段；尾段可能短
    /// - 错误：首个 batch 失败即 yield `Err`，后续 `next()` 返回 `None`（D4）
    /// - 不经过 retry/circuit breaker（D5；与 batch klines 路径一致）
    /// - Wire shape 由 `fetch_minute_klines_range` 保证（INV-2A）
    pub fn fetch_minute_klines_stream<'a>(
        &'a self,
        code: &'a str,
        period: crate::data::models::MinutePeriod,
        date_or_range: crate::data::models::DateOrRange,
        adjust: crate::data::models::AdjustType,
    ) -> impl futures::Stream<Item = Result<Vec<crate::data::models::MinuteBar>>> + 'a {
        use crate::data::models::DateOrRange;
        let (start, end) = match date_or_range {
            DateOrRange::Date(d) => (d, d),
            DateOrRange::Range { start, end } => (start, end),
        };
        let chunks = crate::data::models::chunk_range_weekly(start, end);
        // INV-5A: first Err yields, then stream terminates (next() returns None).
        // `unfold` ensures the next chunk is fetched only when the consumer asks
        // for the next item AND no prior chunk has errored. State carries the
        // iterator + errored flag so the closure body is FnMut-compatible.
        let state = (chunks.into_iter(), false);
        stream::unfold(state, move |(mut iter, errored)| async move {
            if errored {
                return None;
            }
            let (s, e) = iter.next()?;
            let res = self
                .fetch_minute_klines_range(code, period, s, e, adjust)
                .await;
            let next_errored = res.is_err();
            Some((res, (iter, next_errored)))
        })
    }

    /// Private helper: fetch minute klines for an inclusive `[start..=end]` sub-range.
    ///
    /// Wire body field selection (INV-2A, preserving P0.13b-1 / P0.13c):
    ///   - `start == end` → body has only `date` (identical to P0.13b-1 single-day wire)
    ///   - `start != end` → body has only `start_date` + `end_date`
    ///
    /// This helper is shared by:
    ///   - `fetch_minute_klines` (batch API; dispatcher for `DateOrRange`)
    ///   - `fetch_minute_klines_stream` (streaming API; one call per weekly chunk)
    async fn fetch_minute_klines_range(
        &self,
        code: &str,
        period: crate::data::models::MinutePeriod,
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
        adjust: crate::data::models::AdjustType,
    ) -> Result<Vec<crate::data::models::MinuteBar>> {
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
            body["adjust"] = serde_json::Value::String(adj.to_string());
        }
        if start == end {
            body["date"] = serde_json::Value::String(start.format("%Y-%m-%d").to_string());
        } else {
            body["start_date"] = serde_json::Value::String(start.format("%Y-%m-%d").to_string());
            body["end_date"] = serde_json::Value::String(end.format("%Y-%m-%d").to_string());
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
            data: Vec<MinuteBarRecord>,
        }

        #[derive(serde::Deserialize)]
        struct MinuteBarRecord {
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

        let mut out = Vec::with_capacity(bars.data.len());
        for bar in bars.data {
            // Wire time format: "2026-07-02T09:31:00+08:00" → take first 19 chars
            // "2026-07-02T09:31:00" → parse as NaiveDateTime (no timezone).
            let ts = chrono::NaiveDateTime::parse_from_str(&bar.time[..19], "%Y-%m-%dT%H:%M:%S")
                .map_err(|e| {
                    QuantixError::DataParse(format!("解析 minute bars 时间戳失败: {}", e))
                })?;

            out.push(crate::data::models::MinuteBar {
                code: code.to_string(),
                timestamp: ts,
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

        Ok(out)
    }

    /// 消费 MINUTE_DATA category（分时点序列 / 分时图 ticks）。
    ///
    /// 走 `/data/fetch` envelope 路径，复用 retry + circuit breaker
    /// （与 `fetch_stock_codes` / `fetch_trade_dates` 同路径）。
    ///
    /// **调用签名**（对齐 `fetch_stock_codes`）：`fetch<T>()` 接收
    /// `(category: &str, params: Value)` 双参数，内部拼装为
    /// `{data_category, params}` envelope。
    ///
    /// **category 无 period/adjust 维度** — params 仅 `{code, date}`。
    ///
    /// 解析：response.records 是 8 字段的数组，parse_minute_share 裁剪到
    /// 5 业务字段。单条记录关键字段缺失 → warn + skip（INV-2C）。
    pub async fn fetch_minute_share(
        &self,
        code: &str,
        date_or_range: crate::data::models::DateOrRange,
    ) -> Result<Vec<crate::data::models::MinuteShare>> {
        use crate::data::models::{DateOrRange, iter_dates_inclusive};

        match date_or_range {
            DateOrRange::Date(d) => self.fetch_minute_share_single(code, d).await,
            DateOrRange::Range { start, end } => {
                let mut all = Vec::new();
                for d in iter_dates_inclusive(start, end) {
                    let day_records = self.fetch_minute_share_single(code, d).await?;
                    all.extend(day_records);
                }
                Ok(all)
            }
        }
    }

    /// 流式拉取分时点序列（P0.13d D1/D3）。
    ///
    /// 每个自然日（含非交易日）yield 一个 `Vec<MinuteShare>`；非交易日 yield
    /// 空 Vec（D3；batch count == 日历天数，调用方可做完整性检查）。
    ///
    /// - 复用 P0.13c `fetch_minute_share_single`（带 retry + breaker）
    /// - 错误：首个 batch 失败即 yield `Err`，后续 `next()` 返回 `None`（D4）
    pub fn fetch_minute_share_stream<'a>(
        &'a self,
        code: &'a str,
        date_or_range: crate::data::models::DateOrRange,
    ) -> impl futures::Stream<Item = Result<Vec<crate::data::models::MinuteShare>>> + 'a {
        use crate::data::models::{DateOrRange, iter_dates_inclusive};
        let (start, end) = match date_or_range {
            DateOrRange::Date(d) => (d, d),
            DateOrRange::Range { start, end } => (start, end),
        };
        let days: Vec<chrono::NaiveDate> = iter_dates_inclusive(start, end).collect();
        // INV-5A: first Err yields, then stream terminates (next() returns None).
        let state = (days.into_iter(), false);
        stream::unfold(state, move |(mut iter, errored)| async move {
            if errored {
                return None;
            }
            let d = iter.next()?;
            let res = self.fetch_minute_share_single(code, d).await;
            let next_errored = res.is_err();
            Some((res, (iter, next_errored)))
        })
    }

    /// Single-day helper for MINUTE_DATA fetches.
    ///
    /// Both `Date` and `Range` paths route here. The server wraps each result
    /// in a `{meta: {trading_date}, points: [...]}` envelope; we extract
    /// `trading_date` from the envelope (NOT from the request parameter) so
    /// that records are stamped with the actual trading day the server
    /// reported (INV-2C).
    async fn fetch_minute_share_single(
        &self,
        code: &str,
        date: chrono::NaiveDate,
    ) -> Result<Vec<crate::data::models::MinuteShare>> {
        let params = serde_json::json!({
            "code": code,
            "date": date.format("%Y-%m-%d").to_string(),
        });
        let resp = self
            .fetch::<MinuteShareEnvelope>("MINUTE_DATA", params)
            .await?;
        let mut out = Vec::new();
        for env in resp.records {
            let actual_date = env
                .meta
                .trading_date
                .as_deref()
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .unwrap_or(date);
            for raw in env.points {
                if let Some(share) = parse_minute_share(code, &raw, actual_date) {
                    out.push(share);
                } else {
                    tracing::warn!(
                        code = code,
                        requested_date = %date,
                        trading_date = %actual_date,
                        time_minutes = ?raw.time_minutes,
                        time = ?raw.time,
                        "MINUTE_DATA record missing required field or invalid time, skipping"
                    );
                }
            }
        }
        Ok(out)
    }
}

/// MINUTE_DATA 原始记录（8 字段，未裁剪）。
///
/// OpenStock envelope `records` 数组元素的反序列化目标。
/// 字段名对应 eLtdx MINUTE_DATA 输出：
///   - time_minutes: "0930" 或 "09:30" 格式
///   - price/volume/amount/avg_price: 业务字段（保留）
///   - index/time/price_milli: 冗余字段（serde default 容忍缺失）
///
/// **数值字段直接用 `Decimal`**（rust_decimal + serde 自动反序列化 JSON number）。
/// 若 live 测试发现字符串格式数值漂移，切换到 `serde_json::Value` + parse_decimal。
#[derive(Debug, serde::Deserialize)]
struct RawMinuteRecord {
    /// Primary time field. Live OpenStock may return `null` (BUG-B in
    /// OPENSTOCK_HANDOFF_2026-07-07.md) — fall back to `time` below.
    #[serde(default)]
    time_minutes: Option<String>,
    /// Secondary time field (e.g. `"09:31"`). Used when `time_minutes` is null.
    #[serde(default)]
    time: Option<String>,
    price: Option<rust_decimal::Decimal>,
    volume: Option<i64>,
    amount: Option<rust_decimal::Decimal>,
    avg_price: Option<rust_decimal::Decimal>,
}

/// OpenStock MINUTE_DATA envelope wrapper.
///
/// Each element of the `/data/fetch MINUTE_DATA` response `data` array is a
/// `{meta, points}` wrapper (not a flat record). `trading_date` lives in
/// `meta`, not inside each point — see spec §3.2 R1 evidence and
/// `_eltdx_timeseries.py:181-208`.
#[derive(Debug, serde::Deserialize)]
struct MinuteShareEnvelope {
    #[serde(default)]
    meta: MinuteShareMeta,
    #[serde(default)]
    points: Vec<RawMinuteRecord>,
}

/// Meta block of the MINUTE_DATA envelope.
#[derive(Debug, Default, serde::Deserialize)]
struct MinuteShareMeta {
    /// Server-reported trading day for the records in this envelope
    /// (`YYYY-MM-DD`). Per INV-2C, per-record timestamps are stamped from
    /// this value, not from the client-supplied request date.
    #[serde(default)]
    trading_date: Option<String>,
}

/// 解析 MINUTE_DATA 单条记录为 `MinuteShare`。
///
/// 丢弃字段：`index`（内部序号）、`price_milli`（毫表示）。
/// 保留字段：`time_minutes | time, price, volume, amount, avg_price`。
///
/// 返回 `Option<MinuteShare>`：仅当 **time 和 time_minutes 都缺** 或无法解析
/// 时返回 None（调用方 warn + skip，INV-2C）。数值字段（price/volume/amount/
/// avg_price）各自独立可选——live OpenStock 经常对 amount/avg_price 返回 null
/// （见 OPENSTOCK_HANDOFF_2026-07-07.md BUG-C），不应让一条缺字段的记录拖累
/// 整个 envelope。
fn parse_minute_share(
    code: &str,
    raw: &RawMinuteRecord,
    date: chrono::NaiveDate,
) -> Option<crate::data::models::MinuteShare> {
    // BUG-B fallback: prefer `time_minutes`, fall back to `time` when null.
    let time_str = raw.time_minutes.as_deref().or(raw.time.as_deref())?;
    let (hh, mm) = parse_time_minutes(time_str)?;
    let timestamp = date.and_hms_opt(hh, mm, 0)?;
    Some(crate::data::models::MinuteShare {
        code: code.to_string(),
        timestamp,
        price: raw.price,
        volume: raw.volume,
        amount: raw.amount,
        avg_price: raw.avg_price,
    })
}

/// 解析 `time_minutes` 字段为 (HH, MM)。
///
/// 接受两种格式（D4 双格式容错，防御 R2 格式歧义）：
///   - "0930"     → (9, 30)
///   - "09:30"    → (9, 30)
///
/// 长度不匹配或字符非数字 → None（触发 INV-2C skip）。
pub(super) fn parse_time_minutes(s: &str) -> Option<(u32, u32)> {
    let cleaned: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.len() != 4 {
        return None;
    }
    let hh: u32 = cleaned[..2].parse().ok()?;
    let mm: u32 = cleaned[2..].parse().ok()?;
    if hh >= 24 || mm >= 60 {
        return None;
    }
    Some((hh, mm))
}

#[cfg(test)]
mod tests {
    use super::parse_time_minutes;

    #[test]
    fn parse_time_minutes_accepts_compact_format() {
        assert_eq!(parse_time_minutes("0930"), Some((9, 30)));
        assert_eq!(parse_time_minutes("1130"), Some((11, 30)));
        assert_eq!(parse_time_minutes("1500"), Some((15, 0)));
    }

    #[test]
    fn parse_time_minutes_accepts_colon_format() {
        assert_eq!(parse_time_minutes("09:30"), Some((9, 30)));
        assert_eq!(parse_time_minutes("11:30"), Some((11, 30)));
    }

    #[test]
    fn parse_time_minutes_rejects_invalid() {
        assert_eq!(parse_time_minutes("99:99"), None);
        assert_eq!(parse_time_minutes("25:00"), None);
        assert_eq!(parse_time_minutes("12:60"), None);
        assert_eq!(parse_time_minutes("abc"), None);
        assert_eq!(parse_time_minutes("123"), None);
        assert_eq!(parse_time_minutes("12345"), None);
    }
}
