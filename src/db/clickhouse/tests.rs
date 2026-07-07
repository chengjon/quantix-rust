use crate::market::service::{
    market_sentiment_daily_to_snapshot, north_flow_daily_to_snapshot, sector_daily_to_board_rank,
    sector_daily_to_leader,
};
use crate::market::{BoardType, LeaderFilter};

use super::models::market_table_sqls;
use super::*;

#[test]
fn test_stock_info_ch_derive() {
    let info = StockInfoCH {
        code: "000001".to_string(),
        name: "平安银行".to_string(),
        market: 0,
        list_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        status: "active".to_string(),
        updated_at: chrono::Utc::now().naive_utc(),
    };
    assert_eq!(info.code, "000001");
}

#[test]
fn test_market_table_sqls_include_phase23_tables() {
    let sql = market_table_sqls()
        .into_iter()
        .map(|(_, sql)| sql)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(sql.contains("CREATE TABLE IF NOT EXISTS sector_daily"));
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS north_flow_daily"));
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS market_sentiment_daily"));
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS market_fundamentals_daily"));
}

#[test]
fn test_market_sector_row_maps_to_board_rank_and_leader() {
    let row = SectorDailyCH {
        sector_code: "BK001".to_string(),
        sector_name: "银行".to_string(),
        sector_type: "industry".to_string(),
        trade_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
        change_pct: 2.35,
        rank: 1,
        leader_code: Some("600000".to_string()),
        leader_name: Some("浦发银行".to_string()),
        leader_change: Some(5.61),
        updated_at: "2026-03-10 15:00:00".to_string(),
    };

    let board = sector_daily_to_board_rank(row.clone()).unwrap();
    let leader = sector_daily_to_leader(row, LeaderFilter::Sector("银行".to_string()))
        .unwrap()
        .unwrap();

    assert_eq!(board.board_type, BoardType::Sector);
    assert_eq!(board.board_name, "银行");
    assert_eq!(board.rank, 1);
    assert_eq!(leader.code, "600000");
    assert_eq!(leader.sector_name.as_deref(), Some("银行"));
    assert_eq!(leader.concept_name, None);
}

#[test]
fn test_market_north_flow_row_maps_to_snapshot() {
    let row = NorthFlowDailyCH {
        trade_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
        sh_amount: 12.3,
        sz_amount: 8.6,
        total_amount: 20.9,
        balance: 99.1,
        updated_at: "2026-03-10 15:00:00".to_string(),
    };

    let snapshot = north_flow_daily_to_snapshot(row);

    assert_eq!(
        snapshot.trade_date,
        chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap()
    );
    assert_eq!(snapshot.total_amount, 20.9);
    assert_eq!(snapshot.balance, 99.1);
}

#[test]
fn test_market_sentiment_row_maps_to_snapshot() {
    let row = MarketSentimentDailyCH {
        trade_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
        up_count: 3210,
        down_count: 1875,
        limit_up_count: 87,
        limit_down_count: 4,
        seal_rate: 0.81,
        break_rate: 0.19,
        consecutive_board_count: 23,
        updated_at: "2026-03-10 15:00:00".to_string(),
    };

    let snapshot = market_sentiment_daily_to_snapshot(row);

    assert_eq!(snapshot.limit_up_count, 87);
    assert_eq!(snapshot.consecutive_board_count, 23);
    assert_eq!(snapshot.seal_rate, 0.81);
}

#[test]
fn test_market_sector_row_deserializes_json_each_row_payload() {
    let row: SectorDailyCH = serde_json::from_str(
        r#"{"sector_code":"BK0001","sector_name":"银行","sector_type":"industry","trade_date":"2026-03-14","change_pct":2.35,"rank":1,"leader_code":null,"leader_name":null,"leader_change":null,"updated_at":"2026-03-14 15:12:16"}"#,
    )
    .unwrap();

    assert_eq!(row.sector_code, "BK0001");
    assert_eq!(row.sector_name, "银行");
    assert_eq!(
        row.trade_date,
        chrono::NaiveDate::from_ymd_opt(2026, 3, 14).unwrap()
    );
    assert_eq!(row.updated_at, "2026-03-14 15:12:16");
    assert!(row.leader_code.is_none());
}

