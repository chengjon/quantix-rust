# Dirty Worktree Recheck Disposition - 2026-05-27

## Scope

This report records the recheck requested after
`docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE_REVIEW.md`.

The pass is limited to dirty-worktree cleanup governance and residual
classification. It does not modify product code, delete files, move files, or
stage changes.

## Baseline

| Field | Value |
| --- | --- |
| Repository | `/opt/claude/quantix-rust` |
| Branch | `master` |
| HEAD | `d687aad` |
| `origin/master` | `d687aad` |
| HEAD matches `origin/master` | Yes |
| Tracked dirty files | 0 |
| `git diff --stat origin/master` | Empty |
| Compact status entries | 21 |
| Actual untracked files | 34 |

## Guide Review Closure

`docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md` was checked against
`docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE_REVIEW.md`.

| Review item | Result |
| --- | --- |
| 0-9 top-level workflow headings | Pass |
| `Explicit Approval Protocol` nested under `0. Freeze` | Pass |
| Generated/runtime artifact validation nested under `5. Slice Validation` | Pass |
| Unified classification table | Pass |
| `phase0-manifest.json` minimum schema | Pass |
| `restore-instructions.md` minimum template | Pass |
| Clean review worktree removal template | Pass |
| Post-realignment `git diff --stat origin/master` verification | Pass |
| Generic project test appendix | Pass |
| Stale numbering references | None found |

The sensitive-pattern scan found only instructional references to `token`,
private keys, local config, and local ports in the cleanup guide. These are
expected policy examples, not leaked credentials.

No additional guide patch is required by the review baseline.

## Candidate Dirty-Worktree Documentation Slice

These files form one coherent docs/governance evidence slice:

| Path | Class | Evidence | Risk | Recommended disposition | Approval needed |
| --- | --- | --- | --- | --- | --- |
| `docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md` | Formal documentation | Implements the reviewed dirty-worktree cleanup SOP. | Low; docs-only. | Promote with this slice. | Yes, docs/governance slice approval. |
| `docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE_REVIEW.md` | Review evidence | Defines the recheck baseline used in this pass. | Low; docs-only. | Promote with this slice. | Yes, docs/governance slice approval. |
| `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26.md` | Cleanup evidence | Original dirty-worktree cleanup plan evidence. | Low; docs-only. | Promote with this slice. | Yes, docs/governance slice approval. |
| `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-review.md` | Cleanup evidence | Review evidence for the cleanup plan. | Low; docs-only. | Promote with this slice. | Yes, docs/governance slice approval. |
| `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-meta-review.md` | Cleanup evidence | Meta-review evidence for the cleanup plan. | Low; docs-only. | Promote with this slice. | Yes, docs/governance slice approval. |
| `docs/reports/DIRTY_WORKTREE_RECHECK_DISPOSITION_2026-05-27.md` | Cleanup evidence | Records this recheck and path-level disposition. | Low; docs-only. | Promote with this slice. | Yes, docs/governance slice approval. |

No accidental `TODO`, `TBD`, `FIXME`, or empty-checkbox placeholder defects
were found in the five pre-existing dirty-worktree docs/evidence files.

## Residual Untracked Classification

These paths are outside the dirty-worktree documentation slice and should not be
mixed into the same commit or PR.

| Path | Class | Evidence | Risk | Recommended disposition | Approval needed |
| --- | --- | --- | --- | --- | --- |
| `.mcp.json` | Local config | Machine-local MCP configuration. | Medium; may contain local endpoints or credentials. | Keep local unless separately reviewed for commit or ignore policy. | Yes, before commit or ignore-rule change. |
| `docs/architecture/function-add-next-feasibility-report.md` | Architecture docs | Separate architecture material. | Low; unrelated docs drift. | Separate docs slice or defer. | Yes, separate docs approval. |
| `docs/architecture/function-add-next.md` | Architecture docs | Separate architecture material. | Low; unrelated docs drift. | Separate docs slice or defer. | Yes, separate docs approval. |
| `docs/opendog-mcp-test-report-2026-05-10.md` | Other docs | Standalone test report. | Low; unrelated evidence drift. | Separate docs/evidence slice or defer. | Yes, separate docs approval. |
| `docs/superpowers/plans/2026-05-03-qmt-live-query-reconciliation-hardening-implementation.md` | Superpowers plan | Prior plan artifact. | Low; unrelated planning drift. | Separate plan slice or defer. | Yes, separate docs approval. |
| `docs/superpowers/plans/2026-05-09-factor-p1-first-slice-implementation.md` | Superpowers plan | Prior plan artifact. | Low; unrelated planning drift. | Separate plan slice or defer. | Yes, separate docs approval. |
| `docs/superpowers/plans/2026-05-21-miniqmt-direct-clickhouse-read-only-comparison.md` | Superpowers plan | Prior plan artifact. | Low; unrelated planning drift. | Separate plan slice or defer. | Yes, separate docs approval. |
| `docs/superpowers/plans/2026-05-24-function-tree-governance-skill-plan-review.md` | Superpowers plan | Prior plan artifact. | Low; unrelated planning drift. | Separate plan slice or defer. | Yes, separate docs approval. |
| `docs/superpowers/plans/2026-05-24-function-tree-governance-skill-plan.md` | Superpowers plan | Prior plan artifact. | Low; unrelated planning drift. | Separate plan slice or defer. | Yes, separate docs approval. |
| `docs/superpowers/plans/2026-05-27-dirty-worktree-recheck-cleanup.md` | Superpowers plan | This recheck execution plan. | Low; supports this cleanup pass. | Promote with the dirty-worktree docs slice or keep as planning evidence. | Yes, docs/governance slice approval. |
| `docs/superpowers/reviews/2026-05-02-qmt-live-reconciliation-hardening-design-review.md` | Superpowers review | Prior review artifact. | Low; unrelated review drift. | Separate review slice or defer. | Yes, separate docs approval. |
| `docs/superpowers/specs/2026-05-09-factor-p1-first-slice-design-review.md` | Superpowers spec/review | Prior spec review artifact. | Low; unrelated spec drift. | Separate spec slice or defer. | Yes, separate docs approval. |
| `docs/superpowers/specs/2026-05-09-factor-p1-first-slice-design.md` | Superpowers spec | Prior spec artifact. | Low; unrelated spec drift. | Separate spec slice or defer. | Yes, separate docs approval. |
| `docs/superpowers/specs/2026-05-15-code-audit-execution-spec-review.md` | Superpowers spec/review | Prior spec review artifact. | Low; unrelated spec drift. | Separate spec slice or defer. | Yes, separate docs approval. |
| `docs/superpowers/specs/2026-05-18-miniqmt-controlled-evidence-alignment-spec.md` | Superpowers spec | Prior spec artifact. | Low; unrelated spec drift. | Separate spec slice or defer. | Yes, separate docs approval. |

