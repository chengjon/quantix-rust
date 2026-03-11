use super::*;

#[test]
fn parses_monitor_watchlist_command_with_once() {
    let cli = Cli::try_parse_from(["quantix", "monitor", "watchlist", "--once"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Watchlist { once }) => {
            assert!(once);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_watchlist_rejects_missing_once() {
    let err = Cli::try_parse_from(["quantix", "monitor", "watchlist"]).unwrap_err();

    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    assert!(err.to_string().contains("--once"));
}

#[test]
fn parses_monitor_alert_add_command_with_above() {
    let cli = Cli::try_parse_from([
        "quantix", "monitor", "alert", "add", "000001", "--above", "16.0",
    ])
    .unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Alert(MonitorAlertCommands::Add {
            code,
            above,
            below,
        })) => {
            assert_eq!(code, "000001");
            assert_eq!(above, Some(16.0));
            assert_eq!(below, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_alert_add_command_with_below() {
    let cli = Cli::try_parse_from([
        "quantix", "monitor", "alert", "add", "000001", "--below", "15.0",
    ])
    .unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Alert(MonitorAlertCommands::Add {
            code,
            above,
            below,
        })) => {
            assert_eq!(code, "000001");
            assert_eq!(above, None);
            assert_eq!(below, Some(15.0));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_alert_list_command() {
    let cli = Cli::try_parse_from(["quantix", "monitor", "alert", "list"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Alert(MonitorAlertCommands::List)) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_alert_remove_command() {
    let cli = Cli::try_parse_from(["quantix", "monitor", "alert", "remove", "12"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Alert(MonitorAlertCommands::Remove { id })) => {
            assert_eq!(id, 12);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_alert_add_rejects_missing_threshold() {
    let err = Cli::try_parse_from(["quantix", "monitor", "alert", "add", "000001"]).unwrap_err();

    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    assert!(err.to_string().contains("--above"));
    assert!(err.to_string().contains("--below"));
}

#[test]
fn parses_monitor_alert_add_rejects_both_thresholds() {
    let err = Cli::try_parse_from([
        "quantix", "monitor", "alert", "add", "000001", "--above", "16.0", "--below", "15.0",
    ])
    .unwrap_err();

    assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    assert!(err.to_string().contains("--above"));
    assert!(err.to_string().contains("--below"));
}

#[test]
fn parses_monitor_alert_add_rejects_non_numeric_threshold() {
    let err = Cli::try_parse_from([
        "quantix",
        "monitor",
        "alert",
        "add",
        "000001",
        "--above",
        "not-a-number",
    ])
    .unwrap_err();

    assert_eq!(err.kind(), ErrorKind::ValueValidation);
    assert!(err.to_string().contains("not-a-number"));
}
