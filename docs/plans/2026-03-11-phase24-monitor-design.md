# Phase 24 Monitor Design

**Date:** 2026-03-11
**Status:** Proposed and approved in-session
**Scope:** Bottom-up correction of the original Phase 24 plan

---

## Goal

Build the smallest user-facing monitoring loop that is actually useful:

- inspect watchlist quotes on demand
- persist simple price alerts
- trigger terminal alerts from current quotes

This intentionally does not implement the full original Phase 24 document.

## Why The Original Scope Is Too Large

The original `PHASE24_MONITOR_DESIGN.md` mixes several different concerns:

- watchlist quote inspection
- alert persistence
- long-running polling
- board/concept monitoring
- monitor lifecycle commands
- level-2 style market detail

That is top-down scope expansion. It makes the CLI surface look complete before the minimum working loop is proven.

## Bottom-Up P0 Boundary

Phase 24 is split into two batches.

### Phase 24A

Deliver only the minimum closed loop:

- `quantix monitor watchlist --once`
- `quantix monitor alert add <code> --above <price>`
- `quantix monitor alert add <code> --below <price>`
- `quantix monitor alert list`
- `quantix monitor alert remove <id>`
- SQLite persistence for alerts
- terminal notification when an alert is matched during quote checks

### Phase 24B

Only after 24A is stable:

- `quantix monitor watchlist --refresh <secs>`
- repeated polling with alert checks
- `--once` / `--repeat` alert behavior
- optional system notification via `notify-send`

### Explicitly Deferred

- `monitor stocks`
- `monitor sector`
- `monitor concept`
- `monitor start`
- `monitor stop`
- `monitor status`
- order book / intraday / tick detail
- external push integrations

## Command Design

### Watchlist Quotes

```bash
quantix monitor watchlist --once
```

Behavior:

- load watchlist from the existing watchlist store
- fetch current quotes for watchlist codes
- print code, name, latest price, change percent
- check stored alerts against returned quotes
- print matched alerts in highlighted terminal output

### Alert Management

```bash
quantix monitor alert add 000001 --above 16.0
quantix monitor alert add 000001 --below 15.0
quantix monitor alert list
quantix monitor alert remove 12
```

Rules:

- exactly one of `--above` or `--below` in Phase 24A
- no combined above+below creation in the first batch
- no `--repeat` / `--once` flags in 24A
- alerts remain active until manually removed

## Architecture

Create a new user-facing `monitor` domain instead of reusing `src/monitoring/*`.

`src/monitoring/*` already exists, but it is internal strategy/position/performance monitoring infrastructure. Reusing it for Phase 24 CLI would blur the boundary and make later stop/risk phases harder to reason about.

Use this structure:

- `src/monitor/mod.rs`
- `src/monitor/models.rs`
- `src/monitor/storage.rs`
- `src/monitor/service.rs`

Integrations:

- reuse watchlist loading from the Phase 21 watchlist stack
- reuse quote lookup behavior from the current TDX-backed watchlist resolver path
- keep SQLite access limited to monitor alert persistence
- keep CLI validation and printing thin in `src/cli/handlers.rs`

## Data Model

Use a single SQLite table for Phase 24A:

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

Notes:

- `alert_type` is `above` or `below`
- `is_active` is kept even though removal is the only lifecycle operation in 24A
- `last_triggered_at` is included now to avoid schema churn in 24B

## Runtime Configuration

Add a runtime path for the monitor SQLite database, similar to watchlist path handling.

Recommended env var:

- `QUANTIX_MONITOR_DB_PATH`

Default path:

- `~/.quantix/monitor/alerts.db`

## Quote Source Strategy

Use the existing TDX quote path already exercised by watchlist price display.

That gives Phase 24A:

- the same market inference behavior
- the same failure mode when quote data is unavailable
- no new external dependency surface

If quote lookup fails, `monitor watchlist --once` should degrade gracefully:

- print available watchlist rows
- surface a readable error for quote resolution
- do not crash

## Alert Trigger Logic

Match rules:

- `above`: trigger when `current_price >= target_price`
- `below`: trigger when `current_price <= target_price`

Phase 24A behavior after a match:

- print a strong terminal alert line
- persist `last_triggered_at`
- keep the alert active

This is intentionally simpler than single-shot vs repeating semantics. That choice is moved to 24B.

## Testing Strategy

### Parser Tests

Add CLI tests for:

- `monitor watchlist --once`
- `monitor alert add 000001 --above 16.0`
- `monitor alert add 000001 --below 15.0`
- `monitor alert list`
- `monitor alert remove 12`
- invalid combinations for `alert add`

### Storage Tests

Add focused tests for:

- SQLite schema creation
- add/list/remove round trip
- persisted timestamps and ids

### Service Tests

Add fake-reader tests for:

- watchlist rows resolve into monitor rows
- alert matching on above
- alert matching on below
- missing quotes do not panic

### Handler Tests

Add handler tests for:

- `monitor watchlist --once`
- `monitor alert add`
- `monitor alert list`
- `monitor alert remove`
- invalid alert command arguments

## File Boundary

Expected first-batch file touch set:

- `src/cli/mod.rs`
- `src/cli/handlers.rs`
- `src/core/runtime.rs`
- `src/lib.rs`
- `src/monitor/mod.rs`
- `src/monitor/models.rs`
- `src/monitor/storage.rs`
- `src/monitor/service.rs`
- `tests/monitor_service_test.rs`

Additional targeted tests may live beside existing CLI/runtime unit tests.

## Non-Goals

Phase 24A is not a daemon, not a stream processor, and not a market dashboard.

The purpose is narrower:

- prove the CLI commands
- prove alert persistence
- prove quote-to-alert matching

Only after that should polling and richer notification behavior be added.
