use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::core::{QuantixError, Result};
use crate::market::{
    BoardRankRow, BoardType, LeaderFilter, LeaderRow, MarketSentimentSnapshot, NorthFlowSnapshot,
};

/// 股票基本信息 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct StockInfoCH {
    pub code: String,
    pub name: String,
    pub market: u8,
    pub list_date: chrono::NaiveDate,
    pub status: String,
    pub updated_at: chrono::NaiveDateTime,
}

/// 股票实时行情 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct StockQuoteCH {
    pub timestamp: u64,
    pub code: String,
    pub name: String,
    pub price: f64,
    pub preclose: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
    pub amount: f64,
    pub change_percent: f64,
    pub market: u8,
}

/// K线数据 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct KlineDataCH {
    pub timestamp: DateTime<Utc>,
    pub code: String,
    pub name: String,
    pub period: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
    pub trade_count: u32,
    pub source: String,
}

/// 涨停事件 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct LimitUpEventCH {
    pub limit_time: DateTime<Utc>,
    pub code: String,
    pub name: String,
    pub limit_type: String,
    pub open_price: f64,
    pub limit_price: f64,
    pub sealed_amount: f64,
    pub sealed_volume: f64,
    pub buy1_volume: f64,
    pub volume: f64,
    pub amount: f64,
    pub turnover_rate: f32,
    pub sector_name: String,
    pub is_first_board: u8,
    pub preclose: f64,
}

/// GBBQ 事件 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct GbbqEventCH {
    pub event_date: chrono::NaiveDate,
    pub code: String,
    pub category: u8,
    pub dividend: f32,
    pub bonus_price: f32,
    pub bonus_share: f32,
    pub rights_share: f32,
    pub ex_price: Option<f64>,
    pub record_date: Option<chrono::NaiveDate>,
}

/// 板块日线 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct SectorDailyCH {
    pub sector_code: String,
    pub sector_name: String,
    pub sector_type: String,
    pub trade_date: chrono::NaiveDate,
    pub change_pct: f64,
    pub rank: u32,
    pub leader_code: Option<String>,
    pub leader_name: Option<String>,
    pub leader_change: Option<f64>,
    pub updated_at: String,
}

impl SectorDailyCH {
    pub fn try_into_board_rank(self) -> Result<BoardRankRow> {
        Ok(BoardRankRow::new(
            self.sector_code,
            self.sector_name,
            parse_board_type(&self.sector_type)?,
            self.rank as usize,
            self.change_pct,
        ))
    }

    pub fn try_into_leader(self, filter: LeaderFilter) -> Result<Option<LeaderRow>> {
        let board_type = parse_board_type(&self.sector_type)?;
        let leader_code = match self.leader_code {
            Some(code) if !code.trim().is_empty() => code,
            _ => return Ok(None),
        };
        let leader_name = match self.leader_name {
            Some(name) if !name.trim().is_empty() => name,
            _ => return Ok(None),
        };
        let change_pct = self.leader_change.unwrap_or_default();

        let (sector_name, concept_name) = match filter {
            LeaderFilter::Sector(name) => (Some(name), None),
            LeaderFilter::Concept(name) => (None, Some(name)),
            LeaderFilter::All => match board_type {
                BoardType::Sector => (Some(self.sector_name), None),
                BoardType::Concept => (None, Some(self.sector_name)),
            },
        };

        Ok(Some(LeaderRow::new(
            leader_code,
            leader_name,
            sector_name,
            concept_name,
            change_pct,
        )))
    }
}

/// 北向资金日线 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct NorthFlowDailyCH {
    pub trade_date: chrono::NaiveDate,
    pub sh_amount: f64,
    pub sz_amount: f64,
    pub total_amount: f64,
    pub balance: f64,
    pub updated_at: String,
}

impl NorthFlowDailyCH {
    pub fn into_snapshot(self) -> NorthFlowSnapshot {
        NorthFlowSnapshot::new(
            self.trade_date,
            self.sh_amount,
            self.sz_amount,
            self.total_amount,
            self.balance,
        )
    }
}

/// 市场情绪日线 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct MarketSentimentDailyCH {
    pub trade_date: chrono::NaiveDate,
    pub up_count: u32,
    pub down_count: u32,
    pub limit_up_count: u32,
    pub limit_down_count: u32,
    pub seal_rate: f64,
    pub break_rate: f64,
    pub consecutive_board_count: u32,
    pub updated_at: String,
}

