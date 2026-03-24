use chrono::{Duration, TimeZone, Utc};
use quantix_cli::risk::{
    LiveImportCashBusinessType, LiveImportRecord, LiveImportRecordType, LiveImportTradeSide,
    SqliteLiveImportStore, SqliteLiveMirrorRebuildEngine,
};
use rust_decimal_macros::dec;
use tempfile::tempdir;

fn ts(hour: u32, minute: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 24, hour, minute, 0).unwrap()
}

fn buy(external_id: &str, price: rust_decimal::Decimal, volume: i64) -> LiveImportRecord {
    LiveImportRecord {
        record_type: LiveImportRecordType::Trade,
        account_id: "live-001".to_string(),
        external_id: external_id.to_string(),
        code: Some("000001".to_string()),
        side: Some(LiveImportTradeSide::Buy),
        price: Some(price),
        volume: Some(volume),
        fee_total: Some(dec!(5.00)),
        business_type: None,
        amount: None,
        executed_at: Some(ts(9, 35)),
        occurred_at: None,
    }
}

fn sell(external_id: &str, price: rust_decimal::Decimal, volume: i64) -> LiveImportRecord {
    LiveImportRecord {
        record_type: LiveImportRecordType::Trade,
        account_id: "live-001".to_string(),
        external_id: external_id.to_string(),
        code: Some("000001".to_string()),
        side: Some(LiveImportTradeSide::Sell),
        price: Some(price),
        volume: Some(volume),
        fee_total: Some(dec!(3.00)),
        business_type: None,
        amount: None,
        executed_at: Some(ts(10, 0)),
        occurred_at: None,
    }
}

fn cash(
    external_id: &str,
    business_type: LiveImportCashBusinessType,
    amount: rust_decimal::Decimal,
    occurred_at: chrono::DateTime<Utc>,
) -> LiveImportRecord {
    LiveImportRecord {
        record_type: LiveImportRecordType::Cash,
        account_id: "live-001".to_string(),
        external_id: external_id.to_string(),
        code: None,
        side: None,
        price: None,
        volume: None,
        fee_total: None,
        business_type: Some(business_type),
        amount: Some(amount),
        executed_at: None,
        occurred_at: Some(occurred_at),
    }
}

#[tokio::test]
async fn rebuild_buy_only_creates_cash_and_position() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("live_import.db");
    let store = SqliteLiveImportStore::new(&path).await.unwrap();
    store
        .import_records(
            "live-001",
            "buy.csv",
            &[
                cash("cash-1", LiveImportCashBusinessType::Deposit, dec!(100000), ts(9, 0)),
                buy("fill-1", dec!(15.20), 100),
            ],
            ts(11, 0),
        )
        .await
        .unwrap();

    let engine = SqliteLiveMirrorRebuildEngine::new(store.clone());
    let mirror = engine.rebuild_account("live-001", ts(12, 0)).await.unwrap();

    assert_eq!(mirror.account_id, "live-001");
    assert_eq!(mirror.cash_balance, dec!(98475.00));
    assert_eq!(mirror.positions.len(), 1);
    assert_eq!(mirror.positions[0].code, "000001");
    assert_eq!(mirror.positions[0].volume, 100);
    assert_eq!(mirror.positions[0].avg_cost, dec!(15.25));
    assert_eq!(mirror.realized_pnl, dec!(0));
}

#[tokio::test]
async fn rebuild_buy_sell_and_cash_flows_updates_realized_pnl_and_cash() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("live_import.db");
    let store = SqliteLiveImportStore::new(&path).await.unwrap();
    store
        .import_records(
            "live-001",
            "ledger.csv",
            &[
                cash("cash-1", LiveImportCashBusinessType::Deposit, dec!(100000), ts(9, 0)),
                buy("fill-1", dec!(15.20), 100),
                sell("fill-2", dec!(16.30), 40),
                cash(
                    "cash-2",
                    LiveImportCashBusinessType::Withdraw,
                    dec!(-5000),
                    ts(10, 5),
                ),
            ],
            ts(11, 0),
        )
        .await
        .unwrap();

    let engine = SqliteLiveMirrorRebuildEngine::new(store.clone());
    let mirror = engine.rebuild_account("live-001", ts(12, 0)).await.unwrap();

    assert_eq!(mirror.cash_balance, dec!(94124.00));
    assert_eq!(mirror.positions[0].volume, 60);
    assert_eq!(mirror.realized_pnl, dec!(39.00));
    assert_eq!(mirror.total_fees, dec!(8.00));
}

#[tokio::test]
async fn oversell_fails_rebuild_and_preserves_last_successful_snapshot() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("live_import.db");
    let store = SqliteLiveImportStore::new(&path).await.unwrap();
    store
        .import_records(
            "live-001",
            "good.csv",
            &[
                cash("cash-1", LiveImportCashBusinessType::Deposit, dec!(100000), ts(9, 0)),
                buy("fill-1", dec!(15.20), 100),
            ],
            ts(11, 0),
        )
        .await
        .unwrap();

    let engine = SqliteLiveMirrorRebuildEngine::new(store.clone());
    let first = engine.rebuild_account("live-001", ts(12, 0)).await.unwrap();
    assert_eq!(first.positions[0].volume, 100);

    store
        .import_records(
            "live-001",
            "bad.csv",
            &[sell("fill-3", dec!(16.50), 500)],
            ts(13, 0),
        )
        .await
        .unwrap();

    let err = engine
        .rebuild_account("live-001", ts(13, 5))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("卖出数量超过当前持仓"));

    let persisted = store.get_latest_mirror_account("live-001").await.unwrap().unwrap();
    assert_eq!(persisted.positions[0].volume, 100);
}

#[tokio::test]
async fn rebuild_is_deterministic_for_same_import_set() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("live_import.db");
    let store = SqliteLiveImportStore::new(&path).await.unwrap();
    store
        .import_records(
            "live-001",
            "repeat.csv",
            &[
                cash("cash-1", LiveImportCashBusinessType::Deposit, dec!(100000), ts(9, 0)),
                buy("fill-1", dec!(15.20), 100),
                sell("fill-2", dec!(16.30), 40),
            ],
            ts(11, 0),
        )
        .await
        .unwrap();

    let engine = SqliteLiveMirrorRebuildEngine::new(store);
    let first = engine.rebuild_account("live-001", ts(12, 0)).await.unwrap();
    let second = engine
        .rebuild_account("live-001", ts(12, 0) + Duration::minutes(1))
        .await
        .unwrap();

    assert_eq!(first.cash_balance, second.cash_balance);
    assert_eq!(first.realized_pnl, second.realized_pnl);
    assert_eq!(first.positions, second.positions);
}
