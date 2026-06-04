# P0 Preserved Branch Closure - 2026-06-05

## Scope

This report closes the P0 slice defined in
`docs/reports/WORKTREE_PRESERVED_BRANCH_TRIAGE_2026-06-05.md`.

The stop rule remains unchanged: only P0 preserved branches are handled here.
P1/P2 branches stay deferred until a new explicit slice is opened.

## Dispositions

| Branch | Disposition | Evidence |
|---|---|---|
| `chore/mock-policy-qmt-gate-pr` | Closed as stale merged PR branch. Local worktree, local branch, and remote branch were removed. | PR #75 was merged into `master`; branch head `ea12930` was included in PR #75; `git cherry -v master chore/mock-policy-qmt-gate-pr` reported the head as patch-equivalent in `master`. |
| `feature/kill-switch-v1` | Closed as stale merged PR branch. Local worktree, local branch, and remote branch were removed. | PR #76 was merged into `master`; merge commit `cb13d8422617ed2b9a4b3a6875be0c9aa1dfcb02` is an ancestor of current `master`; `git cherry -v master feature/kill-switch-v1` reported the head as patch-equivalent in `master`. |
| `security/dependency-audit-20260518` | Integrated as historical dependency-security evidence, not as current audit status. The source branch should be pruned after this documentation lands. | The branch had no PR and its two documentation commits were not patch-equivalent in `master`; the change set is documentation/evidence only: 6 files, 1004 insertions, no source-code changes. |

## Imported Dependency Security Evidence

The `security/dependency-audit-20260518` branch contributes a dated audit
package under:

- `docs/CODE_AUDIT_EVIDENCE/dependency-security/`
- `docs/reports/DEPENDENCY_SECURITY_AUDIT_ISSUE_DRAFTS_2026-05-18.md`
- `docs/superpowers/specs/2026-05-18-dependency-security-audit-execution-spec.md`

This import preserves the 2026-05-18 evidence boundary. It does not claim the
repository's current dependency graph is clean, does not update dependencies,
and does not add `cargo audit` ignores or CI policy relaxation.

## Closure Boundary

This slice intentionally stops after P0 branch disposition.

Deferred items:

- P1 preserved branches:
  - `chore/ci-audit-workflow-dedupe`
  - `chore/mock-policy-github-rebuild`
- P2 preserved branches:
  - `chore/mock-policy-qmt-gate-full`
  - `cleanup/mainline-repo-hygiene`

Any work on P1/P2 requires a new explicit triage and closure slice.
