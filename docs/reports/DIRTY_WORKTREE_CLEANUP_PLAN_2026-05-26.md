# Dirty Worktree Cleanup Plan - Review Draft

Date: 2026-05-26
Repository: `/opt/claude/quantix-rust`
Status: REVIEW DRAFT - no cleanup actions executed
Review status: revised after `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-review.md`

## Purpose

This document analyzes the current dirty `master` worktree and proposes a conservative cleanup plan for human review.

The plan follows the Matt Pocock skills setup already present in this repository:

- `docs/agents/domain.md` is used as the project-specific domain-doc routing file.
- `FUNCTION_TREE.md` remains the sole feature status registry.
- Audit reports, GitHub issues, and cleanup plans must not become competing feature status sources.
- Broad scans use context-mode summaries instead of raw terminal dumps.
- Architecture and code changes must be split by domain boundary and verified independently.

This plan is intentionally non-destructive. It does not recommend running `git reset`, `git checkout --`, `rm`, branch deletion, or worktree removal until after a preservation snapshot and explicit approval.

## Executive Summary

The current root worktree is not a single cleanup task. It is a mixed-state workspace containing:

1. A local `master` commit that is ahead of `origin/master`.
2. A large unstaged tracked diff.
3. A sizeable untracked set containing review documents, governance tooling, generated evidence, logs, local config, new Rust modules, and tests.
4. A branch divergence from current `origin/master`, which already contains merged PRs #73-#77.

Current branch state:

```text
master...origin/master [ahead 1, behind 6]
local-only commit: 14ab859 chore: complete architecture audit remediation openspec change
origin/master tip: b59955a ci: dedupe audit workflow responsibilities (#77)
merge-base: f242316
```

Dirty worktree summary:

```text
status entries: 202 compact porcelain entries at initial capture; 204 at review reconciliation
tracked dirty entries: 156
untracked compact entries: 46 at initial capture
actual untracked files: 88 at initial capture; 94 at review reconciliation
tracked diff vs local HEAD: 156 files, +4754/-3329
local HEAD diff vs merge-base: 76 files, +8932/-6906
staged entries: 0
deleted tracked files: 3
GitNexus detect_changes(scope=all): CRITICAL, 156 changed files, 1571 changed symbols, 175 affected symbols/process participants
```

The key risk is accidental flattening: if this worktree is committed or reset as one unit, unrelated domains will be either merged together or lost together. Cleanup should proceed by snapshotting first, then extracting independently reviewable slices into clean worktrees based on `origin/master`.

The changing untracked count means the worktree is still live. Refresh the inventory immediately before any approved action, and treat count drift as a reason to pause until the new files are classified.

## Evidence Snapshot

### Branch Divergence

Local `master` is both ahead and behind `origin/master`.

The one local-only commit is:

```text
14ab859 chore: complete architecture audit remediation openspec change
```

That commit adds archived OpenSpec architecture-remediation files and changes 76 source/test/spec files. Current `origin/master` already contains later merged architecture remediation work:

```text
b59955a ci: dedupe audit workflow responsibilities (#77)
cb13d84 feat: add execution kill switch
2683a52 fix: surface monitor notification failures
48cebba Complete architecture audit remediation (#73)
568d20f fix: remediate cargo audit baseline (#74)
3676cb4 ci: restore clean checkout gates
```

Implication: the local-only commit must be treated as a possible duplicate or predecessor of already merged work, not as a straightforward commit to push.

### Dirty Buckets

