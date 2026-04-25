use super::*;
use crate::cli::handlers::market_handler::MarketStrengthStockRankingOutput;
use std::fmt::Write as _;

pub(super) fn print_market_foundation_summary(summary: &MarketFoundationSummary) {
    print!("{}", render_market_foundation_summary(summary));
}

fn render_market_foundation_summary(summary: &MarketFoundationSummary) -> String {
    let mut output = String::new();

    writeln!(&mut output, "== 市场基础数据 ==").unwrap();
    writeln!(&mut output, "A股总数: {}", summary.total_stocks).unwrap();
    writeln!(&mut output, "已匹配行业: {}", summary.classified_stocks).unwrap();
    writeln!(&mut output, "未匹配行业: {}", summary.unclassified_stocks).unwrap();
    writeln!(&mut output, "行业数: {}", summary.sector_count).unwrap();

    if !summary.top_sectors.is_empty() {
        writeln!(&mut output).unwrap();
        writeln!(&mut output, "行业覆盖 Top10:").unwrap();
        writeln!(&mut output, "{:<4} {:<16} 成分股数", "排名", "行业").unwrap();
        writeln!(&mut output, "{}", "-".repeat(40)).unwrap();
        for (idx, row) in summary.top_sectors.iter().enumerate() {
            writeln!(
                &mut output,
                "{:<4} {:<16} {}",
                idx + 1,
                row.industry_name,
                row.stock_count
            )
            .unwrap();
        }
    }

    output
}

pub(super) fn print_market_board_rows(rows: &[BoardRankRow]) {
    print!("{}", render_market_board_rows(rows));
}

fn render_market_board_rows(rows: &[BoardRankRow]) -> String {
    let mut output = String::new();

    if rows.is_empty() {
        writeln!(&mut output, "📭 没有可展示的板块数据").unwrap();
        return output;
    }

    writeln!(&mut output, "{:<8} {:<12} {:<16} 涨跌幅", "排名", "代码", "板块").unwrap();
    writeln!(&mut output, "{}", "-".repeat(56)).unwrap();

    for row in rows {
        writeln!(
            &mut output,
            "{:<8} {:<12} {:<16} {:.2}%",
            row.rank, row.board_code, row.board_name, row.change_pct
        )
        .unwrap();
    }

    output
}

pub(super) fn print_north_flow_snapshot(snapshot: Option<&NorthFlowSnapshot>) {
    let Some(snapshot) = snapshot else {
        println!("📭 没有可展示的北向资金数据");
        return;
    };

    println!("日期: {}", snapshot.trade_date);
    println!("沪股通: {:.2}", snapshot.sh_amount);
    println!("深股通: {:.2}", snapshot.sz_amount);
    println!("合计: {:.2}", snapshot.total_amount);
    println!("余额: {:.2}", snapshot.balance);
}

pub(super) fn print_market_sentiment_snapshot(snapshot: Option<&MarketSentimentSnapshot>) {
    let Some(snapshot) = snapshot else {
        println!("📭 没有可展示的市场情绪数据");
        return;
    };

    println!("日期: {}", snapshot.trade_date);
    println!("上涨: {}", snapshot.up_count);
    println!("下跌: {}", snapshot.down_count);
    println!("涨停: {}", snapshot.limit_up_count);
    println!("跌停: {}", snapshot.limit_down_count);
    println!("封板率: {:.2}", snapshot.seal_rate);
    println!("炸板率: {:.2}", snapshot.break_rate);
    println!("连板股: {}", snapshot.consecutive_board_count);
}

pub(super) fn print_market_leader_rows(rows: &[LeaderRow]) {
    if rows.is_empty() {
        println!("📭 没有可展示的龙头股数据");
        return;
    }

    println!(
        "{:<10} {:<12} {:<12} {:<12} 涨跌幅",
        "代码", "名称", "行业", "概念"
    );
    println!("{}", "-".repeat(72));

    for row in rows {
        println!(
            "{:<10} {:<12} {:<12} {:<12} {:.2}%",
            row.code,
            row.name,
            row.sector_name.as_deref().unwrap_or("-"),
            row.concept_name.as_deref().unwrap_or("-"),
            row.change_pct
        );
    }
}

