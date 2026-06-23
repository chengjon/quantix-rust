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
        Commands::Execution(ExecutionCommands::Bridge(ExecutionBridgeCommands::Status {
            checklist,
        })) => {
            assert!(!checklist);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli =
        Cli::try_parse_from(["quantix", "execution", "bridge", "status", "--checklist"]).unwrap();
    match cli.command {
        Commands::Execution(ExecutionCommands::Bridge(ExecutionBridgeCommands::Status {
            checklist,
        })) => {
            assert!(checklist);
        }
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

    let cli =
        Cli::try_parse_from(["quantix", "execution", "qmt", "status", "--checklist"]).unwrap();
    match cli.command {
        Commands::Execution(ExecutionCommands::Qmt(ExecutionQmtCommands::Status { checklist })) => {
            assert!(checklist);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "execution",
        "qmt",
        "audit",
        "--request-id",
        "req-audit-1",
    ])
    .unwrap();
    match cli.command {
        Commands::Execution(ExecutionCommands::Qmt(ExecutionQmtCommands::Audit {
            request_id,
            task_id,
            local_submission_id,
        })) => {
            assert_eq!(request_id.as_deref(), Some("req-audit-1"));
            assert_eq!(task_id, None);
            assert_eq!(local_submission_id, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "execution",
        "qmt",
        "audit",
        "--task-id",
        "task-audit-1",
    ])
    .unwrap();
    match cli.command {
        Commands::Execution(ExecutionCommands::Qmt(ExecutionQmtCommands::Audit {
            request_id,
            task_id,
            local_submission_id,
        })) => {
            assert_eq!(request_id, None);
            assert_eq!(task_id.as_deref(), Some("task-audit-1"));
            assert_eq!(local_submission_id, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "execution",
        "bridge",
        "qmt-audit",
        "--local-submission-id",
        "local-audit-1",
    ])
    .unwrap();
    match cli.command {
        Commands::Execution(ExecutionCommands::Bridge(ExecutionBridgeCommands::QmtAudit {
            request_id,
            task_id,
            local_submission_id,
        })) => {
            assert_eq!(request_id, None);
            assert_eq!(task_id, None);
            assert_eq!(local_submission_id.as_deref(), Some("local-audit-1"));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}
