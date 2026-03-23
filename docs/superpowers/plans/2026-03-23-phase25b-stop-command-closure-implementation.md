# Phase 25B Stop Command Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the current `stop` command family by adding percent-based thresholds, `stop update`, `stop status`, and `stop history`, while preserving the existing monitor/watchlist stop evaluation path.

**Architecture:** Keep `stop` as its own local subsystem. Extend `stop_rules` and add `stop_history`, keep mutation and audit semantics in `src/stop/*`, and let CLI/monitor code provide quote and paper-position context for percent-anchor resolution rather than pushing trade-store reads deep into the stop storage layer.

**Tech Stack:** Rust, clap, tokio, sqlx/sqlite, serde/serde_json, chrono, existing `src/stop/*`, existing `src/cli/*`, existing `src/monitor/*`, existing `src/trade/*`, GitNexus impact analysis, Graphiti MCP workflow, repo hygiene tests.

---

## Preflight

- Read the approved spec in [2026-03-23-phase25b-stop-command-closure-design.md](/opt/claude/quantix-rust/docs/superpowers/specs/2026-03-23-phase25b-stop-command-closure-design.md).
- Use `@superpowers/test-driven-development` for every behavior change. No production edits before a failing test.
- Use `@superpowers/verification-before-completion` before claiming the phase is done.
- Before editing any existing indexed symbol, run `gitnexus_impact({repo: "quantix-rust", target: "...", direction: "upstream"})`; if the result is HIGH/CRITICAL, review the blast radius before proceeding.
- Before every commit, run `gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})`.
- Graphiti is mandatory for design/review/debug/handoff memory. If ingest fails or hangs, keep an equivalent local summary and mark `Graphiti backfill required`.
- The repository already contains unrelated dirty files. Stage only files in the active task and never revert unrelated user changes.

## File Map

- `src/stop/models.rs`
  - Extend `StopRule` with percent-threshold and fallback-anchor fields.
  - Add status/history model types and any new enums needed for evaluated output.
- `src/stop/mod.rs`
  - Re-export new stop model types used by CLI/tests.
- `src/stop/service.rs`
  - Extend store trait, add `update`/`history`/`status` service logic, and keep evaluation rules centralized here.
- `src/stop/storage.rs`
  - Migrate `stop_rules`, add `stop_history`, and implement new store methods.
- `tests/stop_storage_test.rs`
  - New storage-level schema/migration/history tests.
- `tests/stop_service_test.rs`
  - Expand service-level validation, anchor, update, and trigger-history coverage.
- `src/cli/mod.rs`
  - Extend `StopCommands` with `update`, `status`, `history`, `--loss-pct`, `--profit-pct`, and clear flags.
- `src/cli/tests/stop.rs`
  - Add parser coverage for the expanded stop command surface.
- `src/cli/handlers.rs`
  - Wire new stop commands, resolve setup-time `reference_price`, build current status rows, and persist trigger history from monitor evaluation.
- `src/trade/models.rs`
  - Read-only dependency for `TradePosition.avg_cost` semantics used by stop percent-anchor resolution. Do not modify unless later chunks prove a helper is genuinely necessary.
- `README.md`
  - Replace the deferred Phase 25 boundary with the new command surface and semantics.
- `docs/USER_MANUAL.md`
  - Document `stop update`, `stop status`, `stop history`, percent thresholds, and anchor semantics.
- `tests/repo_hygiene_test.rs`
  - Lock the new docs wording and CLI examples.

## Implementation Assumption To Preserve

When a user creates or updates a percent-based stop rule:

1. if a best-effort quote is available, persist that quote as `reference_price`
2. otherwise, if a local paper position exists, persist `avg_cost` as `reference_price`
3. otherwise, reject the mutation with a clear error because no future fallback anchor can be stored

At evaluation time:

1. use current paper `avg_cost` first if a position exists
2. fall back to persisted `reference_price`
3. otherwise surface `anchor_missing`

This keeps setup-time semantics aligned with the approved design without introducing a user-selectable anchor mode in v1.

## Chunk 1: Stop Models And SQLite Foundation

### Task 1: Extend stop models and add schema support for percent thresholds and history

**Files:**
- Modify: `src/stop/models.rs`
- Modify: `src/stop/mod.rs`
- Modify: `src/stop/storage.rs`
- Create: `tests/stop_storage_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for stop model/storage symbols**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "StopRule", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "StopRuleStore", direction: "upstream"})
```

Expected:
- low/medium blast radius across stop service, CLI, monitor integration, and stop tests. If HIGH/CRITICAL, review direct callers before editing.

- [ ] **Step 2: Write failing storage tests for the new schema**

Add coverage that requires:
- legacy `stop_rules` rows still load after new columns are added
- `StopRule` round-trips:
  - `stop_loss_pct`
  - `take_profit_pct`
  - `reference_price`
