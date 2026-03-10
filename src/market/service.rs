use async_trait::async_trait;
use chrono::NaiveDate;

use crate::core::Result;
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
