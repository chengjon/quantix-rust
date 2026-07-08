# P0.15a Live CLI Import Test Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `tests/openstock_live_import_minute.rs` to close the acceptance-gap in P0.15a design §11 (live CLI apply-path test file named but never created).

**Architecture:** Subprocess-driven tests via `cargo run -q -- data openstock import-minute-{klines,share}` with `QUANTIX_OPENSTOCK_MINUTE_APPLY=yes`. Each test (1) cleans the (code, date) target on ClickHouse, (2) spawns the CLI binary, (3) asserts exit + stdout markers, (4) reads back via `ClickHouseClient::query_json` count query. Tests are gated by both `QUANTIX_OPENSTOCK_LIVE=1` and `QUANTIX_CLICKHOUSE_LIVE=1` (missing either → early return), and serialized with `#[serial_test::serial]`.

**Tech Stack:** Rust, tokio `process::Command`, `clickhouse::Client` (via `ClickHouseClient::with_default_config`), `serial_test = "3"`.

## Global Constraints

- Crate name: `quantix_cli` (Rust 2021 edition)
- Date constant: `2026-07-03` (single trading day, validated by 2026-07-07 manual smoke to return live data for `sh600000`)
- Code constant: `sh600000`
- Live gates: `QUANTIX_OPENSTOCK_LIVE=1` AND `QUANTIX_CLICKHOUSE_LIVE=1` (both required, neither alone)
- Apply env var: `QUANTIX_OPENSTOCK_MINUTE_APPLY=yes` (verbatim value, case-sensitive)
- stdout markers (must match handler output exactly): `applied: true`, `dry_run: true, applied: false`, `OpenStock import-minute-klines (apply)`, `hint: set QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to actually insert`
- No production-code changes. No crate-visibility changes (`import_openstock_minute_*` stay `pub(crate)`).
- Cleanup is **pre-only** (idempotent — each test starts from a known-empty state).
- All count queries filter on the `timestamp` column with `'YYYY-MM-DD 00:00:00'` / `'YYYY-MM-DD 23:59:59'` bounds. No `date` column exists.

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `Cargo.toml` | Modify | Add `serial_test = "3"` to `[dev-dependencies]` |
| `tests/openstock_live_import_minute.rs` | Create | Three `#[tokio::test] #[serial] #[ignore]` fns + helpers |

No production source files are touched.

---

## Task 1: Add `serial_test` dev-dependency

**Files:**
- Modify: `Cargo.toml` (find `[dev-dependencies]` block, currently has `tokio-test = "0.4"`, `criterion = "0.5"`, `tempfile = "3.8"`, `wiremock = "0.6"`)

**Interfaces:**
- Consumes: nothing
- Produces: `serial_test::serial` macro available to `tests/*.rs` and `src/*` test modules

- [ ] **Step 1: Add the dependency line**