pub(super) fn print_market_overview(overview: &MarketOverview) {
    println!("== 市场概览 ==");
    println!("行业板块: {}", overview.top_sectors.len());
    println!("概念板块: {}", overview.top_concepts.len());

    match overview.north_flow.as_ref() {
        Some(snapshot) => println!("北向资金: {:.2}", snapshot.total_amount),
        None => println!("北向资金: -"),
    }

    match overview.sentiment.as_ref() {
        Some(snapshot) => println!("涨停数: {}", snapshot.limit_up_count),
        None => println!("涨停数: -"),
    }

    if !overview.top_sectors.is_empty() {
        println!();
        println!("Top 行业:");
        print_market_board_rows(&overview.top_sectors);
    }

    if !overview.top_concepts.is_empty() {
        println!();
        println!("Top 概念:");
        print_market_board_rows(&overview.top_concepts);
    }
}

pub(super) fn print_market_strength_report(report: &MarketStrengthReport) {
    print!("{}", render_market_strength_report(report));
}

fn render_market_strength_report(report: &MarketStrengthReport) -> String {
    let mut output = String::new();

    writeln!(&mut output, "== 强弱板块分析 ==").unwrap();
    writeln!(
        &mut output,
        "基础数据: A股={} 行业覆盖={} 未覆盖={}",
        report.foundation.total_stocks,
        report.foundation.classified_stocks,
        report.foundation.unclassified_stocks
    )
    .unwrap();
    writeln!(&mut output, "强势板块候选股数: {}", report.candidate_stock_count).unwrap();
    writeln!(
        &mut output,
        "基本面覆盖: 市值={}/{} 利润={}/{}",
        report.market_cap_coverage_count,
        report.candidate_stock_count,
        report.profit_coverage_count,
        report.candidate_stock_count
    )
    .unwrap();
    if report.valuation_error_count > 0 || report.earnings_error_count > 0 {
        writeln!(
            &mut output,
            "基本面抓取异常: 市值请求失败={} 利润请求失败={}",
            report.valuation_error_count, report.earnings_error_count
        )
        .unwrap();
    }

    writeln!(&mut output).unwrap();
    writeln!(&mut output, "强势板块:").unwrap();
    output.push_str(&render_market_board_rows(&report.strong_sectors));

    writeln!(&mut output).unwrap();
    writeln!(&mut output, "弱势板块:").unwrap();
    output.push_str(&render_market_board_rows(&report.weak_sectors));

    writeln!(&mut output).unwrap();
    writeln!(
        &mut output,
        "强势板块个股 Top{} 总市值:",
        report.top_by_market_cap.len()
    )
    .unwrap();
    output.push_str(&render_strong_sector_stock_rows(
        &report.top_by_market_cap,
        "总市值(亿)",
        true,
    ));

    writeln!(&mut output).unwrap();
    writeln!(
        &mut output,
        "强势板块个股 Top{} 推算净利润:",
        report.top_by_profit.len()
    )
    .unwrap();
    output.push_str(&render_strong_sector_stock_rows(
        &report.top_by_profit,
        "推算净利润(亿)",
        false,
    ));

    output
}

pub(super) fn print_market_strength_stock_ranking(ranking: &MarketStrengthStockRankingOutput) {
    print!("{}", render_market_strength_stock_ranking(ranking));
}