| Bucket | Total | Tracked | Untracked | Modified | Deleted | Tracked +/- | Interpretation |
|---|---:|---:|---:|---:|---:|---:|---|
| `src` | 107 | 103 | 4 | 103 | 0 | +3050/-872 | High-risk production code spread across many domains. Must not be bulk-committed. |
| `tests` | 26 | 23 | 3 | 23 | 0 | +311/-64 | Test changes span execution, strategy, market, risk, stop, QMT, script gates. Pair with source slices. |
| `docs/other` | 19 | 14 | 5 | 12 | 2 | +1171/-1915 | Roadmap/doc navigation churn. Needs governance review because roadmap files are deleted. |
| `docs/reports` | 14 | 3 | 11 | 3 | 0 | +9/-0 | Mostly audit/evidence reports. Candidate docs-only slice. |
| `docs/superpowers` | 14 | 3 | 11 | 3 | 0 | +12/-3 | Plans/specs/reviews. Candidate planning artifact slice. |
| `top-level-docs` | 4 | 4 | 0 | 3 | 1 | +69/-197 | `README.md`, `CHANGELOG.md`, `FUNCTION_TREE.md`, deleted `ROADMAP.md`. Requires feature-registry gate. |
| `agent-governance` | 4 | 1 | 3 | 1 | 0 | +14/-0 | Matt Pocock / function-tree agent setup and local command surfaces. Needs repo-vs-user decision. |
| `docs/standards` | 4 | 1 | 3 | 1 | 0 | +2/-0 | Code-audit methodology documents. Candidate docs standards slice. |
| `runtime-artifact` | 3 | 0 | 3 | 0 | 0 | +0/-0 | `logs/`, `var/`, `test_timing.csv`. Usually do not commit. Preserve then ignore/delete only with approval. |
| `cargo` | 2 | 2 | 0 | 2 | 0 | +2/-0 | `Cargo.toml` adds `sha2 = "0.10"`. Must be tied to a code slice. |
| `benches` | 1 | 1 | 0 | 1 | 0 | +2/-1 | Benchmark drift; low priority unless tied to code change. |
| `openspec` | 1 | 0 | 1 | 0 | 0 | +0/-0 | `openspec/config.yaml` untracked. Decide whether repo config should be tracked. |

### Source Module Spread

`src` changes are broad and cross-cutting:

| Module | Dirty files | Examples |
|---|---:|---|
| `src/cli` | 33 | commands, handlers, handler tests, import command types |
| `src/execution` | 10 | QMT bridge/live gate/live adapter/task submit, daemon helpers, reconciliation |
| `src/strategy` | 8 | daemon, registry, runtime, concrete strategies |
| `src/market` | 7 | market service, sentiment, strength runtime |
| `src/news` | 6 | providers and provider types |
| `src/analysis` | 5 | indicator cache/registry/pipeline/polars adapter |
| `src/fundamental` | 5 | EastMoney/fundamental providers |
| `src/ai` | 4 | AI decision/prompt/types |
| `src/risk` | 4 | importer, industry sync, rebuild, service |
| `src/import` | 3 | CSV/text parser and import module |
| Other modules | 22 | account, core, factor, monitoring, sources, stop, sync, tasks, watchlist, test support |

This is the main reason the cleanup must use separate clean worktrees and domain-by-domain extraction.

### Untracked Files

Actual untracked file count is 88. The compact `git status` output collapses some directories, so the compact count is lower.

| Group | Files | Bytes | Initial recommendation |
|---|---:|---:|---|
| `.governance` | 23 | 790042 | Preserve snapshot. Commit only stable governance files; archive or ignore timestamped backups. |
| `.claude` | 13 | 8745 | Decide whether function-tree commands belong in repo. Otherwise keep user-local. |
| `.codex` | 12 | 5754 | Same decision as `.claude`; avoid duplicating tool surfaces unless intentional. |
| `docs/reports` | 12 | 91331 | Candidate docs-only commit if reports are final deliverables. |
| `docs/superpowers` | 11 | 202915 | Candidate planning/spec artifact commit after review. |
| `src` | 4 | 16764 | Must be analyzed with tests before commit. |
| `docs/standards` | 3 | 80444 | Candidate standards/methodology commit. |
| `tests` | 3 | 47291 | Must pair with corresponding `src` slice. |
| `docs/architecture` | 2 | 18301 | Candidate architecture docs commit. |
| `logs` | 2 | 3230 | Runtime artifacts; do not commit by default. |
| `.mcp.json` | 1 | 596 | Needs repo-vs-user config decision. |
| `docs/agents` | 1 | 1338 | Matt Pocock skills context; likely candidate to commit with agent setup. |
| `docs/operations` | 1 | 7933 | Candidate operations doc commit. |
| `openspec` | 1 | 1914 | Decide whether OpenSpec config is canonical repo config. |
| `test_timing.csv` / `var` | 2 | 5399 | Generated timing artifacts; do not commit by default. |

