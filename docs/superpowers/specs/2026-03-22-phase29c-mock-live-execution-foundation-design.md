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
3. model non-final order states such as `submitted`, `accepted`, `partially_filled`, `pending_cancel`, and `unknown`
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

Extend the existing shared order status model with one additional lifecycle state:

- `pending_submit`
- `submitted`
- `accepted`
- `partially_filled`
- `pending_cancel`
- `filled`
- `canceled`
- `rejected`
- `unknown`

`pending_cancel` exists to represent an explicit cancellation-in-flight boundary. Even in mock-live mode, this keeps the lifecycle compatible with later broker-backed adapters where cancel and query may race.

### Allowed transitions

The mock-live adapter must enforce:

- `pending_submit -> submitted | accepted | rejected | unknown`
- `submitted -> accepted | partially_filled | filled | pending_cancel | unknown`
- `accepted -> partially_filled | filled | pending_cancel | unknown`
- `partially_filled -> partially_filled | filled | pending_cancel | unknown`
- `pending_cancel -> canceled | unknown`
- `unknown -> submitted | accepted | partially_filled | filled | canceled | rejected`

Terminal states:

- `filled`
- `canceled`
- `rejected`

`unknown` is explicitly **not** terminal. It means the system cannot currently prove the true state and must allow later recovery.

Recovery policy for `unknown`:

- retain the last known filled quantity
- increment a private `unknown_retries` counter each time query/recovery still returns `unknown`
- once retries exceed a configured threshold (default `3`), mark the private mock-live state as `recovery_exhausted`
- append an `order_events` row with `event_type = recovery_exhausted`
- keep the public order status as `unknown`

This intentionally separates:

- **public order truth**: still unknown
- **local executor state**: retry budget exhausted

The higher-level execution-request layer may later choose to mark the request as failed, but the order itself must not be forced into a false terminal state.

### Fill semantics

The paper account must only change when newly filled quantity is observed.

Implications:

- `submit_order()` does not imply a trade-book mutation
- `partially_filled` can produce multiple trade-store mutations over time
- cancellation after partial fill only cancels the remaining quantity
- partial fills are recorded as real incremental paper-trade executions, not deferred until final fill

### Accounting Consistency Rule

This slice prioritizes **paper account consistency over adapter truth**.

Rule:

- if a newly observed fill cannot be applied to the paper account successfully
- the shared public order row must not advance its `filled_quantity` beyond the amount already reflected in `paper_trade.json`

This avoids a broken state where:

- the public order ledger says a fill happened
- but the paper account, trade history, reporting, and risk state do not match

The adapter may know more than the paper account in failure scenarios, but the public order ledger must remain aligned with the account of record.

## Runtime Store Changes

### Shared order schema extensions

The shared `orders` table should remain self-describing for audit reads, without requiring joins into adapter-private tables.

This slice should extend the shared order schema to include:

- `remaining_quantity`
- `last_transition_at`
- `version`

Expected meaning:

- `remaining_quantity = requested_quantity - filled_quantity`
- `last_transition_at` records the latest lifecycle transition timestamp
- `version` is an integer optimistic-lock counter incremented on each successful order mutation

### New private table

Add `mock_live_orders` to `runtime.db`.

Purpose:

- store adapter-private lifecycle state without polluting the shared public audit schema
- keep recovery, query, and partial-fill simulation idempotent

Recommended shape:

```sql
CREATE TABLE IF NOT EXISTS mock_live_orders (
    order_id TEXT PRIMARY KEY,
    adapter_order_id TEXT,
    state_json TEXT NOT NULL
);
```

`order_id` should reference the shared public `orders.order_id`. Public audit fields such as quantity, remaining quantity, status, and fill price stay in `orders`; `mock_live_orders` exists only for adapter-private simulation state.

`state_json` carries mock-live-specific details such as:

- fill plan
- fault injection
- next step index
- planned fill time
- unknown-until marker
- cancel-requested marker
- last-applied-fill id
- unknown retries
- recovery exhausted flag
- exhausted reason

Example:

