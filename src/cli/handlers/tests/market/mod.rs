//! Market command tests, split by concern.
//!
//! Historical note: the original `market.rs` was 672 lines and was split
//! into submodules per the file-size limits in CLAUDE.md:
//! - [`queries`] — sector/concept/north/sentiment/leader/overview/foundation
//!   read-path tests,
//! - [`strength`] — strength + strength_stocks tests (incl. the sector-filter
//!   and date-format validation cases).
//!
//! The shared fakes (`FakeMarketReader`, `FakeMarketState`,
//! `MarketBoardRequest`, `MarketLeaderRequest`) are `pub(super)` and used by
//! both submodules, so they live here at the `market` module level.

pub(super) use super::*;

use async_trait::async_trait;
use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MarketBoardRequest {
    pub(super) board_type: BoardType,
    pub(super) date: Option<NaiveDate>,
    pub(super) limit: usize,
    pub(super) sort_by: BoardSortBy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MarketLeaderRequest {
    pub(super) filter: LeaderFilter,
    pub(super) limit: usize,
    pub(super) date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct FakeMarketState {
    pub(super) board_requests: Vec<MarketBoardRequest>,
    pub(super) leader_requests: Vec<MarketLeaderRequest>,
}

#[derive(Clone)]
pub(super) struct FakeMarketReader {
    state: Arc<Mutex<FakeMarketState>>,
}

impl FakeMarketReader {
    pub(super) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(FakeMarketState::default())),
        }
    }
}

#[async_trait]
impl MarketDataReader for FakeMarketReader {
    async fn load_board_rankings(
        &self,
        board_type: BoardType,
        date: Option<NaiveDate>,
        limit: usize,
        sort_by: BoardSortBy,
    ) -> Result<Vec<BoardRankRow>> {
        self.state
            .lock()
            .unwrap()
            .board_requests
            .push(MarketBoardRequest {
                board_type,
                date,
                limit,
                sort_by,
            });

        let rows = match board_type {
            BoardType::Sector => vec![BoardRankRow::new("BK001", "银行", board_type, 1, 2.1)],
            BoardType::Concept => {
                vec![BoardRankRow::new("GN001", "人工智能", board_type, 1, 4.2)]
            }
        };

        Ok(rows.into_iter().take(limit).collect())
    }

    async fn load_north_flow(&self, date: Option<NaiveDate>) -> Result<Option<NorthFlowSnapshot>> {
        Ok(Some(NorthFlowSnapshot::new(
            date.unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 3, 10).unwrap()),
            12.3,
            8.6,
            20.9,
            100.0,
        )))
    }

    async fn load_market_sentiment(
        &self,
        date: Option<NaiveDate>,
    ) -> Result<Option<MarketSentimentSnapshot>> {
        Ok(Some(MarketSentimentSnapshot::new(
            date.unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 3, 10).unwrap()),
            3210,
            1875,
            87,
            4,
            0.81,
            0.19,
            23,
        )))
    }

    async fn load_leaders(
        &self,
        filter: LeaderFilter,
        limit: usize,
        date: Option<NaiveDate>,
    ) -> Result<Vec<LeaderRow>> {
        self.state
            .lock()
            .unwrap()
            .leader_requests
            .push(MarketLeaderRequest {
                filter: filter.clone(),
                limit,
                date,
            });

        let rows = match filter {
            LeaderFilter::Sector(name) => {
                vec![LeaderRow::new("600000", "浦发银行", Some(name), None, 5.6)]
            }
            LeaderFilter::Concept(name) => {
                vec![LeaderRow::new("300024", "机器人", None, Some(name), 7.1)]
            }
            LeaderFilter::All => vec![
                LeaderRow::new("300024", "机器人", None, Some("人工智能".to_string()), 7.1),
                LeaderRow::new("600000", "浦发银行", Some("银行".to_string()), None, 5.6),
            ],
        };

        Ok(rows.into_iter().take(limit).collect())
    }
}

mod queries;
mod strength;
