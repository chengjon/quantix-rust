use super::*;
use crate::cli::{
    MonitorAlertCommands, MonitorCommands, MonitorConfigCommands, MonitorDaemonCommands,
    MonitorEventCommands, MonitorServiceCommands, MonitorServiceConfigCommands, StopCommands,
    StrategyServiceCommands, TradeCommands,
};
use crate::core::QuantixError;
use crate::core::config::{
    CLICKHOUSE_DB_ENV, CLICKHOUSE_PASSWORD_ENV, CLICKHOUSE_URL_ENV, CLICKHOUSE_USER_ENV,
};
use crate::core::runtime::{BRIDGE_API_KEY_ENV, BRIDGE_BASE_URL_ENV, STRATEGY_RUNTIME_DB_PATH_ENV};
use crate::data::models::{AdjustType, Kline};
use crate::market::{
    BoardRankRow, BoardSortBy, BoardType, LeaderFilter, LeaderRow, MarketDataReader,
    MarketSentimentSnapshot, NorthFlowSnapshot,
};
use crate::monitor::{
    JsonMonitorConfigStore, JsonMonitorServiceConfigStore, MonitorAlertStore, MonitorEventType,
    MonitorQuoteReader, MonitorQuoteRow, MonitorRunMode, MonitorRunner, MonitorService,
    MonitorServiceConfig, MonitorServiceStatusSummary, MonitorWatchlistReader, PriceAlert,
    PriceAlertKind, SqliteMonitorAlertStore, TriggeredAlert,
};
use crate::screener::DailyKlineLoader;
use crate::stop::{StopRule, StopRuleStore, StopService, StopTriggerKind};
use crate::strategy::runtime::StrategyBarLoader;
use crate::trade::{
    JsonPaperTradeStore, PaperTradeState, PaperTradeStore, TradeService, TradeSide,
};
use crate::watchlist::{WatchlistListItem, WatchlistQuoteLookup, WatchlistQuoteSnapshot};
use crate::{execution::runtime_store::StrategyRuntimeStore, risk::JsonRiskStore};
use async_trait::async_trait;
use chrono::{NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tempfile::tempdir;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

struct ClickHouseDbEnvGuard {
    url: Option<String>,
    database: Option<String>,
    user: Option<String>,
    password: Option<String>,
}

impl ClickHouseDbEnvGuard {
    fn capture() -> Self {
        Self {
            url: std::env::var(CLICKHOUSE_URL_ENV).ok(),
            database: std::env::var(CLICKHOUSE_DB_ENV).ok(),
            user: std::env::var(CLICKHOUSE_USER_ENV).ok(),
            password: std::env::var(CLICKHOUSE_PASSWORD_ENV).ok(),
        }
    }
}

impl Drop for ClickHouseDbEnvGuard {
    fn drop(&mut self) {
        match &self.url {
            Some(value) => unsafe { std::env::set_var(CLICKHOUSE_URL_ENV, value) },
            None => unsafe { std::env::remove_var(CLICKHOUSE_URL_ENV) },
        }

        match &self.database {
            Some(value) => unsafe { std::env::set_var(CLICKHOUSE_DB_ENV, value) },
            None => unsafe { std::env::remove_var(CLICKHOUSE_DB_ENV) },
        }

        match &self.user {
            Some(value) => unsafe { std::env::set_var(CLICKHOUSE_USER_ENV, value) },
            None => unsafe { std::env::remove_var(CLICKHOUSE_USER_ENV) },
        }

        match &self.password {
            Some(value) => unsafe { std::env::set_var(CLICKHOUSE_PASSWORD_ENV, value) },
            None => unsafe { std::env::remove_var(CLICKHOUSE_PASSWORD_ENV) },
        }
    }
}

struct RuntimeEnvGuard {
    strategy_runtime_db_path: Option<String>,
    bridge_base_url: Option<String>,
    bridge_api_key: Option<String>,
}

impl RuntimeEnvGuard {
    fn capture() -> Self {
        Self {
            strategy_runtime_db_path: std::env::var(STRATEGY_RUNTIME_DB_PATH_ENV).ok(),
            bridge_base_url: std::env::var(BRIDGE_BASE_URL_ENV).ok(),
            bridge_api_key: std::env::var(BRIDGE_API_KEY_ENV).ok(),
        }
    }
}

struct NotificationEnvGuard {
    monitor_notify: Option<String>,
    notification_log_path: Option<String>,
    notification_min_level: Option<String>,
    webhook_url: Option<String>,
    wechat_work_webhook_url: Option<String>,
    feishu_webhook_url: Option<String>,
    telegram_bot_token: Option<String>,
    telegram_chat_id: Option<String>,
    discord_webhook_url: Option<String>,
    slack_webhook_url: Option<String>,
    dingtalk_webhook_url: Option<String>,
    pushplus_token: Option<String>,
}

impl NotificationEnvGuard {
    fn capture() -> Self {
        Self {
            monitor_notify: std::env::var("QUANTIX_MONITOR_NOTIFY").ok(),
            notification_log_path: std::env::var("NOTIFICATION_LOG_PATH").ok(),
            notification_min_level: std::env::var("NOTIFICATION_MIN_LEVEL").ok(),
            webhook_url: std::env::var("WEBHOOK_URL").ok(),
            wechat_work_webhook_url: std::env::var("WECHAT_WORK_WEBHOOK_URL").ok(),
            feishu_webhook_url: std::env::var("FEISHU_WEBHOOK_URL").ok(),
            telegram_bot_token: std::env::var("TELEGRAM_BOT_TOKEN").ok(),
            telegram_chat_id: std::env::var("TELEGRAM_CHAT_ID").ok(),
            discord_webhook_url: std::env::var("DISCORD_WEBHOOK_URL").ok(),
            slack_webhook_url: std::env::var("SLACK_WEBHOOK_URL").ok(),
            dingtalk_webhook_url: std::env::var("DINGTALK_WEBHOOK_URL").ok(),
            pushplus_token: std::env::var("PUSHPLUS_TOKEN").ok(),
        }
    }
}

impl Drop for NotificationEnvGuard {
    fn drop(&mut self) {
        restore_optional_env("QUANTIX_MONITOR_NOTIFY", &self.monitor_notify);
        restore_optional_env("NOTIFICATION_LOG_PATH", &self.notification_log_path);
        restore_optional_env("NOTIFICATION_MIN_LEVEL", &self.notification_min_level);
        restore_optional_env("WEBHOOK_URL", &self.webhook_url);
        restore_optional_env("WECHAT_WORK_WEBHOOK_URL", &self.wechat_work_webhook_url);
        restore_optional_env("FEISHU_WEBHOOK_URL", &self.feishu_webhook_url);
        restore_optional_env("TELEGRAM_BOT_TOKEN", &self.telegram_bot_token);
        restore_optional_env("TELEGRAM_CHAT_ID", &self.telegram_chat_id);
        restore_optional_env("DISCORD_WEBHOOK_URL", &self.discord_webhook_url);
        restore_optional_env("SLACK_WEBHOOK_URL", &self.slack_webhook_url);
        restore_optional_env("DINGTALK_WEBHOOK_URL", &self.dingtalk_webhook_url);
        restore_optional_env("PUSHPLUS_TOKEN", &self.pushplus_token);
    }
}

fn restore_optional_env(key: &str, value: &Option<String>) {
    match value {
        Some(value) => unsafe { std::env::set_var(key, value) },
        None => unsafe { std::env::remove_var(key) },
    }
}

impl Drop for RuntimeEnvGuard {
    fn drop(&mut self) {
        match &self.strategy_runtime_db_path {
            Some(value) => unsafe { std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, value) },
            None => unsafe { std::env::remove_var(STRATEGY_RUNTIME_DB_PATH_ENV) },
        }

        match &self.bridge_base_url {
            Some(value) => unsafe { std::env::set_var(BRIDGE_BASE_URL_ENV, value) },
            None => unsafe { std::env::remove_var(BRIDGE_BASE_URL_ENV) },
        }

        match &self.bridge_api_key {
            Some(value) => unsafe { std::env::set_var(BRIDGE_API_KEY_ENV, value) },
            None => unsafe { std::env::remove_var(BRIDGE_API_KEY_ENV) },
        }
    }
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
mod analyze;
mod market;
mod monitor_runtime;
mod monitor_service;
mod stop;
mod strategy_execution;
mod strategy_instances;
mod strategy_requests;
mod trade;
mod trade_quotes;
use self::stop::stop_rule;
use self::strategy_execution::{FakeLoader, fixed_ts, make_kline};
use self::trade::{FakePaperTradeStore, FakeTradeQuoteLookup, trade_service};

fn monitor_sample_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 10, 30, 0).unwrap()
}

