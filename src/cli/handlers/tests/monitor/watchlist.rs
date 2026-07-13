use super::monitor_helpers::*;
use super::stop::stop_rule;
use super::*;

#[tokio::test]
async fn test_execute_monitor_watchlist_once_returns_rows() {
    let service = MonitorService::new(
        FakeMonitorWatchlistReader {
            items: vec![
                monitor_watchlist_item("000001", "core", &["bank"]),
                monitor_watchlist_item("000002", "swing", &["tech"]),
            ],
        },
        FakeMonitorQuoteReader {
            rows: vec![
                monitor_quote_row("000001", 16.2, 1.2),
                monitor_quote_row("000002", 21.4, 2.6),
            ],
        },
        FakeMonitorAlertStore::default(),
    );

    let output = execute_monitor_command_with_service(
        MonitorCommands::Watchlist {
            once: true,
            repeat: false,
        },
        &service,
    )
    .await
    .unwrap();

    match output {
        MonitorCommandOutput::Watchlist {
            snapshot,
            triggered_stops,
        } => {
            assert_eq!(snapshot.rows.len(), 2);
            assert_eq!(snapshot.rows[0].code, "000001");
            assert_eq!(snapshot.rows[0].group, "core");
            assert_eq!(snapshot.rows[0].tags, vec!["bank".to_string()]);
            assert_eq!(snapshot.rows[0].last_price, Some(16.2));
            assert!(snapshot.triggered_alerts.is_empty());
            assert!(triggered_stops.is_empty());
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_monitor_watchlist_once_surfaces_triggered_alerts() {
    let store = FakeMonitorAlertStore {
        state: Arc::new(Mutex::new(FakeMonitorAlertState {
            next_id: 1,
            alerts: vec![monitor_alert(1, "000001", PriceAlertKind::Above, 16.0)],
            removed_ids: Vec::new(),
        })),
    };
    let service = MonitorService::new(
        FakeMonitorWatchlistReader {
            items: vec![monitor_watchlist_item("000001", "core", &[])],
        },
        FakeMonitorQuoteReader {
            rows: vec![monitor_quote_row("000001", 16.8, 3.2)],
        },
        store,
    );

    let output = execute_monitor_command_with_service(
        MonitorCommands::Watchlist {
            once: true,
            repeat: false,
        },
        &service,
    )
    .await
    .unwrap();

    match output {
        MonitorCommandOutput::Watchlist {
            snapshot,
            triggered_stops,
        } => {
            assert_eq!(snapshot.rows.len(), 1);
            assert_eq!(snapshot.triggered_alerts.len(), 1);
            assert_eq!(snapshot.triggered_alerts[0].alert_id, 1);
            assert_eq!(snapshot.triggered_alerts[0].code, "000001");
            assert_eq!(snapshot.triggered_alerts[0].current_price, 16.8);
            assert!(triggered_stops.is_empty());
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_monitor_watchlist_requires_once() {
    let service = MonitorService::new(
        FakeMonitorWatchlistReader::default(),
        FakeMonitorQuoteReader::default(),
        FakeMonitorAlertStore::default(),
    );

    let err = execute_monitor_command_with_service(
        MonitorCommands::Watchlist {
            once: false,
            repeat: false,
        },
        &service,
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Other(_)));
    assert!(err.to_string().contains("--once"));
    assert!(err.to_string().contains("--repeat"));
}

#[tokio::test]
async fn test_execute_monitor_stop_fixed_loss_triggers_from_snapshot_price() {
    let store = FakeStopRuleStore {
        state: Arc::new(Mutex::new(FakeStopRuleState {
            rules: vec![stop_rule("000001")],
            history: Vec::new(),
            removed_codes: Vec::new(),
        })),
    };
    let service = MonitorService::new(
        FakeMonitorWatchlistReader {
            items: vec![monitor_watchlist_item("000001", "core", &[])],
        },
        FakeMonitorQuoteReader {
            rows: vec![monitor_quote_row("000001", 14.2, -2.1)],
        },
        FakeMonitorAlertStore::default(),
    );

    let output = execute_monitor_command_with_stop_store(
        MonitorCommands::Watchlist {
            once: true,
            repeat: false,
        },
        &service,
        &store,
    )
    .await
    .unwrap();

    match output {
        MonitorCommandOutput::Watchlist {
            snapshot,
            triggered_stops,
        } => {
            assert_eq!(snapshot.rows.len(), 1);
            assert_eq!(triggered_stops.len(), 1);
            assert_eq!(triggered_stops[0].kind, StopTriggerKind::Loss);
            assert_eq!(triggered_stops[0].code, "000001");
            assert_eq!(triggered_stops[0].current_price, 14.2);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_monitor_stop_fixed_profit_triggers_from_snapshot_price() {
    let mut rule = stop_rule("000001");
    rule.stop_loss_price = None;
    rule.take_profit_price = Some(18.0);
    let store = FakeStopRuleStore {
        state: Arc::new(Mutex::new(FakeStopRuleState {
            rules: vec![rule],
            history: Vec::new(),
            removed_codes: Vec::new(),
        })),
    };
    let service = MonitorService::new(
        FakeMonitorWatchlistReader {
            items: vec![monitor_watchlist_item("000001", "core", &[])],
        },
        FakeMonitorQuoteReader {
            rows: vec![monitor_quote_row("000001", 18.3, 4.8)],
        },
        FakeMonitorAlertStore::default(),
    );

    let output = execute_monitor_command_with_stop_store(
        MonitorCommands::Watchlist {
            once: true,
            repeat: false,
        },
        &service,
        &store,
    )
    .await
    .unwrap();

    match output {
        MonitorCommandOutput::Watchlist {
            snapshot: _,
            triggered_stops,
        } => {
            assert_eq!(triggered_stops.len(), 1);
            assert_eq!(triggered_stops[0].kind, StopTriggerKind::Profit);
            assert_eq!(triggered_stops[0].threshold_price, 18.0);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_monitor_stop_trailing_updates_highest_price() {
    let mut rule = stop_rule("000001");
    rule.stop_loss_price = None;
    rule.trailing_pct = Some(5.0);
    rule.highest_price = Some(15.0);
    let store = FakeStopRuleStore {
        state: Arc::new(Mutex::new(FakeStopRuleState {
            rules: vec![rule],
            history: Vec::new(),
            removed_codes: Vec::new(),
        })),
    };
    let service = MonitorService::new(
        FakeMonitorWatchlistReader {
            items: vec![monitor_watchlist_item("000001", "core", &[])],
        },
        FakeMonitorQuoteReader {
            rows: vec![monitor_quote_row("000001", 16.8, 3.1)],
        },
        FakeMonitorAlertStore::default(),
    );

    let output = execute_monitor_command_with_stop_store(
        MonitorCommands::Watchlist {
            once: true,
            repeat: false,
        },
        &service,
        &store,
    )
    .await
    .unwrap();

    match output {
        MonitorCommandOutput::Watchlist {
            snapshot: _,
            triggered_stops,
        } => {
            assert!(triggered_stops.is_empty());
        }
        other => panic!("unexpected output: {:?}", other),
    }

    let state = store.state.lock().unwrap();
    assert_eq!(state.rules[0].highest_price, Some(16.8));
}

#[tokio::test]
async fn test_execute_monitor_stop_trailing_triggers_after_drawdown() {
    let mut rule = stop_rule("000001");
    rule.stop_loss_price = None;
    rule.trailing_pct = Some(5.0);
    rule.highest_price = Some(20.0);
    let store = FakeStopRuleStore {
        state: Arc::new(Mutex::new(FakeStopRuleState {
            rules: vec![rule],
            history: Vec::new(),
            removed_codes: Vec::new(),
        })),
    };
    let service = MonitorService::new(
        FakeMonitorWatchlistReader {
            items: vec![monitor_watchlist_item("000001", "core", &[])],
        },
        FakeMonitorQuoteReader {
            rows: vec![monitor_quote_row("000001", 18.8, -3.4)],
        },
        FakeMonitorAlertStore::default(),
    );

    let output = execute_monitor_command_with_stop_store(
        MonitorCommands::Watchlist {
            once: true,
            repeat: false,
        },
        &service,
        &store,
    )
    .await
    .unwrap();

    match output {
        MonitorCommandOutput::Watchlist {
            snapshot: _,
            triggered_stops,
        } => {
            assert_eq!(triggered_stops.len(), 1);
            assert_eq!(triggered_stops[0].kind, StopTriggerKind::TrailingLoss);
            assert_eq!(triggered_stops[0].threshold_price, 19.0);
            assert_eq!(triggered_stops[0].highest_price, Some(20.0));
        }
        other => panic!("unexpected output: {:?}", other),
    }

    let state = store.state.lock().unwrap();
    assert_eq!(
        state.rules[0].last_triggered_at,
        Some(monitor_sample_time())
    );
}

#[tokio::test]
async fn test_execute_monitor_stop_missing_prices_do_not_trigger() {
    let store = FakeStopRuleStore {
        state: Arc::new(Mutex::new(FakeStopRuleState {
            rules: vec![stop_rule("000001")],
            history: Vec::new(),
            removed_codes: Vec::new(),
        })),
    };
    let service = MonitorService::new(
        FakeMonitorWatchlistReader {
            items: vec![monitor_watchlist_item("000001", "core", &[])],
        },
        FakeMonitorQuoteReader {
            rows: vec![MonitorQuoteRow {
                code: "000001".to_string(),
                group: String::new(),
                tags: Vec::new(),
                last_price: None,
                change_pct: None,
                quote_time: Some(monitor_sample_time()),
                note: Some("quote unavailable".to_string()),
            }],
        },
        FakeMonitorAlertStore::default(),
    );

    let output = execute_monitor_command_with_stop_store(
        MonitorCommands::Watchlist {
            once: true,
            repeat: false,
        },
        &service,
        &store,
    )
    .await
    .unwrap();

    match output {
        MonitorCommandOutput::Watchlist {
            snapshot: _,
            triggered_stops,
        } => {
            assert!(triggered_stops.is_empty());
        }
        other => panic!("unexpected output: {:?}", other),
    }

    let state = store.state.lock().unwrap();
    assert_eq!(state.rules[0].highest_price, None);
    assert_eq!(state.rules[0].last_triggered_at, None);
}
