//! P0.14 T3 — L1/L2: live ClickHouse + OpenStock round-trip。
//!
//! `#[ignore]`-gated integration tests that exercise the full pipeline:
//! OpenStock HTTP → P0.13d stream API → minute.rs row converters →
//! ClickHouse insert → reverse query verification.
//!
//! Skipped by default (`cargo test --workspace` runs neither). Manual run:
//!   QUANTIX_CLICKHOUSE_LIVE=1 \
//!   OPENSTOCK_BASE_URL=... OPENSTOCK_API_KEY=... \
//!   CLICKHOUSE_URL=... CLICKHOUSE_USER=... CLICKHOUSE_PASSWORD=... \
//!   cargo test --lib -p quantix-cli -- --ignored live_stream_minute
//!
//! Sinks are `pub(crate)` (INV-4), so the tests MUST live inside this file
//! (same module tree) rather than in `tests/` (separate crate, cannot see
//! `pub(crate)` items).

use crate::db::clickhouse::models::{MinuteKlineCH, MinuteShareCH};

#[tokio::test]
#[ignore = "live ClickHouse + OpenStock; set QUANTIX_CLICKHOUSE_LIVE=1 to run"]
async fn live_stream_minute_klines_to_clickhouse_round_trip() {
    if std::env::var("QUANTIX_CLICKHOUSE_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    use crate::core::runtime::ClickHouseSettings;
    use crate::data::models::{AdjustType, MinutePeriod};
    use crate::db::clickhouse::minute::ClickHouseMinuteKlineSink;
    use crate::db::clickhouse::stream_minute_klines_to_clickhouse;
    use crate::sources::openstock_client::{OpenStockClient, OpenStockClientConfig};
    use chrono::NaiveDate;

    // `OpenStockClientConfig` has 6 fields; use `::default()` then override
    // the two fields we actually need to come from env. `Default` provides
    // sensible retry / circuit-breaker values.
    let os_client = OpenStockClient::new(OpenStockClientConfig {
        base_url: std::env::var("OPENSTOCK_BASE_URL").expect("OPENSTOCK_BASE_URL"),
        api_key: std::env::var("OPENSTOCK_API_KEY").expect("OPENSTOCK_API_KEY"),
        ..OpenStockClientConfig::default()
    })
    .expect("os client");

    let ch_settings = ClickHouseSettings::from_env();
    let ch = crate::db::clickhouse::ClickHouseClient::from_settings(&ch_settings)
        .await
        .expect("ch client");
    ch.init_database()
        .await
        .expect("init_database (creates minute_klines)");

    let start = NaiveDate::from_ymd_opt(2026, 6, 23).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 24).unwrap();
    let sink = ClickHouseMinuteKlineSink {
        client: ch.client(),
    };
    let stats = stream_minute_klines_to_clickhouse(
        &os_client,
        &sink,
        "sh600000",
        MinutePeriod::Minute1,
        start,
        end,
        AdjustType::None,
    )
    .await
    .expect("stream ok");

    assert!(
        stats.batches >= 1,
        "expected at least 1 batch, got {}",
        stats.batches
    );
    assert!(stats.inserted_records > 0, "expected inserted_records > 0");

    // Reverse-check: query the table back.
    let rows: Vec<MinuteKlineCH> = ch
        .client()
        .query(
            "SELECT timestamp, code, period, adjust, open, high, low, close, volume, amount \
             FROM minute_klines WHERE code = ? AND timestamp >= ? AND timestamp <= ? \
             ORDER BY timestamp",
        )
        .bind("sh600000")
        .bind(start.and_hms_opt(0, 0, 0).unwrap())
        .bind(end.and_hms_opt(23, 59, 59).unwrap())
        .fetch_all()
        .await
        .expect("reverse query ok");

    assert!(!rows.is_empty());
    assert_eq!(rows.len() as u64, stats.inserted_records);
}

#[tokio::test]
#[ignore = "live ClickHouse + OpenStock; set QUANTIX_CLICKHOUSE_LIVE=1 to run"]
async fn live_stream_minute_shares_to_clickhouse_round_trip() {
    if std::env::var("QUANTIX_CLICKHOUSE_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    use crate::core::runtime::ClickHouseSettings;
    use crate::db::clickhouse::minute::ClickHouseMinuteShareSink;
    use crate::db::clickhouse::stream_minute_shares_to_clickhouse;
    use crate::sources::openstock_client::{OpenStockClient, OpenStockClientConfig};
    use chrono::NaiveDate;

    let os_client = OpenStockClient::new(OpenStockClientConfig {
        base_url: std::env::var("OPENSTOCK_BASE_URL").expect("OPENSTOCK_BASE_URL"),
        api_key: std::env::var("OPENSTOCK_API_KEY").expect("OPENSTOCK_API_KEY"),
        ..OpenStockClientConfig::default()
    })
    .expect("os client");

    let ch_settings = ClickHouseSettings::from_env();
    let ch = crate::db::clickhouse::ClickHouseClient::from_settings(&ch_settings)
        .await
        .expect("ch client");
    ch.init_database()
        .await
        .expect("init_database (creates minute_shares)");

    let start = NaiveDate::from_ymd_opt(2026, 6, 23).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 24).unwrap();
    let sink = ClickHouseMinuteShareSink {
        client: ch.client(),
    };
    let stats = stream_minute_shares_to_clickhouse(&os_client, &sink, "sh600000", start, end)
        .await
        .expect("stream ok");

    assert!(stats.batches >= 1, "expected at least 1 batch");
    assert!(stats.inserted_records > 0, "expected inserted_records > 0");

    let rows: Vec<MinuteShareCH> = ch
        .client()
        .query(
            "SELECT timestamp, code, price, volume, amount, avg_price FROM minute_shares \
             WHERE code = ? AND timestamp >= ? AND timestamp <= ? ORDER BY timestamp",
        )
        .bind("sh600000")
        .bind(start.and_hms_opt(0, 0, 0).unwrap())
        .bind(end.and_hms_opt(23, 59, 59).unwrap())
        .fetch_all()
        .await
        .expect("reverse query ok");

    assert!(!rows.is_empty());
    assert_eq!(rows.len() as u64, stats.inserted_records);
}
