use chrono::{TimeZone, Utc};
use quantix_cli::trade::{
    FeeConfig, JsonPaperTradeStore, PaperTradeAccount, PaperTradeState, PaperTradeStore,
    TradePosition, TradeRecord, TradeSide,
};
use rust_decimal_macros::dec;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 14, 0, 0).unwrap()
}

fn initialized_state() -> PaperTradeState {
    let ts = fixed_ts();
    let mut positions = BTreeMap::new();
    positions.insert(
        "600000".to_string(),
        TradePosition {
            code: "600000".to_string(),
            volume: 100,
            avg_cost: dec!(10.05),
            last_trade_price: dec!(12),
            opened_at: ts,
            updated_at: ts,
        },
    );

    PaperTradeState {
        version: 1,
        account: Some(PaperTradeAccount {
            account_id: "default".to_string(),
            initial_capital: dec!(1000000),
            available_cash: dec!(998995),
            fee_config: FeeConfig::default(),
            positions,
            created_at: ts,
            updated_at: ts,
        }),
        trade_records: vec![TradeRecord {
            id: "trade-1".to_string(),
            code: "600000".to_string(),
            side: TradeSide::Buy,
            price: dec!(10),
            volume: 100,
            amount: dec!(1000),
            commission: dec!(5),
            stamp_duty: dec!(0),
            transfer_fee: dec!(0.1),
            total_fee: dec!(5.1),
            executed_at: ts,
        }],
    }
}

#[tokio::test]
async fn load_state_returns_none_when_the_file_does_not_exist() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("paper_trade.json");
    let store = JsonPaperTradeStore::new(path.clone());

    let state = store.load_state().await.unwrap();

    assert_eq!(state, None);
    assert!(!path.exists());
}

#[tokio::test]
async fn save_and_load_round_trip_preserves_initialized_state() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("trade").join("paper_trade.json");
    let store = JsonPaperTradeStore::new(path.clone());
    let state = initialized_state();

    store.save_state(&state).await.unwrap();
    let loaded = store.load_state().await.unwrap().unwrap();

    assert!(path.exists());
    assert_eq!(loaded, state);
}

#[tokio::test]
async fn persisted_trades_survive_reopen() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("paper_trade.json");
    let store = JsonPaperTradeStore::new(path.clone());
    store.save_state(&initialized_state()).await.unwrap();

    let reopened = JsonPaperTradeStore::new(path);
    let state = reopened.load_state().await.unwrap().unwrap();

    assert_eq!(state.trade_records.len(), 1);
    assert_eq!(state.trade_records[0].code, "600000");
    assert!(state.account.unwrap().positions.contains_key("600000"));
}

#[tokio::test]
async fn reset_state_persistence_overwrites_previous_content_cleanly() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("paper_trade.json");
    let store = JsonPaperTradeStore::new(path);
    store.save_state(&initialized_state()).await.unwrap();

    let ts = fixed_ts();
    let reset_state = PaperTradeState {
        version: 1,
        account: Some(PaperTradeAccount {
            account_id: "default".to_string(),
            initial_capital: dec!(500000),
            available_cash: dec!(500000),
            fee_config: FeeConfig::default(),
            positions: BTreeMap::new(),
            created_at: ts,
            updated_at: ts,
        }),
        trade_records: Vec::new(),
    };

    store.save_state(&reset_state).await.unwrap();
    let loaded = store.load_state().await.unwrap().unwrap();

    assert_eq!(loaded.trade_records, Vec::<TradeRecord>::new());
    assert!(loaded.account.unwrap().positions.is_empty());
}

#[cfg(unix)]
#[tokio::test]
async fn save_state_replaces_file_via_atomic_rename() {
    use std::os::unix::fs::MetadataExt;

    let dir = tempdir().unwrap();
    let path = dir.path().join("paper_trade.json");
    let store = JsonPaperTradeStore::new(path.clone());
    store.save_state(&initialized_state()).await.unwrap();
    let before_inode = fs::metadata(&path).unwrap().ino();

    let reset_state = PaperTradeState {
        version: 1,
        account: None,
        trade_records: Vec::new(),
    };
    store.save_state(&reset_state).await.unwrap();

    let after_inode = fs::metadata(&path).unwrap().ino();
    let loaded = store.load_state().await.unwrap().unwrap();

    assert_ne!(before_inode, after_inode, "expected atomic replace via temp-file rename");
    assert_eq!(loaded, reset_state);
}
