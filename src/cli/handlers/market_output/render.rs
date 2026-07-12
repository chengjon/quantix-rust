//! Renderers that build the printable string payloads for the various
//! market snapshot reports. Kept separate from the thin `print_*` entry
//! points in [`mod`] so the entry points stay a compact re-export list.

use crate::cli::command_types::StrengthStockMetric;
use crate::cli::handlers::market_handler::MarketStrengthStockRankingOutput;
use crate::market::{
    BoardRankRow, MarketFoundationSummary, MarketOverview, MarketStrengthReport,
    StrongSectorStockRow,
};
use rust_decimal::Decimal;
use std::fmt::Write as _;

macro_rules! append_line {
    ($output:expr) => {{
        let _ = writeln!($output);
    }};
    ($output:expr, $($arg:tt)*) => {{
        let _ = writeln!($output, $($arg)*);
    }};
}

pub(crate) fn format_decimal(value: &Option<Decimal>) -> String {
    match value {
        Some(value) => format!("{:.2}", value),
        None => "-".to_string(),
    }
}

pub(crate) fn render_market_foundation_summary(summary: &MarketFoundationSummary) -> String {
    let mut output = String::new();

    append_line!(&mut output, "== 市场基础数据 ==");
    append_line!(&mut output, "A股总数: {}", summary.total_stocks);
    append_line!(&mut output, "已匹配行业: {}", summary.classified_stocks);
    append_line!(&mut output, "未匹配行业: {}", summary.unclassified_stocks);
    append_line!(&mut output, "行业数: {}", summary.sector_count);

    if !summary.top_sectors.is_empty() {
        append_line!(&mut output);
        append_line!(&mut output, "行业覆盖 Top10:");
        append_line!(&mut output, "{:<4} {:<16} 成分股数", "排名", "行业");
        append_line!(&mut output, "{}", "-".repeat(40));
        for (idx, row) in summary.top_sectors.iter().enumerate() {
            append_line!(
                &mut output,
                "{:<4} {:<16} {}",
                idx + 1,
                row.industry_name,
                row.stock_count
            );
        }
    }

    output
}

pub(crate) fn render_market_board_rows(rows: &[BoardRankRow]) -> String {
    let mut output = String::new();

    if rows.is_empty() {
        append_line!(&mut output, "📭 没有可展示的板块数据");
        return output;
    }

    append_line!(
        &mut output,
        "{:<8} {:<12} {:<16} 涨跌幅",
        "排名",
        "代码",
        "板块"
    );
    append_line!(&mut output, "{}", "-".repeat(56));

    for row in rows {
        append_line!(
            &mut output,
            "{:<8} {:<12} {:<16} {:.2}%",
            row.rank,
            row.board_code,
            row.board_name,
            row.change_pct
        );
    }

    output
}

pub(crate) fn render_market_overview(overview: &MarketOverview) -> String {
    let mut output = String::new();

    append_line!(&mut output, "== 市场概览 ==");
    append_line!(&mut output, "行业板块: {}", overview.top_sectors.len());
    append_line!(&mut output, "概念板块: {}", overview.top_concepts.len());

    match overview.north_flow.as_ref() {
        Some(snapshot) => append_line!(&mut output, "北向资金: {:.2}", snapshot.total_amount),
        None => append_line!(&mut output, "北向资金: -"),
    }

    match overview.sentiment.as_ref() {
        Some(snapshot) => append_line!(&mut output, "涨停数: {}", snapshot.limit_up_count),
        None => append_line!(&mut output, "涨停数: -"),
    }

    if !overview.top_sectors.is_empty() {
        append_line!(&mut output);
        append_line!(&mut output, "Top 行业:");
        output.push_str(&render_market_board_rows(&overview.top_sectors));
    }

    if !overview.top_concepts.is_empty() {
        append_line!(&mut output);
        append_line!(&mut output, "Top 概念:");
        output.push_str(&render_market_board_rows(&overview.top_concepts));
    }

    output
}