- `stop_history` rows round-trip with:
  - `set`
  - `update`
  - `remove`
  - `trigger`

Suggested assertions:

```rust
assert_eq!(saved.stop_loss_pct, Some(5.0));
assert_eq!(saved.reference_price, Some(15.2));
assert_eq!(history[0].event_type, StopHistoryEventType::Set);
```

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
cargo test --test stop_storage_test -- --nocapture
```

Expected:
- FAIL because the new stop fields, history table, and store helpers do not exist yet.

- [ ] **Step 4: Implement model and schema changes**

Add to `StopRule`:
- `stop_loss_pct: Option<f64>`
- `take_profit_pct: Option<f64>`
- `reference_price: Option<f64>`

Add stop model types for:
- history event rows
- history event type enum
- optional trigger type enum or reuse adapter enum cleanly
- status/eval enums required by later chunks

Extend SQLite support:
- migrate `stop_rules` with nullable new columns using the existing `ALTER TABLE ... ADD COLUMN` compatibility pattern already used in `src/execution/runtime_store.rs`
- create `stop_history` with `CREATE TABLE IF NOT EXISTS`
- add indexes:
  - `CREATE INDEX IF NOT EXISTS idx_stop_history_code_created_at ON stop_history(code, created_at)`
  - `CREATE INDEX IF NOT EXISTS idx_stop_history_event_created_at ON stop_history(event_type, created_at)`
- add store helpers:
  - `get_rule(code)`
  - `append_history(entry)`
  - `list_history(filter)`

- [ ] **Step 5: Re-run focused tests to verify GREEN**

Run:
```bash
cargo test --test stop_storage_test -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected:
- only stop model/storage files and the new storage test are in scope.

Commit:
```bash
git add src/stop/models.rs src/stop/mod.rs src/stop/storage.rs tests/stop_storage_test.rs
git commit -m "feat: add stop percent threshold storage foundation"
```

## Chunk 2: Mutation Semantics, Patch Update, And Stop History

### Task 2: Add service-level `set/update/remove/history` behavior

**Files:**
- Modify: `src/stop/service.rs`
- Modify: `tests/stop_service_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for stop service entry points**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "StopService", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "run_stop_command", direction: "upstream"})
```

Expected:
- medium blast radius centered on CLI stop commands and monitor evaluation.

- [ ] **Step 2: Write failing service tests for new mutation semantics**

Add tests covering:
- `set_rule` accepts `loss_pct` and `profit_pct`
- `set_rule` rejects:
  - `loss + loss_pct`
  - `profit + profit_pct`
  - `trailing + loss_pct`
- `update_rule` only patches explicit fields
- clear flags remove exactly one threshold
- `update_rule` rejects resulting empty rule
- `remove_rule` appends a `remove` history entry with snapshot JSON
- `set_rule` and `update_rule` append `set` / `update` history entries

Suggested assertions:

```rust
assert_eq!(updated.stop_loss_pct, Some(5.0));
assert_eq!(updated.take_profit_price, original.take_profit_price);
assert!(err.to_string().contains("不能同时指定"));
```

- [ ] **Step 3: Run focused service tests to verify RED**

Run:
```bash
cargo test --test stop_service_test set_rule_ -- --nocapture
cargo test --test stop_service_test update_rule_ -- --nocapture
```

Expected:
- FAIL because percent thresholds, patch update, and history writes are not implemented yet.

- [ ] **Step 4: Implement service mutation and history logic**

Add:
- a patch input type for `stop update`
- service methods:
  - `set_rule(...)` with percent fields and `reference_price`
  - `update_rule(...)`
  - `history(...)`
- validation covering the approved conflict matrix
- history writes on `set`, `update`, and `remove`

Keep mutation semantics:
- `set` remains full overwrite
- `update` remains partial patch

- [ ] **Step 5: Re-run focused service tests to verify GREEN**

