# Execution Automation Closure Design

**Date:** 2026-03-23
**Status:** Draft for user file review
**Depends On:** Phase 29B strategy signal/request foundation, Phase 29C mock-live execution foundation, and execution-request lifecycle closure

> This document is the source of truth for the next strategy-execution slice: introduce an `execution daemon`, add minimal auto-approval policy support, and close the gap between durable requests and automated execution, while still deferring real live-broker connectivity.

---

## Goal

Build the smallest useful execution-automation slice that turns the current:

`signal -> manual approve -> pending request -> manual execute`

flow into:

`signal -> manual/auto approve -> pending request -> execution daemon consume -> terminal request result`

This slice must:

1. add an independent `execution daemon` role
2. keep `strategy daemon` focused on signal production
3. keep `execution_request` as the only execution handoff object
4. add `in_progress` request state so daemon/manual consumers cannot double-consume the same request
5. support a minimal auto-approval policy model
6. preserve the direct `strategy run` path

This slice must not:

- add a real live broker adapter
- add retry scheduling or a worker pool
- merge `strategy daemon` and `execution daemon` into one process role
- add complex policy routing or portfolio-level approval logic
- redefine order lifecycle semantics

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. start an execution daemon in foreground or one-shot mode
2. see a `pending` request move to `in_progress` and then `completed` or `failed`
3. optionally enable simple auto-approval so new signals create requests without manual approval
4. keep manual request execution as an operator fallback
5. understand from CLI/docs that request completion is distinct from final order settlement

### Exact CLI boundary

This slice adds:

```bash
quantix execution daemon run
quantix execution daemon run --once
quantix execution config init
quantix execution config show
```

Recommended future-compatible placeholders, even if implemented in a later patch:

```bash
quantix execution service install
quantix execution service-config show
quantix execution service-config set --quantix-bin <ABS_PATH> [--env-file <ABS_PATH>]
```

Rules:

- `strategy daemon` continues producing signals only
- `execution daemon` consumes only `pending execution_request`
- `strategy request execute` remains supported as a manual fallback path
- `live` request target mode may remain unsupported, but the daemon boundary must be adapter-ready

### Explicitly deferred

This slice does not include:

- multi-worker concurrency
- request retry backoff
- dead-letter queues
- advanced auto-approval rules
- live broker connectivity
- service mesh / remote orchestration

## Approaches Considered

### Option A: Independent execution daemon plus minimal auto-approval

Add a new `execution daemon` command family and let it consume `pending` requests serially. Keep auto-approval simple and controlled by config in the signal-producing layer.

Pros:

- clean separation between signal production and execution consumption
- reuses the request lifecycle work already completed
- smallest path to automated execution

Cons:

- single-process, single-worker only in the first slice
- requires one more daemon role to operate

### Option B: Put request consumption into the existing `strategy daemon`

Pros:

- fewer processes to reason about
- smaller initial CLI surface

Cons:

- collapses signal production and execution consumption back together
- weakens the durable `request` boundary
- makes future daemon/operator controls harder to reason about

### Option C: Auto-approval only, no execution daemon

Pros:

- smallest diff

Cons:

- leaves the main automation gap unresolved
- approved requests still need manual consumption

## Recommendation

Choose **Option A**.

The project already has a clean seam:

`strategy daemon -> signal -> approval -> execution_request -> execution kernel`

The right next step is to add a consumer on the request side, not to push execution concerns back into signal production.

## Architecture

### Preserved split

Keep the following roles distinct:

1. `strategy daemon`
   - reads bars
   - evaluates strategies
   - writes `signal`
   - optionally auto-approves

2. `signal approval`
   - transitions signals from `pending` to `approved/rejected`
   - creates `execution_request`

3. `execution daemon`
   - consumes `pending execution_request`
   - drives execution
   - updates request terminal state

4. `execution kernel`
   - owns actual execution orchestration
   - owns adapter/risk/fill-delta sequencing

### Why the split matters

The daemon that decides *what to do* should not also be the only component that decides *when the request is considered consumed*. That distinction is what makes the request layer valuable.

## Request State Model

### Public request statuses

Extend the current request lifecycle to:

- `pending`
- `in_progress`
- `completed`
- `failed`
- `canceled`

### Allowed transitions

- `pending -> in_progress`
- `in_progress -> completed`
- `in_progress -> failed`
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

- resulting order is fully settled
- resulting order is terminal

Therefore:

- `paper` request will usually end as `completed + order_status=filled`
- `mock_live` request may end as `completed + order_status=accepted`

### Why `in_progress` is required

Without `in_progress`, these two consumers can race:

- `execution daemon`
- manual `strategy request execute`

The first slice only needs single-worker serial execution, but it still needs a durable claim step so duplicate consumption cannot happen.

## Execution Daemon

### Command model

Recommended commands:

```bash
quantix execution daemon run
quantix execution daemon run --once
```

