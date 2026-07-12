//! K-line query and market-fundamentals import commands, plus the file
//! loader that backs the import command.

use crate::cli::handlers::shared_support::get_kline_for_analysis;
use crate::core::{QuantixError, Result};
use crate::sync::{DataSync, MarketFundamentalSyncRecord};
use chrono::NaiveDate;
use std::fs;

/// 查询 K线数据
pub(crate) async fn query_kline_data(
    code: String,
    start: Option<String>,
    end: Option<String>,
    period_type: String,
    limit: usize,
) -> Result<()> {
    println!("📊 查询 K线数据");
    println!("  代码: {}", code);
    println!("  周期: {}", period_type);
    println!("  限制: {}", limit);

    // 解析日期
    let start_date = start
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok());
    let end_date = end
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok());

    // 查询数据 — 走统一 K 线获取入口
    let klines = get_kline_for_analysis(&code, start_date, end_date, Some(limit)).await?;

    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }

    // 显示数据
    println!("\n📈 查询结果 (共 {} 条):", klines.len());
    println!(
        "{:<12} {:<10} {:<10} {:<10} {:<10} {:<12}",
        "日期", "开盘", "最高", "最低", "收盘", "成交量"
    );
    println!("{}", "-".repeat(70));

    for kline in klines.iter().take(20) {
        println!(
            "{:<12} {:<10.2} {:<10.2} {:<10.2} {:<10.2} {:<12}",
            kline.date, kline.open, kline.high, kline.low, kline.close, kline.volume,
        );
    }

    if klines.len() > 20 {
        println!("... (还有 {} 条)", klines.len() - 20);
    }

    Ok(())
}

pub(crate) async fn import_market_fundamentals(input: String) -> Result<()> {
    let records = load_market_fundamental_records(&input)?;
    if records.is_empty() {
        return Err(QuantixError::Other(
            "输入文件中没有可导入的市场基础面记录".to_string(),
        ));
    }

    println!("📥 导入市场基础面快照");
    println!("  文件: {}", input);
    println!("  记录数: {}", records.len());

    let sync = DataSync::with_default_config().await?;
    let stats = sync.sync_market_fundamentals(&records).await?;

    println!("✅ 市场基础面快照导入完成");
    println!("  已写入: {}", stats.records_synced);
    println!("  耗时(秒): {}", stats.elapsed_seconds);
    Ok(())
}

pub(crate) fn load_market_fundamental_records(
    path: &str,
) -> Result<Vec<MarketFundamentalSyncRecord>> {
    let raw = fs::read_to_string(path)
        .map_err(|e| QuantixError::Other(format!("读取市场基础面文件失败 ({}): {}", path, e)))?;

    serde_json::from_str(&raw)
        .map_err(|e| QuantixError::Config(format!("解析市场基础面文件失败 ({}): {}", path, e)))
}
