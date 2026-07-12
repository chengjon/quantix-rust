//! Thin `print_*` entry points for the various market snapshot reports.
//!
//! The heavy string-building helpers live in [`render`]; this module keeps
//! the entry points themselves so [`super::handlers`] can import a compact
//! re-export list.

#[cfg(test)]
mod tests;

mod render;

use render::{
    render_market_board_rows, render_market_foundation_summary, render_market_overview,
    render_market_strength_report, render_market_strength_stock_ranking,
};

use crate::market::{
    BoardRankRow, LeaderRow, MarketFoundationSummary, MarketOverview, MarketSentimentSnapshot,
    MarketStrengthReport, NorthFlowSnapshot,
};

pub(super) fn print_market_foundation_summary(summary: &MarketFoundationSummary) {
    print!("{}", render_market_foundation_summary(summary));
}

pub(super) fn print_market_board_rows(rows: &[BoardRankRow]) {
    print!("{}", render_market_board_rows(rows));
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
    print!("{}", render_market_overview(overview));
}

pub(super) fn print_market_strength_report(report: &MarketStrengthReport) {
    print!("{}", render_market_strength_report(report));
}

pub(super) fn print_market_strength_stock_ranking(ranking: &MarketStrengthStockRankingOutput) {
    print!("{}", render_market_strength_stock_ranking(ranking));
}

use crate::cli::handlers::market_handler::MarketStrengthStockRankingOutput;
