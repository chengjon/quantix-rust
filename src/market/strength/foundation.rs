use std::collections::HashMap;

use crate::core::{QuantixError, Result};

/// A股市场快照单行：code/name 标的与名称、price 最新价、change_pct 涨跌幅、volume 成交量、amount 成交额。
#[derive(Debug, Clone, PartialEq)]
pub struct MarketSnapshotRow {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
    pub volume: f64,
    pub amount: f64,
}

/// 行业分类单行：code 标的、industry_name 行业名（已按当前 classification standard 解析）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketIndustryClassificationRow {
    pub code: String,
    pub industry_name: String,
}

/// 带行业归属的 A 股快照单行：除基础快照字段外，industry_name 可空（未分类标的为 None）。
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

/// 板块覆盖率行：industry_name 行业名、stock_count 该行业下已分类的 A 股数量。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorCoverageRow {
    pub industry_name: String,
    pub stock_count: usize,
}

/// 市场底层基础汇总：total_stocks A 股总数、classified_stocks 已分类数、unclassified_stocks 未分类数、sector_count 行业数、top_sectors 覆盖股票数 top N 行业。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketFoundationSummary {
    pub total_stocks: usize,
    pub classified_stocks: usize,
    pub unclassified_stocks: usize,
    pub sector_count: usize,
    pub top_sectors: Vec<SectorCoverageRow>,
}

/// 市场底层基础分析结果：rows 带行业的明细行、summary 汇总。
#[derive(Debug, Clone, PartialEq)]
pub struct MarketAnalysisFoundation {
    pub rows: Vec<AShareIndustryRow>,
    pub summary: MarketFoundationSummary,
}

/// 构造市场底层基础：把 stocks 与 industry_rows 按 code join 成 AShareIndustryRow，再聚合出 coverage 与 top_sectors。industry_rows 为空返回错误（提示先 sync industry）。
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
