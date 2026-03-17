use super::*;

#[test]
fn parses_strategy_run_modes() {
    let cli = Cli::try_parse_from([
        "quantix",
        "strategy",
        "run",
        "-n",
        "ma_cross",
        "--mode",
        "paper",
        "-c",
        "000001",
    ])
    .unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Run { name, mode, code }) => {
            assert_eq!(name, "ma_cross");
            assert_eq!(mode, "paper");
            assert_eq!(code.as_deref(), Some("000001"));
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "strategy",
        "run",
        "-n",
        "ma_cross",
        "--mode",
        "live",
        "-c",
        "000001",
    ])
    .unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Run { name, mode, code }) => {
            assert_eq!(name, "ma_cross");
            assert_eq!(mode, "live");
            assert_eq!(code.as_deref(), Some("000001"));
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "strategy", "run", "-n", "ma_cross"]).unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Run { name, mode, code }) => {
            assert_eq!(name, "ma_cross");
            assert_eq!(mode, "backtest");
            assert_eq!(code, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}
