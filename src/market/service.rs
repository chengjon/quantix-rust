use async_trait::async_trait;
use chrono::NaiveDate;
use std::collections::HashSet;

use crate::core::{QuantixError, Result};
use crate::db::clickhouse::{
    ClickHouseClient, MarketSentimentDailyCH, NorthFlowDailyCH, SectorDailyCH,
};
use crate::market::models::{
    BoardRankRow, BoardSortBy, BoardType, LeaderFilter, LeaderRow, MarketOverview,
    MarketSentimentSnapshot, NorthFlowSnapshot,
};

const DEFAULT_BOARD_LIMIT: usize = 10;
const DEFAULT_LEADER_LIMIT: usize = 10;
const DEFAULT_OVERVIEW_TOP: usize = 5;

#[async_trait]
pub trait MarketDataReader: Send + Sync {
    async fn load_board_rankings(
        &self,
        board_type: BoardType,
        date: Option<NaiveDate>,
        limit: usize,
        sort_by: BoardSortBy,
    ) -> Result<Vec<BoardRankRow>>;

    async fn load_north_flow(&self, date: Option<NaiveDate>) -> Result<Option<NorthFlowSnapshot>>;

    async fn load_market_sentiment(
        &self,
        date: Option<NaiveDate>,
    ) -> Result<Option<MarketSentimentSnapshot>>;

    async fn load_leaders(
        &self,
        filter: LeaderFilter,
        limit: usize,
        date: Option<NaiveDate>,
    ) -> Result<Vec<LeaderRow>>;
}

#[derive(Debug, Clone)]
pub struct MarketService<R> {
    reader: R,
}

impl<R> MarketService<R>
where
    R: MarketDataReader,
{
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub async fn get_board_rankings(
        &self,
        board_type: BoardType,
        date: Option<NaiveDate>,
        limit: Option<usize>,
        sort_by: BoardSortBy,
    ) -> Result<Vec<BoardRankRow>> {
        self.reader
            .load_board_rankings(
                board_type,
                date,
                limit.unwrap_or(DEFAULT_BOARD_LIMIT),
                sort_by,
            )
            .await
    }

    pub async fn get_north_flow(
        &self,
        date: Option<NaiveDate>,
    ) -> Result<Option<NorthFlowSnapshot>> {
        self.reader.load_north_flow(date).await
    }

    pub async fn get_market_sentiment(
        &self,
        date: Option<NaiveDate>,
    ) -> Result<Option<MarketSentimentSnapshot>> {
        self.reader.load_market_sentiment(date).await
    }

    pub async fn get_leaders(
        &self,
        filter: LeaderFilter,
        limit: Option<usize>,
        date: Option<NaiveDate>,
    ) -> Result<Vec<LeaderRow>> {
        self.reader
            .load_leaders(filter, limit.unwrap_or(DEFAULT_LEADER_LIMIT), date)
            .await
    }

    pub async fn get_overview(
        &self,
        date: Option<NaiveDate>,
        top: Option<usize>,
    ) -> Result<MarketOverview> {
        let limit = top.unwrap_or(DEFAULT_OVERVIEW_TOP);
        let top_sectors = self
            .reader
            .load_board_rankings(BoardType::Sector, date, limit, BoardSortBy::ChangePct)
            .await?;
        let top_concepts = self
            .reader
            .load_board_rankings(BoardType::Concept, date, limit, BoardSortBy::ChangePct)
            .await?;
        let north_flow = self.reader.load_north_flow(date).await?;
        let sentiment = self.reader.load_market_sentiment(date).await?;

        Ok(MarketOverview {
            top_sectors,
            top_concepts,
            north_flow,
            sentiment,
        })
    }
}

