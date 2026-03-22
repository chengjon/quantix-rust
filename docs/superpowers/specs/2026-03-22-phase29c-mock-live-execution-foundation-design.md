# Phase 29C Mock Live Execution Foundation Design

**Date:** 2026-03-22
**Status:** Draft for user file review
**Depends On:** Current baseline (`master` @ `8753b9f`)

> This document is the source of truth for the first Phase 29C slice: introduce a mock-live execution path with a durable order lifecycle model, while keeping the existing paper path intact and preserving a clean handoff to later execution-request automation.

---

## Goal

Build the smallest useful `mock_live` execution foundation that upgrades the current immediate-fill model into a durable lifecycle model without prematurely building a full execution daemon.

This slice must:

1. add a `mock_live` strategy execution path for `strategy run`
2. preserve the existing `paper` behavior as an immediate-fill adapter
3. model non-final order states such as `submitted`, `accepted`, `partially_filled`, and `unknown`
4. persist adapter-private lifecycle state in `runtime.db`
5. make `query`, `cancel`, and recovery-first execution behavior possible
6. keep the design compatible with later execution-request consumption and daemon automation

This slice must not:

- consume `execution_request` rows
- add an execution daemon
- auto-trade from the strategy daemon
- add a real broker or exchange adapter
- add multi-symbol or multi-account automation

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. run `quantix strategy run --mode mock_live --code <CODE>`
2. see that an order can exist in a non-final state
3. inspect current order status through runtime audit records
4. allow later query or recovery logic to move that order forward
5. keep paper-trade state unchanged until a fill actually happens

### Exact CLI boundary

This slice implements:

```bash
quantix strategy run --name ma_cross --mode mock_live --code <CODE>
```

Rules:

- `paper` remains supported and unchanged
- `live` remains unsupported
- `mock_live` uses the same strategy evaluation and risk pipeline as `paper`
- `mock_live` may return a non-final order status from the initial run
- user-facing summaries must report the current order status, not imply final completion

### Explicitly deferred

This slice does not include:

- execution-request consumers
- daemon-driven order polling or auto-recovery loops
- approval policies
- real market connectivity
- production-grade reconciliation
- service or systemd integration changes

## Approaches Considered

### Option A: Adapter-only patch for `strategy run`

Add a `MockLiveExecutionAdapter` and wire it only into the direct `strategy run` path.

Pros:

- smallest diff
- fastest path to one visible `mock_live` command

Cons:

- leaves lifecycle semantics under-modeled
- risks another model rewrite when execution-request automation arrives

### Option B: Lifecycle-first design with mock-live as the first non-final adapter

Keep the current kernel and adapter boundaries, but add durable lifecycle state, recovery-aware kernel behavior, and a dedicated mock-live adapter.

Pros:

- reuses the current execution architecture
- creates a stable path for later request consumption
- introduces `Unknown` and partial-fill behavior once
- avoids paper-specific leakage into later live work

Cons:

- larger than an adapter-only patch
- requires runtime store changes now

### Option C: Build execution daemon plus mock-live in one slice

Pros:

- most complete end-to-end automation
- fewer future integration steps

Cons:

- too much scope for the first Phase 29C slice
- mixes mock-live modeling with request-consumer and daemon concerns

## Recommendation

Choose **Option B**.

Phase 29C should first establish a durable non-final order lifecycle model and implement `mock_live` as the first adapter that uses it. This keeps the current architecture intact, preserves the working paper path, and creates the right seam for later execution-request automation.

## Architecture

### Preserved top-level chain

Keep the current flow:

`StrategyRuntime -> ExecutionKernel -> ExecutionAdapter`

The current architecture already provides the correct orchestration boundary:

- `StrategyRuntime` produces a signal
- `ExecutionKernel` translates signal to intent, performs risk evaluation, and writes runtime audit rows
- `ExecutionAdapter` owns execution-mode-specific behavior

This slice should not reassign those responsibilities.

### New components

Add:

- `src/execution/mock_live.rs`
  - `MockLiveExecutionAdapter`
  - mock-live clock abstraction
  - mock-live simulation plan helpers

- runtime-store support for adapter-private order state
  - new `mock_live_orders` table in `runtime.db`
  - store helpers for create/query/advance/recovery

- recovery-aware kernel logic
  - `execute_once()` must accept non-final submit results
  - `recover_pending_orders()` must stop being a placeholder

### Adapter identity

The shared `orders.adapter` field must describe the concrete execution adapter, not just the user-facing mode string.

Preferred direction:

- expose adapter identity from the adapter boundary itself
- avoid hardcoding `"paper"` in the kernel
- avoid assuming `mode == adapter name` forever

This keeps room for future cases such as:

- one mode mapping to different concrete adapters
- feature-flagged adapter variants
- test doubles and mock-live implementations that should remain distinguishable in audit rows

### Data ownership

- `paper_trade.json`
  - remains the source of truth for paper account state
  - is only updated when a mock-live fill actually occurs

- `risk_state.json`
  - remains the source of truth for risk rules and lock state

- `runtime.db`
  - remains the source of truth for run, signal, order, and order-event audit rows
  - additionally stores mock-live adapter-private lifecycle state

This slice intentionally does not turn the paper-trade store into a live order book.

## State Model

### Public order statuses

Reuse the existing shared order status model:

- `pending_submit`
- `submitted`
- `accepted`
- `partially_filled`
- `filled`
- `canceled`
- `rejected`
- `unknown`

No new public status enum is required in this slice.

### Allowed transitions

The mock-live adapter must enforce:

- `pending_submit -> submitted | accepted | rejected | unknown`
- `submitted -> accepted | partially_filled | filled | canceled | unknown`
- `accepted -> partially_filled | filled | canceled | unknown`
- `partially_filled -> partially_filled | filled | canceled | unknown`
- `unknown -> submitted | accepted | partially_filled | filled | canceled`

Terminal states:

- `filled`
- `canceled`
- `rejected`

`unknown` is explicitly **not** terminal. It means the system cannot currently prove the true state and must allow later recovery.

### Fill semantics

The paper account must only change when newly filled quantity is observed.

Implications:

- `submit_order()` does not imply a trade-book mutation
- `partially_filled` can produce multiple trade-store mutations over time
- cancellation after partial fill only cancels the remaining quantity

## Runtime Store Changes

### New private table

Add `mock_live_orders` to `runtime.db`.

Purpose:

- store adapter-private lifecycle state without polluting the shared public audit schema
- keep recovery, query, and partial-fill simulation idempotent

Recommended shape:

```sql
CREATE TABLE IF NOT EXISTS mock_live_orders (
    adapter_order_id TEXT PRIMARY KEY,
    client_order_id TEXT NOT NULL UNIQUE,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    requested_quantity INTEGER NOT NULL,
    filled_quantity INTEGER NOT NULL,
    remaining_quantity INTEGER NOT NULL,
    limit_price TEXT NOT NULL,
    status TEXT NOT NULL,
    avg_fill_price TEXT,
    submitted_at TEXT NOT NULL,
    last_transition_at TEXT NOT NULL,
    state_json TEXT NOT NULL
);
```

`state_json` carries mock-live-specific details such as:

- fill plan
- fault plan
- next step index
- unknown-until marker
- cancel-requested marker
- last-applied-fill id

### Typed private state model

`state_json` should not be handled as ad hoc untyped JSON throughout the implementation.