## Cleanup Principles

1. Preserve first.
   Before any reset, checkout, restore, delete, or branch rewrite, create a complete recoverable snapshot of tracked diff, untracked files, local commit, and stash list.

2. Do not clean on dirty `master`.
   Use clean worktrees based on current `origin/master` for every extraction slice.

3. Do not bulk-commit cross-domain changes.
   A commit that touches CLI, execution, strategy, market, risk, docs, governance, and generated artifacts together is not reviewable.

4. Treat `FUNCTION_TREE.md` as authoritative.
   Any changes to `README.md`, `CHANGELOG.md`, `ROADMAP.md`, `docs/DEVELOPMENT_ROADMAP.md`, and `docs/ROADMAP_REVIEW.md` must be reconciled against `FUNCTION_TREE.md`.

5. Keep generated evidence separate from source changes.
   Logs, timing CSVs, retained evidence, and governance backups should not ride along with product-code commits.

6. Any production-code slice requires GitNexus impact and focused gates before commit.
   Current dirty scope is `CRITICAL`; narrow it before running full acceptance gates.

## Proposed Cleanup Plan

### Prerequisites

Complete these checks before Phase 0. If any check fails, stop before creating or moving any recovery artifacts.

1. Confirm no other operator, agent, editor automation, or background workflow is writing to this repository.
2. Confirm the Git index is not locked:

```bash
test ! -e "$(git rev-parse --git-dir)/index.lock"
```

3. Confirm enough disk is available for `var/recovery/dirty-master-2026-05-26/`. Budget for the tracked diff, a full untracked archive, and duplicated report/evidence files. If unsure, measure `logs/`, `var/`, `docs/reports/evidence/`, and the full untracked file list before proceeding.
4. Confirm `git worktree` operations are available and the chosen sibling worktree paths do not already exist.
5. Confirm CI, file watchers, or other automation will not mutate the worktree during Phase 0 and Phase 1. Pause them or explicitly record why they are safe to leave running.
6. Re-run the dirty inventory immediately before snapshot creation. Count drift is expected in a live worktree, but unexplained drift must be classified before proceeding.
7. Assign one named Executor for the cleanup command stream. If multiple operators are active, designate an Approver and Reviewer before Phase 0 starts.

Current reconciliation checks:

```text
git index lock: not present
docs/reports/evidence/: exists
compact status entries: 204
actual untracked files: 94
```

### Phase 0 - Freeze And Snapshot

Goal: make loss impossible before analysis continues.

Actions to perform only after review approval:

1. Create a local safety branch from the current dirty `master` state.
2. Save tracked diff to a patch file under a timestamped rescue directory.
3. Save untracked file inventory, file sizes, and checksums.
4. Save local commit metadata for `14ab859`.
5. Save stash metadata for:
   - `stash@{0}: On feature/kill-switch-v1: kill-switch-v1-before-master-refresh`
   - `stash@{1}: On chore/mock-policy-qmt-gate-pr: mock-policy-qmt-gate-pr-staged-before-master-rebase`
6. Create an optional tar archive of untracked files if disk space permits.

Suggested artifact directory:

```text
var/recovery/dirty-master-2026-05-26/
```

Expected outputs:

```text
tracked.diff
untracked-files.txt
untracked-sha256.txt
local-head-show.txt
stash-list.txt
worktree-list.txt
```

No cleanup should start until these artifacts exist and are verified.

#### Phase 0 Restore Procedure

Use this procedure only if a later phase needs to reconstruct the pre-cleanup state from the approved snapshot. Prefer restoring into a disposable worktree unless the Approver explicitly authorizes restoring over the root worktree.

1. Start from the recorded `local-head-show.txt` commit.
2. Restore tracked modifications:

```bash
git apply var/recovery/dirty-master-2026-05-26/tracked.diff
```

3. Restore untracked files from the recorded archive, if one was created:

```bash
tar -xf var/recovery/dirty-master-2026-05-26/untracked-files.tar -C .
```

4. Re-check the reconstructed state:

```bash
git status --short
```

5. If a stash from `stash-list.txt` is needed, use `git stash apply <recorded-stash-id>` after review. Do not use `git stash pop` during recovery, because it mutates the stash stack and makes repeated recovery harder.

#### Phase 0 Failure Handling

| Failure | Response |
| --- | --- |
| Disk full while writing recovery artifacts | Stop immediately, leave existing partial artifacts in place, record which artifact failed, free or provision space, then restart Phase 0 into a new timestamped directory. |
| Git index lock exists | Stop and identify the owning process. Do not remove the lock unless the owning process is confirmed dead and the Approver accepts the risk. |
| Untracked archive creation fails | Do not proceed to cleanup. Keep `untracked-files.txt` and `untracked-sha256.txt`, fix the archive failure, and regenerate the archive before Phase 1. |
| Inventory changes during snapshot | Stop and rerun the inventory. Classify newly appearing or disappearing files before declaring the snapshot complete. |

### Phase 1 - Establish A Clean Review Base

Goal: recover a clean current-mainline base without losing dirty work.

Recommended approach:

1. Keep the current dirty root worktree as a salvage source.
2. Create a new clean worktree from `origin/master`, for example:

```text
.worktrees/dirty-cleanup-review-base
```

3. Do not reset the root `master` until all selected slices are extracted and reviewed.

Rationale: `master` is ahead 1 and behind 6. Resetting or rebasing it now would mix recovery with conflict resolution.

#### Phase 1 Failure Modes

| Failure | Response |
| --- | --- |
| Worktree path already exists | Stop and inspect the existing path. Use a new explicit worktree name only after confirming the old path is not part of another active cleanup. |
| Cleanup branch already exists | Stop and inspect the branch tip and reflog. Do not overwrite; either reuse it intentionally or create a new uniquely named branch. |
| Patch, cherry-pick, or copy operation conflicts | Stop on the first conflict, record the conflicting paths, and resolve only within the active slice. Do not broaden the slice to make the conflict easier. |
| `origin/master` changes during cleanup | Freeze the current slice until the base update is reviewed. Rebase or recreate clean worktrees only as an explicit follow-up action. |
| Concurrent repo access detected | Stop write operations and coordinate ownership before continuing. The root worktree must remain the salvage source until a new snapshot is approved. |

### Phase 2 - Close Or Reconcile The Local-Only Commit

Goal: decide whether `14ab859` still has unique value.

Evidence:

- The commit is titled `chore: complete architecture audit remediation openspec change`.
- `origin/master` already has `48cebba Complete architecture audit remediation (#73)`.
- The local commit includes OpenSpec archive/spec files and broad source/test changes.

Review action:

1. Compare `14ab859` against `48cebba` and current `origin/master`.
2. Mark each changed file as:
   - already merged in current shape,
   - superseded by later implementation,
   - still missing and worth extracting,
   - obsolete and should be dropped.

Likely outcome:

- Do not push `14ab859` as-is.
- Extract only missing OpenSpec/archive/spec artifacts, if any.
- Treat broad source/test changes inside `14ab859` as superseded unless current evidence proves otherwise.

### Phase 3 - Docs And Governance Slices

Goal: land documentation artifacts without dragging production code.

Candidate slices:

1. Architecture audit report package
   - `docs/reports/ARCHITECTURE_AUDIT_2026-05-23.md`
   - `docs/reports/ARCHITECTURE_AUDIT_REVIEW_2026-05-23.md`
   - `docs/superpowers/specs/2026-05-23-architecture-audit-design.md`

2. Code audit methodology package
   - `docs/standards/CODE_AUDIT_METHODOLOGY.md`
   - `docs/standards/CODE_AUDIT_METHODOLOGY-review.md`
   - `docs/standards/CODE_AUDIT_METHODOLOGY_REVIEW_CODEX_2026-05-11.md`
   - related audit reports only if final deliverables.

