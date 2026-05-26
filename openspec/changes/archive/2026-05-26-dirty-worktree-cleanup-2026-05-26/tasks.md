# Dirty Worktree Cleanup Tasks

## 0. OpenSpec Setup

- [x] 0.1 Create active OpenSpec change `dirty-worktree-cleanup-2026-05-26`.
- [x] 0.2 Link the controlling document set:
  - `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26.md`
  - `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-review.md`
  - `docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_2026-05-26-meta-review.md`
- [x] 0.3 Validate the OpenSpec change in strict non-interactive mode.
  - Status 2026-05-26: `openspec validate dirty-worktree-cleanup-2026-05-26 --strict --no-interactive --json` passed with 1 item passed and 0 failed.

## 1. Phase 0 - Freeze And Snapshot

- [x] 1.1 Confirm phase approval is limited to inventory refresh and recovery artifact creation under `var/recovery/dirty-master-2026-05-26/`.
  - Status 2026-05-26: user approved continuing after being told the scope was Phase 0 only and explicitly excluded reset, rebase, delete, move, and root `master` realignment.
- [x] 1.2 Re-run prerequisite checks:
  - no active Git index lock,
  - enough disk for recovery artifacts,
  - `docs/reports/evidence/` exists,
  - `git worktree` is available,
  - no known competing operator, agent, editor automation, CI job, or watcher is mutating the worktree.
  - Status 2026-05-26 read-only preflight: Git index lock absent, `docs/reports/evidence/` exists, recovery target did not yet exist, 9 worktrees were registered, and approximately 622.3 GiB was free.
- [x] 1.3 Refresh dirty inventory and record any count drift before artifact creation.
  - Status 2026-05-26 snapshot inventory: compact status entries were 206 and actual untracked files were 99. Drift from the reviewed plan is expected because the meta-review document and this OpenSpec change are now untracked inputs.
- [x] 1.4 Create the local safety branch from the current dirty `master` state.
  - Status 2026-05-26: created `rescue/dirty-master-2026-05-26` at `14ab859af2ac1312aec67d8aa78dfca2e5a83f4b`.
- [x] 1.5 Save tracked diff to `var/recovery/dirty-master-2026-05-26/tracked.diff`.
  - Status 2026-05-26: wrote `tracked.diff` with 711339 bytes and SHA-256 `2e0463b1aa4b2f84f0a2368df67d591f469b1504d9a02458b980248bf31eb32f`.
- [x] 1.6 Save untracked file inventory, sizes, and checksums.
  - Status 2026-05-26: wrote `untracked-files.txt`, `untracked-sizes.txt`, and `untracked-sha256.txt` for 99 untracked files.
- [x] 1.7 Archive untracked files or otherwise record why a full archive is intentionally deferred.
  - Status 2026-05-26: wrote `untracked-files.tar` with 1546240 bytes and SHA-256 `084a52d2d01a6c7cfaa0dcfcfa64983c6de0cc257b65028258915018b86a7b41`.
- [x] 1.8 Save local head, stash list, worktree list, and restore instructions.
  - Status 2026-05-26: wrote `local-head-show.txt`, `stash-list.txt`, `worktree-list.txt`, `branch-list.txt`, `rescue-branch.txt`, and `restore-instructions.md`.
- [x] 1.9 Verify all Phase 0 recovery artifacts exist and are readable.
  - Status 2026-05-26: `phase0-manifest.json` reports no missing required artifacts, no inventory errors, and no archive errors.

## 2. Phase 1 - Clean Review Base

- [x] 2.1 Confirm separate approval for named worktree path and branch name.
  - Status 2026-05-26: user approved continuing after Phase 0 closure. Default Phase 1 target was used: worktree `.worktrees/dirty-cleanup-review-base`, branch `cleanup/dirty-worktree-review-base-2026-05-26`.
- [x] 2.2 Create a clean worktree from `origin/master`.
  - Status 2026-05-26: created `.worktrees/dirty-cleanup-review-base` on `cleanup/dirty-worktree-review-base-2026-05-26` tracking `origin/master` at `b59955a9e795`.
