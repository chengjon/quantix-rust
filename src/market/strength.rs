use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use tokio::time::{Duration, sleep};
use tracing::warn;

use crate::anomaly::StockInfo;
use crate::core::{QuantixError, Result};
use crate::db::clickhouse::{ClickHouseClient, MarketFundamentalSnapshotCH};
use crate::market::{BoardRankRow, BoardSortBy, BoardType, MarketDataReader};
use crate::risk::{
    ACTIVE_CLASSIFICATION_STANDARD, ACTIVE_INDUSTRY_LEVEL, IndustryReferenceRecord,
    SqliteIndustryStore,
};

const ALL_SECTOR_SCAN_LIMIT: usize = 512;
const ALL_A_SHARE_PAGE_SIZE: usize = 200;
const EASTMONEY_CLIST_URL: &str = "https://push2.eastmoney.com/api/qt/clist/get";
const EASTMONEY_A_SHARE_FS: &str = "m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23";
const EASTMONEY_RETRY_ATTEMPTS: usize = 3;
const EASTMONEY_RETRY_DELAY_MS: u64 = 800;

#[derive(Debug, Deserialize)]
struct EastMoneyBatchFundamentalResponse {
    data: EastMoneyBatchFundamentalData,
}

#[derive(Debug, Deserialize)]
struct EastMoneyBatchFundamentalData {
    total: usize,
    diff: Vec<EastMoneyBatchFundamentalItem>,
}

