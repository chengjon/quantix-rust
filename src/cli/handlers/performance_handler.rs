use super::*;

use crate::core::{CliRuntime, QuantixError, Result};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;

pub(crate) fn run_performance_command(cmd: PerformanceCommands) -> Result<()> {
    match cmd {
        PerformanceCommands::Report { id } => {
            let report = read_backtest_report(&id)?;
            print_performance_report(&report);
        }
        PerformanceCommands::List => {
            let reports = read_backtest_reports()?;
            print_performance_list(&reports);
        }
        PerformanceCommands::Compare { ids } => {
            if ids.len() < 2 {
                return Err(QuantixError::Other(
                    "performance compare 至少需要两个 --id".to_string(),
                ));
            }

            let reports: Result<Vec<_>> = ids.iter().map(|id| read_backtest_report(id)).collect();
            print_performance_compare(&reports?);
        }
    }

    Ok(())
}

fn print_performance_report(report: &StoredBacktestReport) {
    let metrics = &report.report;
    println!("📈 绩效报告");
    println!("  ID: {}", report.id);
    println!("  策略: {}", report.strategy);
    println!("  股票: {}", report.code);
    println!("  创建时间: {}", report.created_at);
    println!();
    println!("  总收益率: {:.2}%", metrics.total_return * dec!(100));
    println!("  年化收益率: {:.2}%", metrics.annual_return * dec!(100));
    println!("  最大回撤: {:.2}%", metrics.max_drawdown * dec!(100));
    println!("  夏普比率: {:.2}", metrics.sharpe_ratio);
    println!("  索提诺比率: {:.2}", metrics.sortino_ratio);
    println!("  卡玛比率: {:.2}", metrics.calmar_ratio);
    println!("  胜率: {:.2}%", metrics.win_rate);
    println!("  盈亏比: {:.2}", metrics.profit_loss_ratio);
    println!("  总交易次数: {}", metrics.total_trades);
    println!("  盈利交易: {}", metrics.win_trades);
    println!("  亏损交易: {}", metrics.loss_trades);
    println!("  平均盈利: {}", metrics.avg_win);
    println!("  平均亏损: {}", metrics.avg_loss);
    println!("  最大盈利: {}", metrics.max_win);
    println!("  最大亏损: {}", metrics.max_loss);
    println!("  最大连赢: {}", metrics.max_consecutive_wins);
    println!("  最大连亏: {}", metrics.max_consecutive_losses);
    println!("  总手续费: {}", metrics.total_commission);
    println!("  最终权益: {}", report.final_equity);
}

fn print_performance_list(reports: &[StoredBacktestReport]) {
    if reports.is_empty() {
        println!("暂无可用于绩效分析的回测报告");
        return;
    }

    println!(
        "{:<28} {:<10} {:>10} {:>10} {:>10} {:>10}",
        "ID", "CODE", "RETURN%", "SHARPE", "MDD%", "TRADES"
    );
    println!("{}", "-".repeat(86));
    for report in reports {
        let metrics = &report.report;
        println!(
            "{:<28} {:<10} {:>10.2} {:>10.2} {:>10.2} {:>10}",
            report.id,
            report.code,
            (metrics.total_return * dec!(100))
                .to_f64()
                .unwrap_or_default(),
            metrics.sharpe_ratio.to_f64().unwrap_or_default(),
            (metrics.max_drawdown * dec!(100))
                .to_f64()
                .unwrap_or_default(),
            metrics.total_trades
        );
    }
}

fn print_performance_compare(reports: &[StoredBacktestReport]) {
    println!(
        "{:<28} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "ID", "RETURN%", "ANN%", "MDD%", "SHARPE", "CALMAR"
    );
    println!("{}", "-".repeat(92));
    for report in reports {
        let metrics = &report.report;
        println!(
            "{:<28} {:>10.2} {:>10.2} {:>10.2} {:>10.2} {:>10.2}",
            report.id,
            (metrics.total_return * dec!(100))
                .to_f64()
                .unwrap_or_default(),
            (metrics.annual_return * dec!(100))
                .to_f64()
                .unwrap_or_default(),
            (metrics.max_drawdown * dec!(100))
                .to_f64()
                .unwrap_or_default(),
            metrics.sharpe_ratio.to_f64().unwrap_or_default(),
            metrics.calmar_ratio.to_f64().unwrap_or_default(),
        );
    }
}
