# Execution Request Lifecycle Closure Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the current `execution_request` half-loop by freezing executable request snapshots at approval time, adding manual request execute/cancel commands, and writing terminal request results back into `runtime.db`.

**Architecture:** Keep the existing `signal -> request -> execution` split intact. Extend the request payload with a frozen execution snapshot, add conditional request-state update helpers in the runtime store, and introduce a request-oriented execution entrypoint that reuses the current execution kernel/adapters without enabling daemon auto-consumption.

**Tech Stack:** Rust, tokio, clap, sqlx/sqlite, serde/serde_json, chrono, existing `ExecutionKernel`, existing `StrategyRuntimeStore`, existing paper/mock-live adapters, existing trade/risk stores, GitNexus impact analysis, Graphiti MCP for semantic-memory workflow.

---

## Preflight

- Read the approved spec in [2026-03-23-execution-request-lifecycle-closure-design.md](/opt/claude/quantix-rust/docs/superpowers/specs/2026-03-23-execution-request-lifecycle-closure-design.md).
- Use `@superpowers/test-driven-development` for every behavior change. No production edits before a failing test.
- Use `@superpowers/verification-before-completion` before claiming the phase is done.
- Before editing any existing indexed symbol, run `gitnexus_impact({repo: "quantix-rust", target: "...", direction: "upstream"})` and review the blast radius. Stop for user review if risk is HIGH or CRITICAL.
- Before every commit, run `gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})`.
- Graphiti is mandatory for design/review/debug/handoff memories. If ingest fails, keep an equivalent local summary and mark `Graphiti backfill required`.
- The repository already contains unrelated dirty files. Stage only the files listed in the active task and never revert unrelated changes.

## File Map

- `src/strategy/daemon.rs`
  - Extend signal metadata with the execution-context fields needed to freeze later request snapshots deterministically.
- `src/execution/models.rs`
  - Add typed request snapshot / result payload structs if the implementation chooses typed serde wrappers instead of raw JSON assembly.
- `src/execution/runtime_store.rs`
  - Add request lookup and conditional terminal-state update helpers.
  - Extend approval flow so request creation copies a frozen execution snapshot into `execution_requests.payload_json`.
- `src/execution/kernel.rs`
  - Add a request-oriented execution entrypoint that consumes a frozen request snapshot without re-deriving intent from mutable current state.
- `src/cli/mod.rs`
  - Extend `StrategyRequestCommands` with `execute` and `cancel`.
- `src/cli/handlers.rs`
  - Add CLI request execution/cancel handlers and request-row formatting for terminal results.
- `tests/execution_runtime_store_test.rs`
  - Extend request/store tests for snapshot freezing and terminal-state transitions.
- `tests/strategy_daemon_test.rs`
  - Add signal-metadata coverage for execution snapshot prerequisites.
- `tests/execution_kernel_test.rs`
  - Add request-entrypoint tests that cover paper/mock-live request execution semantics.
- `tests/strategy_request_flow_test.rs`
  - New integration-style tests for request execute/cancel flows from runtime rows to request terminal states.
- `src/cli/tests/strategy.rs`
  - Parser coverage for `strategy request execute` and `strategy request cancel`.
- `README.md`
  - Document manual request consumption behavior and request terminal semantics.
- `docs/USER_MANUAL.md`
  - Document request execute/cancel workflow and output semantics.
- `tests/repo_hygiene_test.rs`
  - Lock the new README / USER_MANUAL wording.

## Chunk 1: Freeze Execution Snapshot At Approval Time

### Task 1: Extend signal metadata and request creation payloads

**Files:**
- Modify: `src/strategy/daemon.rs`
- Modify: `src/execution/runtime_store.rs`
- Test: `tests/strategy_daemon_test.rs`
- Test: `tests/execution_runtime_store_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for approval and request-store symbols**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "approve_signal_and_create_request", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "StrategySignalDaemon", direction: "upstream"})
```

Expected:
- `approve_signal_and_create_request` should show low/medium store/CLI risk.
- `StrategySignalDaemon` should show daemon/test callsites only.

- [ ] **Step 2: Write the failing daemon/store tests**

Add tests that require:

```rust
assert_eq!(signal.metadata_json["market_price"], "21");
assert_eq!(signal.metadata_json["signal_value"], "buy");
assert_eq!(request.payload_json["execution_snapshot"]["order_intent"]["requested_quantity"], 800);
assert_eq!(request.payload_json["execution_snapshot"]["order_intent"]["requested_price"], "12.34");
```

Cover:
- daemon-generated signal metadata includes execution snapshot prerequisites
- approve copies a frozen execution snapshot into `execution_requests.payload_json`
- approved request still starts in `pending`

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
cargo test --test strategy_daemon_test daemon_ -- --nocapture
cargo test --test execution_runtime_store_test approve_signal_creates_exactly_one_pending_execution_request -- --nocapture
```

Expected:
- FAIL because signal metadata and request payload snapshot fields do not exist yet.

- [ ] **Step 4: Implement snapshot freezing**

In `src/strategy/daemon.rs`, extend signal metadata with:

```json
{
  "market_price": "<close>",
  "signal_value": "buy|sell|hold",
  "execution_policy": {
    "fixed_cash_per_buy": "10000",
    "slippage_bps": 0
  }
}
```

In `src/execution/runtime_store.rs`, update `approve_signal_and_create_request(...)` so it:
- loads the signal row inside the approval transaction
- builds `execution_snapshot`
- stores it in `execution_requests.payload_json`

Keep approval non-executing:
- do not submit orders
- do not touch trade/risk stores

- [ ] **Step 5: Re-run focused tests to verify GREEN**

Run:
```bash
cargo test --test strategy_daemon_test daemon_ -- --nocapture
cargo test --test execution_runtime_store_test approve_signal_creates_exactly_one_pending_execution_request -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected:
- only daemon/runtime-store/test files related to snapshot freezing are affected.

Commit:
```bash
git add src/strategy/daemon.rs src/execution/runtime_store.rs tests/strategy_daemon_test.rs tests/execution_runtime_store_test.rs
git commit -m "feat: freeze execution snapshots on approved requests"
```

## Chunk 2: Request Terminal-State Store Semantics

### Task 2: Add conditional request-state update helpers

**Files:**
- Modify: `src/execution/runtime_store.rs`
- Modify: `tests/execution_runtime_store_test.rs`

- [ ] **Step 1: Write failing store tests for terminal request transitions**

Add tests that require:

```rust
assert!(store.try_complete_execution_request(...).await.unwrap());
assert!(!store.try_complete_execution_request(...).await.unwrap());
assert_eq!(saved.request_status, ExecutionRequestStatus::Completed);
assert_eq!(saved.payload_json["execution_result"]["order_status"], "accepted");
```

Cover:
- `pending -> completed`
- `pending -> failed`
- `pending -> canceled`
- repeated terminal transitions fail cleanly

- [ ] **Step 2: Run focused store tests to verify RED**

Run:
```bash
cargo test --test execution_runtime_store_test execution_request_ -- --nocapture
```

Expected:
- FAIL because the conditional terminal-state helpers do not exist yet.

- [ ] **Step 3: Implement request lookup and conditional terminal updates**

Add helpers equivalent to:

```rust
pub async fn get_execution_request(&self, request_id: &str) -> Result<Option<ExecutionRequestRecord>>;
pub async fn try_complete_execution_request(&self, request_id: &str, payload_json: Value, updated_at: DateTime<Utc>) -> Result<bool>;
pub async fn try_fail_execution_request(&self, request_id: &str, payload_json: Value, updated_at: DateTime<Utc>) -> Result<bool>;
pub async fn try_cancel_execution_request(&self, request_id: &str, payload_json: Value, updated_at: DateTime<Utc>) -> Result<bool>;
```

Rules:
- every helper must update only rows currently in `pending`
- payload is replaced with the caller-provided merged JSON
- `updated_at` is refreshed on success

- [ ] **Step 4: Re-run focused store tests to verify GREEN**

Run:
```bash
cargo test --test execution_runtime_store_test execution_request_ -- --nocapture
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
git add src/execution/runtime_store.rs tests/execution_runtime_store_test.rs
git commit -m "feat: add execution request terminal state helpers"
```

