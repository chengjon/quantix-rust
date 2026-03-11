use super::*;
use super::support::{FakeMarketReader, MarketBoardRequest, MarketLeaderRequest};

#[tokio::test]
async fn test_execute_market_sector_returns_rows() {
    let reader = FakeMarketReader::new();

    let output = execute_market_command_with_reader(
        MarketCommands::Sector {
            top: Some(1),
            date: Some("2026-03-09".to_string()),
            sort_by: Some("change".to_string()),
        },
        reader.clone(),
    )
    .await
    .unwrap();

    match output {
        MarketCommandOutput::BoardRows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].board_name, "银行");
        }
        other => panic!("unexpected output: {:?}", other),
    }

    let state = reader.state.lock().unwrap();
    assert_eq!(
        state.board_requests,
        vec![MarketBoardRequest {
            board_type: BoardType::Sector,
            date: Some(NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()),
            limit: 1,
            sort_by: BoardSortBy::ChangePct,
        }]
    );
}

#[tokio::test]
async fn test_execute_market_concept_returns_rows() {
    let output = execute_market_command_with_reader(
        MarketCommands::Concept {
            top: Some(1),
            date: None,
            sort_by: None,
        },
        FakeMarketReader::new(),
    )
    .await
    .unwrap();

    match output {
        MarketCommandOutput::BoardRows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].board_name, "人工智能");
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_market_north_returns_snapshot() {
    let output = execute_market_command_with_reader(
        MarketCommands::North {
            date: Some("2026-03-09".to_string()),
        },
        FakeMarketReader::new(),
    )
    .await
    .unwrap();

    match output {
        MarketCommandOutput::NorthFlow(Some(snapshot)) => {
            assert_eq!(snapshot.trade_date, NaiveDate::from_ymd_opt(2026, 3, 9).unwrap());
            assert_eq!(snapshot.total_amount, 20.9);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_market_sentiment_returns_snapshot() {
    let output = execute_market_command_with_reader(
        MarketCommands::Sentiment { date: None },
        FakeMarketReader::new(),
    )
    .await
    .unwrap();

    match output {
        MarketCommandOutput::Sentiment(Some(snapshot)) => {
            assert_eq!(snapshot.limit_up_count, 87);
            assert_eq!(snapshot.consecutive_board_count, 23);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_market_leader_with_sector_returns_rows() {
    let reader = FakeMarketReader::new();

    let output = execute_market_command_with_reader(
        MarketCommands::Leader {
            sector: Some("银行".to_string()),
            concept: None,
            all: false,
            limit: Some(5),
            date: Some("2026-03-09".to_string()),
        },
        reader.clone(),
    )
    .await
    .unwrap();

    match output {
        MarketCommandOutput::Leaders(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].code, "600000");
        }
        other => panic!("unexpected output: {:?}", other),
    }

    let state = reader.state.lock().unwrap();
    assert_eq!(
        state.leader_requests,
        vec![MarketLeaderRequest {
            filter: LeaderFilter::Sector("银行".to_string()),
            limit: 5,
            date: Some(NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()),
        }]
    );
}

#[tokio::test]
async fn test_execute_market_overview_returns_combined_payload() {
    let output = execute_market_command_with_reader(
        MarketCommands::Overview {
            top: Some(1),
            date: None,
        },
        FakeMarketReader::new(),
    )
    .await
    .unwrap();

    match output {
        MarketCommandOutput::Overview(overview) => {
            assert_eq!(overview.top_sectors.len(), 1);
            assert_eq!(overview.top_concepts.len(), 1);
            assert_eq!(overview.north_flow.unwrap().total_amount, 20.9);
            assert_eq!(overview.sentiment.unwrap().limit_up_count, 87);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_market_leader_rejects_invalid_filter_combination() {
    let err = execute_market_command_with_reader(
        MarketCommands::Leader {
            sector: Some("银行".to_string()),
            concept: Some("人工智能".to_string()),
            all: false,
            limit: None,
            date: None,
        },
        FakeMarketReader::new(),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Other(_)));
    assert!(err.to_string().contains("必须且只能指定"));
}
