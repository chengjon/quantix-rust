use super::*;

#[test]
fn parses_trade() {
    let cli = Cli::try_parse_from(["quantix", "trade", "init"]).unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::Init {
            capital,
            commission_rate,
            commission_min,
            stamp_duty_rate,
            transfer_fee_rate,
        }) => {
            assert_eq!(capital, None);
            assert_eq!(commission_rate, None);
            assert_eq!(commission_min, None);
            assert_eq!(stamp_duty_rate, None);
            assert_eq!(transfer_fee_rate, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "trade",
        "init",
        "--capital",
        "1500000",
        "--commission-rate",
        "0.0003",
    ])
    .unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::Init {
            capital,
            commission_rate,
            ..
        }) => {
            assert_eq!(capital, Some(1_500_000.0));
            assert_eq!(commission_rate, Some(0.0003));
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "trade", "reset", "--capital", "500000"]).unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::Reset { capital, .. }) => {
            assert_eq!(capital, Some(500000.0));
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix", "trade", "buy", "000001", "--price", "15.0", "--volume", "1000",
    ])
    .unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::Buy {
            code,
            price,
            volume,
        }) => {
            assert_eq!(code, "000001");
            assert_eq!(price, 15.0);
            assert_eq!(volume, 1000);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix", "trade", "sell", "000001", "--price", "16.0", "--volume", "500",
    ])
    .unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::Sell {
            code,
            price,
            volume,
        }) => {
            assert_eq!(code, "000001");
            assert_eq!(price, 16.0);
            assert_eq!(volume, 500);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "trade", "history"]).unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::History { code, limit }) => {
            assert_eq!(code, None);
            assert_eq!(limit, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "trade",
        "history",
        "--code",
        "000001",
        "--limit",
        "5",
    ])
    .unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::History { code, limit }) => {
            assert_eq!(code, Some("000001".to_string()));
            assert_eq!(limit, Some(5));
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "trade", "fees"]).unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::Fees { code, limit }) => {
            assert_eq!(code, None);
            assert_eq!(limit, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "trade",
        "fees",
        "--code",
        "600000",
        "--limit",
        "10",
    ])
    .unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::Fees { code, limit }) => {
            assert_eq!(code, Some("600000".to_string()));
            assert_eq!(limit, Some(10));
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "trade", "overview"]).unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::Overview { current }) => {
            assert!(!current);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "trade", "overview", "--current"]).unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::Overview { current }) => {
            assert!(current);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "trade", "position"]).unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::Position { current }) => {
            assert!(!current);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "trade", "position", "--current"]).unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::Position { current }) => {
            assert!(current);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "trade", "cash"]).unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::Cash) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_trade_rejects_missing_price_or_volume() {
    let err = Cli::try_parse_from(["quantix", "trade", "buy", "000001", "--volume", "1000"])
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    assert!(err.to_string().contains("--price"));

    let err = Cli::try_parse_from(["quantix", "trade", "sell", "000001", "--price", "16.0"])
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    assert!(err.to_string().contains("--volume"));
}
