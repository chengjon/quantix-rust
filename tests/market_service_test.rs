use async_trait::async_trait;
use chrono::NaiveDate;
use quantix_cli::core::Result;
use quantix_cli::market::{
    BoardRankRow, BoardSortBy, BoardType, LeaderFilter, LeaderRow, MarketDataReader,
    MarketOverview, MarketSentimentSnapshot, MarketService, NorthFlowSnapshot,
};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoardRequest {
    board_type: BoardType,
    date: Option<NaiveDate>,
    limit: usize,
    sort_by: BoardSortBy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LeaderRequest {
    filter: LeaderFilter,
    limit: usize,
    date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Default)]
struct FakeState {
    board_requests: Vec<BoardRequest>,
    leader_requests: Vec<LeaderRequest>,
}

#[derive(Clone)]
struct FakeReader {
    state: Arc<Mutex<FakeState>>,
}

impl FakeReader {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(FakeState::default())),
        }
    }
}

#[async_trait]
impl MarketDataReader for FakeReader {
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
            .push(BoardRequest {
                board_type,
                date,
                limit,
                sort_by,
            });

        let rows = match board_type {
            BoardType::Sector => vec![
                BoardRankRow::new("BK001", "银行", board_type, 1, 2.1),
                BoardRankRow::new("BK002", "证券", board_type, 2, 1.5),
            ],
            BoardType::Concept => vec![
                BoardRankRow::new("GN001", "人工智能", board_type, 1, 4.2),
                BoardRankRow::new("GN002", "机器人", board_type, 2, 3.8),
            ],
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
            .push(LeaderRequest {
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

#[tokio::test]
async fn returns_sector_rankings_with_requested_limit_and_sort() {
    let reader = FakeReader::new();
    let service = MarketService::new(reader.clone());

    let rows = service
        .get_board_rankings(BoardType::Sector, None, Some(1), BoardSortBy::ChangePct)
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].board_name, "银行");
    let state = reader.state.lock().unwrap();
    assert_eq!(state.board_requests.len(), 1);
    assert_eq!(state.board_requests[0].board_type, BoardType::Sector);
    assert_eq!(state.board_requests[0].limit, 1);
    assert_eq!(state.board_requests[0].sort_by, BoardSortBy::ChangePct);
}

#[tokio::test]
async fn returns_concept_rankings_from_same_service_path() {
    let service = MarketService::new(FakeReader::new());

    let rows = service
        .get_board_rankings(BoardType::Concept, None, Some(2), BoardSortBy::ChangePct)
        .await
        .unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].board_name, "人工智能");
    assert_eq!(rows[1].board_name, "机器人");
}

#[tokio::test]
async fn returns_north_flow_snapshot() {
    let service = MarketService::new(FakeReader::new());

    let snapshot = service
        .get_north_flow(Some(NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        snapshot.trade_date,
        NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()
    );
    assert_eq!(snapshot.total_amount, 20.9);
}

#[tokio::test]
async fn returns_market_sentiment_snapshot() {
    let service = MarketService::new(FakeReader::new());

    let snapshot = service.get_market_sentiment(None).await.unwrap().unwrap();

    assert_eq!(snapshot.limit_up_count, 87);
    assert_eq!(snapshot.consecutive_board_count, 23);
}

#[tokio::test]
async fn returns_sector_leaders() {
    let service = MarketService::new(FakeReader::new());

    let rows = service
        .get_leaders(LeaderFilter::Sector("银行".to_string()), Some(10), None)
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].code, "600000");
}

#[tokio::test]
async fn returns_concept_leaders() {
    let service = MarketService::new(FakeReader::new());

    let rows = service
        .get_leaders(
            LeaderFilter::Concept("人工智能".to_string()),
            Some(10),
            None,
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].code, "300024");
}

#[tokio::test]
async fn builds_market_overview_from_component_queries() {
    let service = MarketService::new(FakeReader::new());

    let overview = service.get_overview(None, Some(1)).await.unwrap();

    assert_eq!(overview.top_sectors.len(), 1);
    assert_eq!(overview.top_concepts.len(), 1);
    assert_eq!(overview.north_flow.unwrap().total_amount, 20.9);
    assert_eq!(overview.sentiment.unwrap().limit_up_count, 87);
}

#[tokio::test]
async fn empty_inputs_return_readable_empty_results() {
    #[derive(Clone)]
    struct EmptyReader;

    #[async_trait]
    impl MarketDataReader for EmptyReader {
        async fn load_board_rankings(
            &self,
            _board_type: BoardType,
            _date: Option<NaiveDate>,
            _limit: usize,
            _sort_by: BoardSortBy,
        ) -> Result<Vec<BoardRankRow>> {
            Ok(Vec::new())
        }

        async fn load_north_flow(
            &self,
            _date: Option<NaiveDate>,
        ) -> Result<Option<NorthFlowSnapshot>> {
            Ok(None)
        }

        async fn load_market_sentiment(
            &self,
            _date: Option<NaiveDate>,
        ) -> Result<Option<MarketSentimentSnapshot>> {
            Ok(None)
        }

        async fn load_leaders(
            &self,
            _filter: LeaderFilter,
            _limit: usize,
            _date: Option<NaiveDate>,
        ) -> Result<Vec<LeaderRow>> {
            Ok(Vec::new())
        }
    }

    let service = MarketService::new(EmptyReader);

    let sector_rows = service
        .get_board_rankings(BoardType::Sector, None, None, BoardSortBy::ChangePct)
        .await
        .unwrap();
    let leaders = service
        .get_leaders(LeaderFilter::All, None, None)
        .await
        .unwrap();
    let overview = service.get_overview(None, None).await.unwrap();

    assert!(sector_rows.is_empty());
    assert!(leaders.is_empty());
    assert_eq!(
        overview,
        MarketOverview {
            top_sectors: Vec::new(),
            top_concepts: Vec::new(),
            north_flow: None,
            sentiment: None,
        }
    );
}
