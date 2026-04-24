use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;

use chrono::NaiveDate;
use futures_util::future::join_all;
use rust_decimal::Decimal;

use crate::anomaly::{DataSource, EastMoneyAnomalySource, StockInfo};
use crate::core::{QuantixError, Result};
use crate::fundamental::earnings::EarningsFetcher;
use crate::fundamental::valuation::ValuationFetcher;
use crate::market::{BoardRankRow, BoardSortBy, BoardType, MarketDataReader};
use crate::risk::{
    ACTIVE_CLASSIFICATION_STANDARD, ACTIVE_INDUSTRY_LEVEL, IndustryReferenceRecord,
    SqliteIndustryStore,
};

const ALL_SECTOR_SCAN_LIMIT: usize = 512;

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
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarketAnalysisFoundation {
    pub rows: Vec<AShareIndustryRow>,
    pub summary: MarketFoundationSummary,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FundamentalSnapshot {
    code: String,
    market_cap: Option<Decimal>,
    latest_report_profit: Option<Decimal>,
}

pub fn build_market_analysis_foundation(
    stocks: Vec<StockInfo>,
    industry_rows: Vec<IndustryReferenceRecord>,
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

pub async fn load_market_analysis_foundation(
    risk_state_path: impl AsRef<Path>,
) -> Result<MarketAnalysisFoundation> {
    let source = EastMoneyAnomalySource::new();
    let stocks = source
        .get_stock_list()
        .await
        .map_err(|err| QuantixError::Other(format!("获取全市场 A 股列表失败: {err}")))?;
    let store = SqliteIndustryStore::from_risk_state_path(risk_state_path).await?;
    let industry_rows = store
        .list_current(ACTIVE_CLASSIFICATION_STANDARD, ACTIVE_INDUSTRY_LEVEL)
        .await?;

    build_market_analysis_foundation(stocks, industry_rows)
}

pub async fn analyze_market_strength_with_reader<R>(
    reader: &R,
    date: Option<NaiveDate>,
    risk_state_path: impl AsRef<Path>,
    strong_top: usize,
    weak_top: usize,
    stock_top: usize,
) -> Result<MarketStrengthReport>
where
    R: MarketDataReader,
{
    let strong_top = strong_top.max(1);
    let weak_top = weak_top.max(1);
    let stock_top = stock_top.max(1);

    let foundation = load_market_analysis_foundation(risk_state_path).await?;
    let sector_rows = reader
        .load_board_rankings(
            BoardType::Sector,
            date,
            ALL_SECTOR_SCAN_LIMIT,
            BoardSortBy::ChangePct,
        )
        .await?;
    let snapshots = fetch_fundamental_snapshots(
        foundation
            .rows
            .iter()
            .filter(|row| row.industry_name.is_some())
            .map(|row| row.code.as_str())
            .collect::<Vec<_>>(),
    )
    .await;

    Ok(build_market_strength_report(
        foundation,
        sector_rows,
        snapshots,
        strong_top,
        weak_top,
        stock_top,
    ))
}

pub(crate) fn build_market_strength_report(
    foundation: MarketAnalysisFoundation,
    mut sector_rows: Vec<BoardRankRow>,
    snapshots: Vec<FundamentalSnapshot>,
    strong_top: usize,
    weak_top: usize,
    stock_top: usize,
) -> MarketStrengthReport {
    let strong_top = strong_top.max(1);
    let weak_top = weak_top.max(1);
    let stock_top = stock_top.max(1);

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
    let snapshot_by_code: HashMap<String, FundamentalSnapshot> = snapshots
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

    MarketStrengthReport {
        foundation: foundation.summary,
        strong_sectors,
        weak_sectors,
        top_by_market_cap,
        top_by_profit,
        candidate_stock_count: enriched_rows.len(),
    }
}

async fn fetch_fundamental_snapshots(codes: Vec<&str>) -> Vec<FundamentalSnapshot> {
    let valuation_fetcher = ValuationFetcher::new();
    let earnings_fetcher = EarningsFetcher::new();

    let tasks = codes.into_iter().map(|code| {
        let valuation_fetcher = &valuation_fetcher;
        let earnings_fetcher = &earnings_fetcher;
        async move {
            let valuation = valuation_fetcher.fetch_from_eastmoney(code).await.ok();
            let earnings = earnings_fetcher.fetch_latest(code).await.ok();

            FundamentalSnapshot {
                code: code.to_string(),
                market_cap: valuation.and_then(|row| row.market_cap),
                latest_report_profit: earnings.and_then(|row| row.net_profit),
            }
        }
    });

    join_all(tasks).await
}

fn compare_board_rows_desc(left: &BoardRankRow, right: &BoardRankRow) -> Ordering {
    compare_f64_desc(left.change_pct, right.change_pct)
        .then_with(|| left.rank.cmp(&right.rank))
        .then_with(|| left.board_code.cmp(&right.board_code))
}

fn compare_board_rows_asc(left: &BoardRankRow, right: &BoardRankRow) -> Ordering {
    compare_f64_asc(left.change_pct, right.change_pct)
        .then_with(|| left.rank.cmp(&right.rank))
        .then_with(|| left.board_code.cmp(&right.board_code))
}

fn compare_market_cap_desc(left: &StrongSectorStockRow, right: &StrongSectorStockRow) -> Ordering {
    compare_decimal_desc(left.market_cap.clone(), right.market_cap.clone())
        .then_with(|| {
            compare_decimal_desc(
                left.latest_report_profit.clone(),
                right.latest_report_profit.clone(),
            )
        })
        .then_with(|| left.sector_name.cmp(&right.sector_name))
        .then_with(|| left.code.cmp(&right.code))
}

fn compare_profit_desc(left: &StrongSectorStockRow, right: &StrongSectorStockRow) -> Ordering {
    compare_decimal_desc(
        left.latest_report_profit.clone(),
        right.latest_report_profit.clone(),
    )
    .then_with(|| compare_decimal_desc(left.market_cap.clone(), right.market_cap.clone()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::risk::{ClassificationStandard, IndustryClassificationLevel};

    fn sample_stock(code: &str, name: &str, price: f64, change_pct: f64, amount: f64) -> StockInfo {
        StockInfo {
            code: code.to_string(),
            name: name.to_string(),
            price,
            change_pct,
            volume: 1000.0,
            amount,
            list_date: None,
        }
    }

    fn sample_industry(code: &str, industry_name: &str) -> IndustryReferenceRecord {
        IndustryReferenceRecord {
            code: code.to_string(),
            industry_name: industry_name.to_string(),
            standard: ClassificationStandard::Shenwan,
            level: IndustryClassificationLevel::FirstLevel,
            source: "test".to_string(),
        }
    }

    #[test]
    fn foundation_builds_coverage_summary() {
        let foundation = build_market_analysis_foundation(
            vec![
                sample_stock("600000", "浦发银行", 10.0, 2.1, 1000.0),
                sample_stock("601398", "工商银行", 7.0, 1.5, 800.0),
                sample_stock("300024", "机器人", 15.0, 4.2, 1200.0),
            ],
            vec![
                sample_industry("600000", "银行"),
                sample_industry("601398", "银行"),
                sample_industry("300024", "机械设备"),
            ],
        )
        .unwrap();

        assert_eq!(foundation.summary.total_stocks, 3);
        assert_eq!(foundation.summary.classified_stocks, 3);
        assert_eq!(foundation.summary.unclassified_stocks, 0);
        assert_eq!(foundation.summary.sector_count, 2);
        assert_eq!(foundation.summary.top_sectors[0].industry_name, "银行");
        assert_eq!(foundation.summary.top_sectors[0].stock_count, 2);
    }

    #[test]
    fn foundation_requires_existing_industry_data() {
        let err = build_market_analysis_foundation(
            vec![sample_stock("600000", "浦发银行", 10.0, 2.1, 1000.0)],
            Vec::new(),
        )
        .unwrap_err();

        assert!(err.to_string().contains("risk sync industry"));
    }

    #[test]
    fn ranking_prefers_larger_metric_values() {
        let mut rows = vec![
            StrongSectorStockRow {
                sector_name: "银行".to_string(),
                code: "601398".to_string(),
                name: "工商银行".to_string(),
                latest_price: 7.0,
                latest_change_pct: 1.0,
                market_cap: Some(Decimal::new(300000, 2)),
                latest_report_profit: Some(Decimal::new(10000, 2)),
            },
            StrongSectorStockRow {
                sector_name: "银行".to_string(),
                code: "600000".to_string(),
                name: "浦发银行".to_string(),
                latest_price: 10.0,
                latest_change_pct: 2.0,
                market_cap: Some(Decimal::new(500000, 2)),
                latest_report_profit: Some(Decimal::new(8000, 2)),
            },
        ];

        rows.sort_by(compare_market_cap_desc);
        assert_eq!(rows[0].code, "600000");

        rows.sort_by(compare_profit_desc);
        assert_eq!(rows[0].code, "601398");
    }

    #[test]
    fn board_ordering_supports_strong_and_weak_views() {
        let mut rows = vec![
            BoardRankRow::new("BK001", "银行", BoardType::Sector, 1, 2.5),
            BoardRankRow::new("BK002", "有色金属", BoardType::Sector, 2, -1.8),
            BoardRankRow::new("BK003", "计算机", BoardType::Sector, 3, 4.1),
        ];

        rows.sort_by(compare_board_rows_desc);
        assert_eq!(rows[0].board_name, "计算机");

        rows.sort_by(compare_board_rows_asc);
        assert_eq!(rows[0].board_name, "有色金属");
    }

    #[test]
    fn strength_report_builds_strong_weak_and_ranked_stock_views() {
        let foundation = build_market_analysis_foundation(
            vec![
                sample_stock("600000", "浦发银行", 10.0, 2.1, 1000.0),
                sample_stock("601398", "工商银行", 7.0, 1.5, 800.0),
                sample_stock("300024", "机器人", 15.0, 4.2, 1200.0),
                sample_stock("000960", "锡业股份", 14.0, -1.0, 900.0),
            ],
            vec![
                sample_industry("600000", "银行"),
                sample_industry("601398", "银行"),
                sample_industry("300024", "计算机"),
                sample_industry("000960", "有色金属"),
            ],
        )
        .unwrap();

        let report = build_market_strength_report(
            foundation,
            vec![
                BoardRankRow::new("BK001", "计算机", BoardType::Sector, 1, 4.1),
                BoardRankRow::new("BK002", "银行", BoardType::Sector, 2, 2.5),
                BoardRankRow::new("BK003", "有色金属", BoardType::Sector, 3, -1.8),
            ],
            vec![
                FundamentalSnapshot {
                    code: "600000".to_string(),
                    market_cap: Some(Decimal::new(500000, 2)),
                    latest_report_profit: Some(Decimal::new(8000, 2)),
                },
                FundamentalSnapshot {
                    code: "601398".to_string(),
                    market_cap: Some(Decimal::new(700000, 2)),
                    latest_report_profit: Some(Decimal::new(10000, 2)),
                },
                FundamentalSnapshot {
                    code: "300024".to_string(),
                    market_cap: Some(Decimal::new(200000, 2)),
                    latest_report_profit: Some(Decimal::new(3000, 2)),
                },
            ],
            2,
            1,
            2,
        );

        assert_eq!(report.strong_sectors.len(), 2);
        assert_eq!(report.strong_sectors[0].board_name, "计算机");
        assert_eq!(report.weak_sectors.len(), 1);
        assert_eq!(report.weak_sectors[0].board_name, "有色金属");
        assert_eq!(report.candidate_stock_count, 3);
        assert_eq!(report.top_by_market_cap.len(), 2);
        assert_eq!(report.top_by_market_cap[0].code, "601398");
        assert_eq!(report.top_by_profit.len(), 2);
        assert_eq!(report.top_by_profit[0].code, "601398");
    }
}
