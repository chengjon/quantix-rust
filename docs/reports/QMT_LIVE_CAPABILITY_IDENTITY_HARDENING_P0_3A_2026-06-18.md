# qmt_live Capability And Identity Hardening P0.3a

Date: 2026-06-18

Status: design / governance baseline

FUNCTION_TREE node: `Execution mode semantics hardening`

Governance node: `project-governance/P0.3a`

## Purpose

This report starts the next independent project after P0.2 execution mode
semantics hardening. P0.2 closed the low-intrusion semantic baseline for
`paper_immediate`, future `paper_sim_lifecycle`, and guarded `qmt_live`.
It explicitly did not harden qmt_live miniQMT runtime capabilities, broker
identity reconciliation, bridge compatibility, or status/error layering.

P0.3a is intentionally a design and governance slice. It records the current
qmt_live capability and identity baseline, GitNexus impact boundaries, staged
implementation order, and risk gates. It does not change production runtime
behavior.

## Scope Boundary

Allowed in P0.3a:

- Document the current qmt_live capability and identity baseline.
- Record GitNexus impact findings before any future source edits.
- Define phased implementation slices for qmt_live hardening.
- Update `FUNCTION_TREE.md` and governance records to point at this baseline.

Not allowed in P0.3a:

- No `src/` production source changes.
- No `tests/` changes.
- No `ExecutionAdapter` trait changes.
- No `OrderStatus`, query response, or storage schema changes.
- No qmt_live bridge protocol changes.
- No runtime behavior changes.
- No `.unwrap()` cleanup.

## Current Baseline

### qmt_live Truth Source

`qmt_live` is the guarded live execution path. It must treat miniQMT and the
broker cabinet exposed through the Windows Bridge as the truth source for live
order lifecycle state. Quantix may persist local runtime state, task receipts,
and reconciliation snapshots, but those records are not the broker truth source.

Current FUNCTION_TREE boundary:

- `qmt_live` is available only as a guarded miniQMT/bridge-backed path.
- Generic `live` remains incomplete and must not be treated as an available
  production trading mode.
- Windows Bridge remains an external boundary and does not own Quantix runtime
  state.

### Existing Identity Chain

The current code already contains a basic identity chain:

- `QmtTaskSubmitService` submits `/api/v1/task/execute` requests.
- Submit receipts carry task-level identity.
- Task result polling uses `/api/v1/task/result/{task_id}`.
- Identity validation covers local submission identity mismatch.
- qmt_live reconciliation can persist `external_order_id` from task results.
- qmt_live cancel can resolve `task_id -> external_order_id` before calling the
  compatible cancel endpoint.

Existing tests already cover parts of this chain:

- `tests/qmt_task_contract_test.rs`
- `tests/qmt_live_adapter_test.rs`
- `tests/qmt_live_reconciliation_test.rs`
- `src/cli/handlers/tests/strategy_execution.rs`
- `src/cli/handlers/tests/strategy_bridge.rs`

### Current Missing Hardening

The current baseline does not yet provide:

- miniQMT version compatibility gates.
- A structured broker capability model beyond the current bridge checks.
- Field-level compatibility handling for missing or variant task result fields.
- Startup-time capability/identity self-check reporting.
- Historical task/order reconciliation at process start.
- A fully structured qmt_live error taxonomy separating local validation,
  local risk rejection, bridge failure, broker rejection, unknown broker state,
  and manual-intervention state.
- A stable capability API exposed upward to strategy, risk, and CLI layers.

## GitNexus Impact Record

P0.3a ran GitNexus before authorizing any qmt_live work.

Observed LOW impact candidates:

- `Impl:src/execution/qmt_live_adapter.rs:QmtLiveExecutionAdapter`
  - Risk: LOW
  - Direct callers: 0 in the indexed graph
  - Affected processes: 0
- `Impl:src/execution/qmt_task_submit_service.rs:QmtTaskSubmitService`
  - Risk: LOW
  - Direct callers: 0 in the indexed graph
  - Affected processes: 0

Observed HIGH impact area:

- `src/execution/request_diagnostics.rs::build_bridge_qmt_capability_disabled_diagnostics`
  - Risk: HIGH
  - Direct callers: 2
  - Affected processes: 2
  - Affected modules: Execution, Handlers, Tests
  - Affected process roots include `execute_execution_command` and
    `execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch`.

Interpretation:

- Adapter-local and task-submit-service-local hardening can likely be proposed
  as small future implementation slices, but still requires fresh impact at the
  exact symbol level before edits.
