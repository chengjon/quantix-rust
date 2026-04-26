use super::*;

#[tokio::test]
async fn test_execute_monitor_event_list_returns_filtered_rows() {
    let dir = tempdir().unwrap();
    let store = SqliteMonitorAlertStore::new(dir.path().join("alerts.db"))
        .await
        .unwrap();
    store
        .record_event_edge(
            "price_alert",
            "price_alert:000001",
            true,
            Some(crate::monitor::NewMonitorEvent {
                event_time: monitor_sample_time(),
                event_type: MonitorEventType::PriceAlert,
                code: "000001".to_string(),
                price: Some(16.2),
                message: "000001 triggered".to_string(),
                source_type: "price_alert".to_string(),
                source_key: "price_alert:000001".to_string(),
                observed_at: Some(monitor_sample_time()),
                run_mode: MonitorRunMode::Daemon,
            }),
            1000,
        )
        .await
        .unwrap();

    let output = execute_monitor_event_command_with_store(
        MonitorEventCommands::List {
            limit: 10,
            code: Some("000001".to_string()),
            event_type: Some("price-alert".to_string()),
        },
        &store,
    )
    .await
    .unwrap();

    match output {
        MonitorCommandOutput::EventList(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].code, "000001");
            assert_eq!(rows[0].event_type, MonitorEventType::PriceAlert);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_monitor_repeat_uses_runner_in_foreground_mode() {
    let dir = tempdir().unwrap();
    let runner = MonitorRunner::new(
        FakeMonitorWatchlistReader {
            items: vec![monitor_watchlist_item("000001", "core", &[])],
        },
        FakeMonitorQuoteReader {
            rows: vec![monitor_quote_row("000001", 16.8, 3.2)],
        },
        SqliteMonitorAlertStore::new(dir.path().join("alerts.db"))
            .await
            .unwrap(),
        FakeStopRuleStore::default(),
        FakePaperTradeStore::default(),
    );

    let output = execute_monitor_iteration_with_runner(
        MonitorCommands::Watchlist {
            once: false,
            repeat: true,
        },
        &crate::monitor::MonitorConfig::default(),
        &runner,
        monitor_sample_time(),
    )
    .await
    .unwrap();

    match output {
        MonitorCommandOutput::AutomationIteration { run_mode, output } => {
            assert_eq!(run_mode, MonitorRunMode::Foreground);
            assert_eq!(output.snapshot.rows.len(), 1);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_monitor_daemon_run_uses_runner_in_daemon_mode() {
    let dir = tempdir().unwrap();
    let runner = MonitorRunner::new(
        FakeMonitorWatchlistReader {
            items: vec![monitor_watchlist_item("000001", "core", &[])],
        },
        FakeMonitorQuoteReader {
            rows: vec![monitor_quote_row("000001", 16.8, 3.2)],
        },
        SqliteMonitorAlertStore::new(dir.path().join("alerts.db"))
            .await
            .unwrap(),
        FakeStopRuleStore::default(),
        FakePaperTradeStore::default(),
    );

    let output = execute_monitor_iteration_with_runner(
        MonitorCommands::Daemon(MonitorDaemonCommands::Run),
        &crate::monitor::MonitorConfig {
            notify_enabled: true,
            ..crate::monitor::MonitorConfig::default()
        },
        &runner,
        monitor_sample_time(),
    )
    .await
    .unwrap();

    match output {
        MonitorCommandOutput::AutomationIteration { run_mode, output } => {
            assert_eq!(run_mode, MonitorRunMode::Daemon);
            assert_eq!(output.snapshot.rows.len(), 1);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_monitor_iteration_dispatches_notifications_when_enabled() {
    let dir = tempdir().unwrap();
    let log_path = dir.path().join("notifications.log");

    let store = SqliteMonitorAlertStore::new(dir.path().join("alerts.db"))
        .await
        .unwrap();
    store
        .add_alert(
            "000001",
            crate::monitor::PriceAlertKind::Above,
            15.0,
            monitor_sample_time(),
        )
        .await
        .unwrap();
    let runner = MonitorRunner::new(
        FakeMonitorWatchlistReader {
            items: vec![monitor_watchlist_item("000001", "core", &[])],
        },
        FakeMonitorQuoteReader {
            rows: vec![monitor_quote_row("000001", 16.8, 3.2)],
        },
        store,
        FakeStopRuleStore::default(),
        FakePaperTradeStore::default(),
    );

    let output = execute_monitor_iteration_with_runner(
        MonitorCommands::Daemon(MonitorDaemonCommands::Run),
        &crate::monitor::MonitorConfig::default(),
        &runner,
        monitor_sample_time(),
    )
    .await
    .unwrap();

    let iteration = match output {
        MonitorCommandOutput::AutomationIteration { output, .. } => output,
        other => panic!("unexpected output: {:?}", other),
    };
    let mut service =
        crate::monitoring::NotificationService::new(crate::monitoring::NotificationConfig {
            enabled_channels: vec![crate::monitoring::NotificationChannel::Log],
            webhook_url: None,
            log_path: Some(log_path.display().to_string()),
            min_level: crate::monitoring::AlertLevel::Warning,
            quiet_hours: None,
            templates: std::collections::HashMap::new(),
            wechat_work_webhook: None,
            feishu_webhook: None,
        });
    super::super::monitor_handler::send_monitor_notifications_with_service(
        &iteration,
        &mut service,
    )
    .await
    .unwrap();

    let contents = std::fs::read_to_string(&log_path).unwrap();
    assert!(contents.contains("Monitor price alert 000001"));
    assert!(contents.contains("000001 crossed Above 15.00"));
}

#[derive(Default)]
struct FailingMonitorNotificationService {
    attempts: usize,
}

#[async_trait]
impl super::super::monitor_handler::MonitorNotificationSender for FailingMonitorNotificationService {
    async fn notify(&mut self, _notification: crate::monitoring::Notification) -> Result<()> {
        self.attempts += 1;
        Err(QuantixError::Other("notify boom".to_string()))
    }
}

#[tokio::test]
async fn test_send_monitor_notifications_propagates_notify_errors() {
    let mut service = FailingMonitorNotificationService::default();
    let output = crate::monitor::MonitorIterationOutput {
        snapshot: crate::monitor::MonitorWatchlistSnapshot::default(),
        triggered_stops: Vec::new(),
        new_events: vec![crate::monitor::MonitorEventRow {
            id: 1,
            event_time: monitor_sample_time(),
            event_type: MonitorEventType::PriceAlert,
            code: "000001".to_string(),
            price: Some(16.8),
            message: "000001 crossed Above 15.00".to_string(),
            source_type: "price_alert".to_string(),
            source_key: "price_alert:000001".to_string(),
            observed_at: Some(monitor_sample_time()),
            run_mode: MonitorRunMode::Daemon,
        }],
    };

    let err = super::super::monitor_handler::send_monitor_notifications_with_service(
        &output,
        &mut service,
    )
    .await
    .unwrap_err();

    assert_eq!(service.attempts, 1);
    assert!(err.to_string().contains("notify boom"));
}
