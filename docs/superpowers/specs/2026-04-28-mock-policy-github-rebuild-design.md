# Mock Policy Github Rebuild Design

## Goal

Rebuild the previously validated mock-policy behavior directly on top of `github/master` without replaying the unrelated local `master` history or the later CLI/handler structural refactors.

## Scope

This rebuild is intentionally limited to four semantic fixes:

1. `strategy request execute` must not directly execute `qmt_live` requests.
2. Execution daemon auto-consume must reject pending `qmt_live` requests and point users to the manual bridge path.
3. `risk --source live_import status` must preserve persisted lock semantics, including `buy_lock`, `manual_release_active`, and trigger metadata, instead of recomputing only from current mirror balances.
4. Notification log sending must create parent directories, propagate open/write failures, and flush before returning success.

## Non-Goals

- No replay of market CLI, market strength, or handler split history.
- No broad documentation/help-text rebuild in this branch.
- No cleanup of pre-existing warnings or unrelated failing checks.

## Current Baseline

The rebuild starts from `origin/master` at `40b087be3d4b2aeef056f4ac66972e59a7e09a07`.

Fresh baseline verification on this branch shows:

- `cargo test -q` does not start from fully green.
- The current pre-existing failures are in `tests/repo_hygiene_test.rs` and assert missing README / `USER_MANUAL` wording:
  - `readme_documents_phase29_strategy_paper_boundary`
  - `readme_documents_phase29c_execution_automation_boundary`
  - `user_manual_documents_phase29c_execution_automation_commands`

These baseline failures are unrelated to the narrow rebuild scope and must be treated separately from any regression introduced by this work.

## Target Architecture

Rebuild on the existing `github/master` layout instead of porting the later split files:

- Strategy request execution remains in `src/cli/handlers/mod.rs`.
- Daemon request execution remains in `src/execution/daemon.rs`.
- Live-import risk status remains in `src/cli/handlers/risk.rs` with support from `src/risk/service.rs` and `src/risk/models.rs`.
- Notification log sender remains in `src/monitoring/notification.rs`.

The code changes should be semantic and local:

- add guards and clearer errors around `qmt_live`
- reuse persisted risk state where possible rather than introducing new storage formats
- harden file IO behavior in the existing log sender

## Test Strategy

Use TDD against the old architecture:

1. Add or extend focused regression tests in the existing old-architecture suites.
2. Verify each new test fails for the expected reason on `origin/master`.
3. Implement the minimum production changes to satisfy the new tests.
4. Re-run targeted tests first, then run a broader verification pass.

Likely suites:

- `src/cli/handlers/tests/mod.rs`
- `src/cli/tests/strategy.rs`
- `src/cli/tests/risk.rs`
- `tests/execution_daemon_test.rs`
- existing notification tests in `src/monitoring/notification.rs`

## Success Criteria

- The four scoped behaviors match the validated mock-policy semantics.
- No unrelated structural refactor is introduced.
- New regression tests cover each behavior change.
- Targeted verification for the rebuilt areas passes.
- Any final remaining failures are either fixed by this branch or explicitly identified as pre-existing baseline failures.
