# qmt_live Diagnostics Wiring P0.4e

Date: 2026-06-22

Status: implementation slice

Branch: `feat/p0-4e-qmt-live-diagnostics-wiring`

## Summary

P0.4e wires the existing qmt_live compatibility gate failures into structured request diagnostics payload fields.

The slice is intentionally additive. It preserves the existing diagnostics `code`, `category`, `stage`, `summary`, `operator_action`, and `hint_command` values, while adding qmt_live-local metadata for the two approved HIGH-risk diagnostics builders:

- `build_bridge_qmt_capability_check_failed_diagnostics`
- `build_bridge_qmt_order_submit_capability_missing_diagnostics`

No qmt_live gate behavior, bridge protocol, response schema, storage schema, `OrderStatus`, `ExecutionAdapter`, task submit/query/cancel main flow, or error taxonomy shape was changed.

## Implemented Contract

The following fields are now included in the two qmt_live gate diagnostics payloads:

- `diagnostic_source`: stable local source marker, currently `qmt_live_gate`
- `qmt_live_failure_category`: local failure family for operator diagnostics
- `compatibility_requirement`: human-readable compatibility requirement tied to the failed gate

For bridge capability check failures:

- `qmt_live_failure_category`: `capability_check_failed`
- `compatibility_requirement`: `bridge /api/v1/capabilities returns qmt capability metadata`

For missing order submit support:

- `qmt_live_failure_category`: `missing_required_capability`
- `compatibility_requirement`: `bridge qmt.supports includes order_submit`

The order submit requirement reuses the existing `QMT_LIVE_SUBMIT_SUPPORT_REQUIREMENT` constant from `qmt_live_gate.rs` to avoid diagnostics text drifting from the gate source of truth.

## Preserved Boundaries

- No qmt_live gate semantics change.
- No `QmtLiveErrorCategory` taxonomy expansion.
- No bridge protocol or bridge response schema migration.
- No storage schema change.
- No identity metadata change.
- No `OrderStatus` change.
- No `ExecutionAdapter` trait change.
- No qmt_live submit/query/cancel main-flow change outside diagnostics payload construction.
- No CLI wording rewrite beyond persisted diagnostics test coverage.
- No miniQMT runtime probe or startup self-check implementation.
- No `.unwrap()` cleanup resumed.

## GitNexus Impact

Pre-edit GitNexus impact was run for both modified production diagnostics symbols.

| Symbol | Risk | Direct callers | Affected processes | Affected modules |
|---|---:|---:|---:|---:|
| `build_bridge_qmt_capability_check_failed_diagnostics` | HIGH | 2 | 2 | 3 |
| `build_bridge_qmt_order_submit_capability_missing_diagnostics` | HIGH | 1 | 2 | 3 |

Affected processes recorded by GitNexus:

- `execute_execution_command`
- `execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch`

The HIGH risk was expected from P0.4a and explicitly approved by the user before source edits. The implementation stayed confined to the approved diagnostics payload construction and handler regression-test paths.

## TDD Evidence

RED:

```text
cargo test request_diagnostics::tests::test_build_bridge_qmt_diagnostics_surface_structured_gate_metadata
```

The test failed because the existing diagnostics payload did not include the structured qmt_live fields:

```text
assertion `left == right` failed
  left: None
 right: Some("qmt_live_gate")
```

GREEN:

```text
cargo test request_diagnostics::tests::test_build_bridge_qmt_diagnostics_surface_structured_gate_metadata
```

The same test passed after adding the structured metadata fields to the two approved diagnostics builders.

## Verification

- `cargo test request_diagnostics::tests::test_build_bridge_qmt_diagnostics_surface_structured_gate_metadata`
- `cargo test --lib test_execute_execution_bridge_qmt_live_rejects_when_capability_check_fails`
- `cargo test --lib test_execute_execution_bridge_qmt_live_rejects_live_mode_without_order_submit_support`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `git diff --check`
- `function-tree scope-check`: 9 changed files within active authorization
- `function-tree gate --verbose`: P0.4e implementation-ready, no blocker
- `function-tree validate`: passed
- GitNexus `detect_changes`: LOW risk, 0 affected execution processes

GitNexus reported the known stale-index warning, but it resolved the current worktree diff and reported `fresh_for_staged_diff=true`.

## Remaining Closeout Gates

- FUNCTION_TREE node closeout transition to `closed`
- PR CI and master CI, or documented failure
- Graphiti memory completed, or local backfill record with `Graphiti backfill required`