impl MarketSentimentDailyCH {
    pub fn into_snapshot(self) -> MarketSentimentSnapshot {
        MarketSentimentSnapshot::new(
            self.trade_date,
            self.up_count as usize,
            self.down_count as usize,
            self.limit_up_count as usize,
            self.limit_down_count as usize,
            self.seal_rate,
            self.break_rate,
            self.consecutive_board_count as usize,
        )
    }
}

/// 市场基础面快照 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct MarketFundamentalSnapshotCH {
    pub code: String,
    pub snapshot_date: chrono::NaiveDate,
    pub market_cap: Option<f64>,
    pub latest_report_profit: Option<f64>,
    pub profit_source: String,
    pub pe_dynamic: Option<f64>,
    pub updated_at: String,
}

/// 市场基础面快照写入行。
///
/// ClickHouse RowBinary `Date` / `DateTime` 需要使用自 1970-01-01 起的天数/秒数编码，
/// 不能直接复用查询侧的 `chrono::NaiveDate` / `String` 视图模型。
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct MarketFundamentalSnapshotInsertCH {
    pub code: String,
    pub snapshot_date: u16,
    pub market_cap: Option<f64>,
    pub latest_report_profit: Option<f64>,
    pub profit_source: String,
    pub pe_dynamic: Option<f64>,
    pub updated_at: u32,
}

pub(super) fn market_table_sqls() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "sector_daily",
            r#"
            CREATE TABLE IF NOT EXISTS sector_daily ON CLUSTER '{cluster}' (
                sector_code String,
                sector_name String,
                sector_type String,
                trade_date Date,
                change_pct Float64,
                rank UInt32,
                leader_code Nullable(String),
                leader_name Nullable(String),
                leader_change Nullable(Float64),
                updated_at DateTime DEFAULT now()
            )
            ENGINE = ReplacingMergeTree(updated_at)
            PARTITION BY toYYYYMM(trade_date)
            ORDER BY (trade_date, sector_type, rank, sector_code)
        "#,
        ),
        (
            "north_flow_daily",
            r#"
            CREATE TABLE IF NOT EXISTS north_flow_daily ON CLUSTER '{cluster}' (
                trade_date Date,
                sh_amount Float64,
                sz_amount Float64,
                total_amount Float64,
                balance Float64,
                updated_at DateTime DEFAULT now()
            )
            ENGINE = ReplacingMergeTree(updated_at)
            PARTITION BY toYYYYMM(trade_date)
            ORDER BY trade_date
        "#,
        ),
        (
            "market_sentiment_daily",
            r#"
            CREATE TABLE IF NOT EXISTS market_sentiment_daily ON CLUSTER '{cluster}' (
                trade_date Date,
                up_count UInt32,
                down_count UInt32,
                limit_up_count UInt32,
                limit_down_count UInt32,
                seal_rate Float64,
                break_rate Float64,
                consecutive_board_count UInt32,
                updated_at DateTime DEFAULT now()
            )
            ENGINE = ReplacingMergeTree(updated_at)
            PARTITION BY toYYYYMM(trade_date)
            ORDER BY trade_date
        "#,
        ),
        (
            "market_fundamentals_daily",
            r#"
            CREATE TABLE IF NOT EXISTS market_fundamentals_daily ON CLUSTER '{cluster}' (
                code String,
                snapshot_date Date,
                market_cap Nullable(Float64),
                latest_report_profit Nullable(Float64),
                profit_source String,
                pe_dynamic Nullable(Float64),
                updated_at DateTime DEFAULT now()
            )
            ENGINE = ReplacingMergeTree(updated_at)
            PARTITION BY toYYYYMM(snapshot_date)
            ORDER BY (snapshot_date, code)
        "#,
        ),
    ]
}

pub(super) fn parse_board_type(sector_type: &str) -> Result<BoardType> {
    match sector_type.trim().to_ascii_lowercase().as_str() {
        "industry" | "sector" => Ok(BoardType::Sector),
        "concept" => Ok(BoardType::Concept),
        other => Err(QuantixError::DataParse(format!(
            "未知的板块类型: {}",
            other
        ))),
    }
}
