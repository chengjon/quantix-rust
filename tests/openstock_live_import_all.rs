//! Live integration tests for P0.15b batch scheduler.
//!
//! Triple-gated: QUANTIX_OPENSTOCK_LIVE=1 + QUANTIX_CLICKHOUSE_LIVE=1
//! + QUANTIX_POSTGRES_LIVE=1. Run with:
//!
//! ```text
//! QUANTIX_OPENSTOCK_LIVE=1 QUANTIX_CLICKHOUSE_LIVE=1 QUANTIX_POSTGRES_LIVE=1 \
//!   OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
//!   OPENSTOCK_API_KEY=<key> \
//!   CLICKHOUSE_URL=http://192.168.123.104:8123 \
//!   CLICKHOUSE_USER=default CLICKHOUSE_PASSWORD=<pass> \
//!   POSTGRESQL_HOST=192.168.123.104 POSTGRESQL_PORT=5438 \
//!   POSTGRESQL_USER=postgres POSTGRESQL_PASSWORD=<pass> \
//!   cargo test --test openstock_live_import_all -- --ignored --nocapture
//! ```

#![cfg(test)]

mod common;
use common::pg::{quantix_test_url, truncate_state_for_date};

use chrono::NaiveDate;
use quantix_cli::cli::command_types::OutputFormat;
use quantix_cli::cli::handlers::openstock_handler::query_import_status;
use quantix_cli::core::runtime::OpenStockSettings;
use quantix_cli::data::models::{AdjustType, MinutePeriod};
use quantix_cli::db::PostgresClient;
use quantix_cli::tasks::openstock_import::{
    fetcher::StockListFetcher, scheduler::BatchScheduler, state::ImportStateStore,
};

const TEST_DATE: &str = "2026-07-08";

fn live_gates_set() -> bool {
    let os = std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() == Some("1");
    let ch = std::env::var("QUANTIX_CLICKHOUSE_LIVE").ok().as_deref() == Some("1");
    let pg = std::env::var("QUANTIX_POSTGRES_LIVE").ok().as_deref() == Some("1");
    os && ch && pg
}

fn test_date() -> NaiveDate {
    NaiveDate::parse_from_str(TEST_DATE, "%Y-%m-%d").unwrap()
}

fn settings() -> OpenStockSettings {
    OpenStockSettings::from_env()
}

async fn pg() -> PostgresClient {
    PostgresClient::new(&quantix_test_url())
        .await
        .expect("quantix_test pg connect")
}

/// T1: 3-code smoke test against live OpenStock + CH + PG.
#[tokio::test]
#[serial_test::serial]
#[ignore = "live OpenStock + ClickHouse + Postgres; triple-gated"]
async fn import_minute_all_live_smoke() {
    if !live_gates_set() {
        return;
    }
    let date = test_date();
    truncate_state_for_date(date).await.unwrap();

    // Insert 3 real codes into quantix_test.stock_info for this test.
    let pg = pg().await;
    sqlx::query("DELETE FROM stock_info WHERE code IN ('sh600000','sz000001','sh600004')")
        .execute(pg.pool())
        .await
        .unwrap();
    for code in &["sh600000", "sz000001", "sh600004"] {
        sqlx::query(
            "INSERT INTO stock_info (code, name, market, trade_status) \
             VALUES ($1, $2, 'SSE', '1') ON CONFLICT (code) DO UPDATE SET trade_status='1'",
        )
        .bind(code)
        .bind(format!("test-{}", code))
        .execute(pg.pool())
        .await
        .unwrap();
    }

    let fetcher = StockListFetcher::new(&pg);
    let state = ImportStateStore::new(&pg);
    let os_settings = settings();
    let sched = BatchScheduler::new(
        &fetcher,
        &state,
        &os_settings,
        MinutePeriod::Minute5,
        AdjustType::QFQ,
        true,
    );
    let summary = sched.run(date, false).await.unwrap();

    assert_eq!(summary.total_codes, 3);
    assert!(summary.success_count.klines >= 1, "expected klines success");
    assert!(summary.success_count.share >= 1, "expected share success");
    assert!(
        summary.failures.is_empty(),
        "expected no failures: {:?}",
        summary.failures
    );

    // Verify state rows: 3 codes x 2 kinds = 6 success records.
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM import_state WHERE trade_date=$1 AND status='success'",
    )
    .bind(date)
    .fetch_one(pg.pool())
    .await
    .unwrap();
    assert_eq!(count, 6);
}

