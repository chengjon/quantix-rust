//! Tests for data_handler subcommands. Reaches the (pub(crate)) helpers it
//! asserts on via `super::*`.

use super::*;
use crate::data::models::{AdjustType, Kline};
use chrono::NaiveDate;
use rust_decimal_macros::dec;
use std::fs;
use tempfile::tempdir;

#[test]
fn load_market_fundamental_records_parses_json_array() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "quantix-market-fundamentals-{}.json",
        std::process::id()
    ));

    fs::write(
        &path,
        r#"[{"code":"600519","snapshot_date":"2026-03-14","market_cap":23000.5,"latest_report_profit":862.1,"profit_source":"report","pe_dynamic":27.4}]"#,
    )
    .unwrap();

    let records = load_market_fundamental_records(path.to_str().unwrap()).unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].code, "600519");
    assert_eq!(records[0].market_cap, Some(23000.5));
    assert_eq!(records[0].latest_report_profit, Some(862.1));

    let _ = fs::remove_file(path);
}

#[tokio::test]
async fn export_klines_writes_parquet_file() {
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("000001.parquet");
    let klines = vec![Kline {
        code: "000001".to_string(),
        date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
        open: dec!(10.1),
        high: dec!(10.8),
        low: dec!(9.9),
        close: dec!(10.5),
        volume: 120000,
        amount: Some(dec!(1260000.0)),
        adjust_type: AdjustType::None,
    }];

    export_klines_to_file(&klines, "parquet", &output_path)
        .await
        .unwrap();

    assert!(output_path.exists());
    assert!(fs::metadata(output_path).unwrap().len() > 0);
}
