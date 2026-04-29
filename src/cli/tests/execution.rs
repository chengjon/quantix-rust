use super::*;

#[test]
fn parses_execution_config_and_daemon_commands() {
    let cli = Cli::try_parse_from(["quantix", "execution", "config", "init"]).unwrap();
    match cli.command {
        Commands::Execution(ExecutionCommands::Config(ExecutionConfigCommands::Init)) => {}
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "execution", "config", "show"]).unwrap();
    match cli.command {
        Commands::Execution(ExecutionCommands::Config(ExecutionConfigCommands::Show)) => {}
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "execution", "daemon", "run"]).unwrap();
    match cli.command {
        Commands::Execution(ExecutionCommands::Daemon(ExecutionDaemonCommands::Run { once })) => {
            assert!(!once);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "execution", "daemon", "run", "--once"]).unwrap();
    match cli.command {
        Commands::Execution(ExecutionCommands::Daemon(ExecutionDaemonCommands::Run { once })) => {
            assert!(once);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "execution", "bridge", "status"]).unwrap();
    match cli.command {
        Commands::Execution(ExecutionCommands::Bridge(ExecutionBridgeCommands::Status)) => {}
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "execution",
        "bridge",
        "qmt-preview",
        "--request-id",
        "req-1",
    ])
    .unwrap();
    match cli.command {
        Commands::Execution(ExecutionCommands::Bridge(ExecutionBridgeCommands::QmtPreview {
            request_id,
        })) => {
            assert_eq!(request_id, "req-1");
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "execution",
        "bridge",
        "qmt-live",
        "--request-id",
        "req-2",
        "--yes",
    ])
    .unwrap();
    match cli.command {
        Commands::Execution(ExecutionCommands::Bridge(ExecutionBridgeCommands::QmtLive {
            request_id,
            yes,
        })) => {
            assert_eq!(request_id, "req-2");
            assert!(yes);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "execution",
        "qmt",
        "live",
        "--request-id",
        "req-3",
        "--yes",
    ])
    .unwrap();
    match cli.command {
        Commands::Execution(ExecutionCommands::Qmt(ExecutionQmtCommands::Live {
            request_id,
            yes,
        })) => {
            assert_eq!(request_id, "req-3");
            assert!(yes);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}