#[derive(Debug, Clone, Deserialize)]
struct EastMoneyBatchFundamentalItem {
    #[serde(rename = "f12")]
    code: String,
    #[serde(rename = "f14")]
    name: String,
    #[serde(rename = "f2")]
    price: f64,
    #[serde(rename = "f3")]
    change_pct: Option<f64>,
    #[serde(rename = "f5")]
    volume: Option<f64>,
    #[serde(rename = "f6")]
    amount: Option<f64>,
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FundamentalSnapshotBatch {
    snapshots: Vec<FundamentalSnapshot>,
    valuation_error_count: usize,
    earnings_error_count: usize,
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
    let market_rows = fetch_a_share_market_snapshots_with_retry().await?;
    let stocks = market_rows
        .iter()
        .filter_map(stock_info_from_market_row)
        .collect();
    let store = SqliteIndustryStore::from_risk_state_path(risk_state_path).await?;
    let industry_rows = store
        .list_current(ACTIVE_CLASSIFICATION_STANDARD, ACTIVE_INDUSTRY_LEVEL)
        .await?;

    build_market_analysis_foundation(stocks, industry_rows)
}

async fn fetch_a_share_market_snapshots_with_retry() -> Result<Vec<EastMoneyBatchFundamentalItem>> {
    let page_size = ALL_A_SHARE_PAGE_SIZE.to_string();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .http1_only()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .unwrap_or_default();
    let mut page_no = 1usize;
    let mut all_rows = Vec::new();
    let mut total = None;

    loop {
        let mut last_error = None;
        let page_no_str = page_no.to_string();
        let mut parsed_page = None;

        for attempt in 0..EASTMONEY_RETRY_ATTEMPTS {
            match fetch_market_snapshot_page_via_reqwest(
                &client,
                page_no_str.as_str(),
                page_size.as_str(),
            )
            .await
            {
                Ok(parsed) => {
                    parsed_page = Some(parsed);
                    break;
                }
                Err(err) => {
                    last_error = Some(err.to_string());
                    match fetch_market_snapshot_page_via_curl(
                        page_no_str.as_str(),
                        page_size.as_str(),
                    ) {
                        Ok(parsed) => {
                            parsed_page = Some(parsed);
                            break;
                        }
                        Err(curl_err) => last_error = Some(curl_err.to_string()),
                    }
                }
            }

            if attempt + 1 < EASTMONEY_RETRY_ATTEMPTS {
                sleep(Duration::from_millis(EASTMONEY_RETRY_DELAY_MS)).await;
            }
        }

        let parsed_page = parsed_page.ok_or_else(|| {
            QuantixError::Other(format!(
                "获取全市场 A 股列表失败: {}",
                last_error.unwrap_or_else(|| "未知错误".to_string())
            ))
        })?;

        if total.is_none() {
            total = Some(parsed_page.data.total);
        }

        if parsed_page.data.diff.is_empty() {
            break;
        }

        all_rows.extend(parsed_page.data.diff);

        if let Some(total) = total
            && all_rows.len() >= total
        {
            break;
        }

        page_no += 1;
    }

    Ok(all_rows)
}

async fn fetch_market_snapshot_page_via_reqwest(
    client: &reqwest::Client,
    page_no: &str,
    page_size: &str,
) -> Result<EastMoneyBatchFundamentalResponse> {
    let response = client
        .get(EASTMONEY_CLIST_URL)
        .query(&[
            ("pn", page_no),
            ("pz", page_size),
            ("po", "1"),
            ("np", "1"),
            ("fltt", "2"),
            ("invt", "2"),
            ("fid", "f3"),
            ("fs", EASTMONEY_A_SHARE_FS),
            ("fields", "f12,f14,f2,f3,f5,f6"),
        ])
        .header("Referer", "https://data.eastmoney.com/")
        .send()
        .await
        .map_err(|err| QuantixError::Other(err.to_string()))?;

    response
        .json::<EastMoneyBatchFundamentalResponse>()
        .await
        .map_err(|err| QuantixError::Other(format!("解析全市场快照失败: {err}")))
}

fn fetch_market_snapshot_page_via_curl(
    page_no: &str,
    page_size: &str,
) -> Result<EastMoneyBatchFundamentalResponse> {
    let url = format!(
        "{EASTMONEY_CLIST_URL}?pn={page_no}&pz={page_size}&po=1&np=1&fltt=2&invt=2&fid=f3&fs=m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23&fields=f12,f14,f2,f3,f5,f6"
    );

    let output = Command::new("curl")
        .arg("-sS")
        .arg("-H")
        .arg("Referer: https://data.eastmoney.com/")
        .arg("-H")
        .arg("User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .arg(url)
        .output()
        .map_err(|err| QuantixError::Other(format!("curl 调用失败: {err}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(QuantixError::Other(format!(
            "curl 拉取全市场快照失败: {stderr}"
        )));
    }

    serde_json::from_slice::<EastMoneyBatchFundamentalResponse>(&output.stdout)
        .map_err(|err| QuantixError::Other(format!("解析 curl 全市场快照失败: {err}")))
}

fn stock_info_from_market_row(row: &EastMoneyBatchFundamentalItem) -> Option<StockInfo> {
    if row.price <= 0.0 {
        return None;
    }

    Some(StockInfo {
        code: row.code.clone(),
        name: row.name.clone(),
        price: row.price,
        change_pct: row.change_pct.unwrap_or(0.0),
        volume: row.volume.unwrap_or(0.0),
        amount: row.amount.unwrap_or(0.0),
        list_date: None,
    })
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

    let market_rows = fetch_a_share_market_snapshots_with_retry().await?;
    let stocks = market_rows
        .iter()
        .filter_map(stock_info_from_market_row)
        .collect();
    let store = SqliteIndustryStore::from_risk_state_path(risk_state_path).await?;
    let industry_rows = store
        .list_current(ACTIVE_CLASSIFICATION_STANDARD, ACTIVE_INDUSTRY_LEVEL)
        .await?;
    let foundation = build_market_analysis_foundation(stocks, industry_rows)?;
    let sector_rows = reader
        .load_board_rankings(
            BoardType::Sector,
            date,
            ALL_SECTOR_SCAN_LIMIT,
            BoardSortBy::ChangePct,
        )
        .await?;
    let requested_codes = foundation
        .rows
        .iter()
        .filter(|row| row.industry_name.is_some())
        .map(|row| row.code.clone())
        .collect::<Vec<_>>();
    let snapshot_batch = load_fundamental_snapshots_from_local_store(&requested_codes, date).await;

    Ok(build_market_strength_report(
        foundation,
        sector_rows,
        snapshot_batch,
        strong_top,
        weak_top,
        stock_top,
    ))
}

async fn load_fundamental_snapshots_from_local_store(
    codes: &[String],
    date: Option<NaiveDate>,
) -> FundamentalSnapshotBatch {
    if codes.is_empty() {
        return empty_fundamental_snapshot_batch(0, 0);
    }

    let client = match ClickHouseClient::with_default_config().await {
        Ok(client) => client,
        Err(err) => {
            warn!(
                "市场基础面本地表初始化失败，Top10 排序将返回空覆盖: {}",
                err
            );
            return empty_fundamental_snapshot_batch(codes.len(), codes.len());
        }
    };

    match client
        .get_latest_market_fundamental_snapshots(codes, date)
        .await
    {
        Ok(rows) => build_fundamental_snapshots_from_local_rows(rows),
        Err(err) => {
            warn!("市场基础面本地表查询失败，Top10 排序将返回空覆盖: {}", err);
            empty_fundamental_snapshot_batch(codes.len(), codes.len())
        }
    }
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

fn build_fundamental_snapshots_from_local_rows(
    rows: Vec<MarketFundamentalSnapshotCH>,
) -> FundamentalSnapshotBatch {
    let snapshots = rows
        .into_iter()
        .map(|row| FundamentalSnapshot {
            code: row.code,
            market_cap: row.market_cap.and_then(Decimal::from_f64_retain),
            latest_report_profit: row.latest_report_profit.and_then(Decimal::from_f64_retain),
        })
        .collect();

    FundamentalSnapshotBatch {
        snapshots,
        valuation_error_count: 0,
        earnings_error_count: 0,
    }
}

fn empty_fundamental_snapshot_batch(
    valuation_error_count: usize,
    earnings_error_count: usize,
) -> FundamentalSnapshotBatch {
    FundamentalSnapshotBatch {
        snapshots: Vec::new(),
        valuation_error_count,
        earnings_error_count,
    }
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
    fn local_fundamental_rows_map_into_snapshot_batch() {
        let batch = build_fundamental_snapshots_from_local_rows(vec![
            MarketFundamentalSnapshotCH {
                code: "600000".to_string(),
                snapshot_date: NaiveDate::from_ymd_opt(2026, 3, 14).unwrap(),
                market_cap: Some(500000.25),
                latest_report_profit: Some(8000.5),
                profit_source: "report".to_string(),
                pe_dynamic: Some(6.2),
                updated_at: "2026-03-14 15:12:16".to_string(),
            },
            MarketFundamentalSnapshotCH {
                code: "601398".to_string(),
                snapshot_date: NaiveDate::from_ymd_opt(2026, 3, 14).unwrap(),
                market_cap: Some(700000.0),
                latest_report_profit: Some(10000.0),
                profit_source: "report".to_string(),
                pe_dynamic: Some(7.1),
                updated_at: "2026-03-14 15:12:16".to_string(),
            },
        ]);

        assert_eq!(batch.snapshots.len(), 2);
        assert_eq!(batch.snapshots[0].code, "600000");
        assert_eq!(
            batch.snapshots[0].market_cap.unwrap().round_dp(2),
            Decimal::from_f64_retain(500000.25).unwrap()
        );
        assert_eq!(
            batch.snapshots[0].latest_report_profit.unwrap().round_dp(2),
            Decimal::from_f64_retain(8000.5).unwrap()
        );
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
            FundamentalSnapshotBatch {
                snapshots: vec![
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
                valuation_error_count: 0,
                earnings_error_count: 0,
            },
            2,
            1,
            2,
        );

        assert_eq!(report.strong_sectors.len(), 2);
        assert_eq!(report.strong_sectors[0].board_name, "计算机");
        assert_eq!(report.weak_sectors.len(), 1);
        assert_eq!(report.weak_sectors[0].board_name, "有色金属");
        assert_eq!(report.candidate_stock_count, 3);
        assert_eq!(report.market_cap_coverage_count, 3);
        assert_eq!(report.profit_coverage_count, 3);
        assert_eq!(report.top_by_market_cap.len(), 2);
        assert_eq!(report.top_by_market_cap[0].code, "601398");
        assert_eq!(report.top_by_profit.len(), 2);
        assert_eq!(report.top_by_profit[0].code, "601398");
    }

    #[test]
    fn strength_report_falls_back_to_industry_derived_rankings_when_sector_names_do_not_match() {
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
                BoardRankRow::new("BK0002", "白酒", BoardType::Sector, 1, 1.9),
                BoardRankRow::new("BK0003", "保险", BoardType::Sector, 2, 1.5),
            ],
            FundamentalSnapshotBatch {
                snapshots: vec![
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
                valuation_error_count: 0,
                earnings_error_count: 0,
            },
            2,
            1,
            2,
        );

        assert_eq!(report.strong_sectors.len(), 2);
        assert_eq!(report.strong_sectors[0].board_name, "计算机");
        assert_eq!(report.strong_sectors[1].board_name, "银行");
        assert_eq!(report.weak_sectors[0].board_name, "有色金属");
        assert_eq!(report.candidate_stock_count, 3);
        assert_eq!(report.market_cap_coverage_count, 3);
        assert_eq!(report.profit_coverage_count, 3);
        assert_eq!(report.top_by_market_cap[0].code, "601398");
        assert_eq!(report.top_by_profit[0].code, "601398");
    }
}