fn monitor_watchlist_item(code: &str, group: &str, tags: &[&str]) -> WatchlistListItem {
    WatchlistListItem {
        code: code.to_string(),
        group: group.to_string(),
        tags: tags.iter().map(|tag| tag.to_string()).collect(),
    }
}

fn monitor_quote_row(code: &str, last_price: f64, change_pct: f64) -> MonitorQuoteRow {
    MonitorQuoteRow {
        code: code.to_string(),
        group: String::new(),
        tags: Vec::new(),
        last_price: Some(last_price),
        change_pct: Some(change_pct),
        quote_time: Some(monitor_sample_time()),
        note: None,
    }
}

fn monitor_alert(id: i64, code: &str, kind: PriceAlertKind, target_price: f64) -> PriceAlert {
    PriceAlert {
        id,
        code: code.to_string(),
        kind,
        target_price,
        created_at: monitor_sample_time(),
        last_triggered_at: None,
    }
}

#[derive(Clone, Default)]
struct FakeMonitorWatchlistReader {
    items: Vec<WatchlistListItem>,
}

#[async_trait]
impl MonitorWatchlistReader for FakeMonitorWatchlistReader {
    async fn list_items(&self) -> Result<Vec<WatchlistListItem>> {
        Ok(self.items.clone())
    }
}

