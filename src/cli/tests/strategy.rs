use super::*;

#[test]
fn parses_strategy_run_modes() {
    let cli = Cli::try_parse_from([
        "quantix", "strategy", "run", "-n", "ma_cross", "--mode", "paper", "-c", "000001",
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
        "quantix", "strategy", "run", "-n", "ma_cross", "--mode", "live", "-c", "000001",
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

    let cli = Cli::try_parse_from([
        "quantix",
        "strategy",
        "run",
        "-n",
        "ma_cross",
        "--mode",
        "mock_live",
        "-c",
        "000001",
    ])
    .unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Run { name, mode, code }) => {
            assert_eq!(name, "ma_cross");
            assert_eq!(mode, "mock_live");
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

#[test]
fn parses_strategy_config_commands() {
    let cli = Cli::try_parse_from(["quantix", "strategy", "config", "init"]).unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Config(StrategyConfigCommands::Init)) => {}
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "strategy", "config", "show"]).unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Config(StrategyConfigCommands::Show)) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_strategy_daemon_run_command() {
    let cli = Cli::try_parse_from(["quantix", "strategy", "daemon", "run"]).unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Daemon(StrategyDaemonCommands::Run { once })) => {
            assert!(!once);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "strategy", "daemon", "run", "--once"]).unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Daemon(StrategyDaemonCommands::Run { once })) => {
            assert!(once);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_strategy_signal_commands() {
    let cli = Cli::try_parse_from([
        "quantix",
        "strategy",
        "signal",
        "list",
        "--approval-status",
        "pending",
    ])
    .unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Signal(StrategySignalCommands::List {
            approval_status,
            ..
        })) => {
            assert_eq!(approval_status.as_deref(), Some("pending"));
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "strategy",
        "signal",
        "approve",
        "--signal-id",
        "sig-1",
        "--target-mode",
        "paper",
        "--target-account",
        "default",
    ])
    .unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Signal(StrategySignalCommands::Approve {
            signal_id,
            target_mode,
            target_account,
        })) => {
            assert_eq!(signal_id, "sig-1");
            assert_eq!(target_mode, "paper");
            assert_eq!(target_account, "default");
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "strategy",
        "signal",
        "reject",
        "--signal-id",
        "sig-2",
        "--reason",
        "manual reject",
    ])
    .unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Signal(StrategySignalCommands::Reject {
            signal_id,
            reason,
        })) => {
            assert_eq!(signal_id, "sig-2");
            assert_eq!(reason.as_deref(), Some("manual reject"));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_strategy_request_and_service_commands() {
    let cli = Cli::try_parse_from([
        "quantix", "strategy", "request", "list", "--status", "pending",
    ])
    .unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Request(StrategyRequestCommands::List {
            status,
            ..
        })) => {
            assert_eq!(status.as_deref(), Some("pending"));
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "strategy", "service", "install"]).unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Service(StrategyServiceCommands::Install)) => {}
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "strategy",
        "service-config",
        "set",
        "--quantix-bin",
        "/opt/quantix/bin/quantix",
        "--env-file",
        "/tmp/strategy.env",
    ])
    .unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::ServiceConfig(
            StrategyServiceConfigCommands::Set {
                quantix_bin,
                env_file,
            },
        )) => {
            assert_eq!(quantix_bin, "/opt/quantix/bin/quantix");
            assert_eq!(env_file.as_deref(), Some("/tmp/strategy.env"));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}
