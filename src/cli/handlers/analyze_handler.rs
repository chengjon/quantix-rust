#![allow(clippy::too_many_arguments, clippy::collapsible_if)]

use crate::analysis::candle_patterns::{
    CandleInput, MarketBias, PatternConfig, ReferencePricePolicy, recognize_sequence,
};
use crate::analysis::polars_adapter::{PolarsCalculator, from_kline_vec};
use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use crate::sources::TdxDayFile;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::path::Path;
use std::str::FromStr;

use super::*;

pub(crate) async fn calculate_indicators(code: String, indicators_str: String) -> Result<()> {
    println!("💹 计算技术指标");
    println!("  代码: {}", code);
    println!("  指标: {}", indicators_str);

    let client = create_clickhouse_client().await?;
    let klines = client
        .get_kline_data(&code, "1d", None, None, Some(1000))
        .await?;

    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }

    let batch_data = from_kline_vec(&klines);
    let calc = PolarsCalculator::new();
    let indicators: Vec<&str> = indicators_str.split(',').collect();
    let results = calc.calculate_batch(&batch_data, &indicators);

    println!("\n📊 计算结果:");
    println!(
        "{:<12} {:<20} {:<15} {:<15}",
        "日期", "收盘价", "指标", "值"
    );
    println!("{}", "-".repeat(65));

    for (i, kline) in klines.iter().enumerate().take(20) {
        println!("{:<12} {:<20.2}", kline.date, kline.close,);

        for indicator in &indicators {
            if let Some(values) = results.get(*indicator) {
                if let Some(value) = values.get(i) {
                    println!(
                        "{:<12} {:<20} {:<15} {:<15}",
                        "",
                        "",
                        indicator,
                        value.map(|v| v.to_string()).unwrap_or("N/A".to_string()),
                    );
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PatternCandleRow {
    pub(crate) label: String,
    pub(crate) candle: CandleInput,
}

pub(crate) async fn analyze_candle_patterns(
    candle_specs: Vec<String>,
    code: Option<String>,
    tdx_root: Option<String>,
    market: Option<String>,
    day_file: Option<String>,
    start: Option<String>,
    end: Option<String>,
    period_type: String,
    limit: usize,
    reference: Option<String>,
    previous_close: bool,
) -> Result<()> {
    let rows = load_pattern_candle_rows(
        candle_specs,
        code,
        tdx_root,
        market,
        day_file,
        start,
        end,
        period_type,
        limit,
    )
    .await?;
    let candles: Vec<CandleInput> = rows.iter().map(|row| row.candle).collect();
    let policy = build_reference_policy(reference, previous_close)?;
    let references = sequence_references(&candles, &policy)?;
    let config = PatternConfig {
        epsilon: dec!(0.0001),
    };
    let patterns = recognize_sequence(&candles, &policy, &config)
        .map_err(|err| QuantixError::Other(format!("K线形态识别失败: {:?}", err)))?;

    println!("📈 K线形态识别");
    println!("  数量: {}", candles.len());
    println!(
        "  参考策略: {}",
        if previous_close {
            "前一根收盘价"
        } else {
            "显式参考价"
        }
    );

    for (idx, pattern) in patterns.iter().enumerate() {
        let row = match policy {
            ReferencePricePolicy::Explicit(_) => &rows[idx],
            ReferencePricePolicy::PreviousClose => &rows[idx + 1],
        };
        let candle = &row.candle;

        println!(
            "\n#{} {} O={} H={} L={} C={} P={}",
            idx + 1,
            row.label,
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            references[idx],
        );

        match pattern.canonical_case {
            Some(case_id) => println!("  标准形态: {} {}", case_id.id(), case_id.display_name()),
            None => println!("  标准形态: 扩展形态"),
        }

        println!(
            "  偏向: {}",
            match pattern.bias {
                MarketBias::Bullish => "看多",
                MarketBias::Bearish => "看空",
                MarketBias::Neutral => "看平",
            }
        );
        println!(
            "  扩展结构: {:?} / {:?} / upper_shadow={} / lower_shadow={}",
            pattern.extended.reference_span,
            pattern.extended.body_type,
            pattern.extended.has_upper_shadow,
            pattern.extended.has_lower_shadow
        );
    }

    Ok(())
}

async fn load_pattern_candle_rows(
    candle_specs: Vec<String>,
    code: Option<String>,
    tdx_root: Option<String>,
    market: Option<String>,
    day_file: Option<String>,
    start: Option<String>,
    end: Option<String>,
    period_type: String,
    limit: usize,
) -> Result<Vec<PatternCandleRow>> {
    if !candle_specs.is_empty() {
        return parse_candle_specs(&candle_specs);
    }

    let start_date = start
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok());
    let end_date = end
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok());

    if let Some(day_file) = day_file {
        if period_type != "1d" {
            return Err(QuantixError::Other(
                "day 文件当前只支持 1d 周期".to_string(),
            ));
        }
        return pattern_rows_from_day_file(day_file, start_date, end_date, limit);
    }

    if let Some(tdx_root) = tdx_root {
        if period_type != "1d" {
            return Err(QuantixError::Other(
                "TDX day 根目录当前只支持 1d 周期".to_string(),
            ));
        }
        let code = code.ok_or_else(|| {
            QuantixError::Other("使用 --tdx-root 时必须同时提供 --code".to_string())
        })?;
        let day_file = resolve_tdx_day_file_path(tdx_root, &code, market.as_deref())?;
        return pattern_rows_from_day_file(day_file, start_date, end_date, limit);
    }

    let code = code.ok_or_else(|| {
        QuantixError::Other("缺少 K线输入，请提供 --candle、--day-file 或 --code".to_string())
    })?;

    let client = create_clickhouse_client().await?;
    let klines = client
        .get_kline_data(&code, &period_type, start_date, end_date, Some(limit))
        .await?;

    if klines.is_empty() {
        return Err(QuantixError::Other(format!("未找到 {} 的 K线数据", code)));
    }

    Ok(pattern_rows_from_klines(&klines))
}

fn build_reference_policy(
    reference: Option<String>,
    previous_close: bool,
) -> Result<ReferencePricePolicy> {
    if previous_close {
        return Ok(ReferencePricePolicy::PreviousClose);
    }

    let reference = reference.ok_or_else(|| {
        QuantixError::Other("缺少参考价，请提供 --reference 或 --previous-close".to_string())
    })?;

    let value = Decimal::from_str(&reference)
        .map_err(|e| QuantixError::Other(format!("参考价格式非法: {}", e)))?;

    Ok(ReferencePricePolicy::Explicit(value))
}

fn parse_candle_specs(specs: &[String]) -> Result<Vec<PatternCandleRow>> {
    specs
        .iter()
        .enumerate()
        .map(|(idx, spec)| {
            Ok(PatternCandleRow {
                label: format!("manual-{}", idx + 1),
                candle: parse_candle_spec(spec)?,
            })
        })
        .collect()
}

pub(crate) fn parse_candle_spec(spec: &str) -> Result<CandleInput> {
    let parts: Vec<&str> = spec.split(',').map(str::trim).collect();
    if parts.len() != 4 {
        return Err(QuantixError::Other(format!(
            "K线格式非法: {}，期望 o,h,l,c",
            spec
        )));
    }

    let parse_decimal = |value: &str| {
        Decimal::from_str(value).map_err(|e| QuantixError::Other(format!("价格格式非法: {}", e)))
    };

    Ok(CandleInput {
        open: parse_decimal(parts[0])?,
        high: parse_decimal(parts[1])?,
        low: parse_decimal(parts[2])?,
        close: parse_decimal(parts[3])?,
    })
}

pub(crate) fn sequence_references(
    candles: &[CandleInput],
    policy: &ReferencePricePolicy,
) -> Result<Vec<Decimal>> {
    match policy {
        ReferencePricePolicy::Explicit(value) => Ok(vec![*value; candles.len()]),
        ReferencePricePolicy::PreviousClose => {
            if candles.len() < 2 {
                return Err(QuantixError::Other(
                    "使用 --previous-close 时至少需要两根 K线".to_string(),
                ));
            }

            Ok(candles.windows(2).map(|pair| pair[0].close).collect())
        }
    }
}

pub(crate) fn pattern_rows_from_klines(klines: &[Kline]) -> Vec<PatternCandleRow> {
    klines
        .iter()
        .map(|kline| PatternCandleRow {
            label: kline.date.to_string(),
            candle: CandleInput {
                open: kline.open,
                high: kline.high,
                low: kline.low,
                close: kline.close,
            },
        })
        .collect()
}

pub(crate) fn pattern_rows_from_day_file(
    day_file: impl AsRef<Path>,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    limit: usize,
) -> Result<Vec<PatternCandleRow>> {
    let path = day_file.as_ref();
    let code = infer_tdx_code_from_day_file_path(path)?;
    let mut klines = TdxDayFile::to_klines(code, path, crate::data::models::AdjustType::None)?;

    if let Some(start_date) = start {
        klines.retain(|kline| kline.date >= start_date);
    }
    if let Some(end_date) = end {
        klines.retain(|kline| kline.date <= end_date);
    }
    if limit > 0 && klines.len() > limit {
        klines = klines[klines.len() - limit..].to_vec();
    }

    Ok(pattern_rows_from_klines(&klines))
}

pub(crate) fn infer_tdx_code_from_day_file_path(path: impl AsRef<Path>) -> Result<u32> {
    let stem = path
        .as_ref()
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| QuantixError::Other("无法从 day 文件路径解析股票代码".to_string()))?;

    let digits: String = stem.chars().filter(|ch| ch.is_ascii_digit()).collect();
    if digits.len() != 6 {
        return Err(QuantixError::Other(format!(
            "无法从 day 文件名解析 6 位股票代码: {}",
            path.as_ref().display()
        )));
    }

    digits
        .parse::<u32>()
        .map_err(|e| QuantixError::Other(format!("股票代码解析失败: {}", e)))
}

pub(crate) fn resolve_tdx_day_file_path(
    tdx_root: impl AsRef<Path>,
    code: &str,
    market: Option<&str>,
) -> Result<std::path::PathBuf> {
    let root = tdx_root.as_ref();

    if let Some(market) = market {
        let market = market.to_ascii_lowercase();
        let path = root
            .join("vipdoc")
            .join(&market)
            .join("lday")
            .join(format!("{}{}.day", market, code));
        if path.exists() {
            return Ok(path);
        }
        return Err(QuantixError::Other(format!(
            "未找到指定市场的 day 文件: {}",
            path.display()
        )));
    }

    let matches: Vec<std::path::PathBuf> = ["sh", "sz", "bj", "ds"]
        .iter()
        .map(|market| {
            root.join("vipdoc")
                .join(market)
                .join("lday")
                .join(format!("{}{}.day", market, code))
        })
        .filter(|path| path.exists())
        .collect();

    match matches.as_slice() {
        [single] => Ok(single.clone()),
        [] => Err(QuantixError::Other(format!(
            "未找到 {} 对应的 day 文件，请确认 --tdx-root 或补充 --market",
            code
        ))),
        many => Err(QuantixError::Other(format!(
            "代码 {} 在多个市场目录匹配到多个 day 文件: {}，请补充 --market",
            code,
            many.iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}