## Chunk 3: Request-Oriented Kernel Entry Point

### Task 3: Add manual request execution to the kernel

**Files:**
- Modify: `src/execution/kernel.rs`
- Modify: `tests/execution_kernel_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for kernel execution entrypoints**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "ExecutionKernel", direction: "upstream"})
gitnexus_impact({repo: "quantix-rust", target: "execute_once", direction: "upstream"})
```

Expected:
- `execute_once` is likely HIGH/CRITICAL because it hangs off the CLI strategy path.
- Keep changes isolated by adding a new request-oriented entrypoint instead of mutating direct-run semantics.

- [ ] **Step 2: Write the failing kernel tests for request execution**

Add tests covering:
- request snapshot for `paper` executes to `completed + order_status=filled`
- request snapshot for `mock_live` executes to `completed + order_status=accepted`
- kernel does not recompute quantity from current holdings/price

Suggested assertions:

```rust
assert_eq!(result.order_status, Some(OrderStatus::Filled));
assert_eq!(result.client_order_id.as_deref(), Some("req-exec-1_000001_1"));
```

- [ ] **Step 3: Run focused kernel tests to verify RED**

Run:
```bash
cargo test --test execution_kernel_test request_ -- --nocapture
```

Expected:
- FAIL because request-oriented execution entrypoint does not exist yet.

- [ ] **Step 4: Implement the request-oriented kernel entrypoint**

Add a request-snapshot input model and a kernel entrypoint equivalent to:

```rust
pub async fn execute_request(&self, request: RequestExecutionSnapshot) -> Result<KernelExecutionResult>;
```

Rules:
- do not re-run `translate_signal(...)`
- execute the frozen `order_intent`
- still reuse adapter, fill-delta, and risk orchestration
- keep `execute_once(...)` unchanged

- [ ] **Step 5: Re-run focused kernel tests to verify GREEN**

Run:
```bash
cargo test --test execution_kernel_test request_ -- --nocapture
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
git add src/execution/kernel.rs tests/execution_kernel_test.rs
git commit -m "feat: add request-oriented execution kernel entrypoint"
```

## Chunk 4: CLI Request Execute/Cancel Commands

### Task 4: Add request consumer CLI surface and handlers

**Files:**
- Modify: `src/cli/mod.rs`
- Modify: `src/cli/handlers.rs`
- Modify: `src/cli/tests/strategy.rs`
- Test: `tests/strategy_request_flow_test.rs`

- [ ] **Step 1: Write failing parser and handler tests**

Add parser coverage for:

```bash
quantix strategy request execute --request-id req-1
quantix strategy request cancel --request-id req-2 --reason "manual cancel"
```

Add handler/integration tests covering:
- execute pending paper request -> `completed`
- execute pending mock_live request -> `completed`
- execute non-pending request -> user-facing error
- cancel pending request -> `canceled`
- cancel non-pending request -> user-facing error

- [ ] **Step 2: Run focused CLI tests to verify RED**

Run:
```bash
cargo test --lib cli::tests::strategy:: -- --nocapture
cargo test --test strategy_request_flow_test -- --nocapture
```

Expected:
- FAIL because parser branches and handlers do not exist yet.

- [ ] **Step 3: Implement CLI commands and handlers**

Extend `StrategyRequestCommands` with:

```rust
Execute { request_id: String }
Cancel { request_id: String, reason: Option<String> }
```

In handlers:
- load request by ID
- require `pending`
- execute or cancel
- update request state via runtime-store helpers
- print concise result summaries

- [ ] **Step 4: Re-run focused CLI tests to verify GREEN**

Run:
```bash
cargo test --lib cli::tests::strategy:: -- --nocapture
cargo test --test strategy_request_flow_test -- --nocapture
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
git add src/cli/mod.rs src/cli/handlers.rs src/cli/tests/strategy.rs tests/strategy_request_flow_test.rs
git commit -m "feat: add manual execution request consumer commands"
```

## Chunk 5: Request Result Formatting And Documentation

### Task 5: Document request-consumer behavior and lock repo hygiene

**Files:**
- Modify: `src/cli/handlers.rs`
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Write failing formatting and hygiene tests**

