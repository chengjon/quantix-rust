# Slice 6C Execution / Strategy Runtime Classification

Date: 2026-05-26

Status: classification complete; extraction deferred/excluded from this cleanup
pass by approval on 2026-05-26.

## Scope

Compared root dirty worktree candidates against the clean review worktree for:

- `src/execution/*`
- `src/strategy/*`
- `src/cli/handlers/strategy_handler*`
- execution / strategy / QMT-related tests

## Superseded Or Already Landed

25 candidate files already matched the clean review base and require no copy.
All direct `src/strategy/*` strategy implementation files in the candidate set
matched the clean review base.

## Excluded Stale Root Copies

These root dirty copies would remove behavior that exists in the clean review
base and were not copied:

- `src/execution/reconciliation.rs`
  - Root would remove clean-base `qmt_live.task_identity.external_order_id`
    persistence.
- `src/cli/handlers/strategy_handler.rs`
  - Root would remove clean-base safety kill-switch wiring and diagnostics.

## Production Drift Requiring Explicit Handling

These files differ from the clean review base and remain high-risk:

- `src/execution/qmt_live_adapter.rs`
  - Diff shape: `+25/-15`.
  - Example root-only behavior: `Self::with_default_polling(client, "qmt_live")`.
  - GitNexus impact on `QmtLiveExecutionAdapter`: LOW, but this file is still
    execution adapter production code and should not be copied mechanically.
- `src/execution/request_diagnostics.rs`
  - Diff shape: `+9/-40`.
  - Example root-only behavior: `QMT_STATUS_CHECKLIST_HINT`.
  - GitNexus impact on `build_completion_diagnostics`: CRITICAL.
  - Affected areas include strategy request execution, execution bridge
    `qmt_live`, and execution command flows.
- `src/cli/handlers/strategy_handler/instances.rs`
  - Diff shape: `+2/-0`.
  - GitNexus impact on `execute_strategy_create_with_store`: CRITICAL.
  - Affected areas include strategy command create/update/delete flows.

## Test Drift Candidates

These tests contain root-only additions or drift, but were not copied because
some assertions may be coupled to the high-risk production drift above:

- `tests/execution_kernel_test.rs` (`+16/-0`)
- `tests/execution_runtime_store_test.rs` (`+24/-0`)
- `tests/qmt_bridge_preview_test.rs` (`+7/-0`)
- `tests/qmt_live_adapter_test.rs` (`+34/-4`)
- `tests/strategy_integration_test.rs` (`+2/-3`)

## Decision

No Slice 6C production code or test drift was copied in this pass.

Approved handling on 2026-05-26:

- Do not extract Execution / Strategy production drift in this cleanup pass.
- Do not extract the coupled Slice 6C test drift in this cleanup pass.
- Treat Slice 6C local production drift as deferred/excluded from the dirty
  worktree cleanup branch.
- Open a dedicated execution/strategy high-risk OpenSpec change later if these
  local changes still need product review.

Recommended future options:

1. Drop Slice 6C local production drift as superseded/stale and keep only a
   separately reviewed subset of test-only additions.
2. Open a dedicated execution/strategy review slice with path-level approval
   for each production file and focused gate ownership.
3. Defer Slice 6C entirely until the dirty worktree cleanup branch is split
   into smaller PR-ready branches.
