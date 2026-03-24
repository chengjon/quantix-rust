# Phase 29A Strategy Paper Execution Kernel Design

**Date:** 2026-03-17
**Status:** Approved in-session
**Depends On:** Current green baseline (`master` @ `c941061`)

> This document is the source of truth for the next strategy slice: add a unified execution kernel for `strategy run` and deliver its first concrete adapter as `paper`, while preserving a clean path to future `live` execution.

---

## Goal

Build the smallest useful execution foundation that lets a strategy run once in `paper` mode without making `paper` a dead-end branch.

This phase must:

1. Make `quantix strategy run --mode paper` actually execute a strategy instead of printing an unsupported-mode message
2. Introduce a mode-agnostic execution kernel between strategy signals and order execution
3. Persist strategy-run, signal, order, and order-event records into a dedicated runtime SQLite database
4. Reuse the existing paper-trade and risk services behind stable interfaces
5. Keep the execution model compatible with later `live` work, including non-final `Unknown` order states and explicit idempotency

This phase must not attempt to deliver a real broker integration, daemonized automation, or a high-fidelity exchange simulator.

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. Run an existing strategy once in `paper` mode for one code
2. See whether the strategy emitted a signal and whether an order was attempted
3. Have risk rules either reject or allow that order before it reaches the paper execution adapter
4. Persist a durable audit trail for the run, signal, order, and execution result
5. Keep the resulting paper account state in the existing local trade storage

### Exact CLI boundary

Only implement:

```bash
quantix strategy run --name ma_cross --mode paper --code <CODE>
```

Rules:

- `strategy run` remains a single-shot command in this phase
- only `ma_cross` must support `paper`
- `--code` is required for the paper path if no safe default is already available from existing command behavior
- the command must:
  - load recent market data
  - run the strategy
  - translate the resulting signal through a default execution policy
  - perform pre-trade risk evaluation
  - submit through a paper execution adapter
  - persist runtime audit rows
  - print a concise execution summary
- `backtest` behavior must remain unchanged
- `live` remains explicitly unsupported

### Explicitly deferred

Phase 29A does not include:

- daemon or service mode
- WSL2 `systemd --user` integration for strategy execution
- multi-symbol or watchlist strategy execution
- mock-live execution
- real broker / exchange integration
- partial fills, delayed fills, or cancel flows in the paper adapter
- automatic recovery of pending or unknown orders
- Prometheus endpoints or dashboards
- cross-process concurrency control for paper-trade JSON
- rewriting existing trade or risk storage formats

## Approaches Considered

### Option A: Extend `run_strategy` with paper-specific handler logic only

Pros:

- smallest immediate diff
- fastest path to one green CLI command

Cons:

- bakes paper-specific control flow into CLI handlers
- does not create reusable live-ready boundaries
- mixes strategy driving, risk checks, idempotency, execution, and printing in one place

### Option B: Add a unified execution kernel and implement `paper` as the first adapter

Pros:

- creates a durable abstraction boundary now
- reuses existing paper-trade and risk services without large refactors
- lets later `mock_live` and `live` work swap adapters rather than rewrite the control flow
- introduces runtime audit storage and idempotency once

Cons:

- more files than a handler-only patch
- requires a small new storage subsystem

### Option C: Build full live-ready infrastructure before shipping paper

Pros:

- most complete architecture on day one
- fewer future model changes

Cons:

- too much scope for the current slice
- delays the first executable strategy path
- forces recovery, daemon, mock-live, and order-lifecycle work before the first real user outcome

## Recommendation

Choose **Option B**.

Add a unified execution kernel now, keep `strategy run` single-shot, and implement only a `PaperExecutionAdapter` in this phase.

This preserves short-term delivery while making later `live` work an adapter-and-recovery problem rather than a full rewrite of the strategy execution path.

## Architecture

### Layered model

The execution path is split into four layers:

1. `StrategyRuntime`
   - loads market data
   - builds or reuses the strategy instance
   - drives the strategy to a signal for the current run

