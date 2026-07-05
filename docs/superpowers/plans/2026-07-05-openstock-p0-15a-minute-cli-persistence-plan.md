# OpenStock P0.15a — Minute CLI Persistence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire P0.14's `stream_minute_klines_to_clickhouse` / `stream_minute_shares_to_clickhouse` consumers to two new CLI subcommands `data openstock import-minute-klines` / `import-minute-share` so a human (or future P0.15b scheduler) can persist minute bars and minute shares to ClickHouse by code + date range, behind a double-key gate.

**Architecture:** Purely additive CLI wiring. Two new `OpenStockCommands` variants route to two new `pub(crate) async fn` handlers in `openstock_handler.rs`. Each handler has two branches: dry-run (stream + count, no ClickHouse) and apply (construct `ClickHouseClient` + sink, call P0.14 consumer). A `pub(crate) fn compute_apply(apply: bool) -> bool` helper reads `QUANTIX_OPENSTOCK_MINUTE_APPLY` internally so unit tests exercise the real env-var name. No new modules, no new abstractions, no edits under `src/db/` or `src/sources/`.

**Tech Stack:** Rust (workspace), clap derive, tokio, futures::StreamExt, async_trait, rust_decimal, clickhouse-rs. Existing P0.13d streaming API + P0.14 sinks.

## Global Constraints

Copied verbatim from the spec at `docs/superpowers/specs/2026-07-05-openstock-p0-15a-minute-cli-persistence-design.md`:

- **Apply gate:** writes occur iff `--apply == true` AND `env QUANTIX_OPENSTOCK_MINUTE_APPLY == "yes"` (verbatim). Anything else is dry-run. (INV-CLI-1, REQ-PERSIST-008)
- **Env-var name:** `QUANTIX_OPENSTOCK_MINUTE_APPLY` (single var, both subcommands). Verbatim value `"yes"`. (D2)
- **Dry-run must not construct `ClickHouseClient`:** only `OpenStockClient` is built. (INV-CLI-2, REQ-PERSIST-009)
- **Stream API only:** handlers call `fetch_minute_klines_stream` / `fetch_minute_share_stream` (P0.13d) and `stream_minute_klines_to_clickhouse` / `stream_minute_shares_to_clickhouse` (P0.14). The batch API (`fetch_minute_klines` / `fetch_minute_share`) MUST NOT be called from the new handlers. (INV-CLI-3, REQ-PERSIST-010)
- **Subcommand naming:** `import-minute-klines` / `import-minute-share` (D1, REQ-PERSIST-006/007).
- **Single code per invocation:** `code: String` (not `Vec<String>`). (INV-CLI-4)
- **Date range required:** `--start` AND `--end` (both inclusive, `YYYY-MM-DD`). No `--date` shortform. (INV-CLI-5, D5)
- **Partial failure leaves committed batches in place** (no implicit rollback). (INV-FLOW-1)
- **Forbidden paths (card scope):** `src/db/**`, `src/sources/**`, `src/data/**`, `src/scheduler/**`, `src/backtest/**`, `src/execution/**`. New code only *consumes* P0.14 surfaces; never modifies them.
- **File size limits:** `.rs` module > 800 lines force-split; `handlers.rs` > 1200 lines force-split; `mod.rs` only `pub mod` + `pub use`.
- **Error handling:** no `.unwrap()` / `.expect()` / `panic!()` in production code. Use `?` or `.map_err(|e| QuantixError::Other(format!("...: {}", e)))`.
- **Logging:** `println!` / `eprintln!` only at CLI boundary; never in library modules.
- **Commit format:** `<type>(<scope>): <subject>` with `Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>` trailer.
- **Quality gates (every task ends green):** `cargo fmt --all -- --check` + `cargo clippy --all-targets --workspace -- -D warnings` + `cargo test --workspace`.

---

## File Structure

Decomposition locked-in before tasks:

| File | Responsibility | Change |
|------|----------------|--------|
| `src/cli/commands/data.rs` | CLI argument schema (`OpenStockCommands` enum) | +2 variants after `FetchMinuteShare` (L438) |
| `src/cli/handlers/openstock_handler.rs` | Handler bodies + `compute_apply` helper | +2 `pub(crate) async fn` after `fetch_openstock_minute_share` (L608), +1 `pub(crate) fn compute_apply` |
| `src/cli/handlers/app_shell.rs` | Top-level dispatcher (`OpenStockCommands` → handler) | +2 match arms after `FetchMinuteShare` (L417) |
| `src/cli/handlers/mod.rs` | Re-exports for handler functions | +2 names in the existing `use self::openstock_handler::{...}` block (L128-134) |
| `src/cli/tests/data.rs` | Unit tests for argument parsing + `compute_apply` env contract | +3 tests (U1, U2, U3) |
| `tests/openstock_live_import_minute.rs` | Live round-trip tests gated by env flags | NEW file (L1, L2 — `#[ignore]`) |
| `openspec/changes/openstock-data-consumption-p0-15a/{proposal,tasks,design}.md` + `specs/openstock-data-consumption/spec.md` | Requirements delta | NEW: REQ-PERSIST-006..010 |
| `.governance/programs/project-governance/cards/P0.15a.yaml` | Governance card scoping P0.15a paths | NEW file |

Each handler is self-contained: parses args, constructs clients, dispatches to dry-run or apply branch. No cross-handler state.

---

## Task 0: Baseline And Governance

**Files:**
- Create: `.governance/programs/project-governance/cards/P0.15a.yaml`
- Create: `openspec/changes/openstock-data-consumption-p0-15a/proposal.md`
- Create: `openspec/changes/openstock-data-consumption-p0-15a/tasks.md`
- Create: `openspec/changes/openstock-data-consumption-p0-15a/design.md`
- Create: `openspec/changes/openstock-data-consumption-p0-15a/specs/openstock-data-consumption/spec.md`

**Interfaces:**
- Consumes: existing governance card format (see `.governance/programs/project-governance/cards/P0.14.yaml` for shape)
- Produces: governance card `P0.15a` registered; OpenSpec change `openstock-data-consumption-p0-15a` created (draft state, no requirements yet — those land in Task 7)

- [ ] **Step 1: Create the P0.15a governance card**

Create `.governance/programs/project-governance/cards/P0.15a.yaml`:

```yaml
id: P0.15a
title: "CLI subcommands for minute-level ClickHouse persistence (klines + shares)"
state: in_progress
scope:
  allowed_paths:
    - src/cli/commands/data.rs
    - src/cli/handlers/openstock_handler.rs
    - src/cli/handlers/app_shell.rs
    - src/cli/handlers/mod.rs
    - src/cli/tests/data.rs
    - tests/openstock_live_import_minute.rs
    - openspec/changes/openstock-data-consumption-p0-15a/**
    - docs/superpowers/specs/2026-07-05-openstock-p0-15a-minute-cli-persistence-design.md
    - docs/superpowers/plans/2026-07-05-openstock-p0-15a-minute-cli-persistence-plan.md
    - .governance/programs/project-governance/cards/P0.15a.yaml
  forbidden_paths:
    - src/db/**
    - src/sources/**
    - src/data/**
    - src/scheduler/**
    - src/backtest/**
    - src/execution/**
linked_openspec: openstock-data-consumption-p0-15a
started: "2026-07-05"
acceptance_gates:
  - cargo fmt --all -- --check
  - cargo clippy --all-targets --workspace -- -D warnings
  - cargo test --workspace
  - openspec validate openstock-data-consumption-p0-15a --strict
  - openspec validate --all --strict
non_goals:
  - "Scheduler / cron triggers (P0.15b)"
  - "Multi-code orchestration per invocation (P0.15b iterates codes)"
  - "--date single-day shortform (mirror ImportKlines range-only UX)"
  - "ReplacingMergeTree / idempotent rollback (P0.14 non-goal carried forward)"
  - "assert_cmd subprocess tests (unit + live tests suffice)"
```