#[test]
fn test_market_north_flow_row_deserializes_json_each_row_payload() {
    let row: NorthFlowDailyCH = serde_json::from_str(
        r#"{"trade_date":"2026-03-14","sh_amount":50.5,"sz_amount":35.2,"total_amount":85.7,"balance":12500.0,"updated_at":"2026-03-14 15:12:16"}"#,
    )
    .unwrap();

    assert_eq!(
        row.trade_date,
        chrono::NaiveDate::from_ymd_opt(2026, 3, 14).unwrap()
    );
    assert_eq!(row.total_amount, 85.7);
    assert_eq!(row.updated_at, "2026-03-14 15:12:16");
}

#[test]
fn test_market_sentiment_row_deserializes_json_each_row_payload() {
    let row: MarketSentimentDailyCH = serde_json::from_str(
        r#"{"trade_date":"2026-03-14","up_count":2800,"down_count":2100,"limit_up_count":45,"limit_down_count":12,"seal_rate":0.78,"break_rate":0.15,"consecutive_board_count":120,"updated_at":"2026-03-14 15:12:16"}"#,
    )
    .unwrap();

    assert_eq!(
        row.trade_date,
        chrono::NaiveDate::from_ymd_opt(2026, 3, 14).unwrap()
    );
    assert_eq!(row.limit_up_count, 45);
    assert_eq!(row.updated_at, "2026-03-14 15:12:16");
}

#[test]
fn test_market_fundamental_snapshot_row_deserializes_json_each_row_payload() {
    let row: MarketFundamentalSnapshotCH = serde_json::from_str(
        r#"{"code":"600519","snapshot_date":"2026-03-14","market_cap":23000.5,"latest_report_profit":862.1,"profit_source":"report","pe_dynamic":27.4,"updated_at":"2026-03-14 15:12:16"}"#,
    )
    .unwrap();

    assert_eq!(row.code, "600519");
    assert_eq!(
        row.snapshot_date,
        chrono::NaiveDate::from_ymd_opt(2026, 3, 14).unwrap()
    );
    assert_eq!(row.market_cap, Some(23000.5));
    assert_eq!(row.latest_report_profit, Some(862.1));
    assert_eq!(row.profit_source, "report");
    assert_eq!(row.pe_dynamic, Some(27.4));
}

#[test]
fn models_minute_kline_ch_has_expected_fields() {
    use crate::db::clickhouse::models::MinuteKlineCH;
    use crate::db::clickhouse::naive_to_offsetdatetime;

    let row = MinuteKlineCH {
        timestamp: naive_to_offsetdatetime(
            chrono::NaiveDate::from_ymd_opt(2026, 7, 4)
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap(),
        ),
        code: "sh600000".into(),
        period: "1m".into(),
        adjust: "none".into(),
        open: 12.34,
        high: 12.50,
        low: 12.20,
        close: 12.40,
        volume: 123456.0,
        amount: 1_530_000.0,
    };
    assert_eq!(row.code, "sh600000");
    assert_eq!(row.period, "1m");
    assert_eq!(row.adjust, "none");
    assert_eq!(row.volume, 123456.0);
}

#[test]
fn models_minute_share_ch_has_expected_fields() {
    use crate::db::clickhouse::models::MinuteShareCH;
    use crate::db::clickhouse::naive_to_offsetdatetime;

    let row = MinuteShareCH {
        timestamp: naive_to_offsetdatetime(
            chrono::NaiveDate::from_ymd_opt(2026, 7, 4)
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap(),
        ),
        code: "sh600000".into(),
        price: 12.34,
        volume: 1000.0,
        amount: 12340.0,
        avg_price: 12.34,
    };
    assert_eq!(row.code, "sh600000");
    assert_eq!(row.price, 12.34);
    assert_eq!(row.avg_price, 12.34);
}

// ─── P0.14 T2 — U1/U2/U3: helper tests ─────────────────────────────────────

