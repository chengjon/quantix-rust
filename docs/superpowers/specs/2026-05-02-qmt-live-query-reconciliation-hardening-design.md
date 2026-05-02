# QMT Live Query Reconciliation Hardening Design

## Goal

Harden guarded `qmt_live` order recovery so pending or recovery-path orders can be reconciled by `task_id`, while keeping local runtime state as the single operator-facing truth source.

## Scope

This design is intentionally limited to the first engineering-closed-loop version of `qmt_live` query-based reconciliation hardening:

1. Persist `qmt_live` runtime identity into existing runtime order metadata.
2. Extend reconciliation so qualifying `qmt_live` orders can query bridge task results by `task_id`.
3. Persist the latest query summary and reconciliation outcome back into runtime order metadata.
4. Expose that recovery context through existing CLI/detail/diagnostic surfaces.

## Non-Goals

- No generic `target_mode=live` implementation.
- No callback-based broker workflow integration.
- No automatic cancel-recovery workflow.
- No query-history timeline persistence.
- No new SQLite tables or schema expansion for `qmt_live` runtime state.
- No attempt to fully recover detailed fill accounting for partial executions in this first pass.

## Current Baseline

The current mainline baseline for this design is `origin/master@562fe84`.

At this baseline:

- `QmtTaskSubmitService` already supports `query_task_result_by_task_id(...)`.
- `QmtLiveExecutionAdapter` already uses task-receipt / task-result semantics for submit/query.
- runtime orders already have a flexible `payload_json` storage field.
- `ReconciliationService` is still generic and does not yet implement a dedicated `qmt_live task_id -> query -> state repair` loop.

## Architectural Constraints

This work follows four hard boundaries:

1. `orders` remain the local truth source for operator-facing state.
2. bridge task results are external facts queried only by reconciliation logic.
3. CLI/detail surfaces must read persisted runtime facts, not query bridge directly.
4. this change must remain a low-risk increment on top of existing runtime storage and execution contracts.

That yields the following responsibility split:

- `src/execution/qmt_task_submit_service.rs`
  - submit/query contract mapping only
  - no local state persistence policy
- `src/execution/runtime_store/orders.rs`
  - persist and update `qmt_live` runtime metadata inside `OrderRecord.payload_json`
- `src/execution/reconciliation.rs`
  - adapter-specific `qmt_live` recovery logic
  - decides whether to preserve recoverable state or advance to a terminal/clearer state
- CLI/detail/diagnostic renderers
  - display already-persisted reconciliation facts
  - do not bypass runtime storage to call bridge

## Data Model

No schema change is introduced.

`qmt_live` runtime metadata will be stored under a stable namespace in `OrderRecord.payload_json`:

```json
{
  "qmt_live": {
    "task_identity": {
      "task_id": "task-123",
      "client_order_id": "client-123",
      "local_submission_id": "local-123",
      "external_order_id": "broker-456"
    },
    "last_query": {
      "latest_status": "accepted",
      "filled_quantity": 100,
      "avg_fill_price": "10.50",
      "broker_event_type": "Acknowledgement",
      "rejection_reason": null,
      "updated_at": "2026-05-02T14:00:00Z"
    },
    "reconciliation": {
      "last_action": "state_updated",
      "last_error": null,
      "last_attempt_at": "2026-05-02T14:00:00Z"
    }
  }
}
```

### `task_identity`

- `task_id`
  - required for all successfully submitted `qmt_live` orders
- `client_order_id`
  - required mirror of the local order identity
- `local_submission_id`
  - required to preserve bridge identity-validation context
- `external_order_id`
  - optional until query/cancel paths resolve it

### `last_query`

- stores only the latest query summary
- uses local `OrderStatus` string semantics for `latest_status`
- preserves the latest observed fill snapshot through `filled_quantity` and `avg_fill_price`
- keeps `broker_event_type` and `rejection_reason` for operator-facing observability
- `updated_at` refers to the summary freshness timestamp, not necessarily the order transition time

### `reconciliation`

- `last_action`
  - must align with `ReconciliationAction`
- `last_error`
  - stores the latest bridge/query/reconciliation failure summary
