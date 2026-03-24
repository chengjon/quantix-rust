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

---

# Phase 24B Monitor Automation Design

**Date:** 2026-03-16
**Status:** Approved in-session
**Depends On:** Current `master` with Phase 24A/25A/28A green baseline (`master` @ `3413548`)

> This appendix is the source of truth for the next approved monitor slice: turn the existing one-shot watchlist monitor into a reusable automation loop with foreground repeat mode, a daemon entrypoint, and `systemd --user` service integration on WSL2.

## Goal

Build the smallest useful automation layer on top of the existing monitor and stop rules so a user can:

1. Run the current watchlist monitor continuously in the foreground
2. Run the same loop as a background daemon managed by `systemd --user`
3. Persist business trigger history for later inspection
4. Keep the loop resilient when quote lookup is partially unavailable

This phase should automate monitoring, not redesign the quote pipeline, not replace OS logging, and not introduce desktop notifications or automatic trading actions.

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. Start a foreground repeat loop for the current watchlist
2. Install and manage a user-level monitor service under WSL2 `systemd`
3. Configure the monitor interval and optional watchlist group without editing the unit file
4. Inspect persisted trigger history for price alerts and stop events
5. Avoid duplicate event spam while a condition stays continuously true

### Exact CLI boundary

Only implement:

```bash
quantix monitor watchlist --once
quantix monitor watchlist --repeat

quantix monitor config show
quantix monitor config set --interval-seconds <N>
quantix monitor config set --group <GROUP>
quantix monitor config clear-group
quantix monitor config set --persist-events <true|false>

quantix monitor daemon run

quantix monitor service install
quantix monitor service uninstall
quantix monitor service start
quantix monitor service stop
quantix monitor service status
quantix monitor service enable
quantix monitor service disable

quantix monitor event list [--limit <N>] [--code <CODE>] [--type <TYPE>]
```

Rules:

- `watchlist --once` must preserve existing Phase 24A behavior
- `watchlist --repeat` uses the persisted monitor config and stays in the foreground until interrupted
- `daemon run` is the worker process that `systemd --user` executes
- `service install` only installs the unit file and runs `daemon-reload`; it does not start or enable the service
- `service` commands target `systemctl --user`
- `config set` updates exactly one field per invocation
- `event list` defaults to the newest 20 events
- `event list --type` only filters persisted business events, not system lifecycle logs

### Explicitly deferred

Phase 24B does not include:

- macOS `launchd`
- Windows services
- desktop/system popup notifications
- email, webhook, or push delivery
- reuse of the experimental `task` subsystem
- named monitor profiles or multiple concurrent services
- auto-trading or automatic sell execution after stop triggers
- event ack/delete/export
- persistence of daemon lifecycle or transport errors into the business event table
- HTTP health endpoints or metrics exporters

## Approaches Considered

### Option A: Extend `watchlist --once` into a foreground-only polling loop

Pros:

- smallest code diff
- no service management layer

Cons:

- does not solve background automation
- keeps the user tied to one terminal session
- leaves OS-managed restart and boot integration unsolved

### Option B: Implement only `systemd` integration with no reusable foreground loop

Pros:

- good fit for long-running automation on WSL2
- minimal user-facing operations once installed

Cons:

- duplicates monitor-loop concerns into a daemon-only path
- makes manual debugging harder
- removes a simple foreground mode that is useful during setup

### Option C: Add one shared monitor loop plus separate foreground and `systemd` entrypoints

Pros:

- one business loop shared by repeat mode and daemon mode
- easiest path to stable testing
- foreground mode remains useful for debugging
- `systemd --user` handles restart, enablement, and journald logging cleanly

Cons:

- introduces extra CLI surface
- requires a small service-installation layer and config file

## Recommendation

Choose **Option C**.

Add a shared monitor loop that evaluates watchlist quotes, price alerts, and stop rules. Expose it through:

- `monitor watchlist --repeat` for manual foreground use
- `monitor daemon run` for background execution
- `monitor service *` as a thin `systemd --user` management layer

Keep runtime behavior in a persisted monitor config file so the unit file stays stable across config changes.

## Architecture

### File boundaries

- `src/cli/mod.rs`
  - extend `MonitorCommands`
  - add `Config`, `Daemon`, `Service`, and `Event` subcommands

- `src/cli/handlers.rs`
  - keep current `monitor` command rendering
  - add repeat-loop, daemon, config, event-list, and service handlers

- `src/core/runtime.rs`
  - add monitor config path resolution
  - expose `QUANTIX_MONITOR_CONFIG_PATH`

- `src/monitor/mod.rs`
  - export the new automation pieces

- `src/monitor/config.rs`
  - monitor config model and JSON persistence

- `src/monitor/runner.rs`
  - shared monitor loop and single-iteration execution

- `src/monitor/systemd.rs`
  - user-unit rendering and `systemctl --user` wrappers

- `src/monitor/storage.rs`
  - extend the existing SQLite layer with event history and dedupe state

- existing `src/stop/*`
  - preserve current rule evaluation logic
  - only adapt integration points as needed

- tests
  - parser tests in existing CLI test modules
  - focused integration tests for config, runner, storage, and systemd unit rendering

### Shared loop model

The loop must be single-source-of-truth:

1. Load config
2. Resolve watchlist scope
3. Fetch best-effort quotes
4. Evaluate price alerts
5. Evaluate stop rules
6. Produce trigger candidates
7. Apply dedupe/edge logic
8. Persist new business events
9. Render/log results depending on run mode
10. Sleep until the next interval

