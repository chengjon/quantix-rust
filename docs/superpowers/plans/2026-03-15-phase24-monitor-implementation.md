# Phase 24 Monitor Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the smallest useful Phase 24 monitor loop on top of the current `master`: one-shot watchlist quote inspection plus persistent price alerts with terminal triggering.

**Architecture:** Do not extend `src/monitoring/*` directly into CLI commands. Those modules are in-memory strategy/position/performance trackers without a persistent runtime source. Instead, add a separate user-facing `src/monitor/` domain for watchlist quote snapshots and SQLite-backed price alerts, then wire thin command parsing and rendering through the existing CLI handler pattern.

**Tech Stack:** Rust, clap, existing CLI handlers, `sqlx` SQLite, chrono, existing watchlist storage/service, existing quote lookup path, repo hygiene tests.

---

## Chunk 1: CLI Surface And Domain Core

### Task 1: Add the `monitor` CLI surface

**Files:**
- Modify: `src/cli/mod.rs`
- Test: `src/cli/mod.rs`

- [ ] **Step 1: Write the failing parser tests**

Add tests covering:
- `quantix monitor watchlist --once`
- `quantix monitor alert add --code 000001 --above 16.0`
- `quantix monitor alert add --code 000001 --below 15.0`
- `quantix monitor alert list`
- `quantix monitor alert remove --id 12`
- invalid add combinations:
  - missing both `--above` and `--below`
  - both `--above` and `--below`

- [ ] **Step 2: Run the parser tests to verify RED**

Run:
```bash
cargo test parses_monitor -- --nocapture
```

Expected: FAIL because `Commands::Monitor` and related subcommands do not exist yet.

- [ ] **Step 3: Add the minimal CLI definitions**

Add:
- `Commands::Monitor(MonitorCommands)`
- `MonitorCommands::Watchlist`
- `MonitorCommands::Alert`
- `MonitorAlertCommands::{Add, List, Remove}`

Argument rules:
- `watchlist` only supports `--once` in Phase 24A
- `alert add` requires exactly one of `--above` or `--below`
- do not add `--refresh`, `--follow`, `stocks`, `sector`, or `concept`

- [ ] **Step 4: Re-run the parser tests to verify GREEN**

Run:
```bash
cargo test parses_monitor -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/cli/mod.rs
git commit -m "feat: add phase24 monitor cli surface"
```

### Task 2: Add a dedicated monitor domain

**Files:**
- Create: `src/monitor/mod.rs`
- Create: `src/monitor/models.rs`
- Create: `src/monitor/service.rs`
- Modify: `src/lib.rs`
- Test: `tests/monitor_service_test.rs`

- [ ] **Step 1: Write the failing service tests**

Add integration-style tests with fake collaborators for:
- loading a watchlist snapshot with quote rows
- matching `above` alerts
- matching `below` alerts
- preserving rows when quote lookup partially fails
- returning empty snapshots for empty watchlists
- alert add/list/remove through the service boundary

- [ ] **Step 2: Run the service tests to verify RED**

Run:
```bash
cargo test --test monitor_service_test -- --nocapture
```

Expected: FAIL because the `monitor` module does not exist.

- [ ] **Step 3: Implement the minimal domain**

Create models:
- `MonitorQuoteRow`
- `MonitorWatchlistSnapshot`
- `PriceAlert`
- `PriceAlertKind`
- `TriggeredAlert`

Create collaborator traits:
- `MonitorWatchlistReader`
- `MonitorQuoteReader`
- `MonitorAlertStore`

Create `MonitorService<RW, RQ, RS>` methods:
- `load_watchlist_snapshot()`
- `add_alert(...)`
- `list_alerts()`
- `remove_alert(...)`

Keep matching logic inside the service:
- `above`: `current_price >= target_price`
- `below`: `current_price <= target_price`

- [ ] **Step 4: Re-run the service tests to verify GREEN**

Run:
```bash
cargo test --test monitor_service_test -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/monitor/mod.rs src/monitor/models.rs src/monitor/service.rs src/lib.rs tests/monitor_service_test.rs
git commit -m "feat: add phase24 monitor service core"
```

## Chunk 2: Storage, Runtime, And CLI Wiring

### Task 3: Add runtime path and SQLite alert storage

**Files:**
- Modify: `src/core/runtime.rs`
- Create: `src/monitor/storage.rs`
- Test: `src/core/runtime.rs`
- Test: `src/monitor/storage.rs`

- [ ] **Step 1: Write the failing runtime and storage tests**

Add tests for:
- `QUANTIX_MONITOR_DB_PATH` override
- default fallback path `~/.quantix/monitor/alerts.db`
- schema creation on first open
- add/list/remove round trip
- persisting `last_triggered_at`

