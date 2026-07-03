//! Live OpenStock /data/bars minute-candle integration tests (P0.13b-1).
//!
//! These tests hit the real OpenStock runtime and are `#[ignore]`-gated by
//! default. To run them locally:
//!
//! ```sh
//! QUANTIX_OPENSTOCK_LIVE=1 \
//! OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
//! OPENSTOCK_API_KEY=<key> \
//! cargo test --test openstock_live_minute_klines -- --ignored
//! ```

use quantix_cli::core::runtime::OpenStockSettings;
use quantix_cli::data::models::{AdjustType, DateOrRange, MinutePeriod};
use quantix_cli::sources::openstock_client::OpenStockClient;

fn settings_from_env() -> Option<OpenStockSettings> {
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
async fn fetch_minute_klines_live_1m_none() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let settings = settings_from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let client = OpenStockClient::from_settings(&settings).expect("client ok");
    let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
    let bars = client
        .fetch_minute_klines(
            "sh600000",
            MinutePeriod::Minute1,
            DateOrRange::Date(date),
            AdjustType::None,
        )
        .await
        .expect("fetch ok");
    assert!(!bars.is_empty(), "expected non-empty 1m bars");
    println!("1m+none bars: {}", bars.len());
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_minute_klines_live_5m_qfq() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let settings = settings_from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let client = OpenStockClient::from_settings(&settings).expect("client ok");
    let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
    let bars = client
        .fetch_minute_klines(
            "sh600000",
            MinutePeriod::Minute5,
            DateOrRange::Date(date),
            AdjustType::QFQ,
        )
        .await
        .expect("fetch ok");
    assert!(!bars.is_empty(), "expected non-empty 5m qfq bars");
    assert_eq!(bars[0].adjust_type, AdjustType::QFQ);
    println!("5m+qfq bars: {}", bars.len());
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_minute_klines_live_60m_hfq() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let settings = settings_from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let client = OpenStockClient::from_settings(&settings).expect("client ok");
    let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
    let bars = client
        .fetch_minute_klines(
            "sh600000",
            MinutePeriod::Minute60,
            DateOrRange::Date(date),
            AdjustType::HFQ,
        )
        .await
        .expect("fetch ok");
    assert!(!bars.is_empty(), "expected non-empty 60m hfq bars");
    assert_eq!(bars[0].adjust_type, AdjustType::HFQ);
    println!("60m+hfq bars: {}", bars.len());
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn live_fetch_minute_klines_range_returns_multi_day_records() {
    // L1: multi-day server-side range via /data/bars start_date/end_date
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let settings = settings_from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let client = OpenStockClient::from_settings(&settings).expect("client ok");
    let start = chrono::NaiveDate::from_ymd_opt(2026, 6, 23).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2026, 6, 27).unwrap();
    let bars = client
        .fetch_minute_klines(
            "sh600000",
            MinutePeriod::Minute1,
            DateOrRange::Range { start, end },
            AdjustType::None,
        )
        .await
        .expect("live fetch ok");
    assert!(!bars.is_empty(), "5-day range should return non-empty bars");
    let first_date = bars.first().unwrap().timestamp.date();
    let last_date = bars.last().unwrap().timestamp.date();
    assert!(
        first_date >= start,
        "first.date {} < start {}",
        first_date,
        start
    );
    assert!(last_date <= end, "last.date {} > end {}", last_date, end);
    assert_ne!(
        first_date, last_date,
        "range must span multiple trading days"
    );
    println!(
        "L1 range {}..{} -> {} bars, first={} last={}",
        start,
        end,
        bars.len(),
        first_date,
        last_date
    );
}
