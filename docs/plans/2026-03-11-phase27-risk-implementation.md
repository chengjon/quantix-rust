# Phase 27A Risk Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add the smallest paper-trade risk loop on top of the Phase 26A green baseline: configure local risk rules, block invalid paper-trade buys, lock new buys after daily loss triggers, and inspect current risk status.

**Architecture:** Introduce a dedicated `risk` domain with JSON persistence and keep trade accounting unchanged. Wire risk into the existing CLI-first flow by adding a top-level `risk` command surface and having `src/cli/handlers/trade.rs` consult risk state before `buy` and refresh it after trade mutations.

**Tech Stack:** Rust, clap, chrono, rust_decimal, serde JSON, async-trait, existing CLI handler pattern, Markdown docs.

---

## P0 Boundary

Only implement:

- `quantix risk rule set --type position-limit --value 20%`
- `quantix risk rule set --type daily-loss-limit --value 50000`
- `quantix risk rule set --type daily-loss-limit --value 5%`
- `quantix risk rule list`
- `quantix risk rule enable --type position-limit`
- `quantix risk rule disable --type daily-loss-limit`
- `quantix risk status`
- `trade buy` pre-check against risk state
- `trade init/reset/sell` post-mutation risk sync
- JSON persistence at `QUANTIX_RISK_PATH`

Explicitly exclude:

- `risk pnl`
- `risk position`
- `risk log`
- `risk lock status`
- `risk lock release`
- `risk trigger-history`
- external push/audio/popup alerts
- real-account import/status
- live quote mark-to-market
- auto-liquidation / auto-reduction
- volatility or sector exposure rules

### Task 1: Add top-level `risk` CLI surface

**Files:**
- Modify: `src/cli/mod.rs`
- Modify: `src/cli/tests.rs`

**Step 1: Write the failing test**

Add parser tests for:

- `quantix risk rule set --type position-limit --value 20%`
- `quantix risk rule set --type daily-loss-limit --value 50000`
- `quantix risk rule list`
- `quantix risk rule enable --type position-limit`
- `quantix risk rule disable --type daily-loss-limit`
- `quantix risk status`

Also assert parser rejection for:

- missing `--value` on `rule set`
- missing `--type` on `rule set`

**Step 2: Run test to verify it fails**

Run: `cargo test cli::tests::parses_risk -- --nocapture`

Expected: FAIL because `Commands::Risk` and `RiskCommands` do not exist yet.

**Step 3: Write minimal implementation**

Add:

- `Commands::Risk(RiskCommands)`
- `RiskCommands::Rule(RiskRuleCommands)`
- `RiskCommands::Status`
- `RiskRuleCommands::Set`
- `RiskRuleCommands::List`
- `RiskRuleCommands::Enable`
- `RiskRuleCommands::Disable`

Keep `--value` as a string so `%`-suffix parsing stays in the domain layer rather than clap.

**Step 4: Run test to verify it passes**

Run: `cargo test cli::tests::parses_risk -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/cli/mod.rs src/cli/tests.rs
git commit -m "feat: add phase27 risk cli surface"
```

### Task 2: Add risk models, rule parsing, and service core

**Files:**
- Create: `src/risk/mod.rs`
- Create: `src/risk/models.rs`
- Create: `src/risk/service.rs`
- Modify: `src/lib.rs`
- Test: `tests/risk_service_test.rs`

**Step 1: Write the failing test**

Add service tests covering:

- `rule set` upserts position-limit percentage values
- `rule set` upserts daily-loss amount values
- `rule set` upserts daily-loss percentage values
- position-limit rejects amount syntax
- unknown or malformed value syntax is rejected
- `enable_rule` and `disable_rule` update an existing rule
- `status` initializes the current-day baseline when missing
- day rollover replaces baseline and clears daily lock
- daily-loss amount triggers buy lock
- daily-loss percentage triggers buy lock
- current lock blocks new buys
- position-limit rejects a projected buy that would exceed the cap
- sells remain allowed while buy lock is active