- [ ] **Step 2: Run the focused tests to verify RED**

Run:
```bash
cargo test monitor_db -- --nocapture
```

Expected: FAIL because monitor runtime path and storage do not exist yet.

- [ ] **Step 3: Implement the minimal runtime/storage layer**

Runtime changes:
- add `MONITOR_DB_PATH_ENV`
- add `CliRuntime.monitor_db_path`
- resolve monitor DB path alongside existing watchlist path handling

Storage changes:
- add `SqliteMonitorAlertStore`
- create schema on open
- implement `MonitorAlertStore`

Schema:
```sql
CREATE TABLE IF NOT EXISTS price_alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL,
    alert_type TEXT NOT NULL,
    target_price REAL NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    last_triggered_at TEXT
);
```

Rules:
- keep alerts active after trigger in 24A
- `remove` should soft-disable via `is_active = 0`

- [ ] **Step 4: Re-run the focused tests to verify GREEN**

Run:
```bash
cargo test monitor_db -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/core/runtime.rs src/monitor/storage.rs
git commit -m "feat: add phase24 monitor sqlite storage"
```

### Task 4: Wire monitor commands into CLI handlers

**Files:**
- Modify: `src/cli/handlers.rs`
- Test: `src/cli/handlers.rs`

- [ ] **Step 1: Write the failing handler tests**

Add tests for:
- `execute_monitor_watchlist_once` returns rows
- triggered alerts are surfaced in the output model
- `alert add --above` succeeds
- `alert add --below` succeeds
- `alert list` returns stored alerts
- `alert remove` succeeds
- invalid watchlist/alert arguments return readable errors

Prefer testing a helper like `execute_monitor_command_with_service(...)` instead of stdout capture.

- [ ] **Step 2: Run the handler tests to verify RED**

Run:
```bash
cargo test cli::handlers::tests::test_execute_monitor -- --nocapture
```

Expected: FAIL because monitor handlers do not exist yet.

- [ ] **Step 3: Implement minimal handler wiring**

Add:
- `run_monitor_command`
- a helper that builds monitor command outputs from the service
- terminal print helpers for quote rows and triggered alerts

Behavior:
- `monitor watchlist --once` loads the current watchlist, resolves quotes, checks stored alerts, prints rows, then prints triggered alerts
- `monitor alert add/list/remove` delegates to the SQLite-backed store
- no polling loop, no background mode, no `--refresh`

Reuse existing pieces where practical:
- watchlist loading via existing watchlist storage/service path
- quote lookup via the same best-effort quote path already used for `watchlist list --with-price`

- [ ] **Step 4: Re-run the handler tests to verify GREEN**

Run:
```bash
cargo test cli::handlers::tests::test_execute_monitor -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/cli/handlers.rs
git commit -m "feat: wire phase24 monitor commands into cli"
```

### Task 5: Document Phase 24A and lock the boundary

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Write the failing hygiene test**

Extend doc coverage so the repository must mention:
- `quantix monitor watchlist --once`
- `quantix monitor alert add`
- SQLite alert persistence path
- explicit deferral of refresh/polling behavior

- [ ] **Step 2: Run the hygiene test to verify RED**

Run:
```bash
cargo test --test repo_hygiene_test -- --nocapture
```

Expected: FAIL because docs do not mention the monitor commands yet.

- [ ] **Step 3: Update docs with the 24A boundary only**

Document:
- one-shot watchlist monitoring
- alert add/list/remove
- `QUANTIX_MONITOR_DB_PATH`
- default path `~/.quantix/monitor/alerts.db`
- deferred items: `--refresh`, daemon/polling, sector/concept monitoring

- [ ] **Step 4: Re-run the hygiene test to verify GREEN**

Run:
```bash
cargo test --test repo_hygiene_test -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "docs: add phase24 monitor command usage"
```

### Task 6: Final regression verification

**Files:**
- No code changes expected

- [ ] **Step 1: Run the full test suite**

Run:
```bash
cargo test
```

Expected: PASS

- [ ] **Step 2: Run manual CLI smoke checks**

Run:
```bash
cargo run -- monitor watchlist --once
cargo run -- monitor alert list
```

Expected:
- commands return readable output
- empty watchlist / empty alert store degrade gracefully

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test: verify phase24 monitor integration"
```

## Execution Notes

- This plan intentionally follows the smaller `Phase 24A` boundary from the existing `phase24-monitor` worktree instead of wrapping `src/monitoring/*` directly.
- `src/monitoring/*` should remain internal strategy/portfolio/performance infrastructure for now.
- `Phase 24B` should be a separate plan after 24A lands. It can add polling, repeat behavior, and optional desktop notifications without reopening the storage and command-shape decisions above.
