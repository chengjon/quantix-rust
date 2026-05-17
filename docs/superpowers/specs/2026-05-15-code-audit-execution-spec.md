# Code Audit Execution Spec

**Date:** 2026-05-15

> 状态源说明：本文是代码审计执行规格，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 [`FUNCTION_TREE.md`](../../../FUNCTION_TREE.md) 的状态注册表行为准。

## 1. Purpose

This spec turns `docs/standards/CODE_AUDIT_METHODOLOGY.md` into a concrete execution contract for the next project-level code audit.

The goal is to produce an auditable package of evidence, findings, and closure decisions without turning this spec, the audit report, or any evidence file into a competing source for feature status. `FUNCTION_TREE.md` remains the only feature panorama and status registry.

### Methodology Correspondence

This spec uses the methodology's finding vocabulary and scan defaults as the baseline:

- Pattern-scan severities default to `docs/standards/CODE_AUDIT_METHODOLOGY.md`.
- `findings.csv` confidence values are `confirmed`, `probable`, or `needs-repro`.
- Finding statuses are `open`, `accepted`, `fixed`, `deferred`, `wontfix`, or `needs-repro`.
- The scan label `TODO` refers to the methodology regex `TODO[^-]`.
- Phase ordering may be regrouped for execution efficiency, but any regrouping must be called out in the final report with a mapping back to the methodology phase labels.
- If Graphiti is unavailable, the audit may finish with a documented `Graphiti backfill required` gap; the final report must not claim that ingest completed.

## 2. Inputs

The audit must use these sources as fixed inputs:

| Input | Role |
|---|---|
| `docs/standards/CODE_AUDIT_METHODOLOGY.md` | Primary methodology and phase structure |
| `docs/standards/CODE_AUDIT_METHODOLOGY_REVIEW_CODEX_2026-05-11.md` | Review constraints for scope counts, phase order, tool wording, and finding schema |
| `docs/CODE_AUDIT_EVIDENCE/` | Historical evidence package from the previous partial execution |
| `FUNCTION_TREE.md` | Sole feature status registry for feature availability and boundaries |
| `docs/standards/MOCK_USAGE_POLICY.md` | Mock/real data boundary policy |
| GitNexus | Code graph, execution flow, impact, and change-scope evidence |
| Graphiti `quantix_rust_review` | Historical review conclusions and final audit memory |

The previous evidence package must be treated as historical input, not automatically current truth. Any reused conclusion must be explicitly re-verified or marked as historical.

## 3. Non-Goals

This audit execution spec does not:

- Authorize broad refactors or opportunistic cleanup.
- Replace `FUNCTION_TREE.md` as the feature status registry.
- Treat old evidence as current without re-verification.
- Require every S3/S4 issue to be fixed before the final report.
- Permit a single passing gate to stand in for a completed audit.
- Hide remaining issues behind vague completion language.

## 4. Audit Package

The audit produces one evidence package and one final report:

```text
docs/CODE_AUDIT_EVIDENCE/
├── baseline.md
├── cargo-gates.md
├── gitnexus-queries.md
├── graphiti-memory.md
├── pattern-scan-summary.csv
├── pattern-hotspots.md
├── manual-review-log.md
├── sampled-files.md
├── evidence-manifest.md
├── logs/
├── archive/
└── findings.csv

docs/reports/CODE_AUDIT_FINAL_2026-05-15.md
```

Every generated Markdown report in this package must include a status-source note stating that the report is not a feature status registry and defers feature status, designed/pending items, evidence, and boundaries to `FUNCTION_TREE.md`.

## 5. Phase Contract

### Phase 0: Baseline Freeze

Create or refresh `docs/CODE_AUDIT_EVIDENCE/baseline.md`.

The baseline must record:

- Current timestamp and local timezone.
- `git rev-parse HEAD`.
- `git status --short`, including dirty and untracked counts.
- Toolchain versions for `cargo`, `rustc`, and `rustfmt`.
- GitNexus repository context and freshness result.
- Current counts for:
  - `src/` top-level modules
  - `src/cli/commands/*.rs`
  - `src/cli/handlers/*.rs`
  - `tests/*.rs`
  - `config/` files
  - key docs used by the audit
- Phase 2 scan-scope counts:
  - included Rust files under `src/**/*.rs`, `tests/**/*.rs`, `benches/**/*.rs`, and `examples/**/*.rs`
  - included audit-relevant scripts selected from `scripts/`
  - excluded Rust files under `.worktrees/**`
  - excluded generated, vendor, cache, and build-output paths

Acceptance:

