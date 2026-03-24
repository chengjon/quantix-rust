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
}