Run:
```bash
cargo test --test stop_service_test -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Commit:
```bash
git add src/stop/service.rs tests/stop_service_test.rs
git commit -m "feat: add stop update and history semantics"
```

## Chunk 3: Anchor-Aware Evaluation, Status Rows, And Monitor Trigger History

### Task 3: Add percent-anchor evaluation and evaluated stop status rows

**Files:**
- Modify: `src/stop/models.rs`
- Modify: `src/stop/service.rs`
- Modify: `src/cli/handlers.rs`
- Modify: `tests/stop_service_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for stop evaluation paths**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "evaluate_rules_for_snapshot", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "execute_stop_command_with_service", direction: "upstream"})
```

Expected:
- medium blast radius around monitor snapshot and stop command handling. If HIGH/CRITICAL, review monitor/CLI callers before editing.

- [ ] **Step 2: Write failing tests for anchor selection and status semantics**

Add service/handler coverage for:
- percent stop uses current paper `avg_cost` as anchor when a position exists
- falls back to stored `reference_price` when no position exists
- returns `anchor_missing` when a percent rule has neither current position nor reference price
- returns `quote_missing` when current quote is unavailable
- `trailing + profit_pct` remains valid
- trigger history writes:
  - `trigger_type`
  - `trigger_price`
  - `anchor_price`
  - `anchor_source`

Suggested assertions:

```rust
assert_eq!(row.anchor_source, StopAnchorSource::PositionCost);
assert_eq!(row.loss_threshold, Some(14.25));
assert_eq!(row.eval_state, StopEvalState::AnchorMissing);
```

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
cargo test --test stop_service_test anchor_ -- --nocapture
cargo test --lib cli::handlers::tests::test_execute_stop_ -- --nocapture
```

Expected:
- FAIL because anchor-aware status rows and trigger-history persistence do not exist yet.

- [ ] **Step 4: Implement evaluation and monitor integration**

Add evaluated output types:
- `StopStatusRow`
- `StopEvalState`
- `StopAnchorSource`

Implement:
- threshold derivation for `loss_pct` / `profit_pct`
- quote-missing and anchor-missing states
- in `src/cli/handlers.rs`, add a helper that builds a `code -> avg_cost` map from the current local paper-trade account state
  - this keeps trade-store reads at the CLI layer rather than pushing them into stop storage
- trigger-history writes inside the monitor stop evaluation loop

Do not:
- mutate trade state
- auto-execute sells

- [ ] **Step 5: Re-run focused tests to verify GREEN**

Run:
```bash
cargo test --test stop_service_test -- --nocapture
cargo test --lib cli::handlers::tests::test_execute_stop_ -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Commit:
```bash
git add src/stop/models.rs src/stop/service.rs src/cli/handlers.rs tests/stop_service_test.rs
git commit -m "feat: add stop anchor-aware status evaluation"
```

## Chunk 4: CLI Surface For `update`, `status`, `history`, And Percent Flags

### Task 4: Extend stop CLI parsing and command handlers

**Files:**
- Modify: `src/cli/mod.rs`
- Modify: `src/cli/tests/stop.rs`
- Modify: `src/cli/handlers.rs`

- [ ] **Step 1: Run GitNexus impact analysis for stop CLI symbols**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "StopCommands", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "run_stop_command", direction: "upstream"})
```

Expected:
- low/medium CLI routing risk centered on stop parser/handlers/tests.

- [ ] **Step 2: Write failing parser and handler tests**

Add parser coverage for:
- `stop set 000001 --loss-pct 5`
- `stop set 000001 --profit-pct 10`
- `stop update 000001 --profit-pct 12 --clear-profit`
- `stop status`
- `stop status --code 000001`
- `stop history --code 000001 --limit 10 --date 2026-03-23 --type trigger`

Add handler coverage for:
- percent-rule `set` resolves and persists `reference_price`
- `update` applies patch semantics
- `status` returns evaluated rows
- `history` returns change and trigger events

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
cargo test --lib cli::tests::stop:: -- --nocapture
cargo test --lib cli::handlers::tests::test_execute_stop_ -- --nocapture
```

Expected:
- FAIL because the new commands and flags do not exist yet.

- [ ] **Step 4: Implement CLI surface**

Extend `StopCommands` with:
- `Set { loss_pct, profit_pct }`
- `Update { ... clear flags ... }`
- `Status { code }`
- `History { code, limit, date, event_type }`

Update handler output types so:
- `list` remains raw-rule oriented
- `status` prints evaluated rows
- `history` prints audit rows

Use best-effort quote lookup to resolve `reference_price` during `set/update` when percent thresholds are present:
- first try quote
- then current `avg_cost`
- otherwise return a clear mutation error

- [ ] **Step 5: Re-run focused tests to verify GREEN**

Run:
```bash
cargo test --lib cli::tests::stop:: -- --nocapture
cargo test --lib cli::handlers::tests::test_execute_stop_ -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Commit:
```bash
git add src/cli/mod.rs src/cli/tests/stop.rs src/cli/handlers.rs
git commit -m "feat: add stop status history and percent CLI"
```

## Chunk 5: Docs, Hygiene, And Final Verification

### Task 5: Update user-facing docs and lock them with repo hygiene

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Write failing doc/hygiene expectations**

Add assertions that require README and USER_MANUAL to mention:
- `quantix stop update`
- `quantix stop status`
- `quantix stop history`
- `--loss-pct`
- `--profit-pct`
- percent-anchor semantics:
  - prefer paper `avg_cost`
  - fallback `reference_price`
- change + trigger history semantics

- [ ] **Step 2: Run focused doc tests to verify RED**