- [ ] **Step 2: Scaffold the OpenSpec change**

Create `openspec/changes/openstock-data-consumption-p0-15a/proposal.md`:

```markdown
## Why

P0.14 shipped `stream_minute_klines_to_clickhouse` / `stream_minute_shares_to_clickhouse` consumers and the `minute_klines` / `minute_shares` ClickHouse tables, but zero callers exist anywhere in the codebase. The library is built but unreachable from the CLI. P0.15a wires the P0.14 consumers to two user-invokable CLI subcommands so a human (or a future P0.15b scheduler) can persist minute bars and minute shares to ClickHouse by code + date range.

## What Changes

- Two new CLI subcommands: `data openstock import-minute-klines` and `data openstock import-minute-share`.
- Both gated by a double-key: `--apply == true` AND `env QUANTIX_OPENSTOCK_MINUTE_APPLY == "yes"`.
- Dry-run path streams + counts; never constructs ClickHouse client.
- Apply path consumes the P0.14 `stream_minute_*_to_clickhouse` consumer.
- 5 new requirements REQ-PERSIST-006 through REQ-PERSIST-010.

## Impact

- New surface: 2 subcommands on `OpenStockCommands`. No existing CLI behavior changes.
- New env var: `QUANTIX_OPENSTOCK_MINUTE_APPLY`.
- No database migrations, no schema changes, no new dependencies.

## Non-Goals

- Scheduler / cron triggers (P0.15b).
- Multi-code orchestration per invocation (P0.15b).
- `--date` single-day shortform.
- Idempotent rollback / ReplacingMergeTree migration.
- Real-time / live-tick import.
- `assert_cmd` subprocess tests.
```

Create `openspec/changes/openstock-data-consumption-p0-15a/tasks.md`:

```markdown
# Tasks

## 1. Baseline And Governance

- [x] Create P0.15a governance card
- [x] Scaffold OpenSpec change (proposal/tasks/design)
- [ ] Add REQ-PERSIST-006..010 + scenarios (Task 7 of implementation plan)

## 2. CLI Variants

- [ ] Add `ImportMinuteKlines` variant on `OpenStockCommands`
- [ ] Add `ImportMinuteShare` variant on `OpenStockCommands`

## 3. Handler — compute_apply helper

- [ ] Add `MINUTE_APPLY_ENV` const + `pub(crate) fn compute_apply(apply: bool) -> bool`

## 4. Handler — import_openstock_minute_klines

- [ ] Parse period/adjust/daterange
- [ ] Dry-run branch (stream + count, no ClickHouse)
- [ ] Apply branch (construct sink, call P0.14 consumer)
- [ ] Hint message when `--apply` set but env var unset

## 5. Handler — import_openstock_minute_share

- [ ] Same shape as klines, no period/adjust

## 6. Dispatcher + re-exports

- [ ] Add 2 match arms in `app_shell.rs`
- [ ] Add 2 re-exports in `handlers/mod.rs`

## 7. Unit tests U1/U2/U3

- [ ] `import_minute_args_validate_period_and_adjust`
- [ ] `compute_apply_reads_env_var`
- [ ] `compute_apply_returns_false_when_apply_flag_false`

## 8. Live tests L1/L2

- [ ] `cli_import_minute_klines_round_trip`
- [ ] `cli_import_minute_share_round_trip`

## 9. OpenSpec requirements

- [ ] Add REQ-PERSIST-006..010 with scenarios

## 10. Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --workspace -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `openspec validate openstock-data-consumption-p0-15a --strict`
- [ ] `openspec validate --all --strict`
```

Create `openspec/changes/openstock-data-consumption-p0-15a/design.md`:

```markdown
# Design Decisions

Source: docs/superpowers/specs/2026-07-05-openstock-p0-15a-minute-cli-persistence-design.md

## D1: `import-` prefix (not `fetch-` or `persist-`)

`fetch-*` = read-only. `persist-*` = shadow-write. `import-*` = canonical-write. Matches `ImportKlines` (`data.rs:74`) and `import_openstock_klines` (`openstock_handler.rs:981`).

## D2: Single env var for both subcommands

`QUANTIX_OPENSTOCK_MINUTE_APPLY` gates both. They are always used together in the future scheduler (every code gets both).

## D3: `compute_apply` env-aware helper

Reads env var internally. Tests must `std::env::set_var` the real name to pass — not a `&&` tautology.

## D4: stdout summary / stderr per-batch progress

Mirrors `fetch_openstock_minute_klines` `--stream` pattern. Lets operators redirect stdout to a file while seeing live progress.

## D5: Range-only (no --date shortform)

Mirrors `ImportKlines`. `from_cli(None, start, end)` enforces range-only when `date=None`.

## D6: Live tests in new file

`tests/openstock_live_import_minute.rs` is separate from `tests/openstock_live_minute_klines.rs` because the surfaces differ (import CLI vs fetch stream).

## Risks

See spec §9. R1 (lifetime) handled by inference; R2 (silent dry-run) handled by hint; R3 (partial failure) handled by INV-FLOW-1 documentation + per-batch output; R4 (huge range) handled by weekly chunking.
```

Create `openspec/changes/openstock-data-consumption-p0-15a/specs/openstock-data-consumption/spec.md` with empty `## ADDED Requirements` section header (filled in Task 7):

```markdown
## ADDED Requirements

### REQ-PERSIST-006: import-minute-klines subcommand

Pending — added in implementation Task 7.

### REQ-PERSIST-007: import-minute-share subcommand

Pending — added in implementation Task 7.

### REQ-PERSIST-008: double-key apply gate

Pending — added in implementation Task 7.

### REQ-PERSIST-009: dry-run shall not construct ClickHouse

Pending — added in implementation Task 7.

### REQ-PERSIST-010: stream API only

Pending — added in implementation Task 7.
```

- [ ] **Step 3: Validate OpenSpec scaffolding**

Run: `openspec validate openstock-data-consumption-p0-15a --strict`
Expected: PASS (or "no requirements yet" warning that resolves after Task 7). If validation fails on empty/pending requirements, advance Task 7 to here.

- [ ] **Step 4: Commit baseline**

