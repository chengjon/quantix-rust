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

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn live_fetch_minute_klines_stream_multi_week_range() {
    // L1: stream API and batch API are equivalent for a multi-week range.
    use futures::StreamExt;

    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let settings = settings_from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let client = OpenStockClient::from_settings(&settings).expect("client from settings");

    // 14-day range → 2 weekly chunks
    use chrono::NaiveDate;
    let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
    let dor = DateOrRange::Range { start, end };

    let period = MinutePeriod::Minute1;
    let adjust = AdjustType::None;

    // Batch call
    let batch_result = client
        .fetch_minute_klines("600000", period, dor.clone(), adjust)
        .await
        .expect("batch fetch ok");

    // Stream call: collect
    let mut stream_result = Vec::new();
    let s = client.fetch_minute_klines_stream("600000", period, dor, adjust);
    futures::pin_mut!(s);
    let mut batch_count = 0;
    while let Some(batch) = s.next().await {
        let batch = batch.expect("stream batch ok");
        batch_count += 1;
        stream_result.extend(batch);
    }
    assert!(batch_count >= 2, "14-day range should produce >= 2 chunks");

    // INV-1A: same length and same first/last timestamp
    assert_eq!(
        batch_result.len(),
        stream_result.len(),
        "batch and stream must return same record count"
    );
    if !batch_result.is_empty() {
        assert_eq!(
            batch_result.first().unwrap().timestamp,
            stream_result.first().unwrap().timestamp,
            "first timestamp must match"
        );
        assert_eq!(
            batch_result.last().unwrap().timestamp,
            stream_result.last().unwrap().timestamp,
            "last timestamp must match"
        );
    }
}
