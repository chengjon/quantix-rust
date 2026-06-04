# P1 Preserved Branch Closure - 2026-06-05

## Scope

This report closes the P1 slice defined after
`docs/reports/WORKTREE_P0_PRESERVED_BRANCH_CLOSURE_2026-06-05.md`.

Handled branches:

- `chore/ci-audit-workflow-dedupe`
- `chore/mock-policy-github-rebuild`

The stop rule remains unchanged: this slice does not process P2 branches.

## Dispositions

| Branch | Disposition | Evidence |
|---|---|---|
| `chore/ci-audit-workflow-dedupe` | Closed as stale merged PR branch. Local worktree and local branch were removed. No remote branch remained. | PR #77 was merged into `master`; branch head `5f6a36f` was patch-equivalent in `master` according to `git cherry -v master chore/ci-audit-workflow-dedupe`. |
| `chore/mock-policy-github-rebuild` | Closed as superseded local-only branch. Local worktree and local branch should be removed after this report lands. | The branch had no remote head and no PR. Its plan/spec docs already exist in `master`, and its core behavioral outcomes are covered by later mainline code and tests. |

## Supersession Evidence For `chore/mock-policy-github-rebuild`

The branch's original intent was to rebuild mock-policy safeguards around
manual QMT live execution, live-import risk state, and notification logging.
Current `master` already contains the relevant outcomes in newer structure:

| Outcome | Current mainline evidence |
|---|---|
| QMT live strategy requests are rejected with manual bridge guidance | `src/cli/handlers/tests/strategy_requests.rs` contains `test_execute_strategy_request_execute_rejects_qmt_live_with_manual_bridge_guidance`. |
| Daemon QMT live requests use manual bridge diagnostics | `src/execution/request_diagnostics.rs` contains `build_daemon_qmt_live_manual_bridge_required_diagnostics`; `tests/execution_daemon_test.rs` contains `daemon_run_once_rejects_qmt_live_request_with_manual_bridge_guidance`. |
| Live-import risk state preserves manual release state | `src/cli/handlers/risk.rs` contains `load_risk_status_for_live_import_preserves_manual_release_state`. |
| Notification log sender creates parent directories and surfaces write errors | `src/monitoring/notification/senders/log.rs` uses `tokio::fs::create_dir_all(parent).await?`, `write_all(...).await?`, and `flush().await?`; `src/monitoring/notification.rs` contains `test_log_sender_returns_error_when_log_path_is_directory`. |
| Design and plan documents are preserved | `docs/superpowers/plans/2026-04-28-mock-policy-github-rebuild.md` and `docs/superpowers/specs/2026-04-28-mock-policy-github-rebuild-design.md` exist in `master`. |

Because the code paths have since been reorganized, cherry-picking the old
branch would reintroduce stale implementation shape rather than add missing
behavior.

## Closure Boundary

This slice intentionally stops after P1 disposition.

Deferred P2 branches:

- `chore/mock-policy-qmt-gate-full`
- `cleanup/mainline-repo-hygiene`

Any work on P2 requires a new explicit closure plan.
