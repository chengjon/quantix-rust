use super::*;

#[test]
fn parses_strategy_instance_management_commands() {
    let cli = Cli::try_parse_from([
        "quantix", "strategy", "create", "--id", "ma_demo", "--name", "ma_cross", "--code",
        "000001", "--param", "fast=5", "--param", "slow=20",
    ])
    .unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Create {
            id,
            name,
            code,
            params,
            disabled,
        }) => {
            assert_eq!(id, "ma_demo");
            assert_eq!(name, "ma_cross");
            assert_eq!(code, "000001");
            assert_eq!(params, vec!["fast=5", "slow=20"]);
            assert!(!disabled);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix", "strategy", "update", "--id", "ma_demo", "--code", "600519", "--param",
        "fast=10", "--enable",
    ])
    .unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Update {
            id,
            code,
            params,
            enable,
            disable,
            ..
        }) => {
            assert_eq!(id, "ma_demo");
            assert_eq!(code.as_deref(), Some("600519"));
            assert_eq!(params, vec!["fast=10"]);
            assert!(enable);
            assert!(!disable);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "strategy", "delete", "--id", "ma_demo"]).unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Delete { id }) => {
            assert_eq!(id, "ma_demo");
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

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
fn parses_strategy_show_commands() {
    let cli = Cli::try_parse_from(["quantix", "strategy", "show", "--name", "ma_cross"]).unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Show { name, id }) => {
            assert_eq!(name.as_deref(), Some("ma_cross"));
            assert_eq!(id, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "strategy", "show", "--id", "ma_demo"]).unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Show { name, id }) => {
            assert_eq!(name, None);
            assert_eq!(id.as_deref(), Some("ma_demo"));
        }
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

    let cli = Cli::try_parse_from([
        "quantix",
        "strategy",
        "request",
        "execute",
        "--request-id",
        "req-1",
    ])
    .unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Request(StrategyRequestCommands::Execute {
            request_id,
        })) => {
            assert_eq!(request_id, "req-1");
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "strategy",
        "request",
        "cancel",
        "--request-id",
        "req-2",
        "--reason",
        "manual cancel",
    ])
    .unwrap();
    match cli.command {
        Commands::Strategy(StrategyCommands::Request(StrategyRequestCommands::Cancel {
            request_id,
            reason,
        })) => {
            assert_eq!(request_id, "req-2");
            assert_eq!(reason.as_deref(), Some("manual cancel"));
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
