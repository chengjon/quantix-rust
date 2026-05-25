use std::cmp::Ordering;
use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::market::strength::{
    AShareIndustryRow, MarketAnalysisFoundation, MarketFoundationSummary,
};
use crate::market::{BoardRankRow, BoardType};

#[derive(Debug, Clone, PartialEq)]
pub struct StrongSectorStockRow {
    pub sector_name: String,
    pub code: String,
    pub name: String,
    pub latest_price: f64,
    pub latest_change_pct: f64,
    pub market_cap: Option<Decimal>,
    pub latest_report_profit: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarketStrengthReport {
    pub foundation: MarketFoundationSummary,
    pub strong_sectors: Vec<BoardRankRow>,
    pub weak_sectors: Vec<BoardRankRow>,
    pub top_by_market_cap: Vec<StrongSectorStockRow>,
    pub top_by_profit: Vec<StrongSectorStockRow>,
    pub candidate_stock_count: usize,
    pub market_cap_coverage_count: usize,
    pub profit_coverage_count: usize,
    pub valuation_error_count: usize,
    pub earnings_error_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FundamentalSnapshot {
    pub(crate) code: String,
    pub(crate) market_cap: Option<Decimal>,
    pub(crate) latest_report_profit: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FundamentalSnapshotBatch {
    pub(crate) snapshots: Vec<FundamentalSnapshot>,
    pub(crate) valuation_error_count: usize,
    pub(crate) earnings_error_count: usize,
}

pub(crate) fn build_market_strength_report(
    foundation: MarketAnalysisFoundation,
    sector_rows: Vec<BoardRankRow>,
    snapshot_batch: FundamentalSnapshotBatch,
    strong_top: usize,
    weak_top: usize,
    stock_top: usize,
) -> MarketStrengthReport {
    let strong_top = strong_top.max(1);
    let weak_top = weak_top.max(1);
    let stock_top = stock_top.max(1);

    let mut sector_rows = resolve_sector_rows_for_strength(&foundation, sector_rows);
    sector_rows.sort_by(compare_board_rows_desc);

    let strong_sectors: Vec<BoardRankRow> = sector_rows.iter().take(strong_top).cloned().collect();

    let mut weak_sectors = sector_rows.clone();
    weak_sectors.sort_by(compare_board_rows_asc);
    weak_sectors.truncate(weak_top);

    let strong_sector_names: Vec<String> = strong_sectors
        .iter()
        .map(|row| row.board_name.clone())
        .collect();

    let candidate_rows: Vec<&AShareIndustryRow> = foundation
        .rows
        .iter()
        .filter(|row| {
            row.industry_name
                .as_ref()
                .is_some_and(|name| strong_sector_names.iter().any(|sector| sector == name))
        })
        .collect();
    let snapshot_by_code: HashMap<String, FundamentalSnapshot> = snapshot_batch
        .snapshots
        .iter()
        .cloned()
        .into_iter()
        .map(|snapshot| (snapshot.code.clone(), snapshot))
        .collect();

    let enriched_rows: Vec<StrongSectorStockRow> = candidate_rows
        .into_iter()
        .map(|row| StrongSectorStockRow {
            sector_name: row.industry_name.clone().unwrap_or_default(),
            code: row.code.clone(),
            name: row.name.clone(),
            latest_price: row.price,
            latest_change_pct: row.change_pct,
            market_cap: snapshot_by_code
                .get(&row.code)
                .and_then(|snapshot| snapshot.market_cap),
            latest_report_profit: snapshot_by_code
                .get(&row.code)
                .and_then(|snapshot| snapshot.latest_report_profit),
        })
        .collect();

    let mut top_by_market_cap: Vec<StrongSectorStockRow> = enriched_rows
        .iter()
        .filter(|row| row.market_cap.is_some())
        .cloned()
        .collect();
    top_by_market_cap.sort_by(compare_market_cap_desc);
    top_by_market_cap.truncate(stock_top);

    let mut top_by_profit: Vec<StrongSectorStockRow> = enriched_rows
        .iter()
        .filter(|row| row.latest_report_profit.is_some())
        .cloned()
        .collect();
    top_by_profit.sort_by(compare_profit_desc);
    top_by_profit.truncate(stock_top);

    let market_cap_coverage_count = enriched_rows
        .iter()
        .filter(|row| row.market_cap.is_some())
        .count();
    let profit_coverage_count = enriched_rows
        .iter()
        .filter(|row| row.latest_report_profit.is_some())
        .count();

    MarketStrengthReport {
        foundation: foundation.summary,
        strong_sectors,
        weak_sectors,
        top_by_market_cap,
        top_by_profit,
        candidate_stock_count: enriched_rows.len(),
        market_cap_coverage_count,
        profit_coverage_count,
        valuation_error_count: snapshot_batch.valuation_error_count,
        earnings_error_count: snapshot_batch.earnings_error_count,
    }
}

fn resolve_sector_rows_for_strength(
    foundation: &MarketAnalysisFoundation,
    sector_rows: Vec<BoardRankRow>,
) -> Vec<BoardRankRow> {
    let aligned_sector_rows: Vec<BoardRankRow> = sector_rows
        .into_iter()
        .filter(|row| {
            foundation
                .rows
                .iter()
                .any(|stock| stock.industry_name.as_deref() == Some(row.board_name.as_str()))
        })
        .collect();

    if !aligned_sector_rows.is_empty() {
        return aligned_sector_rows;
    }

    derive_sector_rows_from_foundation(foundation)
}

fn derive_sector_rows_from_foundation(foundation: &MarketAnalysisFoundation) -> Vec<BoardRankRow> {
    let mut aggregates: HashMap<String, (f64, usize)> = HashMap::new();
    for row in &foundation.rows {
        let Some(industry_name) = row.industry_name.as_ref() else {
            continue;
        };

        let entry = aggregates.entry(industry_name.clone()).or_insert((0.0, 0));
        entry.0 += row.change_pct;
        entry.1 += 1;
    }

    let mut rows: Vec<BoardRankRow> = aggregates
        .into_iter()
        .map(|(industry_name, (total_change_pct, stock_count))| {
            let average_change_pct = if stock_count == 0 {
                0.0
            } else {
                total_change_pct / stock_count as f64
            };

            BoardRankRow::new(
                format!("derived:{industry_name}"),
                industry_name,
                BoardType::Sector,
                0,
                average_change_pct,
            )
        })
        .collect();

    rows.sort_by(compare_board_rows_desc);
    for (index, row) in rows.iter_mut().enumerate() {
        row.rank = index + 1;
    }

    rows
}

pub(super) fn compare_board_rows_desc(left: &BoardRankRow, right: &BoardRankRow) -> Ordering {
    compare_f64_desc(left.change_pct, right.change_pct)
        .then_with(|| left.rank.cmp(&right.rank))
        .then_with(|| left.board_code.cmp(&right.board_code))
}

pub(super) fn compare_board_rows_asc(left: &BoardRankRow, right: &BoardRankRow) -> Ordering {
    compare_f64_asc(left.change_pct, right.change_pct)
        .then_with(|| left.rank.cmp(&right.rank))
        .then_with(|| left.board_code.cmp(&right.board_code))
}

pub(super) fn compare_market_cap_desc(
    left: &StrongSectorStockRow,
    right: &StrongSectorStockRow,
) -> Ordering {
    compare_decimal_desc(left.market_cap, right.market_cap)
        .then_with(|| compare_decimal_desc(left.latest_report_profit, right.latest_report_profit))
        .then_with(|| left.sector_name.cmp(&right.sector_name))
        .then_with(|| left.code.cmp(&right.code))
}

pub(super) fn compare_profit_desc(
    left: &StrongSectorStockRow,
    right: &StrongSectorStockRow,
) -> Ordering {
    compare_decimal_desc(left.latest_report_profit, right.latest_report_profit)
        .then_with(|| compare_decimal_desc(left.market_cap, right.market_cap))
        .then_with(|| left.sector_name.cmp(&right.sector_name))
        .then_with(|| left.code.cmp(&right.code))
}

fn compare_decimal_desc(left: Option<Decimal>, right: Option<Decimal>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => right.cmp(&left),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_f64_desc(left: f64, right: f64) -> Ordering {
    right.partial_cmp(&left).unwrap_or(Ordering::Equal)
}

fn compare_f64_asc(left: f64, right: f64) -> Ordering {
    left.partial_cmp(&right).unwrap_or(Ordering::Equal)
}
