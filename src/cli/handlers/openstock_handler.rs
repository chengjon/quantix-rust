use std::fs;
use std::io::Read;

use crate::core::runtime::OpenStockSettings;
use crate::core::{QuantixError, Result};
use crate::db::clickhouse::ClickHouseClient;
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
use crate::sources::openstock_shadow::{
    ShadowWriteError, new_batch_id, rollback_shadow_batch, verify_shadow_batch, write_shadow_klines,
};
use crate::sources::parse_daily_kline_json;

pub(crate) fn validate_openstock_fixture(file: &str) -> Result<()> {
    let content = fs::read_to_string(file).map_err(|error| {
        QuantixError::Other(format!("读取 OpenStock fixture 失败 ({}): {}", file, error))
    })?;
    let klines = parse_daily_kline_json(&content).map_err(|error| {
        QuantixError::Other(format!("解析 OpenStock fixture 失败 ({}): {}", file, error))
    })?;
    let first = klines.first().ok_or_else(|| {
        QuantixError::Other(format!("OpenStock fixture 没有可校验记录: {}", file))
    })?;
    let last = klines.last().ok_or_else(|| {
        QuantixError::Other(format!("OpenStock fixture 没有可校验记录: {}", file))
    })?;

    println!("OpenStock 本地 fixture 校验");
    println!("  文件: {}", file);
    println!("  来源: local_fixture");
    println!("  记录数: {}", klines.len());
    println!("  代码: {}", first.code);
    println!("  日期范围: {}..{}", first.date, last.date);
    println!("  复权: {:?}", first.adjust_type);

    Ok(())
}

pub(crate) fn validate_openstock_live(
    payload_path: &str,
    symbol: &str,
    period: &str,
    start: &str,
    end: &str,
    limit: Option<u32>,
) -> Result<()> {
    let payload = read_payload(payload_path)?;
    let request = LiveShadowRequest {
        symbol: symbol.to_string(),
        period: period.to_string(),
        start_date: start.to_string(),
        end_date: end.to_string(),
        limit,
    };
    let report =
        validate_live_shadow_payload(&payload, &request).map_err(live_shadow_error_into_quantix)?;

    print!("{report}");
    Ok(())
}

pub(crate) fn validate_openstock_codes(payload_path: &str, kind: Option<&str>) -> Result<()> {
    let payload = read_payload(payload_path)?;
    let kind_str = kind.unwrap_or("codes");
    match kind_str {
        "codes" => {
            let env: OpenStockEnvelope<StockCodeRecord> = serde_json::from_str(&payload)
                .map_err(|e| QuantixError::Other(format!("codes envelope 反序列化失败: {}", e)))?;
            let codes = parse_stock_codes(env).map_err(stock_code_error_into_quantix)?;
            println!("OpenStock codes 校验 (STOCK_CODES)");
            println!("  来源: source field absent or captured separately");
            println!("  记录数: {}", codes.len());
            if let (Some(first), Some(last)) = (codes.first(), codes.last()) {
                println!("  首条: code={} name={:?}", first.code, first.name);
                println!("  末条: code={} name={:?}", last.code, last.name);
            }
        }
        "all_stocks" => {
            let env: OpenStockEnvelope<StockListRecord> =
                serde_json::from_str(&payload).map_err(|e| {
                    QuantixError::Other(format!("all_stocks envelope 反序列化失败: {}", e))
                })?;
            let entries = parse_all_stocks(env).map_err(stock_code_error_into_quantix)?;
            println!("OpenStock codes 校验 (ALL_STOCKS)");
            println!("  记录数: {}", entries.len());
            if let Some(first) = entries.first() {
                println!(
                    "  首条: code={} market={:?} listing_date={:?}",
                    first.code,
                    first.market,
                    first.listing_date.map(|d| d.to_string())
                );
            }
        }
        other => {
            return Err(QuantixError::Other(format!(
                "validate-codes kind 不支持: {} (期望 codes 或 all_stocks)",
                other
            )));
        }
    }
    Ok(())
}

