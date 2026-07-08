//! Live integration tests for P0.15a CLI apply-path.
//!
//! Drives the actual CLI binary via `cargo run` and asserts ClickHouse state
//! changes. Closes the acceptance gap from P0.15a design §11.
//!
//! Skipped by default. Run with:
//!   QUANTIX_OPENSTOCK_LIVE=1 QUANTIX_CLICKHOUSE_LIVE=1 \
//!   OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
//!   OPENSTOCK_API_KEY=<key> \
//!   CLICKHOUSE_URL=http://192.168.123.104:8123 \
//!   CLICKHOUSE_USER=default CLICKHOUSE_PASSWORD=<pass> \
//!   cargo test --test openstock_live_import_minute -- --ignored

#![cfg(test)]

use quantix_cli::db::ClickHouseClient;
use std::process::Stdio;

const TEST_CODE: &str = "sh600000";
const TEST_DATE: &str = "2026-07-03";

/// True iff both live gates are set. Each test must call this and early-return
/// if false, so `cargo test` without env passes vacuously.
fn live_gates_set() -> bool {
    let os = std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() == Some("1");
    let ch = std::env::var("QUANTIX_CLICKHOUSE_LIVE").ok().as_deref() == Some("1");
    os && ch
}

/// Construct a ClickHouseClient from env (mirrors handler's
/// `with_default_config` which reads `ClickHouseSettings::from_env`).
async fn ch_from_env() -> ClickHouseClient {
    ClickHouseClient::with_default_config()
        .await
        .expect("ClickHouse client from CLICKHOUSE_* env")
}

/// Run `ALTER TABLE <table> DELETE WHERE code='<code>' AND
/// toDateString(timestamp)='<date>'. Lightweight delete (CH 23.3+, default on).
async fn ch_delete(code: &str, date: &str, table: &str) {
    let ch = ch_from_env().await;
    let sql = format!(
        "ALTER TABLE {table} DELETE WHERE code = '{code}' AND toDateString(timestamp) = '{date}'"
    );
    ch.query_json::<serde_json::Value>(&sql)
        .await
        .expect(&format!("delete on {table} ok"));
}

/// Count rows where code matches and timestamp falls within the given
/// calendar date. Uses HTTP JSON path (bypasses RowBinary).
async fn ch_count(code: &str, date: &str, table: &str) -> u64 {
    let ch = ch_from_env().await;
    let sql = format!(
        "SELECT count() as cnt FROM {table} WHERE code = '{code}' \
         AND timestamp >= '{date} 00:00:00' AND timestamp <= '{date} 23:59:59'"
    );
    #[derive(serde::Deserialize)]
    struct Row {
        cnt: u64,
    }
    let rows: Vec<Row> = ch
        .query_json(&sql)
        .await
        .expect(&format!("count on {table} ok"));
    rows.first().map(|r| r.cnt).unwrap_or(0)
}