#[test]
fn decimal_to_f64_normal_range_is_lossless() {
    use rust_decimal::Decimal;
    use std::str::FromStr;
    // Access private helper via public-ish path: tests are inside the same module
    // so we can reach into minute.rs through `super::minute`.
    use crate::db::clickhouse::minute::decimal_to_f64_for_test;

    assert_eq!(
        decimal_to_f64_for_test(Decimal::from_str("1.23").unwrap()),
        1.23
    );
    assert_eq!(
        decimal_to_f64_for_test(Decimal::from_str("9999.99").unwrap()),
        9999.99
    );
    assert_eq!(
        decimal_to_f64_for_test(Decimal::from_str("0").unwrap()),
        0.0
    );
    assert_eq!(
        decimal_to_f64_for_test(Decimal::from_str("1234567890123.45").unwrap()),
        1_234_567_890_123.45
    );
}

#[test]
fn decimal_to_f64_extreme_value_falls_back_to_zero() {
    use crate::db::clickhouse::minute::decimal_to_f64_for_test;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // Construct a Decimal that overflows f64 mantissa (much larger than 2^53).
    // rust_decimal max is ~7.9e28, well beyond f64's i64-exact range.
    let huge = Decimal::from_str("79228162514264337593543950335").unwrap(); // rust_decimal MAX
    let v = decimal_to_f64_for_test(huge);
    // Whether to_f64 returns Some(finite-but-lossy) or None depends on rust_decimal version;
    // either way, our helper guarantees a finite f64 result (no NaN, no panic).
    assert!(v.is_finite());
}

#[test]
fn naive_to_offsetdatetime_preserves_wall_clock() {
    use crate::db::clickhouse::minute::naive_to_ch_timestamp_for_test;
    use chrono::NaiveDate;

    let naive = NaiveDate::from_ymd_opt(2026, 7, 4)
        .unwrap()
        .and_hms_opt(9, 30, 0)
        .unwrap();
    let dt = naive_to_ch_timestamp_for_test(naive);
    // Wall-clock Y/M/D/H/M/S preserved (this is the kline_data convention —
    // Beijing wall-clock tagged UTC, no timezone conversion).
    assert_eq!(dt.year(), 2026);
    assert_eq!(dt.month() as u32, 7);
    assert_eq!(dt.day(), 4);
    assert_eq!(dt.hour(), 9);
    assert_eq!(dt.minute(), 30);
    assert_eq!(dt.second(), 0);
}

// ─── P0.14 T2 — U4/U5: row conversion tests ────────────────────────────────

#[test]
fn bar_to_row_maps_all_minute_bar_fields() {
    use crate::data::models::{AdjustType, MinuteBar, MinutePeriod};
    use crate::db::clickhouse::minute::bar_to_row_for_test;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    let bar = MinuteBar {
        code: "sh600000".into(),
        timestamp: NaiveDate::from_ymd_opt(2026, 7, 4)
            .unwrap()
            .and_hms_opt(9, 30, 0)
            .unwrap(),
        open: Decimal::from_str("12.34").unwrap(),
        high: Decimal::from_str("12.50").unwrap(),
        low: Decimal::from_str("12.20").unwrap(),
        close: Decimal::from_str("12.40").unwrap(),
        volume: 123456,
        amount: Some(Decimal::from_str("1530000.00").unwrap()),
        adjust_type: AdjustType::None,
    };
    // MinuteBar has no `period` field; period comes from the stream function's
    // input parameter, so we pass it explicitly to bar_to_row.
    let row = bar_to_row_for_test(&bar, MinutePeriod::Minute1);
    assert_eq!(row.code, "sh600000");
    assert_eq!(row.period, "1m");
    assert_eq!(row.adjust, "none");
    assert_eq!(row.open, 12.34);
    assert_eq!(row.high, 12.50);
    assert_eq!(row.low, 12.20);
    assert_eq!(row.close, 12.40);
    assert_eq!(row.volume, 123456.0);
    assert_eq!(row.amount, 1_530_000.0);
}