pub(crate) fn validate_openstock_calendar(payload_path: &str, kind: &str) -> Result<()> {
    let payload = read_payload(payload_path)?;
    match kind {
        "trade_dates" => {
            let env: OpenStockEnvelope<TradeDateRecord> =
                serde_json::from_str(&payload).map_err(|e| {
                    QuantixError::Other(format!("trade_dates envelope 反序列化失败: {}", e))
                })?;
            let dates = parse_trade_dates(env).map_err(calendar_error_into_quantix)?;
            println!("OpenStock calendar 校验 (TRADE_DATES)");
            println!("  记录数: {}", dates.len());
            if let (Some(first), Some(last)) = (dates.first(), dates.last()) {
                println!("  首条: {}", first.date);
                println!("  末条: {}", last.date);
            }
        }
        "workdays" => {
            let env: OpenStockEnvelope<WorkdayRecord> =
                serde_json::from_str(&payload).map_err(|e| {
                    QuantixError::Other(format!("workdays envelope 反序列化失败: {}", e))
                })?;
            let workdays = parse_workdays(env).map_err(calendar_error_into_quantix)?;
            let trading = workdays
                .iter()
                .filter(|w| w.is_workday.unwrap_or(false) || w.today_is_workday.unwrap_or(false))
                .count();
            println!("OpenStock calendar 校验 (WORKDAYS)");
            println!("  记录数: {}", workdays.len());
            println!("  其中交易日: {}", trading);
        }
        other => {
            return Err(QuantixError::Other(format!(
                "validate-calendar kind 不支持: {} (期望 trade_dates 或 workdays)",
                other
            )));
        }
    }
    Ok(())
}

pub(crate) fn validate_openstock_index(
    payload_path: &str,
    symbol: &str,
    _start: Option<&str>,
    _end: Option<&str>,
) -> Result<()> {
    let payload = read_payload(payload_path)?;
    let env: OpenStockEnvelope<IndexKlineRecord> = serde_json::from_str(&payload)
        .map_err(|e| QuantixError::Other(format!("index_klines envelope 反序列化失败: {}", e)))?;
    let klines = parse_index_klines(env)
        .map_err(index_kline_error_into_quantix)
        .map_err(|e| match e {
            QuantixError::DataParse(_) => {
                QuantixError::DataParse(format!("{} (请求 symbol={})", e, symbol))
            }
            other => other,
        })?;
    println!("OpenStock index 校验 (INDEX_KLINES)");
    println!("  请求 symbol: {}", symbol);
    println!("  记录数: {}", klines.len());
    if let (Some(first), Some(last)) = (klines.first(), klines.last()) {
        println!(
            "  首条: code={} date={} close={}",
            first.code, first.date, first.close
        );
        println!(
            "  末条: code={} date={} close={}",
            last.code, last.date, last.close
        );
    }
    // _start/_end unused for now — kept for symmetry with validate-live.
    Ok(())
}

pub(crate) async fn fetch_openstock_codes(settings: &OpenStockSettings) -> Result<()> {
    let client = OpenStockClient::from_settings(settings)?;
    let resp = client.fetch_stock_codes().await?;
    let source = if resp.source.is_empty() {
        "(unknown)".to_string()
    } else {
        resp.source.clone()
    };
    println!("OpenStock live fetch (STOCK_CODES)");
    println!("  来源: {}", source);
    println!("  记录数: {}", resp.records.len());
    if let (Some(first), Some(last)) = (resp.records.first(), resp.records.last()) {
        let first_sym = first
            .extra
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let first_mkt = first
            .extra
            .get("market")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let last_sym = last
            .extra
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let last_mkt = last
            .extra
            .get("market")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        println!(
            "  首条: code={:?} name={:?} symbol={:?} market={:?}",
            first.code, first.name, first_sym, first_mkt
        );
        println!(
            "  末条: code={:?} name={:?} symbol={:?} market={:?}",
            last.code, last.name, last_sym, last_mkt
        );
    }
    println!("  artifact_hash: {}", resp.artifact_hash);
    println!(
        "  latency_ms:    {}",
        resp.latency_ms
            .map(|n| n.to_string())
            .unwrap_or_else(|| "(not reported)".to_string())
    );
    Ok(())
}

