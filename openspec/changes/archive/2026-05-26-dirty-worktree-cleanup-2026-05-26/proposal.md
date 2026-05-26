# Dirty Worktree Cleanup 2026-05-26

## Why

The repository currently has a heavily diverged dirty `master` worktree with unrelated documentation, governance, generated artifact, and product-code changes co-mingled. The reviewed cleanup plan establishes a conservative path: snapshot first, keep the dirty root worktree as the salvage source, then extract independently reviewable slices from clean worktrees.

This change makes OpenSpec the control plane for that cleanup so every phase has an explicit approval gate, recovery evidence, and validation checkpoint before any destructive or broad-scope action is taken.

## What Changes

- Add an OpenSpec-governed cleanup change for the revised dirty worktree cleanup plan.
- Define requirements for snapshot-first cleanup, explicit phase approval, clean-worktree extraction, generated artifact disposition, and root `master` realignment.
- Track implementation tasks for Phase 0 through final acceptance without executing destructive actions implicitly.
- Treat the three cleanup documents as the controlling evidence set:
  - `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26.md`
  - `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-review.md`
  - `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-meta-review.md`

## Impact

- Adds an active OpenSpec change under `openspec/changes/dirty-worktree-cleanup-2026-05-26/`.
- Does not change runtime behavior by itself.
- Does not delete, reset, rebase, move, or archive dirty worktree content by itself.
- Subsequent approved tasks may create recovery artifacts under `var/recovery/dirty-master-2026-05-26/`, create clean worktrees, extract documentation slices, classify generated artifacts, and eventually realign the root worktree.

## Risks

- Treating broad implementation approval as phase approval would violate the cleanup plan. Each phase still needs the approval gate defined in the plan.
- The dirty worktree is live; inventory counts can drift. Each execution phase must refresh counts before acting.
- GitNexus `detect_changes(scope=all)` is expected to be noisy because of the pre-existing dirty worktree. Scope checks must be interpreted against the active cleanup slice.
- Recovery artifacts may be large. Phase 0 must verify disk capacity before creating archives.
