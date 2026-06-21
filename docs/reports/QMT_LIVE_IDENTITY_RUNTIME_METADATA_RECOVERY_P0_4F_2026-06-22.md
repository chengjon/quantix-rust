# qmt_live Identity Runtime Metadata Recovery P0.4f

Date: 2026-06-22

Status: implementation slice

Branch: `feat/p0-4f-qmt-live-identity-runtime-metadata`

## Summary

P0.4f hardens qmt_live runtime metadata compatibility and reconciliation identity recovery.

The slice is intentionally narrow. It keeps the existing storage schema, bridge protocol, response shapes, qmt_live gate behavior, diagnostics behavior, `OrderStatus`, and execution adapter contracts unchanged. The only production behavior change is that qmt_live runtime payloads can now tolerate legacy or partial `task_identity` JSON and can recover missing identity fields during reconciliation without dropping unrelated payload keys.

## Implemented Contract

`QmtLiveTaskIdentity` now deserializes missing string fields as compatibility defaults:

- missing `task_id` becomes an empty string
- missing `client_order_id` becomes an empty string
- missing `local_submission_id` becomes an empty string
- missing `external_order_id` remains `None`

`QmtLiveTaskIdentity::recover_missing_fields` fills only blank or missing identity fields:

- `task_id` is filled from the adapter task id when blank
- `client_order_id` is filled from the local `OrderRecord.client_order_id` when blank
- `local_submission_id` is filled from the completed bridge task result when blank
- `external_order_id` preserves the existing value, or fills from the completed bridge task result when absent

`QmtLiveRuntimeMetadata::recover_task_identity` applies the same recovery contract while preserving existing `last_query` and `reconciliation` metadata.

`ReconciliationService::qmt_live_payload_json` now uses the typed metadata recovery path before persisting qmt_live query results. It updates only `qmt_live.task_identity`, `qmt_live.last_query`, and `qmt_live.reconciliation`, while preserving unrelated keys such as operator notes or strategy payload fields.

## Preserved Boundaries

- No storage schema migration.
- No bridge protocol or bridge response shape change.
- No qmt_live readiness gate behavior change.
- No diagnostics payload or CLI handler change.
- No `OrderStatus` change.
- No `ExecutionAdapter` trait change.
- No qmt_live submit/query/cancel protocol change.
- No background reconciliation daemon.
- No `.unwrap()` cleanup resumed.
- No opportunistic field additions beyond the already-existing qmt_live runtime metadata structure.

## GitNexus Impact

P0.4f was explicitly approved as a HIGH-risk slice before source edits because the primary symbols are identity/runtime metadata types.

Impact review for the final implementation targets:

| Symbol | Risk | Direct callers | Affected processes | Affected modules |
|---|---:|---:|---:|---:|
| `QmtLiveTaskIdentity` | HIGH | 3 | 2 | 3 |
| `QmtLiveRuntimeMetadata` | HIGH | 3 | 2 | 3 |
| `ReconciliationService.qmt_live_payload_json` | LOW | 2 | 2 | 1 |

Affected processes reported by GitNexus for the metadata types:

- `execute_execution_command`
- `execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch`

Affected reconciliation test processes reported for `qmt_live_payload_json`:

- `qmt_live_reconciliation_keeps_pending_submit_when_task_result_is_pending`
- `qmt_live_reconciliation_marks_rejected_and_persists_reason`

Final GitNexus `detect_changes` result:

- risk: MEDIUM
- changed files: 8
- affected processes: 2
- rationale: expected qmt_live reconciliation payload path participation

GitNexus reported the known stale-index warning, with `fresh_for_staged_diff=true`.

## TDD Evidence

RED:

```text
cargo test --test execution_runtime_store_test qmt_live_task_identity_deserializes_missing_fields_as_compatibility_defaults
```

The test failed because legacy `task_identity` JSON without `local_submission_id` could not deserialize:

```text
Error("missing field `local_submission_id`", line: 0, column: 0)
```

GREEN:

```text
cargo test --test execution_runtime_store_test qmt_live_task_identity_deserializes_missing_fields_as_compatibility_defaults
```

The same test passed after adding serde defaults for the required identity string fields.

RED:

```text
cargo test --test qmt_live_reconciliation_test qmt_live_reconciliation_recovers_missing_identity_fields_from_completed_result
```

The test initially failed because reconciliation only persisted `external_order_id`; missing `client_order_id` and `local_submission_id` in `qmt_live.task_identity` remained unrecovered.

GREEN:

```text
cargo test --test qmt_live_reconciliation_test qmt_live_reconciliation_recovers_missing_identity_fields_from_completed_result
```

The final test asserts:

- existing `task_id` is preserved
- local `OrderRecord.client_order_id` is preserved as the local identity source
- `local_submission_id` is recovered from the completed bridge result
- `external_order_id` is recovered from the completed bridge result
- unrelated `qmt_live` payload keys are preserved
- `last_query` and `reconciliation` metadata are still written

## Verification

- `cargo test --test execution_runtime_store_test qmt_live_task_identity_deserializes_missing_fields_as_compatibility_defaults`
- `cargo test --test qmt_live_reconciliation_test qmt_live_reconciliation_recovers_missing_identity_fields_from_completed_result`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `git diff --check`
- `function-tree scope-check`: changed files within active authorization
- `function-tree gate --verbose`: P0.4f implementation-ready, no blocker
- `function-tree validate`: passed
- GitNexus `detect_changes`: MEDIUM risk, expected qmt_live reconciliation scope

## Remaining Closeout Gates

- Re-run lightweight gates after this report is added.
- Commit implementation.
- FUNCTION_TREE node closeout transition to `closed`.
- PR CI and master CI, or documented failure.
- Graphiti memory completed, or local backfill record with `Graphiti backfill required`.