## Recovery Snapshot

The recovery snapshot remains protected. It is not safe to delete in this pass.

| Path | Class | Evidence | Risk | Recommended disposition | Approval needed |
| --- | --- | --- | --- | --- | --- |
| `var/recovery/dirty-master-2026-05-26/branch-list.txt` | Recovery snapshot | Phase 0 recovery package. | High if deleted before cleanup closure. | Preserve untouched. | Yes, explicit path-level approval before removal. |
| `var/recovery/dirty-master-2026-05-26/local-head-show.txt` | Recovery snapshot | Phase 0 recovery package. | High if deleted before cleanup closure. | Preserve untouched. | Yes, explicit path-level approval before removal. |
| `var/recovery/dirty-master-2026-05-26/phase0-manifest.json` | Recovery snapshot | Phase 0 recovery manifest. | High if deleted before cleanup closure. | Preserve untouched. | Yes, explicit path-level approval before removal. |
| `var/recovery/dirty-master-2026-05-26/rescue-branch.txt` | Recovery snapshot | Phase 0 recovery package. | High if deleted before cleanup closure. | Preserve untouched. | Yes, explicit path-level approval before removal. |
| `var/recovery/dirty-master-2026-05-26/restore-instructions.md` | Recovery snapshot | Phase 0 restore instructions. | High if deleted before cleanup closure. | Preserve untouched. | Yes, explicit path-level approval before removal. |
| `var/recovery/dirty-master-2026-05-26/stash-list.txt` | Recovery snapshot | Phase 0 recovery package. | High if deleted before cleanup closure. | Preserve untouched. | Yes, explicit path-level approval before removal. |
| `var/recovery/dirty-master-2026-05-26/status-porcelain.txt` | Recovery snapshot | Phase 0 status inventory. | High if deleted before cleanup closure. | Preserve untouched. | Yes, explicit path-level approval before removal. |
| `var/recovery/dirty-master-2026-05-26/tracked-stat.txt` | Recovery snapshot | Phase 0 tracked diff metadata. | High if deleted before cleanup closure. | Preserve untouched. | Yes, explicit path-level approval before removal. |
| `var/recovery/dirty-master-2026-05-26/tracked.diff` | Recovery snapshot | Phase 0 tracked diff recovery artifact. | High if deleted before cleanup closure. | Preserve untouched. | Yes, explicit path-level approval before removal. |
| `var/recovery/dirty-master-2026-05-26/untracked-files.tar` | Recovery snapshot | Phase 0 untracked archive. | High if deleted before cleanup closure. | Preserve untouched. | Yes, explicit path-level approval before removal. |
| `var/recovery/dirty-master-2026-05-26/untracked-files.txt` | Recovery snapshot | Phase 0 untracked inventory. | High if deleted before cleanup closure. | Preserve untouched. | Yes, explicit path-level approval before removal. |
| `var/recovery/dirty-master-2026-05-26/untracked-sha256.txt` | Recovery snapshot | Phase 0 checksum inventory. | High if deleted before cleanup closure. | Preserve untouched. | Yes, explicit path-level approval before removal. |
| `var/recovery/dirty-master-2026-05-26/untracked-sizes.txt` | Recovery snapshot | Phase 0 size inventory. | High if deleted before cleanup closure. | Preserve untouched. | Yes, explicit path-level approval before removal. |
| `var/recovery/dirty-master-2026-05-26/worktree-list.txt` | Recovery snapshot | Phase 0 worktree inventory. | High if deleted before cleanup closure. | Preserve untouched. | Yes, explicit path-level approval before removal. |

## Decision

- No product-code repair is needed.
- No guide patch is needed.
- No file is safe to delete now.
- `.mcp.json` and `var/recovery/dirty-master-2026-05-26/` must remain untouched
  unless the user gives explicit path-level approval.
- The next cleanup action should be a docs/governance slice for the
  dirty-worktree guide and evidence files listed above.
