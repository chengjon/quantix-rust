# qmt_live Capability Descriptor P0.4b

Date: 2026-06-21

Status: implemented

Branch: `feat/p0-4b-qmt-live-capability-descriptor`

## Summary

P0.4b adds a qmt_live-local compatibility descriptor to `QmtLiveCapabilitySnapshot`.

The descriptor is derived only from the existing bridge `/capabilities` response fields already consumed by Quantix:

- `qmt.enabled`
- `qmt.mode`
- `qmt.supports`

This slice does not modify the bridge protocol, bridge response models, qmt_live gate behavior, request diagnostics, CLI output, storage schema, response shapes, `OrderStatus`, or execution behavior.

## Implemented Contract

New local types in `src/execution/qmt_task_submit_service.rs`:

- `QmtLiveCapabilityReadiness`
- `QmtLiveCompatibilityDescriptor`

`QmtLiveCapabilitySnapshot` now includes:

- `compatibility.readiness`
- `compatibility.missing_required_supports`

Current readiness values:

- `Ready`
- `Disabled`
- `NonLiveMode`
- `MissingOrderSubmit`

The existing `QmtLiveCapabilitySnapshot::is_live_order_submit_ready()` behavior is preserved by delegating to the descriptor readiness. A snapshot is still ready only when qmt is enabled, bridge qmt mode is `live`, and `qmt.supports` contains `order_submit`.

The existing version-like fields remain intentionally unknown:

- `bridge_contract_version: QmtLiveCapabilityValue::Unknown`
- `miniqmt_version: QmtLiveCapabilityValue::Unknown`

## Preserved Boundaries

- No bridge protocol changes.
- No bridge response model changes.
- No qmt_live gate behavior changes.
- No request diagnostics changes.
- No CLI handler or checklist changes.
- No storage schema changes.
- No response shape changes.
- No `OrderStatus` changes.
- No order submit/query/cancel behavior changes.
- No miniQMT runtime probing or startup self-check implementation.
- No `.unwrap()` cleanup resumed.

## GitNexus Impact

Fresh GitNexus impact was run before production edits.

| Symbol | Risk | Direct callers | Affected processes | Notes |
|---|---:|---:|---:|---|
| `QmtTaskSubmitService.qmt_live_capability_snapshot#0` | LOW | 1 | 0 | Direct impact limited to `tests/qmt_task_contract_test.rs`. |
| `QmtLiveCapabilitySnapshot` | LOW | 1 | 0 | Direct/indirect impact limited to tests. |

The bridge model symbols were not edited in this slice, so no bridge protocol impact was introduced.

## TDD Evidence

RED was run before production implementation:

```text
cargo test --test qmt_task_contract_test qmt_task_submit_service_marks_missing_order_submit_in_capability_descriptor
```

Expected RED result:

- `QmtLiveCapabilityReadiness` unresolved.
- `QmtLiveCapabilitySnapshot.compatibility` missing.

GREEN was then run after the minimal implementation:

```text
cargo test --test qmt_task_contract_test qmt_task_submit_service_marks_missing_order_submit_in_capability_descriptor
```

Result:

- 1 passed, 0 failed.

## Verification

The following checks passed:

- `cargo test --test qmt_task_contract_test`
- `cargo test --test qmt_live_adapter_test`
- `cargo fmt --check`
- `git diff --check`
- FUNCTION_TREE scope-check
- FUNCTION_TREE validate
- `cargo clippy -- -D warnings`
- `cargo test`
- GitNexus `detect_changes`: LOW risk, 0 affected processes

## Next Boundary

The next safe follow-up remains P0.4c local qmt_live error taxonomy enrichment, if still desired.

HIGH-risk work remains explicitly out of scope and requires separate approval:

- qmt_live gate runtime compatibility checks
- request diagnostics wiring
- identity / runtime metadata schema changes
- reconciliation recovery behavior changes
