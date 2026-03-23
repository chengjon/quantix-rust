# Execution Automation Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a minimal `execution daemon`, introduce `execution_request.in_progress`, support `manual|always` auto-approval, and close the automation gap between durable requests and automated execution without adding a real live broker.

**Architecture:** Keep `strategy daemon` and `execution daemon` as separate roles. Extend the request lifecycle with a durable claim step, add an execution config store parallel to the existing strategy/monitor patterns, and make daemon consumption reuse the same internal request-execution path as manual `strategy request execute`.

**Tech Stack:** Rust, tokio, clap, sqlx/sqlite, serde/serde_json, chrono, existing `ExecutionKernel`, existing `StrategyRuntimeStore`, existing paper/mock-live adapters, existing `RiskService`, existing strategy and monitor config/systemd patterns, GitNexus impact analysis, Graphiti MCP workflow, repo hygiene tests.

---

## Preflight

- Read the approved spec in [2026-03-23-execution-automation-closure-design.md](/opt/claude/quantix-rust/docs/superpowers/specs/2026-03-23-execution-automation-closure-design.md).
- Use `@superpowers/test-driven-development` for every behavior change. No production edits before a failing test.
- Use `@superpowers/verification-before-completion` before claiming the phase is done.
- Before editing any existing indexed symbol, run `gitnexus_impact({repo: "quantix-rust", target: "...", direction: "upstream"})`; if the result is HIGH/CRITICAL, review the blast radius before proceeding.
- Before every commit, run `gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})`.
- Graphiti is mandatory for design/review/debug/handoff memory. If ingest fails, keep an equivalent local summary and mark `Graphiti backfill required`.
- The repository already contains unrelated dirty files. Stage only files in the active task and never revert unrelated user changes.

## File Map

- `src/core/runtime.rs`
  - Add execution config and optional execution service-config runtime paths.
- `src/execution/mod.rs`
  - Export new execution automation modules.
- `src/execution/config.rs`
  - New JSON config model/store for execution daemon runtime settings and auto-approval mode.
- `src/execution/daemon.rs`
  - New daemon runner plus shared request-consumption helper reused by CLI and daemon.
- `src/execution/models.rs`
  - Add `ExecutionRequestStatus::InProgress` and any execution-config enums if kept in models.
- `src/execution/runtime_store.rs`
  - Add pending-request discovery, conditional claim helper, and terminal update semantics from `in_progress`.
  - Keep supersede logic canceling only `pending`.
- `src/strategy/daemon.rs`
  - Add auto-approval behavior in `manual|always` modes without merging in execution concerns.
- `src/cli/mod.rs`
  - Add top-level `execution` command tree.
- `src/cli/handlers.rs`
  - Add execution config/daemon handlers and wire them to the new execution daemon helper.
- `src/cli/tests/strategy.rs`
  - Update request status parsing expectations if `in_progress` is exposed.
- `src/cli/tests/mod.rs`
  - Register execution parser tests if split out.
- `src/cli/tests/execution.rs`
  - New parser coverage for execution config/daemon commands.
- `tests/execution_runtime_store_test.rs`
  - Extend request status transition and supersede behavior coverage.
- `tests/strategy_daemon_test.rs`
  - Add auto-approval coverage (`manual` vs `always`).
- `tests/execution_daemon_test.rs`
  - New integration-style daemon tests for request claiming and consumption.
- `README.md`
  - Document execution daemon and automation boundary.
- `docs/USER_MANUAL.md`
  - Document execution config, daemon usage, and request status semantics.
- `tests/repo_hygiene_test.rs`
  - Lock the new docs wording and CLI examples.

## Chunk 1: Runtime Paths And Execution Config Store

### Task 1: Add execution runtime paths and JSON config store

**Files:**
- Modify: `src/core/runtime.rs`
- Create: `src/execution/config.rs`
- Modify: `src/execution/mod.rs`
- Test: `src/core/runtime.rs`
- Test: `tests/execution_daemon_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for runtime path consumers**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "CliRuntime", direction: "upstream"})
```

Expected:
- low/medium infrastructure risk centered on CLI path consumers. If HIGH/CRITICAL, review the callers before changing the struct.

