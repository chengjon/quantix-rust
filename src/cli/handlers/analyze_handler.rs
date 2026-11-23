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

    let klines = get_kline_for_analysis(&code, None, None, Some(1000)).await?;

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

/// fin 引擎支持的指标名 → 默认周期（与 `fin::indicators::resolve` 名单逐一对齐）。
/// 周期口径：sma/ema/rsi 与 native 缺省周期一致；boll/atr/stoch 为 fin 侧标准缺省（20/14/9）。
const FIN_INDICATOR_PERIODS: &[(&str, usize)] = &[
    ("sma", 5),
    ("ema", 5),
    ("rsi", 14),
    ("atr", 14),
    ("boll", 20),
    ("stoch", 9),
];

/// fin 引擎 period 上限守卫阈值（T3 quality review #5）：上游指标层
/// `VecDeque::with_capacity(period + 1)`，超大 period 在 debug 构建下有容量溢出 panic 风险。
const MAX_FIN_INDICATOR_PERIOD: usize = 500;

/// period 合法性守卫：0 拒绝（上游流式指标语义非法）、超上限拒绝（with_capacity 雷区）。
fn validate_fin_indicator_period(name: &str, period: usize) -> Result<()> {
    if period == 0 || period > MAX_FIN_INDICATOR_PERIOD {
        return Err(QuantixError::Config(format!(
            "fin 引擎指标 {name} 周期非法: {period}（须 1..={MAX_FIN_INDICATOR_PERIOD}）"
        )));
    }
    Ok(())
}

/// fin 引擎指标查询：版式与 native `calculate_indicators` 完全一致
/// （💹 头部三行 + 表头 + 每根 Kline 一行日期/收盘价 + 每指标一行且前两列留空）。
pub(crate) async fn calculate_indicators_fin(code: String, indicators_str: String) -> Result<()> {
    // 头部三行先于取数（逐字节镜像 native calculate_indicators 顺序；空数据时头仍可见）
    println!("💹 计算技术指标（引擎=fin）");
    println!("  代码: {}", code);
    println!("  指标: {}", indicators_str);

    let klines = get_kline_for_analysis(&code, None, None, Some(1000)).await?;

    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }

    let names: Vec<&str> = indicators_str
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if names.is_empty() {
        return Err(QuantixError::Config("指标列表为空".into()));
    }

    // fail-fast：全部指标名的周期解析 + 上限守卫先于表格输出，
    // 未知名/非法周期不会吐出半张结果表（错误前置且一次性报全）。
    let mut resolved: Vec<(&str, usize)> = Vec::with_capacity(names.len());
    for name in &names {
        let period = FIN_INDICATOR_PERIODS
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, p)| *p)
            .ok_or_else(|| {
                let supported = FIN_INDICATOR_PERIODS
                    .iter()
                    .map(|(n, _)| *n)
                    .collect::<Vec<_>>()
                    .join("/");
                QuantixError::Unsupported(format!(
                    "fin 引擎暂不支持指标: {name}（支持: {supported}）"
                ))
            })?;
        validate_fin_indicator_period(name, period)?;
        resolved.push((name, period));
    }

    println!("\n📊 计算结果:");
    println!(
        "{:<12} {:<20} {:<15} {:<15}",
        "日期", "收盘价", "指标", "值"
    );
    println!("{}", "-".repeat(65));

    // 每指标全序列只算一次（quality review #3：逐 Kline 行内重复 resolve 会把
    // 全序列计算放大 指标数×take(20) 倍）；渲染仅按索引取值。
    let series_list = resolved
        .iter()
        .map(|(name, period)| crate::fin::indicators::resolve(name, &klines, *period))
        .collect::<Result<Vec<_>>>()?;

    for (i, kline) in klines.iter().enumerate().take(20) {
        println!("{:<12} {:<20.2}", kline.date, kline.close,);

        for series in &series_list {
            let value = match series.values.get(i).and_then(|v| *v) {
                Some(v) => v.to_string(),
                None => "N/A".to_string(),
            };
            println!("{:<12} {:<20} {:<15} {:<15}", "", "", series.name, value,);
        }
    }

    // 不变式（quality review #4）：klines 非空（上方守卫）+ series_list 非空
    // （names 非空且逐名解析成功）⇒ 首个 kline 行内至少渲染一行，无空表路径。
    Ok(())
}

