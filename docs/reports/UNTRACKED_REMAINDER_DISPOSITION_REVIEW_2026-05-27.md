# Review: Remaining Untracked Worktree Items Disposition

**Type**: cleanup disposition review | **Perspective**: completeness, consistency, feasibility | **Date**: 2026-05-27 | **Reviewer**: Codex

## Summary

After PR #78 was merged and root `master` was realigned to `origin/master`, the repository has no tracked-file changes. The remaining worktree noise is limited to 19 untracked entries: one local MCP config, 17 candidate documents, and the `var/` recovery tree.

The recommended cleanup path is not a blanket delete. Keep the recovery snapshot for now, do not commit `.mcp.json`, do not commit stale cleanup-plan drafts now that the OpenSpec archive is tracked, and treat the architecture/superpowers documents as separate owner-approved documentation decisions. `docs/agents/issue-tracker.md` has been corrected in this pass to target the current repository's GitHub issue tracker.

## Current State Verified

| Check | Result | Evidence |
| --- | --- | --- |
| Root branch | `master` | `git branch --show-current` returned `master`. |
| Root head | `d8901cd16042` | Matches `origin/master`; `HEAD...origin/master` is `0 0`. |
| Tracked code/docs drift | none | `gitnexus_detect_changes(scope=all)` returned `changed_count: 0`, `risk_level: none`. |
| Remaining worktree entries | 19 untracked | `git status --porcelain=v1` reported only `??` entries. |
| Recovery snapshot | present | `var/recovery/dirty-master-2026-05-26/untracked-files.tar` exists, 1,546,240 bytes. |
| Formal cleanup record | tracked | `openspec/changes/archive/2026-05-26-dirty-worktree-cleanup-2026-05-26/` and `openspec/specs/worktree-cleanup/spec.md` exist on `origin/master`. |

## Disposition Table

| Path | Evidence | Recommendation |
| --- | --- | --- |
| `.mcp.json` | Archived OpenSpec task notes explicitly excluded `.mcp.json` as local MCP host configuration. The file also contains local endpoint/API-tool configuration. | Do not commit. Keep locally if needed, or delete only after confirming current MCP setup no longer depends on it. |
| `var/recovery/dirty-master-2026-05-26/` | Recovery snapshot and manifest are still present; prior cleanup tasks name this as the safety artifact. | Keep until the owner explicitly approves external archival or deletion. |
| `docs/agents/issue-tracker.md` | `docs/agents/domain.md` and `docs/agents/triage-labels.md` are tracked, but `issue-tracker.md` is not. It has now been corrected to use `chengjon/quantix-rust`, matching current `origin`. | Commit as a focused docs fix together with this disposition review, after final path-level approval. |
| `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26.md` | Main cleanup decisions are now represented by the tracked OpenSpec archive and spec. This draft still references stale/missing roadmap paths including `ROADMAP.md`, `docs/DEVELOPMENT_ROADMAP.md`, and `docs/ROADMAP_REVIEW.md`. | Do not commit. Preserve via recovery snapshot; delete from root after owner approval. |
| `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-review.md` | Review of the draft cleanup plan; formal accepted result is tracked in the OpenSpec archive. It also references stale/missing roadmap paths. | Do not commit. Preserve via recovery snapshot; delete from root after owner approval. |
| `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-meta-review.md` | Meta-review of the review document; depends on the untracked review draft. | Do not commit. Preserve via recovery snapshot; delete from root after owner approval. |
| `docs/architecture/function-add-next.md` | Chinese roadmap-style note, 165 lines, no Markdown heading structure and no explicit file references. | Defer. If this still matters, convert into an OpenSpec proposal or tracked architecture note in a separate docs task. |
| `docs/architecture/function-add-next-feasibility-report.md` | Feasibility report references `docs/architecture/function-add-next.md` and a missing `docs/architecture/function-add-next-feasibility-report-v2.md`. | Defer with the source note. Do not commit until the missing v2/reference intent is resolved. |
| `docs/opendog-mcp-test-report-2026-05-10.md` | Historical MCP test report; Graphiti memory notes a deferral banner was added previously. Current reference scan still found local/sample paths such as `settings.local.json`, `claude/settings.local.json`, `A.rs`, and `B.rs` missing from the repo. | Do not commit in this cleanup. Preserve via recovery snapshot; delete from root after owner approval unless an OpenDog documentation task is reopened. |
| `docs/superpowers/plans/2026-05-03-qmt-live-query-reconciliation-hardening-implementation.md` | Large historical implementation plan; tracked repo already has the related design doc `docs/superpowers/specs/2026-05-02-qmt-live-query-reconciliation-hardening-design.md`. Reference scan found missing local/runtime paths. | Defer. Do not mix into cleanup; evaluate only if historical Superpowers plans should be backfilled. |
| `docs/superpowers/reviews/2026-05-02-qmt-live-reconciliation-hardening-design-review.md` | Review document references the tracked QMT live design, but also uses shorthand file names that do not resolve directly from repo root. | Defer with the related QMT live docs family. |
| `docs/superpowers/plans/2026-05-09-factor-p1-first-slice-implementation.md` | Factor implementation plan references mostly tracked code paths, but no factor docs are currently tracked under `docs/` on `origin/master`. | Defer. If factor documentation is desired, create a separate docs PR containing the design, review, and implementation plan together. |
| `docs/superpowers/specs/2026-05-09-factor-p1-first-slice-design.md` | Design doc references relative module names such as `mod.rs`, `types.rs`, `loader.rs`, and `operators.rs` that need normalization before tracking. | Defer with the factor docs family; do not commit as-is. |
| `docs/superpowers/specs/2026-05-09-factor-p1-first-slice-design-review.md` | Review of the untracked factor design; depends on the untracked source design. | Defer with the factor docs family. |
| `docs/superpowers/specs/2026-05-15-code-audit-execution-spec-review.md` | Review of a tracked code-audit execution spec, while the repo already tracks the code-audit evidence set and final reports. Several referenced shorthand evidence files need path qualification. | Defer. If retained, normalize references and commit as a separate audit-docs follow-up. |
| `docs/superpowers/specs/2026-05-18-miniqmt-controlled-evidence-alignment-spec.md` | Main repo has tracked miniQMT runbooks and closeout reports. This spec references generated evidence names that were intentionally removed from root runtime/evidence folders and preserved by recovery snapshot. | Defer. Do not reintroduce generated evidence coupling during cleanup. |
| `docs/superpowers/plans/2026-05-21-miniqmt-direct-clickhouse-read-only-comparison.md` | References resolve to tracked files, and the closeout report is already tracked at `docs/reports/MINIQMT_DIRECT_CLICKHOUSE_READ_ONLY_COMPARISON_CLOSEOUT_2026-05-21.md`. | Optional historical backfill only. Not needed for cleanup; leave out unless owner wants a docs-history PR. |
| `docs/superpowers/plans/2026-05-24-function-tree-governance-skill-plan.md` | Function-tree governance plan is potentially valuable but references many local/relative paths that do not resolve from root as written. | Defer. If still current, convert into a dedicated governance/OpenSpec docs task. |
| `docs/superpowers/plans/2026-05-24-function-tree-governance-skill-plan-review.md` | Review depends on the untracked governance plan and references external OpenDog docs plus local shorthand paths. | Defer with the function-tree governance docs family. |