- [ ] **Step 2: Write the failing runtime/config tests**

Add focused tests that require:
- `QUANTIX_EXECUTION_CONFIG_PATH`
- default `~/.quantix/execution/config.json`
- relative fallback `.quantix/execution/config.json` without `HOME`
- `ExecutionDaemonConfig::load_or_create()` creates:

```json
{
  "poll_interval_secs": 10,
  "max_requests_per_iteration": 1,
  "auto_approval": {
    "mode": "manual"
  }
}
```

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
cargo test --lib core::runtime::tests:: -- --nocapture
cargo test --test execution_daemon_test config_ -- --nocapture
```

Expected:
- FAIL because the execution runtime path and config store do not exist yet.

- [ ] **Step 4: Implement runtime path and config store**

Add:

```rust
pub const EXECUTION_CONFIG_PATH_ENV: &str = "QUANTIX_EXECUTION_CONFIG_PATH";
pub execution_config_path: PathBuf
```

Implement:
- `AutoApprovalMode`
- `ExecutionDaemonConfig`
- `JsonExecutionConfigStore`

Keep the first slice limited to:
- `poll_interval_secs`
- `max_requests_per_iteration`
- `auto_approval.mode`

- [ ] **Step 5: Re-run focused tests to verify GREEN**

Run:
```bash
cargo test --lib core::runtime::tests:: -- --nocapture
cargo test --test execution_daemon_test config_ -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected:
- only runtime/config related files and tests are affected.

Commit:
```bash
git add src/core/runtime.rs src/execution/config.rs src/execution/mod.rs tests/execution_daemon_test.rs
git commit -m "feat: add execution daemon config foundation"
```

## Chunk 2: Request Claiming And `in_progress`

### Task 2: Extend request lifecycle for daemon-safe claiming

**Files:**
- Modify: `src/execution/models.rs`
- Modify: `src/execution/runtime_store.rs`
- Modify: `tests/execution_runtime_store_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for request status/store symbols**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "ExecutionRequestStatus", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "supersede_previous_signals_and_cancel_pending_requests", direction: "upstream"})
```

Expected:
- request status has broad but understandable blast radius in CLI/store/tests.
- supersede helper should be medium risk around request cancellation semantics.

- [ ] **Step 2: Write the failing request-claim tests**

Add tests covering:
- `pending -> in_progress`
- only one claim succeeds
- `in_progress -> completed`
- `in_progress -> failed`
- `pending -> canceled`
- supersede cancels only `pending`, not `in_progress`

Suggested assertions:

```rust
assert!(store.try_start_execution_request(...).await.unwrap());
assert!(!store.try_start_execution_request(...).await.unwrap());
assert_eq!(saved.request_status, ExecutionRequestStatus::InProgress);
```

- [ ] **Step 3: Run focused store tests to verify RED**

Run:
```bash
cargo test --test execution_runtime_store_test execution_request_ -- --nocapture
```

Expected:
- FAIL because `in_progress` and the claim helper do not exist yet.

- [ ] **Step 4: Implement request claiming and status transitions**

Add:
- `ExecutionRequestStatus::InProgress`
- `find_next_pending_execution_request()`
- `try_start_execution_request(request_id, payload_json, updated_at)`

Update terminal helpers so:
- `completed/failed` require current status `in_progress`
- `canceled` still requires current status `pending`

Update supersede so it only cancels `pending`.

- [ ] **Step 5: Re-run focused store tests to verify GREEN**

Run:
```bash
cargo test --test execution_runtime_store_test execution_request_ -- --nocapture
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
git add src/execution/models.rs src/execution/runtime_store.rs tests/execution_runtime_store_test.rs
git commit -m "feat: add in-progress execution request claiming"
```

## Chunk 3: Shared Request Consumer And Execution Daemon

### Task 3: Add shared request-consumption helper and daemon runner

**Files:**
- Create: `src/execution/daemon.rs`
- Modify: `src/execution/mod.rs`
- Modify: `src/cli/handlers.rs`
- Test: `tests/execution_daemon_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for request execution paths**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "execute_request", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "execute_strategy_request_execute_with_components", direction: "upstream"})
```