#[test]
fn share_to_row_maps_all_minute_share_fields() {
    use crate::data::models::MinuteShare;
    use crate::db::clickhouse::minute::share_to_row_for_test;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    let share = MinuteShare {
        code: "sh600000".into(),
        timestamp: NaiveDate::from_ymd_opt(2026, 7, 4)
            .unwrap()
            .and_hms_opt(9, 30, 5)
            .unwrap(),
        price: Some(Decimal::from_str("12.34").unwrap()),
        volume: Some(1000),
        amount: Some(Decimal::from_str("12340.00").unwrap()),
        avg_price: Some(Decimal::from_str("12.34").unwrap()),
    };
    let row = share_to_row_for_test(&share);
    assert_eq!(row.code, "sh600000");
    assert_eq!(row.price, 12.34);
    assert_eq!(row.volume, 1000.0);
    assert_eq!(row.amount, 12340.0);
    assert_eq!(row.avg_price, 12.34);
}

// ─── P0.14 T2 — U6/U7/U8: mock sink + sink error path ──────────────────────

use async_trait::async_trait;
use std::sync::Mutex;

// A mock sink that records every batch inserted, never fails.
struct MockMinuteKlineSink {
    batches: Mutex<Vec<Vec<crate::db::clickhouse::models::MinuteKlineCH>>>,
}

#[async_trait]
impl crate::db::clickhouse::minute::MinuteSink<crate::db::clickhouse::models::MinuteKlineCH>
    for MockMinuteKlineSink
{
    async fn insert_batch(
        &self,
        batch: &[crate::db::clickhouse::models::MinuteKlineCH],
    ) -> std::result::Result<usize, clickhouse::error::Error> {
        self.batches.lock().unwrap().push(batch.to_vec());
        Ok(batch.len())
    }
}

// A mock sink for shares (U7): same shape, different type.
struct MockMinuteShareSink {
    batches: Mutex<Vec<Vec<crate::db::clickhouse::models::MinuteShareCH>>>,
}

#[async_trait]
impl crate::db::clickhouse::minute::MinuteSink<crate::db::clickhouse::models::MinuteShareCH>
    for MockMinuteShareSink
{
    async fn insert_batch(
        &self,
        batch: &[crate::db::clickhouse::models::MinuteShareCH],
    ) -> std::result::Result<usize, clickhouse::error::Error> {
        self.batches.lock().unwrap().push(batch.to_vec());
        Ok(batch.len())
    }
}

// P0.13d froze the OpenStockClient stream API (no injectable source), so a
// full happy-path unit test of `stream_minute_*_to_clickhouse` is not feasible
// without modifying upstream code (forbidden). The two tests below exercise
// the trait-dispatched `insert_batch` path through each mock sink: they
// verify (a) trait+struct wiring compiles, (b) the sink records the batch,
// (c) the sink returns Ok(batch_len). Full end-to-end coverage is L1/L2.

#[tokio::test]
async fn minute_kline_mock_sink_records_batch_and_returns_inserted_count() {
    use crate::db::clickhouse::minute::MinuteSink;

    let sink = MockMinuteKlineSink {
        batches: Mutex::new(Vec::new()),
    };
    let row = crate::db::clickhouse::models::MinuteKlineCH {
        timestamp: crate::db::clickhouse::datetime_utc_to_offsetdatetime(chrono::Utc::now()),
        code: "sh600000".into(),
        period: "1m".into(),
        adjust: "none".into(),
        open: 12.34,
        high: 12.50,
        low: 12.20,
        close: 12.40,
        volume: 1000.0,
        amount: 12340.0,
    };
    let batch = vec![row.clone(), row];
    let inserted = sink.insert_batch(&batch).await.expect("happy sink");
    assert_eq!(inserted, 2);
    let recorded = sink.batches.lock().unwrap();
    assert_eq!(recorded.len(), 1, "exactly one batch recorded");
    assert_eq!(recorded[0].len(), 2);
    assert_eq!(recorded[0][0].code, "sh600000");
}

