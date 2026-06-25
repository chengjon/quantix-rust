# ExecutionCapabilities Checklist Mode Semantics P0.7b

Date: 2026-06-25

Status: implemented

## Summary

P0.7b surfaces the P0.7a execution mode semantics bridge in the human-readable QMT promotion checklist.

The checklist already displayed the local `qmt_live` static `ExecutionCapabilities` identity. This slice adds the registered `qmt_live` risk notice and storage namespace to the same checklist so operators can see the channel meaning before following the live-promotion steps.

## Implemented Scope

- Extended `format_qmt_promotion_checklist` to display:
  - `qmt_live risk_notice`
  - `qmt_live storage_namespace`
- Reused the existing P0.7a helpers:
  - `risk_notice_for_execution_channel`
  - `storage_namespace_for_execution_channel`
- Added regression assertions to the existing checklist test.

## Preserved Boundary

This is a display-only checklist change.

No JSON status payload, bridge request, qmt_live submit/query/cancel flow, daemon behavior, reconciliation logic, storage schema, `OrderStatus`, or `ExecutionAdapter` signature changed.

## Non-Goals

- No upper-layer `mode == ...` migration.
- No qmt_live runtime probing or miniQMT startup self-check.
- No bridge protocol or response shape changes.
- No paper-immediate, paper-sim-lifecycle, or mock-live behavior changes.
- No `.unwrap()` cleanup.

## GitNexus Impact

Pre-edit impact check:

- `format_qmt_promotion_checklist` in `src/cli/handlers/execution_handler.rs`: LOW.
- Direct callers: 0.
- Affected processes: 0.
- Affected modules: 0.

Pre-commit `detect_changes` returned MEDIUM because it conservatively mapped the same source file hunk set to `execute_execution_bridge_status` and one indexed execution process. That symbol was separately impact-checked as LOW, and the exact source diff only touches `format_qmt_promotion_checklist`; it contains no `execute_execution_bridge_status` hunk or behavior change.

The GitNexus index reported stale metadata because the current commit differs from the indexed commit, but it was fresh for staged diff analysis.

## Verification

- RED: `cargo test test_format_qmt_promotion_checklist_surfaces_live_readiness_and_next_steps` failed on missing `qmt_live risk_notice` checklist output.
- GREEN: same focused test passed after adding the checklist mode-semantics lines.
- `cargo test test_format_qmt_promotion_checklist_surfaces_live_readiness_and_next_steps`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `git diff --check`