3. miniQMT evidence and operations package
   - `docs/operations/MINIQMT_QUANTIX_REGRESSION_OPERATOR_RUNBOOK_2026-05-20.md`
   - miniQMT closeout reports under `docs/reports/`
   - exclude logs, raw retained evidence, timing CSVs, and transient local outputs.

4. Matt Pocock / function-tree agent setup
   - `docs/agents/issue-tracker.md`
   - `.claude/commands/ft/*`
   - `.codex/commands/ft/*`
   - `.governance/README.md`
   - `.governance/active-gates.*`

Review question:

Should `.claude/`, `.codex/`, `.governance/`, `.mcp.json`, and `openspec/config.yaml` be committed as repository policy, or kept as local/operator configuration?

### Phase 4 - Roadmap And Registry Reconciliation

Goal: avoid conflicting status sources.

High-risk files:

```text
M FUNCTION_TREE.md
M README.md
M CHANGELOG.md
D ROADMAP.md
D docs/DEVELOPMENT_ROADMAP.md
D docs/ROADMAP_REVIEW.md
```

Required review:

1. Confirm `FUNCTION_TREE.md` remains the sole active feature registry.
2. Decide whether deleted roadmap files should be:
   - restored,
   - archived under `docs/archive/`,
   - replaced by explicit links to `FUNCTION_TREE.md`.
3. Ensure `README.md` and `CHANGELOG.md` describe status-source rules without duplicating active feature state.

Suggested slice name:

```text
docs: reconcile function tree and roadmap status sources
```

This slice should be docs-only unless a gate forces a test update.

### Phase 5 - Generated And Runtime Artifacts

Goal: prevent local evidence and generated files from contaminating product commits.

Default action after snapshot:

| Path | Recommendation |
|---|---|
| `logs/` | Preserve in recovery archive, then remove from worktree or add to ignore policy if appropriate. |
| `var/` | Preserve in recovery archive, then remove from worktree unless repo intentionally tracks generated reports. |
| `test_timing.csv` | Preserve then remove; do not commit by default. |
| `docs/reports/evidence/` | Exists as of review reconciliation. Review carefully. If this is retained audit evidence, commit only curated summaries and stable evidence manifests. |
| `.governance/active-gates.*` | Review separately from backups. Candidate for repo policy if active gate state is intentionally shared. |
| `.governance/backups/*` | Preserve outside git; do not commit timestamped backup churn by default. |

No generated artifact should be removed until Phase 0 recovery artifacts are complete.

### Phase 6 - Product-Code Extraction Slices

Goal: turn broad dirty source changes into reviewable, testable PR-sized work.

The current source diff spans many domains. The cleanup should not start by staging all `src` changes. Instead, extract in this order:

#### Slice 6A - Market Import / Strength Runtime

Candidate files:

```text
src/market/strength_runtime.rs
tests/market_strength_calculation_test.rs
tests/miniqmt_market_import_handler_test.rs
tests/miniqmt_market_manifest_test.rs
related src/cli import/market handler files
related docs/operations or miniQMT docs
```

Reasoning:

- Contains new untracked Rust source and tests.
- Likely a coherent miniQMT market dataset consumer feature.
- Should be isolated before touching broad strategy/execution code.

Required gates:

```text
cargo fmt --check
RUSTFLAGS=-Awarnings cargo test --test market_strength_calculation_test --quiet
RUSTFLAGS=-Awarnings cargo test --test miniqmt_market_import_handler_test --quiet
RUSTFLAGS=-Awarnings cargo test --test miniqmt_market_manifest_test --quiet
GitNexus detect_changes(scope=all) on the slice worktree
```

#### Slice 6B - CLI Import / Validation Surface

Candidate files:

```text
src/cli/command_types.rs
src/cli/tests/import.rs
src/cli/commands/*
src/cli/handlers/data_handler.rs
src/import/*
selected market CLI validation tests
```

Reasoning:

- CLI changes are broad and may be mixed with several domain changes.
- They should be narrowed after Slice 6A identifies which import/market edits are actually needed.

Required gates:

```text
cargo fmt --check
RUSTFLAGS=-Awarnings cargo test --lib --quiet
focused CLI/import integration tests
GitNexus impact on modified handler symbols
GitNexus detect_changes(scope=all)
```

#### Slice 6C - Execution / Strategy Runtime

Candidate files:

```text
src/execution/*
src/strategy/*
tests/execution_kernel_test.rs
tests/execution_runtime_store_test.rs
tests/strategy_daemon_test.rs
tests/strategy_integration_test.rs
tests/strategy_mock_live_run_test.rs
tests/strategy_paper_run_test.rs
```

Reasoning:

- This is high-risk and crosses runtime order state, strategy daemon flow, QMT live flow, and mock/paper execution.
- Several related concepts have already landed via PRs #73-#76, so these local changes may be duplicates or stale predecessors.

Required action before coding:

1. Compare each dirty file to current `origin/master`.
2. Identify whether the change is:
   - already landed,
   - superseded,
   - still missing,
   - test-only drift,
   - generated formatting churn.
3. Run GitNexus impact before editing any function or method.

Suggested gates:

```text
cargo fmt --check
RUSTFLAGS=-Awarnings cargo test --test execution_kernel_test --quiet
RUSTFLAGS=-Awarnings cargo test --test execution_runtime_store_test --quiet
RUSTFLAGS=-Awarnings cargo test --test strategy_daemon_test --quiet
RUSTFLAGS=-Awarnings cargo test --test strategy_integration_test --quiet
RUSTFLAGS=-Awarnings cargo test --test strategy_mock_live_run_test --quiet
RUSTFLAGS=-Awarnings cargo test --test strategy_paper_run_test --quiet
```

#### Slice 6D - Risk / Industry / Live Import

Candidate files:

```text
src/risk/importer.rs
src/risk/industry_sync.rs
src/risk/rebuild.rs
src/risk/service.rs
tests/risk_volatility_test.rs
tests/stop_service_test.rs
```

Reasoning:

- Risk industry behavior was already heavily evolved on current `master`.
- The dirty local changes must be checked against the superseded PR stack #47-#51 and current tests before extraction.

Required gates:

```text
cargo fmt --check
RUSTFLAGS=-Awarnings cargo test --test risk_service_test --quiet
RUSTFLAGS=-Awarnings cargo test --test risk_volatility_test --quiet
RUSTFLAGS=-Awarnings cargo test --test stop_service_test --quiet
GitNexus detect_changes(scope=all)
```

#### Slice 6E - Broad Library Hygiene

Candidate modules:

```text
src/ai/*
src/analysis/*
src/fundamental/*
src/news/*
src/account/models.rs
benches/bench_main.rs
Cargo.toml / Cargo.lock
```

Reasoning:

- These changes may be derive additions, warning cleanup, or API consistency work.
- They should be handled only after the main user-facing/runtime slices are separated.
- `Cargo.toml` adds `sha2 = "0.10"` and must be tied to the exact feature that needs it.

Required gates:

```text
cargo fmt --check
RUSTFLAGS=-Awarnings cargo test --lib --quiet
focused tests for the touched module family
```

## Proposed Review Order

Recommended order for human approval:

1. Approve Phase 0 snapshot procedure.
2. Decide repo-vs-local policy for `.claude/`, `.codex/`, `.governance/`, `.mcp.json`, and `openspec/config.yaml`.
3. Approve docs-only extraction slices:
   - architecture audit docs,
   - code audit methodology docs,
   - miniQMT reports/runbook,
   - function-tree/roadmap reconciliation.
4. Approve generated artifact disposal policy.
5. Pick the first code slice to extract. Recommended first code slice: Slice 6A Market Import / Strength Runtime.
6. Defer execution/strategy/risk extraction until after duplicate/superseded checks against current `origin/master`.

## Approval Protocol

