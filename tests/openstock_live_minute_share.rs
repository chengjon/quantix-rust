//! Live integration tests for OpenStock MINUTE_DATA category (P0.13b-2).
//!
//! Skipped by default. Run with:
//!   QUANTIX_OPENSTOCK_LIVE=1 \
//!   OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
//!   OPENSTOCK_API_KEY=<key> \
//!   cargo test --test openstock_live_minute_share -- --ignored

#![cfg(test)]

use chrono::Timelike;
use quantix_cli::core::runtime::OpenStockSettings;
use quantix_cli::sources::openstock_client::OpenStockClient;

fn settings_from_env() -> Option<OpenStockSettings> {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        return None;
    }
    let base_url = std::env::var("OPENSTOCK_BASE_URL").ok()?;
    let api_key = std::env::var("OPENSTOCK_API_KEY").ok()?;
    Some(OpenStockSettings {
        base_url: Some(base_url),
        api_key: Some(api_key),
        timeout_secs: 30,
    })
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_minute_share_live_sh600000_recent_trading_day() {
    let Some(settings) = settings_from_env() else {
        return;
    };
    let client = OpenStockClient::from_settings(&settings).expect("client build");
    // Use a recent past date; adjust if market was closed (weekend/holiday)
    let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
    let shares = client
        .fetch_minute_share(
            "sh600000",
            quantix_cli::data::models::DateOrRange::Date(date),
        )
        .await
        .expect("fetch ok");
    assert!(!shares.is_empty(), "expected non-empty time-share ticks");
    // Trading hours for SH: 09:30-11:30, 13:00-15:00 -> first tick around 09:30
    let first = &shares[0];
    assert!(
        first.timestamp.hour() >= 9,
        "first tick hour too early: {:?}",
        first
    );
    // Sanity: avg_price should be positive
    assert!(
        first
            .avg_price
            .map(|p| p > rust_decimal::Decimal::ZERO)
            .unwrap_or(false),
        "expected positive avg_price, got: {:?}",
        first
    );
    println!(
        "L1 sh600000 {} -> {} ticks, first={:?}",
        date,
        shares.len(),
        first
    );
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_minute_share_live_weekend_returns_empty_or_error() {
    let Some(settings) = settings_from_env() else {
        return;
    };
    let client = OpenStockClient::from_settings(&settings).expect("client build");
    // 2026-06-27 is a Saturday -> market closed
    let saturday = chrono::NaiveDate::from_ymd_opt(2026, 6, 27).unwrap();
    let result = client
        .fetch_minute_share(
            "sh600000",
            quantix_cli::data::models::DateOrRange::Date(saturday),
        )
        .await;
    // Either empty Vec (graceful) or Err (envelope error) -- both acceptable
    match result {
        Ok(shares) => {
            assert!(
                shares.is_empty(),
                "expected empty ticks on weekend, got {} records: {:?}",
                shares.len(),
                &shares[..shares.len().min(3)]
            );
            println!("L2 weekend -> empty Vec (graceful)");
        }
        Err(e) => {
            println!("L2 weekend -> envelope error (acceptable): {e}");
        }
    }
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_minute_share_live_unknown_code_propagates_error() {
    let Some(settings) = settings_from_env() else {
        return;
    };
    let client = OpenStockClient::from_settings(&settings).expect("client build");
    let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
    let result = client
        .fetch_minute_share(
            "invalid_code_xyz",
            quantix_cli::data::models::DateOrRange::Date(date),
        )
        .await;
    assert!(
        result.is_err(),
        "expected error for unknown code, got: {:?}",
        result
    );
    println!("L3 unknown code -> error: {:?}", result.unwrap_err());
}