- [x] 2.3 Keep root `master` untouched as the salvage source.
  - Status 2026-05-26: root worktree remains on `master` at `14ab859af2ac`; no root reset, rebase, clean, or realignment was performed.
- [x] 2.4 Stop and inspect rather than overwrite if the target worktree or branch already exists.
  - Status 2026-05-26: preflight confirmed the target path and branch did not exist before creation. Post-check confirms the clean worktree has 0 status entries.

## 3. Documentation And Governance Slices

- [x] 3.1 Extract architecture audit documents and OpenSpec-relevant audit evidence as one docs-only slice.
  - Status 2026-05-26: copied the architecture audit report package into `.worktrees/dirty-cleanup-review-base` on branch `cleanup/dirty-worktree-review-base-2026-05-26`:
    - `docs/reports/ARCHITECTURE_AUDIT_2026-05-23.md`
    - `docs/reports/ARCHITECTURE_AUDIT_REVIEW_2026-05-23.md`
    - `docs/superpowers/specs/2026-05-23-architecture-audit-design.md`
  - Verification: all three files match the root snapshot source by SHA-256, the clean worktree has only these three untracked docs for this slice, and root `master` remains untouched. Existing `openspec/specs/architecture-remediation/spec.md` already matches root and clean worktree; `openspec/config.yaml` remains deferred to task 3.5 repository-policy review.
- [x] 3.2 Extract code-audit methodology documents as a separate docs-only slice.
  - Status 2026-05-26: copied the code-audit methodology package into `.worktrees/dirty-cleanup-review-base` on branch `cleanup/dirty-worktree-review-base-2026-05-26`:
    - `docs/standards/CODE_AUDIT_METHODOLOGY.md`
    - `docs/standards/CODE_AUDIT_METHODOLOGY-review.md`
    - `docs/standards/CODE_AUDIT_METHODOLOGY_REVIEW_CODEX_2026-05-11.md`
    - `docs/reports/IMPECCABLE_AUDIT_CODE_AUDIT_EXECUTION_SPEC_2026-05-15.md`
  - Verification: all four files match the root snapshot source by SHA-256. `docs/superpowers/specs/2026-05-15-code-audit-execution-spec.md`, `docs/reports/CODE_AUDIT_FINAL_2026-05-15.md`, and `docs/reports/CODE_AUDIT_MATT_SKILLS_ISSUE_DRAFTS_2026-05-15.md` already exist in the clean worktree from `origin/master`, so they were not duplicated. Root `master` remains untouched.
- [x] 3.3 Extract miniQMT reports and runbook documents as a separate docs-only slice.
  - Status 2026-05-26: copied the miniQMT evidence and operations package into `.worktrees/dirty-cleanup-review-base` on branch `cleanup/dirty-worktree-review-base-2026-05-26`:
    - `docs/operations/MINIQMT_QUANTIX_REGRESSION_OPERATOR_RUNBOOK_2026-05-20.md`
    - `docs/reports/MINIQMT_CONTROLLED_EVIDENCE_ALIGNMENT_RESPONSE_2026-05-18.md`
    - `docs/reports/MINIQMT_DIRECT_CLICKHOUSE_READ_ONLY_COMPARISON_CLOSEOUT_2026-05-21.md`
    - `docs/reports/MINIQMT_LOCAL_REFERENCE_COMPARISON_CLOSEOUT_2026-05-21.md`
    - `docs/reports/MINIQMT_PAYLOAD_ROW_COUNT_VERIFICATION_CLOSEOUT_2026-05-20.md`
    - `docs/reports/MINIQMT_PAYLOAD_SAMPLING_CLOSEOUT_2026-05-20.md`
    - `docs/reports/MINIQMT_QUANTIX_REGRESSION_EVIDENCE_RECEIVE_RESULT_2026-05-18.md`
    - `docs/reports/MINIQMT_SOURCE_OF_TRUTH_SUMMARY_COMPARISON_CLOSEOUT_2026-05-21.md`
  - Verification: all eight Markdown files match the root snapshot source by SHA-256. Raw retained evidence files under `docs/reports/evidence/miniqmt/*.json` remain excluded for later generated/evidence retention policy review. Root `master` remains untouched.
