#![allow(clippy::too_many_arguments)]

use crate::analysis::backtest::{BacktestConfig, BacktestEngine};
use crate::cli::command_types::BacktestCommands;
use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::str::FromStr;

use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StoredBacktestReport {
    pub(crate) id: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) strategy: String,
    pub(crate) code: String,
    pub(crate) start: Option<String>,
    pub(crate) end: Option<String>,
    pub(crate) limit: usize,
    pub(crate) config: BacktestConfig,
    pub(crate) short_period: usize,
    pub(crate) long_period: usize,
    pub(crate) report: crate::analysis::performance::PerformanceReport,
    pub(crate) final_equity: Decimal,
    pub(crate) trades: usize,
}

pub(crate) async fn run_backtest_command(cmd: BacktestCommands) -> Result<()> {
    match cmd {
        BacktestCommands::Run {
            strategy,
            code,
            start,
            end,
            capital,
            commission_rate,
            slippage_bps,
            short_period,
            long_period,
            max_positions,
            max_position_ratio,
            risk_free_rate,
            limit,
        } => {
            let report = execute_backtest_run(
                strategy,
                code,
                start,
                end,
                capital,
                commission_rate,
                slippage_bps,
                short_period,
                long_period,
                max_positions,
                max_position_ratio,
                risk_free_rate,
                limit,
            )
            .await?;
            print_backtest_report_summary(&report);
        }
        BacktestCommands::Walkforward {
            code,
            start,
            end,
            short_start,
            short_end,
            short_step,
            long_start,
            long_end,
            long_step,
            train_window,
            test_window,
            step,
            capital,
            commission_rate,
            position_size,
            min_stability,
        } => {
            execute_fin_walkforward(
                code,
                start,
                end,
                short_start,
                short_end,
                short_step,
                long_start,
                long_end,
                long_step,
                train_window,
                test_window,
                step,
                capital,
                commission_rate,
                position_size,
                min_stability,
            )
            .await?;
        }
        BacktestCommands::Report { id } => {
            let report = load_backtest_report(&id)?;
            print_backtest_report_summary(&report);
        }
        BacktestCommands::List => {
            print_backtest_report_list(&list_backtest_reports()?);
        }
        BacktestCommands::Compare { ids } => {
            if ids.len() < 2 {
                return Err(QuantixError::Other(
                    "backtest compare 至少需要两个 --id".to_string(),
                ));
            }
            let reports: Result<Vec<_>> = ids.iter().map(|id| load_backtest_report(id)).collect();
            print_backtest_comparison(&reports?);
        }
    }

    Ok(())
}

/// 研究侧 walk-forward 回测：ma_cross 双参数网格 + 滚动训练/测试窗口（fin 引擎）。
#[allow(clippy::too_many_arguments)]
pub(crate) async fn execute_fin_walkforward(
    code: String,
    start: Option<String>,
    end: Option<String>,
    short_start: usize,
    short_end: usize,
    short_step: usize,
    long_start: usize,
    long_end: usize,
    long_step: usize,
    train_window: usize,
    test_window: usize,
    step: usize,
    capital: String,
    commission_rate: String,
    position_size: String,
    min_stability: f64,
) -> Result<()> {
    let parse_decimal = |label: &str, raw: &str| -> Result<Decimal> {
        Decimal::from_str(raw)
            .map_err(|e| QuantixError::DataParse(format!("{label} 无效: {raw} ({e})")))
    };
    let capital = parse_decimal("capital", &capital)?;
    let commission_rate = parse_decimal("commission_rate", &commission_rate)?;
    let position_size = parse_decimal("position_size", &position_size)?;
    let start = parse_optional_yyyymmdd(start.as_deref())?;
    let end = parse_optional_yyyymmdd(end.as_deref())?;

    // 头部先于取数（T9/T10 惯例；空数据时头仍可见）
    println!("🔬 walk-forward 回测（fin 引擎，ma_cross）");
    println!("  代码: {}", code);
    println!(
        "  网格: short({}..={}#{}) long({}..={}#{}) 窗口: train({}) test({}) step({})",
        short_start,
        short_end,
        short_step,
        long_start,
        long_end,
        long_step,
        train_window,
        test_window,
        step
    );

    // 全量升序取回（T10 教训：数据源 limit 语义为升序前 l 条即最旧 N 根；区间截断走 --start/--end）
    let mut klines = get_kline_for_analysis(&code, start, end, None).await?;

    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }

    // walk-forward 契约：序列按时间升序、首元素为窗口起点
    klines.sort_by_key(|k| k.date);
    if klines.len() < train_window + test_window {
        return Err(QuantixError::Config(format!(
            "数据量不足: {} 根 < train({}) + test({})",
            klines.len(),
            train_window,
            test_window
        )));
    }

    let params = crate::fin::backtest::WalkForwardParams {
        short_range: (short_start, short_end, short_step),
        long_range: (long_start, long_end, long_step),
        train_window,
        test_window,
        step,
    };
    let report = crate::fin::backtest::walk_forward_ma_cross(
        &klines,
        &params,
        capital,
        commission_rate,
        position_size,
    )?;

    println!(
        "\n📊 逐窗口结果（{} 根日线，{} 窗口）:",
        klines.len(),
        report.periods.len()
    );
    println!("{}", "-".repeat(76));
    println!(
        "{:<6} {:<12} {:<12} {:<16} {:<10} {:<10}",
        "窗口", "train 区间", "test 区间", "best(short,long)", "IS sharpe", "OOS sharpe"
    );
    for (i, p) in report.periods.iter().enumerate() {
        let s = p.best_params.get("short_period").copied().unwrap_or(0.0);
        let l = p.best_params.get("long_period").copied().unwrap_or(0.0);
        println!(
            "{:<6} {:<12} {:<12} {:<16} {:<10} {:<10}",
            i + 1,
            format!("{}-{}", p.train_start, p.train_end),
            format!("{}-{}", p.test_start, p.test_end),
            format!("{s}/{l}"),
            format!("{:.3}", p.in_sample_sharpe),
            format!("{:.3}", p.out_of_sample_sharpe),
        );
    }
    println!("{}", "-".repeat(76));
    println!("aggregate_sharpe:     {:.3}", report.aggregate_sharpe);
    println!(
        "stability_score:      {:.3}（OOS>0 占比）",
        report.stability_score
    );
    println!("mean_oos_return:      {:.5}", report.mean_oos_return);
    // worst_oos_drawdown 为上游聚合层预算的全局值（全部窗口 OOS 回撤最大者，小数口径）；
    // 逐期表不单列 OOS 回撤——WfPeriod.oos_result 为嵌套 fin 类型，门面已收敛至聚合层
    println!("worst_oos_drawdown:   {:.5}", report.worst_oos_drawdown);
    println!(
        "is_robust:            {}（stability ≥ {} 且 OOS Sharpe 均值 > 0）",
        report.is_robust(min_stability),
        min_stability
    );
    Ok(())
}

