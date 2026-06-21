# qmt_live Hardening Design P0.4a

Date: 2026-06-21

Status: design-only slice

Branch: `feat/p0-4a-qmt-live-hardening-design`

## Summary

P0.4a records the next `qmt_live` hardening plan after the P0.3 capability and identity baseline. It does not modify production code, bridge protocol, execution behavior, storage schema, response shapes, or `OrderStatus`.

The main design conclusion is that `qmt_live` hardening must remain split by risk boundary:

- LOW-risk local seeds may continue as small, independently testable slices.
- HIGH-risk gate, diagnostics, identity metadata, and runtime compatibility changes require separate explicit approval, focused tests, and full CI closure.
- No slice should mix bridge capability compatibility, gate behavior, diagnostics presentation, identity persistence, and reconciliation recovery.

## Scope Boundary

This slice is documentation and governance only.

Allowed scope:

- Record the current qmt_live baseline from P0.3a through P0.3f.
- Record GitNexus impact results for likely P0.4 hardening targets.
- Define staged P0.4 follow-up slices and their risk boundaries.
- Update FUNCTION_TREE governance references.

Non-goals:

- Do not modify `src` production code.
- Do not add miniQMT runtime probing or startup self-check implementation.
- Do not change bridge protocol, response shapes, storage schema, `OrderStatus`, or execution behavior.
- Do not modify request diagnostics or CLI output behavior.
- Do not resume `.unwrap()` cleanup.

## Current Baseline

P0.3 established a safer qmt_live foundation without changing core execution semantics:

- P0.3a documented the qmt_live capability and identity hardening baseline.
- P0.3b added `QmtLiveCapabilitySnapshot` as a qmt_live-local read-only capability seed.
- P0.3c tightened `task_id <-> external_order_id` reconciliation identity handling.
- P0.3d seeded qmt_live-local `QmtLiveErrorCategory`.
- P0.3e added static `ExecutionCapabilities` to the `ExecutionAdapter` trait.
- P0.3f surfaced read-only execution capability observability in qmt checklist output.

The preserved architectural boundary remains:

- `qmt_live` truth source is miniQMT / broker / bridge-reported state, not local simulation.
- `paper_immediate` remains local immediate-fill ledger behavior.
- `paper_sim_lifecycle` remains a future design direction and is not a runtime channel.
- Generic `live` remains intentionally incomplete and is not equivalent to `qmt_live`.

## Graphiti Read Note

Required Graphiti pre-read was attempted for:

`qmt_live capability identity hardening P0.4 miniQMT compatibility mapping reconciliation error taxonomy design decisions`

Group: `quantix_rust_main`

Result: timed out. This report therefore proceeds from local repository evidence, prior merged reports, FUNCTION_TREE state, and fresh GitNexus impact analysis. A post-merge memory write or local backfill is still required at closeout.

## GitNexus Impact Matrix

The following impact results were collected before this design was written. They are used to classify future P0.4 implementation slices.

