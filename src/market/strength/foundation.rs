use std::collections::HashMap;

use crate::core::{QuantixError, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct MarketSnapshotRow {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
    pub volume: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketIndustryClassificationRow {
    pub code: String,
    pub industry_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AShareIndustryRow {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
    pub volume: f64,
    pub amount: f64,
    pub industry_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorCoverageRow {
    pub industry_name: String,
    pub stock_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketFoundationSummary {
    pub total_stocks: usize,
    pub classified_stocks: usize,
    pub unclassified_stocks: usize,
    pub sector_count: usize,
    pub top_sectors: Vec<SectorCoverageRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarketAnalysisFoundation {
    pub rows: Vec<AShareIndustryRow>,
    pub summary: MarketFoundationSummary,
}

pub fn build_market_analysis_foundation(
    stocks: Vec<MarketSnapshotRow>,
    industry_rows: Vec<MarketIndustryClassificationRow>,
) -> Result<MarketAnalysisFoundation> {
    if industry_rows.is_empty() {
        return Err(QuantixError::Other(
            "未找到行业分类数据，请先运行 quantix risk sync industry --standard shenwan"
                .to_string(),
        ));
    }

    let industry_by_code: HashMap<String, String> = industry_rows
        .into_iter()
        .map(|row| (row.code, row.industry_name))
        .collect();

    let rows: Vec<AShareIndustryRow> = stocks
        .into_iter()
        .map(|stock| AShareIndustryRow {
            industry_name: industry_by_code.get(&stock.code).cloned(),
            code: stock.code,
            name: stock.name,
            price: stock.price,
            change_pct: stock.change_pct,
            volume: stock.volume,
            amount: stock.amount,
        })
        .collect();

    let classified_stocks = rows
        .iter()
        .filter(|row| row.industry_name.is_some())
        .count();
    let mut sector_counts: HashMap<String, usize> = HashMap::new();
    for row in rows.iter().filter_map(|row| row.industry_name.as_ref()) {
        *sector_counts.entry(row.clone()).or_default() += 1;
    }
    let sector_count = sector_counts.len();

    let mut top_sectors: Vec<SectorCoverageRow> = sector_counts
        .into_iter()
        .map(|(industry_name, stock_count)| SectorCoverageRow {
            industry_name,
            stock_count,
        })
        .collect();
    top_sectors.sort_by(|left, right| {
        right
            .stock_count
            .cmp(&left.stock_count)
            .then_with(|| left.industry_name.cmp(&right.industry_name))
    });
    top_sectors.truncate(10);

    let total_stocks = rows.len();
    let summary = MarketFoundationSummary {
        total_stocks,
        classified_stocks,
        unclassified_stocks: total_stocks.saturating_sub(classified_stocks),
        sector_count,
        top_sectors,
    };

    Ok(MarketAnalysisFoundation { rows, summary })
}