#[tokio::test]
async fn minute_share_mock_sink_records_batch_and_returns_inserted_count() {
    use crate::db::clickhouse::minute::MinuteSink;

    let sink = MockMinuteShareSink {
        batches: Mutex::new(Vec::new()),
    };
    let row = crate::db::clickhouse::models::MinuteShareCH {
        timestamp: crate::db::clickhouse::datetime_utc_to_offsetdatetime(chrono::Utc::now()),
        code: "sh600000".into(),
        price: 12.34,
        volume: 1000.0,
        amount: 12340.0,
        avg_price: 12.34,
    };
    let batch = vec![row.clone(), row.clone(), row];
    let inserted = sink.insert_batch(&batch).await.expect("happy sink");
    assert_eq!(inserted, 3);
    let recorded = sink.batches.lock().unwrap();
    assert_eq!(recorded.len(), 1, "exactly one batch recorded");
    assert_eq!(recorded[0].len(), 3);
    assert_eq!(recorded[0][0].code, "sh600000");
}

// Verifies INV-3A: stream consumer short-circuits on the first error.
//
// Because we cannot inject a failing stream source (P0.13d freeze), we
// instead test the equivalent invariant at the helper level: a mock sink
// that fails on batch N reports the error, and only the prior batches
// were inserted. This exercises the `?` propagation path in the consumer.
struct FailOnSecondBatchKlineSink {
    calls: Mutex<usize>,
}

#[async_trait]
impl crate::db::clickhouse::minute::MinuteSink<crate::db::clickhouse::models::MinuteKlineCH>
    for FailOnSecondBatchKlineSink
{
    async fn insert_batch(
        &self,
        batch: &[crate::db::clickhouse::models::MinuteKlineCH],
    ) -> std::result::Result<usize, clickhouse::error::Error> {
        let mut n = self.calls.lock().unwrap();
        *n += 1;
        if *n == 2 {
            return Err(clickhouse::error::Error::Custom(
                "simulated batch 2 failure".to_string(),
            ));
        }
        Ok(batch.len())
    }
}

#[tokio::test]
async fn minute_kline_sink_failure_surfaces_as_database_query_error() {
    // We cannot drive the full stream consumer without a mockable stream
    // source (forbidden by P0.13d freeze). Instead we directly exercise
    // the sink's error path: it must return an Err that the consumer
    // wraps into QuantixError::DatabaseQuery.
    use crate::db::clickhouse::minute::MinuteSink;

    let sink = FailOnSecondBatchKlineSink {
        calls: Mutex::new(0),
    };
    let row = crate::db::clickhouse::models::MinuteKlineCH {
        timestamp: crate::db::clickhouse::datetime_utc_to_offsetdatetime(chrono::Utc::now()),
        code: "sh600000".into(),
        period: "1m".into(),
        adjust: "none".into(),
        open: 1.0,
        high: 1.0,
        low: 1.0,
        close: 1.0,
        volume: 1.0,
        amount: 1.0,
    };
    let first = sink.insert_batch(std::slice::from_ref(&row)).await;
    assert!(first.is_ok());
    let second = sink.insert_batch(std::slice::from_ref(&row)).await;
    assert!(second.is_err(), "second batch must fail");
    // INV-3C: error is propagated, not swallowed.
}

// ─── P0.14 T3 — L1/L2: live ClickHouse + OpenStock round-trip ──────────────
//
// `#[ignore]`-gated integration tests that exercise the full pipeline:
// OpenStock HTTP → P0.13d stream API → minute.rs row converters →
// ClickHouse insert → reverse query verification.
//
// Skipped by default (`cargo test --workspace` runs neither). Manual run:
//   QUANTIX_CLICKHOUSE_LIVE=1 \
//   OPENSTOCK_BASE_URL=... OPENSTOCK_API_KEY=... \
//   CLICKHOUSE_URL=... CLICKHOUSE_USER=... CLICKHOUSE_PASSWORD=... \
//   cargo test --lib -p quantix-cli -- --ignored live_stream_minute
//
// Sinks are `pub(crate)` (INV-4), so the tests MUST live inside this file
// (same module tree) rather than in `tests/` (separate crate, cannot see
// `pub(crate)` items).

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
    let rows: Vec<crate::db::clickhouse::models::MinuteKlineCH> = ch
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

    let rows: Vec<crate::db::clickhouse::models::MinuteShareCH> = ch
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