/// 回撤风险报告：以收盘价序列为净值曲线（研究口径），fin DrawdownTracker 统计。
pub(crate) async fn analyze_drawdown(code: String, limit: usize) -> Result<()> {
    // 头部先于取数（T9 review #8 顺序惯例；空数据时头仍可见）
    println!("📉 回撤风险报告（净值=收盘价研究口径）");
    println!("  代码: {}", code);
    println!("  统计窗口: 最近 {} 根", limit);

    let mut klines = get_kline_for_analysis(&code, None, None, Some(limit)).await?;

    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }

    // drawdown_report 契约：序列按时间升序、首元素为初始净值（乱序输入会得出错误回撤）
    klines.sort_by_key(|k| k.date);
    let equity: Vec<Decimal> = klines.iter().map(|k| k.close).collect();
    let report = crate::fin::risk::drawdown_report(&equity)?;

    // 回撤口径 0-100（百分数），与 fin BacktestResult.max_drawdown 的 0-1 小数口径不同
    println!("\n📊 回撤统计（{} 根日线，回撤口径 0-100）:", equity.len());
    println!("{}", "-".repeat(48));
    println!("当前回撤:   {} %", report.current_pct);
    println!("最大回撤:   {} %", report.worst_pct);
    println!("净值峰值:   {}", report.peak);
    println!("当前净值:   {}", report.current_equity);
    println!("回撤持续:   {} 根", report.duration_bars);
    println!(
        "处于回撤中: {}",
        if report.in_drawdown { "是" } else { "否" }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// fin 门面 resolve 未知名 → Unsupported（计划 Task 9 Step 5 冒烟兜底；
    /// handler 网络依赖较重，resolve 层单测为主断言）。
    #[test]
    fn fin_indicators_reject_unknown_name() {
        let err = crate::fin::indicators::resolve("kdj", &[], 9).unwrap_err();
        assert!(matches!(err, QuantixError::Unsupported(_)));
    }

    /// 净值代理契约钉（输入侧）：drawdown_report 对乱序与升序输入给出不同报告——
    /// handler 因此必须在调用前 `sort_by_key(date)`（Task 10 计划 Step 4）。
    #[test]
    fn drawdown_report_prefers_sorted_series() {
        let reversed = vec![Decimal::from(99), Decimal::from(110), Decimal::from(100)];
        let sorted: Vec<_> = {
            let mut v = reversed.clone();
            v.sort();
            v
        };
        let unsorted = crate::fin::risk::drawdown_report(&reversed).unwrap();
        let sorted_r = crate::fin::risk::drawdown_report(&sorted).unwrap();
        // 乱序与升序输入报告不同——handler 必须 sort（本测试锁定该契约的输入侧）
        assert_ne!(unsorted.worst_pct, sorted_r.worst_pct);
    }

    /// period 上限守卫（T3 quality review #5）：超大 period → Config 提示，
    /// 绝不到上游指标层 `VecDeque::with_capacity(period + 1)` 的容量溢出雷区。
    #[test]
    fn fin_period_guard_rejects_oversized_period() {
        let err = validate_fin_indicator_period("sma", MAX_FIN_INDICATOR_PERIOD + 1).unwrap_err();
        assert!(matches!(err, QuantixError::Config(_)));
    }

    /// period=0 同样拒绝（fin 门面流式指标 period=0 语义非法，上游已拒）。
    #[test]
    fn fin_period_guard_rejects_zero_period() {
        let err = validate_fin_indicator_period("sma", 0).unwrap_err();
        assert!(matches!(err, QuantixError::Config(_)));
    }

    /// 内置周期表全部通过守卫；上限边界值（=MAX）恰好放行。
    #[test]
    fn fin_period_guard_accepts_table_periods_and_max_boundary() {
        for (name, period) in FIN_INDICATOR_PERIODS {
            assert!(
                validate_fin_indicator_period(name, *period).is_ok(),
                "内置周期应为合法: {name}={period}"
            );
        }
        assert!(validate_fin_indicator_period("sma", MAX_FIN_INDICATOR_PERIOD).is_ok());
    }
}
