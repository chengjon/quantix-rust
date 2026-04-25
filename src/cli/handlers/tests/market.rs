use super::*;
use crate::market::SectorCoverageRow;

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarketBoardRequest {
    board_type: BoardType,
    date: Option<NaiveDate>,
    limit: usize,
    sort_by: BoardSortBy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarketLeaderRequest {
    filter: LeaderFilter,
    limit: usize,
    date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Default)]
struct FakeMarketState {
    board_requests: Vec<MarketBoardRequest>,
    leader_requests: Vec<MarketLeaderRequest>,
}

#[derive(Clone)]
struct FakeMarketReader {
    state: Arc<Mutex<FakeMarketState>>,
}

impl FakeMarketReader {
    fn new() -> Self {
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
            assert_eq!(
                snapshot.trade_date,
                NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()
            );
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
async fn test_execute_market_foundation_returns_injected_summary() {
    let output = execute_market_command_with_test_payloads(
        MarketCommands::Foundation,
        FakeMarketReader::new(),
        Some(MarketFoundationSummary {
            total_stocks: 5300,
            classified_stocks: 5200,
            unclassified_stocks: 100,
            sector_count: 31,
            top_sectors: vec![SectorCoverageRow {
                industry_name: "银行".to_string(),
                stock_count: 42,
            }],
        }),
        None,
    )
    .await
    .unwrap();

    match output {
        MarketCommandOutput::Foundation(summary) => {
            assert_eq!(summary.total_stocks, 5300);
            assert_eq!(summary.classified_stocks, 5200);
            assert_eq!(summary.top_sectors[0].industry_name, "银行");
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_market_strength_returns_injected_report() {
    let output = execute_market_command_with_test_payloads(
        MarketCommands::Strength {
            date: Some("2026-03-09".to_string()),
            strong_top: 3,
            weak_top: 3,
            stock_top: 10,
        },
        FakeMarketReader::new(),
        None,
        Some(MarketStrengthReport {
            foundation: MarketFoundationSummary {
                total_stocks: 5300,
                classified_stocks: 5200,
                unclassified_stocks: 100,
                sector_count: 31,
                top_sectors: vec![],
            },
            strong_sectors: vec![BoardRankRow::new(
                "BK001",
                "计算机",
                BoardType::Sector,
                1,
                4.1,
            )],
            weak_sectors: vec![BoardRankRow::new(
                "BK999",
                "有色金属",
                BoardType::Sector,
                1,
                -1.8,
            )],
            top_by_market_cap: vec![StrongSectorStockRow {
                sector_name: "计算机".to_string(),
                code: "300024".to_string(),
                name: "机器人".to_string(),
                latest_price: 15.0,
                latest_change_pct: 4.2,
                market_cap: Some(Decimal::new(200000, 2)),
                latest_report_profit: Some(Decimal::new(3000, 2)),
            }],
            top_by_profit: vec![StrongSectorStockRow {
                sector_name: "银行".to_string(),
                code: "601398".to_string(),
                name: "工商银行".to_string(),
                latest_price: 7.0,
                latest_change_pct: 1.5,
                market_cap: Some(Decimal::new(700000, 2)),
                latest_report_profit: Some(Decimal::new(10000, 2)),
            }],
            candidate_stock_count: 12,
            market_cap_coverage_count: 8,
            profit_coverage_count: 6,
            valuation_error_count: 0,
            earnings_error_count: 0,
        }),
    )
    .await
    .unwrap();

    match output {
        MarketCommandOutput::Strength(report) => {
            assert_eq!(report.strong_sectors[0].board_name, "计算机");
            assert_eq!(report.weak_sectors[0].board_name, "有色金属");
            assert_eq!(report.top_by_market_cap[0].code, "300024");
            assert_eq!(report.top_by_profit[0].code, "601398");
            assert_eq!(report.candidate_stock_count, 12);
            assert_eq!(report.market_cap_coverage_count, 8);
            assert_eq!(report.profit_coverage_count, 6);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_market_strength_stocks_returns_profit_ranking() {
    let output = execute_market_command_with_test_payloads(
        MarketCommands::StrengthStocks {
            date: Some("2026-03-09".to_string()),
            strong_top: 3,
            sector: None,
            metric: StrengthStockMetric::Profit,
            top: 5,
        },
        FakeMarketReader::new(),
        None,
        Some(MarketStrengthReport {
            foundation: MarketFoundationSummary {
                total_stocks: 5300,
                classified_stocks: 5200,
                unclassified_stocks: 100,
                sector_count: 31,
                top_sectors: vec![],
            },
            strong_sectors: vec![],
            weak_sectors: vec![],
            top_by_market_cap: vec![StrongSectorStockRow {
                sector_name: "计算机".to_string(),
                code: "300024".to_string(),
                name: "机器人".to_string(),
                latest_price: 15.0,
                latest_change_pct: 4.2,
                market_cap: Some(Decimal::new(200000, 2)),
                latest_report_profit: Some(Decimal::new(3000, 2)),
            }],
            top_by_profit: vec![StrongSectorStockRow {
                sector_name: "银行".to_string(),
                code: "601398".to_string(),
                name: "工商银行".to_string(),
                latest_price: 7.0,
                latest_change_pct: 1.5,
                market_cap: Some(Decimal::new(700000, 2)),
                latest_report_profit: Some(Decimal::new(10000, 2)),
            }],
            candidate_stock_count: 12,
            market_cap_coverage_count: 8,
            profit_coverage_count: 6,
            valuation_error_count: 0,
            earnings_error_count: 0,
        }),
    )
    .await
    .unwrap();

    match output {
        MarketCommandOutput::StrengthStocks(ranking) => {
            assert_eq!(ranking.metric, StrengthStockMetric::Profit);
            assert_eq!(ranking.strong_top, 3);
            assert_eq!(ranking.sector_filter, None);
            assert_eq!(ranking.candidate_stock_count, 12);
            assert_eq!(ranking.covered_count, 6);
            assert_eq!(ranking.rows.len(), 1);
            assert_eq!(ranking.rows[0].code, "601398");
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_market_strength_stocks_filters_selected_sector() {
    let output = execute_market_command_with_test_payloads(
        MarketCommands::StrengthStocks {
            date: Some("2026-03-09".to_string()),
            strong_top: 3,
            sector: Some("银行".to_string()),
            metric: StrengthStockMetric::MarketCap,
            top: 10,
        },
        FakeMarketReader::new(),
        None,
        Some(MarketStrengthReport {
            foundation: MarketFoundationSummary {
                total_stocks: 5300,
                classified_stocks: 5200,
                unclassified_stocks: 100,
                sector_count: 31,
                top_sectors: vec![],
            },
            strong_sectors: vec![],
            weak_sectors: vec![],
            top_by_market_cap: vec![
                StrongSectorStockRow {
                    sector_name: "银行".to_string(),
                    code: "601398".to_string(),
                    name: "工商银行".to_string(),
                    latest_price: 7.0,
                    latest_change_pct: 1.5,
                    market_cap: Some(Decimal::new(700000, 2)),
                    latest_report_profit: Some(Decimal::new(10000, 2)),
                },
                StrongSectorStockRow {
                    sector_name: "计算机".to_string(),
                    code: "300024".to_string(),
                    name: "机器人".to_string(),
                    latest_price: 15.0,
                    latest_change_pct: 4.2,
                    market_cap: Some(Decimal::new(200000, 2)),
                    latest_report_profit: Some(Decimal::new(3000, 2)),
                },
            ],
            top_by_profit: vec![],
            candidate_stock_count: 12,
            market_cap_coverage_count: 8,
            profit_coverage_count: 6,
            valuation_error_count: 0,
            earnings_error_count: 0,
        }),
    )
    .await
    .unwrap();

    match output {
        MarketCommandOutput::StrengthStocks(ranking) => {
            assert_eq!(ranking.metric, StrengthStockMetric::MarketCap);
            assert_eq!(ranking.strong_top, 3);
            assert_eq!(ranking.sector_filter.as_deref(), Some("银行"));
            assert_eq!(ranking.candidate_stock_count, 1);
            assert_eq!(ranking.covered_count, 1);
            assert_eq!(ranking.rows.len(), 1);
            assert_eq!(ranking.rows[0].sector_name, "银行");
            assert_eq!(ranking.rows[0].code, "601398");
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
