# Phase 21 Integration Baseline Report

Date: 2026-03-10

## Baseline

- Branch: `phase21-integration`
- Stable commit: `ebca55c`
- Stable tag: `phase21-integration-green-20260310`
- Validation command: `cargo test --all-targets`
- Validation result: full green on 2026-03-10

## Backup and Recovery Points

- Master dirty-state stash: `stash@{0}` (`pre-phase21-merge-backup-2026-03-10`)
- Patch backup directory: `/tmp/phase21-merge-backup-20260310T191502`

These recovery points must be kept until the team explicitly decides the old master-local changes are no longer needed.

## Integration Rule Used

Priority order:

1. Preserve the tested Phase 21 watchlist implementation as the primary baseline.
2. Review master dirty changes only where they are clearly better than the green baseline.
3. Reject changes that weaken P0 boundaries, remove guardrails, or introduce placeholder behavior.
4. Keep master unchanged while using `phase21-integration` as the active development baseline.

## Review Outcome for Non-overlapping Stash Changes

No non-overlapping stash changes were integrated into the baseline.

Reasons:

- Many diffs were formatting-only and provided no product or reliability gain.
- Several diffs were clear regressions, including removal of guard tests and weakening of scheduler behavior.
- Several diffs converted explicit `Unsupported` behavior into silent empty returns, which hides incomplete implementations.
- Several diffs introduced placeholder construction using `zeroed()`, which is not acceptable for a stable baseline.

Representative rejected areas:

- `src/tasks/scheduler.rs`
  - Removed registered callback execution path and related test coverage.
- `tests/repo_hygiene_test.rs`
  - Deleted repository hygiene guard tests.
- `tests/runtime_placeholder_guard_test.rs`
  - Deleted placeholder-safety guard test.
- `src/sync/etl.rs`
  - Replaced explicit unsupported paths with mock data and `zeroed()` placeholders.
- `src/sources/auction_collector.rs`
  - Added `Default` implementation using `zeroed()` placeholder state.
- `src/sources/tdx.rs`
  - Changed unsupported fetches into silent empty results.
- `src/sources/akshare.rs`
  - Changed unsupported fetches into silent empty results.

## Intentional Exclusions

- `.gitnexus/` was left untouched.
- Master branch was left unchanged after backup.
- The old stash was retained for future manual reference only.

## Recommended Forward Workflow

- Continue all new development from `phase21-integration`.
- Treat `phase21-integration-green-20260310` as the rollback anchor.
- Do not auto-apply `stash@{0}`.
- If any old master-local idea is later needed, cherry-pick it manually after targeted review and fresh tests.
