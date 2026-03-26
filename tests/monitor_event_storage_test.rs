use chrono::{TimeZone, Utc};
use quantix_cli::monitor::{
    MonitorEventFilter, MonitorEventType, MonitorRunMode, NewMonitorEvent, SqliteMonitorAlertStore,
};
use tempfile::tempdir;

fn sample_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 16, 9, 30, 0).unwrap()
}

fn sample_event(code: &str, event_type: MonitorEventType) -> NewMonitorEvent {
    NewMonitorEvent {
        event_time: sample_time(),
        event_type,
        code: code.to_string(),
        price: Some(15.2),
        message: format!("{code} triggered"),
        source_type: "price_alert".to_string(),
        source_key: format!("price_alert:{code}"),
        observed_at: Some(sample_time()),
        run_mode: MonitorRunMode::Daemon,
    }
}

#[tokio::test]
async fn monitor_event_storage_creates_event_tables_automatically() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("alerts.db");

    let store = SqliteMonitorAlertStore::new(&db_path).await.unwrap();
    let rows = store
        .list_events(&MonitorEventFilter {
            limit: 20,
            code: None,
            event_type: None,
        })
        .await
        .unwrap();

    assert!(rows.is_empty());
}

#[tokio::test]
async fn monitor_event_storage_dedupes_active_edges() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("alerts.db");
    let store = SqliteMonitorAlertStore::new(&db_path).await.unwrap();

    assert!(
        store
            .record_event_edge(
                "price_alert",
                "price_alert:000001",
                true,
                Some(sample_event("000001", MonitorEventType::PriceAlert)),
                1000,
            )
            .await
            .unwrap()
    );
    assert!(
        !store
            .record_event_edge(
                "price_alert",
                "price_alert:000001",
                true,
                Some(sample_event("000001", MonitorEventType::PriceAlert)),
                1000,
            )
            .await
            .unwrap()
    );

    let rows = store
        .list_events(&MonitorEventFilter {
            limit: 20,
            code: None,
            event_type: None,
        })
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
}

#[tokio::test]
async fn monitor_event_storage_writes_new_row_after_clear_and_retrigger() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("alerts.db");
    let store = SqliteMonitorAlertStore::new(&db_path).await.unwrap();

    store
        .record_event_edge(
            "price_alert",
            "price_alert:000001",
            true,
            Some(sample_event("000001", MonitorEventType::PriceAlert)),
            1000,
        )
        .await
        .unwrap();
    store
        .record_event_edge("price_alert", "price_alert:000001", false, None, 1000)
        .await
        .unwrap();
    store
        .record_event_edge(
            "price_alert",
            "price_alert:000001",
            true,
            Some(sample_event("000001", MonitorEventType::PriceAlert)),
            1000,
        )
        .await
        .unwrap();

    let rows = store
        .list_events(&MonitorEventFilter {
            limit: 20,
            code: None,
            event_type: None,
        })
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn monitor_event_storage_filters_by_code_type_and_limit() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("alerts.db");
    let store = SqliteMonitorAlertStore::new(&db_path).await.unwrap();

    store
        .record_event_edge(
            "price_alert",
            "price_alert:000001",
            true,
            Some(sample_event("000001", MonitorEventType::PriceAlert)),
            1000,
        )
        .await
        .unwrap();
    store
        .record_event_edge(
            "stop_rule",
            "stop_rule:000002",
            true,
            Some(sample_event("000002", MonitorEventType::StopLoss)),
            1000,
        )
        .await
        .unwrap();

    let rows = store
        .list_events(&MonitorEventFilter {
            limit: 1,
            code: Some("000001".to_string()),
            event_type: Some(MonitorEventType::PriceAlert),
        })
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].code, "000001");
    assert_eq!(rows[0].event_type, MonitorEventType::PriceAlert);
}

#[tokio::test]
async fn monitor_event_storage_trims_history_to_max_rows() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("alerts.db");
    let store = SqliteMonitorAlertStore::new(&db_path).await.unwrap();

    for (idx, code) in ["000001", "000002", "000003"].iter().enumerate() {
        let key = format!("price_alert:{code}");
        let event = NewMonitorEvent {
            event_time: sample_time() + chrono::Duration::minutes(idx as i64),
            event_type: MonitorEventType::PriceAlert,
            code: (*code).to_string(),
            price: Some(10.0 + idx as f64),
            message: format!("{code} triggered"),
            source_type: "price_alert".to_string(),
            source_key: key.clone(),
            observed_at: Some(sample_time() + chrono::Duration::minutes(idx as i64)),
            run_mode: MonitorRunMode::Foreground,
        };
        store
            .record_event_edge("price_alert", &key, true, Some(event), 2)
            .await
            .unwrap();
    }

    let rows = store
        .list_events(&MonitorEventFilter {
            limit: 20,
            code: None,
            event_type: None,
        })
        .await
        .unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].code, "000003");
    assert_eq!(rows[1].code, "000002");
}