**Step 2: Run test to verify it fails**

Run: `cargo test --test risk_service_test -v`

Expected: FAIL because the `risk` module does not exist yet.

**Step 3: Write minimal implementation**

Create core types:

- `RiskState`
- `RiskRule`
- `RiskRuleType`
- `RuleValue`
- `DailyRiskBaseline`
- `BuyLockState`
- `RiskStatus`
- `PositionRiskRow`
- `RiskRuleSnapshot`

Create traits and service:

- `RiskStore`
- `RiskService<Store>`

Add methods:

- `set_rule(...)`
- `list_rules()`
- `enable_rule(...)`
- `disable_rule(...)`
- `status(...)`
- `check_buy(...)`
- `sync_after_trade_snapshot(...)`
- `sync_after_trade_reset(...)`

Keep trade bookkeeping input explicit: the risk service should consume account/cash-snapshot data, not own trade accounting logic.

**Step 4: Run test to verify it passes**

Run: `cargo test --test risk_service_test -v`

Expected: PASS

**Step 5: Commit**

```bash
git add src/risk/mod.rs src/risk/models.rs src/risk/service.rs src/lib.rs tests/risk_service_test.rs
git commit -m "feat: add phase27 risk service core"
```

### Task 3: Add risk runtime path support and JSON storage

**Files:**
- Modify: `src/core/runtime.rs`
- Create: `src/risk/storage.rs`
- Test: `tests/risk_storage_test.rs`
- Test: `src/core/runtime.rs`

**Step 1: Write the failing test**

Add runtime tests for:

- `QUANTIX_RISK_PATH` override
- default fallback to `~/.quantix/risk/risk_state.json`

Add storage tests for:

- loading `None` when the file does not exist
- save/load round-trip for configured rules and lock state
- persistence survives reopen
- save uses temp-file replace semantics like trade storage

**Step 2: Run test to verify it fails**

Run: `cargo test risk_path -- --nocapture`

Run: `cargo test --test risk_storage_test -v`

Expected: FAIL because risk runtime path and storage do not exist yet.

**Step 3: Write minimal implementation**

In `src/core/runtime.rs` add:

- `RISK_PATH_ENV`
- `CliRuntime.risk_path`
- default resolver for `~/.quantix/risk/risk_state.json`

In `src/risk/storage.rs` implement:

- `JsonRiskStore`
- `load_state()`
- `save_state()`
- temp-file write plus atomic replace

Follow the same parent-directory creation and cleanup pattern used by trade storage.

**Step 4: Run test to verify it passes**

Run: `cargo test risk_path -- --nocapture`

Run: `cargo test --test risk_storage_test -v`

Expected: PASS

**Step 5: Commit**

```bash
git add src/core/runtime.rs src/risk/storage.rs tests/risk_storage_test.rs
git commit -m "feat: add phase27 risk json storage"
```

### Task 4: Add `risk` CLI handlers

**Files:**
- Modify: `src/cli/handlers.rs`
- Create: `src/cli/handlers/risk.rs`
- Modify: `src/cli/handlers/tests/mod.rs`
- Modify: `src/cli/handlers/tests/support.rs`
- Create: `src/cli/handlers/tests/risk.rs`

**Step 1: Write the failing test**

Add handler tests covering:

- `risk rule set` stores a position-limit rule
- `risk rule set` stores a daily-loss rule
- `risk rule list` returns configured rules
- `risk rule enable` toggles an existing rule on
- `risk rule disable` toggles an existing rule off
- `risk status` returns a computed snapshot for an initialized trade account
- `risk status` errors when paper-trade account is not initialized

Use a fake risk store plus explicit trade snapshot inputs. Do not capture stdout; test a helper such as `execute_risk_command_with_service(...)`.

**Step 2: Run test to verify it fails**

Run: `cargo test cli::handlers::tests::risk -- --nocapture`

Expected: FAIL because `run_risk_command` does not exist.

**Step 3: Write minimal implementation**

Add:

