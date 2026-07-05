# OpenStock P0.15a — CLI subcommands for minute-level ClickHouse persistence

> **Status:** design
> **Date:** 2026-07-05
> **Depends on:** P0.14 (`stream_minute_*_to_clickhouse` consumers + `minute_klines`/`minute_shares` tables, merged)
> **Sibling:** P0.15b (scheduler / cron triggers — designed against the proven P0.15a surface)
> **Scope:** additive CLI wiring only. No new modules under `src/db/`, `src/sources/`, or `src/data/`.

---

## 0. Motivation

P0.14 shipped `stream_minute_klines_to_clickhouse` / `stream_minute_shares_to_clickhouse` consumers and the `minute_klines` / `minute_shares` ClickHouse tables. **Zero callers exist anywhere in the codebase.** The library is built but unreachable from the CLI.

P0.15a wires the P0.14 consumers to two user-invokable CLI subcommands so a human (or a future scheduler) can persist minute bars and minute shares to ClickHouse by code + date range.

---

## 1. Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ CLI: data openstock import-minute-klines/share              │
│ (src/cli/commands/data.rs:OpenStockCommands += 2 variants)  │
└──────────────────┬──────────────────────────────────────────┘
                   │ parse → DateOrRange, MinutePeriod, AdjustType
                   ▼
┌─────────────────────────────────────────────────────────────┐
│ Handler: import_openstock_minute_klines/share               │
│ (src/cli/handlers/openstock_handler.rs, new pub(crate) fns) │
└────┬───────────────────────────────────┬─────────────────────┘
     │ dry-run branch                    │ apply branch
     ▼                                   ▼
┌──────────────────┐            ┌─────────────────────────────┐
│ OpenStockClient  │            │ OpenStockClient construction │
│ stream + count   │            │ + ClickHouseClient           │
│ (no CH built)    │            │ + ClickHouseMinuteKlineSink  │
└──────────────────┘            │ stream_minute_*_to_clickhouse│
                                 │ (P0.14 consumer, reused)     │
                                 └─────────────────────────────┘