- CLI diagnostics, request diagnostics, and handler-facing wording must be
  treated as higher-risk follow-up work, not mixed into the first runtime slice.
- The GitNexus index reports a known stale warning caused by commit/index
  mismatch. It remains fresh for staged diff detection, but future production
  slices should re-check index health before source edits.

## Proposed Implementation Stages

### P0.3b: qmt_live Capability Snapshot Seed

Goal:

- Add an internal qmt_live capability snapshot owned by the qmt_live execution
  area.

Candidate behavior:

- Capture bridge mode, order submit capability, query capability, cancel
  capability, contract version, and optional miniQMT version fields when
  available.
- Fail closed when required live-order-submit capability is unavailable.
- Treat missing optional fields as `Unknown`, not as a panic path.

Hard boundary:

- Do not change `ExecutionAdapter` yet.
- Do not expose a new generic capabilities trait yet.
- Do not change CLI diagnostic wording in this slice.

Primary candidate files:

- `src/execution/qmt_live_adapter.rs`
- `src/execution/qmt_task_submit_service.rs`
- qmt_live-specific tests only

### P0.3c: qmt_live Identity Reconciliation Tightening

Goal:

- Strengthen `task_id <-> external_order_id` persistence and reconciliation
  without changing broker semantics.

Candidate behavior:

- Preserve local task identity and broker external identity separately.
- Keep identity mismatch fail-closed.
- Ensure reconciliation can fill missing external identity from later task
  results.
- Keep manual-intervention fallback when required identity is absent.

Hard boundary:

- Do not add a background daemon.
- Do not change storage schema unless separately authorized.
- Do not treat local runtime rows as broker truth.

Primary candidate files:

- `src/execution/reconciliation.rs`
- `src/execution/runtime_store/orders.rs`
- `src/execution/qmt_task_submit_service.rs`
- qmt_live reconciliation tests

### P0.3d: qmt_live Error Taxonomy Seed

Goal:

- Define structured qmt_live error categories before broad handler integration.

Candidate categories:

- Local validation rejected before bridge submit.
- Local risk gate rejected before bridge submit.
- Bridge connectivity or protocol failure.
- Broker cabinet rejected the submitted task/order.
- Broker returned unknown or ambiguous state.
- Local manual intervention required because identity/state is incomplete.

Hard boundary:

- This should start as a qmt_live-local taxonomy, not a global CLI response
  rewrite.
- Do not edit `request_diagnostics` or CLI handlers until a separate impact
  slice is approved.

Primary candidate files:

- qmt_live-local execution files first
- request diagnostics only in a later HIGH-risk slice

### P0.3e: ExecutionCapabilities MVP

Goal:

- Introduce a narrow adapter capability abstraction after qmt_live-local
  capability semantics are stable.

Candidate API direction:

- Prefer one `capabilities()` return value over scattered boolean methods.
- Include capability source and confidence where useful.
- Keep paper, mock_live, and qmt_live capabilities explicit and testable.

Hard boundary:

- This is cross-cutting and must not be mixed with qmt_live runtime hardening.
- Requires fresh impact across all `ExecutionAdapter` implementations.

## Testing Strategy For Future Slices

Future production slices should use targeted tests before broad full-suite gates:

- qmt task contract tests for submit receipt and task result identity.
- qmt live adapter tests for capability missing, preview-only, submit, query,
  and cancel mapping.
- qmt live reconciliation tests for delayed external identity, rejection,
  unknown state, filled state, and manual intervention.
- CLI handler tests only when the approved slice touches handler output or
  request diagnostics.

Every implementation slice should still close with:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- relevant targeted tests
- full `cargo test` when the slice touches shared execution behavior
- GitNexus detect_changes before commit
- FUNCTION_TREE governance validate when FUNCTION_TREE/governance files change

## Risk Admission Rule

Do not mix LOW qmt_live-local runtime work with HIGH request diagnostics or CLI
handler work in the same PR.

If GitNexus reports HIGH or CRITICAL risk for a candidate symbol, stop and open
a dedicated design/approval step with:

- explicit affected process list,
- tailored tests,
- rollback plan,
- and manual review of live-trading failure semantics.

## Acceptance Boundary For P0.3a

P0.3a is complete when:

- This design report exists.
- `FUNCTION_TREE.md` references it as the qmt_live capability/identity hardening
  planning baseline.
- Governance records show P0.3a as docs/governance-only.
- GitNexus detect_changes confirms no production symbols changed.
