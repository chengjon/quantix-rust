use super::*;

#[test]
fn parses_stop_set_command_with_loss() {
    let cli = Cli::try_parse_from(["quantix", "stop", "set", "000001", "--loss", "14.5"]).unwrap();

    match cli.command {
        Commands::Stop(StopCommands::Set {
            code,
            loss,
            profit,
            loss_pct,
            profit_pct,
            trailing,
        }) => {
            assert_eq!(code, "000001");
            assert_eq!(loss, Some(14.5));
            assert_eq!(profit, None);
            assert_eq!(loss_pct, None);
            assert_eq!(profit_pct, None);
            assert_eq!(trailing, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_stop_set_command_with_profit() {
    let cli =
        Cli::try_parse_from(["quantix", "stop", "set", "000001", "--profit", "18.0"]).unwrap();

    match cli.command {
        Commands::Stop(StopCommands::Set {
            code,
            loss,
            profit,
            loss_pct,
            profit_pct,
            trailing,
        }) => {
            assert_eq!(code, "000001");
            assert_eq!(loss, None);
            assert_eq!(profit, Some(18.0));
            assert_eq!(loss_pct, None);
            assert_eq!(profit_pct, None);
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
            loss_pct,
            profit_pct,
            trailing,
        }) => {
            assert_eq!(code, "000001");
            assert_eq!(loss, Some(14.5));
            assert_eq!(profit, Some(18.0));
            assert_eq!(loss_pct, None);
            assert_eq!(profit_pct, None);
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
            loss_pct,
            profit_pct,
            trailing,
        }) => {
            assert_eq!(code, "000001");
            assert_eq!(loss, None);
            assert_eq!(profit, None);
            assert_eq!(loss_pct, None);
            assert_eq!(profit_pct, None);
            assert_eq!(trailing, Some(5.0));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_stop_set_command_with_loss_pct() {
    let cli =
        Cli::try_parse_from(["quantix", "stop", "set", "000001", "--loss-pct", "5"]).unwrap();

    match cli.command {
        Commands::Stop(StopCommands::Set {
            code,
            loss,
            profit,
            loss_pct,
            profit_pct,
            trailing,
        }) => {
            assert_eq!(code, "000001");
            assert_eq!(loss, None);
            assert_eq!(profit, None);
            assert_eq!(loss_pct, Some(5.0));
            assert_eq!(profit_pct, None);
            assert_eq!(trailing, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_stop_set_command_with_profit_pct() {
    let cli =
        Cli::try_parse_from(["quantix", "stop", "set", "000001", "--profit-pct", "12"]).unwrap();

    match cli.command {
        Commands::Stop(StopCommands::Set {
            code,
            loss,
            profit,
            loss_pct,
            profit_pct,
            trailing,
        }) => {
            assert_eq!(code, "000001");
            assert_eq!(loss, None);
            assert_eq!(profit, None);
            assert_eq!(loss_pct, None);
            assert_eq!(profit_pct, Some(12.0));
            assert_eq!(trailing, None);
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
fn parses_stop_update_command_with_patch_and_clear_flags() {
    let cli = Cli::try_parse_from([
        "quantix",
        "stop",
        "update",
        "000001",
        "--profit-pct",
        "12",
        "--clear-profit",
    ])
    .unwrap();

    match cli.command {
        Commands::Stop(StopCommands::Update {
            code,
            loss,
            profit,
            loss_pct,
            profit_pct,
            trailing,
            clear_loss,
            clear_profit,
            clear_loss_pct,
            clear_profit_pct,
            clear_trailing,
        }) => {
            assert_eq!(code, "000001");
            assert_eq!(loss, None);
            assert_eq!(profit, None);
            assert_eq!(loss_pct, None);
            assert_eq!(profit_pct, Some(12.0));
            assert!(!clear_loss);
            assert!(clear_profit);
            assert!(!clear_loss_pct);
            assert!(!clear_profit_pct);
            assert!(!clear_trailing);
            assert_eq!(trailing, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_stop_status_command() {
    let cli = Cli::try_parse_from(["quantix", "stop", "status", "--code", "000001"]).unwrap();

    match cli.command {
        Commands::Stop(StopCommands::Status { code }) => {
            assert_eq!(code.as_deref(), Some("000001"));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_stop_history_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "stop",
        "history",
        "--code",
        "000001",
        "--limit",
        "10",
        "--date",
        "2026-03-24",
        "--type",
        "trigger",
    ])
    .unwrap();

    match cli.command {
        Commands::Stop(StopCommands::History {
            code,
            limit,
            date,
            event_type,
        }) => {
            assert_eq!(code.as_deref(), Some("000001"));
            assert_eq!(limit, 10);
            assert_eq!(date.as_deref(), Some("2026-03-24"));
            assert_eq!(event_type.as_deref(), Some("trigger"));
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
    assert!(err.to_string().contains("--loss-pct"));
    assert!(err.to_string().contains("--profit-pct"));
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
