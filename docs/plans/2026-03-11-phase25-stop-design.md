# Phase 25A Stop Design

**Date:** 2026-03-11
**Status:** Approved in-session
**Depends On:** Phase 24A monitor baseline (`phase24-monitor`)

---

## Goal

Build the smallest useful stop-management loop:

- users can persist stop rules for watchlist codes
- `quantix monitor watchlist --once` evaluates those rules with current quotes
- triggered rules are shown in the terminal

This phase does not attempt to become an order-management system.

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. Set a fixed stop-loss for a code
2. Set a fixed take-profit for a code
3. Set a simple trailing stop for a code
4. List current stop rules
5. Remove a stop rule
6. See triggered stop/take-profit events during one-shot monitor scans

Anything beyond those jobs is out of scope for Phase 25A.

### Exact CLI boundary

Only implement:

```bash
quantix stop set <CODE> --loss <PRICE>
quantix stop set <CODE> --profit <PRICE>
quantix stop set <CODE> --loss <PRICE> --profit <PRICE>
quantix stop set <CODE> --trailing <PCT>
quantix stop set <CODE> --trailing <PCT> --profit <PRICE>
quantix stop list
quantix stop remove <CODE>
```

Rules:

- `--loss` and `--trailing` are mutually exclusive
- at least one of `--loss`, `--profit`, `--trailing` is required
- `stop set` overwrites the full active rule set for that code
- `stop set` is only valid for codes already present in the local watchlist

Explicitly defer:

- `stop show`
- `stop update`
- `stop history`
- `stop status`
- `--loss-pct`
- `--profit-pct`
- `--trailing-base`
- ATR / moving-average stops
- automatic orders
- system notifications / multi-channel alerts
- non-watchlist stop management

## Approaches Considered

### Option A: Standalone `stop status` / `stop check` command

Pros:

- `stop` domain stays self-contained

Cons:

- duplicates Phase 24 monitor quote-loading path
- creates another top-level command surface to maintain
- encourages feature growth into polling and daemon behavior

### Option B: `stop` only manages rules, monitor performs evaluation

Pros:

- reuses the existing quote-fetch and terminal output path
- smallest additional scope
- aligns with the current bottom-up requirement

Cons:

- stop triggering is tied to `monitor watchlist --once`

### Option C: Broader order-style stop engine

Pros:

- could align later with Phase 26 paper trading

Cons:

- too early
- introduces execution semantics we do not support yet

## Recommendation

Choose **Option B**.

Phase 25A should add rule CRUD plus monitor-time evaluation only. That gives a real usable workflow without inventing a second monitoring system.

## Architecture

### Domain split

- `src/stop/*` owns stop rule models, evaluation logic, and persistence
- `src/cli/mod.rs` owns CLI parsing
- `src/cli/handlers.rs` owns stop command execution and monitor integration
- `src/monitor/*` remains focused on watchlist snapshots and price alerts

### Storage choice

Reuse the existing monitor SQLite database pointed to by `QUANTIX_MONITOR_DB_PATH`.

Reason:

- one runtime path already exists
- stop rules are operationally part of the one-shot monitor loop
- avoids adding another env var and another SQLite file

### Minimal data model

Use one active rule row per code.

Suggested table:

```sql
CREATE TABLE IF NOT EXISTS stop_rules (
    code TEXT PRIMARY KEY,
    stop_loss_price REAL,
    take_profit_price REAL,
    trailing_pct REAL,
    highest_price REAL,
    last_triggered_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

Notes:

- no separate history table in Phase 25A
- no status column in Phase 25A
- `highest_price` is only meaningful when `trailing_pct` is set
- removing a rule deletes the row

## Evaluation Model

### Fixed loss

Trigger when:

- `current_price <= stop_loss_price`

### Fixed profit

Trigger when:

- `current_price >= take_profit_price`

### Trailing stop

State:

- `highest_price = max(previous_highest_price, current_price)`

Trigger threshold:

- `highest_price * (1 - trailing_pct / 100.0)`

Trigger when:

- `current_price <= trailing_stop_price`

### Trigger lifecycle

Phase 25A does **not** auto-remove or auto-disable triggered rules.

Reason:

- there is no order execution
- there is no acknowledgement flow
- repeated terminal reminders are preferable to silently disabling protection

The only persisted trigger state is `last_triggered_at`.

## Watchlist Constraint

`stop set <CODE> ...` must reject codes that are not already in the local watchlist.

Reason:

- the only evaluation path is `monitor watchlist --once`
- allowing arbitrary codes would create silently inert rules

This constraint can be revisited once Phase 26 introduces a real position model.

## Terminal Output

`quantix stop list` should print a compact table:

- code
- stop-loss
- take-profit
- trailing %
- highest price
- last triggered

`quantix monitor watchlist --once` should keep its existing quote section, then print a new stop section if needed:

```text
== 止盈止损 ==
000001 当前价 14.20 触发 stop-loss 14.50
600519 当前价 1820.00 触发 trailing-stop 5.00% (highest 1920.00)
```

## Error Handling

- invalid flag combinations return CLI-facing validation errors
- setting a stop for a non-watchlist code returns a user-facing error
- missing quote data does not panic and does not trigger rules
- trailing rules without `highest_price` initialize from the first observed quote

## Testing Strategy

Required tests:

- CLI parser tests for `stop set/list/remove`
- stop service tests for:
  - fixed loss trigger
  - fixed profit trigger
  - trailing highest-price update
  - trailing trigger
  - set/list/remove store delegation
- handler tests for:
  - watchlist membership validation
  - stop list output helper behavior
  - monitor integration producing triggered stop events
- SQLite tests for:
  - schema creation
  - upsert rule
  - remove rule
  - highest price persistence
  - `last_triggered_at` updates

## Explicit Non-Goals

Phase 25A is not:

- a broker integration
- a daemon
- a scheduler
- a notification hub
- a full position-risk engine

It is only a rule-management extension on top of the existing one-shot monitor loop.