- `last_attempt_at`
  - stores the latest reconciliation attempt time

### Explicitly Rejected Data Shapes

- no raw full bridge payload snapshot in first pass
- no query-history array
- no top-level schema expansion
- no separate `qmt_live_order_runtime` table

## Reconciliation Eligibility

Only the following orders enter the query-based `qmt_live` reconciliation path:

- `adapter == "qmt_live"`
- `payload_json.qmt_live.task_identity.task_id` exists
- local order status is one of:
  - `PendingSubmit`
  - `Submitted`
  - `Accepted`
  - `Unknown`

First-pass reconciliation does not actively target:

- `Filled`
- `Canceled`
- `Rejected`
- `PendingCancel`
- `PartiallyFilled`

Those either already represent clear states or require a broader workflow and accounting design.

`PartiallyFilled` remains a required visibility state even though it is not part of the first-pass automatic query-based recovery set. CLI/detail surfaces must not silently hide it.

## Reconciliation State Rules

### 1. Bridge still reports pending or non-terminal task state

Behavior:

- preserve the current recoverable local state
- update `qmt_live.last_query`
- write reconciliation attempt metadata
- do not force a terminal state

Intended operator meaning:

- the order is still recoverable
- later reconciliation runs may converge it further

### 2. Bridge returns completed + `Acknowledgement`

Behavior:

- advance local status to `Accepted`
- persist latest query summary
- set reconciliation action to `state_updated`

### 3. Bridge returns completed + `Reject`

Behavior:

- advance local status to `Rejected`
- persist `rejection_reason`
- persist latest query summary
- set reconciliation action to `state_updated`

### 4. Bridge returns completed + `Execution`

Behavior:

- advance local status to `Filled`
- persist latest query summary
- do not attempt full fill-accounting reconstruction in first pass

Rationale:

- the contract is stable enough to recognize execution as a clear terminal fact
- precise fill-accounting reconstruction is deferred to a later, more focused workflow hardening pass, but the latest observed `filled_quantity` and `avg_fill_price` must still be preserved in `last_query`

### 5. Bridge task result path returns failure

Behavior:

- do not map all failures to a fake local terminal state
- only move to `Rejected` when the bridge fact is clearly a business rejection
- otherwise preserve current state or fall back to `Unknown`
- persist `last_error`

Examples that should remain non-terminal or ambiguous:

- timeout
- unavailable bridge
- invalid result payload
- protocol mismatch
- identity mismatch
- completed/failed result missing expected fields

### 6. `qmt_live` order missing `task_id`

Behavior:

- do not panic
- do not attempt query-based recovery
- preserve local state
- persist a clear reconciliation error summary stating that task-id-based recovery is unavailable
- mark reconciliation action as `manual_intervention`

## Failure Strategy

This design adopts the gradual recovery strategy:

- query failure does not automatically become terminal order failure
- recoverable local states remain recoverable unless explicit bridge facts justify advancement
- latest query summary and reconciliation diagnostics are updated even when the order status is not

This avoids false certainty and keeps the system aligned with the guarded-`qmt_live` boundary.

## CLI and Diagnostic Observability

CLI must present persisted runtime facts only.

### Summary/List Views

For recoverable `qmt_live` orders, list/status outputs should expose a compact recovery summary:

- `adapter`
- local `order_status`
- whether `task_id` exists
- `last_query.latest_status`
- `last_query.broker_event_type`
- `reconciliation.last_action`
- `reconciliation.last_attempt_at`

Orders missing `task_id` must be clearly labeled as unrecoverable by automatic query-based reconciliation.

`PartiallyFilled` orders that are outside the first-pass automatic recovery path must still remain visible in operator-facing summaries as non-terminal states requiring attention.

### Detail Views

Order/request/execution detail surfaces should expose full recovery context:

- `task_identity.task_id`
- `task_identity.client_order_id`
- `task_identity.local_submission_id`
- `task_identity.external_order_id`
- `last_query.latest_status`
- `last_query.broker_event_type`
- `last_query.rejection_reason`
- `last_query.updated_at`
- `reconciliation.last_action`
- `reconciliation.last_error`
- `reconciliation.last_attempt_at`

### Required Wording Discipline