Introduce a strongly typed Rust model, serialized with `serde`, for example:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MockLiveOrderState {
    pub fill_plan: Vec<FillStep>,
    pub next_step_index: usize,
    pub planned_fill_time: Option<DateTime<Utc>>,
    pub fault_injection: Option<FaultInjectionConfig>,
    pub unknown_until: Option<DateTime<Utc>>,
    pub cancel_requested: bool,
    pub last_applied_fill_id: u64,
    pub unknown_retries: u32,
    pub recovery_exhausted: bool,
    pub exhausted_reason: Option<String>,
}
```

Requirements:

- use `serde` defaults or equivalent backward-compatible decoding
- make adding future fields non-breaking for existing runtime rows
- keep JSON parsing localized to store or adapter-private helpers

### Store API additions

Add dedicated store helpers rather than scattering SQL in the adapter:

- `insert_mock_live_order(...)`
- `find_mock_live_order(...)`
- `advance_mock_live_order(...)`
- `list_recoverable_mock_live_orders(...)`

`advance_mock_live_order(...)` should atomically:

1. update shared `orders`
2. append the corresponding `order_events`
3. update `mock_live_orders`

This keeps the public audit trail and the adapter-private state in sync.

## Adapter Semantics

### `submit_order`

`submit_order()` creates the adapter-private mock-live record and returns the current known state.

Expected first-slice behavior:

- normally returns `submitted` or `accepted`
- may return `rejected` for deterministic local validation failure
- may return `unknown` for simulated non-deterministic execution faults
- must not mutate the paper-trade account unless the initial simulation step explicitly includes a fill

### `query_order`

`query_order()` is the primary lifecycle-advance mechanism.

It may:

- keep the current state
- advance to `partially_filled`
- advance to `filled`
- move into or out of `unknown`

### `cancel_order`

`cancel_order()` is allowed only for non-terminal orders.

Allowed source states:

- `submitted`
- `accepted`
- `partially_filled`
- `unknown`

Disallowed source states:

- `filled`
- `rejected`
- `canceled`

## Kernel Changes

### `execute_once()`

Keep the current orchestration contract, but remove the assumption that a successful submit must be final.

Required behavior:

- create the shared order row before adapter submission
- write `pending_submit`
- write the adapter-returned current status
- update the order row to the adapter-returned current status even if it is non-final
- call `risk.sync_after_fill()` only when newly filled quantity is observed
- apply account mutations through a kernel-owned fill-delta path, not from the adapter
- record adapter identity from the concrete adapter boundary rather than hardcoding `"paper"`

Recommended internal helper:

- `apply_fill_delta(order_id, old_filled_qty, new_filled_qty, fill_price, fill_details)`

This helper should:

1. detect incremental newly filled quantity
2. update the paper-trade account for only that delta
3. append the corresponding fill event details

The adapter returns raw execution state; the kernel remains the owner of account mutation and shared audit semantics.

### `recover_pending_orders()`

Replace the current placeholder with a real recovery pass.

First-slice recovery scope:

- scan non-terminal mock-live orders
- query the adapter for their current status
- persist any resulting transitions
- count scanned and transitioned orders

Field semantics:

- `failed` means **recovery attempts that failed to complete**, such as adapter query failures or unrecoverable version-conflict retries
- `failed` does **not** mean the underlying order reached a terminal failed status
- order truth remains encoded in public order status plus private mock-live exhaustion state

This slice does not require a background loop; a direct recovery call is sufficient.

## CLI Changes

### New mode

Add:

```bash
quantix strategy run --name ma_cross --mode mock_live --code <CODE>
```

Mode rules:

- `paper`: immediate-fill semantics, unchanged
- `mock_live`: lifecycle semantics, non-final states allowed
- `live`: still unsupported

### Summary rules

CLI output must report the current state truthfully.

Examples:

- `signal=buy order_status=submitted`
- `signal=buy order_status=accepted`
- `signal=buy order_status=partially_filled`
- `signal=buy order_status=unknown`

The first-slice summary must not imply that the order has completed unless the status is actually terminal.

## Test Strategy

### Layer 1: adapter tests

Add `tests/mock_live_adapter_test.rs`.

Cover:

- submit returns non-final state
- query advances to partial fill
- query advances to final fill
- cancel transitions to canceled
- unknown can recover to a known state
- repeated query/recovery does not duplicate fills

### Layer 2: kernel tests

Extend `tests/execution_kernel_test.rs`.

Cover:

- non-final submit results are persisted correctly
- order events become multi-step
- `sync_after_fill()` only runs when new fill quantity is observed
- `recover_pending_orders()` scans and advances recoverable orders

### Layer 3: CLI tests

Extend `src/cli/tests/strategy.rs`.

Cover:

- `--mode mock_live` parsing
- mode validation boundaries remain correct

### Layer 4: integration tests

Add `tests/strategy_mock_live_run_test.rs`.

Cover:

- direct `strategy run --mode mock_live`
- runtime rows exist
- paper account changes only after fill
- dedupe behavior still works

## Acceptance Criteria

This slice is complete when:

1. `strategy run --mode mock_live --code <CODE>` works
2. initial submit may return a non-final order status
3. `runtime.db` stores shared audit rows plus `mock_live_orders`
4. paper account mutations happen only on fill
5. `recover_pending_orders()` no longer returns a fixed empty summary
6. `paper` remains immediate-fill and backward compatible

## Implementation Plan

Recommended implementation waves:

1. runtime-store schema and mock-live state primitives
2. `MockLiveExecutionAdapter`
3. kernel support for non-final states and recovery
4. CLI mode wiring and user-facing summaries
5. docs and tests

Suggested commit sequence:

1. `feat: add phase29c mock live runtime store primitives`
2. `feat: add phase29c mock live execution adapter`
3. `feat: support phase29c pending order recovery in execution kernel`
4. `feat: wire phase29c mock live strategy run mode`
5. `docs: document phase29c mock live execution boundary`
6. `test: cover phase29c mock live lifecycle`

## Non-Goals

This slice does not attempt to deliver:

- execution-request consumption
- daemon-driven execution automation
- multi-symbol scheduling
- broker-grade reconciliation
- real live adapter support
- service-management changes
