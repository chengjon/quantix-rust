# Phase 29C Fill Delta Accounting Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add real incremental accounting for mock-live fills so both `execute_once()` and `recover_pending_orders()` apply trade/account/risk side effects through a single idempotent `apply_fill_delta(...)` path.

**Architecture:** Keep adapters responsible only for execution-state observation and move all real paper-account mutations behind a kernel-owned fill-delta collaborator. Extend adapter responses with per-delta fill details, add a dedicated fill-delta applier bridge near the CLI wiring, and ensure both direct-path and recovery-path accounting use the same idempotent logic.

**Tech Stack:** Rust, Tokio, SQLx/SQLite, serde, clap, existing `TradeService`, existing risk bridge, Graphiti MCP for semantic memory, GitNexus for impact analysis.

---

## Critical Design Note

Current `ExecutionKernel` only owns:

- adapter
- risk evaluator

It does **not** own a trade/account mutation collaborator today. This plan therefore introduces a third collaborator for fill-delta accounting rather than pushing trade writes into:

- the adapter
- the risk evaluator
- or ad hoc CLI branches

That is the minimum architecture change required to implement the approved design cleanly.

## File Map

### Core execution files

- Modify: `src/execution/adapter.rs`
  - add `FillDetails`
  - extend `OrderInitialResponse`
  - extend `OrderQueryResponse`
- Modify: `src/execution/models.rs`
  - add `FillDeltaContext`
  - add `FillDeltaResult`
- Modify: `src/execution/mock_live.rs`
  - emit `fill_details`
  - maintain `last_applied_fill_id` semantics correctly
- Modify: `src/execution/kernel.rs`
  - add third collaborator for fill-delta application
  - implement `apply_fill_delta(...)` sequencing
  - prevent public order filled quantity from advancing when accounting fails
  - reuse the same helper in `execute_once()` and `recover_pending_orders()`

### CLI / bridge files

- Modify: `src/cli/handlers.rs`
  - add concrete fill-delta bridge backed by `TradeService`
  - wire kernel with adapter + fill-delta bridge + risk bridge

### Trade / reporting files

- Modify: `tests/trade_reporting_test.rs`
  - cover multiple `TradeRecord` rows from a single logical mock-live order

### Tests

- Modify: `tests/mock_live_adapter_test.rs`
  - assert `fill_details` are present and stable
- Modify: `tests/execution_kernel_test.rs`
  - direct-path delta accounting
  - recovery-path delta accounting
  - `fill_apply_failed`
  - idempotency by `fill_id`
- Modify: `tests/strategy_mock_live_run_test.rs`
  - account changes only after delta fill application
  - recovery path applies only new deltas
- Modify: `tests/strategy_paper_run_test.rs`
  - keep paper path green if response shape changes
- Modify: `src/cli/tests/strategy.rs`
  - only if parser/help text changes
- Modify: `README.md`
  - document account-delta semantics if user-facing behavior changes
- Modify: `docs/USER_MANUAL.md`
  - document account-delta semantics
- Modify: `tests/repo_hygiene_test.rs`
  - enforce updated docs if README / USER_MANUAL change

## Chunk 1: Fill Details and Adapter Response Shape

### Task 1: Extend adapter response types with per-delta fill information

**Files:**
- Modify: `src/execution/adapter.rs`
- Modify: `src/execution/models.rs`
- Test: `tests/mock_live_adapter_test.rs`

- [ ] **Step 1: Add failing adapter tests that require `fill_details`**

Add assertions that:

```rust
assert_eq!(response.fill_details.as_ref().unwrap().fill_id, 1);
assert_eq!(response.fill_details.as_ref().unwrap().fill_quantity, 40);
assert_eq!(response.fill_details.as_ref().unwrap().fill_price, dec!(12.34));
```

Cover:

- partial fill query emits fill details
- final fill query emits a distinct next fill id
- accepted-with-no-fill emits `fill_details = None`

- [ ] **Step 2: Run the adapter test target and verify failure**

Run:

```bash
cargo test --test mock_live_adapter_test -- --nocapture
```

Expected:
- compile errors because `FillDetails` / `fill_details` do not exist yet

- [ ] **Step 3: Add `FillDetails` and fill-delta model structs**

In `src/execution/adapter.rs` and/or `src/execution/models.rs`, add:

```rust
pub struct FillDetails {
    pub fill_id: u64,
    pub fill_quantity: i64,
    pub fill_price: Decimal,
}
```