pub(crate) async fn execute_backtest_run(
    strategy: String,
    code: String,
    start: Option<String>,
    end: Option<String>,
    capital: String,
    commission_rate: String,
    slippage_bps: u32,
    short_period: usize,
    long_period: usize,
    max_positions: usize,
    max_position_ratio: String,
    risk_free_rate: String,
    limit: usize,
) -> Result<StoredBacktestReport> {
    if strategy != "ma_cross" {
        return Err(QuantixError::Unsupported(format!(
            "当前仅支持 ma_cross，收到: {strategy}"
        )));
    }
    if short_period == 0 || long_period == 0 || short_period >= long_period {
        return Err(QuantixError::Config(format!(
            "无效均线参数: short={short_period}, long={long_period}"
        )));
    }

    let start_date = parse_optional_yyyymmdd(start.as_deref())?;
    let end_date = parse_optional_yyyymmdd(end.as_deref())?;
    let config = BacktestConfig {
        initial_capital: parse_decimal_arg("capital", &capital)?,
        commission_rate: parse_decimal_arg("commission_rate", &commission_rate)?,
        slippage_bps,
        max_positions,
        max_position_ratio: parse_decimal_arg("max_position_ratio", &max_position_ratio)?,
        risk_free_rate: parse_decimal_arg("risk_free_rate", &risk_free_rate)?,
    };

    let klines = get_kline_for_analysis(&code, start_date, end_date, Some(limit)).await?;

    if klines.len() < long_period {
        return Err(QuantixError::Other(format!(
            "数据不足，至少需要 {long_period} 条 K 线，当前 {}",
            klines.len()
        )));
    }

    let kline_data: Vec<Kline> = klines
        .into_iter()
        .map(|k| Kline {
            code: code.clone(),
            date: k.date,
            open: k.open,
            high: k.high,
            low: k.low,
            close: k.close,
            volume: k.volume,
            amount: k.amount,
            adjust_type: crate::data::models::AdjustType::None,
        })
        .collect();

    let mut data_map = HashMap::new();
    data_map.insert(code.clone(), kline_data);

    let mut engine = BacktestEngine::new(config.clone());
    let mut strategy_impl =
        crate::strategy::ma_cross::MACrossStrategy::new(short_period, long_period);
    let result = engine
        .run(&mut strategy_impl, &data_map)
        .await
        .map_err(|err| QuantixError::Other(format!("回测失败: {err}")))?;

    let id = format!(
        "{}-{}-{}",
        strategy,
        code,
        Utc::now().format("%Y%m%d%H%M%S")
    );
    let stored = StoredBacktestReport {
        id: id.clone(),
        created_at: Utc::now(),
        strategy,
        code,
        start,
        end,
        limit,
        config,
        short_period,
        long_period,
        report: result.report,
        final_equity: result.final_equity,
        trades: result.trades.len(),
    };

    save_backtest_report(&stored)?;
    Ok(stored)
}

pub(crate) fn show_backtest_report(id: String) -> Result<()> {
    let report = load_backtest_report(&id)?;
    print_backtest_report_summary(&report);
    Ok(())
}