| Candidate | File | Risk | Direct callers | Affected processes | Notes |
|---|---|---:|---:|---:|---|
| `BridgeCapabilitiesResponse` | `src/bridge/models.rs` | LOW | 0 | 0 | Bridge capability schema changes can still become contract-sensitive; keep in dedicated slice. |
| `BridgeQmtCapabilitySection` | `src/bridge/models.rs` | LOW | 0 | 0 | Suitable for additive descriptor design only, with bridge contract tests. |
| `BridgeHttpClient.capabilities` | `src/bridge/client.rs` | Not separately indexed | n/a | n/a | Treat endpoint changes through response models and status/gate callers. |
| `check_bridge_qmt_live_mode` | `src/execution/qmt_live_gate.rs` | HIGH | 2 | 2 | Impacts `execute_execution_command` and qmt_live runtime path; future gate changes require explicit approval. |
| `ensure_bridge_qmt_live_mode` | `src/execution/qmt_live_gate.rs` | LOW | 1 | 0 | Lower graph risk, but still gate-adjacent. Keep separate from diagnostics and protocol changes. |
| `QmtTaskSubmitService.submit_order#1` | `src/execution/qmt_task_submit_service.rs` | LOW | 2 | 0 | Real submit path; despite LOW graph risk, tests must use bridge mocks and no live broker dependency. |
| `QmtTaskSubmitService.qmt_live_capability_snapshot#0` | `src/execution/qmt_task_submit_service.rs` | LOW | 1 | 0 | Best candidate for local capability snapshot enrichment. |
| `QmtLiveExecutionAdapter.submit_order#1` | `src/execution/qmt_live_adapter.rs` | LOW | 3 | 0 | Real adapter boundary; keep behavior unchanged unless separately approved. |
| `ReconciliationService.reconcile_qmt_live_order#1` | `src/execution/reconciliation.rs` | LOW | 1 | 2 | Affects reconciliation tests; do not combine with identity schema changes. |
| `QmtLiveTaskIdentity` | `src/execution/models.rs` | HIGH | 3 | 2 | Identity schema changes are high-risk and need migration/no-data-loss assertions. |
| `QmtLiveRuntimeMetadata` | `src/execution/models.rs` | HIGH | 3 | 2 | Runtime metadata schema changes are high-risk and must be isolated. |
| `execute_execution_bridge_qmt_live` | `src/cli/handlers/execution_handler.rs` | LOW | 1 | 2 | CLI live path; do not change in local taxonomy or snapshot seed slices. |
| `execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch` | `src/cli/handlers/execution_handler.rs` | LOW | 1 | 1 | Runtime path remains sensitive despite LOW graph risk. |
| `QmtLiveErrorCategory` | `src/execution/qmt_task_submit_service.rs` | LOW | 0 | 0 | Safe candidate for local taxonomy enrichment before wiring. |
| `build_bridge_qmt_capability_check_failed_diagnostics` | `src/execution/request_diagnostics.rs` | HIGH | 2 | 2 | Diagnostics output changes should be separate from gate logic. |
| `build_bridge_qmt_order_submit_capability_missing_diagnostics` | `src/execution/request_diagnostics.rs` | HIGH | 1 | 2 | Same boundary as above. |

HIGH-risk affected process names recorded by GitNexus:

- `execute_execution_command`
- `execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch`

## Risk Classification

### LOW Entry Points

The following can be used for small future slices when the change is additive and local:

- `QmtTaskSubmitService.qmt_live_capability_snapshot#0`
- `QmtLiveErrorCategory`
- `QmtTaskSubmitService.submit_order#1`, only with behavior-preserving mocks/tests
- `QmtLiveExecutionAdapter.submit_order#1`, only with behavior-preserving mocks/tests
- `ReconciliationService.reconcile_qmt_live_order#1`, only when not combined with schema changes

### HIGH Entry Points

The following must not be touched without a new explicit approval:

- `check_bridge_qmt_live_mode`
- `QmtLiveTaskIdentity`
- `QmtLiveRuntimeMetadata`
- `build_bridge_qmt_capability_check_failed_diagnostics`
- `build_bridge_qmt_order_submit_capability_missing_diagnostics`

Any HIGH-risk slice must state the impacted processes, expected behavior changes, rollback assumptions, and tailored test plan before implementation.

## Proposed P0.4 Stages

### P0.4b: Capability Snapshot Compatibility Descriptor

Risk: LOW

Primary target:

- `QmtTaskSubmitService.qmt_live_capability_snapshot#0`

Goal:

- Add qmt_live-local compatibility descriptor fields derived from existing bridge `/capabilities` data.
- Preserve missing or unknown version/interface fields as `Unknown` or equivalent non-panicking values.

Non-goals:

- No qmt_live gate behavior change.
- No bridge protocol change.
- No CLI diagnostics change.
- No order submit behavior change.

Expected tests:

- `qmt_task_contract_test`
- bridge capability model/client tests
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- GitNexus detect_changes

### P0.4c: Error Taxonomy Local Enrichment

Risk: LOW

Primary target:

- `QmtLiveErrorCategory`

Goal:

- Extend local qmt_live error taxonomy only where current task-contract surfaces already expose enough information.
- Keep classification local to qmt_live task submission service.

