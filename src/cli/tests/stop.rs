use super::*;

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