- The baseline names the exact commit and dirty-worktree state.
- No hard-coded file counts from the methodology are reused without re-measuring.
- If GitNexus is stale, the baseline records whether `gitnexus analyze` was run or why it was not run.

### Phase 1: Gate Baseline

Create or refresh `docs/CODE_AUDIT_EVIDENCE/cargo-gates.md`.

Run and record:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test --all-targets
cargo build --release
```

If a gate cannot be run, the evidence must record the exact blocker, environment condition, and whether the audit can proceed with a degraded confidence level.

Long-running gate contract for `cargo build --release`:

- Run with an explicit maximum runtime budget. Default budget is 60 minutes unless the user sets a smaller task budget.
- Capture the full log to `docs/CODE_AUDIT_EVIDENCE/logs/cargo-build-release-<timestamp>.log` or record why log capture was impossible.
- If the command is detached or monitored outside the foreground tool call, record the process owner, PID/session id, start time, and cleanup command in `cargo-gates.md`.
- On timeout, confirm whether any owned `cargo`, `rustc`, or linker process remains and record the cleanup result.
- Classify exit `0` as `pass`, nonzero exit with complete compiler/linker output as `fail`, and timeout/tool-window loss without a complete cargo exit as `needs-repro`.
- Promote `needs-repro` to `pass` or `fail` only after a later complete run records a cargo exit status and the relevant log tail.

Clippy warning policy:

- `cargo clippy --all-targets --all-features` exit status controls the clippy gate result.
- Warnings are still audit evidence. `cargo-gates.md` must group warning diagnostics by lint name.
- Any warning class with 10 or more diagnostics, or any safety/concurrency warning class such as `clippy::await_holding_lock`, must either become a `findings.csv` row or be documented as a non-issue with rationale in `cargo-gates.md`.

Acceptance:

- Every gate has an exit status and command output summary.
- Any failure is classified as pre-existing, audit-discovered, or caused by audit changes.
- The final audit report does not claim gate closure unless the latest recorded run supports it.

### Phase 2: Full Automatic Scan

Create or refresh:

- `docs/CODE_AUDIT_EVIDENCE/pattern-scan-summary.csv`
- `docs/CODE_AUDIT_EVIDENCE/pattern-hotspots.md`

Scan only the main workspace according to this scope contract.

Include:

- `src/**/*.rs`
- `tests/**/*.rs`
- `benches/**/*.rs`
- `examples/**/*.rs` when present
- selected audit-relevant scripts under `scripts/` when they participate in build, test, smoke, delivery, data, or bridge workflows

Exclude:

- `.worktrees/**`
- `target/**`
- `.git/**`
- generated, vendor, cache, and tool-output paths such as `vendor/**`, `node_modules/**`, `.zread/**`, `.gitnexus/**`, and `docs/CODE_AUDIT_EVIDENCE/archive/**`

Within the included scope, scan for:

| Pattern | Default severity | Required classification |
|---|---:|---|
| `.unwrap()` | HIGH | production/test/example; boundary crossing; recoverability |
| `.expect(` | HIGH | invariant text quality; operator-facing failure risk |
| `panic!(` | HIGH | unreachable invariant vs runtime/config failure |
| `unsafe {` | CRITICAL | safety comment, external invariant, test coverage |
| `let _ =` | MEDIUM | intentionally ignored value vs dropped error/result |
| `TODO[^-]` | MEDIUM | harmless note vs hidden acceptance gap |
| `println!` | MEDIUM | CLI output path vs library/runtime side effect |

Acceptance:

- `pattern-scan-summary.csv` includes total matches and production/test/non-code buckets.
- `pattern-hotspots.md` identifies representative hotspots by module and exact path.
- Both scan artifacts record the include/exclude scope and file counts used for the scan.
- Scan results are not findings until manually reviewed and classified.
- These severities mirror the methodology defaults; any lower-risk reclassification belongs in manual review evidence, not in the scan contract.

### Phase 2b: Architecture and Status Consistency Scan

Create or refresh `docs/CODE_AUDIT_EVIDENCE/gitnexus-queries.md`.
Create or refresh `docs/CODE_AUDIT_EVIDENCE/graphiti-memory.md`.

Use GitNexus to inspect:

- Repository context and indexed date.
- Clusters and major execution flows.
- P0 paths: execution, strategy, risk, bridge, CLI dispatch.
- Any stale index warning.

Use Graphiti before review conclusions and record:

- query text
- `group_id`
- result count and compact result summary
- fact UUIDs used in any conclusion
- no-result outcomes
- read failure fallback and whether `Graphiti backfill required` applies

Also verify `FUNCTION_TREE.md` consistency:

- Feature-status rows still include status, evidence, and boundary.
- Designed/pending rows are not described as available elsewhere.
- Auxiliary status-bearing docs defer to `FUNCTION_TREE.md`.

Acceptance:

- GitNexus query outputs are summarized with query name, purpose, result, and follow-up.
- Graphiti read outputs are summarized in `graphiti-memory.md` with query, group, result summary, fact UUIDs used, and fallback status.
- Any mismatch between code and `FUNCTION_TREE.md` becomes either a finding or a documented non-issue with evidence.

### Phase 3: P0 Manual Deep Review

Create or refresh `docs/CODE_AUDIT_EVIDENCE/manual-review-log.md`.

Review these modules first:

| Area | Required focus |
|---|---|
| `src/execution/` | adapter identity, request lifecycle, live gate, mock/live isolation, cancellation, reconciliation |
| `src/strategy/` | daemon state, strategy registry, evaluator behavior, execution integration |
| `src/risk/` | risk rules, persistence, industry sync, execution blocking, diagnostics |
| `src/bridge/` and QMT integration | broker payload preservation, external process boundaries, failure reporting |
| CLI commands and handlers | command/handler wiring, advertised capability vs actual behavior |

Acceptance:

- Each reviewed area has an entry with exact files, key symbols, GitNexus evidence, and conclusion.
- Any confirmed problem is added to `findings.csv`.
- The previous open finding `AUDIT-S3-009` is explicitly re-evaluated.

### Phase 4: P1 Manual Review

Review these exact path groups:

| Path group | Coverage requirement |
|---|---|
| `src/monitor/` | Mark `not present`, `sampled`, `deep-reviewed`, or `out of scope` with rationale |
| `src/monitoring/` | Mark `not present`, `sampled`, `deep-reviewed`, or `out of scope` with rationale |
| `src/stop/` | Mark `not present`, `sampled`, `deep-reviewed`, or `out of scope` with rationale |
| `src/account/` | Mark `not present`, `sampled`, `deep-reviewed`, or `out of scope` with rationale |
| `src/trade/` | Mark `not present`, `sampled`, `deep-reviewed`, or `out of scope` with rationale |
| `src/db/` | Mark `not present`, `sampled`, `deep-reviewed`, or `out of scope` with rationale |
| `src/*/storage.rs` | Mark `not present`, `sampled`, `deep-reviewed`, or `out of scope` with rationale |
| `scripts/init-*.sql` | Mark `not present`, `sampled`, `deep-reviewed`, or `out of scope` with rationale |
| `src/monitoring/notification.rs` | Mark `not present`, `sampled`, `deep-reviewed`, or `out of scope` with rationale |
| `src/monitoring/notification/` | Mark `not present`, `sampled`, `deep-reviewed`, or `out of scope` with rationale |
| `src/cli/handlers/notify.rs` | Mark `not present`, `sampled`, `deep-reviewed`, or `out of scope` with rationale |

Acceptance:

- Each area is either deep-reviewed or explicitly marked as sampled with a reason.
- Each exact path group is listed in `manual-review-log.md` with one of the required coverage statuses.
- No S0/S1/S2 finding is left outside `findings.csv`.

### Phase 5: P2/P3 Sampled Review

Review lower-risk or broader modules using sampling:

- `src/analysis/`
- `src/factor/`
- `src/market/`
- `src/news/`
- `src/fundamental/`
- `src/import/`
- `src/sources/`
- selected tests and docs

Acceptance:

- Sampling criteria are written in `sampled-files.md`.
- Sampled modules list exact files and why they were selected.
- Unreviewed areas are clearly named as residual risk, not silently omitted.

### Phase 6: Finding Closure and Final Report

Create or refresh `docs/CODE_AUDIT_EVIDENCE/findings.csv` with this exact schema:

```csv
id,severity,confidence,module,file:line,evidence,rule,impact,reproduction,recommended_fix,acceptance_criteria,tests_required,status
```

Confidence values follow the methodology and are `confirmed`, `probable`, or `needs-repro`.

Allowed finding statuses:

- `open`
- `accepted`
- `fixed`
- `deferred`
- `wontfix`
- `needs-repro`

Create `docs/reports/CODE_AUDIT_FINAL_2026-05-15.md`.

The final report must include:

- Scope and baseline summary.
- Gate summary.
- Pattern scan summary.
- GitNexus process/query summary.
- Graphiti read summary from `graphiti-memory.md`, including groups searched and fact UUIDs used.
- Manual review summary by module.
- Finding counts by severity and status.
- Remaining open/deferred/wontfix/needs-repro findings.
- Fixed finding verification commands.
- Residual risk.
- Positive observations.
- Explicit statement that `FUNCTION_TREE.md` remains the feature status registry.
- Graphiti review-memory episode id and ingest status.

Acceptance:

- Every S0/S1/S2 finding is `fixed`, `deferred`, `wontfix`, or `needs-repro` with an explicit rationale.
- Any S3/S4 issue left open is listed in the final report.
- The final report references the evidence files rather than embedding long raw logs.
- Graphiti `quantix_rust_review` receives a compact conclusion memory and ingest is verified as `completed`, or the final report records the explicit `Graphiti backfill required` gap if the service is unavailable.

## 6. Severity Rules

| Severity | Meaning | Required response |
|---|---|---|
| S0 | Funds safety, real-trading gate bypass, data integrity breakage | Stop and fix or explicitly block release |
| S1 | Core execution, state machine, broker bridge, or mock/live correctness issue | Fix or formally defer with rationale |
| S2 | Recoverability, persistence, observability, test reliability, or user-facing diagnostic issue | Fix if feasible; otherwise track with acceptance criteria |
| S3 | Maintainability, UX wording, low-risk half-wired surface | May remain open if listed |
| S4 | Observation or future improvement | May be documented only |

## 7. Finding Lifecycle

Each finding must move through one of these paths:

```text
open -> accepted -> fixed
open -> accepted -> deferred
open -> wontfix
open -> needs-repro
```

Rules:

- `fixed` requires a verification command and a passing result.
- `deferred` requires a reason, residual risk, and suggested follow-up owner or module.
- `wontfix` requires evidence that the finding is not being fixed for a documented technical reason.
- `needs-repro` requires a concrete reproduction gap, not vague uncertainty.

## 8. Handling Existing Evidence

The previous `docs/CODE_AUDIT_EVIDENCE/` package must be handled as follows:

- Keep it as historical context.
- Re-run or explicitly re-validate baseline, gates, and open findings.
- Carry forward `AUDIT-S3-009` unless current evidence proves it is fixed, `wontfix`, or intentionally deferred.
- Do not overwrite old evidence without preserving enough timestamp or summary context to reconstruct what changed.

Refresh mechanics:

- Before replacing an existing evidence file, copy the previous version to `docs/CODE_AUDIT_EVIDENCE/archive/<timestamp>/`.
- Update `docs/CODE_AUDIT_EVIDENCE/evidence-manifest.md` for every created, refreshed, or archived artifact.
- Each manifest row must include artifact path, action (`created`, `refreshed`, `archived`, or `unchanged`), old checksum when applicable, new checksum when applicable, command/source, reason, and archive path when applicable.
- The final report must reference the manifest whenever evidence was refreshed rather than newly created.

## 9. Execution Constraints

The audit must follow repository operating rules:

- Use context-mode for broad scans and summaries.
- Use GitNexus for code structure, process flow, and impact evidence.
- Use Graphiti reads before review conclusions and writes after conclusions converge.
- Do not rely on Graphiti as the source for current code truth.
- Do not modify production code during the audit unless the task explicitly shifts from audit to remediation.
- Do not revert unrelated dirty worktree changes.

## 10. Completion Criteria

The audit is complete only when all of these are true:

- `baseline.md` reflects the current execution date and commit.
- `cargo-gates.md` records current gate outcomes.
- `pattern-scan-summary.csv` and `pattern-hotspots.md` exist and are current.
- `manual-review-log.md` covers P0, P1, and sampled P2/P3 areas.
- `graphiti-memory.md` records required read-side Graphiti evidence or an explicit read failure fallback.
- `evidence-manifest.md` records created/refreshed/archived evidence artifacts.
- `findings.csv` uses the required schema.
- The final report exists at `docs/reports/CODE_AUDIT_FINAL_2026-05-15.md`.
- S0/S1/S2 findings have explicit closure status and evidence.
- Remaining S3/S4 findings are listed rather than hidden.
- Graphiti review memory has completed ingestion.
- The final report passes repository documentation hygiene checks:
  - `cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test repo_hygiene_test`
  - `git diff --check -- docs/CODE_AUDIT_EVIDENCE docs/reports/CODE_AUDIT_FINAL_2026-05-15.md docs/superpowers/specs/2026-05-15-code-audit-execution-spec.md`
  - generated Markdown artifacts contain the required `FUNCTION_TREE.md` status-source note and do not present themselves as feature status registries.

If Graphiti is unavailable, the audit may still produce a provisional final report, but that report must include `Graphiti backfill required` and must not claim the ingest gate passed.

## 11. Out-of-Scope Follow-Ups

If the audit finds broad remediation work, the final report should propose separate follow-up implementation plans. Those plans must not be folded into the audit spec unless the user explicitly asks to switch from audit execution to remediation.
