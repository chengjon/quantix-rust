# Execution Request Lifecycle Closure Design

**Date:** 2026-03-23
**Status:** Draft for user file review
**Depends On:** Phase 29B signal/request foundation and completed Phase 29C fill-delta accounting

> This document is the source of truth for the next strategy-execution slice: close the current `execution_request` half-loop by introducing a manual request consumer, durable request result updates, and request-level execution snapshots, while explicitly deferring daemon-driven execution automation.

---

## Goal

Build the smallest useful execution-request closure that turns the current:

`signal approve -> pending execution_request`

half-loop into:

`signal approve -> pending execution_request -> manual execute/cancel -> completed/failed/canceled`

This slice must:

1. allow a user to execute one pending request from the CLI
2. allow a user to cancel one pending request from the CLI
3. write execution results back into `runtime.db`
4. preserve the direct `strategy run` path unchanged
5. support both `paper` and `mock_live` as request target modes
6. keep request state separate from later order lifecycle state

This slice must not:

- auto-consume requests from the strategy daemon
- add auto-approval policies
- add a real live adapter
- add request retry scheduling
- conflate request completion with final order settlement

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. list pending execution requests
2. execute one request manually
3. see the request become `completed` or `failed`
4. cancel one pending request manually
5. inspect enough request result detail to troubleshoot what happened

### Exact CLI boundary

This slice adds:

```bash
quantix strategy request execute --request-id <ID>
quantix strategy request cancel --request-id <ID> [--reason <TEXT>]
```

This slice keeps:

```bash
quantix strategy request list [--status <STATUS>]
```

Rules:

- `strategy signal approve` still creates exactly one request
- `strategy request execute` is the only new consumer path
- `strategy daemon run` still does not execute requests automatically
- `target_mode=paper` and `target_mode=mock_live` are supported
- `target_mode=live` remains unsupported and must not silently succeed

## Problem Statement

Phase 29B introduced durable `signal` and `execution_request` objects, but intentionally stopped at:

- manual approval
- `request_status = pending`
- no consumer

That leaves three practical gaps:

1. request rows never reach terminal states unless they are superseded and canceled
2. operators cannot ask the system to execute an approved request
3. `request` rows do not currently store enough execution context to reproduce the approved intent deterministically later

The third gap is the most important.

If a request consumer only stores:

- `signal_id`
- `target_mode`
- `target_account`

then later execution has to reconstruct:

- market price
- execution policy
- intended order quantity

from current state, not approved state.

That is not acceptable for a durable request boundary.

## Approaches Considered

### Option A: Reconstruct execution intent at consume time from signal plus current state

Consumer loads the signal, re-derives `market_price`, current holdings, and execution policy at execution time, then calls the existing kernel path.

Pros:

- smallest diff
- reuses current execution kernel with minimal adaptation

Cons:

- request meaning changes over time
- sell quantity can drift if holdings changed after approval
- audit trail no longer proves what was actually approved

### Option B: Freeze an execution-intent snapshot at approval time and execute that snapshot later

Approval remains non-executing, but it computes and stores a durable execution snapshot that later consumers can execute without reinterpretation.

Pros:

- stable request semantics
- deterministic replay of approved intent
- request result rows become self-describing
- clean separation between approval and later execution

Cons:

- approval path must gather more context now
- requires one new request-consumer-oriented kernel entrypoint or adapter bridge

### Option C: Approval immediately executes and request becomes only a derived audit row

Pros:

- shortest path to user-visible action

Cons:

- breaks the Phase 29B `signal -> request -> execution` seam
- removes a useful operator checkpoint
- mixes approval with execution side effects

## Recommendation

Choose **Option B**.

Approval should stay non-mutating with respect to trading, but it must freeze the approved execution intent into the request payload. The request consumer should later execute that frozen snapshot and write back a durable terminal request result.

This keeps the current architecture coherent:

- `signal` expresses strategy output
- `approval` expresses operator intent
- `request` expresses executable handoff state
- `order` expresses actual execution lifecycle

## Architecture

### Preserved top-level split

Keep:

- `strategy daemon` produces signals
- `signal approve/reject` manages operator gating
- `execution request` is the durable handoff object
- `execution kernel` remains the owner of execution orchestration

Do not collapse these back into one step.

### New responsibilities

Add:

- request-execution snapshot generation during approval
- request terminal-state updates in `runtime_store`
- CLI request consumer commands
- request-result formatting in CLI output

### New execution entrypoint

Recommended direction:

- keep `ExecutionKernel::execute_once(...)` for direct strategy-run usage
- add a second request-oriented entrypoint that accepts a frozen request snapshot

For example:

```rust
ExecutionKernel::execute_request(...)
```

This avoids forcing request consumers to fake a full `SignalEnvelope` and avoids re-running `translate_signal(...)` from drifting runtime data.

## Request Snapshot Model

### Key rule

An approved request must capture the execution intent that was approved, not just the raw signal.

### Why signal-only is insufficient

For `buy`:

- quantity depends on policy and market price

For `sell`:

- quantity depends on held volume at approval time

If those are recomputed later, request meaning can drift.

### Recommended payload shape

Store this inside `execution_requests.payload_json`:

