# Generated And Runtime Artifact Classification

Date: 2026-05-26
Change: `dirty-worktree-cleanup-2026-05-26`
Scope: Phase 4 classification only. No delete, move, clean, reset, rebase, or root `master` realignment was executed.

## Preservation Check

Phase 0 recovery artifacts already preserve the generated/runtime paths that existed before cleanup began:

- Recovery manifest: `var/recovery/dirty-master-2026-05-26/phase0-manifest.json`
- Untracked archive SHA-256: `084a52d2d01a6c7cfaa0dcfcfa64983c6de0cc257b65028258915018b86a7b41`
- Inventory errors: `0`
- Archive errors: `0`
- Missing required artifacts: `0`

Preserved archive entry counts:

| Path group | Archive entries |
| --- | ---: |
| `logs/` | 2 |
| `var/` before recovery artifacts | 1 |
| `test_timing.csv` | 1 |
| `docs/reports/evidence/` | 2 |
| `.governance/active-gates.json` | 1 |
| `.governance/active-gates.md` | 1 |
| `.governance/backups/` | 9 |

## Classification

| Path group | Current root state | Classification | Policy decision | Disposal status |
| --- | --- | --- | --- | --- |
| `logs/` | Untracked; 2 files; 3230 bytes | Local runtime logs | Do not commit by default. Keep only if converted into curated report text. | No disposal executed; eligible for path-level approval after review. |
| `var/` excluding `var/recovery/dirty-master-2026-05-26/` | Untracked; 1 file; 3511 bytes | Local generated report output | Do not commit by default. Treat `var/reports/test_timing.csv` as generated timing evidence unless promoted by a separate evidence-retention decision. | No disposal executed; eligible for path-level approval after review. |
| `var/recovery/dirty-master-2026-05-26/` | Untracked; 14 files; 2310786 bytes | Approved Phase 0 recovery artifact | Preserve until the cleanup is fully accepted and root recovery is no longer needed. Do not commit into the product branch. | Must not be removed during this OpenSpec phase. |
| `test_timing.csv` | Untracked; 1 file; 1888 bytes | Local test timing output | Do not commit by default. Keep only as local performance evidence unless promoted by a separate report. | No disposal executed; eligible for path-level approval after review. |
| `docs/reports/evidence/` | Untracked; 2 JSON files; 5301 bytes | Raw retained evidence | Do not mix raw evidence into docs-only slices. Decide separately whether to commit raw evidence, replace with a curated manifest, or keep it outside git. | No disposal executed; deferred to evidence-retention approval. |
| `.governance/active-gates.json` and `.governance/active-gates.md` | Untracked in root; copied to clean worktree in Phase 3.5 | Shared active governance state | Include with repo policy slice because these files describe the current function-tree governance gate state. | No disposal; already promoted to clean worktree. |
| `.governance/backups/` | Untracked; 9 files; 750273 bytes | Timestamped generated backups | Do not commit by default. Phase 3.5 intentionally excluded these from the repo policy slice. | No disposal executed; eligible for path-level approval after review. |

## Deferred Actions

Approved disposition on 2026-05-26:

- No path-level archive or remove action is selected for this cleanup pass.
- Do not delete or move `logs/`, non-recovery `var/`, `test_timing.csv`,
  `docs/reports/evidence/`, or `.governance/backups/*` from the dirty root
  worktree.
- Do not copy those generated/runtime/raw evidence artifacts into the clean
  review worktree.
- Use the Phase 0 recovery archive as the preservation record until cleanup
  acceptance and any later root realignment are separately approved.

The following actions remain explicitly unexecuted and require a later
path-level approval if root cleanup is requested:

1. Remove or archive `logs/` from the dirty root worktree.
2. Remove or archive generated `var/` outputs other than the Phase 0 recovery directory.
3. Remove or archive `test_timing.csv`.
4. Decide whether `docs/reports/evidence/` raw JSON belongs in git, in an external archive, or behind a curated evidence manifest.
5. Remove or archive `.governance/backups/*` timestamped backups.

No `.gitignore` or evidence-retention policy file was changed in this phase. The classification supports local cleanup decisions, but it does not yet prove a repository-wide ignore policy change is required.