pub(crate) fn render_market_strength_report(report: &MarketStrengthReport) -> String {
    let mut output = String::new();

    append_line!(&mut output, "== 强弱板块分析 ==");
    append_line!(
        &mut output,
        "基础数据: A股={} 行业覆盖={} 未覆盖={}",
        report.foundation.total_stocks,
        report.foundation.classified_stocks,
        report.foundation.unclassified_stocks
    );
    append_line!(
        &mut output,
        "强势板块候选股数: {}",
        report.candidate_stock_count
    );
    append_line!(
        &mut output,
        "基本面覆盖: 市值={}/{} 利润={}/{}",
        report.market_cap_coverage_count,
        report.candidate_stock_count,
        report.profit_coverage_count,
        report.candidate_stock_count
    );
    if report.valuation_error_count > 0 || report.earnings_error_count > 0 {
        append_line!(
            &mut output,
            "基本面抓取异常: 市值请求失败={} 利润请求失败={}",
            report.valuation_error_count,
            report.earnings_error_count
        );
    }

    append_line!(&mut output);
    append_line!(&mut output, "强势板块:");
    output.push_str(&render_market_board_rows(&report.strong_sectors));

    append_line!(&mut output);
    append_line!(&mut output, "弱势板块:");
    output.push_str(&render_market_board_rows(&report.weak_sectors));

    append_line!(&mut output);
    append_line!(
        &mut output,
        "强势板块个股 Top{} 总市值:",
        report.top_by_market_cap.len()
    );
    output.push_str(&render_strong_sector_stock_rows(
        &report.top_by_market_cap,
        "总市值(亿)",
        true,
    ));

    append_line!(&mut output);
    append_line!(
        &mut output,
        "强势板块个股 Top{} 推算净利润:",
        report.top_by_profit.len()
    );
    output.push_str(&render_strong_sector_stock_rows(
        &report.top_by_profit,
        "推算净利润(亿)",
        false,
    ));

    output
}

pub(crate) fn render_market_strength_stock_ranking(
    ranking: &MarketStrengthStockRankingOutput,
) -> String {
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

    append_line!(&mut output, "== 强势板块个股排行 ==");
    append_line!(&mut output, "强势板块范围: Top{}", ranking.strong_top);
    if let Some(sector_name) = ranking.sector_filter.as_deref() {
        append_line!(&mut output, "行业过滤: {}", sector_name);
    }
    append_line!(&mut output, "候选股数: {}", ranking.candidate_stock_count);
    append_line!(
        &mut output,
        "{}覆盖: {}/{}",
        metric_name,
        ranking.covered_count,
        ranking.candidate_stock_count
    );
    append_line!(&mut output);
    append_line!(
        &mut output,
        "按{}从大到小 Top{}:",
        metric_name,
        ranking.rows.len()
    );
    output.push_str(&render_strong_sector_stock_rows(
        &ranking.rows,
        metric_label,
        use_market_cap,
    ));

    output
}

pub(crate) fn render_strong_sector_stock_rows(
    rows: &[StrongSectorStockRow],
    metric_label: &str,
    use_market_cap: bool,
) -> String {
    let mut output = String::new();

    if rows.is_empty() {
        append_line!(&mut output, "📭 没有可展示的个股数据");
        return output;
    }

    append_line!(
        &mut output,
        "{:<4} {:<10} {:<12} {:<12} {:<12} {}",
        "排名",
        "行业",
        "代码",
        "名称",
        "现价",
        metric_label
    );
    append_line!(&mut output, "{}", "-".repeat(84));

    for (idx, row) in rows.iter().enumerate() {
        let metric = if use_market_cap {
            row.market_cap
        } else {
            row.latest_report_profit
        };
        append_line!(
            &mut output,
            "{:<4} {:<10} {:<12} {:<12} {:<12.2} {}",
            idx + 1,
            row.sector_name,
            row.code,
            row.name,
            row.latest_price,
            format_decimal(&metric)
        );
    }

    output
}