Non-goals:

- No global error response rewrite.
- No request diagnostics wiring.
- No CLI output changes.
- No bridge response shape changes.

Expected tests:

- `qmt_task_contract_test`
- taxonomy-specific regression coverage
- full cargo gates
- GitNexus detect_changes

### P0.4d: qmt_live Gate Runtime Compatibility Check

Risk: HIGH

Primary target:

- `check_bridge_qmt_live_mode`

Goal:

- Add fail-closed compatibility checks for bridge/miniQMT readiness once the compatibility contract is stable.
- Preserve qmt_live as the only real broker submit path.

Non-goals:

- No diagnostics formatting rewrite in the same slice.
- No storage schema change.
- No identity metadata change.

Required approval:

- Explicit HIGH-risk approval before edits.

Expected tests:

- preview-only mode rejected
- live mode accepted
- capability missing rejected fail-closed
- ambiguous or unknown capability produces structured failure, not panic
- qmt_live handler tests
- full cargo gates
- GitNexus detect_changes

### P0.4e: Diagnostics Wiring

Risk: HIGH

Primary targets:

- `build_bridge_qmt_capability_check_failed_diagnostics`
- `build_bridge_qmt_order_submit_capability_missing_diagnostics`

Goal:

- Surface structured qmt_live compatibility and failure categories in diagnostics after gate behavior is stable.

Non-goals:

- No gate semantics change.
- No taxonomy expansion unless separately justified.
- No response/schema migration.

Required approval:

- Explicit HIGH-risk approval before edits.

Expected tests:

- request diagnostics formatting tests
- CLI handler tests
- existing qmt_live strategy request formatting tests
- full cargo gates
- GitNexus detect_changes

### P0.4f: Identity And Runtime Metadata Recovery

Risk: HIGH

Primary targets:

- `QmtLiveTaskIdentity`
- `QmtLiveRuntimeMetadata`

Goal:

- Design and implement durable recovery or reconciliation metadata improvements only after schema compatibility is specified.

Non-goals:

- No opportunistic field additions.
- No untested migration.
- No combined gate/diagnostics behavior change.

Required approval:

- Explicit HIGH-risk approval before edits.

Expected tests:

- serialization compatibility
- persistence round-trip
- missing-field compatibility
- reconciliation identity recovery
- no-data-loss assertions
- full cargo gates
- GitNexus detect_changes

### P0.4g: Reconciliation Polling / Query Refinement

Risk: LOW to MEDIUM depending on target

Primary target:

- `ReconciliationService.reconcile_qmt_live_order#1`

Goal:

- Improve reconciliation behavior only after identity schema decisions are stable.

Non-goals:

- No metadata schema change in the same slice.
- No gate or diagnostics changes.

Expected tests:

- existing qmt_live reconciliation tests
- targeted pending/rejected/unknown-state cases
- full cargo gates
- GitNexus detect_changes

## Design Rules For Future Slices

- Keep local adapter capabilities separate from bridge/broker capabilities.
- Keep qmt_live truth source bound to miniQMT / broker / bridge state.
- Treat missing bridge fields as unknown, not as a panic condition.
- Fail closed when live broker readiness is ambiguous.
- Do not change bridge protocol or response shapes without dedicated contract tests.
- Do not mix gate logic, diagnostics presentation, identity schema, and reconciliation behavior in one PR.
- Do not introduce background daemons for this hardening track.
- Do not reuse paper simulation lifecycle concepts to interpret broker state.
- Keep generic `live` distinct from guarded `qmt_live`.

## Acceptance Boundary For P0.4a

P0.4a is complete when:

- This report exists and records the GitNexus risk matrix.
- FUNCTION_TREE records P0.4a as a design-only qmt_live hardening plan.
- `git diff --check` passes.
- FUNCTION_TREE validation passes.
- GitNexus detect_changes reports only documentation/governance scope.
- PR CI and master CI pass, or a failure is documented.
- Graphiti memory is written and ingest reaches `completed`, or a local backfill document is committed with `Graphiti backfill required`.