- [x] 3.4 Reconcile roadmap and registry documents against `FUNCTION_TREE.md`.
  - Status 2026-05-26: copied the updated root `FUNCTION_TREE.md` into `.worktrees/dirty-cleanup-review-base` on branch `cleanup/dirty-worktree-review-base-2026-05-26`.
  - Verification: copied `FUNCTION_TREE.md` matches the root snapshot source by SHA-256 (`2978936d4e0774300067026d3bc8555f69ce237525b7116196124378df49d96e`). The legacy roadmap files `ROADMAP.md`, `docs/DEVELOPMENT_ROADMAP.md`, and `docs/ROADMAP_REVIEW.md` are absent from both the root salvage state and the clean `origin/master` worktree, and no tracked clean-worktree docs reference those legacy paths. Root `master` remains untouched.
- [x] 3.5 Decide repo policy for `.claude/`, `.codex/`, `.governance/`, `.mcp.json`, and `openspec/config.yaml`.
  - Status 2026-05-26: copied the repo-shared policy surface into `.worktrees/dirty-cleanup-review-base` on branch `cleanup/dirty-worktree-review-base-2026-05-26`:
    - 12 `.claude/commands/ft/*.md` function-tree command files,
    - 12 `.codex/commands/ft/*.md` function-tree command files,
    - `.governance/README.md`,
    - `.governance/profile.md`,
    - `.governance/active-gates.json`,
    - `.governance/active-gates.md`,
    - `.governance/guards/ft-scope-check.sh`,
    - `.governance/programs/project-governance/nodes.json`,
    - `.governance/programs/project-governance/tree.md`,
    - 7 `.governance/programs/project-governance/cards/Q1.*.yaml` task cards,
    - `openspec/config.yaml`.
  - Verification: all 39 included files match the root snapshot source by SHA-256; `.governance/guards/ft-scope-check.sh` keeps executable mode `755`. `.mcp.json` is excluded as local MCP host configuration, `.claude/tdd-guard/data/test.json` is excluded as local guard sample data, and 9 `.governance/backups/FUNCTION_TREE.*.md` files are excluded as timestamped backups. Root `master` remains untouched.

## 4. Generated And Runtime Artifacts

- [x] 4.1 Preserve generated/runtime artifacts in the approved recovery snapshot before any disposal.
  - Status 2026-05-26: Phase 0 archive `var/recovery/dirty-master-2026-05-26/untracked-files.tar` preserves `logs/`, pre-recovery `var/` outputs, `test_timing.csv`, `docs/reports/evidence/`, `.governance/active-gates.*`, and `.governance/backups/*`. Manifest reports 0 inventory errors, 0 archive errors, and 0 missing required artifacts.
- [x] 4.2 Classify `logs/`, `var/`, `test_timing.csv`, `docs/reports/evidence/`, `.governance/active-gates.*`, and `.governance/backups/*` by retention policy.
  - Status 2026-05-26: wrote `openspec/changes/dirty-worktree-cleanup-2026-05-26/generated-runtime-artifact-classification.md`. Classification keeps `var/recovery/dirty-master-2026-05-26/` as the safety artifact, treats `.governance/active-gates.*` as repo policy already promoted in 3.5, and defers `logs/`, non-recovery `var/`, `test_timing.csv`, `docs/reports/evidence/`, and `.governance/backups/*` to path-level disposal or evidence-retention approval.
- [x] 4.3 Apply only path-level approved archive/remove actions.
  - Approved disposition 2026-05-26: no path-level archive/remove action is
    selected for this cleanup pass. Do not delete or move `logs/`,
    non-recovery `var/`, `test_timing.csv`, `docs/reports/evidence/`, or
    `.governance/backups/*` from the dirty root worktree; do not copy those
    generated/runtime/raw evidence artifacts into the clean review worktree.
    Use the Phase 0 recovery archive as the preservation record until cleanup
    acceptance and any later root realignment are separately approved.
- [x] 4.4 Update ignore or evidence-retention policy only if the classification proves it is repository policy, not local cleanup.
  - Status 2026-05-26: no `.gitignore` or evidence-retention policy update was made. The classification supports local cleanup decisions but does not yet prove a repository-wide ignore or evidence-retention policy change is required.

