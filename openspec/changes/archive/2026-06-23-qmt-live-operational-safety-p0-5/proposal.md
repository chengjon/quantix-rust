# qmt_live Operational Safety P0.5

## Why

`qmt_live` has already been hardened through capability, identity, diagnostics, metadata recovery, and reconciliation query refinement slices. The remaining readiness gap is operational rather than purely structural: an operator needs a repeatable way to decide whether the miniQMT bridge path is safe to use, capture evidence before and after a canary run, stop real submissions immediately, audit every real-money action, and surface manual-intervention cases.

Without this change, the code can be green while the real-money workflow still depends on ad-hoc judgment:

- Bridge and miniQMT readiness is checked through scattered commands instead of a single preflight gate.
- Canary execution has no standard evidence artifact.
- Existing kill-switch behavior is implemented, but not yet treated as a required qmt_live release-readiness gate.
- qmt_live submit, query, reconciliation, and manual-intervention records are not yet packaged as an operator-facing safety loop.

This change makes qmt_live usable only through an explicit operational safety path before any broader live trading claim is made.

## What Changes

- Add an active OpenSpec change under `openspec/changes/qmt-live-operational-safety-p0-5/`.
- Govern the P0.5 qmt_live operational safety stage as a sequence of small implementation slices:
  - P0.5a qmt_live preflight doctor.
  - P0.5b qmt_live canary runbook and evidence artifact.
  - P0.5c qmt_live kill-switch acceptance and operator documentation.
  - P0.5d qmt_live audit evidence closure.
  - P0.5e qmt_live manual-intervention report.
- Prefer reusing or extending existing `quantix execution qmt status --checklist`, `quantix execution qmt preview`, `quantix execution qmt live`, `quantix execution qmt query`, `quantix execution qmt cancel`, and `quantix safety kill-switch` surfaces before introducing new command namespaces.
- Preserve the current `paper_immediate`, `mock_live`, and qmt_live execution semantics unless a later slice explicitly receives approval for a behavior change.

## Capabilities

### New Capabilities

- `qmt-live-operational-safety`: qmt_live real-money readiness SHALL be governed by preflight, canary evidence, kill-switch verification, audit evidence, and manual-intervention reporting.

### Modified Capabilities

- `execution/`: qmt_live operational use becomes explicitly gated by operator-facing safety evidence. The change does not by itself alter submit/query/cancel semantics.

## Impact

- Adds OpenSpec proposal, design, task list, and spec delta files.
- Adds a FUNCTION_TREE registration entry for the active P0.5 OpenSpec change.
- Does not change runtime code by itself.
- Future implementation may touch, after fresh GitNexus impact:
  - `src/cli/commands/trade.rs`
  - `src/cli/commands/safety.rs`
  - `src/cli/handlers/trade_handler.rs` or the current execution/QMT handler owner
  - `src/cli/handlers/strategy_handler.rs`
  - `src/execution/qmt_live_gate.rs`
  - `src/execution/qmt_task_submit_service.rs`
  - `src/execution/reconciliation.rs`
  - `src/safety/kill_switch.rs`
  - `tests/`
  - `docs/operations/`
  - `docs/reports/evidence/`
- Requires a real or test-owned Windows Bridge and miniQMT environment for canary closure. Secrets, account identifiers, raw broker logs, and host-specific credentials MUST NOT be committed.

## Non-Goals

- Do not implement generic broker live execution.
- Do not replace miniQMT as the qmt_live source of truth.
- Do not change bridge protocol, response shape, storage schema, `OrderStatus`, or `ExecutionAdapter` in the same slice.
- Do not implement `paper_sim_lifecycle`.
- Do not modify `paper_immediate` behavior.
- Do not resume `.unwrap()` cleanup.
- Do not add background daemons.
- Do not treat Graphiti memory as runtime state or release evidence.
