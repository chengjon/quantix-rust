# Phase 25A Stop Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add minimal stop rule management plus monitor-time stop evaluation on top of the Phase 24A green baseline.

**Architecture:** Introduce a dedicated `stop` domain for rule storage and evaluation, but keep quote loading inside the existing `monitor watchlist --once` flow. Reuse the monitor SQLite database and local watchlist storage instead of inventing a second runtime path or second polling command.

**Tech Stack:** Rust, clap, existing CLI handler pattern, chrono, sqlx SQLite, existing watchlist storage, existing monitor command flow, Markdown docs.

---

## P0 Boundary

Only implement:

- `quantix stop set <CODE> --loss <PRICE>`
- `quantix stop set <CODE> --profit <PRICE>`
- `quantix stop set <CODE> --loss <PRICE> --profit <PRICE>`
- `quantix stop set <CODE> --trailing <PCT>`
- `quantix stop set <CODE> --trailing <PCT> --profit <PRICE>`
- `quantix stop list`
- `quantix stop remove <CODE>`
- monitor-time evaluation during `quantix monitor watchlist --once`
- SQLite persistence inside the existing monitor DB

Explicitly exclude:

- `stop show`
- `stop update`
- `stop history`
- `stop status`
- `--loss-pct`
- `--profit-pct`
- `--trailing-base`
- ATR / moving-average stops
- automatic orders
- system notifications

### Task 1: Add top-level `stop` CLI surface

**Files:**
- Modify: `src/cli/mod.rs`
- Test: `src/cli/mod.rs`

**Step 1: Write the failing test**

Add parser tests for:

- `quantix stop set 000001 --loss 14.5`
- `quantix stop set 000001 --profit 18.0`
- `quantix stop set 000001 --loss 14.5 --profit 18.0`
- `quantix stop set 000001 --trailing 5`
- `quantix stop list`
- `quantix stop remove 000001`

Also assert rejected combinations:

- missing all of `--loss`, `--profit`, `--trailing`
- both `--loss` and `--trailing`

**Step 2: Run test to verify it fails**

Run: `cargo test cli::tests::parses_stop -- --nocapture`

Expected: FAIL because `Commands::Stop` does not exist yet.

**Step 3: Write minimal implementation**

Add:

- `Commands::Stop(StopCommands)`
- `StopCommands::Set`
- `StopCommands::List`
- `StopCommands::Remove`

Argument rules:

- `set` requires at least one of `loss`, `profit`, `trailing`
- `loss` and `trailing` are mutually exclusive
- do not add `show`, `update`, `history`, or `status`

**Step 4: Run test to verify it passes**

Run: `cargo test cli::tests::parses_stop -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/cli/mod.rs
git commit -m "feat: add phase25 stop cli surface"
```

### Task 2: Add stop domain models and evaluation service

**Files:**
- Create: `src/stop/mod.rs`
- Create: `src/stop/models.rs`
- Create: `src/stop/service.rs`
- Modify: `src/lib.rs`
- Test: `tests/stop_service_test.rs`

**Step 1: Write the failing test**

Add service tests using fake collaborators covering:

- setting a fixed loss rule
- setting a fixed profit rule
- setting a trailing rule
- listing stored rules
- removing a rule
- fixed loss trigger detection
- fixed profit trigger detection
- trailing stop updates `highest_price`
- trailing stop triggers after drawdown from updated high
- missing quote data produces no trigger

**Step 2: Run test to verify it fails**

Run: `cargo test --test stop_service_test -v`

Expected: FAIL because the `stop` module does not exist yet.

**Step 3: Write minimal implementation**

Create core types:

- `StopRule`
- `StopTriggerKind` (`Loss`, `Profit`, `TrailingLoss`)
- `TriggeredStop`
- `StopEvaluationResult`

Create traits:

- `StopRuleStore`

Create `StopService<RS>` methods:

- `set_rule(...)`
- `list_rules()`
- `remove_rule(code)`
- `evaluate_rule(rule, current_price, observed_at)`
- `evaluate_rules(rules, quote_rows, observed_at)`

Design rules:

- one active rule per code
- `set_rule` overwrites the full current rule for that code
- trailing stop initializes `highest_price` from the first observed quote
- service owns evaluation logic

**Step 4: Run test to verify it passes**

Run: `cargo test --test stop_service_test -v`

Expected: PASS

**Step 5: Commit**

```bash
git add src/stop/mod.rs src/stop/models.rs src/stop/service.rs src/lib.rs tests/stop_service_test.rs
git commit -m "feat: add phase25 stop service core"
```

### Task 3: Add SQLite stop rule storage in the monitor DB

**Files:**
- Create: `src/stop/storage.rs`
- Test: `src/stop/storage.rs`

**Step 1: Write the failing test**

Add focused tests for:

- schema creation
- upsert by code
- list returns active rules
- remove deletes by code
- trailing `highest_price` persists across reopen
- `last_triggered_at` update persists

