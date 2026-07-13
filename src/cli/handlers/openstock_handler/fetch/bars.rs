//! 基于 `/data/bars` 直读端点的 OpenStock 实时拉取命令。
//!
//! 从原生 `openstock_handler/fetch.rs` 拆出。调用 `/data/bars` 端点的
//! 命令（指数/K 线、分钟线、分时）不走 `/data/fetch` 信封（不返回
//! `source` / `artifact_hash` / `latency_ms`）。

use crate::core::runtime::OpenStockSettings;
use crate::core::{QuantixError, Result};

pub(crate) async fn fetch_openstock_index(
    settings: &OpenStockSettings,
    symbol: &str,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<()> {
    use crate::sources::openstock_client::OpenStockClient;

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
    use crate::sources::openstock_client::OpenStockClient;

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