2. `SignalTranslator`
   - wraps raw strategy output in a `SignalEnvelope`
   - applies a default `ExecutionPolicy`
   - converts the signal into an `OrderIntent` or a no-op decision

3. `ExecutionKernel`
   - owns run-level orchestration
   - enforces idempotency
   - performs risk evaluation
   - calls the execution adapter
   - writes runtime audit rows

4. `ExecutionAdapter`
   - adapter boundary for actual execution
   - `paper` is the first implementation
   - future `mock_live` and real `live` adapters plug in here

### Data ownership

- `paper_trade.json`
  - remains the source of truth for paper account state and trade records

- `risk_state.json`
  - remains the source of truth for risk rules and risk lock state

- `runtime.db`
  - becomes the source of truth for strategy-run audit history, order lifecycle events, and daemon/recovery checkpoints

This phase intentionally does not convert paper accounting into event sourcing.

### File boundaries

- `src/execution/mod.rs`
  - exports the execution subsystem

- `src/execution/models.rs`
  - shared execution data models
  - `SignalEnvelope`
  - `ExecutionPolicy`
  - `OrderIntent`
  - `OrderStatus`
  - `StrategyRunRecord`
  - `OrderRecord`
  - event payload enums/structs

- `src/execution/adapter.rs`
  - `ExecutionAdapter` trait
  - adapter response/query structs

- `src/execution/kernel.rs`
  - `ExecutionKernel`
  - run orchestration
  - pre-submit idempotency check
  - event and order record sequencing
  - placeholder `recover_pending_orders` entry point for later phases

- `src/execution/paper.rs`
  - `PaperExecutionAdapter`
  - wraps the existing paper trade service
  - maps adapter responses into unified order states

- `src/execution/runtime_store.rs`
  - runtime SQLite access
  - schema creation
  - run/signal/order/order-event persistence
  - dedupe and lookup helpers

- `src/strategy/runtime.rs`
  - strategy-driving helper for single-shot runs
  - isolates signal generation from CLI printing

- `src/cli/handlers.rs`
  - routes `strategy run --mode paper` into the new runtime/kernel path
  - preserves `backtest`

- `src/cli/mod.rs`
  - only adjust parsing if needed for stricter paper-mode input handling

- `tests/execution_kernel_test.rs`
  - kernel-level unit/integration tests

- `tests/execution_runtime_store_test.rs`
  - SQLite store tests

- `tests/strategy_paper_run_test.rs`
  - end-to-end CLI-style tests for paper mode

- `docs/USER_MANUAL.md`
  - update `strategy run` mode table and examples

## Core Models

### `SignalEnvelope`

`SignalEnvelope` is introduced in Phase 29A even if `metadata` is empty for all current strategies.

Purpose:

- keep the existing `Strategy` trait usable
- give the translator a stable input shape
- create a forward-compatible place for strategy-provided metadata later

Shape:

```rust
pub struct SignalEnvelope {
    pub signal: Signal,
    pub metadata_json: serde_json::Value,
}
```

Initial rule:

- current strategies emit `metadata_json = {}` through an adapter helper
- later phases may allow strategies to supply structured hints such as `target_weight`, `limit_price`, or explicit execution reasons

### `ExecutionPolicy`

Phase 29A uses a global default execution policy for paper mode.

Initial behavior:

- buy sizing: fixed cash amount
- sell sizing: sell all current position
- board-lot rule: round down to A-share 100-share lots
- price behavior: use the derived execution price plus configurable slippage

The translator may evolve later to combine global policy with strategy-provided metadata.

### `OrderIntent`

`OrderIntent` is the kernel-facing order request after signal translation and optional risk adjustment.

Minimum fields:

```rust
pub struct OrderIntent {
    pub symbol: String,
    pub side: OrderSide,
    pub requested_quantity: i64,
    pub requested_price: Decimal,
    pub order_type: OrderType,
    pub reason: String,
    pub policy_snapshot_json: serde_json::Value,
}
```

### Risk decision contract

The risk layer must be allowed to either reject or modify an order intent, even though Phase 29A only needs reject-or-pass behavior.

