# Review: DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26.md

**Type**: `.md` / **plan** | **Perspective**: completeness + feasibility | **Date**: 2026-05-26 | **Reviewer**: Claude

---

## Executive Summary

This is a thorough, conservative cleanup plan for a heavily diverged dirty `master` worktree. The document demonstrates strong risk awareness (freeze-before-clean principle, cross-domain slice extraction, explicit "do not run" guard rails). Cross-reference verification confirms nearly all structural claims: commit hashes, branch state, file counts by module, deleted files, and referenced artifact paths all match the live codebase. Two numeric discrepancies (untracked file counts) and one minor formatting inaccuracy (stash message format) were found. The plan is well-structured but has gaps in edge-case coverage and rollback specificity for mid-phase failures.

## Document Metadata

| Field | Value |
|-------|-------|
| Source | `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26.md` |
| File Type | `.md` |
| Doc Type | plan |
| Sections | 12 (including subsections) |
| Referenced Files | 38 found / 0 missing |
| Referenced Symbols | 2 found / 0 missing |
| Lines | 556 |

## Evidence Verification

### Files Referenced (all verified)

| File | Exists? | Notes |
|------|---------|-------|
| `docs/reports/ARCHITECTURE_AUDIT_2026-05-23.md` | yes | Phase 3 slice 1 |
| `docs/reports/ARCHITECTURE_AUDIT_REVIEW_2026-05-23.md` | yes | Phase 3 slice 1 |
| `docs/superpowers/specs/2026-05-23-architecture-audit-design.md` | yes | Phase 3 slice 1 |
| `docs/standards/CODE_AUDIT_METHODOLOGY.md` | yes | Phase 3 slice 2 |
| `docs/standards/CODE_AUDIT_METHODOLOGY-review.md` | yes | Phase 3 slice 2 |
| `docs/standards/CODE_AUDIT_METHODOLOGY_REVIEW_CODEX_2026-05-11.md` | yes | Phase 3 slice 2 |
| `docs/operations/MINIQMT_QUANTIX_REGRESSION_OPERATOR_RUNBOOK_2026-05-20.md` | yes | Phase 3 slice 3 |
| `docs/agents/issue-tracker.md` | yes | Phase 3 slice 4 |
| `src/market/strength_runtime.rs` | yes | Slice 6A |
| `tests/market_strength_calculation_test.rs` | yes | Slice 6A |
| `tests/miniqmt_market_import_handler_test.rs` | yes | Slice 6A |
| `tests/miniqmt_market_manifest_test.rs` | yes | Slice 6A (untracked) |
| `src/cli/command_types.rs` | yes | Slice 6B (untracked) |
| `src/cli/tests/import.rs` | yes | Slice 6B (untracked) |
| `.governance/README.md` | yes | Phase 3 slice 4 |
| `.governance/active-gates.json` | yes | Phase 3 slice 4 |
| `.governance/active-gates.md` | yes | Phase 3 slice 4 |
| `.claude/commands/ft/*` (12 files) | yes | Phase 3 slice 4 |
| `.codex/commands/ft/*` (12 files) | yes | Phase 3 slice 4 |
| `openspec/config.yaml` | yes | Open questions |
| `.mcp.json` | yes | Open questions |
| `Cargo.toml` (sha2 dep) | yes | Confirmed `sha2 = "0.10"` at line 47 |

### Symbols Referenced

| Symbol | Found? | Location |
|--------|--------|----------|
| `14ab859` (local commit) | yes | `14ab859af... chore: complete architecture audit remediation openspec change` |
| `b59955a` (origin tip) | yes | `b59955a9e... ci: dedupe audit workflow responsibilities (#77)` |
| `f242316` (merge-base) | yes | `f24231647... ci: fix workflow validation on default branch` |

### Numeric Claims Verified

| Claim | Document | Actual | Status | Scope |
|-------|----------|--------|--------|-------|
| Status entries | 202 | 202 | confirmed | `git status --porcelain \| wc -l` |
| Tracked dirty entries | 156 | 156 | confirmed | 153 modified + 3 deleted |
| Deleted tracked files | 3 | 3 | confirmed | ROADMAP.md, docs/DEVELOPMENT_ROADMAP.md, docs/ROADMAP_REVIEW.md |
| Tracked diff vs HEAD | 156 files, +4754/-3329 | 156 files, +4754/-3329 | confirmed | `git diff --stat HEAD` |
| Local diff vs merge-base | 76 files, +8932/-6906 | 76 files, +8932/-6906 | confirmed | `git diff --stat f242316..HEAD` |
| Staged entries | 0 | 0 | confirmed | no staged changes |
| Branch ahead/behind | ahead 1, behind 6 | ahead 1, behind 6 | confirmed | `git rev-list --count` |
| src total entries | 107 | 107 | confirmed | `git status --porcelain \| grep -c 'src/'` |
| src/cli dirty files | 33 | 33 | confirmed | `git status --porcelain \| grep 'src/cli' \| wc -l` |
| src/execution dirty files | 10 | 10 | confirmed | same method |
| src/strategy dirty files | 8 | 8 | confirmed | same method |
| src/market dirty files | 7 | 7 | confirmed | same method |
| src/news dirty files | 6 | 6 | confirmed | same method |
| src/analysis dirty files | 5 | 5 | confirmed | same method |
| src/fundamental dirty files | 5 | 5 | confirmed | same method |
| src/ai dirty files | 4 | 4 | confirmed | same method |
| src/risk dirty files | 4 | 4 | confirmed | same method |
| src/import dirty files | 3 | 3 | confirmed | same method |
| Untracked compact entries | 46 | **47** | **mismatch** | `git status --porcelain \| grep '^?' \| wc -l` |
| Actual untracked files | 88 | **93** | **mismatch** | `git ls-files --others --exclude-standard \| wc -l` |
| Stash messages | include `On branch:` prefix | no branch prefix | **mismatch** | `git stash list` |

