# Worktree Closure Audit - 2026-06-04

> Scope: local worktree hygiene after sealing the CLI fail-closed scan phase.
> This report does not authorize deleting unmerged local work.

## Purpose

Create a visible board-level state for the remaining non-`master` worktrees so cleanup does not continue as ad hoc branch deletion.

Repository baseline during the audit:

- `master`: `ca13ac1e70f8021e10f4d83fe530abba76a7f351`
- `origin/master`: `ca13ac1e70f8021e10f4d83fe530abba76a7f351`
- Root worktree status: clean
- Open GitHub PRs: 0

## Closure Action Taken

One local worktree was safe to remove because it was clean, had no commits ahead of `master`, had no upstream, had no open PR, and its branch was already an ancestor of `master`.

| Branch | Worktree | Action | Evidence |
|---|---|---|---|
| `integration/mock-policy-master` | `.worktrees/integration-mock-policy-master` | Removed worktree and deleted local branch | clean; 0 ahead; 201 behind; branch ancestor of `master`; no upstream; no open PR |

## Preserved Worktrees

The remaining worktrees are clean but contain unmerged local work. They were intentionally preserved.

| Branch | Worktree | Ahead | Behind | Upstream | Head | Subject |
|---|---|---:|---:|---|---|---|
| `chore/ci-audit-workflow-dedupe` | `.worktrees/ci-audit-workflow-dedupe` | 1 | 162 | `origin/chore/ci-audit-workflow-dedupe` | `5f6a36f` | ci: dedupe audit workflow responsibilities |
| `chore/mock-policy-github-rebuild` | `.worktrees/mock-policy-github-rebuild` | 2 | 256 | `origin/master` | `22b750b` | test: align repo hygiene docs contract |
| `chore/mock-policy-qmt-gate-full` | `.worktrees/mock-policy-qmt-gate-full` | 3 | 204 | none | `78ffda1` | fix notification log sender flush semantics |
| `chore/mock-policy-qmt-gate-pr` | `.worktrees/mock-policy-qmt-gate-pr` | 1 | 164 | `origin/chore/mock-policy-qmt-gate-pr` | `ea12930` | fix: surface monitor notification failures |
| `cleanup/mainline-repo-hygiene` | `.worktrees/cleanup-mainline` | 88 | 259 | none | `3c5412e` | chore: preserve cleanup mainline hygiene work |
| `feature/kill-switch-v1` | `.worktrees/kill-switch-v1` | 1 | 163 | `origin/feature/kill-switch-v1` | `ab3d798` | feat: add execution kill switch |
| `security/dependency-audit-20260518` | `.worktrees/security-dependency-audit-20260518` | 2 | 166 | `origin/security/dependency-audit-20260518` | `0716d75` | docs: record dependency security audit evidence |

## Stop Rule

Do not delete preserved worktrees solely because they are old or behind `master`. Each preserved branch contains local commits not present on `master`; closing any of them requires a branch-specific triage decision:

- merge or rebase and PR,
- explicitly archive,
- or explicitly delete after confirming the work is obsolete.

## Next Restart Gate

The next worktree-hygiene phase should start with a bounded board of the seven preserved branches above. Pick one branch family at a time, starting with the branches that have upstreams and the smallest ahead count, then close the phase when that bounded board is resolved.
