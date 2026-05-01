use super::*;

#[test]
fn parses_backtest_run_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "backtest",
        "run",
        "--strategy",
        "ma_cross",
        "--code",
        "000001",
        "--start",
        "20240101",
        "--end",
        "20241231",
        "--capital",
        "250000",
        "--short",
        "5",
        "--long",
        "20",
    ])
    .unwrap();

    match cli.command {
        Commands::Backtest(BacktestCommands::Run {
            strategy,
            code,
            start,
            end,
            capital,
            short_period,
            long_period,
            ..
        }) => {
            assert_eq!(strategy, "ma_cross");
            assert_eq!(code, "000001");
            assert_eq!(start.as_deref(), Some("20240101"));
            assert_eq!(end.as_deref(), Some("20241231"));
            assert_eq!(capital, "250000");
            assert_eq!(short_period, 5);
            assert_eq!(long_period, 20);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_backtest_report_command() {
    let cli = Cli::try_parse_from(["quantix", "backtest", "report", "--id", "bt-001"]).unwrap();

    match cli.command {
        Commands::Backtest(BacktestCommands::Report { id }) => {
            assert_eq!(id, "bt-001");
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_backtest_list_command() {
    let cli = Cli::try_parse_from(["quantix", "backtest", "list"]).unwrap();

    match cli.command {
        Commands::Backtest(BacktestCommands::List) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_backtest_compare_command() {
    let cli = Cli::try_parse_from([
        "quantix", "backtest", "compare", "--id", "bt-001", "--id", "bt-002",
    ])
    .unwrap();

    match cli.command {
        Commands::Backtest(BacktestCommands::Compare { ids }) => {
            assert_eq!(ids, vec!["bt-001", "bt-002"]);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}
