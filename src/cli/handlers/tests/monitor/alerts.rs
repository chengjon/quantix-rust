use super::monitor_helpers::*;
use super::*;

#[tokio::test]
async fn test_execute_monitor_alert_add_above_succeeds() {
    let store = FakeMonitorAlertStore::default();
    let service = MonitorService::new(
        FakeMonitorWatchlistReader::default(),
        FakeMonitorQuoteReader::default(),
        store.clone(),
    );

    let output = execute_monitor_command_with_service(
        MonitorCommands::Alert(MonitorAlertCommands::Add {
            code: "000001".to_string(),
            above: Some(16.0),
            below: None,
        }),
        &service,
    )
    .await
    .unwrap();

    match output {
        MonitorCommandOutput::AlertAdded(alert) => {
            assert_eq!(alert.code, "000001");
            assert_eq!(alert.kind, PriceAlertKind::Above);
            assert_eq!(alert.target_price, 16.0);
        }
        other => panic!("unexpected output: {:?}", other),
    }

    assert_eq!(store.state.lock().unwrap().alerts.len(), 1);
}

#[tokio::test]
async fn test_execute_monitor_alert_add_below_succeeds() {
    let store = FakeMonitorAlertStore::default();
    let service = MonitorService::new(
        FakeMonitorWatchlistReader::default(),
        FakeMonitorQuoteReader::default(),
        store.clone(),
    );

    let output = execute_monitor_command_with_service(
        MonitorCommands::Alert(MonitorAlertCommands::Add {
            code: "000001".to_string(),
            above: None,
            below: Some(15.0),
        }),
        &service,
    )
    .await
    .unwrap();

    match output {
        MonitorCommandOutput::AlertAdded(alert) => {
            assert_eq!(alert.code, "000001");
            assert_eq!(alert.kind, PriceAlertKind::Below);
            assert_eq!(alert.target_price, 15.0);
        }
        other => panic!("unexpected output: {:?}", other),
    }

    assert_eq!(store.state.lock().unwrap().alerts.len(), 1);
}