User-facing wording must distinguish:

1. waiting for later convergence
   - not a failure
2. bridge query failure
   - not automatically a broker reject
3. missing recovery identity
   - automatic reconciliation unavailable because `task_id` metadata is absent

### Explicit CLI Non-Goals

- no new direct bridge query command in first pass
- no CLI-side bridge polling
- no recovery timeline UI

## Storage Update Strategy

This work requires extending existing order persistence/update logic so `payload_json.qmt_live` can be:

- written at submit time for `task_identity`
- amended during reconciliation for `last_query`
- amended during reconciliation for `reconciliation`

This implies the runtime store needs a targeted payload-update path that can safely modify order payload JSON without destroying unrelated payload fields.

### Storage Update Path

The first-pass implementation should use a typed Rust-level read-modify-write path rather than SQLite JSON mutation functions.

Recommended shape:

- add a typed runtime-store helper dedicated to `qmt_live` order metadata updates
- fetch the current order record
- deserialize / mutate only the `payload_json.qmt_live` namespace
- preserve all unrelated `payload_json` fields
- write the full updated payload back through a dedicated order-payload update method

This design intentionally does not introduce a generic `patch_payload_json(order_id, path, value)` API in first pass. The targeted typed helper keeps the mutation surface narrower and lowers accidental drift risk while the data shape is still stabilizing.

### Concurrency Expectation

First-pass concurrency semantics are last-writer-wins for `qmt_live.last_query` and `qmt_live.reconciliation`.

That is acceptable for this pass because:

- reconciliation stores the latest observed summary, not an append-only audit history
- the design does not claim precise multi-writer merge semantics
- later workflow hardening can revisit stronger coordination if overlapping reconciliation runs become a demonstrated problem

## Test Strategy

Use focused regression coverage rather than broad end-to-end expansion.

### Core Reconciliation Tests

Must cover:

1. `PendingSubmit` + non-terminal query result
   - local state preserved
   - query summary updated
2. completed + `Acknowledgement`
   - local state becomes `Accepted`
3. completed + `Reject`
   - local state becomes `Rejected`
   - rejection reason persisted
4. completed + `Execution`
   - local state becomes `Filled`
5. timeout / unavailable / invalid result
   - terminal state not forced
   - reconciliation error persisted
6. missing `task_id`
   - no panic
   - no bridge query
   - manual-intervention summary persisted

### Runtime Metadata Persistence Tests

Must cover:

- submit-time identity persistence
- reconciliation-time `last_query` persistence
- reconciliation-time `last_action` / `last_error` persistence
- unrelated payload JSON keys remain intact

### CLI/Display Tests

Must cover:

- recoverable `qmt_live` orders show task-id and reconciliation summaries
- missing-task-id path shows explicit unrecoverable messaging
- query failure does not display as broker rejection unless the bridge fact actually says reject

## Expected File Surface

Likely production files:

- `src/execution/reconciliation.rs`
- `src/execution/runtime_store/orders.rs`
- `src/execution/models.rs`
- `src/execution/qmt_task_submit_service.rs`
- existing CLI/detail renderers for request/order/execution detail output

Likely test files:

- focused reconciliation tests
- focused runtime-store order metadata tests
- focused CLI/detail tests around `qmt_live` recovery visibility

## Acceptance Commands

First-pass acceptance should stay narrow:

```bash
cargo test qmt_live -- --nocapture
cargo test reconciliation -- --nocapture
cargo test --test repo_hygiene_test -- --nocapture
```

If test placement requires more precise commands, implementation may substitute exact suite names while preserving the same narrow verification intent.

When these commands rely on Rust test-name substring filters, the corresponding test names should intentionally include `qmt_live` or `reconciliation` so the acceptance commands remain meaningful and stable.

## Success Criteria

This design is considered successfully implemented only when:

1. `qmt_live` orders with persisted `task_id` can be reconciled by query result.
2. non-terminal or failing queries do not create false terminal certainty.
3. latest query summary and reconciliation outcome are persisted into runtime order metadata.
4. operator-facing detail/list surfaces can explain why an order did or did not converge.
5. focused verification passes without expanding the work into unrelated cleanup.
