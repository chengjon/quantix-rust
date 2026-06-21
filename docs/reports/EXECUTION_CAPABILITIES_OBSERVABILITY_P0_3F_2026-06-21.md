# P0.3f ExecutionCapabilities Read-Only Observability

Date: 2026-06-21

## Summary

P0.3f exposes the static `ExecutionCapabilities` MVP through the existing QMT promotion checklist. The change is read-only and operator-facing: `quantix execution bridge status --checklist` can now surface the local `qmt_live` execution adapter capability identity alongside bridge-side QMT readiness checks.

## Implemented Scope

- Added stable string views for execution capability enums:
  - `ExecutionChannel::as_str`
  - `ExecutionStatusSource::as_str`
  - `ExecutionFillSource::as_str`
  - `ExecutionCancelSemantics::as_str`
- Extended `format_qmt_promotion_checklist` to include local `qmt_live` capability identity:
  - adapter channel
  - status source
  - fill source
  - cancel semantics
- Updated `execute_execution_bridge_status` to read `QmtLiveExecutionAdapter::capabilities()` from the local adapter declaration.
- Added regression coverage for checklist capability visibility.

## Preserved Boundaries

- No submit, query, or cancel behavior changes.
- No upper-layer mode-check migration.
- No `OrderStatus` changes.
- No order query response shape changes.
- No storage schema changes.
- No bridge protocol changes.
- No miniQMT runtime probing or startup self-check changes.
- No `.unwrap()` cleanup resumed.
- No JSON status response shape change; capability visibility is limited to the checklist text path.

## GitNexus Impact

GitNexus index was refreshed before P0.3f impact analysis because the previous index predated P0.3e symbols.

- `ExecutionCapabilities`: LOW, direct=3, affected processes=0
- `ExecutionChannel`: LOW, affected processes=0
- `format_qmt_promotion_checklist`: LOW, affected processes=0
- `execute_execution_bridge_status`: LOW, direct=1, affected processes=2

Post-implementation `detect_changes` reported MEDIUM risk because `execute_execution_bridge_status` participates in one indexed CLI status flow. This is expected for the read-only checklist output change; no submit, query, cancel, bridge protocol, or persistence flow is changed.

`AGENTS.md` and `CLAUDE.md` contain the generated GitNexus index count refresh from the required index update.

## Verification Plan

- TDD RED/GREEN for `test_format_qmt_promotion_checklist_surfaces_live_readiness_and_next_steps`.
- Targeted capability and CLI handler tests.
- `cargo fmt --check`.
- `cargo clippy -- -D warnings`.
- `cargo test`.
- `git diff --check`.
- GitNexus `detect_changes` before commit.
- FUNCTION_TREE P0.3f closeout and validation.