#[tokio::test]
async fn test_execute_monitor_alert_list_returns_persisted_rows() {
    let service = MonitorService::new(
        FakeMonitorWatchlistReader::default(),
        FakeMonitorQuoteReader::default(),
        FakeMonitorAlertStore {
            state: Arc::new(Mutex::new(FakeMonitorAlertState {
                next_id: 2,
                alerts: vec![
                    monitor_alert(1, "000001", PriceAlertKind::Above, 16.0),
                    monitor_alert(2, "000002", PriceAlertKind::Below, 15.0),
                ],
                removed_ids: Vec::new(),
            })),
        },
    );

    let output = execute_monitor_command_with_service(
        MonitorCommands::Alert(MonitorAlertCommands::List),
        &service,
    )
    .await
    .unwrap();

    match output {
        MonitorCommandOutput::AlertList(alerts) => {
            assert_eq!(alerts.len(), 2);
            assert_eq!(alerts[0].code, "000001");
            assert_eq!(alerts[1].kind, PriceAlertKind::Below);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_monitor_alert_remove_succeeds() {
    let store = FakeMonitorAlertStore {
        state: Arc::new(Mutex::new(FakeMonitorAlertState {
            next_id: 1,
            alerts: vec![monitor_alert(1, "000001", PriceAlertKind::Above, 16.0)],
            removed_ids: Vec::new(),
        })),
    };
    let service = MonitorService::new(
        FakeMonitorWatchlistReader::default(),
        FakeMonitorQuoteReader::default(),
        store.clone(),
    );

    let output = execute_monitor_command_with_service(
        MonitorCommands::Alert(MonitorAlertCommands::Remove { id: 1 }),
        &service,
    )
    .await
    .unwrap();

    match output {
        MonitorCommandOutput::AlertRemoved { id, removed } => {
            assert_eq!(id, 1);
            assert!(removed);
        }
        other => panic!("unexpected output: {:?}", other),
    }

    let state = store.state.lock().unwrap();
    assert!(state.alerts.is_empty());
    assert_eq!(state.removed_ids, vec![1]);
}

#[tokio::test]
async fn test_execute_monitor_alert_add_rejects_invalid_threshold_combinations() {
    let service = MonitorService::new(
        FakeMonitorWatchlistReader::default(),
        FakeMonitorQuoteReader::default(),
        FakeMonitorAlertStore::default(),
    );

    let both_err = execute_monitor_command_with_service(
        MonitorCommands::Alert(MonitorAlertCommands::Add {
            code: "000001".to_string(),
            above: Some(16.0),
            below: Some(15.0),
        }),
        &service,
    )
    .await
    .unwrap_err();
    assert!(matches!(both_err, QuantixError::Other(_)));
    assert!(both_err.to_string().contains("必须且只能指定"));

    let none_err = execute_monitor_command_with_service(
        MonitorCommands::Alert(MonitorAlertCommands::Add {
            code: "000001".to_string(),
            above: None,
            below: None,
        }),
        &service,
    )
    .await
    .unwrap_err();
    assert!(matches!(none_err, QuantixError::Other(_)));
    assert!(none_err.to_string().contains("必须且只能指定"));
}

#[tokio::test]
async fn test_execute_monitor_persist_triggered_alerts_falls_back_to_observed_time() {
    let store = FakeMonitorAlertStore {
        state: Arc::new(Mutex::new(FakeMonitorAlertState {
            next_id: 1,
            alerts: vec![monitor_alert(1, "000001", PriceAlertKind::Above, 16.0)],
            removed_ids: Vec::new(),
        })),
    };
    let observed_at = Utc.with_ymd_and_hms(2026, 3, 11, 10, 31, 0).unwrap();
    let snapshot = MonitorWatchlistSnapshot {
        rows: Vec::new(),
        triggered_alerts: vec![TriggeredAlert {
            alert_id: 1,
            code: "000001".to_string(),
            kind: PriceAlertKind::Above,
            target_price: 16.0,
            current_price: 16.8,
            triggered_at: None,
        }],
        warnings: Vec::new(),
    };

    persist_triggered_monitor_alerts(&store, &snapshot, observed_at)
        .await
        .unwrap();

    let alerts = store.state.lock().unwrap().alerts.clone();
    assert_eq!(alerts[0].last_triggered_at, Some(observed_at));
}

#[tokio::test]
async fn test_execute_monitor_persist_triggered_alerts_preserves_snapshot_time() {
    let store = FakeMonitorAlertStore {
        state: Arc::new(Mutex::new(FakeMonitorAlertState {
            next_id: 1,
            alerts: vec![monitor_alert(1, "000001", PriceAlertKind::Above, 16.0)],
            removed_ids: Vec::new(),
        })),
    };
    let observed_at = Utc.with_ymd_and_hms(2026, 3, 11, 10, 31, 0).unwrap();
    let snapshot = MonitorWatchlistSnapshot {
        rows: Vec::new(),
        triggered_alerts: vec![TriggeredAlert {
            alert_id: 1,
            code: "000001".to_string(),
            kind: PriceAlertKind::Above,
            target_price: 16.0,
            current_price: 16.8,
            triggered_at: Some(monitor_sample_time()),
        }],
        warnings: Vec::new(),
    };

    persist_triggered_monitor_alerts(&store, &snapshot, observed_at)
        .await
        .unwrap();

    let alerts = store.state.lock().unwrap().alerts.clone();
    assert_eq!(alerts[0].last_triggered_at, Some(monitor_sample_time()));
}

#[test]
fn test_execute_monitor_config_show_returns_default_config() {
    let dir = tempdir().unwrap();
    let store = JsonMonitorConfigStore::new(dir.path().join("monitor-config.json"));

    let output =
        execute_monitor_config_command_with_store(MonitorConfigCommands::Show, &store).unwrap();

    match output {
        MonitorCommandOutput::Config(config) => {
            assert_eq!(config.interval_seconds, 30);
            assert_eq!(config.watchlist_group, None);
            assert!(config.persist_events);
            assert_eq!(config.max_event_history, 1000);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[test]
fn test_execute_monitor_config_set_updates_persisted_values() {
    let dir = tempdir().unwrap();
    let store = JsonMonitorConfigStore::new(dir.path().join("monitor-config.json"));

    let output = execute_monitor_config_command_with_store(
        MonitorConfigCommands::Set {
            interval_seconds: Some(15),
            group: None,
            persist_events: None,
            notify: None,
        },
        &store,
    )
    .unwrap();

    match output {
        MonitorCommandOutput::Config(config) => {
            assert_eq!(config.interval_seconds, 15);
        }
        other => panic!("unexpected output: {:?}", other),
    }

    let reloaded = store.load_or_create().unwrap();
    assert_eq!(reloaded.interval_seconds, 15);
}