```

**Layering:** CLI → existing P0.14 `db::clickhouse` consumers. No new service layer, no new abstractions.

**P0.15a vs P0.15b:** P0.15a ships the user-invokable CLI. P0.15b (separate slice, designed later) adds the scheduler that drives the CLI surface across all codes.

---

## 2. CLI subcommand shapes

Two new variants on `OpenStockCommands` (`src/cli/commands/data.rs:176`), mirroring the existing `ImportKlines` variant (`data.rs:74`):

### 2.1 `ImportMinuteKlines`

```rust
/// Import minute klines into ClickHouse `minute_klines` (OpenStock).
/// Default is dry-run; pass --apply AND set QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to write.
ImportMinuteKlines {
    /// 股票代码 (e.g. sh600000)
    #[arg(short, long)] code: String,

    /// 周期: 1m / 5m / 15m / 30m / 60m
    #[arg(long, default_value = "1m")] period: String,

    /// 复权: none / qfq / hfq
    #[arg(long, default_value = "none")] adjust: String,

    /// 起始日期 (YYYY-MM-DD, inclusive)
    #[arg(long)] start: Option<String>,

    /// 结束日期 (YYYY-MM-DD, inclusive)
    #[arg(long)] end: Option<String>,

    /// 实际写入 ClickHouse (默认 dry-run; 需配合 QUANTIX_OPENSTOCK_MINUTE_APPLY=yes)
    #[arg(long, default_value_t = false)] apply: bool,
},
```

### 2.2 `ImportMinuteShare`

```rust
/// Import minute shares into ClickHouse `minute_shares` (OpenStock).
/// Default is dry-run; pass --apply AND set QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to write.
ImportMinuteShare {
    /// 股票代码 (e.g. sh600000)
    #[arg(short, long)] code: String,

    /// 起始日期 (YYYY-MM-DD, inclusive)
    #[arg(long)] start: Option<String>,

    /// 结束日期 (YYYY-MM-DD, inclusive)
    #[arg(long)] end: Option<String>,

    /// 实际写入 ClickHouse (默认 dry-run; 需配合 QUANTIX_OPENSTOCK_MINUTE_APPLY=yes)
    #[arg(long, default_value_t = false)] apply: bool,
},
```

**Why no `--date` single-day shortform:** The `Import*` family uses `--start/--end` ranges only (mirrors `ImportKlines`). The existing `DateOrRange::from_cli` requires either `--date` OR (`--start` AND `--end`); P0.15a handlers pass `(None, start, end)` exclusively, so the user MUST supply `--start` + `--end`. `from_cli(None, None, None)` returns `Err("at least one of --date or (--start, --end) is required")` (`src/data/models.rs:340-342`).

---

## 3. Handler contracts

Two new `pub(crate) async fn` in `src/cli/handlers/openstock_handler.rs`:

### 3.1 `import_openstock_minute_klines`

```rust
pub(crate) async fn import_openstock_minute_klines(
    settings: &OpenStockSettings,
    code: String,
    period: String,
    adjust: String,
    start: Option<String>,
    end: Option<String>,
    apply: bool,
) -> Result<()>
```

**Steps:**

1. `let period_enum = MinutePeriod::from_str(&period).map_err(|e| QuantixError::Config(format!("--period: {}", e)))?;`
2. `let adjust_enum = AdjustType::from_str(&adjust).map_err(|e| QuantixError::Config(format!("--adjust: {}", e)))?;`
3. `let dor = DateOrRange::from_cli(None, start.as_deref(), end.as_deref())?;` (returns `Range` because step 1-3 of `from_cli` require either `date` or paired `start/end`; if `start/end` are both `None`, this errors with the message at `models.rs:340-342`)
4. Extract `(start_date, end_date)`: `dor` is guaranteed to be a `Range` because step 3 calls `from_cli(None, start, end)` (no `date`), but use a safe pattern: `let (start_date, end_date) = match dor { DateOrRange::Range { start, end } => (start, end), DateOrRange::Date(_) => return Err(QuantixError::Config("internal: DateOrRange unexpectedly Date".into())) };` — avoids `unreachable!()` so a future parser change cannot panic the handler
5. `let client = OpenStockClient::from_settings(settings)?;`
6. `let will_apply = compute_apply(apply);` — see D3 for definition. Internally reads `QUANTIX_OPENSTOCK_MINUTE_APPLY` env var.

**Dry-run branch (`!will_apply`):**

```
println!("OpenStock import-minute-klines (dry-run)");
println!("  code: {}, period: {}, adjust: {}", code, period_enum.as_str(), adjust_enum.as_str());
println!("  range: {} .. {}", start_date, end_date);
eprintln!("  Streaming weekly chunks (counting only, no ClickHouse writes):");
let s = client.fetch_minute_klines_stream(&code, period_enum, dor.clone(), adjust_enum);
futures::pin_mut!(s);
let mut batches = 0usize;
let mut total = 0usize;
let started = std::time::Instant::now();
while let Some(result) = s.next().await {
    let batch = result?;
    batches += 1;
    total += batch.len();
    eprintln!("  [batch {}] would_insert: +{} (cumulative: {})", batches, batch.len(), total);
}
println!("  dry_run: true, applied: false");
println!("  would_insert_total: {}", total);
println!("  batches: {}, elapsed: {:?}", batches, started.elapsed());
if apply && !env_confirmed {
    println!("  hint: set QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to actually insert");
}
return Ok(());
```

**Apply branch (`will_apply`):**

```
use crate::db::clickhouse::minute::{ClickHouseMinuteKlineSink, stream_minute_klines_to_clickhouse};
// ↑ the sink structs are pub(crate) inside minute.rs (mod minute; is private — INV-4D from P0.14)
// stream_minute_* are also re-exported at mod.rs:12-14; either path works. Spec uses the
// minute:: path for symmetry with the sink type. Implementation may use either.

