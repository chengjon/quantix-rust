# Worktree Preserved Branch Triage - 2026-06-05

> Scope: triage the seven preserved non-`master` worktrees from `docs/reports/WORKTREE_CLOSURE_AUDIT_2026-06-04.md`.
> This report is a bounded triage board only. It does not authorize deleting or rebasing any preserved branch.

## Purpose

Turn the preserved-worktree backlog into a small, visible board with a stop rule. The goal is to avoid deleting old branches blindly while also avoiding an unbounded cleanup thread.

Current baseline during triage:

- `master`: `5354380da4da3a1e83174368d31db160e14ba04c`
- Root worktree status: clean
- Open GitHub PRs: 0
- Previously safe cleanup completed: `integration/mock-policy-master` was removed in the 2026-06-04 closure audit.

## Triage Board

| Priority | Branch | Worktree | Ahead | Behind | Upstream State | Size | Initial Disposition |
|---|---|---|---:|---:|---|---|---|
| P0 | `chore/mock-policy-qmt-gate-pr` | `.worktrees/mock-policy-qmt-gate-pr` | 1 | 165 | local matches `origin/chore/mock-policy-qmt-gate-pr` | 2 files, +90/-1 | Small upstreamed branch; review first for rebase/PR or archive decision |
| P0 | `feature/kill-switch-v1` | `.worktrees/kill-switch-v1` | 1 | 164 | local matches `origin/feature/kill-switch-v1` | 25 files, +2688/-62 | Small commit count but broad feature; review second, likely needs feature-level decision |
| P0 | `security/dependency-audit-20260518` | `.worktrees/security-dependency-audit-20260518` | 2 | 167 | local matches `origin/security/dependency-audit-20260518` | 6 docs files, +1004/-0 | Security evidence branch; review as documentation/archive candidate, do not delete casually |
| P1 | `chore/ci-audit-workflow-dedupe` | `.worktrees/ci-audit-workflow-dedupe` | 1 | 163 | configured upstream missing | 2 files, +16/-44 | Small branch but remote tracking is stale; review after P0 |
| P1 | `chore/mock-policy-github-rebuild` | `.worktrees/mock-policy-github-rebuild` | 2 | 257 | upstream points at `origin/master`; local/upstream diverged | 8 files, +422/-117 | Upstream misconfigured; inspect lineage before any PR/archive decision |
| P2 | `chore/mock-policy-qmt-gate-full` | `.worktrees/mock-policy-qmt-gate-full` | 3 | 205 | local-only | 54 files, +6328/-4135 | Large local-only line; defer until smaller upstreamed branches are resolved |
| P2 | `cleanup/mainline-repo-hygiene` | `.worktrees/cleanup-mainline` | 88 | 260 | local-only | 134 files, +8610/-5091 | Very large local-only line; requires its own closure plan |

## Recommended First Slice

The next bounded implementation slice should cover only the P0 upstreamed branches:

1. `chore/mock-policy-qmt-gate-pr`
2. `feature/kill-switch-v1`
3. `security/dependency-audit-20260518`

For each branch, choose exactly one closure path:

- rebase and open/update a PR,
- archive with an explicit preservation note,
- or delete only after confirming the work is obsolete and recoverable from remote or backup.

## Stop Rule

Stop after the three P0 branches have explicit dispositions. Do not continue into P1/P2 branches in the same phase. P1/P2 require a new bounded board after P0 is closed.

## Guardrails

- Do not delete local-only branches without an explicit branch-level decision.
- Do not treat a clean worktree as proof that the branch is disposable.
- Do not rebase broad feature work just to reduce branch count.
- Keep any future PRs branch-specific; do not combine unrelated preserved worktrees.