Add tests asserting:
- request list rows include terminal result summaries
- README documents `strategy request execute`
- README documents `strategy request cancel`
- USER_MANUAL documents request completion semantics, especially that `mock_live accepted` still means request `completed`

- [ ] **Step 2: Run focused tests to verify RED**

Run:
```bash
cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture
cargo test --test repo_hygiene_test -- --nocapture
```

Expected:
- FAIL because the new request-consumer wording and formatting do not exist yet.

- [ ] **Step 3: Implement result formatting and docs**

Request list output should include result summary when present, for example:

```text
req-1 signal=signal-1 target=mock_live/default status=completed result=order_status=accepted client_order_id=req-1_000001_1
```

Docs must explicitly state:
- request `completed` means execution succeeded, not that order settlement is final
- `mock_live` requests may complete with non-final order status
- request consumer is manual in this slice

- [ ] **Step 4: Re-run focused tests to verify GREEN**

Run:
```bash
cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture
cargo test --test repo_hygiene_test -- --nocapture
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
git add src/cli/handlers.rs README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "docs: add execution request consumer guidance"
```

## Full Verification

- [ ] **Step 1: Run the end-to-end request lifecycle suite**

Run:
```bash
cargo test --test execution_runtime_store_test -- --nocapture
cargo test --test execution_kernel_test -- --nocapture
cargo test --test strategy_daemon_test -- --nocapture
cargo test --test strategy_request_flow_test -- --nocapture
cargo test --lib cli::tests::strategy:: -- --nocapture
cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture
cargo test --test repo_hygiene_test -- --nocapture
```

Expected:
- all request lifecycle tests pass
- direct `strategy run` behavior remains unchanged

- [ ] **Step 2: Run final change detection**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected:
- changed files remain inside request lifecycle closure scope only.

- [ ] **Step 3: Summarize Graphiti outcome**

Write one design/implementation memory to the appropriate Graphiti groups and verify ingest. If ingest fails, add a local backfill note and mark:

```text
Graphiti backfill completed on 2026-03-24; memory is present in `quantix_rust_main`.
```

---

## Local Memory Backfill Notes

### 2026-03-23 request lifecycle closure checkpoint

Graphiti backfill completed on 2026-03-24; memory is present in `quantix_rust_main`.

- Approval now freezes `execution_snapshot` into `execution_requests.payload_json`
- `StrategyRuntimeStore` now supports request lookup and conditional `pending -> completed/failed/canceled` transitions
- `ExecutionKernel` now supports request-oriented execution through `execute_request(...)`
- CLI now supports:
  - `quantix strategy request execute --request-id <ID>`
  - `quantix strategy request cancel --request-id <ID> [--reason <TEXT>]`
- Request list formatting now includes result summaries from `execution_result`, `execution_error`, or `cancellation`
- README and USER_MANUAL now document:
  - manual request consumption
  - non-automatic execution
  - `mock_live accepted -> request completed` semantics
- Verification completed:
  - `cargo test --test strategy_daemon_test daemon_writes_run_signal_and_checkpoint_when_new_bar_arrives -- --nocapture`
  - `cargo test --test execution_runtime_store_test approve_signal_creates_exactly_one_pending_execution_request -- --nocapture`
  - `cargo test --test execution_runtime_store_test execution_request_ -- --nocapture`
  - `cargo test --test execution_kernel_test request_prepared_execution -- --nocapture`
  - `cargo test --lib cli::tests::strategy:: -- --nocapture`
  - `cargo test --lib cli::handlers::tests::test_execute_strategy_request_execute_and_cancel -- --nocapture`
  - `cargo test --lib cli::handlers::tests::test_format_strategy_request_row_includes_target_and_status -- --nocapture`
  - `cargo test --test execution_runtime_store_test -- --nocapture`
  - `cargo test --test execution_kernel_test -- --nocapture`
  - `cargo test --test strategy_daemon_test -- --nocapture`
  - `cargo test --test strategy_mock_live_run_test -- --nocapture`
  - `cargo test --test strategy_paper_run_test -- --nocapture`
  - `cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture`
  - `cargo test --test repo_hygiene_test -- --nocapture`