#[async_trait]
impl MarketDataReader for ClickHouseClient {
    async fn load_board_rankings(
        &self,
        board_type: BoardType,
        date: Option<NaiveDate>,
        limit: usize,
        sort_by: BoardSortBy,
    ) -> Result<Vec<BoardRankRow>> {
        let sector_type = board_type_label(board_type);
        let date_clause = latest_date_clause(
            "sector_daily",
            "trade_date",
            date,
            Some(format!("sector_type = '{}'", sector_type)),
        );
        let sql = format!(
            r#"
            SELECT
                sector_code,
                sector_name,
                sector_type,
                trade_date,
                change_pct,
                rank,
                leader_code,
                leader_name,
                leader_change,
                updated_at
            FROM sector_daily
            WHERE sector_type = '{sector_type}' AND {date_clause}
            ORDER BY {sort_expr}, rank ASC, sector_code ASC
            LIMIT {limit}
            "#,
            sector_type = sector_type,
            date_clause = date_clause,
            sort_expr = board_sort_expr(sort_by),
            limit = limit,
        );

        let rows: Vec<SectorDailyCH> = self
            .client()
            .query(&sql)
            .fetch_all()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("查询板块排名失败: {}", e)))?;

        rows.into_iter()
            .map(SectorDailyCH::try_into_board_rank)
            .collect()
    }

    async fn load_north_flow(&self, date: Option<NaiveDate>) -> Result<Option<NorthFlowSnapshot>> {
        let date_clause = latest_date_clause("north_flow_daily", "trade_date", date, None);
        let sql = format!(
            r#"
            SELECT
                trade_date,
                sh_amount,
                sz_amount,
                total_amount,
                balance,
                updated_at
            FROM north_flow_daily
            WHERE {date_clause}
            ORDER BY trade_date DESC
            LIMIT 1
            "#,
            date_clause = date_clause,
        );

        let row = self
            .client()
            .query(&sql)
            .fetch_all::<NorthFlowDailyCH>()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("查询北向资金失败: {}", e)))?
            .into_iter()
            .next();

        Ok(row.map(NorthFlowDailyCH::into_snapshot))
    }

    async fn load_market_sentiment(
        &self,
        date: Option<NaiveDate>,
    ) -> Result<Option<MarketSentimentSnapshot>> {
        let date_clause = latest_date_clause("market_sentiment_daily", "trade_date", date, None);
        let sql = format!(
            r#"
            SELECT
                trade_date,
                up_count,
                down_count,
                limit_up_count,
                limit_down_count,
                seal_rate,
                break_rate,
                consecutive_board_count,
                updated_at
            FROM market_sentiment_daily
            WHERE {date_clause}
            ORDER BY trade_date DESC
            LIMIT 1
            "#,
            date_clause = date_clause,
        );

        let row = self
            .client()
            .query(&sql)
            .fetch_all::<MarketSentimentDailyCH>()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("查询市场情绪失败: {}", e)))?
            .into_iter()
            .next();

        Ok(row.map(MarketSentimentDailyCH::into_snapshot))
    }

    async fn load_leaders(
        &self,
        filter: LeaderFilter,
        limit: usize,
        date: Option<NaiveDate>,
    ) -> Result<Vec<LeaderRow>> {
        let board_filter = leader_filter_clause(&filter);
        let date_clause = latest_date_clause(
            "sector_daily",
            "trade_date",
            date,
            Some(board_filter.clone()),
        );
        let sql = format!(
            r#"
            SELECT
                sector_code,
                sector_name,
                sector_type,
                trade_date,
                change_pct,
                rank,
                leader_code,
                leader_name,
                leader_change,
                updated_at
            FROM sector_daily
            WHERE {board_filter} AND {date_clause}
            ORDER BY leader_change DESC, rank ASC, sector_code ASC
            LIMIT {limit}
            "#,
            board_filter = board_filter,
            date_clause = date_clause,
            limit = limit,
        );

        let rows: Vec<SectorDailyCH> = self
            .client()
            .query(&sql)
            .fetch_all()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("查询龙头股失败: {}", e)))?;

        let mut seen = HashSet::new();
        let mut leaders = Vec::new();
        for row in rows {
            if let Some(leader) = row.try_into_leader(filter.clone())? {
                if seen.insert(leader.code.clone()) {
                    leaders.push(leader);
                }
            }
        }

        Ok(leaders)
    }
}

fn board_type_label(board_type: BoardType) -> &'static str {
    match board_type {
        BoardType::Sector => "industry",
        BoardType::Concept => "concept",
    }
}

fn board_sort_expr(sort_by: BoardSortBy) -> &'static str {
    match sort_by {
        BoardSortBy::ChangePct => "change_pct DESC",
    }
}

fn latest_date_clause(
    table: &str,
    date_column: &str,
    date: Option<NaiveDate>,
    extra_filter: Option<String>,
) -> String {
    match date {
        Some(date) => format!("{} = '{}'", date_column, date),
        None => {
            let subquery_filter = extra_filter
                .map(|filter| format!(" WHERE {}", filter))
                .unwrap_or_default();
            format!(
                "{} = (SELECT max({}) FROM {}{})",
                date_column, date_column, table, subquery_filter
            )
        }
    }
}

fn leader_filter_clause(filter: &LeaderFilter) -> String {
    match filter {
        LeaderFilter::Sector(name) => format!(
            "sector_type = 'industry' AND sector_name = '{}'",
            escape_sql_literal(name)
        ),
        LeaderFilter::Concept(name) => format!(
            "sector_type = 'concept' AND sector_name = '{}'",
            escape_sql_literal(name)
        ),
        LeaderFilter::All => "leader_code IS NOT NULL".to_string(),
    }
}

fn escape_sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}
