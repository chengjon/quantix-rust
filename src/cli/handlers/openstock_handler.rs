use crate::core::{QuantixError, Result};
use crate::sources::parse_daily_kline_json;
use std::fs;

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
