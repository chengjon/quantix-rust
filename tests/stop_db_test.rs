use chrono::{DateTime, TimeZone, Utc};
use quantix_cli::stop::{
    SqliteStopHistoryEvent, SqliteStopHistoryFilter, SqliteStopHistoryTriggerKind,
    SqliteStopRuleStore, StopHistoryEventType, StopRule, StopRuleStore,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use tempfile::tempdir;

fn sample_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 12, 0, 0).unwrap()
}

fn sample_rule(code: &str) -> StopRule {
    StopRule {
        code: code.to_string(),
        stop_loss_price: Some(14.5),
        take_profit_price: None,
        stop_loss_pct: Some(5.0),
        take_profit_pct: None,
        trailing_pct: None,
        highest_price: None,
        reference_price: Some(15.2),
        last_triggered_at: None,
        created_at: sample_time(),
        updated_at: sample_time(),
    }
}

#[tokio::test]
async fn stop_db_storage_creates_schema_automatically() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("monitor.db");

    let store = SqliteStopRuleStore::new(&db_path).await.unwrap();
    let rules = store.list_rules().await.unwrap();

    assert!(rules.is_empty());
}

#[tokio::test]
async fn stop_db_storage_upsert_overwrites_rule_by_code() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("monitor.db");
    let store = SqliteStopRuleStore::new(&db_path).await.unwrap();

    let mut initial_rule = sample_rule("000001");
    initial_rule.stop_loss_price = Some(14.5);
    store.upsert_rule(initial_rule).await.unwrap();

    let replacement_rule = StopRule {
        code: "000001".to_string(),
        stop_loss_price: None,
        take_profit_price: Some(18.0),
        stop_loss_pct: None,
        take_profit_pct: None,
        trailing_pct: Some(5.0),
        highest_price: Some(19.8),
        reference_price: None,
        last_triggered_at: Some(sample_time()),
        created_at: sample_time(),
        updated_at: sample_time(),
    };
    store.upsert_rule(replacement_rule.clone()).await.unwrap();

    let rules = store.list_rules().await.unwrap();
    assert_eq!(rules, vec![replacement_rule]);
}

#[tokio::test]
async fn stop_db_storage_remove_deletes_rule_by_code() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("monitor.db");
    let store = SqliteStopRuleStore::new(&db_path).await.unwrap();
    store.upsert_rule(sample_rule("000001")).await.unwrap();

    let removed = store.remove_rule("000001").await.unwrap();

    assert!(removed);
    assert!(store.list_rules().await.unwrap().is_empty());
}

#[tokio::test]
async fn stop_db_storage_persists_trailing_highest_price_across_reopen() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("monitor.db");

    {
        let store = SqliteStopRuleStore::new(&db_path).await.unwrap();
        let mut rule = sample_rule("000001");
        rule.stop_loss_price = None;
        rule.trailing_pct = Some(5.0);
        rule.highest_price = Some(20.0);
        store.upsert_rule(rule).await.unwrap();
    }

    let reopened = SqliteStopRuleStore::new(&db_path).await.unwrap();
    let rules = reopened.list_rules().await.unwrap();

    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].trailing_pct, Some(5.0));
    assert_eq!(rules[0].highest_price, Some(20.0));
}

#[tokio::test]
async fn stop_db_storage_persists_last_triggered_state_updates() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("monitor.db");
    let store = SqliteStopRuleStore::new(&db_path).await.unwrap();
    let mut rule = sample_rule("000001");
    store.upsert_rule(rule.clone()).await.unwrap();

    let triggered_at = Utc.with_ymd_and_hms(2026, 3, 11, 12, 5, 0).unwrap();
    rule.highest_price = Some(19.5);
    rule.last_triggered_at = Some(triggered_at);
    rule.updated_at = triggered_at;
    store.upsert_rule(rule).await.unwrap();

    let reopened = SqliteStopRuleStore::new(&db_path).await.unwrap();
    let rules = reopened.list_rules().await.unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].highest_price, Some(19.5));
    assert_eq!(rules[0].last_triggered_at, Some(triggered_at));
    assert_eq!(rules[0].updated_at, triggered_at);
}

#[tokio::test]
async fn stop_db_storage_migrates_legacy_rows_with_new_nullable_columns() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("monitor.db");

    let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))
        .unwrap()
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();

    sqlx::query(
        r#"
CREATE TABLE stop_rules (
    code TEXT PRIMARY KEY,
    stop_loss_price REAL,
    take_profit_price REAL,
    trailing_pct REAL,
    highest_price REAL,
    last_triggered_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
)
"#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
INSERT INTO stop_rules (
    code,
    stop_loss_price,
    take_profit_price,
    trailing_pct,
    highest_price,
    last_triggered_at,
    created_at,
    updated_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
"#,
    )
    .bind("000001")
    .bind(14.5_f64)
    .bind(Option::<f64>::None)
    .bind(Option::<f64>::None)
    .bind(Option::<f64>::None)
    .bind(Option::<String>::None)
    .bind(sample_time().to_rfc3339())
    .bind(sample_time().to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    drop(pool);

    let store = SqliteStopRuleStore::new(&db_path).await.unwrap();
    let saved = store.get_rule("000001").await.unwrap().unwrap();

    assert_eq!(saved.stop_loss_price, Some(14.5));
    assert_eq!(saved.stop_loss_pct, None);
    assert_eq!(saved.take_profit_pct, None);
    assert_eq!(saved.reference_price, None);
}

#[tokio::test]
async fn stop_db_storage_history_round_trips_change_and_trigger_events() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("monitor.db");
    let store = SqliteStopRuleStore::new(&db_path).await.unwrap();

    store
        .append_history(SqliteStopHistoryEvent {
            id: "hist-set-1".to_string(),
            code: "000001".to_string(),
            event_type: StopHistoryEventType::Set,
            trigger_kind: None,
            trigger_price: None,
            anchor_price: Some(15.2),
            anchor_source: Some("reference_price".to_string()),
            snapshot_json: serde_json::json!({
                "code": "000001",
                "stop_loss_pct": 5.0
            }),
            created_at: sample_time(),
        })
        .await
        .unwrap();

    store
        .append_history(SqliteStopHistoryEvent {
            id: "hist-trigger-1".to_string(),
            code: "000001".to_string(),
            event_type: StopHistoryEventType::Trigger,
            trigger_kind: Some(SqliteStopHistoryTriggerKind::Loss),
            trigger_price: Some(14.1),
            anchor_price: Some(15.0),
            anchor_source: Some("position_cost".to_string()),
            snapshot_json: serde_json::json!({
                "code": "000001",
                "stop_loss_pct": 5.0
            }),
            created_at: sample_time(),
        })
        .await
        .unwrap();

    let history = store
        .list_history(SqliteStopHistoryFilter {
            code: Some("000001".to_string()),
            date: None,
            event_type: None,
            limit: Some(10),
        })
        .await
        .unwrap();

    assert_eq!(history.len(), 2);
    assert_eq!(history[0].event_type, StopHistoryEventType::Trigger);
    assert_eq!(
        history[0].trigger_kind,
        Some(SqliteStopHistoryTriggerKind::Loss)
    );
    assert_eq!(history[1].event_type, StopHistoryEventType::Set);
}
