use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use quantix_cli::core::Result;
use quantix_cli::monitor::{
    MonitorAlertStore, MonitorConfig, MonitorEventFilter, MonitorEventType, MonitorQuoteReader,
    MonitorQuoteRow, MonitorRunMode, MonitorRunner, MonitorWatchlistReader,
    SqliteMonitorAlertStore,
};
use quantix_cli::stop::{StopRule, StopRuleStore, StopTriggerKind};
use quantix_cli::watchlist::WatchlistListItem;
use tempfile::tempdir;

#[derive(Clone, Default)]
struct FakeWatchlistReader {
    items: Vec<WatchlistListItem>,
}

#[async_trait]
impl MonitorWatchlistReader for FakeWatchlistReader {
    async fn list_items(&self) -> Result<Vec<WatchlistListItem>> {
        Ok(self.items.clone())
    }
}

#[derive(Clone, Default)]
struct FakeQuoteReader {
    rows: Vec<MonitorQuoteRow>,
}

#[async_trait]
impl MonitorQuoteReader for FakeQuoteReader {
    async fn load_quotes(&self, _codes: &[String]) -> Result<Vec<MonitorQuoteRow>> {
        Ok(self.rows.clone())
    }
}

#[derive(Debug, Clone, Default)]
struct FakeStopRuleState {
    rules: Vec<StopRule>,
}

#[derive(Clone, Default)]
struct FakeStopRuleStore {
    state: Arc<Mutex<FakeStopRuleState>>,
}

#[async_trait]
impl StopRuleStore for FakeStopRuleStore {
    async fn upsert_rule(&self, rule: StopRule) -> Result<StopRule> {
        let mut state = self.state.lock().unwrap();
        if let Some(existing) = state.rules.iter_mut().find(|existing| existing.code == rule.code) {
            *existing = rule.clone();
        } else {
            state.rules.push(rule.clone());
        }
        Ok(rule)
    }

    async fn list_rules(&self) -> Result<Vec<StopRule>> {
        Ok(self.state.lock().unwrap().rules.clone())
    }

    async fn remove_rule(&self, code: &str) -> Result<bool> {
        let mut state = self.state.lock().unwrap();
        let before = state.rules.len();
        state.rules.retain(|rule| rule.code != code);
        Ok(before != state.rules.len())
    }
}

fn sample_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 16, 10, 0, 0).unwrap()
}

fn item(code: &str, group: &str) -> WatchlistListItem {
    WatchlistListItem {
        code: code.to_string(),
        group: group.to_string(),
        tags: Vec::new(),
    }
}

fn quote_row(code: &str, group: &str, price: Option<f64>) -> MonitorQuoteRow {
    MonitorQuoteRow {
        code: code.to_string(),
        group: group.to_string(),
        tags: Vec::new(),
        last_price: price,
        change_pct: None,
        quote_time: Some(sample_time()),
        note: None,
    }
}

fn stop_rule(code: &str) -> StopRule {
    StopRule {
        code: code.to_string(),
        stop_loss_price: Some(14.5),
        take_profit_price: None,
        trailing_pct: None,
        highest_price: None,
        last_triggered_at: None,
        created_at: sample_time(),
        updated_at: sample_time(),
    }
}

fn config() -> MonitorConfig {
    MonitorConfig {
        interval_seconds: 30,
        watchlist_group: None,
        persist_events: true,
        max_event_history: 1000,
    }
}

#[tokio::test]
async fn monitor_runner_empty_watchlist_returns_no_data() {
    let dir = tempdir().unwrap();
    let store = SqliteMonitorAlertStore::new(dir.path().join("alerts.db"))
        .await
        .unwrap();
    let runner = MonitorRunner::new(
        FakeWatchlistReader::default(),
        FakeQuoteReader::default(),
        store,
        FakeStopRuleStore::default(),
    );

    let output = runner
        .run_once(&config(), MonitorRunMode::Foreground, sample_time())
        .await
        .unwrap();

    assert!(output.snapshot.rows.is_empty());
    assert!(output.triggered_stops.is_empty());
    assert!(output.new_events.is_empty());
}

