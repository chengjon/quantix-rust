//! Live HTTP smoke tests for `OpenStockClient::fetch_klines` (P0.13a).
//! Gated by `QUANTIX_OPENSTOCK_LIVE=1`.

#![cfg(test)]

use quantix_cli::data::models::{AdjustType, BarPeriod};
use quantix_cli::sources::openstock_client::OpenStockClient;
use std::str::FromStr;

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_klines_live_day_none() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping: QUANTIX_OPENSTOCK_LIVE not set");
        return;
    }
    let symbol = std::env::var("OPENSTOCK_LIVE_SYMBOL").unwrap_or_else(|_| "600000".to_string());
    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let klines = client
        .fetch_klines(&symbol, BarPeriod::Day, AdjustType::None, None, None)
        .await
        .expect("fetch ok");
    assert!(!klines.is_empty(), "day klines should return records");
    assert_eq!(klines[0].adjust_type, AdjustType::None);
    println!(
        "fetch_klines day+none ({}): {} records, first date={} close={}",
        symbol,
        klines.len(),
        klines[0].date,
        klines[0].close
    );
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_klines_live_week_qfq() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping: QUANTIX_OPENSTOCK_LIVE not set");
        return;
    }
    let symbol = std::env::var("OPENSTOCK_LIVE_SYMBOL").unwrap_or_else(|_| "600000".to_string());
    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let klines = client
        .fetch_klines(&symbol, BarPeriod::Week, AdjustType::QFQ, None, None)
        .await
        .expect("fetch ok");
    assert!(!klines.is_empty(), "week klines should return records");
    assert_eq!(klines[0].adjust_type, AdjustType::QFQ);
    println!(
        "fetch_klines week+qfq ({}): {} records, first date={} close={}",
        symbol,
        klines.len(),
        klines[0].date,
        klines[0].close
    );
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_klines_live_month_hfq() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping: QUANTIX_OPENSTOCK_LIVE not set");
        return;
    }
    let symbol = std::env::var("OPENSTOCK_LIVE_SYMBOL").unwrap_or_else(|_| "600000".to_string());
    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let klines = client
        .fetch_klines(&symbol, BarPeriod::Month, AdjustType::HFQ, None, None)
        .await
        .expect("fetch ok");
    assert!(!klines.is_empty(), "month klines should return records");
    assert_eq!(klines[0].adjust_type, AdjustType::HFQ);
    println!(
        "fetch_klines month+hfq ({}): {} records, first date={} close={}",
        symbol,
        klines.len(),
        klines[0].date,
        klines[0].close
    );
    // Sanity: FromStr agrees with what we just used.
    assert!(matches!(BarPeriod::from_str("month"), Ok(BarPeriod::Month)));
}
