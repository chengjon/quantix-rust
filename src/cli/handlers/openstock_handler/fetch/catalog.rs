//! 基于信封（envelope）的 OpenStock 实时拉取命令。
//!
//! 从原生 `openstock_handler/fetch.rs` 拆出。调用 `/data/fetch` 端点的
//! 命令通过 OpenStockClient 获取带 `source` / `artifact_hash` /
//! `latency_ms` 信封字段的响应。

use crate::core::runtime::OpenStockSettings;
use crate::core::{QuantixError, Result};
use crate::sources::openstock_client::OpenStockClient;

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