#[tokio::test]
async fn monitor_runner_partial_quotes_do_not_abort_iteration() {
    let dir = tempdir().unwrap();
    let store = SqliteMonitorAlertStore::new(dir.path().join("alerts.db"))
        .await
        .unwrap();
    let runner = MonitorRunner::new(
        FakeWatchlistReader {
            items: vec![item("000001", "core"), item("000002", "core")],
        },
        FakeQuoteReader {
            rows: vec![quote_row("000001", "core", Some(15.2))],
        },
        store,
        FakeStopRuleStore::default(),
    );

    let output = runner
        .run_once(&config(), MonitorRunMode::Foreground, sample_time())
        .await
        .unwrap();

    assert_eq!(output.snapshot.rows.len(), 2);
    assert!(output
        .snapshot
        .warnings
        .iter()
        .any(|warning| warning.contains("000002")));
}

#[tokio::test]
async fn monitor_runner_persists_alert_event_only_on_first_activation() {
    let dir = tempdir().unwrap();
    let store = SqliteMonitorAlertStore::new(dir.path().join("alerts.db"))
        .await
        .unwrap();
    store
        .add_alert(
            "000001",
            quantix_cli::monitor::PriceAlertKind::Above,
            15.0,
            sample_time(),
        )
        .await
        .unwrap();

    let runner = MonitorRunner::new(
        FakeWatchlistReader {
            items: vec![item("000001", "core")],
        },
        FakeQuoteReader {
            rows: vec![quote_row("000001", "core", Some(15.2))],
        },
        store.clone(),
        FakeStopRuleStore::default(),
    );

    let first = runner
        .run_once(&config(), MonitorRunMode::Daemon, sample_time())
        .await
        .unwrap();
    let second = runner
        .run_once(&config(), MonitorRunMode::Daemon, sample_time())
        .await
        .unwrap();

    assert_eq!(first.new_events.len(), 1);
    assert!(second.new_events.is_empty());

    let rows = store
        .list_events(&MonitorEventFilter {
            limit: 20,
            code: None,
            event_type: Some(MonitorEventType::PriceAlert),
        })
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
}

#[tokio::test]
async fn monitor_runner_retriggers_after_condition_clears() {
    let dir = tempdir().unwrap();
    let store = SqliteMonitorAlertStore::new(dir.path().join("alerts.db"))
        .await
        .unwrap();
    store
        .add_alert(
            "000001",
            quantix_cli::monitor::PriceAlertKind::Above,
            15.0,
            sample_time(),
        )
        .await
        .unwrap();

    let watchlist = FakeWatchlistReader {
        items: vec![item("000001", "core")],
    };
    let runner_on = MonitorRunner::new(
        watchlist.clone(),
        FakeQuoteReader {
            rows: vec![quote_row("000001", "core", Some(15.2))],
        },
        store.clone(),
        FakeStopRuleStore::default(),
    );
    let runner_off = MonitorRunner::new(
        watchlist,
        FakeQuoteReader {
            rows: vec![quote_row("000001", "core", Some(14.8))],
        },
        store.clone(),
        FakeStopRuleStore::default(),
    );

    runner_on
        .run_once(&config(), MonitorRunMode::Daemon, sample_time())
        .await
        .unwrap();
    runner_off
        .run_once(&config(), MonitorRunMode::Daemon, sample_time())
        .await
        .unwrap();
    let retrigger = runner_on
        .run_once(&config(), MonitorRunMode::Daemon, sample_time())
        .await
        .unwrap();

    assert_eq!(retrigger.new_events.len(), 1);

    let rows = store
        .list_events(&MonitorEventFilter {
            limit: 20,
            code: None,
            event_type: Some(MonitorEventType::PriceAlert),
        })
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn monitor_runner_persists_stop_trigger_events() {
    let dir = tempdir().unwrap();
    let store = SqliteMonitorAlertStore::new(dir.path().join("alerts.db"))
        .await
        .unwrap();
    let stop_store = FakeStopRuleStore {
        state: Arc::new(Mutex::new(FakeStopRuleState {
            rules: vec![stop_rule("000001")],
        })),
    };
    let runner = MonitorRunner::new(
        FakeWatchlistReader {
            items: vec![item("000001", "core")],
        },
        FakeQuoteReader {
            rows: vec![quote_row("000001", "core", Some(14.2))],
        },
        store.clone(),
        stop_store,
    );

    let output = runner
        .run_once(&config(), MonitorRunMode::Daemon, sample_time())
        .await
        .unwrap();

    assert_eq!(output.triggered_stops.len(), 1);
    assert_eq!(output.triggered_stops[0].kind, StopTriggerKind::Loss);
    assert_eq!(output.new_events.len(), 1);
    assert_eq!(output.new_events[0].event_type, MonitorEventType::StopLoss);
}
