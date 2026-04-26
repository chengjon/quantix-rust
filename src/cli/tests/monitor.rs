use super::*;

#[test]
fn parses_monitor_watchlist_command_with_once() {
    let cli = Cli::try_parse_from(["quantix", "monitor", "watchlist", "--once"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Watchlist { once, repeat }) => {
            assert!(once);
            assert!(!repeat);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_watchlist_rejects_missing_once() {
    let err = Cli::try_parse_from(["quantix", "monitor", "watchlist"]).unwrap_err();

    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    assert!(err.to_string().contains("--once"));
    assert!(err.to_string().contains("--repeat"));
}

#[test]
fn parses_monitor_watchlist_command_with_repeat() {
    let cli = Cli::try_parse_from(["quantix", "monitor", "watchlist", "--repeat"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Watchlist { once, repeat }) => {
            assert!(!once);
            assert!(repeat);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_watchlist_rejects_once_and_repeat_together() {
    let err =
        Cli::try_parse_from(["quantix", "monitor", "watchlist", "--once", "--repeat"]).unwrap_err();

    assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    assert!(err.to_string().contains("--once"));
    assert!(err.to_string().contains("--repeat"));
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

#[test]
fn parses_monitor_config_show_command() {
    let cli = Cli::try_parse_from(["quantix", "monitor", "config", "show"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Config(MonitorConfigCommands::Show)) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_config_set_interval_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "monitor",
        "config",
        "set",
        "--interval-seconds",
        "15",
    ])
    .unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Config(MonitorConfigCommands::Set {
            interval_seconds,
            group,
            persist_events,
            notify,
        })) => {
            assert_eq!(interval_seconds, Some(15));
            assert_eq!(group, None);
            assert_eq!(persist_events, None);
            assert_eq!(notify, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_config_set_group_command() {
    let cli =
        Cli::try_parse_from(["quantix", "monitor", "config", "set", "--group", "core"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Config(MonitorConfigCommands::Set {
            interval_seconds,
            group,
            persist_events,
            notify,
        })) => {
            assert_eq!(interval_seconds, None);
            assert_eq!(group.as_deref(), Some("core"));
            assert_eq!(persist_events, None);
            assert_eq!(notify, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_config_set_persist_events_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "monitor",
        "config",
        "set",
        "--persist-events",
        "false",
    ])
    .unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Config(MonitorConfigCommands::Set {
            interval_seconds,
            group,
            persist_events,
            notify,
        })) => {
            assert_eq!(interval_seconds, None);
            assert_eq!(group, None);
            assert_eq!(persist_events, Some(false));
            assert_eq!(notify, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_config_set_notify_command() {
    let cli =
        Cli::try_parse_from(["quantix", "monitor", "config", "set", "--notify", "true"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Config(MonitorConfigCommands::Set {
            interval_seconds,
            group,
            persist_events,
            notify,
        })) => {
            assert_eq!(interval_seconds, None);
            assert_eq!(group, None);
            assert_eq!(persist_events, None);
            assert_eq!(notify, Some(true));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_config_set_rejects_multiple_mutations() {
    let err = Cli::try_parse_from([
        "quantix",
        "monitor",
        "config",
        "set",
        "--group",
        "core",
        "--persist-events",
        "true",
    ])
    .unwrap_err();

    assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    assert!(err.to_string().contains("--group"));
    assert!(err.to_string().contains("--persist-events"));
}

#[test]
fn parses_monitor_config_clear_group_command() {
    let cli = Cli::try_parse_from(["quantix", "monitor", "config", "clear-group"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Config(MonitorConfigCommands::ClearGroup)) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_daemon_run_command() {
    let cli = Cli::try_parse_from(["quantix", "monitor", "daemon", "run"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Daemon(MonitorDaemonCommands::Run)) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_service_install_command() {
    let cli = Cli::try_parse_from(["quantix", "monitor", "service", "install"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Service(MonitorServiceCommands::Install)) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_service_status_command() {
    let cli = Cli::try_parse_from(["quantix", "monitor", "service", "status"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Service(MonitorServiceCommands::Status)) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_event_list_command_with_filters() {
    let cli = Cli::try_parse_from([
        "quantix",
        "monitor",
        "event",
        "list",
        "--limit",
        "10",
        "--code",
        "000001",
        "--type",
        "price-alert",
    ])
    .unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::Event(MonitorEventCommands::List {
            limit,
            code,
            event_type,
        })) => {
            assert_eq!(limit, 10);
            assert_eq!(code.as_deref(), Some("000001"));
            assert_eq!(event_type.as_deref(), Some("price-alert"));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_service_config_show_command() {
    let cli = Cli::try_parse_from(["quantix", "monitor", "service-config", "show"]).unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::ServiceConfig(MonitorServiceConfigCommands::Show)) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_service_config_set_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "monitor",
        "service-config",
        "set",
        "--quantix-bin",
        "/abs/path/to/quantix",
    ])
    .unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::ServiceConfig(MonitorServiceConfigCommands::Set {
            quantix_bin,
        })) => {
            assert_eq!(quantix_bin, "/abs/path/to/quantix");
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_monitor_service_config_set_rejects_missing_quantix_bin() {
    let err = Cli::try_parse_from(["quantix", "monitor", "service-config", "set"]).unwrap_err();

    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    assert!(err.to_string().contains("--quantix-bin"));
}

#[test]
fn parses_monitor_service_config_set_accepts_relative_path_at_parser_level() {
    let cli = Cli::try_parse_from([
        "quantix",
        "monitor",
        "service-config",
        "set",
        "--quantix-bin",
        "relative/path/to/quantix",
    ])
    .unwrap();

    match cli.command {
        Commands::Monitor(MonitorCommands::ServiceConfig(MonitorServiceConfigCommands::Set {
            quantix_bin,
        })) => {
            assert_eq!(quantix_bin, "relative/path/to/quantix");
        }
        other => panic!("unexpected command: {:?}", other),
    }
}
