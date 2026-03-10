# Phase 24 Monitor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build Phase 24A as the smallest useful monitor loop: one-shot watchlist quote inspection plus persistent price alerts with terminal triggering.

**Architecture:** Introduce a new user-facing `monitor` domain instead of extending `src/monitoring/*`. Keep CLI parsing and output thin, keep orchestration in a trait-backed service, and isolate SQLite persistence inside a dedicated storage module. Reuse existing watchlist loading and TDX quote lookup behavior where possible.

**Tech Stack:** Rust, clap, existing CLI handler pattern, `sqlx` SQLite, chrono, existing watchlist domain, existing TDX-backed quote resolution path, Markdown docs.

---

## P0 Boundary

Only implement:

- `quantix monitor watchlist --once`
- `quantix monitor alert add <code> --above <price>`
- `quantix monitor alert add <code> --below <price>`
- `quantix monitor alert list`
- `quantix monitor alert remove <id>`
- SQLite persistence for alerts
- terminal alert output when a quote matches an alert during `watchlist --once`

Explicitly exclude:

- `--refresh`
- `--repeat`
- `--once` alert lifecycle flag
- `monitor stocks`
- `monitor sector`
- `monitor concept`
- `monitor start` / `stop` / `status`
- system notifications

### Task 1: Add the top-level `monitor` CLI surface

**Files:**
- Modify: `src/cli/mod.rs`
- Test: `src/cli/mod.rs`

**Step 1: Write the failing test**

Add parser tests for:

- `quantix monitor watchlist --once`
- `quantix monitor alert add 000001 --above 16.0`
- `quantix monitor alert add 000001 --below 15.0`
- `quantix monitor alert list`
- `quantix monitor alert remove 12`

Also assert rejected combinations for `alert add`:

- missing both `--above` and `--below`
- both `--above` and `--below` together

**Step 2: Run test to verify it fails**

Run: `cargo test cli::tests::parses_monitor -- --nocapture`

Expected: FAIL because `Commands::Monitor` does not exist yet.

**Step 3: Write minimal implementation**

Add:

- `Commands::Monitor(MonitorCommands)`
- `MonitorCommands::Watchlist`
- `MonitorCommands::Alert`
- `MonitorAlertCommands::Add`
- `MonitorAlertCommands::List`
- `MonitorAlertCommands::Remove`

Argument rules:

- `watchlist` only supports `--once` in this task
- `add` must accept exactly one of `--above` or `--below`
- do not add `--refresh`, `--repeat`, `--notify`, or stock/sector subcommands

**Step 4: Run test to verify it passes**

Run: `cargo test cli::tests::parses_monitor -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/cli/mod.rs
git commit -m "feat: add phase24 monitor cli surface"
```

### Task 2: Add monitor domain models and service boundary

**Files:**
- Create: `src/monitor/mod.rs`
- Create: `src/monitor/models.rs`
- Create: `src/monitor/service.rs`
- Modify: `src/lib.rs`
- Test: `tests/monitor_service_test.rs`

**Step 1: Write the failing test**

Add service tests using fake collaborators, covering:

- `watchlist --once` builds rows from watchlist items plus current quotes
- matching `above` alerts are returned as triggered alerts
- matching `below` alerts are returned as triggered alerts
- missing quote rows do not panic and produce readable partial output
- empty watchlist returns an empty result without crashing

**Step 2: Run test to verify it fails**

Run: `cargo test --test monitor_service_test -v`

Expected: FAIL because the `monitor` module does not exist yet.

**Step 3: Write minimal implementation**

Create core types:

- `MonitorQuoteRow`
- `PriceAlert`
- `PriceAlertKind` (`Above`, `Below`)
- `TriggeredAlert`
- `MonitorWatchlistSnapshot`

Create traits for the minimum collaborators:

- `MonitorWatchlistReader`
- `MonitorQuoteReader`
- `MonitorAlertStore`

Create `MonitorService<RW, RQ, RS>` methods:

- `load_watchlist_snapshot()`
- `add_alert(code, kind, target_price, now)`
- `list_alerts()`
- `remove_alert(id)`

Design rules:

- service owns matching logic
- storage is hidden behind the store trait
- no polling loop in this task

**Step 4: Run test to verify it passes**

Run: `cargo test --test monitor_service_test -v`

Expected: PASS

**Step 5: Commit**

```bash
git add src/monitor/mod.rs src/monitor/models.rs src/monitor/service.rs src/lib.rs tests/monitor_service_test.rs
git commit -m "feat: add phase24 monitor service core"
```

