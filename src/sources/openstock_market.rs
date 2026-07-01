//! MarketDataReader 的 OpenStock 实现。
//!
//! 通过 OpenStockClient 的 `POST /data/fetch` 端点获取板块排名/情绪/北向等数据，
//! 替代原有的 ClickHouse 查询路径。
//!
//! 映射关系：
//!   `load_board_rankings` → SECTOR_QUOTES (akshare)
//!   `load_market_sentiment` → UPDOWN_DISTRIBUTION (zzshare)
//!   `load_north_flow` → NORTHBOUND_FLOW (akshare)
//!   `load_leaders` → SECTOR_QUOTES 子字段 (akshare)

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::core::{QuantixError, Result};
use crate::market::{
    BoardRankRow, BoardSortBy, BoardType, LeaderFilter, LeaderRow, MarketDataReader,
    MarketSentimentSnapshot, NorthFlowSnapshot,
};
use crate::sources::openstock_client::OpenStockClient;

/// 包装 OpenStockClient 实现 MarketDataReader
pub struct OpenStockMarketReader {
    client: OpenStockClient,
}

impl OpenStockMarketReader {
    pub fn new(client: OpenStockClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl MarketDataReader for OpenStockMarketReader {
    async fn load_board_rankings(
        &self,
        board_type: BoardType,
        _date: Option<NaiveDate>,
        limit: usize,
        _sort_by: BoardSortBy,
    ) -> Result<Vec<BoardRankRow>> {
        let sector_type = match board_type {
            BoardType::Sector => "industry",
            BoardType::Concept => "concept",
        };

        let resp = self
            .client
            .fetch::<serde_json::Value>(
                "SECTOR_QUOTES",
                serde_json::json!({
                    "sector_type": sector_type,
                }),
            )
            .await?;

        let rows: Vec<BoardRankRow> = resp
            .records
            .into_iter()
            .filter_map(|v| {
                let rank = v.get("rank").and_then(|r| r.as_i64())? as usize;
                let sector_code = v.get("sector_code").and_then(|c| c.as_str())?.to_string();
                let sector_name = v.get("sector_name").and_then(|n| n.as_str())?.to_string();
                let change_pct = v.get("change_pct").and_then(|p| p.as_f64())?;
                Some(BoardRankRow::new(
                    sector_code,
                    sector_name,
                    board_type,
                    rank,
                    change_pct,
                ))
            })
            .take(limit)
            .collect();

        Ok(rows)
    }

    async fn load_north_flow(&self, _date: Option<NaiveDate>) -> Result<Option<NorthFlowSnapshot>> {
        let resp = self
            .client
            .fetch::<serde_json::Value>("NORTHBOUND_FLOW", serde_json::json!({}))
            .await?;

        // 过滤北向（fund_direction="北向"），汇总沪/深合计
        let north_records: Vec<&serde_json::Value> = resp
            .records
            .iter()
            .filter(|v| {
                v.get("fund_direction")
                    .and_then(|d| d.as_str())
                    .map(|d| d == "北向")
                    .unwrap_or(false)
            })
            .collect();

        if north_records.is_empty() {
            return Ok(None);
        }

        // 取第一个北向记录中的 trade_date
        let trade_date_str = north_records[0]
            .get("trade_date")
            .and_then(|d| d.as_str())
            .unwrap_or("");
        let trade_date = NaiveDate::parse_from_str(trade_date_str, "%Y-%m-%d")
            .map_err(|e| QuantixError::DataParse(format!("解析北向资金日期失败: {}", e)))?;

        // 汇总 sh + sz 北向金额
        let mut total_amount = 0.0_f64;
        let mut sh_amount = 0.0_f64;
        let mut sz_amount = 0.0_f64;
        let mut balance = 0.0_f64;
        let mut has_data = false;

        for record in &north_records {
            let board_name = record
                .get("board_name")
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let net_buy = record
                .get("net_buy_amount")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let net_inflow = record
                .get("fund_net_inflow")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            if board_name.contains("沪") {
                sh_amount = net_buy;
                balance += net_inflow;
                total_amount += net_buy;
                has_data = true;
            } else if board_name.contains("深") {
                sz_amount = net_buy;
                balance += net_inflow;
                total_amount += net_buy;
                has_data = true;
            }
        }

        if !has_data {
            return Ok(None);
        }

        Ok(Some(NorthFlowSnapshot::new(
            trade_date,
            sh_amount,
            sz_amount,
            total_amount,
            balance,
        )))
    }

    async fn load_market_sentiment(
        &self,
        date: Option<NaiveDate>,
    ) -> Result<Option<MarketSentimentSnapshot>> {
        // UPDOWN_DISTRIBUTION 必须传 trade_date，默认取当天
        let trade_date = date
            .unwrap_or_else(|| chrono::Local::now().date_naive())
            .format("%Y-%m-%d")
            .to_string();

        let resp = self
            .client
            .fetch::<serde_json::Value>(
                "UPDOWN_DISTRIBUTION",
                serde_json::json!({
                    "trade_date": trade_date,
                }),
            )
            .await?;

        let record = match resp.records.into_iter().next() {
            Some(r) => r,
            None => return Ok(None),
        };

        let trade_date_str = record
            .get("trade_date")
            .and_then(|d| d.as_str())
            .unwrap_or("");
        let trade_date = NaiveDate::parse_from_str(trade_date_str, "%Y-%m-%d")
            .map_err(|e| QuantixError::DataParse(format!("解析情绪日期失败: {}", e)))?;

        let up_count = parse_number_str_or_int(&record, "up_count") as usize;
        let down_count = parse_number_str_or_int(&record, "down_count") as usize;
        let limit_up_count = parse_number_str_or_int(&record, "limit_up_count") as usize;
        let limit_down_count = parse_number_str_or_int(&record, "limit_down_count") as usize;

        Ok(Some(MarketSentimentSnapshot::new(
            trade_date,
            up_count,
            down_count,
            limit_up_count,
            limit_down_count,
            0.0, // seal_rate — 需从 LIMIT_UP_POOL 计算
            0.0, // break_rate — 需从 LIMIT_UP_POOL 计算
            0,   // consecutive_board_count
        )))
    }

    async fn load_leaders(
        &self,
        filter: LeaderFilter,
        limit: usize,
        _date: Option<NaiveDate>,
    ) -> Result<Vec<LeaderRow>> {
        let sector_type = match &filter {
            LeaderFilter::Sector(_) => "industry",
            LeaderFilter::Concept(_) => "concept",
            LeaderFilter::All => "industry", // 默认取行业
        };

        let resp = self
            .client
            .fetch::<serde_json::Value>(
                "SECTOR_QUOTES",
                serde_json::json!({
                    "sector_type": sector_type,
                }),
            )
            .await?;

        let mut leaders: Vec<LeaderRow> = Vec::new();

        for record in resp.records.iter() {
            let leading_name = match record.get("leading_name").and_then(|n| n.as_str()) {
                Some(name) if !name.is_empty() => name.to_string(),
                _ => continue,
            };

            let sector_code = record
                .get("sector_code")
                .and_then(|c| c.as_str())
                .unwrap_or("");
            let sector_name = record
                .get("sector_name")
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let leading_change = record
                .get("leading_change_pct")
                .and_then(|p| p.as_f64())
                .unwrap_or(0.0);

            // 根据 filter 类型设置板块/概念名
            let (sector, concept) = match &filter {
                LeaderFilter::Sector(_) => (Some(sector_name.to_string()), None),
                LeaderFilter::Concept(_) => (None, Some(sector_name.to_string())),
                LeaderFilter::All => match sector_type {
                    "industry" => (Some(sector_name.to_string()), None),
                    _ => (None, Some(sector_name.to_string())),
                },
            };

            // 去重：如果已经加入过该股票（同一只股票可以同时是一个板块的龙头）
            if leaders.iter().any(|l| l.name == leading_name) {
                continue;
            }

            leaders.push(LeaderRow::new(
                sector_code,
                leading_name,
                sector,
                concept,
                leading_change,
            ));

            if leaders.len() >= limit {
                break;
            }
        }

        Ok(leaders)
    }
}

/// 从 JSON value 中解析数字（支持字符串 "2899" 或数字 2899）
fn parse_number_str_or_int(value: &serde_json::Value, key: &str) -> i64 {
    value
        .get(key)
        .and_then(|v| {
            v.as_str()
                .and_then(|s| s.parse::<i64>().ok())
                .or_else(|| v.as_i64())
        })
        .unwrap_or(0)
}
