# OpenStock P0.15a — Live CLI import test

> **Status:** design
> **Date:** 2026-07-07
> **Depends on:** P0.15a (`import_openstock_minute_klines` / `import_openstock_minute_share` handlers, merged commit `92b2b1e`)
> **Closes:** Acceptance gap in [P0.15a design §11](2026-07-05-openstock-p0-15a-minute-cli-persistence-design.md) — design names `cargo test --test openstock_live_import_minute -- --ignored` as a manual gate but no such test file exists.
> **Scope:** additive test file + one dev-dependency. No production code changes.

---

## 0. Motivation

P0.15a shipped the apply-path on 2026-07-07 (commit `a4fc6da`), live-validated manually against CH 26.2.4.23 (4 rows klines + 240 rows shares for `sh600000` 2026-07-03). The design's automated gate `cargo test --test openstock_live_import_minute` has no backing file. Future regressions in:

- env-var gate (`compute_apply`)
- CLI binary wiring (`app_shell.rs` dispatcher)
- sink lifetime / RowBinary serialization (the bug class that P0.15a's `DateTime` fix addressed)

would not be caught automatically. This test closes that gap.

---

## 1. Surface

**File:** `tests/openstock_live_import_minute.rs` (matches design §11 filename exactly)

**Invocation:**
```bash
QUANTIX_OPENSTOCK_LIVE=1 QUANTIX_CLICKHOUSE_LIVE=1 \
  OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
  OPENSTOCK_API_KEY=<key> \
  CLICKHOUSE_URL=http://192.168.123.104:8123 \
  CLICKHOUSE_USER=default CLICKHOUSE_PASSWORD=<pass> \
  cargo test --test openstock_live_import_minute -- --ignored
```

**Test surface (subprocess):** Each test spawns `cargo run -q -- data openstock import-minute-{klines,share} ...` with `QUANTIX_OPENSTOCK_MINUTE_APPLY=yes` (or unset, for the dry-run test). This exercises the **actual CLI binary** end-to-end — arg parse → dispatcher → handler → ClickHouse sink — which is what an operator runs. Matches the design §11 smoke command verbatim.

**Why not call handlers directly:** `import_openstock_minute_*` are `pub(crate)`. Integration tests in `/tests/` need a public entry point. Subprocess avoids promoting them to `pub` just for tests.

**Pattern reference:** `tests/openstock_live_minute_share.rs` (env-var early-return + `#[ignore]` gate).

**Gates:** Both `QUANTIX_OPENSTOCK_LIVE=1` AND `QUANTIX_CLICKHOUSE_LIVE=1` required. Missing either → early return (test passes vacuously, matching the pattern in `openstock_live_minute_share.rs`). The two gates are independent because they gate qualitatively different kinds of side-effect: `QUANTIX_OPENSTOCK_LIVE=1` permits real HTTP egress to the OpenStock NAS box, `QUANTIX_CLICKHOUSE_LIVE=1` permits real writes to the shared CH instance. Either alone is insufficient — a developer running these tests must opt into both explicitly, which prevents an accidental `cargo test --all` from corrupting shared state.

---

## 2. Test cases

**Cleanup strategy (T1/T2):** Pre-test `ALTER TABLE ... DELETE` only — no post-test cleanup. Reason: a panic between run and post-cleanup would leak rows into the next run; pre-only is idempotent and self-healing (each test starts from a known-empty state regardless of how the previous run ended). Verified compatible with local CH 26.2.4.23 (lightweight DELETE is default-on since CH 23.3).

**Date `$D = 2026-07-03`** for all tests — the 2026-07-07 manual smoke validated this date returns live data.

**Reverse-check column:** All count queries filter on `timestamp` (the `DateTime` column), not a derived `date` column — `minute_klines` / `minute_shares` have no `date` column. Use the same `timestamp >= 'YYYY-MM-DD 00:00:00' AND timestamp <= 'YYYY-MM-DD 23:59:59'` shape in every query.

### T1: `import_minute_klines_apply_writes_to_clickhouse`

```
pre:  ALTER TABLE minute_klines DELETE WHERE code='sh600000' AND toDateString(timestamp)='2026-07-03'
run:  cargo run -q -- data openstock import-minute-klines \
        --code sh600000 --period 5m --start 2026-07-03 --end 2026-07-03 --apply
      env: OPENSTOCK_*, CLICKHOUSE_*, QUANTIX_OPENSTOCK_MINUTE_APPLY=yes
```

Assertions:
- exit code 0
- stdout contains `applied: true`
- stdout contains `OpenStock import-minute-klines (apply)`
- Reverse-check: `SELECT count() FROM minute_klines WHERE code='sh600000' AND timestamp >= '2026-07-03 00:00:00' AND timestamp <= '2026-07-03 23:59:59'` → value > 0

### T2: `import_minute_share_apply_writes_to_clickhouse`

Same shape against `import-minute-share`, no `--period`/`--adjust` flags (variant rejects them).

Assertions:
- exit 0
- stdout contains `applied: true`
- Reverse-check: `SELECT count() FROM minute_shares WHERE code='sh600000' AND timestamp >= '2026-07-03 00:00:00' AND timestamp <= '2026-07-03 23:59:59'` → value > 0

### T3: `import_minute_klines_dry_run_no_env_does_not_write`

```
pre:  snapshot count = SELECT count() FROM minute_klines
        WHERE code='sh600000' AND timestamp >= '2026-07-03 00:00:00'
        AND timestamp <= '2026-07-03 23:59:59'
run:  cargo run -q -- data openstock import-minute-klines \
        --code sh600000 --period 5m --start 2026-07-03 --end 2026-07-03 --apply
      env: OPENSTOCK_*, CLICKHOUSE_* (NO QUANTIX_OPENSTOCK_MINUTE_APPLY)
```

Assertions:
- exit 0
- stdout contains `dry_run: true, applied: false`
- stdout contains `hint: set QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to actually insert` (proves `--apply` flag was parsed but env gate refused)
- Reverse-check: same count query returns **unchanged** value (no rows inserted)

---

## 3. Concurrency

`#[serial_test::serial]` on all three tests. They share the same `(code, date)` coordinates and would race without serialization.

**Dev-dependency to add** (M1 from review):
```toml
# Cargo.toml
[dev-dependencies]
serial_test = "3"
```
Not currently declared; review confirmed `grep -c serial_test Cargo.toml` → 0.

---

## 4. Reverse-check query helper

Direct `clickhouse::Client::query(...)` via a thin in-test client construction (mirrors `with_default_config` env reading). The test only needs `count()`, so a single bound query per check is sufficient — no need to extend `ClickHouseClient` API.

---

## 5. Acceptance

Static gates (run before live):
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --test openstock_live_import_minute  # no env → all 3 tests early-return, exit 0
```

Live gate (manual):
```bash
QUANTIX_OPENSTOCK_LIVE=1 QUANTIX_CLICKHOUSE_LIVE=1 ... \
  cargo test --test openstock_live_import_minute -- --ignored
# expect: 3 passed
```

---

## 6. Non-goals

- No promotion of `import_openstock_minute_*` to `pub` (would change crate API just for tests).
- No new `ClickHouseClient` method (count via inline query is enough).
- No coverage of `--adjust` variants (qfq/hfq) — out of scope, those are data-correctness concerns for OpenStock client tests, not P0.15a CLI plumbing.
- No concurrency-stress test (single `--apply` round per variant).
- No coverage of negative arg-validation (start>end, missing flags) — already covered in `src/cli/tests/data.rs` (unit tests, no live infra).
