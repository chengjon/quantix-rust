# ExecutionCapabilities Preflight Mode Semantics P0.7c

Date: 2026-06-26

Status: implemented

## Summary

P0.7c surfaces the existing execution mode semantics in the human-readable `qmt_live` preflight report.

P0.7b already added `qmt_live` risk notice and storage namespace to the promotion checklist. This slice applies the same display-only semantics to the preflight text emitted by `format_qmt_live_preflight_report`, so the readiness section carries the same channel meaning without changing runtime behavior.

## Implemented Scope

- Extended `format_qmt_live_preflight_report` to display:
  - `risk_notice`
  - `storage_namespace`
- Reused the P0.7a helpers:
  - `risk_notice_for_execution_channel`
  - `storage_namespace_for_execution_channel`
- Added focused regression assertions to the existing preflight report test.

## Preserved Boundary

This is a human-readable CLI text change only.

The JSON `qmt_live_preflight` payload is intentionally unchanged. No bridge request, qmt_live submit/query/cancel flow, daemon behavior, reconciliation logic, storage schema, `OrderStatus`, or `ExecutionAdapter` signature changed.

## Non-Goals

- No JSON status response shape changes.
- No qmt_live runtime probing or miniQMT startup self-check.
- No bridge protocol or response shape changes.
- No paper-immediate, paper-sim-lifecycle, or mock-live behavior changes.
- No upper-layer `mode == ...` migration.
- No `.unwrap()` cleanup.

## GitNexus Impact

Pre-edit impact checks after refreshing the GitNexus index:

- `format_qmt_live_preflight_report`: LOW.
  - Direct callers: 0.
  - Affected processes: 0.
  - Affected modules: 0.
- `build_qmt_live_preflight_report`: LOW, recorded as fallback context if report construction needed to be touched.
  - Direct callers: 1.
  - Affected processes: 1.
  - Affected modules: 1.

The implementation only changes `format_qmt_live_preflight_report` and the existing preflight formatting test.

Pre-commit `detect_changes` returned MEDIUM because it conservatively mapped the same source file hunk set to `execute_execution_bridge_status` and two indexed execution processes. That symbol was separately impact-checked as LOW, and the exact source diff only touches `format_qmt_live_preflight_report`; it contains no `execute_execution_bridge_status` or `qmt_live_preflight_report_json` hunk.

## Verification

- RED: `cargo test test_qmt_live_preflight_report_marks_ready_and_surfaces_kill_switch_state` failed on missing `risk_notice` output.
- GREEN: same focused test passed after adding preflight mode-semantics lines.
- `cargo fmt --check`
- `git diff --check`
- FUNCTION_TREE `scope-check`