## 5. Product-Code Extraction Slices

- [x] 5.1 Extract Slice 6A Market Import / Strength Runtime after duplicate/superseded checks.
  - Result: `src/market/strength_runtime.rs`, `src/market/mod.rs`,
    `src/cli/tests/market.rs`, `src/cli/handlers/import.rs`, and
    `src/import/mod.rs` were already present in the clean review base with
    matching content, so only the three missing test files were copied into
    the clean worktree.
  - Added tests: `tests/market_strength_calculation_test.rs`,
    `tests/miniqmt_market_import_handler_test.rs`, and
    `tests/miniqmt_market_manifest_test.rs`.
  - GitNexus impact review: `build_market_analysis_foundation` reported
    CRITICAL upstream risk because it participates in market command flows;
    this slice did not modify that production symbol. Import manifest and
    manifest parsing targets reported LOW risk.
  - Validation passed: `cargo fmt --check`,
    `RUSTFLAGS=-Awarnings cargo test --test market_strength_calculation_test --quiet`,
    `RUSTFLAGS=-Awarnings cargo test --test miniqmt_market_import_handler_test --quiet`,
    and `RUSTFLAGS=-Awarnings cargo test --test miniqmt_market_manifest_test --quiet`.
  - Scope note: `gitnexus_detect_changes(scope=all)` returned LOW risk but
    only reported tracked `FUNCTION_TREE.md`; the newly copied test files are
    still untracked in the clean worktree, so SHA checks and focused tests are
    the direct evidence for this slice until staging/commit review.
- [x] 5.2 Extract Slice 6B CLI Import / Validation Surface after boundary ownership is confirmed.
  - Result: no additional files were copied for this slice. The import surface
    candidates `src/import/*`, `src/cli/handlers/data_handler.rs`, and
    `src/cli/tests/import.rs` already matched the clean review base.
  - Boundary decision: the remaining non-matching CLI files
    `src/cli/command_types.rs`, `src/cli/commands/mod.rs`, and
    `src/cli/commands/trade.rs` are not part of the import/validation slice.
    The root copies either omit clean-base safety command wiring or revert
    qmt_live trade wording, so they were excluded from Slice 6B.
  - Validation passed:
    `RUSTFLAGS=-Awarnings cargo test --lib cli::tests::import --quiet`
    and `RUSTFLAGS=-Awarnings cargo test --lib --quiet`.
- [x] 5.3 Extract Slice 6C Execution / Strategy Runtime only after GitNexus impact review and explicit high-risk handling.
  - Classification artifact:
    `openspec/changes/dirty-worktree-cleanup-2026-05-26/slice-6c-execution-strategy-classification.md`.
  - Approved handling 2026-05-26: no Slice 6C production code or coupled test
    drift is copied in this cleanup pass. GitNexus impact reported CRITICAL
    risk for `build_completion_diagnostics` and
    `execute_strategy_create_with_store`, and several root dirty copies would
    delete clean-base behavior. Treat Slice 6C as deferred/excluded from this
    cleanup and open a dedicated execution/strategy high-risk change later if
    those local changes still need product review.
- [x] 5.4 Extract Slice 6D Risk / Industry / Live Import after adapter/domain boundaries are confirmed.
  - Result: only test drift was copied. `src/risk/importer.rs`,
    `src/risk/industry_sync.rs`, `src/risk/rebuild.rs`, and
    `src/risk/service.rs` already matched the clean review base.
  - Added test changes: `tests/risk_volatility_test.rs` and
    `tests/stop_service_test.rs`.
  - Excluded: `src/cli/handlers/risk/output.rs` only added redundant imports;
    `src/execution/qmt_live_adapter.rs`, `tests/qmt_bridge_preview_test.rs`,
    and `tests/qmt_live_adapter_test.rs` overlap with blocked Slice 6C.
  - GitNexus impact review: `RiskService` LOW, `StopService` LOW.
  - Validation passed: `cargo fmt --check`,
    `RUSTFLAGS=-Awarnings cargo test --test risk_service_test --quiet`,
    `RUSTFLAGS=-Awarnings cargo test --test risk_volatility_test --quiet`,
    and `RUSTFLAGS=-Awarnings cargo test --test stop_service_test --quiet`.
  - GitNexus detect_changes: LOW risk; changed tracked files were
    `FUNCTION_TREE.md`, `tests/risk_volatility_test.rs`, and
    `tests/stop_service_test.rs`.