Use a contract like:

```rust
pub enum RiskDecision {
    Allow(OrderIntent),
    Reject { reason: String },
}
```

Phase 29A implementation rules:

- pre-trade risk may reject
- pre-trade risk does not yet rewrite size or price
- interface shape must still permit later rewrites

### `OrderStatus`

Order states are explicit and shared across adapters:

```text
PendingSubmit
Submitted
Accepted
PartiallyFilled
Filled
Canceled
Rejected
Unknown
```

Rules:

- `Filled`, `Canceled`, and `Rejected` are terminal states
- `Unknown` is not terminal
- `Unknown` means the system cannot currently prove the latest order state

Allowed transitions in this phase:

- `PendingSubmit -> Submitted | Rejected | Unknown`
- `Submitted -> Accepted | Rejected | Unknown`
- `Accepted -> PartiallyFilled | Filled | Canceled | Unknown`
- `PartiallyFilled -> Filled | Canceled | Unknown`
- `Unknown -> Submitted | Accepted | PartiallyFilled | Filled | Canceled | Rejected`

Phase 29A paper adapter rules:

- paper orders normally move from `PendingSubmit` to `Filled`
- paper does not intentionally emit `Unknown` in this phase
- the model still supports `Unknown` now so later adapters do not require a schema reset

## Runtime Database

### Path

- default: `~/.quantix/strategy/runtime.db`

### Tables

#### `strategy_runs`

Purpose:

- one logical strategy execution attempt per dedupe key

Fields:

- `run_id`
- `strategy_name`
- `mode`
- `trigger`
- `status`
- `symbol`
- `timeframe`
- `bar_end`
- `started_at`
- `finished_at`
- `metadata_json`

Indexes:

- unique: `(strategy_name, mode, symbol, timeframe, bar_end)`
- secondary: `status`
- secondary: `started_at`

#### `signal_events`

Purpose:

- immutable record of what the strategy emitted for a run

Fields:

- `event_id`
- `run_id`
- `strategy_name`
- `symbol`
- `signal`
- `ts`
- `payload_json`

Indexes:

- `run_id`
- `(symbol, ts)`

#### `orders`

Purpose:

- current materialized view of each logical order attempt, including pre-trade rejections

Fields:

- `order_id`
- `client_order_id`
- `run_id`
- `symbol`
- `side`
- `order_type`
- `requested_quantity`
- `requested_price`
- `filled_quantity`
- `avg_fill_price`
- `status`
- `adapter`
- `created_at`
- `updated_at`
- `payload_json`

Indexes:

- unique: `client_order_id`
- secondary: `run_id`
- secondary: `(symbol, status)`

#### `order_events`

Purpose:

- immutable order lifecycle history

Fields:

- `event_id`
- `order_id`
- `client_order_id`
- `event_type`
- `event_time`
- `details_json`

Indexes:

- `order_id`
- `(client_order_id, event_time)`

#### `runner_checkpoints`

Purpose:

- later daemon/recovery storage
- pre-created in Phase 29A to avoid another schema break

Fields:

- `checkpoint_id`
- `strategy_name`
- `mode`
- `symbol`
- `timeframe`
- `last_processed_bar`
- `last_run_id`
- `state_json`
- `updated_at`

Indexes:

- unique: `(strategy_name, mode, symbol, timeframe)`

### Schema notes

- `client_order_id` is the business-level idempotency key
- `order_id` is the persistent internal order row identifier
- JSON fields must remain small and diagnostic, not become a dump of whole account state
- Phase 29A assumes a single-process writer for paper execution; cross-process coordination is deferred

## Idempotency

### Run-level idempotency

Key:

- `(strategy_name, mode, symbol, timeframe, bar_end)`

Rule:

- the same strategy should not execute the same symbol and bar twice
- if a duplicate run is detected in a completed state, return the existing result
- if a duplicate run is detected in a non-final state, treat it as a recoverable conflict and report it clearly

### Order-level idempotency

Key:

- `client_order_id`

Format:

- `<run_id>_<symbol>_<sequence>`

Rules:

- generate `client_order_id` before submission
- check for an existing order row before calling the adapter
- if an order already exists:
  - terminal state: return the stored result instead of resubmitting
  - non-terminal state: return a recoverable conflict and do not resubmit
- Phase 29A does not auto-retry rejected orders

## Execution Flow

For a single `strategy run --mode paper` call:

1. Create or resume the runtime store
2. Determine the dedupe key using strategy, mode, symbol, timeframe, and current bar end
3. Create the `strategy_runs` row with `status = running`
4. Load market data and drive the strategy runtime
5. Wrap the raw strategy output into `SignalEnvelope`
6. Persist the `signal_events` row
7. Translate the envelope and policy into either:
   - no-op, or
   - `OrderIntent`
8. If no order is needed, mark the run successful and exit
9. Generate `client_order_id`
10. Check for an existing order by `client_order_id`
11. Load the paper account snapshot
12. Run pre-trade risk and get `RiskDecision`
13. If risk rejects:
   - insert the `orders` row with terminal state `Rejected`
   - append a `risk_rejected` order event with the rejection reason
   - mark the run completed without adapter submission
   - print the rejection summary
14. If risk allows, insert the `orders` row with initial state `PendingSubmit`
15. Call `ExecutionAdapter::submit_order`
16. Map the adapter response into `Submitted`, `Filled`, `Rejected`, or `Unknown`
17. Append one or more `order_events`
18. Update the `orders` row to its latest known state
19. If the order is filled in paper mode:
   - the paper adapter must already have updated `paper_trade.json` through the existing trade service
20. Run post-trade risk synchronization
21. Mark the run `success` or `failed`
22. Print a concise summary

### Ordering guarantees

Phase 29A does not claim cross-store atomicity.

Instead it guarantees:

- runtime audit rows are written in a consistent order
- paper account changes still go through the existing trade service
- if a later step fails, the runtime store captures enough state for diagnosis and future recovery work

### Recovery hook

`ExecutionKernel` should include a placeholder entry point such as:

```rust
pub async fn recover_pending_orders(&self) -> Result<RecoverySummary>
```

Phase 29A rules:

- the method may return a not-implemented or empty summary
- the method exists to lock the structural boundary now
- automatic recovery behavior starts in Phase 29C

## Adapter Contract

### `ExecutionAdapter`

Minimum interface:

```rust
#[async_trait]
pub trait ExecutionAdapter: Send + Sync {
    async fn submit_order(
        &self,
        request: AdapterOrderRequest,
    ) -> Result<OrderInitialResponse, AdapterError>;

    async fn query_order(
        &self,
        order_id: &str,
    ) -> Result<OrderQueryResponse, AdapterError>;

    async fn cancel_order(&self, order_id: &str) -> Result<(), AdapterError>;
}
```

Phase 29A rules:

- `submit_order` is required
- `query_order` and `cancel_order` exist now for compatibility, even if paper returns simple placeholder behavior
- `submit_order` must not assume the caller only needs terminal outcomes

### `PaperExecutionAdapter`

Responsibilities:

- adapt `OrderIntent` into the existing paper trade service request shape
- call buy/sell through the existing trade service
- return a unified adapter response

Behavior in this phase:

- successful paper execution may collapse immediately to a filled response
- no simulated delay
- no simulated partial fill
- no simulated `Unknown`
- no cancel behavior beyond a clear unsupported response

## Strategy Runtime

### Single-shot strategy execution

`StrategyRuntime` is responsible for:

- loading the required market data
- creating the strategy instance
- driving it to a current signal
- returning both signal and diagnostic context

Phase 29A simplifications:

- only `ma_cross`
- only one symbol
- daily bars
- no background polling

### `SignalTranslator`

Rules:

- `Hold` becomes a no-op execution result
- `Buy` becomes a buy-side `OrderIntent` using the default paper execution policy
- `Sell` becomes a sell-side `OrderIntent` using current paper position state
- translator may read current holdings when needed for sell sizing

