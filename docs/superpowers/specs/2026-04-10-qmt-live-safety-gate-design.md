# QMT Live Safety Gate Design

**Date:** 2026-04-10

> Historical context:
> This dated design spec captures the repository state at the time it was written.
> Current project docs now describe guarded `qmt_live` real submission as shipped.
> Generic `live` target mode remains unimplemented.

## Context

The repository already contains three separate QMT-related paths:

1. QMT preview via `execution bridge qmt-preview`
2. Manual real-order submission via `execution bridge qmt-live`
3. Real-order execution through `QmtLiveExecutionAdapter` when an execution request targets `qmt_live`

At the time this spec was written, user-facing docs still described the public broker contract as preview-only, and `live` target mode remained unimplemented.

This creates a safety gap:

- the code can submit real orders
- but there is no hard runtime gate that first proves the Windows bridge is actually in `live` mode rather than `preview_only`

## Problem

Today, the real-order paths trust operator intent but do not require the bridge capability contract to confirm that QMT is running in live mode before submitting a real order.

That means a misconfigured bridge can still be reached through a code path that the docs present as operationally constrained.

## Non-Goals

- Do not change the public meaning of `live` target mode in this slice.
- Do not wire execution requests with `target_mode = live` into real order submission yet.
- Do not redefine `execution_request.completed` versus broker terminal order state in this slice.
- Do not refactor `live` vs `qmt_live` naming yet.

## Options Considered

### Option A: Add a hard bridge capability gate before any real order submission

Before real QMT submission:

- fetch `GET /api/v1/capabilities`
- require `qmt.enabled == true`
- require `qmt.mode == "live"`
- otherwise reject without placing the order

Pros:

- small behavioral change
- aligns code with current preview-only documentation contract
- low blast radius
- preserves future path to fully wire `live`

Cons:

- does not solve naming ambiguity between `live` and `qmt_live`
- does not unify request lifecycle and manual bridge submission

### Option B: Alias `live` to `qmt_live`

Allow `target_mode = live` to use the QMT live adapter.

Pros:

- unifies terminology faster

Cons:

- changes the current product contract
- too large for a safety patch
- increases risk before lifecycle semantics are clarified

### Option C: Remove manual `qmt-live` submission until lifecycle work is complete

Pros:

- strongest lock-down

Cons:

- removes an existing capability
- not necessary if a hard capability gate is added

## Decision

Choose **Option A**.

Add a mandatory bridge capability check to every real QMT submission path and reject real order submission unless the bridge explicitly reports `qmt.mode == "live"`.

## Desired Behavior

### Real-order paths covered

The following paths must refuse to submit a real order unless the bridge confirms live mode:

1. `QmtLiveExecutionAdapter.submit_order(...)`
2. `execute_execution_bridge_qmt_live(...)`

### Gate rules

Before submitting a real order:

1. Call `BridgeHttpClient.capabilities()`
2. Read `capabilities.qmt.enabled`
3. Read `capabilities.qmt.mode`
4. Reject if:
   - QMT is disabled
   - QMT mode is anything other than `"live"`

### Error semantics

Rejected-by-gate errors should:

- be explicit that real QMT submission is blocked
- mention the observed bridge mode when available
- make it clear that preview-only mode is not sufficient for real order placement

### Preview path behavior

`execution bridge qmt-preview` must remain unchanged.

Preview requests should continue working even when the bridge is in `preview_only` mode.

## File Scope

Primary implementation files:

- `src/execution/qmt_live_adapter.rs`
- `src/cli/handlers/mod.rs`

Related tests:

- `tests/qmt_bridge_preview_test.rs`
- add or extend QMT live safety tests near bridge / execution coverage

Reference models:

- `src/bridge/client.rs`
- `src/bridge/models.rs`

## Testing Strategy

Use TDD.

### Required failing tests first

1. Real adapter rejects submission when capabilities return `preview_only`
2. Manual `execution bridge qmt-live` path rejects submission when capabilities return `preview_only`
3. Existing preview path still succeeds under `preview_only`

### Expected passing behavior after implementation

1. Real adapter proceeds when capabilities return `live`
2. Manual `qmt-live` proceeds when capabilities return `live`
3. No real-order HTTP submission happens when the gate rejects

## Risks

- The manual CLI path and adapter path could diverge if they each implement their own gate differently.
- Tests could accidentally prove only mocked transport behavior rather than the actual capability decision.

## Mitigations

- Reuse a single small guard helper if the duplication is meaningful and stays local.
- Verify in tests that `/api/v1/broker/qmt/orders` is not hit when the gate rejects.

## Follow-Up

After this safety slice lands, the next execution-mainline design question is whether `live` should remain a separate target mode or become a validated alias of `qmt_live`.
