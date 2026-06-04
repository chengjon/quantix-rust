# P2 Preserved Branch Closure - 2026-06-05

## Scope

This report closes the local preserved-worktree cleanup sequence that began with
the P0 and P1 reports:

- `docs/reports/WORKTREE_P0_PRESERVED_BRANCH_CLOSURE_2026-06-05.md`
- `docs/reports/WORKTREE_P1_PRESERVED_BRANCH_CLOSURE_2026-06-05.md`

P2 handled two large local-only branches:

- `chore/mock-policy-qmt-gate-full`
- `cleanup/mainline-repo-hygiene`

The goal of this slice is local closure and evidence preservation, not broad
source-code integration.

## Archive Tags

Before removing local worktrees/branches, the P2 branch heads were preserved as
remote archive tags:

| Branch | Archived Head | Archive Tag |
|---|---|---|
| `chore/mock-policy-qmt-gate-full` | `78ffda1284cadcc19710193fd283c0f4e23aabfd` | `archive/p2-mock-policy-qmt-gate-full-20260605` |
| `cleanup/mainline-repo-hygiene` | `3c5412e7f7e751c2ad145cb527b29be3a0cf64a3` | `archive/p2-cleanup-mainline-repo-hygiene-20260605` |

Recovery command shape:

```bash
git fetch origin --tags
git switch -c <new-branch-name> <archive-tag-name>
```

## Dispositions

| Branch | Disposition | Evidence |
|---|---|---|
| `chore/mock-policy-qmt-gate-full` | Closed as superseded after archive. Local worktree and branch can be removed. | The branch had no remote head and no PR. One of its three commits was already patch-equivalent in `master`; the remaining behavior is covered by later mainline implementation and tests. |
| `cleanup/mainline-repo-hygiene` | Closed as archived, not integrated. Local worktree and branch can be removed after archive. | The branch is an 88-commit, 134-file broad cleanup/refactor/deploy/docs line. It has no remote head and no PR. Integrating it would require a separate design and gate plan; preserving the exact head as an archive tag avoids losing work while clearing the active preserved-worktree board. |

## Supersession Evidence For `chore/mock-policy-qmt-gate-full`

The branch originally targeted mock-policy and QMT-live review gaps. Current
`master` already contains the relevant outcomes in newer mainline structure:

| Outcome | Current mainline evidence |
|---|---|
| Monitor notification failures are surfaced instead of ignored | `src/cli/handlers/monitor_handler.rs` contains `MonitorNotificationSender` and `send_monitor_notifications_with_service`; `src/cli/handlers/tests/monitor_runtime.rs` contains `test_send_monitor_notifications_propagates_notify_errors`. |
| QMT live strategy requests are rejected with manual bridge guidance | `src/cli/handlers/tests/strategy_requests.rs` contains `test_execute_strategy_request_execute_rejects_qmt_live_with_manual_bridge_guidance`. |
| Daemon QMT live requests use manual bridge diagnostics | `src/execution/request_diagnostics.rs` contains `build_daemon_qmt_live_manual_bridge_required_diagnostics`; `tests/execution_daemon_test.rs` contains `daemon_run_once_rejects_qmt_live_request_with_manual_bridge_guidance`. |
| Live-import risk state preserves manual release state | `src/cli/handlers/risk.rs` contains `load_risk_status_for_live_import_preserves_manual_release_state`. |
| Notification log sender creates parent directories and surfaces write/flush errors | `src/monitoring/notification/senders/log.rs` uses `tokio::fs::create_dir_all(parent).await?`, `write_all(...).await?`, and `flush().await?`; `src/monitoring/notification.rs` contains parent-directory and directory-error tests. |

Because the current code has since been reorganized, cherry-picking this branch
would reintroduce stale module shape rather than add missing behavior.

## `cleanup/mainline-repo-hygiene` Boundary

`cleanup/mainline-repo-hygiene` is not a small branch-closure candidate. Its
diff spans workflows, deployment scripts, documentation archive moves, runtime
store extraction, CLI handler extraction, source modules, and repo hygiene
tests.

This closure intentionally does not merge or cherry-pick it. Any future use of
the archived head should start from a new explicit plan with separate slices for:

- docs/archive moves
- CI/deployment/script hardening
- runtime store extraction
- CLI handler extraction
- source-module cleanup

## Closure Boundary

After this report lands and local worktrees/branches are removed, the preserved
worktree board from the P0/P1/P2 sequence is locally closed.

The only expected root worktree dirt outside this closure remains the unrelated
external-line files:

- `src/sources/mod.rs`
- `src/sources/tdx_api.rs`