```bash
git add .governance/programs/project-governance/cards/P0.15a.yaml \
        openspec/changes/openstock-data-consumption-p0-15a/
git commit -m "$(cat <<'EOF'
chore(p0.15a): scaffold governance card and openspec change

P0.15a wires P0.14 stream_minute_*_to_clickhouse consumers to two new
CLI subcommands (import-minute-klines / import-minute-share). This
commits the governance card scoping allowed/forbidden paths and the
OpenSpec change scaffold (proposal/tasks/design + empty requirements
to be filled in implementation Task 7).

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 1: CLI Variants

**Files:**
- Modify: `src/cli/commands/data.rs` (insert after L438 closing `}` of `FetchMinuteShare`)

**Interfaces:**
- Consumes: nothing (pure enum addition)
- Produces: `OpenStockCommands::ImportMinuteKlines { code, period, adjust, start, end, apply }` and `OpenStockCommands::ImportMinuteShare { code, start, end, apply }` — these are matched by Task 5's dispatcher arms.

- [ ] **Step 1: Read the existing FetchMinuteShare block to confirm exact insertion point**

Run: `grep -n "FetchMinuteShare\|^}" /opt/claude/quantix-rust/src/cli/commands/data.rs | head -10`
Expected: `FetchMinuteShare` at L418; the closing `}` of the enum at L439.

- [ ] **Step 2: Add ImportMinuteKlines and ImportMinuteShare variants**

Edit `src/cli/commands/data.rs`. Insert immediately before the closing `}` of `OpenStockCommands` (after the `FetchMinuteShare { ... }` block ends at L438):

```rust
    /// Import minute klines into ClickHouse `minute_klines` (OpenStock).
    /// Default is dry-run; pass --apply AND set QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to write.
    ImportMinuteKlines {
        /// 股票代码 (e.g. sh600000)
        #[arg(short, long)]
        code: String,

        /// 周期: 1m / 5m / 15m / 30m / 60m
        #[arg(long, default_value = "1m")]
        period: String,

        /// 复权: none / qfq / hfq
        #[arg(long, default_value = "none")]
        adjust: String,

        /// 起始日期 (YYYY-MM-DD, inclusive)
        #[arg(long)]
        start: Option<String>,

        /// 结束日期 (YYYY-MM-DD, inclusive)
        #[arg(long)]
        end: Option<String>,

        /// 实际写入 ClickHouse (默认 dry-run; 需配合 QUANTIX_OPENSTOCK_MINUTE_APPLY=yes)
        #[arg(long, default_value_t = false)]
        apply: bool,
    },

    /// Import minute shares into ClickHouse `minute_shares` (OpenStock).
    /// Default is dry-run; pass --apply AND set QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to write.
    ImportMinuteShare {
        /// 股票代码 (e.g. sh600000)
        #[arg(short, long)]
        code: String,

        /// 起始日期 (YYYY-MM-DD, inclusive)
        #[arg(long)]
        start: Option<String>,

        /// 结束日期 (YYYY-MM-DD, inclusive)
        #[arg(long)]
        end: Option<String>,

        /// 实际写入 ClickHouse (默认 dry-run; 需配合 QUANTIX_OPENSTOCK_MINUTE_APPLY=yes)
        #[arg(long, default_value_t = false)]
        apply: bool,
    },
```

- [ ] **Step 3: Verify the file compiles**

Run: `cargo build -p quantix-cli 2>&1 | tail -20`
Expected: errors on the dispatcher arms in `app_shell.rs` (Task 5 fixes those). The enum itself must parse. If errors mention `unused variable` or `never read` on the new variants, that's expected — Task 5 will use them.

- [ ] **Step 4: Run cargo fmt to normalize**

Run: `cargo fmt --all`
Expected: no functional change; just whitespace normalization.

- [ ] **Step 5: Commit**

```bash
git add src/cli/commands/data.rs
git commit -m "$(cat <<'EOF'
feat(p0.15a): add ImportMinuteKlines and ImportMinuteShare CLI variants

Two new variants on OpenStockCommands mirroring the existing ImportKlines
shape (data.rs:74). import-minute-klines carries period/adjust; import-minute-share
does not. Both gated by --apply + QUANTIX_OPENSTOCK_MINUTE_APPLY=yes (handlers
land in Task 4/5).

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: compute_apply Helper

**Files:**
- Modify: `src/cli/handlers/openstock_handler.rs` (insert before `pub(crate) async fn fetch_openstock_minute_share` at L520; new function placed near the top of the file's public surface for visibility — actual placement is the implementer's call, but it MUST be `pub(crate)` and live in this file).

**Interfaces:**
- Consumes: nothing (pure env read + bool AND)
- Produces: `pub(crate) const MINUTE_APPLY_ENV: &str = "QUANTIX_OPENSTOCK_MINUTE_APPLY";` and `pub(crate) fn compute_apply(apply: bool) -> bool`. Consumed by Task 3 (klines handler), Task 4 (share handler), and Task 6 (unit tests U2/U3).

- [ ] **Step 1: Write the failing test**

Append to `src/cli/tests/data.rs` (or create the file if missing — confirm with `ls src/cli/tests/data.rs` first):

```rust
#[cfg(test)]
mod tests_p0_15a {
    use crate::cli::handlers::openstock_handler::compute_apply;

    #[test]
    fn compute_apply_returns_false_when_apply_flag_false() {
        // U3: --apply not set → false regardless of env
        std::env::set_var("QUANTIX_OPENSTOCK_MINUTE_APPLY", "yes");
        assert!(!compute_apply(false));
        std::env::remove_var("QUANTIX_OPENSTOCK_MINUTE_APPLY");
    }

    #[test]
    fn compute_apply_reads_env_var() {
        // U2: env var must be exactly "yes" (not "true", not "1", not unset)
        std::env::set_var("QUANTIX_OPENSTOCK_MINUTE_APPLY", "yes");
        assert!(compute_apply(true));

        std::env::set_var("QUANTIX_OPENSTOCK_MINUTE_APPLY", "true");
        assert!(!compute_apply(true)); // wrong value

        std::env::set_var("QUANTIX_OPENSTOCK_MINUTE_APPLY", "1");
        assert!(!compute_apply(true)); // wrong value

        std::env::remove_var("QUANTIX_OPENSTOCK_MINUTE_APPLY");
        assert!(!compute_apply(true)); // unset
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --lib -p quantix-cli tests_p0_15a 2>&1 | tail -20`
Expected: COMPILE ERROR — `compute_apply` not found in `openstock_handler`. Confirms the test is testing the contract we're about to add.

- [ ] **Step 3: Implement compute_apply**

