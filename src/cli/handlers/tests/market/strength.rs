use super::*;
use rust_decimal::Decimal;

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
async fn test_execute_market_strength_stocks_returns_empty_ranking_when_sector_has_no_matches() {
    let output = execute_market_command_with_test_payloads(
        MarketCommands::StrengthStocks {
            date: Some("2026-03-09".to_string()),
            strong_top: 3,
            sector: Some("银行".to_string()),
            metric: StrengthStockMetric::Profit,
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
            top_by_market_cap: vec![],
            top_by_profit: vec![StrongSectorStockRow {
                sector_name: "计算机".to_string(),
                code: "300024".to_string(),
                name: "机器人".to_string(),
                latest_price: 15.0,
                latest_change_pct: 4.2,
                market_cap: Some(Decimal::new(200000, 2)),
                latest_report_profit: Some(Decimal::new(3000, 2)),
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
            assert_eq!(ranking.sector_filter.as_deref(), Some("银行"));
            assert_eq!(ranking.candidate_stock_count, 0);
            assert_eq!(ranking.covered_count, 0);
            assert!(ranking.rows.is_empty());
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_market_strength_requires_runtime_risk_path_context() {
    let err = execute_market_command_with_reader(
        MarketCommands::Strength {
            date: Some("2026-03-09".to_string()),
            strong_top: 3,
            weak_top: 3,
            stock_top: 10,
        },
        FakeMarketReader::new(),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Other(_)));
    assert!(err.to_string().contains("risk_path"));
}

#[tokio::test]
async fn test_execute_market_strength_rejects_invalid_date_format() {
    let err = execute_market_command_with_reader(
        MarketCommands::Strength {
            date: Some("20260309".to_string()),
            strong_top: 3,
            weak_top: 3,
            stock_top: 10,
        },
        FakeMarketReader::new(),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Other(_)));
    assert!(err.to_string().contains("无效日期格式"));
    assert!(err.to_string().contains("YYYY-MM-DD"));
}

#[tokio::test]
async fn test_execute_market_strength_stocks_requires_runtime_risk_path_context() {
    let err = execute_market_command_with_reader(
        MarketCommands::StrengthStocks {
            date: Some("2026-03-09".to_string()),
            strong_top: 3,
            sector: Some("银行".to_string()),
            metric: StrengthStockMetric::Profit,
            top: 10,
        },
        FakeMarketReader::new(),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Other(_)));
    assert!(err.to_string().contains("risk_path"));
}

#[tokio::test]
async fn test_execute_market_strength_stocks_rejects_invalid_date_format() {
    let err = execute_market_command_with_reader(
        MarketCommands::StrengthStocks {
            date: Some("2026/03/09".to_string()),
            strong_top: 3,
            sector: Some("银行".to_string()),
            metric: StrengthStockMetric::Profit,
            top: 10,
        },
        FakeMarketReader::new(),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Other(_)));
    assert!(err.to_string().contains("无效日期格式"));
    assert!(err.to_string().contains("YYYY-MM-DD"));
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