fn render_market_strength_stock_ranking(ranking: &MarketStrengthStockRankingOutput) -> String {
    let metric_label = match ranking.metric {
        StrengthStockMetric::MarketCap => "总市值(亿)",
        StrengthStockMetric::Profit => "上一会计周期净利润(亿)",
    };
    let metric_name = match ranking.metric {
        StrengthStockMetric::MarketCap => "总市值",
        StrengthStockMetric::Profit => "上一会计周期净利润",
    };
    let use_market_cap = matches!(ranking.metric, StrengthStockMetric::MarketCap);
    let mut output = String::new();

    writeln!(&mut output, "== 强势板块个股排行 ==").unwrap();
    writeln!(&mut output, "强势板块范围: Top{}", ranking.strong_top).unwrap();
    if let Some(sector_name) = ranking.sector_filter.as_deref() {
        writeln!(&mut output, "行业过滤: {}", sector_name).unwrap();
    }
    writeln!(&mut output, "候选股数: {}", ranking.candidate_stock_count).unwrap();
    writeln!(
        &mut output,
        "{}覆盖: {}/{}",
        metric_name, ranking.covered_count, ranking.candidate_stock_count
    )
    .unwrap();
    writeln!(&mut output).unwrap();
    writeln!(&mut output, "按{}从大到小 Top{}:", metric_name, ranking.rows.len()).unwrap();
    output.push_str(&render_strong_sector_stock_rows(
        &ranking.rows,
        metric_label,
        use_market_cap,
    ));

    output
}

fn render_strong_sector_stock_rows(
    rows: &[StrongSectorStockRow],
    metric_label: &str,
    use_market_cap: bool,
) -> String {
    let mut output = String::new();

    if rows.is_empty() {
        writeln!(&mut output, "📭 没有可展示的个股数据").unwrap();
        return output;
    }

    writeln!(
        &mut output,
        "{:<4} {:<10} {:<12} {:<12} {:<12} {}",
        "排名", "行业", "代码", "名称", "现价", metric_label
    )
    .unwrap();
    writeln!(&mut output, "{}", "-".repeat(84)).unwrap();

    for (idx, row) in rows.iter().enumerate() {
        let metric = if use_market_cap {
            row.market_cap.clone()
        } else {
            row.latest_report_profit.clone()
        };
        writeln!(
            &mut output,
            "{:<4} {:<10} {:<12} {:<12} {:<12.2} {}",
            idx + 1,
            row.sector_name,
            row.code,
            row.name,
            row.latest_price,
            format_decimal(&metric)
        )
        .unwrap();
    }

    output
}