let ch = ClickHouseClient::with_default_config().await?;
let sink = ClickHouseMinuteKlineSink { client: ch.client() };
// ↑ lifetime is inferred: ClickHouseMinuteKlineSink<'a> where 'a borrows from `ch`.
//   `ch` and `sink` must outlive the await call below; this is satisfied because
//   both are in the same scope as `stream_minute_klines_to_clickhouse(...).await`.
println!("OpenStock import-minute-klines (apply)");
println!("  code: {}, period: {}, adjust: {}", code, period_enum.as_str(), adjust_enum.as_str());
println!("  range: {} .. {}", start_date, end_date);
let stats = stream_minute_klines_to_clickhouse(
    &client, &sink, &code, period_enum, start_date, end_date, adjust_enum,
).await?;
println!("  batches: {}", stats.batches);
println!("  input_records: {}", stats.input_records);
println!("  inserted_records: {}", stats.inserted_records);
println!("  dry_run: false, applied: true");
```

### 3.2 `import_openstock_minute_share`

Symmetric to §3.1, with these differences:

- No `period` / `adjust` parameters or parsing
- Calls `fetch_minute_share_stream(&code, dor.clone())` (dry-run) or `stream_minute_shares_to_clickhouse(&client, &sink, &code, start_date, end_date)` (apply)
- Inline import: `use crate::db::clickhouse::minute::{ClickHouseMinuteShareSink, stream_minute_shares_to_clickhouse};`
- Constructs `ClickHouseMinuteShareSink { client: ch.client() }` (same lifetime-inference pattern as klines)
- Output labels use `share` not `kline`

---

## 4. Invariants

### INV-CLI-1: `--apply` alone is never sufficient
**Rule:** Write occurs iff `--apply == true` AND `env QUANTIX_OPENSTOCK_MINUTE_APPLY == "yes"` (verbatim).
**Why:** Mirrors `ImportKlines`'s double-key (`--apply` + `QUANTIX_OPENSTOCK_KLINE_APPLY=yes`) and `persist_openstock_live`'s pattern. A misconfigured alias or stale shell history cannot trigger writes alone.
**Tested by:** U2, U3.

### INV-CLI-2: Dry-run never constructs ClickHouse
**Rule:** When `will_apply == false`, no `ClickHouseClient` is constructed. The dry-run path's only external dependency is `OpenStockClient`.
**Why:** Lets users validate OpenStock connectivity + range sizing without needing ClickHouse credentials. Also makes dry-run safe to run in CI / dev environments that have OpenStock but no ClickHouse.
**Enforced by:** handler control flow (`let ch = ...` lives inside the `if will_apply` branch).

### INV-CLI-3: Stream API only
**Rule:** Both handlers always call `fetch_minute_*_stream` (P0.13d streaming API). The batch (`fetch_minute_*`) API is never used.
**Why:** Unifies codepath. P0.13d's chunking + per-batch progress already solves the partial-failure telemetry story. The batch API would require collecting to `Vec` first, which is wasteful for multi-week ranges.
**Enforced by:** code review (no `fetch_minute_klines` / `fetch_minute_share` calls in the new handlers).

### INV-CLI-4: Single code per invocation
**Rule:** Each invocation processes exactly one `code`. Multi-code orchestration is P0.15b (scheduler).
**Why:** Smallest blast radius. Scheduler can iterate codes by calling the handler in-process or shelling out.
**Enforced by:** `#[arg(short, long)] code: String` (not `Vec<String>`).

### INV-CLI-5: Date range required, no single-day shortform
**Rule:** Both handlers require `--start` AND `--end`. No `--date` shortform.
**Why:** Mirrors `ImportKlines` convention. Avoids two-mode UX. `from_cli(None, None, None)` returns a clear error if both are missing.
**Enforced by:** `from_cli` parsing + `DateOrRange::from_cli(None, start.as_deref(), end.as_deref())?`.

### INV-FLOW-1: Partial failure leaves committed batches in place
**Rule:** If batch N fails during the apply branch, batches `1..N-1` remain inserted (no implicit rollback).
**Why:** P0.14 uses `async_insert=1` + `wait_for_async_insert=1`, so each batch flushes before the next begins. Re-running the command is safe — `MergeTree` has no deduplication, but `ORDER BY (date, code, period, adjust, timestamp)` makes duplicates filterable downstream. Idempotent rollback would require a `ReplacingMergeTree` migration (P0.14 non-goal).
**Documented in:** handler output prints `batches` and `inserted_records` so the operator can see exactly how many batches committed before the failure.

---

## 5. Files touched