#[derive(Clone, Default)]
struct FakeMonitorQuoteReader {
    rows: Vec<MonitorQuoteRow>,
}

#[async_trait]
impl MonitorQuoteReader for FakeMonitorQuoteReader {
    async fn load_quotes(&self, _codes: &[String]) -> Result<Vec<MonitorQuoteRow>> {
        Ok(self.rows.clone())
    }
}

#[derive(Debug, Clone, Default)]
struct FakeMonitorAlertState {
    next_id: i64,
    alerts: Vec<PriceAlert>,
    removed_ids: Vec<i64>,
}

#[derive(Clone, Default)]
struct FakeMonitorAlertStore {
    state: Arc<Mutex<FakeMonitorAlertState>>,
}

#[derive(Debug, Clone, Default)]
struct FakeStopRuleState {
    rules: Vec<StopRule>,
    history: Vec<crate::stop::StopHistoryEvent>,
    removed_codes: Vec<String>,
}

#[derive(Clone, Default)]
struct FakeStopRuleStore {
    state: Arc<Mutex<FakeStopRuleState>>,
}

#[async_trait]
impl StopRuleStore for FakeStopRuleStore {
    async fn upsert_rule(&self, rule: StopRule) -> Result<StopRule> {
        let mut state = self.state.lock().unwrap();
        if let Some(existing) = state
            .rules
            .iter_mut()
            .find(|existing| existing.code == rule.code)
        {
            *existing = rule.clone();
        } else {
            state.rules.push(rule.clone());
        }
        Ok(rule)
    }

    async fn list_rules(&self) -> Result<Vec<StopRule>> {
        Ok(self.state.lock().unwrap().rules.clone())
    }

    async fn get_rule(&self, code: &str) -> Result<Option<StopRule>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .rules
            .iter()
            .find(|rule| rule.code == code)
            .cloned())
    }

    async fn append_history(&self, _event: crate::stop::StopHistoryEvent) -> Result<()> {
        self.state.lock().unwrap().history.push(_event);
        Ok(())
    }

    async fn list_history(
        &self,
        _filter: crate::stop::StopHistoryFilter,
    ) -> Result<Vec<crate::stop::StopHistoryEvent>> {
        Ok(self.state.lock().unwrap().history.clone())
    }

    async fn remove_rule(&self, code: &str) -> Result<bool> {
        let mut state = self.state.lock().unwrap();
        let before = state.rules.len();
        state.rules.retain(|rule| rule.code != code);
        if before != state.rules.len() {
            state.removed_codes.push(code.to_string());
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

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

#[tokio::test]
async fn test_execute_strategy_request_execute_rejects_qmt_live_with_manual_bridge_guidance() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_store = JsonRiskStore::new(dir.path().join("risk_state.json"));

    let run = crate::execution::models::StrategyRunRecord {
        run_id: uuid::Uuid::new_v4().to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "signal".to_string(),
        trigger: "daemon".to_string(),
        status: crate::execution::models::StrategyRunStatus::Running,
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        started_at: fixed_ts(),
        finished_at: None,
        metadata_json: serde_json::json!({}),
    };
    runtime_store.insert_run(&run).await.unwrap();

    let signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-request-qmt-live".to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: run.run_id.clone(),
        metadata_json: json!({
            "market_price": "12.34",
            "signal_value": "buy",
            "execution_policy": {
                "fixed_cash_per_buy": "10000",
                "slippage_bps": 0
            },
            "bar_source_id": "test-primary",
            "bar_source_fallback": false
        }),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
    };
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = execute_strategy_signal_approve_with_store(
        &runtime_store,
        "signal-request-qmt-live",
        "qmt_live",
        "default",
    )
    .await
    .unwrap();

    let err = execute_strategy_request_execute_with_components(
        &runtime_store,
        &request.request_id,
        trade_store,
        risk_store,
    )
    .await
    .unwrap_err();

    let message = err.to_string();
    assert!(message.contains("qmt_live"));
    assert!(message.contains("execution bridge qmt-live"));
}
