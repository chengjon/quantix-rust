use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use quantix_cli::core::Result;
use quantix_cli::monitor::{
    MonitorAlertStore, MonitorQuoteReader, MonitorQuoteRow, MonitorService,
    MonitorWatchlistReader, PriceAlert, PriceAlertKind,
};
use quantix_cli::watchlist::WatchlistListItem;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct FakeWatchlistReader {
    items: Vec<WatchlistListItem>,
}

#[async_trait]
impl MonitorWatchlistReader for FakeWatchlistReader {
    async fn list_items(&self) -> Result<Vec<WatchlistListItem>> {
        Ok(self.items.clone())
    }
}

#[derive(Clone)]
struct FakeQuoteReader {
    quotes: Vec<MonitorQuoteRow>,
}

#[async_trait]
impl MonitorQuoteReader for FakeQuoteReader {
    async fn load_quotes(&self, _codes: &[String]) -> Result<Vec<MonitorQuoteRow>> {
        Ok(self.quotes.clone())
    }
}

#[derive(Clone, Default)]
struct FakeAlertStore {
    state: Arc<Mutex<FakeAlertState>>,
}

#[derive(Default)]
struct FakeAlertState {
    next_id: i64,
    alerts: Vec<PriceAlert>,
}

#[async_trait]
impl MonitorAlertStore for FakeAlertStore {
    async fn add_alert(
        &self,
        code: &str,
        kind: PriceAlertKind,
        target_price: f64,
        now: DateTime<Utc>,
    ) -> Result<PriceAlert> {
        let mut state = self.state.lock().unwrap();
        state.next_id += 1;
        let alert = PriceAlert {
            id: state.next_id,
            code: code.to_string(),
            kind,
            target_price,
            created_at: now,
            last_triggered_at: None,
        };
        state.alerts.push(alert.clone());
        Ok(alert)
    }

    async fn list_alerts(&self) -> Result<Vec<PriceAlert>> {
        Ok(self.state.lock().unwrap().alerts.clone())
    }

    async fn remove_alert(&self, id: i64) -> Result<bool> {
        let mut state = self.state.lock().unwrap();
        let before = state.alerts.len();
        state.alerts.retain(|alert| alert.id != id);
        Ok(before != state.alerts.len())
    }
}

fn sample_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 9, 30, 0).unwrap()
}

fn watchlist_item(code: &str, group: &str, tags: &[&str]) -> WatchlistListItem {
    WatchlistListItem {
        code: code.to_string(),
        group: group.to_string(),
        tags: tags.iter().map(|tag| tag.to_string()).collect(),
    }
}

fn quote_row(code: &str, last_price: f64, change_pct: f64) -> MonitorQuoteRow {
    MonitorQuoteRow {
        code: code.to_string(),
        group: String::new(),
        tags: Vec::new(),
        last_price: Some(last_price),
        change_pct: Some(change_pct),
        quote_time: Some(sample_time()),
        note: None,
    }
}

fn price_alert(id: i64, code: &str, kind: PriceAlertKind, target_price: f64) -> PriceAlert {
    PriceAlert {
        id,
        code: code.to_string(),
        kind,
        target_price,
        created_at: sample_time(),
        last_triggered_at: None,
    }
}

#[tokio::test]
async fn watchlist_once_builds_rows_from_watchlist_items_plus_current_quotes() {
    let service = MonitorService::new(
        FakeWatchlistReader {
            items: vec![
                watchlist_item("000001", "core", &["bank"]),
                watchlist_item("000002", "swing", &["tech", "ai"]),
            ],
        },
        FakeQuoteReader {
            quotes: vec![quote_row("000001", 15.2, 1.8), quote_row("000002", 22.5, 3.1)],
        },
        FakeAlertStore::default(),
    );

    let snapshot = service.load_watchlist_snapshot().await.unwrap();

    assert_eq!(snapshot.rows.len(), 2);
    assert_eq!(snapshot.rows[0].code, "000001");
    assert_eq!(snapshot.rows[0].group, "core");
    assert_eq!(snapshot.rows[0].tags, vec!["bank".to_string()]);
    assert_eq!(snapshot.rows[0].last_price, Some(15.2));
    assert_eq!(snapshot.rows[1].code, "000002");
    assert_eq!(snapshot.rows[1].group, "swing");
    assert_eq!(
        snapshot.rows[1].tags,
        vec!["tech".to_string(), "ai".to_string()]
    );
    assert_eq!(snapshot.rows[1].last_price, Some(22.5));
    assert!(snapshot.triggered_alerts.is_empty());
}