| File | Change | LOC estimate |
|------|--------|------|
| `src/cli/commands/data.rs` | +2 enum variants on `OpenStockCommands` (after `FetchMinuteShare` at L438) | +30 |
| `src/cli/handlers/openstock_handler.rs` | +2 `pub(crate) async fn` (after `fetch_openstock_minute_share` ends at L608, before `fetch_openstock_all_stocks` at L610) + 1 `pub(crate) fn compute_apply` helper | +200 |
| `src/cli/handlers/app_shell.rs` | +2 match arms (after L417 `FetchMinuteShare`) | +20 |
| `src/cli/handlers/mod.rs` | +2 re-exports | +2 |
| `src/cli/tests/data.rs` | +3 unit tests (U1, U2, U3) | +60 |
| `tests/openstock_live_import_minute.rs` | NEW: 2 `#[ignore]` live tests (L1, L2) | +120 |
| `openspec/changes/openstock-data-consumption-p0-15a/{proposal,tasks,design}.md` + `specs/openstock-data-consumption/spec.md` | NEW: 5 REQ-PERSIST-006..010 | +200 |
| `.governance/programs/project-governance/cards/P0.15a.yaml` | NEW: card scoped to P0.15a paths | +50 |
| **Total** | | **~680 LOC** |

**Forbidden paths (card scope):** `src/db/**`, `src/sources/**`, `src/data/**`, `src/scheduler/**`, `src/backtest/**`, `src/execution/**`. The new code only *consumes* P0.14 surfaces; it must not modify them.

---

## 6. Test matrix

### Unit tests (U1-U3) — `src/cli/tests/data.rs`

Test-isolation baseline: every test that touches `compute_apply` starts with `std::env::remove_var("QUANTIX_OPENSTOCK_MINUTE_APPLY")` and ends the same way, so cargo's serial default keeps tests independent. Codebase precedent: `src/cli/tests/risk.rs:352-353` uses plain `std::env::set_var` without `serial_test`.

- **U1: `import_minute_args_validate_period_and_adjust`** — sanity test. Wires `MinutePeriod::from_str("1m")` + `AdjustType::from_str("none")` + `DateOrRange::from_cli(None, Some("2026-01-01"), Some("2026-01-05"))` and asserts the parsed values match expectations. Confirms the new handler invokes the existing parsers without breaking them.
- **U2: `compute_apply_reads_env_var`** — env-integration test (not a tautology). Calls `std::env::set_var("QUANTIX_OPENSTOCK_MINUTE_APPLY", "yes"); assert!(compute_apply(true));` then `std::env::remove_var("QUANTIX_OPENSTOCK_MINUTE_APPLY"); assert!(!compute_apply(true));`. If the env-var name is misspelled anywhere, this test fails. Mirrors `src/cli/tests/risk.rs:352-353` pattern.
- **U3: `compute_apply_returns_false_when_apply_flag_false`** — `assert!(!compute_apply(false));` with env var unset. Covers the case where `--apply` flag is missing.

### Live tests (L1/L2) — new file `tests/openstock_live_import_minute.rs`

Gated by `QUANTIX_OPENSTOCK_LIVE=1` + `QUANTIX_CLICKHOUSE_LIVE=1` + `OPENSTOCK_*` + `CLICKHOUSE_*` env vars. Both `#[ignore]` by default.

- **L1: `cli_import_minute_klines_round_trip`** — sets env, invokes `import_openstock_minute_klines` for one code × one day, queries ClickHouse: `SELECT count() FROM minute_klines WHERE code = 'sh600000' AND timestamp >= ? AND timestamp < ?`, asserts count > 0, cleans up (DELETE by code + date range).
- **L2: `cli_import_minute_share_round_trip`** — symmetric against `minute_shares`.

### Not covered (deferred)

- Full ClickHouse insert path — already covered by P0.14 U1-U8 + L1/L2; re-testing would violate DRY.
- Scheduler orchestration (per-code iteration) — P0.15b scope.
- `assert_cmd` subprocess CLI tests — P0.15a's unit tests + live tests suffice; `assert_cmd` would add a test dependency for marginal coverage.

---

## 7. OpenSpec requirements (preview)

5 new requirements to be added to `openstock-data-consumption/spec.md` as `REQ-PERSIST-006` through `REQ-PERSIST-010`:

