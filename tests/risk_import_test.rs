use chrono::{TimeZone, Utc};
use quantix_cli::risk::{
    parse_live_import_csv, parse_live_import_json, LiveImportCashBusinessType, LiveImportRecord,
    LiveImportRecordType, LiveImportTradeSide, SqliteLiveImportStore,
};
use rust_decimal_macros::dec;
use tempfile::tempdir;

fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 24, 9, 35, 0).unwrap()
}

#[test]
fn parses_normalized_csv_trade_and_cash_records() {
    let csv = r#"record_type,account_id,external_id,code,side,price,volume,fee_total,business_type,amount,executed_at,occurred_at
trade,live-001,fill-1,000001,buy,15.20,100,5.00,,,2026-03-24T09:35:00Z,
cash,live-001,cash-1,,,,,,deposit,100000.00,,2026-03-24T09:00:00Z
"#;

    let records = parse_live_import_csv(csv).unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].record_type, LiveImportRecordType::Trade);
    assert_eq!(records[0].account_id, "live-001");
    assert_eq!(records[0].external_id, "fill-1");
    assert_eq!(records[0].code.as_deref(), Some("000001"));
    assert_eq!(records[0].side, Some(LiveImportTradeSide::Buy));
    assert_eq!(records[0].price, Some(dec!(15.20)));
    assert_eq!(records[0].volume, Some(100));
    assert_eq!(records[0].fee_total, Some(dec!(5.00)));
    assert_eq!(records[0].executed_at, Some(fixed_ts()));

    assert_eq!(records[1].record_type, LiveImportRecordType::Cash);
    assert_eq!(
        records[1].business_type,
        Some(LiveImportCashBusinessType::Deposit)
    );
    assert_eq!(records[1].amount, Some(dec!(100000.00)));
}

#[test]
fn parses_normalized_json_trade_and_cash_records() {
    let json = r#"
[
  {
    "record_type": "trade",
    "account_id": "live-001",
    "external_id": "fill-1",
    "code": "000001",
    "side": "sell",
    "price": "16.30",
    "volume": 200,
    "fee_total": "8.20",
    "executed_at": "2026-03-24T10:00:00Z"
  },
  {
    "record_type": "cash",
    "account_id": "live-001",
    "external_id": "cash-2",
    "business_type": "withdraw",
    "amount": "-5000.00",
    "occurred_at": "2026-03-24T10:05:00Z"
  }
]
"#;

    let records = parse_live_import_json(json).unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].record_type, LiveImportRecordType::Trade);
    assert_eq!(records[0].side, Some(LiveImportTradeSide::Sell));
    assert_eq!(records[0].price, Some(dec!(16.30)));
    assert_eq!(records[1].record_type, LiveImportRecordType::Cash);
    assert_eq!(
        records[1].business_type,
        Some(LiveImportCashBusinessType::Withdraw)
    );
    assert_eq!(records[1].amount, Some(dec!(-5000.00)));
}

#[tokio::test]
async fn duplicate_import_skips_identical_rows_and_counts_them() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("live_import.db");
    let store = SqliteLiveImportStore::new(&path).await.unwrap();
    let record = LiveImportRecord {
        record_type: LiveImportRecordType::Trade,
        account_id: "live-001".to_string(),
        external_id: "fill-1".to_string(),
        code: Some("000001".to_string()),
        side: Some(LiveImportTradeSide::Buy),
        price: Some(dec!(15.20)),
        volume: Some(100),
        fee_total: Some(dec!(5.00)),
        business_type: None,
        amount: None,
        executed_at: Some(fixed_ts()),
        occurred_at: None,
    };

    let first = store
        .import_records("live-001", "batch-a.csv", &[record.clone()], fixed_ts())
        .await
        .unwrap();
    let second = store
        .import_records("live-001", "batch-a.csv", &[record], fixed_ts())
        .await
        .unwrap();

    assert_eq!(first.inserted, 1);
    assert_eq!(second.inserted, 0);
    assert_eq!(second.skipped_duplicates, 1);
    assert_eq!(second.conflicts, 0);
}

#[tokio::test]
async fn conflicting_duplicate_is_recorded_and_reported() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("live_import.db");
    let store = SqliteLiveImportStore::new(&path).await.unwrap();
    let original = LiveImportRecord {
        record_type: LiveImportRecordType::Trade,
        account_id: "live-001".to_string(),
        external_id: "fill-1".to_string(),
        code: Some("000001".to_string()),
        side: Some(LiveImportTradeSide::Buy),
        price: Some(dec!(15.20)),
        volume: Some(100),
        fee_total: Some(dec!(5.00)),
        business_type: None,
        amount: None,
        executed_at: Some(fixed_ts()),
        occurred_at: None,
    };
    let conflicting = LiveImportRecord {
        price: Some(dec!(15.30)),
        ..original.clone()
    };

    store
        .import_records("live-001", "batch-a.csv", &[original], fixed_ts())
        .await
        .unwrap();
    let summary = store
        .import_records("live-001", "batch-b.csv", &[conflicting], fixed_ts())
        .await
        .unwrap();

    assert_eq!(summary.inserted, 0);
    assert_eq!(summary.skipped_duplicates, 0);
    assert_eq!(summary.conflicts, 1);

    let conflicts = store.list_conflicts(&summary.batch_id).await.unwrap();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].account_id, "live-001");
    assert_eq!(conflicts[0].external_id, "fill-1");
}
