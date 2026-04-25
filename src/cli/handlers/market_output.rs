use super::*;
use crate::cli::handlers::market_handler::MarketStrengthStockRankingOutput;

pub(super) fn print_market_foundation_summary(summary: &MarketFoundationSummary) {
    println!("== 市场基础数据 ==");
    println!("A股总数: {}", summary.total_stocks);
    println!("已匹配行业: {}", summary.classified_stocks);
    println!("未匹配行业: {}", summary.unclassified_stocks);
    println!("行业数: {}", summary.sector_count);

    if !summary.top_sectors.is_empty() {
        println!();
        println!("行业覆盖 Top10:");
        println!("{:<4} {:<16} 成分股数", "排名", "行业");
        println!("{}", "-".repeat(40));
        for (idx, row) in summary.top_sectors.iter().enumerate() {
            println!(
                "{:<4} {:<16} {}",
                idx + 1,
                row.industry_name,
                row.stock_count
            );
        }
    }
}

pub(super) fn print_market_board_rows(rows: &[BoardRankRow]) {
    if rows.is_empty() {
        println!("📭 没有可展示的板块数据");
        return;
    }

    println!("{:<8} {:<12} {:<16} 涨跌幅", "排名", "代码", "板块");
    println!("{}", "-".repeat(56));

    for row in rows {
        println!(
            "{:<8} {:<12} {:<16} {:.2}%",
            row.rank, row.board_code, row.board_name, row.change_pct
        );
    }
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
    println!("== 强弱板块分析 ==");
    println!(
        "基础数据: A股={} 行业覆盖={} 未覆盖={}",
        report.foundation.total_stocks,
        report.foundation.classified_stocks,
        report.foundation.unclassified_stocks
    );
    println!("强势板块候选股数: {}", report.candidate_stock_count);
    println!(
        "基本面覆盖: 市值={}/{} 利润={}/{}",
        report.market_cap_coverage_count,
        report.candidate_stock_count,
        report.profit_coverage_count,
        report.candidate_stock_count
    );
    if report.valuation_error_count > 0 || report.earnings_error_count > 0 {
        println!(
            "基本面抓取异常: 市值请求失败={} 利润请求失败={}",
            report.valuation_error_count, report.earnings_error_count
        );
    }

    println!();
    println!("强势板块:");
    print_market_board_rows(&report.strong_sectors);

    println!();
    println!("弱势板块:");
    print_market_board_rows(&report.weak_sectors);

    println!();
    println!("强势板块个股 Top{} 总市值:", report.top_by_market_cap.len());
    print_strong_sector_stock_rows(&report.top_by_market_cap, "总市值(亿)", true);

    println!();
    println!("强势板块个股 Top{} 推算净利润:", report.top_by_profit.len());
    print_strong_sector_stock_rows(&report.top_by_profit, "推算净利润(亿)", false);
}

pub(super) fn print_market_strength_stock_ranking(ranking: &MarketStrengthStockRankingOutput) {
    let metric_label = match ranking.metric {
        StrengthStockMetric::MarketCap => "总市值(亿)",
        StrengthStockMetric::Profit => "上一会计周期净利润(亿)",
    };
    let metric_name = match ranking.metric {
        StrengthStockMetric::MarketCap => "总市值",
        StrengthStockMetric::Profit => "上一会计周期净利润",
    };
    let use_market_cap = matches!(ranking.metric, StrengthStockMetric::MarketCap);

    println!("== 强势板块个股排行 ==");
    println!("强势板块范围: Top{}", ranking.strong_top);
    if let Some(sector_name) = ranking.sector_filter.as_deref() {
        println!("行业过滤: {}", sector_name);
    }
    println!("候选股数: {}", ranking.candidate_stock_count);
    println!(
        "{}覆盖: {}/{}",
        metric_name, ranking.covered_count, ranking.candidate_stock_count
    );
    println!();
    println!("按{}从大到小 Top{}:", metric_name, ranking.rows.len());
    print_strong_sector_stock_rows(&ranking.rows, metric_label, use_market_cap);
}

fn print_strong_sector_stock_rows(
    rows: &[StrongSectorStockRow],
    metric_label: &str,
    use_market_cap: bool,
) {
    if rows.is_empty() {
        println!("📭 没有可展示的个股数据");
        return;
    }

    println!(
        "{:<4} {:<10} {:<12} {:<12} {:<12} {}",
        "排名", "行业", "代码", "名称", "现价", metric_label
    );
    println!("{}", "-".repeat(84));

    for (idx, row) in rows.iter().enumerate() {
        let metric = if use_market_cap {
            row.market_cap.clone()
        } else {
            row.latest_report_profit.clone()
        };
        println!(
            "{:<4} {:<10} {:<12} {:<12} {:<12.2} {}",
            idx + 1,
            row.sector_name,
            row.code,
            row.name,
            row.latest_price,
            format_decimal(&metric)
        );
    }
}

fn format_decimal(value: &Option<Decimal>) -> String {
    match value {
        Some(value) => format!("{:.2}", value),
        None => "-".to_string(),
    }
}
