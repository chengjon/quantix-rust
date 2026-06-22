# qmt_live Reconciliation Query Refinement P0.4g

Date: 2026-06-22

Status: implementation slice

Branch: `feat/p0-4g-qmt-live-reconciliation-query-refinement`

## Summary

P0.4g refines the qmt_live reconciliation query path so orders with complete local task identity are queried with bridge-side identity validation.

The slice is intentionally narrow. It changes only `ReconciliationService::reconcile_qmt_live_order` and qmt_live reconciliation tests. It does not change metadata schema, bridge protocol, qmt_live gate behavior, diagnostics payloads, `OrderStatus`, `ExecutionAdapter`, submit/cancel behavior, or CLI wording.

## Implemented Contract

When a qmt_live order has:

- a non-empty `task_identity.task_id`
- a non-empty `task_identity.local_submission_id`

reconciliation now calls:

```text
QmtTaskSubmitService::query_task_result_once(task_id, OrderRecord.client_order_id, local_submission_id)
```

That keeps `OrderRecord.client_order_id` as the local identity source and lets the existing qmt task service reject bridge task results whose `client_order_id` or `local_submission_id` does not match local runtime identity.

When `local_submission_id` is missing, reconciliation preserves the P0.4f recovery path and still calls:

```text
QmtTaskSubmitService::query_task_result_by_task_id(task_id)
```

This keeps legacy or partial qmt_live runtime metadata recoverable by task id.

## Preserved Boundaries

- No qmt_live metadata schema change.
- No bridge protocol or bridge response shape change.
- No qmt_live gate behavior change.
- No diagnostics payload or CLI wording change.
- No `OrderStatus` change.
- No `ExecutionAdapter` trait change.
- No submit or cancel workflow rewrite.
- No background polling daemon.
- No `.unwrap()` cleanup resumed.

## GitNexus Impact

Pre-edit impact:

| Symbol | Risk | Direct callers | Affected processes | Affected modules |
|---|---:|---:|---:|---:|
| `ReconciliationService.reconcile_qmt_live_order#1` | LOW | 1 | 2 | 2 |

Affected processes reported in pre-edit impact:

- `qmt_live_reconciliation_keeps_pending_submit_when_task_result_is_pending`
- `qmt_live_reconciliation_marks_rejected_and_persists_reason`

Final GitNexus `detect_changes` result:

- risk: LOW
- changed files: 6
- affected processes: 0
- note: GitNexus reported the known stale-index warning with `fresh_for_staged_diff=true`

## TDD Evidence

RED:

```text
cargo test --test qmt_live_reconciliation_test qmt_live_reconciliation_marks_manual_intervention_when_task_result_identity_mismatches
```

The new regression test failed because the previous task-id-only reconciliation query accepted a completed bridge result whose `client_order_id` did not match the local order:

```text
assertion `left == right` failed
  left: Accepted
 right: PendingSubmit
```

GREEN:

```text
cargo test --test qmt_live_reconciliation_test qmt_live_reconciliation_marks_manual_intervention_when_task_result_identity_mismatches
```

The same test passed after reconciliation began using `query_task_result_once` when local `local_submission_id` is available.

## Verification

- `cargo test --test qmt_live_reconciliation_test qmt_live_reconciliation_marks_manual_intervention_when_task_result_identity_mismatches`
- `cargo test --test qmt_live_reconciliation_test`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `git diff --check`
- `function-tree scope-check`: changed files within active authorization
- `function-tree gate --verbose`: P0.4g implementation-ready, no blocker
- `function-tree validate`: passed
- GitNexus `detect_changes`: LOW risk, 0 affected processes

## Graphiti Status

Graphiti pre-read for P0.4g/P0.4f design context was attempted against `quantix_rust_main` and timed out twice with `Request timed out.`.

Graphiti backfill required if final P0.4g memory ingest also fails.

## Remaining Closeout Gates

- Commit implementation.
- FUNCTION_TREE node closeout transition to `closed`.
- PR CI and master CI, or documented failure.
- Graphiti memory completed, or local backfill record with `Graphiti backfill required`.