fn format_decimal(value: &Option<Decimal>) -> String {
    match value {
        Some(value) => format!("{:.2}", value),
        None => "-".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::handlers::market_handler::MarketStrengthStockRankingOutput;
    use crate::market::{BoardRankRow, BoardType, MarketFoundationSummary, MarketStrengthReport};
    use crate::market::SectorCoverageRow;
    use rust_decimal::Decimal;

    #[test]
    fn render_market_foundation_summary_shows_core_counts_without_sector_table() {
        let summary = MarketFoundationSummary {
            total_stocks: 100,
            classified_stocks: 0,
            unclassified_stocks: 100,
            sector_count: 0,
            top_sectors: vec![],
        };

        let output = render_market_foundation_summary(&summary);

        assert!(output.contains("== 市场基础数据 =="));
        assert!(output.contains("A股总数: 100"));
        assert!(output.contains("已匹配行业: 0"));
        assert!(output.contains("未匹配行业: 100"));
        assert!(output.contains("行业数: 0"));
        assert!(!output.contains("行业覆盖 Top10:"));
    }

    #[test]
    fn render_market_foundation_summary_includes_sector_coverage_table() {
        let summary = MarketFoundationSummary {
            total_stocks: 5300,
            classified_stocks: 5200,
            unclassified_stocks: 100,
            sector_count: 31,
            top_sectors: vec![
                SectorCoverageRow {
                    industry_name: "银行".to_string(),
                    stock_count: 42,
                },
                SectorCoverageRow {
                    industry_name: "计算机".to_string(),
                    stock_count: 38,
                },
            ],
        };

        let output = render_market_foundation_summary(&summary);

        assert!(output.contains("行业覆盖 Top10:"));
        assert!(output.contains("排名"));
        assert!(output.contains("行业"));
        assert!(output.contains("成分股数"));
        assert!(output.contains("1    银行"));
        assert!(output.contains("42"));
        assert!(output.contains("2    计算机"));
        assert!(output.contains("38"));
    }

    #[test]
    fn render_market_strength_report_shows_zero_candidate_empty_sections() {
        let report = MarketStrengthReport {
            foundation: MarketFoundationSummary {
                total_stocks: 100,
                classified_stocks: 85,
                unclassified_stocks: 15,
                sector_count: 23,
                top_sectors: vec![],
            },
            strong_sectors: vec![],
            weak_sectors: vec![],
            top_by_market_cap: vec![],
            top_by_profit: vec![],
            candidate_stock_count: 0,
            market_cap_coverage_count: 0,
            profit_coverage_count: 0,
            valuation_error_count: 0,
            earnings_error_count: 0,
        };

        let output = render_market_strength_report(&report);

        assert!(output.contains("== 强弱板块分析 =="));
        assert!(output.contains("基础数据: A股=100 行业覆盖=85 未覆盖=15"));
        assert!(output.contains("强势板块候选股数: 0"));
        assert!(output.contains("基本面覆盖: 市值=0/0 利润=0/0"));
        assert!(output.contains("强势板块:"));
        assert!(output.contains("📭 没有可展示的板块数据"));
        assert!(output.contains("弱势板块:"));
        assert!(output.contains("强势板块个股 Top0 总市值:"));
        assert!(output.contains("强势板块个股 Top0 推算净利润:"));
        assert!(output.contains("📭 没有可展示的个股数据"));
    }

    #[test]
    fn render_market_strength_report_shows_fetch_error_counts() {
        let report = MarketStrengthReport {
            foundation: MarketFoundationSummary {
                total_stocks: 5300,
                classified_stocks: 5200,
                unclassified_stocks: 100,
                sector_count: 31,
                top_sectors: vec![],
            },
            strong_sectors: vec![BoardRankRow::new(
                "BK001",
                "银行",
                BoardType::Sector,
                1,
                2.1,
            )],
            weak_sectors: vec![],
            top_by_market_cap: vec![],
            top_by_profit: vec![],
            candidate_stock_count: 12,
            market_cap_coverage_count: 8,
            profit_coverage_count: 6,
            valuation_error_count: 2,
            earnings_error_count: 1,
        };

        let output = render_market_strength_report(&report);

        assert!(output.contains("强势板块候选股数: 12"));
        assert!(output.contains("基本面覆盖: 市值=8/12 利润=6/12"));
        assert!(output.contains("基本面抓取异常: 市值请求失败=2 利润请求失败=1"));
        assert!(output.contains("银行"));
    }

    #[test]
    fn render_market_strength_stock_ranking_includes_sector_filter_and_row_values() {
        let ranking = MarketStrengthStockRankingOutput {
            metric: StrengthStockMetric::Profit,
            strong_top: 3,
            sector_filter: Some("银行".to_string()),
            candidate_stock_count: 1,
            covered_count: 1,
            rows: vec![StrongSectorStockRow {
                sector_name: "银行".to_string(),
                code: "601398".to_string(),
                name: "工商银行".to_string(),
                latest_price: 7.0,
                latest_change_pct: 1.5,
                market_cap: Some(Decimal::new(700000, 2)),
                latest_report_profit: Some(Decimal::new(10000, 2)),
            }],
        };

        let output = render_market_strength_stock_ranking(&ranking);

        assert!(output.contains("== 强势板块个股排行 =="));
        assert!(output.contains("强势板块范围: Top3"));
        assert!(output.contains("行业过滤: 银行"));
        assert!(output.contains("上一会计周期净利润覆盖: 1/1"));
        assert!(output.contains("按上一会计周期净利润从大到小 Top1:"));
        assert!(output.contains("601398"));
        assert!(output.contains("工商银行"));
        assert!(output.contains("100.00"));
    }

    #[test]
    fn render_market_strength_stock_ranking_shows_empty_state() {
        let ranking = MarketStrengthStockRankingOutput {
            metric: StrengthStockMetric::MarketCap,
            strong_top: 3,
            sector_filter: None,
            candidate_stock_count: 0,
            covered_count: 0,
            rows: vec![],
        };

        let output = render_market_strength_stock_ranking(&ranking);

        assert!(output.contains("强势板块范围: Top3"));
        assert!(output.contains("总市值覆盖: 0/0"));
        assert!(output.contains("按总市值从大到小 Top0:"));
        assert!(output.contains("📭 没有可展示的个股数据"));
    }
}
