use super::*;

#[test]
fn parses_screener_preset_list_command() {
    let cli = Cli::try_parse_from(["quantix", "analyze", "screener", "preset-list"]).unwrap();

    match cli.command {
        Commands::Analyze(AnalyzeCommands::Screener(ScreenerCommands::PresetList)) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_screener_presets_alias_command() {
    let cli = Cli::try_parse_from(["quantix", "analyze", "screener", "presets"]).unwrap();

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