```json
{
  "execution_snapshot": {
    "strategy_name": "ma_cross",
    "strategy_instance_id": "ma_fast_5_slow_20",
    "symbol": "000001",
    "timeframe": "1d",
    "bar_end": "2026-03-23T07:00:00Z",
    "signal_value": "buy",
    "order_intent": {
      "side": "buy",
      "requested_quantity": 800,
      "requested_price": "12.34",
      "order_type": "market",
      "reason": "signal_buy"
    },
    "execution_policy": {
      "fixed_cash_per_buy": "10000",
      "slippage_bps": 0
    },
    "bar_source_id": "primary",
    "bar_source_fallback": false
  }
}
```

Notes:

- `order_intent` is the real frozen execution boundary
- `signal_value` is retained for audit readability
- `bar_source_*` remains useful for operator troubleshooting

## Approval Path Changes

### Approval remains non-executing

Approval still must not:

- submit orders
- mutate paper accounts
- change risk state

### Approval must now freeze intent

`approve_signal_and_create_request(...)` should:

1. load the signal row
2. load enough runtime context to compute a stable `OrderIntent`
3. store that snapshot in `execution_requests.payload_json`
4. create the request in `pending`

### Required input context

To compute a stable `OrderIntent`, approval needs:

- signal direction
- market price at signal bar
- execution policy
- held volume at approval time for sell intents

Recommended sources:

- signal metadata stores market/bar/source context
- approval path may read the current paper-trade store to freeze sell quantity

This is acceptable because approval still does not mutate trading state.

## Request State Model

Use the existing public enum unchanged:

- `pending`
- `completed`
- `failed`
- `canceled`

### Allowed transitions

- `pending -> completed`
- `pending -> failed`
- `pending -> canceled`

Terminal states:

- `completed`
- `failed`
- `canceled`

### Request completion semantics

`completed` means:

- the request was successfully consumed by the execution layer
- the execution kernel returned a valid execution result

It does not mean:

- the resulting order is fully settled
- the resulting order is terminal

Therefore:

- `paper` request usually ends as `completed + order_status=filled`
- `mock_live` request may end as `completed + order_status=accepted`

## Request Consumer Semantics

### `strategy request execute --request-id <ID>`

Required behavior:

1. load request by ID
2. require `request_status = pending`
3. parse the frozen execution snapshot
4. dispatch to the correct execution adapter for `target_mode`
5. run execution
6. on success, update request to `completed`
7. on failure, update request to `failed`

### Success payload

Append to request payload:

```json
{
  "execution_result": {
    "executed_at": "2026-03-23T09:30:00Z",
    "run_id": "run-...",
    "client_order_id": "run-..._000001_1",
    "order_status": "accepted",
    "adapter": "mock_live"
  }
}
```

### Failure payload

Append:

```json
{
  "execution_error": {
    "failed_at": "2026-03-23T09:31:00Z",
    "message": "..."
  }
}
```

### `strategy request cancel --request-id <ID> [--reason <TEXT>]`

Required behavior:

1. load request by ID
2. require `request_status = pending`
3. update request to `canceled`
4. write cancellation metadata into payload

This is request cancellation only. It does not call `cancel_order(...)` because no execution has happened yet.

## Store Changes

Add:

- `get_execution_request(request_id)`
- conditional request-state update helpers

Recommended helpers:

```rust
pub async fn get_execution_request(&self, request_id: &str) -> Result<Option<ExecutionRequestRecord>>;
pub async fn try_complete_execution_request(...expected_status: Pending...) -> Result<bool>;
pub async fn try_fail_execution_request(...expected_status: Pending...) -> Result<bool>;
pub async fn try_cancel_execution_request(...expected_status: Pending...) -> Result<bool>;
```

Rules:

- all request terminal updates must be conditional on `pending`
- repeated consume/cancel attempts must fail clearly instead of silently re-running

## CLI Changes

### Parser

Extend:

```bash
quantix strategy request execute --request-id <ID>
quantix strategy request cancel --request-id <ID> [--reason <TEXT>]
```

### Request list output

Keep `request list`, but improve result visibility.

Recommended line shape:

```text
<request_id> signal=<ID> target=<MODE>/<ACCOUNT> status=<STATUS> result=<SUMMARY>
```

Examples:

- `status=pending`
- `status=completed order_status=accepted client_order_id=...`
- `status=failed error=...`
- `status=canceled reason=...`

## Testing

### Store tests

Add coverage for:

- approval writes execution snapshot into request payload
- only `pending` requests can transition
- repeated complete/fail/cancel attempts are rejected

### CLI handler tests

Add coverage for:

- `request execute` on `paper`
- `request execute` on `mock_live`
- `request cancel`
- `request list` formatting for terminal request states

### Integration tests

Add coverage for:

- pending paper request executes to `completed`
- pending mock-live request executes to `completed` with non-final order status allowed
- failed request writes `execution_error`
- canceled request never calls execution

### Daemon/signal tests

Add coverage that signal metadata contains enough context to freeze request execution snapshots deterministically.

## Success Criteria

This slice is complete when a user can:

1. approve a signal
2. see exactly one pending request
3. execute that request manually
4. see the request move to `completed` or `failed`
5. cancel a different pending request manually
6. inspect request result summaries from the CLI
7. do all of the above without daemon auto-execution

## Fixed Decisions

These decisions are intentionally fixed for this slice and should not be reopened during implementation unless a blocker appears:

- keep request consumption manual
- keep request completion separate from order final settlement
- freeze execution intent at approval time
- do not auto-retry failed requests
- do not add execution daemon behavior yet
- keep direct `strategy run` unchanged
