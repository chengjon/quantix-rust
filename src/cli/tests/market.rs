use super::*;

#[test]
fn parses_market_foundation_command() {
    let cli = Cli::try_parse_from(["quantix", "market", "foundation"]).unwrap();

    match cli.command {
        Commands::Market(MarketCommands::Foundation) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_market_sector_command_with_top() {
    let cli = Cli::try_parse_from(["quantix", "market", "sector", "--top", "10"]).unwrap();

    match cli.command {
        Commands::Market(MarketCommands::Sector { top, date, sort_by }) => {
            assert_eq!(top, Some(10));
            assert_eq!(date, None);
            assert_eq!(sort_by, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_market_concept_command_with_date() {
    let cli =
        Cli::try_parse_from(["quantix", "market", "concept", "--date", "2026-03-09"]).unwrap();

    match cli.command {
        Commands::Market(MarketCommands::Concept { top, date, sort_by }) => {
            assert_eq!(top, None);
            assert_eq!(date.as_deref(), Some("2026-03-09"));
            assert_eq!(sort_by, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_market_north_command() {
    let cli = Cli::try_parse_from(["quantix", "market", "north"]).unwrap();

    match cli.command {
        Commands::Market(MarketCommands::North { date }) => {
            assert_eq!(date, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_market_sentiment_command() {
    let cli = Cli::try_parse_from(["quantix", "market", "sentiment"]).unwrap();

    match cli.command {
        Commands::Market(MarketCommands::Sentiment { date }) => {
            assert_eq!(date, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_market_leader_command_with_sector_and_limit() {
    let cli = Cli::try_parse_from([
        "quantix", "market", "leader", "--sector", "银行", "--limit", "5",
    ])
    .unwrap();

    match cli.command {
        Commands::Market(MarketCommands::Leader {
            sector,
            concept,
            all,
            limit,
            date,
        }) => {
            assert_eq!(sector.as_deref(), Some("银行"));
            assert_eq!(concept, None);
            assert!(!all);
            assert_eq!(limit, Some(5));
            assert_eq!(date, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_market_overview_command() {
    let cli = Cli::try_parse_from(["quantix", "market", "overview"]).unwrap();

    match cli.command {
        Commands::Market(MarketCommands::Overview { top, date }) => {
            assert_eq!(top, None);
            assert_eq!(date, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_market_strength_command_with_explicit_thresholds() {
    let cli = Cli::try_parse_from([
        "quantix",
        "market",
        "strength",
        "--date",
        "2026-03-09",
        "--strong-top",
        "5",
        "--weak-top",
        "4",
        "--stock-top",
        "12",
    ])
    .unwrap();

    match cli.command {
        Commands::Market(MarketCommands::Strength {
            date,
            strong_top,
            weak_top,
            stock_top,
        }) => {
            assert_eq!(date.as_deref(), Some("2026-03-09"));
            assert_eq!(strong_top, 5);
            assert_eq!(weak_top, 4);
            assert_eq!(stock_top, 12);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_market_strength_stocks_command_with_metric_and_top() {
    let cli = Cli::try_parse_from([
        "quantix",
        "market",
        "strength-stocks",
        "--date",
        "2026-03-09",
        "--strong-top",
        "4",
        "--metric",
        "profit",
        "--top",
        "8",
    ])
    .unwrap();

    match cli.command {
        Commands::Market(MarketCommands::StrengthStocks {
            date,
            strong_top,
            sector,
            metric,
            top,
        }) => {
            assert_eq!(date.as_deref(), Some("2026-03-09"));
            assert_eq!(strong_top, 4);
            assert_eq!(sector, None);
            assert_eq!(metric, StrengthStockMetric::Profit);
            assert_eq!(top, 8);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_market_strength_stocks_command_with_sector() {
    let cli = Cli::try_parse_from([
        "quantix",
        "market",
        "strength-stocks",
        "--sector",
        "银行",
        "--metric",
        "market-cap",
        "--top",
        "10",
    ])
    .unwrap();

    match cli.command {
        Commands::Market(MarketCommands::StrengthStocks {
            date,
            strong_top,
            sector,
            metric,
            top,
        }) => {
            assert_eq!(date, None);
            assert_eq!(strong_top, 3);
            assert_eq!(sector.as_deref(), Some("银行"));
            assert_eq!(metric, StrengthStockMetric::MarketCap);
            assert_eq!(top, 10);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn rejects_market_leader_with_sector_and_concept_together() {
    let result = Cli::try_parse_from([
        "quantix",
        "market",
        "leader",
        "--sector",
        "银行",
        "--concept",
        "人工智能",
    ]);

    assert!(result.is_err());
}

#[test]
fn rejects_market_leader_without_any_filter() {
    let result = Cli::try_parse_from(["quantix", "market", "leader"]);

    assert!(result.is_err());
}
