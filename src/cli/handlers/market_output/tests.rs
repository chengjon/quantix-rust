//! Snapshot render tests extracted from the original inline [`mod tests`].
//!
//! The heavy `render_*` helpers now live in [`super::render`]; this module
//! keeps the assertions that lock in the rendered string contracts.

use super::render::{
    render_market_foundation_summary, render_market_overview, render_market_strength_report,
    render_market_strength_stock_ranking,
};
use crate::cli::command_types::StrengthStockMetric;
use crate::cli::handlers::market_handler::MarketStrengthStockRankingOutput;
use crate::market::SectorCoverageRow;
use crate::market::{
    BoardRankRow, BoardType, MarketFoundationSummary, MarketOverview, MarketSentimentSnapshot,
    MarketStrengthReport, NorthFlowSnapshot, StrongSectorStockRow,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[test]
fn render_market_overview_shows_missing_optional_summary_values() {
    let overview = MarketOverview {
        top_sectors: vec![],
        top_concepts: vec![],
        north_flow: None,
        sentiment: None,
    };

    let output = render_market_overview(&overview);

    assert!(output.contains("== 市场概览 =="));
    assert!(output.contains("行业板块: 0"));
    assert!(output.contains("概念板块: 0"));
    assert!(output.contains("北向资金: -"));
    assert!(output.contains("涨停数: -"));
    assert!(!output.contains("Top 行业:"));
    assert!(!output.contains("Top 概念:"));
}

#[test]
fn render_market_overview_includes_sector_and_concept_sections() {
    let trade_date = NaiveDate::from_ymd_opt(2026, 3, 10).unwrap_or(NaiveDate::MIN);
    let overview = MarketOverview {
        top_sectors: vec![BoardRankRow::new(
            "BK001",
            "银行",
            BoardType::Sector,
            1,
            2.10,
        )],
        top_concepts: vec![BoardRankRow::new(
            "GN001",
            "人工智能",
            BoardType::Concept,
            1,
            4.20,
        )],
        north_flow: Some(NorthFlowSnapshot::new(trade_date, 12.3, 8.6, 20.9, 100.0)),
        sentiment: Some(MarketSentimentSnapshot::new(
            trade_date, 3210, 1875, 87, 4, 0.81, 0.19, 23,
        )),
    };

    let output = render_market_overview(&overview);

    assert!(output.contains("行业板块: 1"));
    assert!(output.contains("概念板块: 1"));
    assert!(output.contains("北向资金: 20.90"));
    assert!(output.contains("涨停数: 87"));
    assert!(output.contains("Top 行业:"));
    assert!(output.contains("BK001"));
    assert!(output.contains("银行"));
    assert!(output.contains("Top 概念:"));
    assert!(output.contains("GN001"));
    assert!(output.contains("人工智能"));
}

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
