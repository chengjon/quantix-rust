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

#[test]
fn parses_backtest_walkforward_command_with_defaults() {
    let cli =
        Cli::try_parse_from(["quantix", "backtest", "walkforward", "--code", "600000"]).unwrap();

    match cli.command {
        Commands::Backtest(BacktestCommands::Walkforward {
            code,
            start,
            end,
            short_start,
            short_end,
            short_step,
            long_start,
            long_end,
            long_step,
            train_window,
            test_window,
            step,
            capital,
            commission_rate,
            position_size,
            min_stability,
        }) => {
            assert_eq!(code, "600000");
            assert!(start.is_none() && end.is_none());
            assert_eq!((short_start, short_end, short_step), (2, 5, 1));
            assert_eq!((long_start, long_end, long_step), (5, 20, 1));
            assert_eq!((train_window, test_window, step), (60, 20, 20));
            assert_eq!(capital, "100000");
            assert_eq!(commission_rate, "0.0003");
            assert_eq!(position_size, "100");
            assert_eq!(min_stability, 0.5);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_backtest_walkforward_command_with_custom_args() {
    let cli = Cli::try_parse_from([
        "quantix",
        "backtest",
        "walkforward",
        "--code",
        "000001",
        "--start",
        "20240101",
        "--end",
        "20241231",
        "--short-start",
        "2",
        "--short-end",
        "4",
        "--short-step",
        "1",
        "--long-start",
        "10",
        "--long-end",
        "30",
        "--long-step",
        "5",
        "--train-window",
        "120",
        "--test-window",
        "20",
        "--step",
        "60",
        "--capital",
        "250000",
        "--commission-rate",
        "0.0005",
        "--position-size",
        "200",
        "--min-stability",
        "0.6",
    ])
    .unwrap();

    match cli.command {
        Commands::Backtest(BacktestCommands::Walkforward {
            code,
            start,
            end,
            short_start,
            short_end,
            short_step,
            long_start,
            long_end,
            long_step,
            train_window,
            test_window,
            step,
            capital,
            commission_rate,
            position_size,
            min_stability,
        }) => {
            assert_eq!(code, "000001");
            assert_eq!(start.as_deref(), Some("20240101"));
            assert_eq!(end.as_deref(), Some("20241231"));
            assert_eq!((short_start, short_end, short_step), (2, 4, 1));
            assert_eq!((long_start, long_end, long_step), (10, 30, 5));
            assert_eq!((train_window, test_window, step), (120, 20, 60));
            assert_eq!(capital, "250000");
            assert_eq!(commission_rate, "0.0005");
            assert_eq!(position_size, "200");
            assert_eq!(min_stability, 0.6);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}
