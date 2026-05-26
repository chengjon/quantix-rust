# worktree-cleanup Specification

## Purpose
TBD - created by archiving change dirty-worktree-cleanup-2026-05-26. Update Purpose after archive.
## Requirements
### Requirement: OpenSpec-Governed Dirty Worktree Cleanup
The dirty worktree cleanup SHALL be executed through the active OpenSpec change before cleanup actions are taken.

#### Scenario: Starting cleanup execution
- **WHEN** cleanup work begins from the revised 2026-05-26 dirty worktree cleanup plan
- **THEN** the active OpenSpec change SHALL identify the phase, task item, approval gate, recovery evidence, and validation gate

#### Scenario: Reading the cleanup evidence set
- **WHEN** cleanup scope or priority is ambiguous
- **THEN** `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26.md`, `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-review.md`, and `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-meta-review.md` SHALL be read as the controlling evidence set

### Requirement: Snapshot Before Cleanup
Cleanup actions SHALL NOT start until Phase 0 recovery artifacts exist and are verified.

#### Scenario: Creating Phase 0 artifacts
- **WHEN** Phase 0 is approved
- **THEN** the executor SHALL create recovery artifacts under the approved recovery directory before any cleanup, deletion, reset, rebase, or root-worktree realignment

#### Scenario: Phase 0 prerequisite failure
- **WHEN** a prerequisite check fails or dirty inventory drift is unexplained
- **THEN** Phase 0 SHALL stop before recovery artifacts are created or cleanup begins

#### Scenario: Restoring from snapshot
- **WHEN** a later phase needs the pre-cleanup dirty state
- **THEN** restoration SHALL be attempted in a disposable worktree unless root-worktree restoration is separately approved

### Requirement: Explicit Phase Approval
Each cleanup phase SHALL require explicit approval scoped to that phase.

#### Scenario: Broad implementation request
- **WHEN** a broad request asks to implement the cleanup plan
- **THEN** it SHALL NOT authorize destructive cleanup, generated artifact disposal, or root `master` realignment unless those actions are explicitly approved by phase and path group

#### Scenario: Generated artifact disposal
- **WHEN** `logs/`, `var/`, `test_timing.csv`, `docs/reports/evidence/`, `.governance/active-gates.*`, or `.governance/backups/*` are proposed for deletion or movement
- **THEN** each path group SHALL have path-level approval after preservation is verified

### Requirement: Clean Worktree Slice Extraction
Selected cleanup slices SHALL be extracted from a clean review base while the dirty root worktree remains the salvage source.

#### Scenario: Creating a clean review base
- **WHEN** Phase 1 is approved
- **THEN** the executor SHALL use a named clean worktree and SHALL NOT reset, rebase, or clean the dirty root `master`

#### Scenario: Slice conflict
- **WHEN** extraction of a documentation, governance, generated-artifact, or product-code slice conflicts
- **THEN** the executor SHALL stop at the slice boundary and record the conflicting paths before continuing

### Requirement: Code Slice Validation
Product-code slices SHALL pass impact and validation gates before closure.

#### Scenario: Editing a symbol during cleanup
- **WHEN** a cleanup slice edits a function, method, class, or refactor target
- **THEN** GitNexus impact analysis SHALL run before the edit and GitNexus detect_changes SHALL run before closure

#### Scenario: Closing a product-code slice
- **WHEN** a product-code cleanup slice is ready for closure
- **THEN** focused tests and task-relevant formatting or lint gates SHALL be run before the slice is marked complete

