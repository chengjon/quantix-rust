# Dirty Worktree Cleanup Design

## Context

The cleanup plan was produced after analyzing the current dirty worktree and was subsequently reviewed twice. The first review returned `APPROVE_WITH_NOTES` and identified missing prerequisites, restore procedure, failure modes, and approval protocol. The main plan was revised to add those sections. The meta-review confirmed the review was diagnostically accurate and that no further review-document edits were required.

The current execution boundary is intentionally conservative: the dirty root worktree remains the salvage source until recovery artifacts exist and later slices are extracted and reviewed.

## Goals / Non-Goals

**Goals:**

- Use OpenSpec to coordinate the cleanup instead of executing ad hoc shell cleanup.
- Preserve all current work before any cleanup or extraction action.
- Keep each cleanup phase independently approvable and auditable.
- Separate documentation/governance slices, generated/runtime artifacts, and product-code slices.
- Require GitNexus and focused validation before code-slice closure.

**Non-Goals:**

- Do not decide which dirty changes should be kept, deleted, or rewritten as part of OpenSpec setup.
- Do not reset, rebase, or realign the root `master` worktree before the approved recovery snapshot exists and selected slices are extracted.
- Do not delete generated artifacts, logs, backups, or evidence directories without explicit path-level approval.
- Do not perform opportunistic formatting, renames, or architecture refactors while closing cleanup gates.

## Decisions

- **OpenSpec change boundary:** `dirty-worktree-cleanup-2026-05-26` governs the cleanup plan and its recovery, approval, and extraction gates.
- **Evidence precedence:** the revised cleanup plan is the execution source of truth; the review and meta-review explain why the plan is acceptable and which gaps were closed.
- **Phase approval:** approval is phase-specific. Broad permission to implement this OpenSpec change does not authorize destructive cleanup, root realignment, or generated artifact disposal.
- **Phase 0 first:** recovery artifacts under `var/recovery/dirty-master-2026-05-26/` must exist and be verified before any cleanup starts.
- **Root worktree preservation:** the dirty root worktree stays as the salvage source until later root realignment is separately approved.
- **Slice extraction:** docs, governance, generated artifacts, and product code are extracted or disposed through separate tasks with separate review gates.
- **Code-slice gates:** any function, method, class, or refactor target edit requires GitNexus impact before edits and `gitnexus_detect_changes` before closure.

## Failure Handling

- If dirty inventory counts drift before a phase starts, pause and classify the drift before proceeding.
- If recovery artifact creation fails, keep partial artifacts in place, record the failing artifact, and restart Phase 0 into a new timestamped directory after the failure is resolved.
- If a worktree or branch already exists, inspect it rather than overwriting it.
- If conflicts occur during slice extraction, stop at the slice boundary and do not broaden scope to make conflict resolution easier.

## Validation

- Validate this OpenSpec change with `openspec validate dirty-worktree-cleanup-2026-05-26 --strict --no-interactive --json`.
- For Phase 0, validate artifact existence, file inventories, checksums, and restore command viability without applying the restore over the root worktree.
- For documentation slices, validate path-scoped diffs and OpenSpec task status.
- For code slices, validate focused tests, formatting as task-relevant, GitNexus impact, and GitNexus detect_changes.