Run:
```bash
cargo test --test repo_hygiene_test stop_ -- --nocapture
```

Expected:
- FAIL because docs still describe these features as deferred.

- [ ] **Step 3: Update README and USER_MANUAL**

Replace the old deferred wording with the new command surface and semantics. Keep docs honest about remaining non-goals:
- no auto-sell
- no real broker
- no custom anchor mode

- [ ] **Step 4: Re-run focused doc tests to verify GREEN**

Run:
```bash
cargo test --test repo_hygiene_test stop_ -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Run full verification**

Run:
```bash
cargo test --test stop_storage_test -- --nocapture
cargo test --test stop_service_test -- --nocapture
cargo test --lib cli::tests::stop:: -- --nocapture
cargo test --lib cli::handlers::tests::test_execute_stop_ -- --nocapture
cargo test --test repo_hygiene_test -- --nocapture
```

Then run broader regression:

```bash
cargo test --test stop_service_test --test strategy_daemon_test --test execution_daemon_test --test repo_hygiene_test -- --nocapture
cargo test --lib cli::tests::strategy:: -- --nocapture
cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture
```

Expected:
- stop closure passes without regressing current strategy/execution handler paths that are outside the direct stop surface.

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected:
- affected files remain within stop, CLI, docs, and targeted monitor integration scope.

Commit:
```bash
git add src/stop/models.rs src/stop/mod.rs src/stop/service.rs src/stop/storage.rs tests/stop_storage_test.rs tests/stop_service_test.rs src/cli/mod.rs src/cli/tests/stop.rs src/cli/handlers.rs README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "feat: close stop command lifecycle"
```

## Final Memory

- [ ] **Step 1: Record Graphiti outcome**

Write a conclusion-oriented Graphiti memory for the design and implementation outcome. If ingest fails, preserve an equivalent local summary and mark:

```text
Graphiti backfill completed on 2026-03-24; memory is present in `quantix_rust_main`.
```

- [ ] **Step 2: Write a local phase completion summary if Graphiti is unavailable**

If Graphiti ingest fails or hangs, append a local completion note to this plan or create a short completion summary documenting:
- commands delivered
- schema changes
- integration points
- deferred non-goals
- fresh verification commands and outcomes

- [ ] **Step 3: Verify acceptance criteria before declaring the phase complete**

Confirm all acceptance criteria from the design spec are satisfied:
1. [ ] users can define percent-based stop thresholds
2. [ ] users can patch rules via `stop update`
3. [ ] `stop status` shows evaluated thresholds and anchor source
4. [ ] `stop history` shows rule-change and trigger audit entries
5. [ ] monitor stop evaluation remains compatible
6. [ ] docs and hygiene tests reflect the new command surface

## Local Completion Note

- 2026-03-24 implementation reached the planned Phase 25B stop command closure slice.
- Delivered command surface:
  - `quantix stop set --loss-pct/--profit-pct`
  - `quantix stop update`
  - `quantix stop status`
  - `quantix stop history`
- Extended `StopRule` with:
  - `stop_loss_pct`
  - `take_profit_pct`
  - `reference_price`
- Added `stop_history` storage in the shared monitor SQLite database, with rule-change and trigger audit rows.
- `StopService` now supports:
  - partial-patch updates
  - audit history reads
  - evaluated status rows
  - percent-anchor resolution preferring current paper `avg_cost` and falling back to `reference_price`
- `MonitorRunner` now reads the local paper trade store so percent stop triggers in runner/daemon paths can use current `avg_cost`.
- Fresh verification completed successfully:
  - `CARGO_TARGET_DIR=/tmp/quantix-target-phase25b cargo test --test stop_db_test --test stop_service_test --test monitor_runner_test --test repo_hygiene_test -- --nocapture`
  - `CARGO_TARGET_DIR=/tmp/quantix-target-phase25b cargo test --lib cli::tests::stop:: -- --nocapture`
  - `CARGO_TARGET_DIR=/tmp/quantix-target-phase25b cargo test --lib cli::handlers::tests::test_execute_stop_ -- --nocapture`
  - `CARGO_TARGET_DIR=/tmp/quantix-target-phase25b cargo test --lib cli::tests::strategy:: -- --nocapture`
  - `CARGO_TARGET_DIR=/tmp/quantix-target-phase25b cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture`
  - `CARGO_TARGET_DIR=/tmp/quantix-target-phase25b cargo test --test strategy_daemon_test --test execution_daemon_test -- --nocapture`
- `gitnexus_detect_changes({scope: "all"})` reported CRITICAL because the workspace still contains many unrelated user-side modifications. Focused diff for this slice remained within stop / monitor / CLI / docs / targeted tests.

Graphiti backfill completed on 2026-03-24; memory is present in `quantix_rust_main`.