### Consumption loop

`run --once` must:

1. load execution config
2. fetch at most one `pending` request
3. conditionally mark it `in_progress`
4. execute it through the existing request-execution logic
5. write `completed` or `failed`
6. exit

`run` must:

1. loop forever
2. sleep for configured interval when no work is found
3. consume requests serially
4. continue running after request-level failures

### First-slice constraints

- single worker only
- one request at a time
- no retry queue
- no request prioritization

## Auto-Approval Policy

### Ownership

Auto-approval belongs to the signal-producing side, not the execution-consuming side.

That means:

- `strategy daemon` decides whether a newly generated signal should remain pending or be auto-approved
- `execution daemon` never decides signal approval

### First-slice policy model

Support only:

- `manual`
- `always`

Semantics:

- `manual`
  - current behavior
  - signal remains `approval_status = pending`
- `always`
  - strategy daemon immediately runs the existing approval transaction
  - request is created in `pending`

### Explicitly deferred policy logic

Do not add in the first slice:

- symbol allowlists
- strategy-specific rule tables
- risk-aware policy approval
- time-window policy approval
- account routing policies

## Execution Config

### New config file

Recommended path:

- `~/.quantix/execution/config.json`

Recommended shape:

```json
{
  "poll_interval_secs": 10,
  "max_requests_per_iteration": 1,
  "auto_approval": {
    "mode": "manual"
  }
}
```

First-slice implementation may keep:

- `poll_interval_secs`
- `max_requests_per_iteration`
- `auto_approval.mode`

Even if only one request is processed per loop today, `max_requests_per_iteration` is a useful explicit limit boundary for later expansion.

## Store Changes

The runtime store needs:

- `find_next_pending_execution_request()`
- `try_start_execution_request(request_id, executor_metadata, started_at)`
- existing terminal-state transitions extended from `in_progress`, not `pending`

Recommended semantics:

- `try_start_execution_request(...)` succeeds only when current status is `pending`
- `try_complete_execution_request(...)` and `try_fail_execution_request(...)` succeed only when current status is `in_progress`
- `try_cancel_execution_request(...)` succeeds only when current status is `pending`

### Supersede interaction

Current supersede behavior cancels pending requests for superseded signals.

After this slice:

- supersede must continue canceling only `pending` requests
- it must not touch `in_progress`
- it must not rewrite terminal request states

## Reuse Of Existing Request Execution

The daemon must not invent a second execution path.

Recommended implementation rule:

- manual `strategy request execute`
- daemon `execution daemon run`

must both reuse the same internal request-consumer logic.

That logic should:

- parse the frozen request snapshot
- choose adapter by `target_mode`
- call `ExecutionKernel::execute_request(...)`
- update request terminal state

## CLI / Output Semantics

### Request rows

Keep `strategy request list`, but make result summary explicit:

```text
<request_id> signal=<ID> target=<MODE>/<ACCOUNT> status=<STATUS> result=<SUMMARY>
```

Examples:

- `status=pending`
- `status=in_progress`
- `status=completed result=order_status=accepted client_order_id=...`
- `status=failed result=error=...`
- `status=canceled result=reason=...`

### Execution daemon output

Recommended one-line summaries:

- `execution daemon 未找到 pending request`
- `execution daemon consumed request=<ID> status=completed`
- `execution daemon consumed request=<ID> status=failed error=...`

## Testing

### Store tests

Cover:

- `pending -> in_progress`
- `in_progress -> completed`
- `in_progress -> failed`
- `pending -> canceled`
- supersede cancels only `pending`

### Daemon tests

Cover:

- `run --once` consumes one request
- `run --once` no-ops when no pending request exists
- daemon serially consumes requests
- daemon leaves later requests untouched when first request fails in one-shot mode

### Strategy-daemon tests

Cover:

- `auto_approval.mode = manual` leaves signal pending
- `auto_approval.mode = always` creates request automatically

### CLI tests

Cover:

- execution daemon parser
- execution config parser
- request list output with `in_progress`
- request list output with result summary

### Integration tests

Cover:

- signal -> auto-approved request -> execution daemon -> completed paper request
- signal -> auto-approved request -> execution daemon -> completed mock_live request with non-final order status

## Success Criteria

This slice is complete when a user can:

1. run `strategy daemon`
2. optionally enable `auto_approval=always`
3. run `execution daemon --once`
4. see a pending request move through `in_progress` into `completed` or `failed`
5. still manually execute requests when daemon is not running
6. do all of the above without introducing a real live adapter

## Fixed Decisions

These decisions are intentionally fixed for this slice and should not be reopened during implementation unless a blocker appears:

- keep `strategy daemon` and `execution daemon` as separate roles
- add `in_progress` before daemon automation
- keep auto-approval minimal (`manual|always`)
- keep execution consumption serial
- keep manual request execute as a supported fallback
- do not add live broker connectivity in this slice