pub(crate) async fn fetch_openstock_calendar(
    settings: &OpenStockSettings,
    year: Option<u32>,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<()> {
    // 解析互斥组：clap group 保证三者只可能出现 (a) year=Some 或 (b) start/end 任一组合
    let (effective_start, effective_end, hint) = match (year, start, end) {
        (Some(y), None, None) => (
            Some(format!("{:04}-01-01", y)),
            Some(format!("{:04}-12-31", y)),
            format!("year={} → 展开为 {:04}-01-01..{:04}-12-31", y, y, y),
        ),
        (None, s, e) => {
            let hint = match (s, e) {
                (Some(s), Some(e)) => format!("start={}, end={}", s, e),
                (Some(s), None) => format!("start={} (end 开放)", s),
                (None, Some(e)) => format!("(start 开放), end={}", e),
                (None, None) => "(无范围，runtime 会返回全历史且可能截断)".to_string(),
            };
            (s.map(|x| x.to_string()), e.map(|x| x.to_string()), hint)
        }
        _ => {
            return Err(QuantixError::Other(
                "fetch-calendar: --year 与 --start/--end 互斥（clap 应已阻止）".to_string(),
            ));
        }
    };

    let client = OpenStockClient::from_settings(settings)?;
    let resp = client
        .fetch_trade_dates(effective_start.as_deref(), effective_end.as_deref())
        .await?;
    let source = if resp.source.is_empty() {
        "(unknown)".to_string()
    } else {
        resp.source.clone()
    };
    println!("OpenStock live fetch (TRADE_DATES)");
    println!("  范围: {}", hint);
    println!("  来源: {}", source);
    println!("  记录数: {}", resp.records.len());
    if let (Some(first), Some(last)) = (resp.records.first(), resp.records.last()) {
        println!("  首条: {:?}", first.date);
        println!("  末条: {:?}", last.date);
    }
    if let (Some(req_end), Some(last)) = (effective_end.as_deref(), resp.records.last())
        && let Some(last_date) = last.date.as_deref()
        && last_date != req_end
    {
        println!(
            "  ⚠️ 末条 {} 早于请求 end={}（可能被 runtime 截断，建议分段拉取）",
            last_date, req_end
        );
    }
    println!("  artifact_hash: {}", resp.artifact_hash);
    println!(
        "  latency_ms:    {}",
        resp.latency_ms
            .map(|n| n.to_string())
            .unwrap_or_else(|| "(not reported)".to_string())
    );
    Ok(())
}

pub(crate) async fn fetch_openstock_index(
    settings: &OpenStockSettings,
    symbol: &str,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<()> {
    let client = OpenStockClient::from_settings(settings)?;
    let resp = client.fetch_index_klines(symbol, start, end).await?;
    let source = if resp.source.is_empty() {
        "(unknown)".to_string()
    } else {
        resp.source.clone()
    };
    println!("OpenStock live fetch (INDEX_KLINES, symbol={})", symbol);
    println!("  来源: {}", source);
    println!("  记录数: {}", resp.records.len());
    if let (Some(first), Some(last)) = (resp.records.first(), resp.records.last()) {
        println!(
            "  首条: symbol={:?} time={:?} close={:?}",
            first.symbol, first.time, first.close
        );
        println!(
            "  末条: symbol={:?} time={:?} close={:?}",
            last.symbol, last.time, last.close
        );
    }
    println!("  artifact_hash: {}", resp.artifact_hash);
    println!(
        "  latency_ms:    {}",
        resp.latency_ms
            .map(|n| n.to_string())
            .unwrap_or_else(|| "(not reported)".to_string())
    );
    Ok(())
}

/// 实时拉取多周期 K 线（P0.13a）。
///
/// 通过 `/data/bars` 端点拉取 day/week/month 周期 + none/qfq/hfq 复权的
/// K 线数据。`--period` 与 `--adjust` 通过 `FromStr` 严格解析，非法值在
/// 任何 HTTP 请求之前即以 `QuantixError::Config` 快速失败。
///
/// 注意：`/data/bars` 不返回 `/data/fetch` 信封中的 `source` /
/// `artifact_hash` / `latency_ms` 字段，因此本 handler 不打印这些字段
/// （与 `fetch_openstock_index` 不同）。
pub(crate) async fn fetch_openstock_klines(
    settings: &OpenStockSettings,
    symbol: &str,
    period: &str,
    adjust: &str,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<()> {
    use std::str::FromStr;

    use crate::data::models::{AdjustType, BarPeriod};

    let period_enum =
        BarPeriod::from_str(period).map_err(|e| QuantixError::Config(format!("--period {}", e)))?;
    let adjust_enum = AdjustType::from_str(adjust)
        .map_err(|e| QuantixError::Config(format!("--adjust {}", e)))?;

    let client = OpenStockClient::from_settings(settings)?;
    let klines = client
        .fetch_klines(symbol, period_enum, adjust_enum, start, end)
        .await?;

    println!("OpenStock live fetch (/data/bars, symbol={})", symbol);
    println!("  Period:  {}", period_enum.as_str());
    println!(
        "  Adjust:  {}",
        adjust_enum
            .as_openstock_param()
            .unwrap_or("none (field omitted)")
    );
    println!("  记录数: {}", klines.len());
    if let (Some(first), Some(last)) = (klines.first(), klines.last()) {
        println!(
            "  首条: date={} open={} close={}",
            first.date, first.open, first.close
        );
        println!(
            "  末条: date={} open={} close={}",
            last.date, last.open, last.close
        );
    }
    // /data/bars is a direct reqwest path; it does NOT echo source,
    // artifact_hash, or latency_ms (only the /data/fetch envelope does).
    println!("  Source:        (not reported by /data/bars)");
    println!("  artifact_hash: (not reported by /data/bars)");
    println!("  latency_ms:    (not reported by /data/bars)");
    Ok(())
}

/// 实时拉取分钟级 K 线（P0.13b-1）。
///
/// 通过 `/data/bars` 端点拉取 1m|5m|15m|30m|60m 周期 + none/qfq/hfq 复权的
/// 分钟级 K 线数据。`--period` 与 `--adjust` 通过 `FromStr` 严格解析，
/// 非法值在任何 HTTP 请求之前即以 `QuantixError::Config` 快速失败。
///
/// 注意：`/data/bars` 不返回 `/data/fetch` 信封中的 `source` /
/// `artifact_hash` / `latency_ms` 字段，因此本 handler 不打印这些字段
/// （与 `fetch_openstock_klines` 一致）。
#[allow(clippy::too_many_arguments)]
pub(crate) async fn fetch_openstock_minute_klines(
    settings: &OpenStockSettings,
    symbol: String,
    period: String,
    date: Option<String>,
    start: Option<String>,
    end: Option<String>,
    adjust: String,
    stream: bool,
) -> Result<()> {
    use std::str::FromStr;

    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
    use crate::sources::openstock_client::OpenStockClient;
    use futures::StreamExt;

    let period_enum = MinutePeriod::from_str(&period)
        .map_err(|e| QuantixError::Config(format!("--period: {}", e)))?;
    let adjust_enum = AdjustType::from_str(&adjust)
        .map_err(|e| QuantixError::Config(format!("--adjust: {}", e)))?;
    let dor = DateOrRange::from_cli(date.as_deref(), start.as_deref(), end.as_deref())?;

    let client = OpenStockClient::from_settings(settings)?;

    if stream {
        let mode_label = match &dor {
            DateOrRange::Date(d) => format!("date={}", d),
            DateOrRange::Range { start, end } => format!("range={}..{}", start, end),
        };
        println!(
            "OpenStock stream fetch (/data/bars, symbol={}, minute={}, {})",
            symbol,
            period_enum.as_str(),
            mode_label
        );
        println!(
            "  Adjust: {}",
            adjust_enum
                .as_openstock_param()
                .unwrap_or("none (field omitted)")
        );
        eprintln!("  Streaming weekly chunks:");
        let s = client.fetch_minute_klines_stream(&symbol, period_enum, dor.clone(), adjust_enum);
        futures::pin_mut!(s);
        let mut total = 0usize;
        let mut batches = 0usize;
        let started = std::time::Instant::now();
        while let Some(result) = s.next().await {
            let batch = result?;
            batches += 1;
            total += batch.len();
            eprintln!(
                "  [batch {}] +{} bars (cumulative: {}, elapsed: {:?})",
                batches,
                batch.len(),
                total,
                started.elapsed()
            );
            for bar in &batch {
                println!("{:?}", bar);
            }
        }
        eprintln!(
            "  Done. Total: {} bars across {} batches, {:?} total",
            total,
            batches,
            started.elapsed()
        );
        return Ok(());
    }

    let bars = client
        .fetch_minute_klines(&symbol, period_enum, dor.clone(), adjust_enum)
        .await?;

    let mode_label = match &dor {
        DateOrRange::Date(d) => format!("date={}", d),
        DateOrRange::Range { start, end } => format!("range={}..{}", start, end),
    };
    println!(
        "OpenStock live fetch (/data/bars, symbol={}, minute={}, {})",
        symbol,
        period_enum.as_str(),
        mode_label
    );
    println!(
        "  Adjust: {}",
        adjust_enum
            .as_openstock_param()
            .unwrap_or("none (field omitted)")
    );
    println!("  记录数: {}", bars.len());
    if !bars.is_empty() {
        println!("  First:  {:?}", bars.first());
        println!("  Last:   {:?}", bars.last());
    }
    println!("  Source:        (not reported by /data/bars)");
    println!("  artifact_hash: (not reported by /data/bars)");
    println!("  latency_ms:    (not reported by /data/bars)");
    if bars.len() > 10_000 {
        eprintln!(
            "warning: range returns {} records, consider narrowing or use --stream",
            bars.len()
        );
    }
    Ok(())
}

pub(crate) async fn fetch_openstock_minute_share(
    settings: &OpenStockSettings,
    symbol: String,
    date: Option<String>,
    start: Option<String>,
    end: Option<String>,
    stream: bool,
) -> Result<()> {
    use crate::data::models::DateOrRange;
    use crate::sources::openstock_client::OpenStockClient;
    use futures::StreamExt;

    let dor = DateOrRange::from_cli(date.as_deref(), start.as_deref(), end.as_deref())?;
    let client = OpenStockClient::from_settings(settings)?;

    if stream {
        let mode_label = match &dor {
            DateOrRange::Date(d) => format!("date={}", d),
            DateOrRange::Range { start, end } => format!("range={}..{}", start, end),
        };
        println!("OpenStock MINUTE_DATA stream (time-share ticks)");
        println!("  Code:     {}", symbol);
        println!("  Mode:     {}", mode_label);
        eprintln!("  Streaming one batch per calendar day:");
        let s = client.fetch_minute_share_stream(&symbol, dor.clone());
        futures::pin_mut!(s);
        let mut total = 0usize;
        let mut batches = 0usize;
        let mut empty_batches = 0usize;
        let started = std::time::Instant::now();
        while let Some(result) = s.next().await {
            let batch = result?;
            batches += 1;
            if batch.is_empty() {
                empty_batches += 1;
            }
            total += batch.len();
            eprintln!(
                "  [batch {}] +{} records (cumulative: {}, empty: {}, elapsed: {:?})",
                batches,
                batch.len(),
                total,
                empty_batches,
                started.elapsed()
            );
            for share in &batch {
                println!("{:?}", share);
            }
        }
        eprintln!(
            "  Done. Total: {} records across {} batches ({} empty), {:?} total",
            total,
            batches,
            empty_batches,
            started.elapsed()
        );
        return Ok(());
    }

    let started = std::time::Instant::now();
    let shares = client.fetch_minute_share(&symbol, dor.clone()).await?;
    let latency_ms = started.elapsed().as_millis();

    let base_url = settings.base_url.as_deref().unwrap_or("(not set)");
    let mode_label = match &dor {
        DateOrRange::Date(d) => format!("date={}", d),
        DateOrRange::Range { start, end } => format!("range={}..{}", start, end),
    };
    println!("OpenStock MINUTE_DATA (time-share ticks)");
    println!("  Code:     {}", symbol);
    println!("  Mode:     {}", mode_label);
    println!("  Endpoint: {}/data/fetch", base_url);
    println!("  Records:  {}", shares.len());
    if let (Some(first), Some(last)) = (shares.first(), shares.last()) {
        println!("  First:    {:?}", first);
        println!("  Last:     {:?}", last);
    }
    println!("  latency_ms: {}", latency_ms);
    if let DateOrRange::Range { start, end } = &dor {
        let n_days = (*end - *start).num_days() + 1;
        if n_days > 10 {
            eprintln!(
                "warning: range spans {} days; consider using --stream for live progress",
                n_days
            );
        }
    }
    Ok(())
}

pub(crate) async fn fetch_openstock_all_stocks(
    settings: &OpenStockSettings,
    day: Option<&str>,
) -> Result<()> {
    let client = OpenStockClient::from_settings(settings)?;
    let resp = client.fetch_all_stocks(day).await?;
    let source = if resp.source.is_empty() {
        "(unknown)".to_string()
    } else {
        resp.source.clone()
    };
    println!("OpenStock live fetch (ALL_STOCKS, day={:?})", day);
    println!("  来源: {}", source);
    println!("  记录数: {}", resp.records.len());
    if let (Some(first), Some(last)) = (resp.records.first(), resp.records.last()) {
        println!(
            "  首条: code={:?} name={:?} market={:?} trade_status={:?}",
            first.code, first.name, first.market, first.trade_status
        );
        println!(
            "  末条: code={:?} name={:?} market={:?} trade_status={:?}",
            last.code, last.name, last.market, last.trade_status
        );
    }
    println!("  artifact_hash: {}", resp.artifact_hash);
    println!(
        "  latency_ms:    {}",
        resp.latency_ms
            .map(|n| n.to_string())
            .unwrap_or_else(|| "(not reported)".to_string())
    );
    Ok(())
}

pub(crate) async fn fetch_openstock_workdays(
    settings: &OpenStockSettings,
    action: &str,
    date: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<()> {
    let client = OpenStockClient::from_settings(settings)?;
    let resp = client.fetch_workdays(action, date, start, end).await?;
    let source = if resp.source.is_empty() {
        "(unknown)".to_string()
    } else {
        resp.source.clone()
    };
    let trading = resp
        .records
        .iter()
        .filter(|w| w.is_workday.unwrap_or(false) || w.today_is_workday.unwrap_or(false))
        .count();
    let params_hint = match action {
        "range" => format!("range={}..{}", start.unwrap_or("?"), end.unwrap_or("?")),
        "is_workday" | "next_workday" | "previous_workday" => {
            format!("date={}", date.unwrap_or("?"))
        }
        _ => String::new(),
    };
    println!(
        "OpenStock live fetch (WORKDAYS, action={}{})",
        action,
        if params_hint.is_empty() {
            String::new()
        } else {
            format!(", {}", params_hint)
        }
    );
    println!("  来源: {}", source);
    println!("  记录数: {}", resp.records.len());
    println!("  其中交易日: {}", trading);
    if let (Some(first), Some(last)) = (resp.records.first(), resp.records.last()) {
        println!(
            "  首条: action={:?} date={:?} is_workday={:?} today_is_workday={:?} next_workday={:?} previous_workday={:?}",
            first.action,
            first.date,
            first.is_workday,
            first.today_is_workday,
            first.next_workday,
            first.previous_workday
        );
        println!(
            "  末条: action={:?} date={:?} is_workday={:?} today_is_workday={:?} next_workday={:?} previous_workday={:?}",
            last.action,
            last.date,
            last.is_workday,
            last.today_is_workday,
            last.next_workday,
            last.previous_workday
        );
    }
    println!("  artifact_hash: {}", resp.artifact_hash);
    println!(
        "  latency_ms:    {}",
        resp.latency_ms
            .map(|n| n.to_string())
            .unwrap_or_else(|| "(not reported)".to_string())
    );
    Ok(())
}

fn read_payload(payload_path: &str) -> Result<String> {
    if payload_path == "-" {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| QuantixError::Other(format!("读取 stdin 失败: {}", error)))?;
        Ok(buffer)
    } else {
        fs::read_to_string(payload_path).map_err(|error| {
            QuantixError::Other(format!(
                "读取 OpenStock 线上响应失败 ({}): {}",
                payload_path, error
            ))
        })
    }
}

const SHADOW_ENV_CONFIRM: &str = "QUANTIX_SHADOW_PERSIST_CONFIRM";
const SHADOW_INGESTED_BY: &str = "quantix-cli";

async fn shadow_client() -> Result<ClickHouseClient> {
    let settings = crate::core::runtime::ClickHouseSettings::from_env();
    ClickHouseClient::from_settings(&settings)
        .await
        .map_err(|e| QuantixError::Other(format!("创建 ClickHouse 客户端失败: {}", e)))
}

fn shadow_env_confirmed() -> bool {
    std::env::var(SHADOW_ENV_CONFIRM).ok().as_deref() == Some("yes")
}

fn map_shadow_write_error(error: ShadowWriteError) -> QuantixError {
    let msg = match error {
        ShadowWriteError::ApplyFlagRequired => {
            "shadow 写入需要 --apply 标志（当前仅 dry-run）".to_string()
        }
        ShadowWriteError::EnvConfirmRequired => format!(
            "shadow 写入需要环境变量 {}=yes（双保险未通过）",
            SHADOW_ENV_CONFIRM
        ),
        ShadowWriteError::FailClosedNotEmpty { count } => {
            format!("shadow 拒绝写入：{} 条 fail-closed 解析错误", count)
        }
        ShadowWriteError::DriftNotEmpty { count } => {
            format!(
                "shadow 拒绝写入：{} 条 drift（请求与服务端返回不一致）",
                count
            )
        }
        ShadowWriteError::EmptyPayload => "shadow 拒绝写入：映射后 0 行".to_string(),
        ShadowWriteError::MappedCountMismatch {
            record_count,
            mapped_count,
        } => format!(
            "shadow 拒绝写入：record_count={} 与 mapped_count={} 不一致",
            record_count, mapped_count
        ),
        ShadowWriteError::DuplicateKeys { count } => {
            format!(
                "shadow 拒绝写入：{} 条重复 (source, period, code, date, adjust_type) 键",
                count
            )
        }
        ShadowWriteError::DbError(inner) => format!("shadow ClickHouse 错误：{}", inner),
    };
    QuantixError::Other(msg)
}

pub(crate) async fn persist_openstock_live(
    payload_path: &str,
    symbol: &str,
    period: &str,
    start: &str,
    end: &str,
    limit: Option<u32>,
    apply: bool,
) -> Result<()> {
    let payload = read_payload(payload_path)?;
    let request = LiveShadowRequest {
        symbol: symbol.to_string(),
        period: period.to_string(),
        start_date: start.to_string(),
        end_date: end.to_string(),
        limit,
    };
    let report =
        validate_live_shadow_payload(&payload, &request).map_err(live_shadow_error_into_quantix)?;
    let batch_id = new_batch_id();
    let env_confirmed = shadow_env_confirmed();

    let client = shadow_client().await?;
    let write_report = write_shadow_klines(
        &client,
        &report,
        &payload,
        &batch_id,
        SHADOW_INGESTED_BY,
        apply,
        env_confirmed,
    )
    .await
    .map_err(map_shadow_write_error)?;

    println!("OpenStock shadow persist");
    println!("  batch_id: {}", write_report.batch_id);
    println!("  artifact_hash: {}", write_report.artifact_hash);
    println!("  dry_run: {}", write_report.dry_run);
    println!("  applied: {}", write_report.applied);
    println!("  row_count: {}", write_report.row_count);
    if write_report.dry_run && apply {
        println!("  hint: 设 {}=yes 后再跑一次以真正写入", SHADOW_ENV_CONFIRM);
    }
    Ok(())
}

pub(crate) async fn shadow_rollback(batch_id: &str) -> Result<()> {
    let client = shadow_client().await?;
    let removed = rollback_shadow_batch(&client, batch_id)
        .await
        .map_err(map_shadow_write_error)?;
    println!("OpenStock shadow rollback");
    println!("  batch_id: {}", batch_id);
    println!("  rows_removed: {}", removed);
    Ok(())
}

pub(crate) async fn shadow_verify(batch_id: &str) -> Result<()> {
    let client = shadow_client().await?;
    let count = verify_shadow_batch(&client, batch_id)
        .await
        .map_err(map_shadow_write_error)?;
    println!("OpenStock shadow verify");
    println!("  batch_id: {}", batch_id);
    println!("  rows_present: {}", count);
    Ok(())
}

// ============================================================================
// OpenStock import-* canonical paths.
// Reachable via DataCommands::ImportTicks / ImportKlines.
// ============================================================================

/// `quantix data import-ticks` (OpenStock only).
///
/// Writes to TDengine
/// gated by `--apply` + `QUANTIX_OPENSTOCK_TICK_APPLY=yes`; default dry-run.
pub(crate) async fn import_openstock_ticks(
    code: &str,
    date: Option<&str>,
    apply: bool,
) -> Result<()> {
    use crate::core::config::AppConfig;
    use crate::db::TDengineClient;
    use crate::sources::openstock_ticks::parse_tick_data;

    let osc = OpenStockClient::from_env()?;
    let resp = osc
        .fetch_tick_data(code, date)
        .await
        .map_err(|e| QuantixError::Other(format!("fetch_tick_data: {e}")))?;
    let envelope = OpenStockEnvelope {
        data: resp.records,
        source: Some(resp.source.clone()),
        data_category: Some("TICK_DATA".to_string()),
        request_id: None,
        route_decision_id: None,
        quality_flags: Vec::new(),
        cache_state: None,
        circuit_state: None,
        latency_ms: resp.latency_ms,
        received_at: resp.received_at.clone(),
    };
    let (meta, ticks) = parse_tick_data(envelope)
        .map_err(|e| QuantixError::DataParse(format!("parse_tick_data: {e}")))?;

    println!("OpenStock import-ticks dry-run (category=TICK_DATA)");
    println!("  代码:    {}", code);
    println!("  日期:    {}", date.unwrap_or("(latest)"));
    println!("  来源:    {}", resp.source);
    println!("  Tick 数: {}", ticks.len());
    if let Some(trading_date) = meta.trading_date.as_deref() {
        println!("  交易日:  {}", trading_date);
    }
    if let Some(first) = ticks.first() {
        println!(
            "  首条:    {} price={} vol={} amount={} dir={:?}",
            first.timestamp, first.price, first.volume, first.amount, first.direction
        );
    }
    if let Some(last) = ticks.last() {
        println!(
            "  末条:    {} price={} vol={} amount={} dir={:?}",
            last.timestamp, last.price, last.volume, last.amount, last.direction
        );
    }
    println!("  artifact_hash: {}", resp.artifact_hash);
    if let Some(ms) = resp.latency_ms {
        println!("  latency_ms:    {}", ms);
    }

    if ticks.is_empty() {
        println!("  → 无 tick 数据; 跳过写入");
        return Ok(());
    }

    if !apply {
        println!("  → dry-run; 加 --apply 实际写入 (需 QUANTIX_OPENSTOCK_TICK_APPLY=yes)");
        return Ok(());
    }
    if std::env::var("QUANTIX_OPENSTOCK_TICK_APPLY")
        .ok()
        .as_deref()
        != Some("yes")
    {
        return Err(QuantixError::Other(
            "已 --apply 但 QUANTIX_OPENSTOCK_TICK_APPLY != yes; 拒绝写入 TDengine".to_string(),
        ));
    }

    let config =
        AppConfig::load("config").map_err(|e| QuantixError::Other(format!("加载配置失败: {e}")))?;
    let td = config
        .database
        .tdengine
        .ok_or_else(|| QuantixError::Config("缺少 TDengine 配置".to_string()))?;
    let token = format!("{}:{}", td.username, td.password);
    let tde = TDengineClient::new_with_database(
        &format!("http://{}:{}", td.host, td.port),
        &token,
        &td.database,
    )?;
    tde.check_connection().await?;
    tde.create_tick_table().await?;

    let rows: Vec<(i64, f64, i32, f64, i32)> = ticks
        .iter()
        .map(|t| {
            let ts_ms = t.timestamp.and_utc().timestamp_millis();
            let price_f = super::decimal_to_f64(t.price, "import-ticks")?;
            let amount_f = super::decimal_to_f64(t.amount, "import-ticks")?;
            // Maps TradeDirection → direction TINYINT byte (TDengine schema).
            let status_i = match t.direction {
                crate::data::models::TradeDirection::Buy => 1,
                crate::data::models::TradeDirection::Sell => -1,
                crate::data::models::TradeDirection::Neutral => 0,
            };
            Ok::<(i64, f64, i32, f64, i32), QuantixError>((
                ts_ms,
                price_f,
                t.volume as i32,
                amount_f,
                status_i,
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    tde.insert_ticks(code, &rows).await?;
    println!(
        "  → 已写入 TDengine ({} 条 tick, source=OPENSTOCK)",
        rows.len()
    );
    Ok(())
}

/// `quantix data import-klines` (OpenStock only).
///
/// Writes to ClickHouse
/// `kline_data` table gated by `--apply` + `QUANTIX_OPENSTOCK_KLINE_APPLY=yes`;
/// default dry-run.
pub(crate) async fn import_openstock_klines(
    code: &str,
    kline_type: &str,
    start: Option<&str>,
    end: Option<&str>,
    apply: bool,
) -> Result<()> {
    use crate::db::ClickHouseClient;
    use crate::sources::openstock_index::parse_index_klines;

    // 选择 category: 指数代码 (sh/sz/cn 前缀) 用 INDEX_KLINES,
    // 其余股票代码用 HISTORICAL_KLINES。
    let is_index = code.starts_with("sh.") || code.starts_with("sz.") || code.starts_with("cn.");
    let osc = OpenStockClient::from_env()?;
    let resp = if is_index {
        osc.fetch_index_klines(code, start, end).await?
    } else {
        osc.fetch_historical_klines(code, start, end).await?
    };

    let envelope = OpenStockEnvelope {
        data: resp.records,
        source: Some(resp.source.clone()),
        data_category: Some(
            if is_index {
                "INDEX_KLINES"
            } else {
                "HISTORICAL_KLINES"
            }
            .to_string(),
        ),
        request_id: None,
        route_decision_id: None,
        quality_flags: Vec::new(),
        cache_state: None,
        circuit_state: None,
        latency_ms: resp.latency_ms,
        received_at: resp.received_at.clone(),
    };
    let klines =
        parse_index_klines(envelope).map_err(|e| QuantixError::DataParse(e.to_string()))?;

    println!(
        "OpenStock import-klines dry-run (category={})",
        if is_index {
            "INDEX_KLINES"
        } else {
            "HISTORICAL_KLINES"
        }
    );
    println!("  代码:    {}", code);
    println!("  来源:    {}", resp.source);
    println!("  记录数:  {}", klines.len());
    if let Some(first) = klines.first() {
        println!(
            "  首条:    {} O={} H={} L={} C={}",
            first.date, first.open, first.high, first.low, first.close
        );
    }
    if let Some(last) = klines.last() {
        println!(
            "  末条:    {} O={} H={} L={} C={}",
            last.date, last.open, last.high, last.low, last.close
        );
    }
    println!("  artifact_hash: {}", resp.artifact_hash);
    if let Some(ms) = resp.latency_ms {
        println!("  latency_ms:    {}", ms);
    }

    if !apply {
        println!("  → dry-run; 加 --apply 实际写入 (需 QUANTIX_OPENSTOCK_KLINE_APPLY=yes)");
        return Ok(());
    }
    if std::env::var("QUANTIX_OPENSTOCK_KLINE_APPLY")
        .ok()
        .as_deref()
        != Some("yes")
    {
        return Err(QuantixError::Other(
            "已 --apply 但 QUANTIX_OPENSTOCK_KLINE_APPLY != yes; 拒绝写入 kline_data 主表"
                .to_string(),
        ));
    }

    let ch = ClickHouseClient::with_default_config().await?;
    ch.check_connection().await?;
    ch.insert_kline_data_batch_with_source(&klines, kline_type, "OPENSTOCK")
        .await?;
    println!(
        "  → 已写入 ClickHouse kline_data ({} 条, source=OPENSTOCK)",
        klines.len()
    );
    Ok(())
}
