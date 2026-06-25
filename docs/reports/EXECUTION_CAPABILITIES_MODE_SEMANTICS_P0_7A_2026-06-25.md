# ExecutionCapabilities Mode Semantics P0.7a

Date: 2026-06-25

Status: implemented

## Summary

P0.7a adds a narrow, read-only bridge from `ExecutionChannel` to the existing execution mode semantics registry.

The slice exposes helper functions for callers that already hold static `ExecutionCapabilities` metadata and need to look up the risk notice or storage namespace that is already registered in `mode_semantics`.

## Implemented Scope

- Added `risk_notice_for_execution_channel(ExecutionChannel) -> Option<&'static str>`.
- Added `storage_namespace_for_execution_channel(ExecutionChannel) -> Option<&'static str>`.
- Added focused regression coverage in `tests/execution_mode_semantics_test.rs`.

## Preserved Boundary

`ExecutionChannel::MockLive` intentionally returns `None` from both new helpers.

This preserves the P0.2 execution mode semantics decision that `mock_live` is not folded into the configured execution-mode storage binding. It also avoids mislabeling `mock_live` as `paper_sim_lifecycle`, which remains a separate future simulator direction rather than the existing mock-live adapter.

## Non-Goals

- No upper-layer `mode == ...` migration.
- No submit, query, cancel, reconciliation, or daemon behavior changes.
- No qmt_live runtime probing or startup self-check.
- No bridge protocol, response shape, storage schema, `OrderStatus`, or `ExecutionAdapter` signature changes.
- No paper-immediate or paper-sim-lifecycle behavior changes.
- No `.unwrap()` cleanup.

## GitNexus Impact

Pre-edit impact checks were LOW:

- `ExecutionChannel` enum: direct callers 0, affected processes 0.
- `ExecutionChannel` impl: direct callers 0, affected processes 0.
- `risk_notice_for_channel`: direct callers 1, affected processes 0.
- `storage_namespace_for_channel`: direct callers 1, affected processes 0.

The GitNexus index reported stale metadata because the current commit differs from the indexed commit, but it was fresh for staged diff analysis.

## Verification

- RED: `cargo test --test execution_mode_semantics_test execution_channels_reuse_mode_semantics_risk_and_storage_contracts` failed on missing helper imports.
- GREEN: same focused test passed after adding the helper functions and preserving `MockLive -> None`.
- Regression: `cargo test --test execution_mode_semantics_test` passed.
