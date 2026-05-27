# Dirty Worktree Recheck Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Recheck and close the remaining dirty-worktree residuals using `docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE_REVIEW.md` as the review baseline.

**Architecture:** This is a documentation/governance cleanup pass, not a product-code change. The root worktree is already aligned with `origin/master`; the remaining work is to classify untracked residuals, fix any documentation gaps in the dirty-worktree guide set, and either promote, preserve, or explicitly defer each untracked path without blanket deletion.

**Tech Stack:** Git, Markdown documentation, OpenSpec archive artifacts, project recovery snapshot under `var/recovery/`.

---

## Current Baseline

- Root worktree: `/opt/claude/quantix-rust`
- Current branch: `master`
- `HEAD`: `d687aad`
- `origin/master`: `d687aad`
- Tracked diff: none
- Compact status entries: 20 untracked entries
- Actual untracked files: 33
- Review baseline: `docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE_REVIEW.md`
- Implementation guide under review: `docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md`

## File Structure

- Modify: `docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md`
  - Only if a second pass finds remaining review gaps or stale references.
- Keep or promote: `docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE_REVIEW.md`
  - Review evidence for the guide.
- Keep or promote: `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26.md`
  - Original cleanup plan evidence.
- Keep or promote: `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-review.md`
  - Review evidence for the cleanup plan.
- Keep or promote: `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-meta-review.md`
  - Meta-review evidence.
- Review separately: `docs/architecture/function-add-next.md`
  - Architecture document unrelated to dirty-worktree guide repair.
- Review separately: `docs/architecture/function-add-next-feasibility-report.md`
  - Architecture feasibility report unrelated to dirty-worktree guide repair.
- Review separately: `docs/opendog-mcp-test-report-2026-05-10.md`
  - Standalone test report.
- Review separately: `docs/superpowers/plans/*.md`, `docs/superpowers/reviews/*.md`, `docs/superpowers/specs/*.md`
  - Prior planning/spec/review artifacts; do not mix into the dirty-worktree guide slice unless explicitly approved.
- Preserve: `var/recovery/dirty-master-2026-05-26/*`
  - Recovery snapshot. Do not delete without path-level approval.
- Keep local or explicitly ignore: `.mcp.json`
  - Local MCP config. Do not commit unless reviewed for machine-specific or sensitive data.

## Task 1: Reconfirm Baseline And No Tracked Drift

- [ ] **Step 1: Recompute status counts**

Run:

```bash
git status --porcelain=v1
git rev-parse --short HEAD
git rev-parse --short origin/master
git diff --stat origin/master
git ls-files --others --exclude-standard
```

Expected:

```text
HEAD == origin/master
git diff --stat origin/master prints no tracked file stat
status contains only untracked residuals
```

- [ ] **Step 2: Record the updated count**

If the count differs from the current baseline, update this plan's `Current Baseline` section before proceeding.

## Task 2: Verify Guide Review Findings Are Actually Closed

- [ ] **Step 1: Check numbering and section ownership**

Inspect `docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md` for these headings:

```text
## 0. Freeze
## 1. Inventory
## 2. Recovery Snapshot
## 3. Clean Review Worktree
## 4. Slice Extraction
## 5. Slice Validation
## 6. PR And Commit Strategy
## 7. Root Tracked Realignment
## 8. Residual Untracked Disposition
## 9. Final Cleanup
```

Expected:

```text
All headings exist.
No top-level "## 10. Residual Untracked Disposition".
No top-level "## 11. Final Cleanup".
Explicit Approval Protocol is under 0. Freeze.
Generated And Runtime Artifact Validation is under 5. Slice Validation.
```

- [ ] **Step 2: Check review-specific content**

Confirm `docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md` includes:

```text
phase0-manifest.json minimum schema
restore-instructions.md minimum template
git diff --stat origin/master in post-realignment validation
clean review worktree cleanup template with git worktree remove
generic project test commands rather than Rust-only guidance
```

Expected:

```text
No documentation edit is needed if all checks pass.
If any check fails, patch only docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md.
```

## Task 3: Decide The Dirty-Worktree Documentation Slice

- [ ] **Step 1: Treat these five files as one candidate docs/governance slice**

Files:

```text
docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md
docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE_REVIEW.md
docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26.md
docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-review.md
docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-meta-review.md
```

Expected disposition:

```text
Promote together as the dirty-worktree cleanup documentation/evidence slice, unless content review finds sensitive or obsolete material.
```

- [ ] **Step 2: Validate links and obvious placeholders**

Run:

```bash
rg -n "TODO|TBD|FIXME|\\[\\]|<[^>]+>" docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE_REVIEW.md docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26.md docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-review.md docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-meta-review.md
```

Expected:

```text
Only intentional placeholders in templates, such as YYYY-MM-DD or <git-sha>, remain.
No accidental TODO/TBD/FIXME text remains.
```