/// Spawn `cargo run -q -- <args>` with QUANTIX_OPENSTOCK_MINUTE_APPLY set or
/// unset per `apply_env`. Returns (exit_status, stdout, stderr).
async fn run_cli<I, S>(args: I, apply_env: Option<&str>) -> (std::process::ExitStatus, String, String)
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.arg("run").arg("-q").arg("--");
    for a in args {
        cmd.arg(a);
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Inherit OPENSTOCK_*, CLICKHOUSE_* env vars; set apply env only if Some.
    if let Some(v) = apply_env {
        cmd.env("QUANTIX_OPENSTOCK_MINUTE_APPLY", v);
    } else {
        cmd.env_remove("QUANTIX_OPENSTOCK_MINUTE_APPLY");
    }
    let output = cmd.output().await.expect("cargo run spawn ok");
    (
        output.status,
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[tokio::test]
#[serial_test::serial]
#[ignore = "live OpenStock + ClickHouse; set QUANTIX_OPENSTOCK_LIVE=1 + QUANTIX_CLICKHOUSE_LIVE=1"]
async fn import_minute_klines_apply_writes_to_clickhouse() {
    if !live_gates_set() {
        return;
    }
    ch_delete(TEST_CODE, TEST_DATE, "minute_klines").await;

    let args = [
        "data",
        "openstock",
        "import-minute-klines",
        "--code",
        TEST_CODE,
        "--period",
        "5m",
        "--start",
        TEST_DATE,
        "--end",
        TEST_DATE,
        "--apply",
    ];
    let (status, stdout, stderr) = run_cli(args, Some("yes")).await;
    assert!(status.success(), "exit not success: {status}\nstderr: {stderr}");
    assert!(
        stdout.contains("OpenStock import-minute-klines (apply)"),
        "missing apply header in stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("applied: true"),
        "missing applied:true marker:\n{stdout}"
    );

    let count = ch_count(TEST_CODE, TEST_DATE, "minute_klines").await;
    assert!(count > 0, "expected rows in minute_klines for {TEST_CODE} {TEST_DATE}, got 0");
    println!("T1 minute_klines rows: {count}");
}

#[tokio::test]
#[serial_test::serial]
#[ignore = "live OpenStock + ClickHouse; set QUANTIX_OPENSTOCK_LIVE=1 + QUANTIX_CLICKHOUSE_LIVE=1"]
async fn import_minute_share_apply_writes_to_clickhouse() {
    if !live_gates_set() {
        return;
    }
    ch_delete(TEST_CODE, TEST_DATE, "minute_shares").await;

    let args = [
        "data",
        "openstock",
        "import-minute-share",
        "--code",
        TEST_CODE,
        "--start",
        TEST_DATE,
        "--end",
        TEST_DATE,
        "--apply",
    ];
    let (status, stdout, stderr) = run_cli(args, Some("yes")).await;
    assert!(status.success(), "exit not success: {status}\nstderr: {stderr}");
    assert!(
        stdout.contains("OpenStock import-minute-share (apply)"),
        "missing apply header in stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("applied: true"),
        "missing applied:true marker:\n{stdout}"
    );

    let count = ch_count(TEST_CODE, TEST_DATE, "minute_shares").await;
    assert!(count > 0, "expected rows in minute_shares for {TEST_CODE} {TEST_DATE}, got 0");
    println!("T2 minute_shares rows: {count}");
}

#[tokio::test]
#[serial_test::serial]
#[ignore = "live OpenStock + ClickHouse; set QUANTIX_OPENSTOCK_LIVE=1 + QUANTIX_CLICKHOUSE_LIVE=1"]
async fn import_minute_klines_dry_run_no_env_does_not_write() {
    if !live_gates_set() {
        return;
    }
    let before = ch_count(TEST_CODE, TEST_DATE, "minute_klines").await;

    let args = [
        "data",
        "openstock",
        "import-minute-klines",
        "--code",
        TEST_CODE,
        "--period",
        "5m",
        "--start",
        TEST_DATE,
        "--end",
        TEST_DATE,
        "--apply",
    ];
    // apply_env=None — env var is unset, must stay dry-run.
    let (status, stdout, stderr) = run_cli(args, None).await;
    assert!(status.success(), "exit not success: {status}\nstderr: {stderr}");
    assert!(
        stdout.contains("OpenStock import-minute-klines (dry-run)"),
        "missing dry-run header in stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("dry_run: true, applied: false"),
        "missing dry_run marker:\n{stdout}"
    );
    assert!(
        stdout.contains("hint: set QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to actually insert"),
        "missing hint message:\n{stdout}"
    );

    let after = ch_count(TEST_CODE, TEST_DATE, "minute_klines").await;
    assert_eq!(
        before, after,
        "row count changed despite dry-run: before={before}, after={after}"
    );
    println!("T3 dry-run preserved row count: {before}");
}