### Task 3: Add runtime path and SQLite alert storage

**Files:**
- Modify: `src/core/runtime.rs`
- Create: `src/monitor/storage.rs`
- Test: `src/core/runtime.rs`
- Test: `src/monitor/storage.rs`

**Step 1: Write the failing test**

Add focused tests for:

- runtime loads `QUANTIX_MONITOR_DB_PATH`
- runtime falls back to `~/.quantix/monitor/alerts.db`
- storage creates schema automatically
- storage add/list/remove round-trips an alert
- storage updates `last_triggered_at`

**Step 2: Run test to verify it fails**

Run: `cargo test monitor_db -- --nocapture`

Expected: FAIL because monitor runtime path and storage do not exist yet.

**Step 3: Write minimal implementation**

Add runtime configuration:

- `MONITOR_DB_PATH_ENV`
- `CliRuntime.monitor_db_path`
- default path resolution next to the existing watchlist path logic

Implement SQLite storage:

- `SqliteMonitorAlertStore`
- schema creation helper
- `add_alert`
- `list_alerts`
- `remove_alert`
- `mark_triggered`

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

Do not add lifecycle flags beyond `is_active` in this task.

**Step 4: Run test to verify it passes**

Run: `cargo test monitor_db -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/core/runtime.rs src/monitor/storage.rs
git commit -m "feat: add phase24 monitor sqlite storage"
```

### Task 4: Wire monitor commands into CLI handlers

**Files:**
- Modify: `src/cli/handlers.rs`
- Test: `src/cli/handlers.rs`

**Step 1: Write the failing test**

Add handler-level tests with fake monitor collaborators covering:

- `monitor watchlist --once` returns rows
- `monitor watchlist --once` surfaces triggered alerts
- `monitor alert add --above` succeeds
- `monitor alert add --below` succeeds
- `monitor alert list` returns persisted rows
- `monitor alert remove` succeeds
- invalid `alert add` combinations return user-facing errors

Prefer testing a helper such as `execute_monitor_command_with_service(...)` instead of stdout capture.

**Step 2: Run test to verify it fails**

Run: `cargo test cli::handlers::tests::test_execute_monitor -- --nocapture`

Expected: FAIL because `run_monitor_command` does not exist.

**Step 3: Write minimal implementation**

Add:

- `run_monitor_command`
- monitor request builders / validators
- monitor print helpers

Implementation rules:

- `watchlist` only accepts `--once`
- validation should reject future deferred flags if they were accidentally added later
- triggered alerts should be printed after quote rows in a clear terminal-friendly section

Reuse existing behavior where sensible:

- load watchlist from the configured watchlist store path
- use the current quote lookup flow already proven by watchlist pricing

**Step 4: Run test to verify it passes**

Run: `cargo test cli::handlers::tests::test_execute_monitor -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/cli/handlers.rs
git commit -m "feat: wire phase24 monitor commands into cli"
```

### Task 5: Document Phase 24A and lock the boundary with hygiene tests

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

**Step 1: Write the failing test**

Extend repo hygiene coverage so docs must mention:

- `quantix monitor watchlist --once`
- `quantix monitor alert add`
- SQLite-backed alert persistence
- deferred `--refresh` / system notification behavior

**Step 2: Run test to verify it fails**

Run: `cargo test --test repo_hygiene_test -- --nocapture`

Expected: FAIL because docs do not mention the new monitor commands yet.

**Step 3: Write minimal implementation**

Update docs to document only Phase 24A:

- one-shot watchlist monitoring
- alert add/list/remove
- persistence location / env override
- explicit deferred Phase 24B items

Do not document unimplemented stock/sector/start/stop commands.

**Step 4: Run test to verify it passes**

Run: `cargo test --test repo_hygiene_test -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "docs: add phase24 monitor command usage"
```

### Task 6: Final regression verification

**Files:**
- No code changes expected

**Step 1: Run focused monitor verification**

Run: `cargo test monitor -- --nocapture`

Expected: PASS

**Step 2: Run full regression**

Run: `cargo test --all-targets`

Expected: PASS

**Step 3: Check worktree status**

Run: `git status --short`

Expected: only intended files changed or a clean tree if everything is committed.

**Step 4: If verification is green, stop and report**

Report:

- implemented commands
- test evidence
- any deferred items still intentionally excluded

If `cargo test --all-targets` fails, stop and debug before claiming completion.
