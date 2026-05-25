use std::path::Path;
use std::process::Command;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use tokio::time::{Duration, sleep};
use tracing::warn;

use crate::core::{QuantixError, Result};
use crate::db::clickhouse::{ClickHouseClient, MarketFundamentalSnapshotCH};
use crate::market::strength::{
    FundamentalSnapshot, FundamentalSnapshotBatch, MarketAnalysisFoundation,
    MarketIndustryClassificationRow, MarketSnapshotRow, MarketStrengthReport,
    build_market_analysis_foundation, build_market_strength_report,
};
use crate::market::{BoardSortBy, BoardType, MarketDataReader};
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

pub async fn load_market_analysis_foundation(
    risk_state_path: impl AsRef<Path>,
) -> Result<MarketAnalysisFoundation> {
    let market_rows = fetch_a_share_market_snapshots_with_retry().await?;
    let stocks = market_rows
        .iter()
        .filter_map(market_snapshot_from_market_row)
        .collect();
    let store = SqliteIndustryStore::from_risk_state_path(risk_state_path).await?;
    let industry_rows = store
        .list_current(ACTIVE_CLASSIFICATION_STANDARD, ACTIVE_INDUSTRY_LEVEL)
        .await?;

    build_market_analysis_foundation(stocks, industry_rows_from_reference_records(industry_rows))
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
        .filter_map(market_snapshot_from_market_row)
        .collect();
    let store = SqliteIndustryStore::from_risk_state_path(risk_state_path).await?;
    let industry_rows = store
        .list_current(ACTIVE_CLASSIFICATION_STANDARD, ACTIVE_INDUSTRY_LEVEL)
        .await?;
    let foundation = build_market_analysis_foundation(
        stocks,
        industry_rows_from_reference_records(industry_rows),
    )?;
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

fn market_snapshot_from_market_row(
    row: &EastMoneyBatchFundamentalItem,
) -> Option<MarketSnapshotRow> {
    if row.price <= 0.0 {
        return None;
    }

    Some(MarketSnapshotRow {
        code: row.code.clone(),
        name: row.name.clone(),
        price: row.price,
        change_pct: row.change_pct.unwrap_or(0.0),
        volume: row.volume.unwrap_or(0.0),
        amount: row.amount.unwrap_or(0.0),
    })
}

fn industry_rows_from_reference_records(
    rows: Vec<IndustryReferenceRecord>,
) -> Vec<MarketIndustryClassificationRow> {
    rows.into_iter()
        .map(|row| MarketIndustryClassificationRow {
            code: row.code,
            industry_name: row.industry_name,
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
