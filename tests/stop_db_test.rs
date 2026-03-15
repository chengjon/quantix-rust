use chrono::{DateTime, TimeZone, Utc};
use quantix_cli::stop::{SqliteStopRuleStore, StopRule, StopRuleStore};
use tempfile::tempdir;

fn sample_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 12, 0, 0).unwrap()
}

fn sample_rule(code: &str) -> StopRule {
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
        trailing_pct: Some(5.0),
        highest_price: Some(19.8),
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