- **REQ-PERSIST-006:** System SHALL expose `data openstock import-minute-klines` subcommand accepting `--code`, `--period`, `--adjust`, `--start`, `--end`, `--apply`.
- **REQ-PERSIST-007:** System SHALL expose `data openstock import-minute-share` subcommand accepting `--code`, `--start`, `--end`, `--apply`.
- **REQ-PERSIST-008:** System SHALL gate writes on `--apply == true` AND `env QUANTIX_OPENSTOCK_MINUTE_APPLY == "yes"`; absence of either SHALL result in dry-run.
- **REQ-PERSIST-009:** Dry-run SHALL NOT construct a ClickHouse client; only OpenStock stream consumption SHALL occur.
- **REQ-PERSIST-010:** Apply path SHALL consume the P0.14 `stream_minute_*_to_clickhouse` consumer; the batch (`fetch_minute_*`) API SHALL NOT be used.

Each requirement has 2-3 scenarios (happy path, gate refusal, dry-run count).

---

## 8. Decisions

### D1: `import-` prefix (not `fetch-` or `persist-`)
**Choice:** Subcommands named `import-minute-klines` / `import-minute-share`.
**Rationale:** Matches the existing `ImportKlines` (`data.rs:74`) and `import_openstock_klines` (`openstock_handler.rs:988`) canonical-write naming. `fetch-*` is read-only (existing `fetch-minute-*`); `persist-*` is shadow-write (existing `persist_openstock_live`). `import-*` is canonical-write. Three distinct verbs for three distinct semantics.
**Rejected:** `persist-minute-*` (collides with shadow-write semantics); `fetch-minute-*-write` (mixed verbs).

### D2: Single env var for both subcommands
**Choice:** One env var `QUANTIX_OPENSTOCK_MINUTE_APPLY` gates both klines and share writes.
**Rationale:** The two operations are always used together in the future scheduler (every code gets both). One env var simplifies operator workflow.
**Rejected:** Separate `QUANTIX_OPENSTOCK_MINUTE_KLINES_APPLY` + `QUANTIX_OPENSTOCK_MINUTE_SHARE_APPLY` (operator friction without security benefit — both are equally privileged operations on the same tables).

### D3: `compute_apply` extracted as an env-aware helper
**Choice:** A `pub(crate) fn compute_apply(apply: bool) -> bool` function lives in `openstock_handler.rs`, reads `QUANTIX_OPENSTOCK_MINUTE_APPLY` from the environment internally, and both handlers + U2/U3 call it.

```rust
const MINUTE_APPLY_ENV: &str = "QUANTIX_OPENSTOCK_MINUTE_APPLY";

pub(crate) fn compute_apply(apply: bool) -> bool {
    apply && std::env::var(MINUTE_APPLY_ENV).ok().as_deref() == Some("yes")
}
```

**Rationale:** A unit test that hardcodes `compute_apply(true, Some("yes"))` is a tautology — it tests `&&`, not the env contract. By reading the env internally, U2/U3 must `std::env::set_var(MINUTE_APPLY_ENV, "yes")` (then `remove_var` for cleanup) to make the test pass, exercising the real env-var name. Mirrors the existing pattern in `src/cli/tests/risk.rs:352-353` (`std::env::set_var("QUANTIX_RISK_PATH", ...)`).
**Test isolation:** Tests run serially by default in `cargo test`. U2/U3 use `std::env::set_var` + `std::env::remove_var(MINUTE_APPLY_ENV)` at start/end so subsequent tests see the default (unset) state.
**Rejected:** (a) Inline `let will_apply = apply && env_confirmed;` (untestable). (b) `compute_apply(apply: bool, env: Option<&str>)` (tautological test). (c) `serial_test` crate (overkill — codebase already uses plain `set_var` without it).

### D4: Dry-run prints to stdout, batch progress to stderr
**Choice:** `println!` for summary (`dry_run`, `would_insert_total`, `batches`); `eprintln!` for per-batch progress.
**Rationale:** Mirrors the existing `fetch_openstock_minute_klines` `--stream` behavior (`openstock_handler.rs:463` `eprintln!` per batch, `println!` for final summary). Lets operators redirect stdout to a file while still seeing live progress.

### D5: No `--date` single-day shortform
**Choice:** Only `--start/--end` ranges.
**Rationale:** Mirrors `ImportKlines`. The `from_cli` parser already enforces this when called with `date=None`.
**Rejected:** Adding `--date` would require either (a) two-mode UX, or (b) auto-promotion of `date` to `start=end=date`, both of which add code without serving the import use case (operators typically import ranges, not single days).