Expected:
- request execution helper is medium/high risk because it hangs off CLI behavior.
- Keep the daemon reusing this same path instead of inventing another one.

- [ ] **Step 2: Write the failing daemon tests**

Add tests covering:
- `run_once` consumes one pending request
- request becomes `in_progress` before final state
- successful `paper` request ends `completed`
- successful `mock_live` request ends `completed` with non-final order status allowed
- no pending request returns a no-op summary

Suggested assertions:

```rust
assert_eq!(summary.claimed, 1);
assert_eq!(saved.request_status, ExecutionRequestStatus::Completed);
assert_eq!(saved.payload_json["execution_result"]["order_status"], "accepted");
```

- [ ] **Step 3: Run focused daemon tests to verify RED**

Run:
```bash
cargo test --test execution_daemon_test daemon_ -- --nocapture
```

Expected:
- FAIL because execution daemon runner/helper does not exist yet.

- [ ] **Step 4: Implement the shared request-consumption helper**

Create a focused module that:
- loads one request
- claims it with `try_start_execution_request(...)`
- reuses `ExecutionKernel::execute_request(...)`
- writes `completed` or `failed`

Recommended output:

```rust
pub struct ExecutionDaemonIterationSummary {
    pub claimed: usize,
    pub completed: usize,
    pub failed: usize,
}
```

Then make both:
- CLI manual `request execute`
- daemon runner

reuse that same helper.

- [ ] **Step 5: Re-run focused daemon tests to verify GREEN**

Run:
```bash
cargo test --test execution_daemon_test daemon_ -- --nocapture
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
git add src/execution/daemon.rs src/execution/mod.rs src/cli/handlers.rs tests/execution_daemon_test.rs
git commit -m "feat: add execution daemon request consumer"
```

## Chunk 4: Auto-Approval In Strategy Daemon

### Task 4: Support `manual|always` auto-approval policy

**Files:**
- Modify: `src/execution/config.rs`
- Modify: `src/strategy/daemon.rs`
- Modify: `tests/strategy_daemon_test.rs`

- [ ] **Step 1: Write the failing auto-approval tests**

Add tests covering:
- `manual` leaves new signals at `approval_status=pending`
- `always` auto-approves and creates exactly one `pending execution_request`

Suggested assertions:

```rust
assert_eq!(signal.approval_status, ApprovalStatus::Approved);
assert_eq!(requests.len(), 1);
assert_eq!(requests[0].request_status, ExecutionRequestStatus::Pending);
```

- [ ] **Step 2: Run focused daemon tests to verify RED**

Run:
```bash
cargo test --test strategy_daemon_test auto_approval_ -- --nocapture
```

Expected:
- FAIL because auto-approval mode is not wired into `strategy daemon`.

- [ ] **Step 3: Implement `manual|always` auto-approval**

Rules:
- `manual`: existing behavior
- `always`: call the existing approval transaction immediately after signal creation

Do not:
- move approval into execution daemon
- add richer policy logic

- [ ] **Step 4: Re-run focused daemon tests to verify GREEN**

Run:
```bash
cargo test --test strategy_daemon_test auto_approval_ -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Commit:
```bash
git add src/execution/config.rs src/strategy/daemon.rs tests/strategy_daemon_test.rs
git commit -m "feat: add execution auto-approval modes"
```

## Chunk 5: Execution CLI Surface, Docs, And Final Verification

### Task 5: Add execution CLI commands and document automation boundary

**Files:**
- Modify: `src/cli/mod.rs`
- Modify: `src/cli/handlers.rs`
- Create: `src/cli/tests/execution.rs`
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Write the failing parser/doc tests**

Add parser coverage for:

```bash
quantix execution daemon run --once
quantix execution config init
quantix execution config show
```

Add hygiene assertions that docs mention:
- independent execution daemon
- `in_progress`
- `manual|always` auto-approval
- `mock_live accepted -> request completed`

- [ ] **Step 2: Run focused parser/doc tests to verify RED**

Run:
```bash
cargo test --lib cli::tests::execution:: -- --nocapture
cargo test --test repo_hygiene_test -- --nocapture
```

Expected:
- FAIL because execution command tree and docs do not exist yet.

- [ ] **Step 3: Implement execution CLI**

Add:

```bash
quantix execution daemon run
quantix execution daemon run --once
quantix execution config init
quantix execution config show
```

Execution daemon output should make request-consumption visible, for example:

```text
execution daemon consumed request=<ID> status=completed
```

- [ ] **Step 4: Update docs**

README and USER_MANUAL must explicitly state:
- `strategy daemon` produces signals
- `execution daemon` consumes requests
- `manual|always` auto-approval is the current policy surface
- `live` broker integration is still deferred

- [ ] **Step 5: Re-run focused parser/doc tests to verify GREEN**

Run:
```bash
cargo test --lib cli::tests::execution:: -- --nocapture
cargo test --test repo_hygiene_test -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run full verification**