- [x] 5.5 Extract Slice 6E Broad Library Hygiene only after production-risk classification and focused validation.
  - Result: copied only low-risk hygiene changes in
    `src/ai/prompt.rs` and `benches/bench_main.rs`.
  - Excluded: `Cargo.toml` and `Cargo.lock` because the root dirty copies
    downgrade current clean-base dependency versions; `src/analysis/polars_adapter.rs`
    because GitNexus impact for `init_polars` reported CRITICAL risk across
    CLI initialization flows.
  - GitNexus impact review: `PromptRegistry` LOW,
    `bench_batch_processing` LOW, `init_polars` CRITICAL.
  - Validation passed: `cargo fmt --check`,
    `RUSTFLAGS=-Awarnings cargo test --lib ai::prompt --quiet`,
    `RUSTFLAGS=-Awarnings cargo test --lib --quiet`, and
    `RUSTFLAGS=-Awarnings cargo test --benches --no-run --quiet`.
  - GitNexus detect_changes: LOW risk; no affected processes reported.

## 6. Closure Gates

- [x] 6.1 Confirm every selected slice has a clean path-scoped diff.
  - Status 2026-05-26: clean review worktree contains 62 changed paths after
    removing validation-generated `logs/notifications.log`; path audit reported
    0 unexpected paths. The selected scope is docs/governance/OpenSpec policy,
    `FUNCTION_TREE.md`, five focused test files, `src/ai/prompt.rs`, and
    `benches/bench_main.rs`.
- [x] 6.2 Confirm recovery artifacts can reconstruct the pre-cleanup state in a disposable worktree.
  - Status 2026-05-26: Phase 0 archive SHA matches manifest
    `084a52d2d01a6c7cfaa0dcfcfa64983c6de0cc257b65028258915018b86a7b41`,
    `tar -tf` reads 99 archived untracked entries, and
    `git apply --check var/recovery/dirty-master-2026-05-26/tracked.diff`
    passed in a temporary worktree created from
    `rescue/dirty-master-2026-05-26`.
- [x] 6.3 Run task-relevant validation for every slice.
  - Status 2026-05-26: validation passed for `cargo fmt --check`,
    `RUSTFLAGS=-Awarnings cargo test --lib --quiet`,
    `RUSTFLAGS=-Awarnings cargo test --test market_strength_calculation_test --quiet`,
    `RUSTFLAGS=-Awarnings cargo test --test miniqmt_market_import_handler_test --quiet`,
    `RUSTFLAGS=-Awarnings cargo test --test miniqmt_market_manifest_test --quiet`,
    `RUSTFLAGS=-Awarnings cargo test --test risk_service_test --quiet`,
    `RUSTFLAGS=-Awarnings cargo test --test risk_volatility_test --quiet`,
    `RUSTFLAGS=-Awarnings cargo test --test stop_service_test --quiet`, and
    `RUSTFLAGS=-Awarnings cargo test --benches --no-run --quiet`.
- [x] 6.4 Run GitNexus detect_changes before committing code slices or final root realignment.
  - Status 2026-05-26: `gitnexus_detect_changes(scope=all)` on the clean
    review worktree reported LOW risk, 5 changed tracked files, and 0 affected
    processes. No commit or root realignment was performed.
- [x] 6.5 Realign root `master` only after separate explicit approval.
  - Approved disposition 2026-05-26: no root `master` realignment is selected
    for this cleanup pass. Root `master` remains the salvage source; any future
    root reset, rebase, cleanup, or realignment still requires separate
    explicit approval.
- [x] 6.6 Archive this OpenSpec change only after cleanup acceptance criteria are met.
  - Approved 2026-05-26: final acceptance granted by the user after closure
    gates 6.1-6.5 passed. Archive this change with spec updates.