## Task 4: Classify Non-Dirty-Cleanup Documentation Residuals

- [ ] **Step 1: Keep these files out of the dirty-worktree guide slice**

Files:

```text
docs/architecture/function-add-next-feasibility-report.md
docs/architecture/function-add-next.md
docs/opendog-mcp-test-report-2026-05-10.md
docs/superpowers/plans/2026-05-03-qmt-live-query-reconciliation-hardening-implementation.md
docs/superpowers/plans/2026-05-09-factor-p1-first-slice-implementation.md
docs/superpowers/plans/2026-05-21-miniqmt-direct-clickhouse-read-only-comparison.md
docs/superpowers/plans/2026-05-24-function-tree-governance-skill-plan-review.md
docs/superpowers/plans/2026-05-24-function-tree-governance-skill-plan.md
docs/superpowers/reviews/2026-05-02-qmt-live-reconciliation-hardening-design-review.md
docs/superpowers/specs/2026-05-09-factor-p1-first-slice-design-review.md
docs/superpowers/specs/2026-05-09-factor-p1-first-slice-design.md
docs/superpowers/specs/2026-05-15-code-audit-execution-spec-review.md
docs/superpowers/specs/2026-05-18-miniqmt-controlled-evidence-alignment-spec.md
```

Expected:

```text
They are classified as separate docs/spec/plan residuals.
No commit mixes them with the dirty-worktree guide slice.
No deletion occurs without path-level approval.
```

- [ ] **Step 2: Produce a disposition table if execution continues**

Write a compact table with:

```text
Path | Class | Evidence | Risk | Recommended disposition | Approval needed
```

Expected:

```text
The table is written to a docs/report file before any promote/delete decision.
```

## Task 5: Preserve Recovery Snapshot And Local Config

- [ ] **Step 1: Preserve recovery snapshot**

Files:

```text
var/recovery/dirty-master-2026-05-26/branch-list.txt
var/recovery/dirty-master-2026-05-26/local-head-show.txt
var/recovery/dirty-master-2026-05-26/phase0-manifest.json
var/recovery/dirty-master-2026-05-26/rescue-branch.txt
var/recovery/dirty-master-2026-05-26/restore-instructions.md
var/recovery/dirty-master-2026-05-26/stash-list.txt
var/recovery/dirty-master-2026-05-26/status-porcelain.txt
var/recovery/dirty-master-2026-05-26/tracked-stat.txt
var/recovery/dirty-master-2026-05-26/tracked.diff
var/recovery/dirty-master-2026-05-26/untracked-files.tar
var/recovery/dirty-master-2026-05-26/untracked-files.txt
var/recovery/dirty-master-2026-05-26/untracked-sha256.txt
var/recovery/dirty-master-2026-05-26/untracked-sizes.txt
var/recovery/dirty-master-2026-05-26/worktree-list.txt
```

Expected:

```text
No deletion or move.
Any future removal requires explicit path-level approval.
```

- [ ] **Step 2: Review `.mcp.json` as local config**

Run:

```bash
git status --porcelain=v1 -- .mcp.json
```

Expected:

```text
.mcp.json remains untracked unless the user explicitly approves committing or ignoring it.
```

## Task 6: Optional Repair Edits

- [ ] **Step 1: Patch guide only if Task 2 finds a gap**

Modify:

```text
docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md
```

Expected:

```text
Edits are limited to review-gap repair.
No unrelated wording cleanup.
```

- [ ] **Step 2: Validate Markdown structure**

Run:

```bash
rg -n "^## (10|11)\\." docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md
rg -n "5 Slice Extraction, 8 PR And Commit Strategy|6 Product Code Rules|10 Residual Untracked|11 Final Cleanup" docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md
```

Expected:

```text
Both commands return no stale numbering matches.
```

## Task 7: Verification And Scope Check

- [ ] **Step 1: Verify no product-code files changed**

Run:

```bash
git diff --name-only
git status --porcelain=v1
```

Expected:

```text
Tracked diff is empty or docs-only if guide repair was needed.
Remaining status is classified and intentional.
```

- [ ] **Step 2: Run GitNexus changed-scope check if any files are staged or committed**

Run through MCP:

```text
gitnexus_detect_changes(scope: "all")
```

Expected:

```text
No product-code execution flow is affected by this docs/governance pass.
```

- [ ] **Step 3: Write Graphiti conclusion memory after execution**

Write to:

```text
group_id: quantix_rust_review
```

Expected:

```text
Memory records the final review conclusion and ingest status reaches completed.
```

## Execution Recommendation

Start with Tasks 1-3. If the guide still passes all review checks, the next real repair is not code or guide editing; it is preparing a path-level disposition table and promoting the dirty-worktree guide/evidence files as one docs/governance slice while preserving recovery and local config residuals.