## Error Handling

Hard errors:

- unsupported strategy name for paper mode
- insufficient market data
- runtime database initialization failure
- invalid order intent after policy translation
- pre-trade risk store failure
- paper trade store failure
- order persistence failure

Soft outcomes:

- `Hold` signal
- risk rejection
- duplicate run already completed

Deferred error classes:

- adapter timeout recovery
- unknown-order background reconciliation
- concurrent writer conflict resolution

## Concurrency And Consistency Assumptions

Phase 29A explicitly assumes:

- one foreground `strategy run --mode paper` process at a time per paper account
- no concurrent daemon worker
- no concurrent non-Quantix writers to `paper_trade.json`

This assumption must be documented in code comments where account snapshot loading happens.

Phase 29B or later may replace this with stronger file locking or a migrated storage layer if needed.

## Testing Strategy

### Phase 29A tests

1. Runtime store tests
   - schema bootstraps on empty path
   - unique run key blocks duplicates
   - unique `client_order_id` blocks duplicate logical orders

2. Translator tests
   - `Hold` produces no order
   - `Buy` uses fixed cash sizing and board-lot rounding
   - `Sell` becomes sell-all for the current position

3. Kernel tests
   - successful buy run writes:
     - one `strategy_runs` row
     - one `signal_events` row
     - one `orders` row
     - one or more `order_events`
   - risk rejection does not call the adapter
   - duplicate `client_order_id` does not resubmit
   - initial order state is `PendingSubmit`

4. Paper integration tests
   - `strategy run --mode paper` updates paper account JSON correctly
   - paper filled order updates post-trade risk status
   - paper mode preserves current `backtest` behavior

5. CLI tests
   - unsupported `live` still reports unsupported
   - unsupported strategies still fail clearly

### Future-phase tests already anticipated by this design

- delayed fills
- partial fills
- `Unknown` recovery
- query-based order reconciliation
- daemon checkpoint recovery

## Delivery Plan By Phase

### Phase 29A: Paper once

Deliver:

- unified execution module skeleton
- runtime SQLite store
- `SignalEnvelope`
- default execution policy
- paper adapter
- single-shot `strategy run --mode paper`
- run/signal/order/order-event persistence

Success standard:

- user can run `ma_cross` once in `paper`
- paper account updates correctly
- runtime audit rows are durable and queryable

### Phase 29B: Paper daemon and WSL2 operations

Deliver:

- daemon loop
- checkpoint persistence and replay boundary
- WSL2 `systemd --user` service management
- graceful shutdown
- environment-file support
- structured journald logging
- basic metrics instrumentation

### Phase 29C: Live-ready execution hardening

Deliver:

- `mock_live` adapter
- delayed and partial fills
- `Unknown` injection and recovery
- open-order scanning and query reconciliation
- simulated network fault handling
- basic account/order reconciliation scaffolding

### Phase 29D: First real live adapter

Deliver:

- one real broker or exchange adapter
- order submit/query/cancel
- fill and account reconciliation
- operational circuit breakers and human intervention hooks

## Documentation Updates

Update user-facing docs to state:

- `paper` is now supported for the defined Phase 29A boundary
- `live` remains in development
- `strategy run` in paper mode is single-shot only
- daemon/service automation arrives in the next phase

## Open Decisions Locked For This Phase

These decisions are intentionally fixed for Phase 29A and should not be reopened during implementation unless a blocker appears:

- keep existing paper trade and risk stores as authoritative
- use a separate runtime SQLite database
- introduce `SignalEnvelope` now
- introduce `Unknown` now, but do not exercise it in paper mode yet
- keep paper execution synchronous and immediately filled
- keep `live` unsupported

## Appendix: Minimal Summary Output

The paper run output should stay concise. A successful filled path should summarize:

- strategy name
- mode
- symbol
- signal
- order side / quantity / price
- final order status
- risk outcome
- run id

A `Hold` or risk-rejected path should still print:

- strategy name
- mode
- symbol
- signal
- no-op or rejection reason
- run id