**Step 2: Run test to verify it fails**

Run: `cargo test stop_db -- --nocapture`

Expected: FAIL because stop storage does not exist yet.

**Step 3: Write minimal implementation**

Implement:

- `SqliteStopRuleStore`
- schema creation helper
- `upsert_rule`
- `list_rules`
- `remove_rule`
- `update_evaluation_state`

Use the existing monitor DB path from `CliRuntime::load().monitor_db_path`.

Schema:

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

Do not add history tables or status fields in this task.

**Step 4: Run test to verify it passes**

Run: `cargo test stop_db -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/stop/storage.rs
git commit -m "feat: add phase25 stop sqlite storage"
```

### Task 4: Wire stop commands into CLI handlers

**Files:**
- Modify: `src/cli/handlers.rs`
- Test: `src/cli/handlers.rs`

**Step 1: Write the failing test**

Add handler-level tests with fake stop collaborators covering:

- `stop set --loss` succeeds
- `stop set --profit` succeeds
- `stop set --trailing` succeeds
- invalid `set` combinations return user-facing errors
- `stop set` rejects codes not found in watchlist
- `stop list` returns persisted rules
- `stop remove` succeeds

Prefer testing a helper such as `execute_stop_command_with_service(...)` instead of stdout capture.

**Step 2: Run test to verify it fails**

Run: `cargo test cli::handlers::tests::test_execute_stop -- --nocapture`

Expected: FAIL because `run_stop_command` does not exist.

**Step 3: Write minimal implementation**

Add:

- `run_stop_command`
- stop request builders / validators
- stop print helpers
- watchlist membership validation helper

Reuse existing behavior where sensible:

- read watchlist from current local watchlist storage
- reuse monitor DB path through the stop SQLite store

**Step 4: Run test to verify it passes**

Run: `cargo test cli::handlers::tests::test_execute_stop -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/cli/handlers.rs src/cli/mod.rs
git commit -m "feat: wire phase25 stop commands into cli"
```

### Task 5: Integrate stop evaluation into `monitor watchlist --once`

**Files:**
- Modify: `src/cli/handlers.rs`
- Test: `src/cli/handlers.rs`

**Step 1: Write the failing test**

Add monitor integration tests covering:

- fixed loss rules trigger from watchlist snapshot prices
- fixed profit rules trigger from watchlist snapshot prices
- trailing rules update `highest_price`
- trailing rules trigger after drawdown
- missing prices do not trigger

Prefer extending monitor execution helpers rather than capturing stdout.

**Step 2: Run test to verify it fails**

Run: `cargo test cli::handlers::tests::test_execute_monitor_stop -- --nocapture`

Expected: FAIL because monitor does not evaluate stop rules yet.

**Step 3: Write minimal implementation**

Add:

- loading active stop rules during `monitor watchlist --once`
- stop evaluation using the existing snapshot rows
- persistence of updated `highest_price` and `last_triggered_at`
- a terminal-friendly stop section printed after quotes and price alerts

Do not add new monitor subcommands in this task.

**Step 4: Run test to verify it passes**

Run: `cargo test cli::handlers::tests::test_execute_monitor_stop -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/cli/handlers.rs
git commit -m "feat: add phase25 stop monitor integration"
```

### Task 6: Document Phase 25A and extend repo hygiene coverage

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

**Step 1: Write the failing test**

Extend repo hygiene coverage so docs must mention:

- `quantix stop set`
- `quantix stop list`
- `quantix stop remove`
- monitor-time stop evaluation
- deferred `status/history/update` and percentage-based stop features

**Step 2: Run test to verify it fails**

Run: `cargo test --test repo_hygiene_test -- --nocapture`

Expected: FAIL because docs do not mention the new stop commands yet.

**Step 3: Write minimal implementation**

Update docs to describe only Phase 25A:

- rule CRUD
- watchlist-only constraint
- monitor-time trigger behavior
- reused monitor DB path
- explicit deferred items

Do not document unimplemented `stop status`, `stop history`, or multi-channel notifications.

**Step 4: Run test to verify it passes**

Run: `cargo test --test repo_hygiene_test -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "docs: add phase25 stop command usage"
```

### Task 7: Final regression verification

**Files:**
- No code changes expected

**Step 1: Run focused stop verification**

Run: `cargo test stop -- --nocapture`

Expected: PASS

**Step 2: Run monitor regression because monitor behavior changed**

Run: `cargo test monitor -- --nocapture`

Expected: PASS

**Step 3: Run full regression**

Run: `cargo test --all-targets`

Expected: PASS

**Step 4: Check worktree status**

Run: `git status --short`

Expected: only intended files changed or a clean tree if everything is committed.

**Step 5: If verification is green, stop and report**

Report:

- implemented commands
- monitor integration behavior
- test evidence
- any deferred items still intentionally excluded
