//! Auto-extracted child module of openstock_handler.rs
use super::*;

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
