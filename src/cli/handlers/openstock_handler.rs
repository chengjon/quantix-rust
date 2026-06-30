use std::fs;
use std::io::Read;

use crate::core::{QuantixError, Result};
use crate::db::clickhouse::ClickHouseClient;
use crate::sources::openstock::{
    LiveShadowRequest, live_shadow_error_into_quantix, validate_live_shadow_payload,
};
use crate::sources::openstock_calendar::{
    TradeDateRecord, WorkdayRecord, calendar_error_into_quantix, parse_trade_dates, parse_workdays,
};
use crate::sources::openstock_codes::{
    StockCodeRecord, StockListRecord, parse_all_stocks, parse_stock_codes,
    stock_code_error_into_quantix,
};
use crate::sources::openstock_envelope::OpenStockEnvelope;
use crate::sources::openstock_index::{
    IndexKlineParseError, IndexKlineRecord, index_kline_error_into_quantix, parse_index_klines,
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
            let trading = workdays.iter().filter(|w| w.is_trading_day).count();
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
    let _ = (IndexKlineParseError::EmptyRecords,); // silence unused import if enum unused
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