Use explicit phase approvals rather than one blanket approval for the whole cleanup. If multiple operators or agents are active, designate exactly one Executor for write operations and keep the root worktree as the salvage source until a later root realignment is separately approved.

| Action | Approval Required | Approval Scope |
| --- | --- | --- |
| Phase 0 snapshot | Yes | Inventory refresh and recovery artifact creation under the approved recovery directory only. |
| Phase 1 clean worktree creation | Yes | Named worktree path and branch name only. No root `master` reset, rebase, or cleanup. |
| Documentation extraction slices | Yes, per slice | Specific source files, target branch/worktree, and review gate for each slice. |
| Generated artifact disposal | Yes, per path group | Deleting or moving `logs/`, `var/`, `.governance/backups/*`, or `docs/reports/evidence/` requires explicit path-level approval after preservation is verified. |
| Product-code extraction | Yes, per domain | Domain scope, GitNexus impact check before symbol edits, and `gitnexus_detect_changes` verification before any commit. |
| Root `master` realignment | Separate explicit approval | Highest-risk operation. Only after selected slices are extracted, reviewed, and recovery artifacts are verified usable. |

## Commands To Avoid Until Approved

Do not run these against the dirty root worktree until the snapshot exists and the user approves:

```text
git reset --hard
git checkout -- .
git restore .
git clean -fd
git stash push --include-untracked
rm -rf logs var .governance/backups
git branch -D ...
git worktree remove ...
```

`git stash push --include-untracked` is reversible in normal cases, but it still rewrites the visible working state. Treat it as an approved cleanup action, not an analysis action.

## Proposed Commit / PR Slices

If approved, the cleanup should produce small PRs or commits similar to:

1. `docs: add architecture audit deliverables`
2. `docs: add code audit methodology`
3. `docs: archive miniqmt evidence closeouts`
4. `docs: reconcile function tree and roadmap status sources`
5. `chore: add agent function-tree command surfaces`
6. `feat: add miniqmt market manifest import surface`
7. `feat: add market strength runtime`
8. `test: cover market import and strength runtime`
9. `fix: align execution strategy runtime drift` only if unique changes remain after duplicate analysis
10. `chore: drop generated local artifacts` only after snapshot and explicit approval

## Open Questions For Review

1. Should `.claude/commands/ft/*` and `.codex/commands/ft/*` be committed as first-class repo tooling?
2. Should `.governance/active-gates.*` be committed, while `.governance/backups/*` stays untracked?
3. Is `openspec/config.yaml` intended to be repository config?
4. Should `ROADMAP.md`, `docs/DEVELOPMENT_ROADMAP.md`, and `docs/ROADMAP_REVIEW.md` be deleted, archived, or restored as pointers to `FUNCTION_TREE.md`?
5. Are the miniQMT closeout reports final deliverables or temporary evidence?
6. Should `docs/reports/evidence/` be committed, compressed, moved under an evidence archive, or excluded?
7. Which code slice should be extracted first: market import/strength, CLI import, execution/strategy, or risk?
8. Who are the Approver, Executor, and Reviewer for Phase 0 and Phase 1?
9. Should CI, file watchers, or other background automation be paused during the snapshot and worktree creation window?

## Acceptance Criteria For Cleanup Completion

Cleanup is complete only when all of the following are true:

1. Root `master` is clean and aligned with `origin/master`, or intentionally on a documented branch.
2. No untracked generated artifacts remain unless documented in `.gitignore` or committed by policy.
3. Each retained source change is in a small reviewed slice with focused tests.
4. `FUNCTION_TREE.md`, `README.md`, and `CHANGELOG.md` do not conflict on current feature status.
5. GitNexus `detect_changes` for each source slice is reviewed and expected.
6. Required gates for each slice pass.
7. Stashes are preserved until the user explicitly authorizes deletion.
8. This cleanup plan is either marked superseded by the final closeout or archived with final decisions.

## Recommended Next Action

Approve Phase 0 only.

Phase 0 produces recovery artifacts and makes subsequent cleanup safe. It does not decide which changes to keep or delete. After Phase 0, review the generated inventories and then choose whether to extract docs slices or code slices first.