#[tokio::test]
async fn matching_above_alerts_are_returned_as_triggered_alerts() {
    let store = FakeAlertStore::default();
    store
        .state
        .lock()
        .unwrap()
        .alerts
        .push(price_alert(1, "000001", PriceAlertKind::Above, 16.0));

    let service = MonitorService::new(
        FakeWatchlistReader {
            items: vec![watchlist_item("000001", "core", &[])],
        },
        FakeQuoteReader {
            quotes: vec![quote_row("000001", 16.2, 2.0)],
        },
        store,
    );

    let snapshot = service.load_watchlist_snapshot().await.unwrap();

    assert_eq!(snapshot.triggered_alerts.len(), 1);
    assert_eq!(snapshot.triggered_alerts[0].alert_id, 1);
    assert_eq!(snapshot.triggered_alerts[0].code, "000001");
    assert_eq!(snapshot.triggered_alerts[0].current_price, 16.2);
    assert_eq!(snapshot.triggered_alerts[0].target_price, 16.0);
}

#[tokio::test]
async fn matching_below_alerts_are_returned_as_triggered_alerts() {
    let store = FakeAlertStore::default();
    store
        .state
        .lock()
        .unwrap()
        .alerts
        .push(price_alert(7, "000002", PriceAlertKind::Below, 12.0));

    let service = MonitorService::new(
        FakeWatchlistReader {
            items: vec![watchlist_item("000002", "dip", &["mean_reversion"])],
        },
        FakeQuoteReader {
            quotes: vec![quote_row("000002", 11.8, -2.4)],
        },
        store,
    );

    let snapshot = service.load_watchlist_snapshot().await.unwrap();

    assert_eq!(snapshot.triggered_alerts.len(), 1);
    assert_eq!(snapshot.triggered_alerts[0].alert_id, 7);
    assert_eq!(snapshot.triggered_alerts[0].code, "000002");
    assert_eq!(snapshot.triggered_alerts[0].current_price, 11.8);
    assert_eq!(snapshot.triggered_alerts[0].target_price, 12.0);
}

#[tokio::test]
async fn missing_quote_rows_do_not_panic_and_produce_readable_partial_output() {
    let service = MonitorService::new(
        FakeWatchlistReader {
            items: vec![
                watchlist_item("000001", "core", &[]),
                watchlist_item("000003", "core", &["watch"]),
            ],
        },
        FakeQuoteReader {
            quotes: vec![quote_row("000001", 18.6, 1.1)],
        },
        FakeAlertStore::default(),
    );

    let snapshot = service.load_watchlist_snapshot().await.unwrap();

    assert_eq!(snapshot.rows.len(), 2);
    let missing = snapshot
        .rows
        .iter()
        .find(|row| row.code == "000003")
        .unwrap();
    assert_eq!(missing.last_price, None);
    assert_eq!(missing.change_pct, None);
    assert_eq!(missing.note.as_deref(), Some("quote unavailable"));
    assert!(
        snapshot
            .warnings
            .iter()
            .any(|warning| warning.contains("000003") && warning.contains("quote unavailable"))
    );
}

#[tokio::test]
async fn empty_watchlist_returns_an_empty_result_without_crashing() {
    let service = MonitorService::new(
        FakeWatchlistReader { items: Vec::new() },
        FakeQuoteReader { quotes: Vec::new() },
        FakeAlertStore::default(),
    );

    let snapshot = service.load_watchlist_snapshot().await.unwrap();

    assert!(snapshot.rows.is_empty());
    assert!(snapshot.triggered_alerts.is_empty());
    assert!(snapshot.warnings.is_empty());
}