In `src/cli/handlers/openstock_handler.rs`, near the top of the file after the `use` declarations (or just before the first handler function — implementer's judgment, both work), add:

```rust
/// P0.15a double-key gate env-var name.
///
/// Writes to ClickHouse `minute_klines` / `minute_shares` occur iff
/// `--apply == true` AND this env var is `"yes"` (verbatim).
/// Mirrors `QUANTIX_OPENSTOCK_KLINE_APPLY` semantics (openstock_handler.rs:1055).
pub(crate) const MINUTE_APPLY_ENV: &str = "QUANTIX_OPENSTOCK_MINUTE_APPLY";

/// Compute whether to actually write to ClickHouse.
///
/// Returns `true` iff `apply` (from `--apply` CLI flag) AND the env var
/// `QUANTIX_OPENSTOCK_MINUTE_APPLY` is `"yes"` (verbatim). Anything else
/// returns `false` (dry-run).
///
/// Reading the env internally (rather than passing `env: Option<&str>`)
/// forces tests U2/U3 to set the real env-var name, exercising the contract.
pub(crate) fn compute_apply(apply: bool) -> bool {
    apply && std::env::var(MINUTE_APPLY_ENV).ok().as_deref() == Some("yes")
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib -p quantix-cli tests_p0_15a 2>&1 | tail -20`
Expected: PASS — 2 tests.

- [ ] **Step 5: Run clippy**

Run: `cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -10`
Expected: no warnings.

- [ ] **Step 6: Run fmt**

Run: `cargo fmt --all -- --check`
Expected: clean.

- [ ] **Step 7: Commit**

```bash
git add src/cli/handlers/openstock_handler.rs src/cli/tests/data.rs
git commit -m "$(cat <<'EOF'
feat(p0.15a): add compute_apply helper for minute-import double-key gate

pub(crate) fn compute_apply(apply: bool) -> bool reads
QUANTIX_OPENSTOCK_MINUTE_APPLY internally so unit tests exercise the
real env-var name (not a && tautology). Mirrors the QUANTIX_OPENSTOCK_KLINE_APPLY
pattern at openstock_handler.rs:1055.

Tests U2/U3 in src/cli/tests/data.rs::tests_p0_15a cover:
- false when apply flag false
- false when env unset / wrong value
- true only when both apply AND env == "yes"

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: import_openstock_minute_klines Handler

**Files:**
- Modify: `src/cli/handlers/openstock_handler.rs` (insert after `fetch_openstock_minute_share` ends at L608, before `fetch_openstock_all_stocks` at L610).

**Interfaces:**
- Consumes:
  - `OpenStockSettings` (existing type from `src/core/runtime/settings.rs`)
  - `OpenStockClient::from_settings(&settings)` (existing — `openstock_handler.rs:533`)
  - `OpenStockClient::fetch_minute_klines_stream(&code, period, dor, adjust)` (existing P0.13d — `openstock_handler.rs:462` uses it)
  - `crate::db::ClickHouseClient::with_default_config()` (existing — `openstock_handler.rs:1066`)
  - `ClickHouseClient::client()` returning `&clickhouse::Client` (existing — `src/db/clickhouse/mod.rs:94`)
  - `crate::db::clickhouse::minute::ClickHouseMinuteKlineSink { client }` (P0.14 — `src/db/clickhouse/minute.rs:136`)
  - `crate::db::clickhouse::minute::stream_minute_klines_to_clickhouse(&client, &sink, &code, period, start, end, adjust)` (P0.14 — `src/db/clickhouse/minute.rs:210`)
  - `crate::data::models::{MinutePeriod, AdjustType, DateOrRange}` and their `FromStr` impls (existing — `src/data/models.rs`)
  - `compute_apply(apply)` and `MINUTE_APPLY_ENV` from Task 2
- Produces: `pub(crate) async fn import_openstock_minute_klines(settings, code, period, adjust, start, end, apply) -> Result<()>` — matched by Task 5's dispatcher arm.

- [ ] **Step 1: Write the failing test (U1 — arg parser sanity)**

Append to `src/cli/tests/data.rs::tests_p0_15a`:

```rust
    #[test]
    fn import_minute_args_validate_period_and_adjust() {
        use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
        use std::str::FromStr;

        // Mirror the parsing the handler will do.
        let period_enum =
            MinutePeriod::from_str("1m").expect("1m parses");
        let adjust_enum =
            AdjustType::from_str("none").expect("none parses");
        let dor = DateOrRange::from_cli(
            None,
            Some("2026-01-01"),
            Some("2026-01-05"),
        )
        .expect("range parses");

        assert_eq!(period_enum.as_str(), "1m");
        assert_eq!(adjust_enum.as_str(), "none");
        match dor {
            DateOrRange::Range { start, end } => {
                assert_eq!(start.to_string(), "2026-01-01");
                assert_eq!(end.to_string(), "2026-01-05");
            }
            DateOrRange::Date(_) => panic!("expected Range, got Date"),
        }
    }
```

- [ ] **Step 2: Run the test to verify it passes (it tests existing parsers, not new code)**

Run: `cargo test --lib -p quantix-cli tests_p0_15a 2>&1 | tail -20`
Expected: PASS — 3 tests now. If `MinutePeriod::as_str` doesn't exist, this fails; check `src/data/models.rs:95` for the actual method name (it does exist).

- [ ] **Step 3: Implement the handler**

Insert into `src/cli/handlers/openstock_handler.rs` after L608 (after `fetch_openstock_minute_share` ends, before `fetch_openstock_all_stocks`):

```rust
/// P0.15a: `quantix data openstock import-minute-klines`.
///
/// Persists minute klines to ClickHouse `minute_klines` (P0.14 table) for a
/// single code + date range. Default is dry-run (stream + count, no
/// ClickHouse). Writes occur iff `--apply == true` AND
/// `env QUANTIX_OPENSTOCK_MINUTE_APPLY == "yes"` (see `compute_apply`).
///
/// Stream-only (P0.13d weekly chunking); never uses the batch API.
/// Partial failure leaves committed batches in place (INV-FLOW-1).
pub(crate) async fn import_openstock_minute_klines(
    settings: &OpenStockSettings,
    code: String,
    period: String,
    adjust: String,
    start: Option<String>,
    end: Option<String>,
    apply: bool,
) -> Result<()> {
    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
    use crate::sources::openstock_client::OpenStockClient;
    use futures::StreamExt;
    use std::str::FromStr;

    let period_enum = MinutePeriod::from_str(&period)
        .map_err(|e| QuantixError::Config(format!("--period: {}", e)))?;
    let adjust_enum = AdjustType::from_str(&adjust)
        .map_err(|e| QuantixError::Config(format!("--adjust: {}", e)))?;
    let dor = DateOrRange::from_cli(None, start.as_deref(), end.as_deref())?;
    let (start_date, end_date) = match dor {
        DateOrRange::Range { start, end } => (start, end),
        DateOrRange::Date(_) => {
            return Err(QuantixError::Config(
                "internal: DateOrRange unexpectedly Date".into(),
            ))
        }
    };

    let client = OpenStockClient::from_settings(settings)?;
    let will_apply = compute_apply(apply);

    println!("OpenStock import-minute-klines ({})", if will_apply { "apply" } else { "dry-run" });
    println!("  code: {}, period: {}, adjust: {}", code, period_enum.as_str(), adjust_enum.as_str());
    println!("  range: {} .. {}", start_date, end_date);

    if !will_apply {
        eprintln!("  Streaming weekly chunks (counting only, no ClickHouse writes):");
        let s = client.fetch_minute_klines_stream(
            &code,
            period_enum,
            dor.clone(),
            adjust_enum,
        );
        futures::pin_mut!(s);
        let mut batches = 0usize;
        let mut total = 0usize;
        let started = std::time::Instant::now();
        while let Some(result) = s.next().await {
            let batch = result?;
            batches += 1;
            total += batch.len();
            eprintln!(
                "  [batch {}] would_insert: +{} (cumulative: {})",
                batches, batch.len(), total
            );
        }
        println!("  dry_run: true, applied: false");
        println!("  would_insert_total: {}", total);
        println!("  batches: {}, elapsed: {:?}", batches, started.elapsed());
        if apply {
            // --apply was set but env var was not "yes" — give the operator a hint.
            println!(
                "  hint: set {}=yes to actually insert",
                MINUTE_APPLY_ENV
            );
        }
        return Ok(());
    }

    // Apply branch — construct ClickHouse client + sink, call P0.14 consumer.
    use crate::db::clickhouse::minute::{
        ClickHouseMinuteKlineSink, stream_minute_klines_to_clickhouse,
    };
    use crate::db::ClickHouseClient;

    let ch = ClickHouseClient::with_default_config().await?;
    // Lifetime is inferred: ClickHouseMinuteKlineSink<'a> borrows from `ch`.
    // `ch` and `sink` both live in this scope, outliving the await below.
    let sink = ClickHouseMinuteKlineSink { client: ch.client() };
    let stats = stream_minute_klines_to_clickhouse(
        &client,
        &sink,
        &code,
        period_enum,
        start_date,
        end_date,
        adjust_enum,
    )
    .await?;
    println!("  batches: {}", stats.batches);
    println!("  input_records: {}", stats.input_records);
    println!("  inserted_records: {}", stats.inserted_records);
    println!("  dry_run: false, applied: true");
    Ok(())
}
```

- [ ] **Step 4: Build to verify it compiles**

Run: `cargo build -p quantix-cli 2>&1 | tail -20`
Expected: errors only on the unmatched variants in `app_shell.rs` (Task 5). The handler itself must compile cleanly. Common issues:
- `ClickHouseMinuteKlineSink` not found: confirm `pub(crate)` on the struct in `src/db/clickhouse/minute.rs:136` (it is — `pub(crate) struct ClickHouseMinuteKlineSink<'a>`). The `use crate::db::clickhouse::minute::...` import should resolve.
- Lifetime mismatch: the sink struct literal must NOT have an explicit lifetime parameter — Rust infers it from `client: ch.client()` field assignment. If you wrote `ClickHouseMinuteKlineSink::<'_> { ... }`, remove the `::<'_>`.
- `MinutePeriod::as_str` not found: confirm it exists at `src/data/models.rs:95`.

- [ ] **Step 5: Run unit tests**

Run: `cargo test --lib -p quantix-cli tests_p0_15a 2>&1 | tail -20`
Expected: PASS — 3 tests.

- [ ] **Step 6: Run clippy + fmt**

Run: `cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -10`
Expected: no warnings on the new handler.

Run: `cargo fmt --all -- --check`
Expected: clean.

- [ ] **Step 7: Commit**

```bash
git add src/cli/handlers/openstock_handler.rs src/cli/tests/data.rs
git commit -m "$(cat <<'EOF'
feat(p0.15a): implement import_openstock_minute_klines handler

Two branches:
- dry-run (default): streams weekly chunks via fetch_minute_klines_stream
  and counts; no ClickHouse client constructed (INV-CLI-2).
- apply (--apply + QUANTIX_OPENSTOCK_MINUTE_APPLY=yes): constructs
  ClickHouseClient + ClickHouseMinuteKlineSink and calls the P0.14
  stream_minute_klines_to_clickhouse consumer (INV-CLI-3).

Partial failure leaves committed batches in place (INV-FLOW-1).
Hint printed when --apply set but env var not "yes" (R2 mitigation).

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: import_openstock_minute_share Handler

**Files:**
- Modify: `src/cli/handlers/openstock_handler.rs` (insert immediately after `import_openstock_minute_klines` from Task 3).

**Interfaces:**
- Consumes: same as Task 3 minus `MinutePeriod` / `AdjustType` (no period/adjust params).
- Produces: `pub(crate) async fn import_openstock_minute_share(settings, code, start, end, apply) -> Result<()>` — matched by Task 5's dispatcher arm.

- [ ] **Step 1: Implement the handler (no new test — U1/U2/U3 cover the contract; live L2 covers the round-trip)**

Insert into `src/cli/handlers/openstock_handler.rs` immediately after `import_openstock_minute_klines`:

```rust
/// P0.15a: `quantix data openstock import-minute-share`.
///
/// Persists minute shares (time-share ticks) to ClickHouse `minute_shares`
/// (P0.14 table) for a single code + date range. Default is dry-run.
/// Writes occur iff `--apply == true` AND
/// `env QUANTIX_OPENSTOCK_MINUTE_APPLY == "yes"` (see `compute_apply`).
///
/// Stream-only (P0.13d weekly chunking). Partial failure leaves committed
/// batches in place (INV-FLOW-1).
pub(crate) async fn import_openstock_minute_share(
    settings: &OpenStockSettings,
    code: String,
    start: Option<String>,
    end: Option<String>,
    apply: bool,
) -> Result<()> {
    use crate::data::models::DateOrRange;
    use crate::sources::openstock_client::OpenStockClient;
    use futures::StreamExt;

    let dor = DateOrRange::from_cli(None, start.as_deref(), end.as_deref())?;
    let (start_date, end_date) = match dor {
        DateOrRange::Range { start, end } => (start, end),
        DateOrRange::Date(_) => {
            return Err(QuantixError::Config(
                "internal: DateOrRange unexpectedly Date".into(),
            ))
        }
    };

    let client = OpenStockClient::from_settings(settings)?;
    let will_apply = compute_apply(apply);

    println!(
        "OpenStock import-minute-share ({})",
        if will_apply { "apply" } else { "dry-run" }
    );
    println!("  code: {}", code);
    println!("  range: {} .. {}", start_date, end_date);

    if !will_apply {
        eprintln!("  Streaming weekly chunks (counting only, no ClickHouse writes):");
        let s = client.fetch_minute_share_stream(&code, dor.clone());
        futures::pin_mut!(s);
        let mut batches = 0usize;
        let mut total = 0usize;
        let started = std::time::Instant::now();
        while let Some(result) = s.next().await {
            let batch = result?;
            batches += 1;
            total += batch.len();
            eprintln!(
                "  [batch {}] would_insert: +{} (cumulative: {})",
                batches, batch.len(), total
            );
        }
        println!("  dry_run: true, applied: false");
        println!("  would_insert_total: {}", total);
        println!("  batches: {}, elapsed: {:?}", batches, started.elapsed());
        if apply {
            println!(
                "  hint: set {}=yes to actually insert",
                MINUTE_APPLY_ENV
            );
        }
        return Ok(());
    }

    use crate::db::clickhouse::minute::{
        ClickHouseMinuteShareSink, stream_minute_shares_to_clickhouse,
    };
    use crate::db::ClickHouseClient;

    let ch = ClickHouseClient::with_default_config().await?;
    let sink = ClickHouseMinuteShareSink { client: ch.client() };
    let stats = stream_minute_shares_to_clickhouse(
        &client,
        &sink,
        &code,
        start_date,
        end_date,
    )
    .await?;
    println!("  batches: {}", stats.batches);
    println!("  input_records: {}", stats.input_records);
    println!("  inserted_records: {}", stats.inserted_records);
    println!("  dry_run: false, applied: true");
    Ok(())
}
```

- [ ] **Step 2: Build to verify it compiles**

Run: `cargo build -p quantix-cli 2>&1 | tail -20`
Expected: errors only on unmatched variants in `app_shell.rs` (Task 5).

- [ ] **Step 3: Run unit tests**

Run: `cargo test --lib -p quantix-cli tests_p0_15a 2>&1 | tail -10`
Expected: PASS — 3 tests (no new test added).

- [ ] **Step 4: Run clippy + fmt**

Run: `cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -10`
Expected: no warnings.

Run: `cargo fmt --all -- --check`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add src/cli/handlers/openstock_handler.rs
git commit -m "$(cat <<'EOF'
feat(p0.15a): implement import_openstock_minute_share handler

Symmetric to import_openstock_minute_klines but without period/adjust.
Two branches (dry-run / apply) gated by compute_apply. Apply branch
constructs ClickHouseMinuteShareSink and calls the P0.14
stream_minute_shares_to_clickhouse consumer.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Dispatcher Arms + Re-exports

**Files:**
- Modify: `src/cli/handlers/app_shell.rs` (insert after the `FetchMinuteShare` arm ends at L417, before `FetchAllStocks` arm at L418).
- Modify: `src/cli/handlers/mod.rs` (extend the `use self::openstock_handler::{...}` block at L128-134).

**Interfaces:**
- Consumes: `import_openstock_minute_klines` (Task 3), `import_openstock_minute_share` (Task 4).
- Produces: end-to-end CLI wiring. After this task, `data openstock import-minute-klines --code sh600000 --start ... --end ...` runs the handler.

- [ ] **Step 1: Add the dispatcher arms**

In `src/cli/handlers/app_shell.rs`, find the `OpenStockCommands::FetchMinuteShare { ... } => { ... }` arm ending at L417. Insert immediately after it (before the `FetchAllStocks` arm at L418):

```rust
            OpenStockCommands::ImportMinuteKlines {
                code,
                period,
                adjust,
                start,
                end,
                apply,
            } => {
                let rt = CliRuntime::load();
                import_openstock_minute_klines(
                    &rt.openstock,
                    code,
                    period,
                    adjust,
                    start,
                    end,
                    apply,
                )
                .await?;
            }
            OpenStockCommands::ImportMinuteShare {
                code,
                start,
                end,
                apply,
            } => {
                let rt = CliRuntime::load();
                import_openstock_minute_share(&rt.openstock, code, start, end, apply)
                    .await?;
            }
```

- [ ] **Step 2: Add the re-exports**

In `src/cli/handlers/mod.rs`, the existing block at L128-134 reads:

```rust
use self::openstock_handler::{
    fetch_openstock_all_stocks, fetch_openstock_calendar, fetch_openstock_codes,
    fetch_openstock_index, fetch_openstock_klines, fetch_openstock_minute_klines,
    fetch_openstock_minute_share, fetch_openstock_workdays, persist_openstock_live,
    shadow_rollback, shadow_verify, validate_openstock_calendar, validate_openstock_codes,
    validate_openstock_fixture, validate_openstock_index, validate_openstock_live,
};
```

Add `import_openstock_minute_klines` and `import_openstock_minute_share` (alphabetical position — between `fetch_openstock_workdays` and `persist_openstock_live`):

```rust
use self::openstock_handler::{
    fetch_openstock_all_stocks, fetch_openstock_calendar, fetch_openstock_codes,
    fetch_openstock_index, fetch_openstock_klines, fetch_openstock_minute_klines,
    fetch_openstock_minute_share, fetch_openstock_workdays, import_openstock_minute_klines,
    import_openstock_minute_share, persist_openstock_live,
    shadow_rollback, shadow_verify, validate_openstock_calendar, validate_openstock_codes,
    validate_openstock_fixture, validate_openstock_index, validate_openstock_live,
};
```

- [ ] **Step 3: Build to verify the dispatcher arms resolve**

Run: `cargo build -p quantix-cli 2>&1 | tail -10`
Expected: clean build. No errors.

- [ ] **Step 4: Smoke-test the CLI invocation prints help**

Run: `cargo run -q -- data openstock import-minute-klines --help 2>&1 | head -30`
Expected: clap-generated help text listing `--code`, `--period`, `--adjust`, `--start`, `--end`, `--apply`. Confirms the variant is wired.

Run: `cargo run -q -- data openstock import-minute-share --help 2>&1 | head -20`
Expected: clap help listing `--code`, `--start`, `--end`, `--apply` (no period/adjust).

- [ ] **Step 5: Smoke-test dry-run path with no live backend (should error on OpenStock connectivity, NOT on argument parsing)**

Run: `cargo run -q -- data openstock import-minute-klines --code sh600000 --start 2026-01-01 --end 2026-01-02 2>&1 | head -20`
Expected: error mentioning `OPENSTOCK_BASE_URL` or connectivity — confirms the args parse and the handler reaches `OpenStockClient::from_settings`. NOT an argument error.

- [ ] **Step 6: Run clippy + fmt + tests**

Run: `cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -10`
Expected: no warnings.

Run: `cargo fmt --all -- --check`
Expected: clean.

Run: `cargo test --workspace 2>&1 | tail -10`
Expected: all tests pass (1487 prior + 3 new = 1490). The two new live tests L1/L2 don't exist yet — Task 6 adds them as `#[ignore]`.

- [ ] **Step 7: Commit**

```bash
git add src/cli/handlers/app_shell.rs src/cli/handlers/mod.rs
git commit -m "$(cat <<'EOF'
feat(p0.15a): wire ImportMinuteKlines/Share dispatcher arms and re-exports

End-to-end CLI wiring for P0.15a. After this commit:

  data openstock import-minute-klines --code sh600000 --start 2026-01-01 \
    --end 2026-01-02 [--apply] [--period 1m] [--adjust none]
  data openstock import-minute-share  --code sh600000 --start 2026-01-01 \
    --end 2026-01-02 [--apply]

Both default to dry-run. Writes gated by --apply + QUANTIX_OPENSTOCK_MINUTE_APPLY=yes.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Live Tests L1/L2

**Files:**
- Create: `tests/openstock_live_import_minute.rs`

**Interfaces:**
- Consumes: `quantix_cli::cli::handlers::openstock_handler::{import_openstock_minute_klines, import_openstock_minute_share}` (must be reachable from integration tests; if they're `pub(crate)` this requires going through a `pub` entry point — see Step 1 for resolution).
- Produces: 2 `#[ignore]` live tests, gated by env vars.

- [ ] **Step 1: Determine the integration-test entry point**

Run: `grep -rn "import_openstock_klines\|pub fn run_\|pub mod handlers" /opt/claude/quantix-rust/src/cli/mod.rs /opt/claude/quantix-rust/src/cli/handlers/mod.rs | head -20`

The `import_openstock_*` handlers are `pub(crate)`. Integration tests in `/tests/` are external crates and cannot name `pub(crate)` items directly. Two resolution options:

**Option A (preferred):** Check if the codebase has an existing pattern where integration tests construct `CliRuntime` and call handlers via a `pub` wrapper. Look at `tests/openstock_live_minute_klines.rs` (existing) — if it calls a `pub` wrapper like `run_data_command`, follow that pattern.

**Option B:** If no `pub` wrapper exists, add a `pub` test-only entry point on `openstock_handler` gated by `#[cfg(any(test, feature = "test-integration"))]`. This avoids widening the public API.

Pick whichever matches the existing P0.13d / P0.14 live-test pattern. Read `tests/openstock_live_minute_klines.rs` first to see how it invokes the live path — that's the precedent.

- [ ] **Step 2: Read the existing live test for the pattern**

Run: `cat /opt/claude/quantix-rust/tests/openstock_live_minute_klines.rs | head -50`
Expected: shows env-var gating (`QUANTIX_OPENSTOCK_LIVE=1`) + how the live path is invoked. Mirror this exactly.

- [ ] **Step 3: Create the live test file**

Create `tests/openstock_live_import_minute.rs`:

```rust
//! P0.15a live round-trip tests for `import-minute-klines` / `import-minute-share`.
//!
//! Gated by env vars (all required):
//!   QUANTIX_OPENSTOCK_LIVE=1
//!   QUANTIX_CLICKHOUSE_LIVE=1
//!   OPENSTOCK_BASE_URL, OPENSTOCK_API_KEY
//!   CLICKHOUSE_URL, CLICKHOUSE_USER, CLICKHOUSE_PASSWORD
//!   QUANTIX_OPENSTOCK_MINUTE_APPLY=yes   (so the apply branch runs)
//!
//! Run: cargo test --test openstock_live_import_minute -- --ignored

#![cfg(test)]

use quantix_cli::core::runtime::CliRuntime;
// Adjust the import path based on Step 1's resolution. Likely one of:
//   use quantix_cli::cli::run_data_command;
// or via a test-integration entry point.

fn live_enabled() -> bool {
    std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() == Some("1")
        && std::env::var("QUANTIX_CLICKHOUSE_LIVE").ok().as_deref() == Some("1")
}

fn minute_apply_enabled() -> bool {
    std::env::var("QUANTIX_OPENSTOCK_MINUTE_APPLY").ok().as_deref() == Some("yes")
}

#[tokio::test]
#[ignore = "live OpenStock + ClickHouse; set QUANTIX_OPENSTOCK_LIVE=1 QUANTIX_CLICKHOUSE_LIVE=1 QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to run"]
async fn cli_import_minute_klines_round_trip() {
    if !live_enabled() || !minute_apply_enabled() {
        eprintln!("skipping: set QUANTIX_OPENSTOCK_LIVE=1 QUANTIX_CLICKHOUSE_LIVE=1 QUANTIX_OPENSTOCK_MINUTE_APPLY=yes");
        return;
    }
    let rt = CliRuntime::load();
    let code = "sh600000".to_string();
    let start = "2026-07-01".to_string();
    let end = "2026-07-01".to_string();

    // Invoke the handler via the entry point identified in Step 1.
    // If using a run_data_command wrapper:
    //   run_data_command(...).await.expect("import ok");
    // If the handler is pub(crate) and there is no wrapper, this test file
    // must instead shell out via std::process::Command to:
    //   cargo run -q -- data openstock import-minute-klines --code sh600000 \
    //     --start 2026-07-01 --end 2026-07-01 --apply
    // and assert on stdout containing "applied: true".

    // TODO: replace this stub with the actual invocation per Step 1/2.
    eprintln!("L1 stub — implement per existing live-test pattern");

    // After import, assert via ClickHouse client that rows exist:
    //   SELECT count() FROM minute_klines WHERE code = 'sh600000'
    //     AND timestamp >= '2026-07-01 00:00:00' AND timestamp < '2026-07-02 00:00:00'
    // Assert count > 0.

    // Cleanup: DELETE FROM minute_klines WHERE code = 'sh600000'
    //   AND timestamp >= '2026-07-01 00:00:00' AND timestamp < '2026-07-02 00:00:00'
}

#[tokio::test]
#[ignore = "live OpenStock + ClickHouse; set QUANTIX_OPENSTOCK_LIVE=1 QUANTIX_CLICKHOUSE_LIVE=1 QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to run"]
async fn cli_import_minute_share_round_trip() {
    if !live_enabled() || !minute_apply_enabled() {
        eprintln!("skipping: set QUANTIX_OPENSTOCK_LIVE=1 QUANTIX_CLICKHOUSE_LIVE=1 QUANTIX_OPENSTOCK_MINUTE_APPLY=yes");
        return;
    }
    // Symmetric to L1 but for minute_shares table.
    eprintln!("L2 stub — implement per Existing live-test pattern");
}
```

- [ ] **Step 4: Fill in the stubs based on Step 2's findings**

Replace the `eprintln!("L1 stub ...")` and `eprintln!("L2 stub ...")` lines with the actual invocation pattern from `tests/openstock_live_minute_klines.rs`. If that file uses `std::process::Command` to shell out (most likely — it avoids the `pub(crate)` problem entirely), use the same approach. If it imports a `pub` wrapper, use that.

For the assertions, use whatever ClickHouse query helper the existing `tests/clickhouse_live_minute_klines.rs` (P0.14) uses — `grep -n "SELECT count\|clickhouse::Client\|ClickHouseClient" /opt/claude/quantix-rust/tests/clickhouse_live_minute_klines.rs` to find it.

- [ ] **Step 5: Verify the file compiles**

Run: `cargo build --tests -p quantix-cli 2>&1 | tail -10`
Expected: clean build.

- [ ] **Step 6: Verify the tests are picked up as `ignored`**

Run: `cargo test --test openstock_live_import_minute 2>&1 | tail -10`
Expected: 0 tests run, 2 ignored. NO test failures (env vars unset → early return).

- [ ] **Step 7: Run clippy + fmt**

Run: `cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -10`
Expected: no warnings.

Run: `cargo fmt --all -- --check`
Expected: clean.

- [ ] **Step 8: Commit**

```bash
git add tests/openstock_live_import_minute.rs
git commit -m "$(cat <<'EOF'
test(p0.15a): add live round-trip tests for import-minute-* handlers

L1 cli_import_minute_klines_round_trip and L2 cli_import_minute_share_round_trip,
both #[ignore]'d. Gated by QUANTIX_OPENSTOCK_LIVE=1 + QUANTIX_CLICKHOUSE_LIVE=1
+ QUANTIX_OPENSTOCK_MINUTE_APPLY=yes. Pattern mirrors existing P0.13d live tests.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: OpenSpec Requirements

**Files:**
- Modify: `openspec/changes/openstock-data-consumption-p0-15a/specs/openstock-data-consumption/spec.md` (replace the placeholder text from Task 0).

**Interfaces:**
- Consumes: nothing
- Produces: 5 requirements (REQ-PERSIST-006..010) with scenarios, validated by `openspec validate`.

- [ ] **Step 1: Replace the placeholder requirements with full text**

Overwrite `openspec/changes/openstock-data-consumption-p0-15a/specs/openstock-data-consumption/spec.md`:

```markdown
## ADDED Requirements

### REQ-PERSIST-006: import-minute-klines subcommand

The system SHALL expose a `data openstock import-minute-klines` subcommand that persists OpenStock minute klines to ClickHouse `minute_klines`.

#### Scenario: dry-run by default

- **WHEN** the operator runs `data openstock import-minute-klines --code sh600000 --start 2026-01-01 --end 2026-01-05` (no `--apply`, no env)
- **THEN** the system streams weekly chunks from OpenStock, prints per-batch `would_insert` counts to stderr, prints a final `dry_run: true, applied: false` summary to stdout
- **AND** does NOT construct a ClickHouse client

#### Scenario: happy path apply

- **WHEN** the operator runs `data openstock import-minute-klines --code sh600000 --start 2026-01-01 --end 2026-01-05 --apply` with `QUANTIX_OPENSTOCK_MINUTE_APPLY=yes`
- **THEN** the system constructs `ClickHouseClient`, calls the P0.14 `stream_minute_klines_to_clickhouse` consumer, and prints `batches`, `input_records`, `inserted_records`, `applied: true`

#### Scenario: bad period

- **WHEN** the operator passes `--period 7m`
- **THEN** the system exits with a Config error: `--period: <reason>`

### REQ-PERSIST-007: import-minute-share subcommand

The system SHALL expose a `data openstock import-minute-share` subcommand that persists OpenStock minute shares (time-share ticks) to ClickHouse `minute_shares`.

#### Scenario: dry-run by default

- **WHEN** the operator runs `data openstock import-minute-share --code sh600000 --start 2026-01-01 --end 2026-01-05`
- **THEN** the system streams weekly chunks, prints counts, does NOT construct a ClickHouse client

#### Scenario: happy path apply

- **WHEN** the operator runs `data openstock import-minute-share --code sh600000 --start 2026-01-01 --end 2026-01-05 --apply` with `QUANTIX_OPENSTOCK_MINUTE_APPLY=yes`
- **THEN** the system calls the P0.14 `stream_minute_shares_to_clickhouse` consumer and prints `applied: true`

### REQ-PERSIST-008: double-key apply gate

The system SHALL gate ClickHouse writes on BOTH `--apply == true` AND `env QUANTIX_OPENSTOCK_MINUTE_APPLY == "yes"`. Either alone SHALL result in dry-run.

#### Scenario: apply flag set, env unset

- **WHEN** the operator runs with `--apply` but `QUANTIX_OPENSTOCK_MINUTE_APPLY` is unset or not `"yes"`
- **THEN** the system performs a dry-run and prints a hint: `hint: set QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to actually insert`

#### Scenario: env set, apply flag absent

- **WHEN** the operator has `QUANTIX_OPENSTOCK_MINUTE_APPLY=yes` in the environment but omits `--apply`
- **THEN** the system performs a dry-run (env alone is insufficient)

#### Scenario: both set

- **WHEN** `--apply` is present AND `QUANTIX_OPENSTOCK_MINUTE_APPLY=yes`
- **THEN** the system writes to ClickHouse

### REQ-PERSIST-009: dry-run shall not construct ClickHouse

In dry-run mode, the system SHALL NOT construct a `ClickHouseClient` instance. Only `OpenStockClient` is constructed.

#### Scenario: no ClickHouse credentials needed for dry-run

- **WHEN** the operator runs a dry-run on a host without `CLICKHOUSE_URL` set
- **THEN** the dry-run succeeds (no connection error)

### REQ-PERSIST-010: stream API only

The `import-minute-*` handlers SHALL consume the P0.13d streaming API (`fetch_minute_klines_stream` / `fetch_minute_share_stream`) and the P0.14 stream consumers. The batch API (`fetch_minute_klines` / `fetch_minute_share`) SHALL NOT be invoked.

#### Scenario: large range

- **WHEN** the operator runs `--start 2025-01-01 --end 2026-01-01` (1 year)
- **THEN** the system processes weekly chunks one at a time without buffering the full year into memory
```

- [ ] **Step 2: Validate the change**

Run: `openspec validate openstock-data-consumption-p0-15a --strict`
Expected: PASS.

- [ ] **Step 3: Validate all specs**

Run: `openspec validate --all --strict`
Expected: PASS (3/3 or however many are currently active).

- [ ] **Step 4: Commit**

```bash
git add openspec/changes/openstock-data-consumption-p0-15a/specs/openstock-data-consumption/spec.md
git commit -m "$(cat <<'EOF'
docs(p0.15a): add REQ-PERSIST-006..010 with scenarios

5 new requirements:
- REQ-PERSIST-006: import-minute-klines subcommand
- REQ-PERSIST-007: import-minute-share subcommand
- REQ-PERSIST-008: double-key apply gate (--apply + env)
- REQ-PERSIST-009: dry-run shall not construct ClickHouse
- REQ-PERSIST-010: stream API only (no batch API)

Each has 1-3 scenarios covering happy path, gate refusal, and edge cases.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Final Verification

**Files:**
- No file changes. Pure verification.

**Interfaces:**
- Consumes: all prior tasks.
- Produces: green quality gates, ready for review.

- [ ] **Step 1: Run cargo fmt check**

Run: `cargo fmt --all -- --check`
Expected: clean (no diff).

- [ ] **Step 2: Run cargo clippy**

Run: `cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -10`
Expected: no warnings.

- [ ] **Step 3: Run full test suite**

Run: `cargo test --workspace 2>&1 | tail -10`
Expected: all tests pass. 1487 prior + 3 new (U1/U2/U3) = 1490. The 2 live tests L1/L2 are `#[ignore]` and don't run.

- [ ] **Step 4: Run openspec validate**

Run: `openspec validate openstock-data-consumption-p0-15a --strict && openspec validate --all --strict`
Expected: both PASS.

- [ ] **Step 5: Run gitnexus detect_changes**

Run: `gitnexus detect_changes`
Expected: LOW risk on the changed files. No modifications under `src/db/`, `src/sources/`, or `src/data/`.

- [ ] **Step 6: Verify git diff --check**

Run: `git diff --check`
Expected: clean (no whitespace errors).

- [ ] **Step 7: Optional manual CLI smoke (only if OpenStock + ClickHouse are reachable)**

If you have a live OpenStock at `http://192.168.123.104:8040` and ClickHouse available:

```bash
OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
OPENSTOCK_API_KEY=<key> \
cargo run -q -- data openstock import-minute-klines \
  --code sh600000 --start 2026-07-01 --end 2026-07-02
```

Expected: dry-run output with `would_insert_total: N`. NOT an error.

Then with apply:

```bash
OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
OPENSTOCK_API_KEY=<key> \
QUANTIX_OPENSTOCK_MINUTE_APPLY=yes \
CLICKHOUSE_URL=... CLICKHOUSE_USER=... CLICKHOUSE_PASSWORD=... \
cargo run -q -- data openstock import-minute-klines \
  --code sh600000 --start 2026-07-01 --end 2026-07-02 --apply
```

Expected: `applied: true` + non-zero `inserted_records`.

- [ ] **Step 8: Final commit (if any whitespace/format fixes from Steps 1-6)**

If Steps 1-6 required any fixes, commit them now. Otherwise this step is a no-op.

```bash
git add -A
git commit -m "$(cat <<'EOF'
chore(p0.15a): final verification fixes

Quality gates green: fmt, clippy -D warnings, cargo test --workspace,
openspec validate --all --strict.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 9: Flip P0.15a governance card to complete**

Edit `.governance/programs/project-governance/cards/P0.15a.yaml`: change `state: in_progress` to `state: complete`.

```bash
git add .governance/programs/project-governance/cards/P0.15a.yaml
git commit -m "$(cat <<'EOF'
chore(p0.15a): mark governance card complete

All acceptance gates green: cargo fmt, clippy -D warnings, cargo test
--workspace, openspec validate --all --strict. P0.15a surface ready
for P0.15b scheduler design.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Self-Review Notes

**Spec coverage:**

- §0 Motivation → addressed by the whole plan (P0.14 consumers reachable from CLI).
- §1 Architecture → Tasks 1-5 wire the layers shown in the diagram.
- §2 CLI shapes → Task 1 adds both variants verbatim from spec.
- §3 Handler contracts → Tasks 2-4 implement `compute_apply` + both handlers verbatim.
- §4 Invariants → INV-CLI-1 (Task 2 + tests U2/U3), INV-CLI-2 (Task 3 dry-run branch), INV-CLI-3 (Tasks 3-4 use stream API), INV-CLI-4 (Task 1 `code: String`), INV-CLI-5 (Task 1 `--start/--end` only), INV-FLOW-1 (Task 7 REQ-PERSIST-008 + handler output prints `batches`/`inserted_records`).
- §5 Files touched → matches Task table at top of plan; all 8 files covered.
- §6 Test matrix → U1/U2/U3 in Task 2/3, L1/L2 in Task 6.
- §7 OpenSpec requirements → Task 7.
- §8 Decisions D1-D6 → encoded in handler code (D1-D5) + Task 6 file choice (D6).
- §9 Risks R1-R4 → addressed by handler code (lifetime inference, hint, partial-failure output, weekly chunking).
- §10 Non-goals → forbidden paths enforced by P0.15a.yaml card scope.
- §11 Acceptance gates → Task 8 runs all of them.
- §12 Glossary → n/a (no code action).

**Placeholder scan:** Task 6 Step 4 says "replace stub" but provides explicit instructions on how to find the pattern — this is NOT a placeholder, it's a documented unknown that requires reading the existing test to resolve (the implementer must look at `tests/openstock_live_minute_klines.rs`). All other steps contain complete code.

**Type consistency:** handler signatures in Task 3 (`import_openstock_minute_klines(settings, code, period, adjust, start, end, apply)`) match Task 5's dispatcher arm exactly. Handler signature in Task 4 (`import_openstock_minute_share(settings, code, start, end, apply)`) matches Task 5's dispatcher arm exactly. `compute_apply(apply: bool) -> bool` signature in Task 2 matches U2/U3 tests and both handler call sites.