### D6: Live tests in new file `tests/openstock_live_import_minute.rs`
**Choice:** New test file rather than extending `tests/openstock_live_minute_klines.rs`.
**Rationale:** `openstock_live_minute_klines.rs` tests the P0.13d *fetch* stream; the new tests exercise the P0.15a *import* CLI handler. Different surfaces, different concerns. Mirrors the P0.14 split where `tests/clickhouse_live_minute_klines.rs` is a separate file.

---

## 9. Risks

| # | Risk | Likelihood | Mitigation |
|---|------|-----------|------------|
| R1 | Borrow-lifetime friction: `ClickHouseMinuteKlineSink<'a>` borrows from `ch`, both must outlive the `stream_minute_*_to_clickhouse` await | Low | Lifetime is inferred from struct-literal field assignment (`client: ch.client()`); no turbofish needed. Both `ch` and `sink` declared in same scope as the await call. Existing `kline.rs` uses the same `ClickHouseClient::client()` borrow pattern successfully. Plan task verifies with a compile check. |
| R2 | Operator confusion: `--apply` set but env not set → silent dry-run | Medium | Hint message at end of dry-run output (`hint: set QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to actually insert`) mirrors `persist_openstock_live` L821-823. |
| R3 | Partial-failure confusion: 3 of 5 batches committed, error on batch 4 — operator believes write failed entirely | Medium | INV-FLOW-1 docs + handler output prints `batches` + `inserted_records` so the operator sees partial progress. |
| R4 | Range too large (year+) exhausts memory in dry-run | Low | Dry-run does not buffer — it streams and counts (`fetch_minute_*_stream` yields `Vec<MinuteBar>` per chunk, but each chunk is at most 1 week). Documented in `non_goals`. |

---

## 10. Non-goals

- Scheduler / cron triggers (P0.15b)
- Multi-code orchestration per invocation (P0.15b scheduler iterates codes)
- `--date` single-day shortform
- Idempotent rollback / `ReplacingMergeTree` migration (P0.14 non-goal carried forward)
- Real-time / live-tick import (separate capability; minute-* is historical backfill only)
- CLI subprocess tests via `assert_cmd` (P0.15a unit tests + live tests suffice; `assert_cmd` would add a test dependency)
- Refactor of existing `fetch-minute-*` to share parsing with `import-minute-*` (YAGNI; ~10 lines duplicated is acceptable)
- Shadow / staging table for minute data (no `openstock_minute_*_shadow`; canonical writes only, matches `ImportKlines`)

---

## 11. Acceptance gates

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace                # U1-U3 + existing tests pass; L1/L2 ignored
openspec validate openstock-data-consumption-p0-15a --strict
openspec validate --all --strict
```

Manual (live OpenStock + live ClickHouse):

```bash
QUANTIX_OPENSTOCK_LIVE=1 QUANTIX_CLICKHOUSE_LIVE=1 \
OPENSTOCK_BASE_URL=... OPENSTOCK_API_KEY=... \
CLICKHOUSE_URL=... CLICKHOUSE_USER=... CLICKHOUSE_PASSWORD=... \
cargo test --test openstock_live_import_minute -- --ignored
```

CLI smoke (live):

```bash
QUANTIX_OPENSTOCK_MINUTE_APPLY=yes \
cargo run -q -- data openstock import-minute-klines --code sh600000 --start 2026-07-01 --end 2026-07-02 --apply
```

---

## 12. Glossary

- **Apply branch:** code path that actually writes to ClickHouse (`will_apply == true`)
- **Dry-run branch:** code path that counts only, no ClickHouse construction (`will_apply == false`)
- **Double-key gate:** write occurs iff `--apply == true` AND `env QUANTIX_OPENSTOCK_MINUTE_APPLY == "yes"`
- **P0.13d stream API:** `fetch_minute_klines_stream` / `fetch_minute_share_stream` (chunked weekly)
- **P0.14 consumer:** `stream_minute_klines_to_clickhouse` / `stream_minute_shares_to_clickhouse`
- **INV-FLOW-1:** partial-failure leaves committed batches in place (no implicit rollback)