pub(crate) fn read_backtest_report(id: &str) -> Result<StoredBacktestReport> {
    load_backtest_report(id)
}

pub(crate) fn read_backtest_reports() -> Result<Vec<StoredBacktestReport>> {
    list_backtest_reports()
}

fn parse_decimal_arg(name: &str, raw: &str) -> Result<Decimal> {
    Decimal::from_str(raw)
        .map_err(|err| QuantixError::Config(format!("{name} 不是合法数字: {err}")))
}

fn parse_optional_yyyymmdd(raw: Option<&str>) -> Result<Option<NaiveDate>> {
    raw.map(|value| {
        NaiveDate::parse_from_str(value, "%Y%m%d")
            .map_err(|err| QuantixError::Config(format!("日期格式应为 YYYYMMDD: {err}")))
    })
    .transpose()
}

fn backtest_reports_dir() -> std::path::PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        return std::path::PathBuf::from(home)
            .join(".quantix")
            .join("backtest")
            .join("reports");
    }

    std::path::PathBuf::from(".quantix")
        .join("backtest")
        .join("reports")
}

fn save_backtest_report(report: &StoredBacktestReport) -> Result<()> {
    let dir = backtest_reports_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.json", report.id));
    std::fs::write(path, serde_json::to_string_pretty(report)?)?;
    Ok(())
}

fn load_backtest_report(id: &str) -> Result<StoredBacktestReport> {
    let path = backtest_reports_dir().join(format!("{}.json", id));
    if !path.exists() {
        return Err(QuantixError::Other(format!(
            "未找到回测报告: {}",
            path.display()
        )));
    }
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn list_backtest_reports() -> Result<Vec<StoredBacktestReport>> {
    let dir = backtest_reports_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut reports = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let content = std::fs::read_to_string(&path)?;
        let report: StoredBacktestReport = serde_json::from_str(&content)?;
        reports.push(report);
    }

    reports.sort_by_key(|report| std::cmp::Reverse(report.created_at));
    Ok(reports)
}

fn print_backtest_report_summary(report: &StoredBacktestReport) {
    println!("📊 回测报告");
    println!("  ID: {}", report.id);
    println!("  策略: {}", report.strategy);
    println!("  股票: {}", report.code);
    println!(
        "  区间: {} -> {}",
        report.start.as_deref().unwrap_or("最早"),
        report.end.as_deref().unwrap_or("最新")
    );
    println!("  创建时间: {}", report.created_at);
    println!(
        "  参数: short={} long={} capital={} commission_rate={} slippage_bps={}",
        report.short_period,
        report.long_period,
        report.config.initial_capital,
        report.config.commission_rate,
        report.config.slippage_bps
    );
    println!();
    println!("  总收益率: {:.2}%", report.report.total_return * dec!(100));
    println!(
        "  年化收益率: {:.2}%",
        report.report.annual_return * dec!(100)
    );
    println!("  最大回撤: {:.2}%", report.report.max_drawdown * dec!(100));
    println!("  夏普比率: {:.2}", report.report.sharpe_ratio);
    println!("  索提诺比率: {:.2}", report.report.sortino_ratio);
    println!("  胜率: {:.2}%", report.report.win_rate);
    println!("  交易次数: {}", report.report.total_trades);
    println!("  最终权益: {}", report.final_equity);
}

fn print_backtest_report_list(reports: &[StoredBacktestReport]) {
    if reports.is_empty() {
        println!("暂无已保存的回测报告");
        println!("报告目录: {}", backtest_reports_dir().display());
        return;
    }

    println!("已保存回测报告");
    println!("报告目录: {}", backtest_reports_dir().display());
    println!(
        "{:<28} {:<10} {:<12} {:<12} {:<20} {:>10}",
        "ID", "CODE", "START", "END", "CREATED_AT", "RETURN%"
    );
    println!("{}", "-".repeat(102));
    for report in reports {
        println!(
            "{:<28} {:<10} {:<12} {:<12} {:<20} {:>10.2}",
            report.id,
            report.code,
            report.start.as_deref().unwrap_or("最早"),
            report.end.as_deref().unwrap_or("最新"),
            report.created_at.format("%Y-%m-%d %H:%M:%S"),
            (report.report.total_return * dec!(100))
                .to_f64()
                .unwrap_or_default()
        );
    }
}

fn print_backtest_comparison(reports: &[StoredBacktestReport]) {
    println!(
        "{:<28} {:<10} {:>10} {:>10} {:>10} {:>10}",
        "ID", "CODE", "RETURN%", "MDD%", "SHARPE", "TRADES"
    );
    println!("{}", "-".repeat(86));
    for report in reports {
        println!(
            "{:<28} {:<10} {:>10.2} {:>10.2} {:>10.2} {:>10}",
            report.id,
            report.code,
            (report.report.total_return * dec!(100))
                .to_f64()
                .unwrap_or_default(),
            (report.report.max_drawdown * dec!(100))
                .to_f64()
                .unwrap_or_default(),
            report.report.sharpe_ratio.to_f64().unwrap_or_default(),
            report.report.total_trades
        );
    }
}
