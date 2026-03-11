use super::*;
use clap::Parser;
use clap::error::ErrorKind;

#[test]
fn parses_watchlist_add_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "watchlist",
        "add",
        "--code",
        "000001",
        "--group",
        "core",
    ])
    .unwrap();

    match cli.command {
        Commands::Watchlist(WatchlistCommands::Add { code, group }) => {
            assert_eq!(code, "000001");
            assert_eq!(group.as_deref(), Some("core"));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_watchlist_list_command_with_filters_and_price_flag() {
    let cli = Cli::try_parse_from([
        "quantix",
        "watchlist",
        "list",
        "--group",
        "core",
        "--tag",
        "bank",
        "--with-price",
    ])
    .unwrap();

    match cli.command {
        Commands::Watchlist(WatchlistCommands::List {
            group,
            tag,
            with_price,
        }) => {
            assert_eq!(group.as_deref(), Some("core"));
            assert_eq!(tag.as_deref(), Some("bank"));
            assert!(with_price);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_watchlist_group_create_command() {
    let cli = Cli::try_parse_from(["quantix", "watchlist", "group", "create", "--name", "core"])
        .unwrap();

    match cli.command {
        Commands::Watchlist(WatchlistCommands::Group(WatchlistGroupCommands::Create {
            name,
        })) => {
            assert_eq!(name, "core");
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_watchlist_history_command_with_limit() {
    let cli = Cli::try_parse_from([
        "quantix",
        "watchlist",
        "history",
        "--code",
        "000001",
        "--limit",
        "20",
    ])
    .unwrap();

    match cli.command {
        Commands::Watchlist(WatchlistCommands::History { code, limit }) => {
            assert_eq!(code.as_deref(), Some("000001"));
            assert_eq!(limit, 20);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_screener_preset_list_command() {
    let cli = Cli::try_parse_from(["quantix", "analyze", "screener", "preset-list"]).unwrap();

    match cli.command {
        Commands::Analyze(AnalyzeCommands::Screener(ScreenerCommands::PresetList)) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_screener_run_command_with_codes_and_preset() {
    let cli = Cli::try_parse_from([
        "quantix",
        "analyze",
        "screener",
        "run",
        "--codes",
        "000001,600519",
        "--preset",
        "close_above_ma:period=20",
    ])
    .unwrap();

    match cli.command {
        Commands::Analyze(AnalyzeCommands::Screener(ScreenerCommands::Run {
            codes,
            watchlist,
            group,
            preset,
            limit,
            sort_by,
        })) => {
            assert_eq!(codes.as_deref(), Some("000001,600519"));
            assert!(!watchlist);
            assert_eq!(group, None);
            assert_eq!(preset, vec!["close_above_ma:period=20"]);
            assert_eq!(limit, None);
            assert_eq!(sort_by.as_deref(), None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_screener_run_command_with_watchlist_group_and_multiple_presets() {
    let cli = Cli::try_parse_from([
        "quantix",
        "analyze",
        "screener",
        "run",
        "--watchlist",
        "--group",
        "core",
        "--preset",
        "close_above_ma:period=20",
        "--preset",
        "rsi_gte:period=14,value=55",
    ])
    .unwrap();

    match cli.command {
        Commands::Analyze(AnalyzeCommands::Screener(ScreenerCommands::Run {
            codes,
            watchlist,
            group,
            preset,
            limit,
            sort_by,
        })) => {
            assert_eq!(codes, None);
            assert!(watchlist);
            assert_eq!(group.as_deref(), Some("core"));
            assert_eq!(
                preset,
                vec![
                    "close_above_ma:period=20".to_string(),
                    "rsi_gte:period=14,value=55".to_string()
                ]
            );
            assert_eq!(limit, None);
            assert_eq!(sort_by.as_deref(), None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_watchlist_command_with_once() {
    let cli = Cli::try_parse_from(["quantix", "monitor", "watchlist", "--once"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Watchlist { once }) => {
            assert!(once);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_watchlist_rejects_missing_once() {
    let err = Cli::try_parse_from(["quantix", "monitor", "watchlist"]).unwrap_err();

    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    assert!(err.to_string().contains("--once"));
}

#[test]
fn parses_monitor_alert_add_command_with_above() {
    let cli = Cli::try_parse_from([
        "quantix", "monitor", "alert", "add", "000001", "--above", "16.0",
    ])
    .unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Alert(MonitorAlertCommands::Add {
            code,
            above,
            below,
        })) => {
            assert_eq!(code, "000001");
            assert_eq!(above, Some(16.0));
            assert_eq!(below, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_alert_add_command_with_below() {
    let cli = Cli::try_parse_from([
        "quantix", "monitor", "alert", "add", "000001", "--below", "15.0",
    ])
    .unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Alert(MonitorAlertCommands::Add {
            code,
            above,
            below,
        })) => {
            assert_eq!(code, "000001");
            assert_eq!(above, None);
            assert_eq!(below, Some(15.0));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_alert_list_command() {
    let cli = Cli::try_parse_from(["quantix", "monitor", "alert", "list"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Alert(MonitorAlertCommands::List)) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_alert_remove_command() {
    let cli = Cli::try_parse_from(["quantix", "monitor", "alert", "remove", "12"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Alert(MonitorAlertCommands::Remove { id })) => {
            assert_eq!(id, 12);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_alert_add_rejects_missing_threshold() {
    let err = Cli::try_parse_from(["quantix", "monitor", "alert", "add", "000001"]).unwrap_err();

    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    assert!(err.to_string().contains("--above"));
    assert!(err.to_string().contains("--below"));
}

#[test]
fn parses_monitor_alert_add_rejects_both_thresholds() {
    let err = Cli::try_parse_from([
        "quantix", "monitor", "alert", "add", "000001", "--above", "16.0", "--below", "15.0",
    ])
    .unwrap_err();

    assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    assert!(err.to_string().contains("--above"));
    assert!(err.to_string().contains("--below"));
}

#[test]
fn parses_monitor_alert_add_rejects_non_numeric_threshold() {
    let err = Cli::try_parse_from([
        "quantix",
        "monitor",
        "alert",
        "add",
        "000001",
        "--above",
        "not-a-number",
    ])
    .unwrap_err();

    assert_eq!(err.kind(), ErrorKind::ValueValidation);
    assert!(err.to_string().contains("not-a-number"));
}

#[test]
fn parses_stop_set_command_with_loss() {
    let cli = Cli::try_parse_from(["quantix", "stop", "set", "000001", "--loss", "14.5"]).unwrap();

    match cli.command {
        Commands::Stop(StopCommands::Set {
            code,
            loss,
            profit,
            trailing,
        }) => {
            assert_eq!(code, "000001");
            assert_eq!(loss, Some(14.5));
            assert_eq!(profit, None);
            assert_eq!(trailing, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_stop_set_command_with_profit() {
    let cli = Cli::try_parse_from(["quantix", "stop", "set", "000001", "--profit", "18.0"]).unwrap();

    match cli.command {
        Commands::Stop(StopCommands::Set {
            code,
            loss,
            profit,
            trailing,
        }) => {
            assert_eq!(code, "000001");
            assert_eq!(loss, None);
            assert_eq!(profit, Some(18.0));
            assert_eq!(trailing, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_stop_set_command_with_loss_and_profit() {
    let cli = Cli::try_parse_from([
        "quantix", "stop", "set", "000001", "--loss", "14.5", "--profit", "18.0",
    ])
    .unwrap();

    match cli.command {
        Commands::Stop(StopCommands::Set {
            code,
            loss,
            profit,
            trailing,
        }) => {
            assert_eq!(code, "000001");
            assert_eq!(loss, Some(14.5));
            assert_eq!(profit, Some(18.0));
            assert_eq!(trailing, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_stop_set_command_with_trailing() {
    let cli = Cli::try_parse_from(["quantix", "stop", "set", "000001", "--trailing", "5"]).unwrap();

    match cli.command {
        Commands::Stop(StopCommands::Set {
            code,
            loss,
            profit,
            trailing,
        }) => {
            assert_eq!(code, "000001");
            assert_eq!(loss, None);
            assert_eq!(profit, None);
            assert_eq!(trailing, Some(5.0));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_stop_list_command() {
    let cli = Cli::try_parse_from(["quantix", "stop", "list"]).unwrap();

    match cli.command {
        Commands::Stop(StopCommands::List) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_stop_remove_command() {
    let cli = Cli::try_parse_from(["quantix", "stop", "remove", "000001"]).unwrap();

    match cli.command {
        Commands::Stop(StopCommands::Remove { code }) => {
            assert_eq!(code, "000001");
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_stop_set_rejects_missing_thresholds() {
    let err = Cli::try_parse_from(["quantix", "stop", "set", "000001"]).unwrap_err();

    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    assert!(err.to_string().contains("--loss"));
    assert!(err.to_string().contains("--profit"));
    assert!(err.to_string().contains("--trailing"));
}

#[test]
fn parses_stop_set_rejects_loss_and_trailing_together() {
    let err = Cli::try_parse_from([
        "quantix",
        "stop",
        "set",
        "000001",
        "--loss",
        "14.5",
        "--trailing",
        "5",
    ])
    .unwrap_err();

    assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    assert!(err.to_string().contains("--loss"));
    assert!(err.to_string().contains("--trailing"));
}

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

    let cli = Cli::try_parse_from(["quantix", "trade", "position"]).unwrap();
    match cli.command {
        Commands::Trade(TradeCommands::Position) => {}
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
    let cli = Cli::try_parse_from(["quantix", "market", "concept", "--date", "2026-03-09"]).unwrap();

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