Run:
```bash
cargo test --test execution_runtime_store_test -- --nocapture
cargo test --test execution_kernel_test -- --nocapture
cargo test --test strategy_daemon_test -- --nocapture
cargo test --test execution_daemon_test -- --nocapture
cargo test --test strategy_mock_live_run_test -- --nocapture
cargo test --test strategy_paper_run_test -- --nocapture
cargo test --lib cli::tests::strategy:: -- --nocapture
cargo test --lib cli::tests::execution:: -- --nocapture
cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture
cargo test --test repo_hygiene_test -- --nocapture
```

Expected:
- execution automation slice passes without regressing current strategy/request paths.

- [ ] **Step 7: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected:
- affected files remain within execution automation scope only.

Commit:
```bash
git add src/core/runtime.rs src/execution/config.rs src/execution/daemon.rs src/execution/mod.rs src/execution/models.rs src/execution/runtime_store.rs src/strategy/daemon.rs src/cli/mod.rs src/cli/handlers.rs src/cli/tests/execution.rs README.md docs/USER_MANUAL.md tests/execution_runtime_store_test.rs tests/strategy_daemon_test.rs tests/execution_daemon_test.rs tests/repo_hygiene_test.rs
git commit -m "feat: add execution automation closure"
```

## Final Memory

- [ ] **Step 1: Record Graphiti outcome**

Write a conclusion-oriented Graphiti memory for the design and implementation outcome. If ingest fails, preserve an equivalent local summary and mark:

```text
Graphiti backfill completed on 2026-03-24; memory is present in `quantix_rust_main`.
```

## Local Completion Note

- 2026-03-23 implementation reached the planned automation closure slice.
- Added top-level execution commands:
  - `quantix execution config init`
  - `quantix execution config show`
  - `quantix execution daemon run`
  - `quantix execution daemon run --once`
- Added `JsonExecutionConfigStore` at `~/.quantix/execution/config.json` with `QUANTIX_EXECUTION_CONFIG_PATH` override.
- Extended `ExecutionRequestStatus` with `in_progress` and added store-side pending discovery plus atomic request claim.
- Added shared request consumer in `src/execution/daemon.rs`, reused by manual `strategy request execute` and execution daemon.
- `StrategySignalDaemon` now supports `manual|always` auto-approval using execution config; current first slice auto-routes approved requests to `paper/default`.
- Updated README and USER_MANUAL to document Phase 29C execution automation boundary and top-level execution command semantics.
- Fresh verification completed successfully:
  - `cargo test --test execution_daemon_test --test execution_runtime_store_test --test execution_kernel_test --test strategy_daemon_test --test strategy_mock_live_run_test --test strategy_paper_run_test --test repo_hygiene_test -- --nocapture`
  - `cargo test --lib cli::tests::strategy:: -- --nocapture`
  - `cargo test --lib cli::tests::execution:: -- --nocapture`
  - `cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture`
- `gitnexus_detect_changes({scope: "all"})` reported CRITICAL because the workspace contains many unrelated user-side modifications; focused diff for this slice remained within execution / strategy / CLI / docs / targeted tests.

Graphiti backfill completed on 2026-03-24; memory is present in `quantix_rust_main`.