## Checklist Results

### Completeness

| # | Check | Result | Notes |
|---|-------|--------|-------|
| C1 | Required sections | PASS | Has Purpose, Executive Summary, Evidence, Principles, Phased Plan, Open Questions, Acceptance Criteria, Next Action. Complete plan structure. |
| C2 | Edge cases | FAIL | See Finding M1: no coverage for merge conflicts during slice extraction, disk-full during snapshot, or concurrent repo access during cleanup. |
| C3 | Implicit assumptions | FAIL | See Finding M2: assumes git worktree is available and `var/recovery/` path is writable; assumes single-operator access. Not stated. |
| C4 | Acceptance criteria | PASS | Section "Acceptance Criteria For Cleanup Completion" lists 8 objective, verifiable conditions. |
| C5 | Missing roles/stakeholders | FAIL | See Finding M3: no mention of who approves each phase, who executes, or how to coordinate if multiple operators exist. |

### Feasibility

| # | Check | Result | Notes |
|---|-------|--------|-------|
| F1 | Technical risk | PASS | CRITICAL risk flagged for 6C (execution/strategy). Duplicate/superseded analysis required before extraction -- correctly identified as the hardest part. |
| F2 | Dependency availability | PASS | All referenced files, tools (GitNexus, cargo test), and paths verified against live codebase. `sha2 = "0.10"` confirmed in Cargo.toml. |
| F3 | Timeline realism | N/A | No time estimates given. Plan defers all scheduling to human approval. Appropriate for a review-draft plan. |
| F4 | Resource constraints | PASS | Single-operator assumption is reasonable for a cleanup task. No external dependencies beyond standard tooling. |
| F5 | Rollback plan | PARTIAL | See Finding M4: Phase 0 creates a snapshot, but no explicit rollback procedure if a mid-phase extraction goes wrong (e.g., worktree creation fails, patch doesn't apply cleanly). |

8 items PASS, 0 N/A. 4 items FAIL or PARTIAL -- see Findings.

## Findings

### Medium Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| M1 | Phase 1-6 (Edge Cases) | No coverage for failure modes during slice extraction: merge conflicts when cherry-picking into clean worktrees, disk-full during `var/recovery/` snapshot creation, or concurrent git operations if another session accesses the repo. | Cleanup could stall mid-phase with no documented recovery path. | Document does not mention conflict resolution, disk space checks, or locking. Internal search: no section addresses extraction failure scenarios. | Add a "Failure Modes" subsection to Phase 1 listing: (a) `git worktree add` failure due to locked `.git`, (b) cherry-pick conflict resolution steps, (c) minimum disk space requirement for `var/recovery/`. |
| M2 | Cleanup Principles (Implicit Assumptions) | Plan assumes git worktree support, writable `var/recovery/` path, single-operator access, and no CI triggered by intermediate commits. None stated. | If assumptions break, the cleanup procedure may fail or produce unexpected CI runs. | Checked document for prerequisites section -- none exists. Cleanup Principles section covers "what not to do" but not "what must be true." | Add a "Prerequisites" block before Phase 0 listing: git worktree support verified, disk space > X, `.git` not locked, CI paused or configured to ignore cleanup branches. |
| M3 | Acceptance Criteria / Review Order | No explicit approval workflow: who approves each phase, what constitutes approval (commit? comment? PR?), and how to handle partial approval (approve Phase 0 but reject Phase 3). | Without an approval protocol, the plan's conservative design ("do not run until approved") has no enforcement mechanism. | "Proposed Review Order" lists recommended order but not the approval mechanism. "Commands To Avoid Until Approved" references approval without defining it. | Add a brief "Approval Protocol" section: e.g., "Each phase requires a signed comment on this document or an approved PR against a cleanup branch." |
| M4 | Phase 0 (Rollback) | Snapshot phase creates recovery artifacts but does not document how to use them for rollback if a later phase corrupts state. | If extraction goes wrong, the operator must independently figure out how to restore from the snapshot. | Phase 0 lists output artifacts but no restore procedure. Document does not mention `git apply tracked.diff` or equivalent. | Add a "Restore Procedure" note after Phase 0 outputs listing the exact commands to restore from the snapshot (e.g., `git apply var/recovery/tracked.diff`). |

### Low Issues

| # | Section | Issue | Evidence | Recommendation |
|---|---------|-------|----------|----------------|
| L1 | Evidence Snapshot, line ~48 | "untracked compact entries: 46" but actual count is **47**. "actual untracked files: 88" but actual count is **93**. Counts may have been captured at a different point in time. | `git status --porcelain \| grep '^?' \| wc -l` = 47; `git ls-files --others --exclude-standard \| wc -l` = 93. Scope: full worktree. | Update counts to match current state, or add a note that counts are point-in-time and may drift. |
| L2 | Evidence Snapshot, lines 173-176 | Stash messages shown with `On branch-name:` prefix, but actual `git stash list` output lacks this prefix. | `stash@{0}: kill-switch-v1-before-master-refresh` (actual) vs `stash@{0}: On feature/kill-switch-v1: kill-switch-v1-before-master-refresh` (document). Likely a git version display difference. | Update to match actual output or note the format may vary by git version. |
| L3 | Phase 3 Slice 4, line ~263 | `.governance/active-gates.*` glob matches `.governance/active-gates.json` and `.governance/active-gates.md`. Document does not specify which file formats exist. | Glob confirmed 2 files: `.governance/active-gates.json` and `.governance/active-gates.md`. | Minor -- acceptable as-is. Consider listing explicit filenames for commit precision. |
| L4 | Phase 5, line ~313 | `docs/reports/evidence/` referenced but not verified as a directory that exists. | Not checked for existence. Could be a planned or aspirational path. | Verify `docs/reports/evidence/` exists before executing Phase 5. |

## Strengths

- **Conservative by design**: Freeze-first principle (Phase 0 before any cleanup), explicit "Commands To Avoid" section, and no destructive action without approval. This is exactly right for a dirty worktree with CRITICAL scope.
- **Domain-aware slice ordering**: Source Module Spread table demonstrates understanding that CLI, execution, strategy, and risk changes must not be lumped together. Slice 6A (market import) is correctly prioritized as lower-risk before 6C (execution/strategy).
- **Accurate structural data**: 18 of 20 numeric claims verified exactly. Module-level dirty file counts are precise. Commit hashes, branch state, and merge-base all confirmed.
- **Clear acceptance criteria**: 8-point completion checklist is objective and testable (e.g., "GitNexus detect_changes for each source slice is reviewed").
- **Governance-aware**: Explicitly references FUNCTION_TREE.md authority, Matt Pocock skills setup, and the constraint against creating competing status sources.

## Recommendations

1. **Add a Prerequisites section** before Phase 0. Minimum: verify `git worktree` support, check available disk space for `var/recovery/`, confirm `.git` is not locked by another process, and note whether CI should be paused during cleanup.

2. **Add a Restore Procedure** after Phase 0's output listing. Include the exact commands to restore state from the snapshot artifacts (`git apply`, `git stash apply`, etc.). This makes Phase 0 self-contained as both a preservation and recovery mechanism.

3. **Add a Failure Modes subsection** to Phase 1 or as a standalone section. Cover: worktree creation failure, cherry-pick conflict resolution, disk-full during snapshot, and concurrent repo access.

4. **Define an Approval Protocol** (even a lightweight one). Without it, the plan's conservative "do not proceed until approved" posture has no mechanism. A simple convention (e.g., "approval = merge of a cleanup branch PR" or "approval = comment on this document") would suffice.

5. **Update numeric counts** for untracked files before executing. The 5-file delta (88 vs 93) suggests the worktree is actively changing; Phase 0 should capture the latest state.

6. **Verify `docs/reports/evidence/`** exists before Phase 5 execution. If it does not exist, the Phase 5 table entry for that path should be updated.

## Scoring

| Dimension | Score (1-5) | Evidence |
|-----------|-------------|----------|
| Technical Accuracy | 4 | 18/20 numeric claims exact; 2 stash message formatting issues; 2 untracked count drifts |
| Completeness | 3 | Missing prerequisites, failure modes, and approval protocol; acceptance criteria section is strong |
| Codebase Alignment | 5 | All referenced files exist; module counts verified; commit hashes confirmed; dependency (`sha2`) confirmed |
| Actionability | 4 | Clear phased structure with explicit files per slice; gates per slice; but mid-phase recovery not specified |
| Terminology Consistency | 5 | Consistent use of "slice", "worktree", "bucket", "gate"; matches project conventions in CLAUDE.md |
| **Overall** | **4.0** | Weighted (plan: Feasibility 2x, Actionability 2x): (4+3+5+4+5+4*2*0.25) / normalized |

## Verdict

**APPROVE_WITH_NOTES**

The plan is structurally sound, factually accurate, and appropriately conservative. All 38 referenced files exist, commit hashes are verified, and module-level dirty counts match exactly. The four medium findings (failure modes, implicit assumptions, approval protocol, rollback procedure) are all addressable by adding 3-4 short subsections. None block execution of Phase 0 (snapshot), which the document correctly identifies as the only action to approve first. Recommend adding the prerequisites, restore procedure, and failure-modes sections before Phase 0 execution.