Find the `[dev-dependencies]` section in `Cargo.toml`. Add `serial_test = "3"` after the existing entries, preserving alphabetical-ish ordering (it's not strictly alphabetical, just append after `wiremock = "0.6"`):

```toml
[dev-dependencies]
tokio-test = "0.4"
criterion = "0.5"
tempfile = "3.8"
wiremock = "0.6"
serial_test = "3"
```

- [ ] **Step 2: Verify Cargo accepts the new dep**

Run: `cargo check --tests 2>&1 | tail -5`
Expected: exit 0, no errors. (Output should end with `Finished` line. First run will fetch the `serial_test` crate — may take ~10s.)

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore(p0.15a): add serial_test dev-dependency for live import tests"
```

---

## Task 2: Create test file skeleton with helpers + early-return gates

**Files:**
- Create: `tests/openstock_live_import_minute.rs`

**Interfaces:**
- Consumes: `quantix_cli::db::ClickHouseClient`, `quantix_cli::db::clickhouse::ClickHouseClient::with_default_config`, `quantix_cli::db::clickhouse::ClickHouseClient::query_json`, `tokio::process::Command`, `std::process::Stdio`
- Produces: file exists, compiles, all three test fns are `#[ignore]`-gated and early-return (so `cargo test --test openstock_live_import_minute` with no env passes vacuously)

- [ ] **Step 1: Write the test file**

Create `tests/openstock_live_import_minute.rs` with this exact content:

```rust
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
    // T1 body — Task 3
}

#[tokio::test]
#[serial_test::serial]
#[ignore = "live OpenStock + ClickHouse; set QUANTIX_OPENSTOCK_LIVE=1 + QUANTIX_CLICKHOUSE_LIVE=1"]
async fn import_minute_share_apply_writes_to_clickhouse() {
    if !live_gates_set() {
        return;
    }
    // T2 body — Task 4
}

#[tokio::test]
#[serial_test::serial]
#[ignore = "live OpenStock + ClickHouse; set QUANTIX_OPENSTOCK_LIVE=1 + QUANTIX_CLICKHOUSE_LIVE=1"]
async fn import_minute_klines_dry_run_no_env_does_not_write() {
    if !live_gates_set() {
        return;
    }
    // T3 body — Task 5
}
```

- [ ] **Step 2: Run `cargo check --tests` to verify the skeleton compiles**

Run: `cargo check --test openstock_live_import_minute 2>&1 | tail -10`
Expected: exit 0, `Finished`. (Will produce dead-code warnings for the unused helpers, but they'll be exercised in Tasks 3-5.)

- [ ] **Step 3: Run `cargo test --test openstock_live_import_minute` with no env (vacuous pass)**

Run: `cargo test --test openstock_live_import_minute 2>&1 | tail -10`
Expected: exit 0, all 3 tests reported as ignored (or running with `return` early). Output should include `3 passed` or `0 passed, 3 ignored`.

- [ ] **Step 4: Commit**

```bash
git add tests/openstock_live_import_minute.rs
git commit -m "test(p0.15a): scaffold live import-minute tests with helpers"
```

---

## Task 3: T1 — klines apply-path

**Files:**
- Modify: `tests/openstock_live_import_minute.rs` (replace `// T1 body — Task 3` comment with the test body)

**Interfaces:**
- Consumes: `ch_delete`, `ch_count`, `run_cli`, `TEST_CODE`, `TEST_DATE`
- Produces: assertion that `import-minute-klines --apply` writes rows to ClickHouse

- [ ] **Step 1: Write the failing test body**

Replace the line `    // T1 body — Task 3` inside `import_minute_klines_apply_writes_to_clickhouse()` with:

```rust
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
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check --test openstock_live_import_minute 2>&1 | tail -5`
Expected: exit 0, no errors.

- [ ] **Step 3: Verify env-gated early-return still passes vacuously**

Run: `cargo test --test openstock_live_import_minute 2>&1 | tail -5`
Expected: exit 0.

- [ ] **Step 4: Run T1 live (manual)**

```bash
QUANTIX_OPENSTOCK_LIVE=1 QUANTIX_CLICKHOUSE_LIVE=1 \
  OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
  OPENSTOCK_API_KEY=<key> \
  cargo test --test openstock_live_import_minute -- --ignored import_minute_klines_apply_writes_to_clickhouse
```
Expected: 1 passed. If failure: read stderr from the test output and the assertion message — likely causes are (a) `cargo run` taking too long (CI timeout — not an issue locally), (b) OpenStock API key wrong, (c) CH DELETE permission denied (verify `allow_experimental_lightweight_delete` on the CH box).

- [ ] **Step 5: Commit**

```bash
git add tests/openstock_live_import_minute.rs
git commit -m "test(p0.15a): live klines apply-path with reverse-check"
```

---

## Task 4: T2 — share apply-path

**Files:**
- Modify: `tests/openstock_live_import_minute.rs` (replace `// T2 body — Task 4`)

**Interfaces:**
- Consumes: same helpers as T1
- Produces: assertion that `import-minute-share --apply` writes rows

- [ ] **Step 1: Write the failing test body**

Replace `    // T2 body — Task 4` with:

```rust
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
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check --test openstock_live_import_minute 2>&1 | tail -5`
Expected: exit 0.

- [ ] **Step 3: Run T2 live (manual)**

```bash
QUANTIX_OPENSTOCK_LIVE=1 QUANTIX_CLICKHOUSE_LIVE=1 ... \
  cargo test --test openstock_live_import_minute -- --ignored import_minute_share_apply_writes_to_clickhouse
```
Expected: 1 passed.

- [ ] **Step 4: Commit**

```bash
git add tests/openstock_live_import_minute.rs
git commit -m "test(p0.15a): live share apply-path with reverse-check"
```

---

## Task 5: T3 — dry-run no-env does not write

**Files:**
- Modify: `tests/openstock_live_import_minute.rs` (replace `// T3 body — Task 5`)

**Interfaces:**
- Consumes: `ch_count`, `run_cli`, `TEST_CODE`, `TEST_DATE`
- Produces: assertion that `--apply` without env var stays in dry-run and writes nothing

- [ ] **Step 1: Write the failing test body**

Replace `    // T3 body — Task 5` with:

```rust
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
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check --test openstock_live_import_minute 2>&1 | tail -5`
Expected: exit 0.

- [ ] **Step 3: Run T3 live (manual)**

```bash
QUANTIX_OPENSTOCK_LIVE=1 QUANTIX_CLICKHOUSE_LIVE=1 ... \
  cargo test --test openstock_live_import_minute -- --ignored import_minute_klines_dry_run_no_env_does_not_write
```
Expected: 1 passed.

- [ ] **Step 4: Commit**

```bash
git add tests/openstock_live_import_minute.rs
git commit -m "test(p0.15a): live dry-run no-env does not write"
```

---

## Task 6: Static gates + final live run

**Files:**
- No file changes; verification only

**Interfaces:**
- Consumes: completed T1-T5
- Produces: green build + 3 live tests passing

- [ ] **Step 1: fmt**

Run: `cargo fmt --all -- --check 2>&1 | tail -5`
Expected: no output (clean).

- [ ] **Step 2: clippy**

Run: `cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -5`
Expected: `cargo clippy: No issues found`.

- [ ] **Step 3: full test suite (vacuous)**

Run: `cargo test --test openstock_live_import_minute 2>&1 | tail -5`
Expected: 3 ignored / passing.

- [ ] **Step 4: full live suite (manual)**

```bash
QUANTIX_OPENSTOCK_LIVE=1 QUANTIX_CLICKHOUSE_LIVE=1 ... \
  cargo test --test openstock_live_import_minute -- --ignored
```
Expected: `3 passed`.

- [ ] **Step 5: Run `gitnexus_detect_changes()` to verify scope**

Run: `gitnexus_detect_changes(scope: "unstaged")` then `gitnexus_detect_changes(scope: "compare", base_ref: "master~10")`
Expected: changes are limited to `Cargo.toml` (1 line) and `tests/openstock_live_import_minute.rs` (new). No production source modified.

---

## Self-Review

**1. Spec coverage** — all three T1/T2/T3 from spec §2 mapped to Tasks 3/4/5. Tasks 1/2/6 are setup + verification, matching spec §3 (dev-dep) and §5 (acceptance). Cleanup strategy (§2 pre-only) implemented in T1/T2 via `ch_delete` before run. Reverse-check helper (§4) is `ch_count` using `query_json`. ✓

**2. Placeholder scan** — no "TBD", "TODO", "implement later". Each task step has concrete code or exact command + expected output.

**3. Type consistency** — `ch_count` returns `u64`, `ch_delete` returns unit, `run_cli` returns `(ExitStatus, String, String)`. All used consistently across T1/T2/T3.

**4. Spec gates** — both `QUANTIX_OPENSTOCK_LIVE=1` AND `QUANTIX_CLICKHOUSE_LIVE=1` checked in `live_gates_set()` (matches spec §1 dual-gate rationale).

**5. Edge case — dry-run stdout**: T3 asserts `dry-run` header. Reading `openstock_handler.rs:670-672` confirms `if will_apply { "apply" } else { "dry-run" }` — so the marker is `OpenStock import-minute-klines (dry-run)`. Captured in T3.

**6. Edge case — apply-path assertion variance**: T1 asserts `OpenStock import-minute-klines (apply)`, T2 asserts `OpenStock import-minute-share (apply)`. Both confirmed against handler source at `openstock_handler.rs:670` (klines) and `:770` (share).
