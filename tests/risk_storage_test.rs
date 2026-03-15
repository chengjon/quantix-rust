use chrono::{TimeZone, Utc};
use quantix_cli::risk::{
    BuyLockState, DailyRiskBaseline, JsonRiskStore, RiskLogEvent, RiskLogEventType, RiskRule,
    RiskRuleType, RiskState, RiskStore, RuleValue,
};
use rust_decimal_macros::dec;
use std::fs;
use tempfile::tempdir;

fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 14, 0, 0).unwrap()
}

fn configured_state() -> RiskState {
    let ts = fixed_ts();
    RiskState {
        version: 1,
        account_id: "default".to_string(),
        rules: vec![
            RiskRule {
                rule_type: RiskRuleType::PositionLimit,
                value: RuleValue::Percentage(dec!(20)),
                enabled: true,
                created_at: ts,
                updated_at: ts,
            },
            RiskRule {
                rule_type: RiskRuleType::DailyLossLimit,
                value: RuleValue::Amount(dec!(50000)),
                enabled: false,
                created_at: ts,
                updated_at: ts,
            },
        ],
        daily_baseline: Some(DailyRiskBaseline {
            trading_date: ts.date_naive(),
            starting_total_assets: dec!(1000000),
        }),
        buy_lock: BuyLockState {
            locked: true,
            reason: Some("daily-loss-limit 50000 已触发".to_string()),
            triggered_at: Some(ts),
            trading_date: Some(ts.date_naive()),
            released_for_date: None,
        },
        events: vec![RiskLogEvent {
            ts,
            event_type: RiskLogEventType::DailyLossLockTriggered,
            trading_date: Some(ts.date_naive()),
            detail: "daily-loss-limit 50000 已触发".to_string(),
        }],
    }
}

#[tokio::test]
async fn load_state_returns_none_when_the_file_does_not_exist() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("risk_state.json");
    let store = JsonRiskStore::new(path.clone());

    let state = store.load_state().await.unwrap();

    assert_eq!(state, None);
    assert!(!path.exists());
}

#[tokio::test]
async fn save_and_load_round_trip_preserves_rules_and_lock_state() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("risk").join("risk_state.json");
    let store = JsonRiskStore::new(path.clone());
    let state = configured_state();

    store.save_state(&state).await.unwrap();
    let loaded = store.load_state().await.unwrap().unwrap();

    assert!(path.exists());
    assert_eq!(loaded, state);
}

#[tokio::test]
async fn persisted_state_survives_reopen() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("risk_state.json");
    let store = JsonRiskStore::new(path.clone());
    store.save_state(&configured_state()).await.unwrap();

    let reopened = JsonRiskStore::new(path);
    let state = reopened.load_state().await.unwrap().unwrap();

    assert_eq!(state.rules.len(), 2);
    assert!(state.buy_lock.locked);
    assert_eq!(state.daily_baseline.unwrap().starting_total_assets, dec!(1000000));
}

#[cfg(unix)]
#[tokio::test]
async fn save_state_replaces_file_via_atomic_rename() {
    use std::os::unix::fs::MetadataExt;

    let dir = tempdir().unwrap();
    let path = dir.path().join("risk_state.json");
    let store = JsonRiskStore::new(path.clone());
    store.save_state(&configured_state()).await.unwrap();
    let before_inode = fs::metadata(&path).unwrap().ino();

    let reset_state = RiskState::default();
    store.save_state(&reset_state).await.unwrap();

    let after_inode = fs::metadata(&path).unwrap().ino();
    let loaded = store.load_state().await.unwrap().unwrap();

    assert_ne!(before_inode, after_inode, "expected atomic replace via temp-file rename");
    assert_eq!(loaded, reset_state);
}
