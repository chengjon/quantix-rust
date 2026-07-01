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