Add:

```rust
pub struct FillDeltaContext { ... }
pub struct FillDeltaResult { ... }
```

Include fields for:

- order identity
- side / symbol
- old/new filled quantity
- requested price
- `fill_details`
- event time

- [ ] **Step 4: Extend adapter response structs**

Extend:

```rust
pub struct OrderInitialResponse {
    ...
    pub fill_details: Option<FillDetails>,
}

pub struct OrderQueryResponse {
    ...
    pub fill_details: Option<FillDetails>,
}
```

- [ ] **Step 5: Run the adapter tests and confirm green**

Run:

```bash
cargo test --test mock_live_adapter_test -- --nocapture
```

Expected:
- response-shape tests PASS
- existing mock-live tests remain green

- [ ] **Step 6: Commit**

```bash
git add src/execution/adapter.rs src/execution/models.rs tests/mock_live_adapter_test.rs
git commit -m "feat: add phase29c fill detail response types"
```

## Chunk 2: Fill-Delta Applier Bridge

### Task 2: Add a concrete accounting collaborator for the kernel

**Files:**
- Modify: `src/execution/kernel.rs`
- Modify: `src/cli/handlers.rs`
- Test: `tests/execution_kernel_test.rs`

- [ ] **Step 1: Write failing kernel tests for a third collaborator**

Add tests covering:

- direct-path partial fill applies one `TradeRecord`
- direct-path accepted-without-fill does not mutate account
- repeated same `fill_id` is a no-op

Use explicit assertions on:

- account position volume
- trade history length
- returned `FillDeltaResult.applied`

- [ ] **Step 2: Run the kernel test target and verify failure**

Run:

```bash
cargo test --test execution_kernel_test -- --nocapture
```

Expected:
- compile or logic failure because kernel has no fill-delta collaborator yet

- [ ] **Step 3: Introduce a fill-delta application trait**

In `src/execution/kernel.rs`, add something equivalent to:

```rust
#[async_trait]
pub trait FillDeltaApplier: Send + Sync {
    async fn apply_fill_delta(&self, ctx: FillDeltaContext) -> Result<FillDeltaResult>;
}
```

Update kernel generic shape from:

```rust
ExecutionKernel<A, R>
```

to:

```rust
ExecutionKernel<A, F, R>
```

- [ ] **Step 4: Add a concrete bridge in CLI handlers**

In `src/cli/handlers.rs`, add a bridge near `StrategyRiskBridge`, for example:

```rust
struct StrategyFillDeltaBridge<TradeStore> {
    trade_store: TradeStore,
}
```

Back it with `TradeService::buy/sell`.

The bridge should:

- treat each delta as one real execution
- write one `TradeRecord`
- return `FillDeltaResult`

- [ ] **Step 5: Keep paper-trade mutation outside adapters**

Do **not** write account changes inside:

- `MockLiveExecutionAdapter`
- `PaperExecutionAdapter`
- risk bridge

Only the new fill-delta bridge may mutate the paper account.

- [ ] **Step 6: Run kernel tests and confirm green**

Run:

```bash
cargo test --test execution_kernel_test -- --nocapture
```

Expected:
- direct-path fill-delta tests PASS

- [ ] **Step 7: Commit**

```bash
git add src/execution/kernel.rs src/cli/handlers.rs tests/execution_kernel_test.rs
git commit -m "feat: add phase29c fill delta accounting bridge"
```

## Chunk 3: Direct Path Accounting Semantics

### Task 3: Make `execute_once()` honor ledger-first accounting

**Files:**
- Modify: `src/execution/kernel.rs`
- Modify: `tests/execution_kernel_test.rs`
- Modify: `tests/strategy_mock_live_run_test.rs`

- [ ] **Step 1: Add failing tests for ledger-first sequencing**

Add assertions that:

- accepted-without-fill updates only lifecycle status
- partial fill produces exactly one new trade record
- order `filled_quantity` only advances after successful accounting
- `fill_apply_failed` leaves public filled quantity unchanged

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --test execution_kernel_test -- --nocapture
cargo test --test strategy_mock_live_run_test -- --nocapture
```

Expected:
- failures because `execute_once()` still updates public order rows before true delta accounting semantics are enforced

- [ ] **Step 3: Reorder `execute_once()`**

Required semantics:

1. create order row
2. submit adapter request
3. compare old/new filled quantity
4. call `apply_fill_delta(...)`
5. only then advance public `filled_quantity` if accounting succeeds
6. on accounting failure, write `fill_apply_failed`

- [ ] **Step 4: Add explicit events**

Keep state-change and accounting-change events separate:

- status event
- `fill_applied`
- `fill_apply_failed`

- [ ] **Step 5: Re-run tests**

Run:

```bash
cargo test --test execution_kernel_test -- --nocapture
cargo test --test strategy_mock_live_run_test -- --nocapture
```

Expected:
- direct-path accounting tests PASS

- [ ] **Step 6: Commit**

```bash
git add src/execution/kernel.rs tests/execution_kernel_test.rs tests/strategy_mock_live_run_test.rs
git commit -m "feat: enforce phase29c ledger-first direct fill accounting"
```

## Chunk 4: Recovery Path Accounting and Idempotency

### Task 4: Make `recover_pending_orders()` apply only new deltas

**Files:**
- Modify: `src/execution/kernel.rs`
- Modify: `src/execution/mock_live.rs`
- Modify: `tests/execution_kernel_test.rs`
- Modify: `tests/strategy_mock_live_run_test.rs`

---

## Local Memory Backfill Notes

### 2026-03-23 Chunk 1-2 checkpoint

Graphiti backfill completed on 2026-03-24; related memory is present in `quantix_rust_main`.

- Chunk 1 completed:
  - added `FillDetails`, `FillDeltaContext`, `FillDeltaResult`
  - extended adapter responses with optional `fill_details`
  - `MockLiveExecutionAdapter` now emits deterministic per-step fill details and persists `simulated_fill_price` in private `state_json` so adapter-only flows do not depend on a pre-existing public `orders` row
  - `PaperExecutionAdapter` immediate fills now include one `fill_details` payload
- Chunk 2 completed:
  - added kernel-side `FillDeltaApplier` trait plus default `NoopFillDeltaApplier`
  - added `ExecutionKernel::with_fill_delta(...)` for real accounting collaborators while preserving `ExecutionKernel::new(...)` as the no-op default constructor
  - `execute_once()` now invokes the fill-delta collaborator when submit returns positive filled quantity
  - added `StrategyFillDeltaBridge` in CLI handlers and wired it into the `mock_live` strategy run path
- Verification completed:
  - `cargo test --test mock_live_adapter_test -- --nocapture`
  - `cargo test --test execution_kernel_test -- --nocapture`
  - `cargo test --test strategy_mock_live_run_test -- --nocapture`
  - `cargo test --test strategy_paper_run_test -- --nocapture`
  - `cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture`

### 2026-03-23 Chunk 3-4 checkpoint

Graphiti backfill completed on 2026-03-24; related memory is present in `quantix_rust_main`.

- Chunk 3 completed:
  - `execute_once()` is now ledger-first for positive fill deltas
  - fill-delta application happens before public order advancement
  - successful direct fills append separate lifecycle and `fill_applied` events
  - failed direct fills append `fill_apply_failed` and leave public orders at pre-fill state
- Chunk 4 completed:
  - `MockLiveExecutionAdapter` now distinguishes revealed external fill progress from locally applied fill progress
  - `query_order()` repeats the same `fill_details` until `last_applied_fill_id` catches up, then reveals the next delta
  - kernel writes `last_applied_fill_id` back to mock-live private state only after successful order update
  - `recover_pending_orders()` now uses the same fill-delta path as `execute_once()` for positive deltas
  - recovery keeps `unknown` exhaustion account-neutral and records no paper-account mutation in that path
- Verification completed:
  - `cargo test --test mock_live_adapter_test -- --nocapture`
  - `cargo test --test execution_kernel_test kernel_recover_pending_orders -- --nocapture`
  - `cargo test --test execution_kernel_test -- --nocapture`
  - `cargo test --test strategy_mock_live_run_test -- --nocapture`
  - `cargo test --test strategy_paper_run_test -- --nocapture`
  - `cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture`

### 2026-03-23 Chunk 5 checkpoint

Graphiti backfill completed on 2026-03-24; related memory is present in `quantix_rust_main`.

- Chunk 5 completed:
  - `trade_reporting_test` now asserts that multiple partial fills remain separate rows in history and fee views while overview totals aggregate across them
  - README now documents that one mock-live order may emit multiple `TradeRecord` rows under partial-fill progression
  - USER_MANUAL now documents the same reporting-facing multi-delta semantics and explicitly states that those rows appear in `trade history` / `trade fees` / `trade overview`
  - `repo_hygiene_test` now enforces the new README / USER_MANUAL wording
- Verification completed:
  - `cargo test --test trade_reporting_test -- --nocapture`
  - `cargo test --test repo_hygiene_test -- --nocapture`

- [ ] **Step 1: Add failing recovery/idempotency tests**

Cover:

- recovery from `Accepted -> PartiallyFilled` writes exactly one delta trade
- second recovery from same `fill_id` writes nothing
- next `fill_id` writes the next delta only
- unknown exhaustion still does not mutate account

- [ ] **Step 2: Run recovery-focused tests and verify failure**

Run:

```bash
cargo test --test execution_kernel_test kernel_recover_pending_orders -- --nocapture
cargo test --test strategy_mock_live_run_test -- --nocapture
```

Expected:
- failures because recovery currently relies on coarse `filled_quantity > 0` semantics rather than strict fill-id accounting

- [ ] **Step 3: Track `last_applied_fill_id` properly**

Ensure mock-live state updates:

- assign monotonic `fill_id`
- preserve `last_applied_fill_id`
- return `fill_details` on each new fill

- [ ] **Step 4: Make recovery call the same helper**

`recover_pending_orders()` must:

- compare old/new fill totals
- call the same `apply_fill_delta(...)`
- only advance public order after accounting succeeds
- stay ledger-first on failures

- [ ] **Step 5: Re-run tests**

Run:

```bash
cargo test --test execution_kernel_test -- --nocapture
cargo test --test strategy_mock_live_run_test -- --nocapture
```

Expected:
- recovery/idempotency tests PASS

- [ ] **Step 6: Commit**

```bash
git add src/execution/kernel.rs src/execution/mock_live.rs tests/execution_kernel_test.rs tests/strategy_mock_live_run_test.rs
git commit -m "feat: add phase29c idempotent recovery fill accounting"
```

## Chunk 5: Reporting and Documentation

### Task 5: Make reporting and docs reflect multiple partial executions

**Files:**
- Modify: `tests/trade_reporting_test.rs`
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Add failing reporting tests**

Add tests showing that:

- one logical mock-live order may produce multiple `TradeRecord` rows
- `history_rows` shows all delta executions
- `fee_rows` accumulates fees across those deltas
- `overview` aggregates totals correctly

- [ ] **Step 2: Run reporting/hygiene tests to verify failure**

Run:

```bash
cargo test --test trade_reporting_test -- --nocapture
cargo test --test repo_hygiene_test -- --nocapture
```

Expected:
- reporting or docs assertions fail until semantics are documented

- [ ] **Step 3: Update docs**

Document:

- mock-live partial fills create multiple real trade records
- reporting aggregates them naturally
- account-delta semantics are now part of the current slice

- [ ] **Step 4: Run final regression pack**

Run:

```bash
cargo test --test execution_runtime_store_test -- --nocapture
cargo test --test mock_live_adapter_test -- --nocapture
cargo test --test execution_kernel_test -- --nocapture
cargo test --test strategy_mock_live_run_test -- --nocapture
cargo test --test strategy_paper_run_test -- --nocapture
cargo test --test trade_reporting_test -- --nocapture
cargo test --lib cli::tests::strategy:: -- --nocapture
cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture
cargo test --test repo_hygiene_test -- --nocapture
```

Expected:
- all listed targets PASS

- [ ] **Step 5: Commit**

```bash
git add tests/trade_reporting_test.rs README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "docs: record phase29c fill delta accounting semantics"
```

## Final Verification

- [ ] Run the full slice verification set:

```bash
cargo test --test execution_runtime_store_test -- --nocapture
cargo test --test mock_live_adapter_test -- --nocapture
cargo test --test execution_kernel_test -- --nocapture
cargo test --test strategy_mock_live_run_test -- --nocapture
cargo test --test strategy_paper_run_test -- --nocapture
cargo test --test trade_reporting_test -- --nocapture
cargo test --lib cli::tests::strategy:: -- --nocapture
cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture
cargo test --test repo_hygiene_test -- --nocapture
```

- [ ] Review changed scope:

```bash
git diff --stat
git diff --name-only
```

- [ ] Run GitNexus changed-scope review before final handoff:

```bash
# Review compare scope once implementation commits are complete
```

Plan complete and saved to `docs/superpowers/plans/2026-03-23-phase29c-fill-delta-accounting-implementation.md`. Ready to execute?