```json
{
  "fill_plan": [],
  "next_step_index": 0,
  "planned_fill_time": "2026-03-22T10:00:00Z",
  "fault_injection": null,
  "unknown_until": null,
  "cancel_requested": false,
  "last_applied_fill_id": 0,
  "unknown_retries": 0,
  "recovery_exhausted": false,
  "exhausted_reason": null
}
```

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
- `try_update_order_with_version(...)`

`advance_mock_live_order(...)` should atomically:

1. update shared `orders`
2. append the corresponding `order_events`
3. update `mock_live_orders`
4. increment shared `orders.version`

This keeps the public audit trail and the adapter-private state in sync.

## Adapter Semantics

### `submit_order`

`submit_order()` creates the adapter-private mock-live record and returns the current known state.

Expected first-slice behavior:

- normally returns `accepted`
- may return `submitted` when explicitly required by a simulation plan
- may return `rejected` for deterministic local validation failure
- may return `unknown` for simulated non-deterministic execution faults
- must not mutate the paper-trade account unless the initial simulation step explicitly includes a fill

### Incremental fill details

For real partial-fill accounting, adapter responses must expose both:

- order-level summary state
- increment-level fill details

Recommended shape:

```rust
pub struct FillDetails {
    pub fill_id: u64,
    pub fill_quantity: i64,
    pub fill_price: Decimal,
}
```

Extend both `OrderInitialResponse` and `OrderQueryResponse` with:

```rust
pub fill_details: Option<FillDetails>
```

Reason:

- `avg_fill_price` alone is not enough for correct incremental accounting
- the kernel must know the actual delta fill quantity and price for this specific step
- `fill_id` is required for idempotency across retry and recovery

### `query_order`

`query_order()` is the primary lifecycle-advance mechanism.

It may:

- keep the current state
- advance to `partially_filled`
- advance to `filled`
- advance `pending_cancel -> canceled`
- move into or out of `unknown`

### `cancel_order`

`cancel_order()` is allowed only for non-terminal orders.

Allowed source states:

- `pending_submit`
- `submitted`
- `accepted`
- `partially_filled`
- `unknown`

Required cancel semantics:

- a cancel request first transitions the order to `pending_cancel`
- a later direct adapter result or recovery query resolves that state to `canceled` or `unknown`

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
- treat adapter output as proposed execution truth, not immediate accounting truth
- apply account mutations through a kernel-owned fill-delta path, not from the adapter
- advance the shared order row only after the accounting side accepts the new fill delta
- record adapter identity from the concrete adapter boundary rather than hardcoding `"paper"`

Recommended internal helper:

- `apply_fill_delta(FillDeltaContext) -> Result<FillDeltaResult>`

`execute_once()` sequencing should be:

1. create the shared order row with `pending_submit`
2. call `submit_order()`
3. compare `old_filled_quantity = 0` with the adapter-returned `new_filled_quantity`
4. call `apply_fill_delta(...)`
5. if no delta was applied, update only lifecycle status
6. if delta was applied successfully, then update the shared order row and write fill events
7. if fill application fails, write a failure event and leave public filled quantity unchanged

The adapter returns raw execution state; the kernel remains the owner of account mutation and shared audit semantics.

### `recover_pending_orders()`

Replace the current placeholder with a real recovery pass.

First-slice recovery scope:

- scan non-terminal mock-live orders in states:
  `submitted`, `accepted`, `partially_filled`, `unknown`, `pending_cancel`
- read the current shared order `version`
- query the adapter for the latest status
- if status changes without new fill quantity, update the shared order via optimistic locking
- if new fill quantity is observed, call the same `apply_fill_delta(...)` helper used by `execute_once()`
- only after accounting succeeds may the shared order advance to the higher filled quantity
- if a version conflict occurs, reload once and retry; on repeated conflict, record and skip
- if `unknown` exceeds retry budget, append `recovery_exhausted` and set private exhaustion flags

Recovery summary should include at least:

- `scanned`
- `recovered`
- `unchanged`
- `failed`
- `skipped`

Field semantics:

- `failed` means **recovery attempts that failed to complete**, such as adapter query failures or unrecoverable version-conflict retries
- `failed` does **not** mean the underlying order reached a terminal failed status
- order truth remains encoded in public order status plus private mock-live exhaustion state

### `apply_fill_delta(...)`

This helper is the single accounting gateway for mock-live fills.

Recommended input:

```rust
pub struct FillDeltaContext {
    pub order_id: String,
    pub client_order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub requested_price: Decimal,
    pub old_filled_quantity: i64,
    pub new_filled_quantity: i64,
    pub fill_details: Option<FillDetails>,
    pub event_time: DateTime<Utc>,
}
```

Recommended output:

```rust
pub struct FillDeltaResult {
    pub applied: bool,
    pub delta_quantity: i64,
    pub trade_record_id: Option<String>,
}
```

Required semantics:

- if `new_filled_quantity <= old_filled_quantity`, return `applied = false`
- if `fill_details` is missing while a new fill must be applied, return an error
- if `fill_details.fill_id <= last_applied_fill_id`, return `applied = false`
- otherwise treat the delta as one real partial execution and call `TradeService::buy/sell`

Persistence requirements:

- update `MockLiveOrderState.last_applied_fill_id` only after successful accounting
- append `fill_applied` after successful accounting
- append `fill_apply_failed` when accounting fails

### Trade accounting compatibility

This slice keeps `TradeService` as the executor of paper-account mutations.

Meaning:

- each delta fill becomes one real `TradeRecord`
- `history_rows`, `fee_rows`, and `overview` naturally aggregate partial fills as multiple executions
- no order-level fee smoothing is performed in this slice

Price rules:

- use `fill_details.fill_price` for the incremental trade
- use `avg_fill_price` only as an order-level summary field

This is necessary because cumulative average price alone is not sufficient for correct per-delta accounting.

### Failure ordering

If `apply_fill_delta(...)` fails:

- do not advance the public order `filled_quantity`
- do not write `fill_applied`
- write `fill_apply_failed`
- return an error to the caller

This slice intentionally prefers account/ledger consistency over adapter truth visibility.

The shared order ledger, paper account, reporting, and risk view must stay aligned.

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
- submit may return `accepted` as the default initial state
- query advances to partial fill
- query advances to final fill
- cancel transitions to canceled
- unknown can recover to a known state
- cancel enters `pending_cancel` before final resolution
- repeated query/recovery does not duplicate fills

### Layer 2: kernel tests

Extend `tests/execution_kernel_test.rs`.

Cover:

- non-final submit results are persisted correctly
- order events become multi-step
- `sync_after_fill()` only runs when new fill quantity is observed
- `recover_pending_orders()` scans and advances recoverable orders
- version conflicts do not corrupt order state
- `unknown` retry exhaustion writes `recovery_exhausted` without changing public status away from `unknown`
- direct-path partial fill creates one real incremental paper-trade record
- recovery-path fill delta creates only the newly observed trade delta
- repeated recovery on the same `fill_id` does not double-apply accounting
- `fill_apply_failed` prevents public filled quantity from advancing

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
- paper account changes only after real delta fill application
- dedupe behavior still works
- `submit -> unknown -> recover -> filled` works end to end
- partial fill followed by cancel preserves already filled quantity and cancels only the remainder
- mock-live partial fills appear as multiple trade records rather than a single deferred final write

### Layer 5: reporting tests

Extend `tests/trade_reporting_test.rs`.

Cover:

- multiple delta fills from one mock-live order appear as multiple trade records
- `history_rows` shows each delta execution
- `fee_rows` reflects per-delta fee accumulation
- `overview` aggregates totals from those multiple executions

## Acceptance Criteria

This slice is complete when:

1. `strategy run --mode mock_live --code <CODE>` works
2. initial submit may return a non-final order status
3. shared `orders` rows remain self-describing and versioned
4. `runtime.db` stores shared audit rows plus `mock_live_orders`
5. paper account mutations happen only on successful delta fill application
6. `recover_pending_orders()` no longer returns a fixed empty summary
7. `paper` remains immediate-fill and backward compatible
8. repeated recovery does not double-apply the same fill delta

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