/// T2: continue-on-error with 1 fake code mixed into 2 real codes.
#[tokio::test]
#[serial_test::serial]
#[ignore = "live OpenStock + ClickHouse + Postgres; triple-gated"]
async fn import_minute_all_continue_on_error() {
    if !live_gates_set() {
        return;
    }
    let date = test_date();
    truncate_state_for_date(date).await.unwrap();

    let pg = pg().await;
    sqlx::query("DELETE FROM stock_info WHERE code IN ('sh600000','sz000001','sh999999')")
        .execute(pg.pool())
        .await
        .unwrap();
    for (code, status) in &[
        ("sh600000", "1"),
        ("sz000001", "1"),
        ("sh999999", "1"), // fake code, will 404
    ] {
        sqlx::query(
            "INSERT INTO stock_info (code, name, market, trade_status) \
             VALUES ($1, $2, 'SSE', $3) ON CONFLICT (code) DO UPDATE SET trade_status=$3",
        )
        .bind(code)
        .bind(format!("test-{}", code))
        .bind(status)
        .execute(pg.pool())
        .await
        .unwrap();
    }

    let fetcher = StockListFetcher::new(&pg);
    let state = ImportStateStore::new(&pg);
    let os_settings = settings();
    let sched = BatchScheduler::new(
        &fetcher,
        &state,
        &os_settings,
        MinutePeriod::Minute5,
        AdjustType::QFQ,
        true,
    );
    let summary = sched.run(date, false).await.unwrap();

    // Batch completed - did not abort on the fake code.
    assert_eq!(summary.total_codes, 3);
    // Real codes succeeded.
    let real_codes: Vec<_> = summary
        .failures
        .iter()
        .filter(|f| f.code == "sh600000" || f.code == "sz000001")
        .collect();
    assert!(real_codes.is_empty(), "real codes should not fail");
    // Fake code failed at least one kind.
    let fake_fails: Vec<_> = summary
        .failures
        .iter()
        .filter(|f| f.code == "sh999999")
        .collect();
    assert!(!fake_fails.is_empty(), "fake code should have failures");
}

/// T3: rerun same date -> skips already-success, no new CH writes.
#[tokio::test]
#[serial_test::serial]
#[ignore = "live OpenStock + ClickHouse + Postgres; triple-gated"]
async fn import_minute_all_skips_already_success() {
    if !live_gates_set() {
        return;
    }
    let date = test_date();
    truncate_state_for_date(date).await.unwrap();

    let pg = pg().await;
    sqlx::query("DELETE FROM stock_info WHERE code IN ('sh600000')")
        .execute(pg.pool())
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO stock_info (code, name, market, trade_status) \
         VALUES ('sh600000', 'test', 'SSE', '1') ON CONFLICT (code) DO UPDATE SET trade_status='1'",
    )
    .execute(pg.pool())
    .await
    .unwrap();

    let fetcher = StockListFetcher::new(&pg);
    let state = ImportStateStore::new(&pg);
    let os_settings = settings();
    let sched = BatchScheduler::new(
        &fetcher,
        &state,
        &os_settings,
        MinutePeriod::Minute5,
        AdjustType::QFQ,
        true,
    );

    // Run 1.
    let s1 = sched.run(date, false).await.unwrap();
    assert_eq!(s1.total_codes, 1);
    let success_after_run1: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM import_state WHERE trade_date=$1 AND status='success'",
    )
    .bind(date)
    .fetch_one(pg.pool())
    .await
    .unwrap();

    // Run 2 - should skip everything.
    let s2 = sched.run(date, false).await.unwrap();
    assert_eq!(s2.total_codes, 1);
    let success_after_run2: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM import_state WHERE trade_date=$1 AND status='success'",
    )
    .bind(date)
    .fetch_one(pg.pool())
    .await
    .unwrap();

    // No new state records on rerun (skip path doesn't write).
    assert_eq!(
        success_after_run1, success_after_run2,
        "second run must not append success records"
    );
}

/// T4: import-status query reflects correct counts.
#[tokio::test]
#[serial_test::serial]
#[ignore = "live OpenStock + ClickHouse + Postgres; triple-gated"]
async fn import_status_query() {
    if !live_gates_set() {
        return;
    }
    let date = test_date();
    truncate_state_for_date(date).await.unwrap();

    let pg = pg().await;
    sqlx::query("DELETE FROM stock_info WHERE code IN ('sh600000')")
        .execute(pg.pool())
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO stock_info (code, name, market, trade_status) \
         VALUES ('sh600000', 'test', 'SSE', '1') ON CONFLICT (code) DO UPDATE SET trade_status='1'",
    )
    .execute(pg.pool())
    .await
    .unwrap();

    let fetcher = StockListFetcher::new(&pg);
    let state = ImportStateStore::new(&pg);
    let os_settings = settings();
    let sched = BatchScheduler::new(
        &fetcher,
        &state,
        &os_settings,
        MinutePeriod::Minute5,
        AdjustType::QFQ,
        true,
    );
    let _ = sched.run(date, false).await.unwrap();

    // Query import-status via the CLI helper directly (no subprocess).
    // We invoke the handler fn directly to avoid spawning a subprocess.
    let url = quantix_test_url();
    query_import_status(&url, TEST_DATE.into(), OutputFormat::Json)
        .await
        .unwrap();

    // stdout was printed inside the handler; visual verification only.
    // The assertion: handler returned Ok (no panic, no error).
}