## Issues

- [x] **[RESOLVED]** `docs/agents/issue-tracker.md` was the only missing file from the Matt Pocock agent-docs trio, and its repository target did not match the current remote.
      Evidence: `origin` is `git@github.com:chengjon/quantix-rust.git`; the file now says to use `chengjon/quantix-rust` and lists `origin` as `git@github.com:chengjon/quantix-rust.git`.

- [ ] **[MED]** The three dirty-worktree cleanup report drafts are superseded by tracked OpenSpec archive artifacts.
      Evidence: `openspec/changes/archive/2026-05-26-dirty-worktree-cleanup-2026-05-26/` and `openspec/specs/worktree-cleanup/spec.md` exist on `origin/master`; the untracked report drafts are review working papers, not the accepted repository policy surface.

- [ ] **[LOW]** Several historical Superpowers/architecture documents contain stale shorthand or local-only references.
      Evidence: reference scans found examples such as missing `docs/architecture/function-add-next-feasibility-report-v2.md`, generated evidence names, local sample paths, and shorthand filenames like `operators.rs`, `export.rs`, `nodes.json`, or `active-gates.json` that do not resolve from repo root as written.

## Recommended Next Step

Use explicit path-level approvals for the remaining items:

1. Commit corrected `docs/agents/issue-tracker.md` as a focused docs fix if the current repository should be the default issue tracker.
2. Delete the three `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26*` working drafts from root if the OpenSpec archive is accepted as the authoritative record.
3. Decide whether to create a separate historical-docs PR for the architecture/Superpowers documents; default is to leave them uncommitted and rely on the recovery snapshot.
4. Keep `.mcp.json` local-only unless the user confirms it is obsolete.
5. Keep `var/recovery/dirty-master-2026-05-26/` until all document disposition decisions are complete.

## Verdict

`APPROVE_WITH_NOTES` — root tracked state is clean and safe, but 17 candidate documents need owner-level retention decisions. Only `.mcp.json` and `var/recovery/` have clear local-only dispositions today.