Foreground repeat and daemon mode differ only in:

- output style
- signal handling / process lifecycle owner

They must not diverge in business evaluation.

## Configuration Model

### Storage path

Use a dedicated monitor config JSON file:

- default: `~/.quantix/monitor/config.json`
- env override: `QUANTIX_MONITOR_CONFIG_PATH`

### Config shape

```rust
pub struct MonitorConfig {
    pub interval_seconds: u64,
    pub watchlist_group: Option<String>,
    pub persist_events: bool,
    pub max_event_history: usize,
}
```

Defaults:

- `interval_seconds = 30`
- `watchlist_group = None`
- `persist_events = true`
- `max_event_history = 1000`

Rules:

- missing config file auto-creates defaults
- malformed config file is a hard startup error
- `watchlist --repeat` and `daemon run` both read the same config
- service install does not write runtime settings into the unit file

## `systemd --user` Integration

### Unit scope

Only support `systemctl --user`.

Install path:

- `~/.config/systemd/user/quantix-monitor.service`

### Unit rendering

The rendered unit should:

- use the absolute path from `std::env::current_exe()`
- execute `quantix monitor daemon run`
- set `Restart=on-failure`
- set a small `RestartSec`
- optionally embed `Environment=` lines for resolved non-default monitor/watchlist paths when needed for reliable user-local operation

This keeps the service deterministic in WSL2 even when the user's interactive shell has custom env overrides.

### Service semantics

- `install`
  - render/write the unit
  - run `systemctl --user daemon-reload`
- `uninstall`
  - require the service to be stopped first
  - remove the unit
  - run `daemon-reload`
- `start/stop/enable/disable`
  - pass through to `systemctl --user`
- `status`
  - show a readable summary derived from `systemctl --user status` or `show`

## Event History And Dedupe

### Event persistence boundary

Persist only business trigger events:

- price alert triggered
- fixed stop-loss triggered
- fixed take-profit triggered
- trailing stop triggered

Do not persist:

- daemon start/stop
- restart counts
- quote transport failures
- config parse errors
- systemd lifecycle messages

Those stay in `journald`.

### Storage strategy

Extend the existing monitor SQLite database instead of creating a new file.

Add:

- a business event table
- a trigger-state table for dedupe

Example shape:

```sql
CREATE TABLE IF NOT EXISTS monitor_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_time TEXT NOT NULL,
    event_type TEXT NOT NULL,
    code TEXT NOT NULL,
    price REAL,
    message TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_key TEXT NOT NULL,
    observed_at TEXT,
    run_mode TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS monitor_trigger_states (
    source_type TEXT NOT NULL,
    source_key TEXT NOT NULL,
    is_triggered INTEGER NOT NULL,
    last_transition_at TEXT NOT NULL,
    PRIMARY KEY (source_type, source_key)
);
```

`source_key` examples:

- `price_alert:12`
- `stop_rule:000001`

### Edge-trigger rule

Persist an event only on transition from `false -> true`.

That means:

- condition stays true across many loops:
  - one event only
- condition returns false:
  - trigger state clears
- condition becomes true again later:
  - persist a new event

This prevents log and SQLite spam while keeping re-trigger behavior correct after recovery.

### Event query output

`monitor event list` should show:

- event time
- event type
- code
- observed price
- message
- source key
- run mode

Filters:

- `--limit`
- `--code`
- `--type`

## Output And Logging

### Foreground `watchlist --repeat`

Print:

- watchlist snapshot rows for each loop
- newly triggered business events
- readable partial-failure notes

### Daemon mode

Write:

- summary progress
- new trigger messages
- non-fatal quote/DB warnings

to stdout/stderr so they flow into `journald`.

### Persistence and logs together

For new business triggers:

- write to `journald`
- persist to `monitor_events` when `persist_events = true`

For non-business operational failures:

- log only
- do not write to `monitor_events`

## Error Handling

Hard errors:

- malformed config file
- monitor DB open/schema failure
- unit install/uninstall command failure
- invalid CLI combinations

Soft errors:

- partial quote lookup failure
- one code missing from the quote result
- event persistence failure for one candidate row
- empty watchlist or empty watchlist group

Rules:

- soft errors must not abort the loop
- when only part of the quote set succeeds, still evaluate what can be evaluated safely
- if one event insert fails, continue with the rest and log the failure
- empty watchlists are valid and should render a readable no-data state

## Testing Strategy

1. Parser tests
   - `watchlist --repeat`
   - `config`
   - `daemon`
   - `service`
   - `event list`

2. Config tests
   - default-file creation
   - round-trip persistence
   - malformed JSON failure

3. Runner tests
   - empty watchlist does not fail
   - partial quote coverage does not stop the loop
   - triggered candidates are produced from current alert/stop rules

4. Storage tests
   - event history persists across reopen
   - dedupe state suppresses repeated writes while active
   - new event is written after a false-to-true transition
   - history trimming respects `max_event_history`

5. Service tests
   - unit rendering uses `current_exe`
   - install/uninstall command generation targets `systemctl --user`
   - install does not imply start or enable

6. Docs/hygiene tests
   - README and user manual reflect the new monitor automation boundary

## Non-Goals

Phase 24B is not:

- a desktop notification system
- a cross-platform service manager
- a task-scheduler replacement
- an auto-trading engine
- a monitoring web dashboard
- a transport-level observability framework