- `run_risk_command`
- `execute_risk_command_with_service(...)`
- print helpers for rules and status
- `create_risk_store()`

Keep handler output parallel to the existing trade/stop handler style.

**Step 4: Run test to verify it passes**

Run: `cargo test cli::handlers::tests::risk -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/cli/handlers.rs src/cli/handlers/risk.rs src/cli/handlers/tests/mod.rs src/cli/handlers/tests/support.rs src/cli/handlers/tests/risk.rs
git commit -m "feat: add phase27 risk command handlers"
```

### Task 5: Wire trade handlers to consult risk before buy

**Files:**
- Modify: `src/cli/handlers/trade.rs`
- Modify: `src/cli/handlers/tests/trade.rs`
- Optionally Modify: `src/cli/handlers/tests/support.rs`

**Step 1: Write the failing test**

Add handler-level trade tests covering:

- `trade buy` is rejected when a daily-loss lock is active
- `trade buy` is rejected when projected position ratio exceeds the configured cap
- `trade sell` still succeeds while buy lock is active
- `trade init` or `trade reset` clears risk lock while preserving configured rules
- `trade sell` refreshes risk state after execution

Keep the existing pure trade-service tests unchanged; these new tests should verify the CLI integration boundary only.

**Step 2: Run test to verify it fails**

Run: `cargo test cli::handlers::tests::trade -- --nocapture`

Expected: FAIL because the trade handler does not read risk state yet.

**Step 3: Write minimal implementation**

In `src/cli/handlers/trade.rs`:

- create a risk store alongside the trade store
- on `trade init/reset`, sync risk derived state from the reset account snapshot
- on `trade buy`, call risk pre-check before `service.buy(...)`
- on successful `trade buy`, refresh risk state from the updated trade snapshot
- on successful `trade sell`, refresh risk state from the updated trade snapshot

Do not move risk logic into `TradeService` for Phase 27A. Keep the trade domain focused on accounting and keep risk enforcement at the current CLI integration boundary.

**Step 4: Run test to verify it passes**

Run: `cargo test cli::handlers::tests::trade -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/cli/handlers/trade.rs src/cli/handlers/tests/trade.rs src/cli/handlers/tests/support.rs
git commit -m "feat: enforce phase27 risk checks in trade cli"
```

### Task 6: Update user-facing docs and repo hygiene expectations

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Optionally Modify: `tests/repo_hygiene_test.rs`

**Step 1: Write the failing test**

If repo hygiene tests already assert command coverage, extend them to mention:

- `quantix risk rule set`
- `quantix risk status`
- paper-trade-only scope

If there is no stable assertion point for this yet, skip the automated doc assertion and document manually.

**Step 2: Run test to verify it fails**

Run: `cargo test --test repo_hygiene_test -v`

Expected: FAIL only if you added a new doc assertion.

**Step 3: Write minimal implementation**

Document:

- `risk` command quick start
- supported rule types
- daily-loss bookkeeping limitation
- buy-lock versus sell-allow semantics
- explicit non-goals

Keep the docs aligned with Phase 27A only.

**Step 4: Run test to verify it passes**

Run: `cargo test --test repo_hygiene_test -v`

Expected: PASS

**Step 5: Commit**

```bash
git add README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "docs: add phase27 risk command usage"
```

## Final Verification

### Step 1: Run focused regressions

Run:

- `cargo test cli::tests::parses_risk -- --nocapture`
- `cargo test cli::handlers::tests::risk -- --nocapture`
- `cargo test cli::handlers::tests::trade -- --nocapture`
- `cargo test --test risk_service_test -v`
- `cargo test --test risk_storage_test -v`

Expected: PASS

### Step 2: Run full suite

Run:

- `cargo test`

Expected: PASS with zero failures. Pre-existing warnings may remain unless directly caused by this phase.

### Step 3: Check file-length guardrail

Run:

- `find src tests -type f -name '*.rs' -print0 | xargs -0 wc -l | sort -nr | head -20`

Expected: no Rust file exceeds `700` lines.
