# Code Audit Final Report 2026-05-15

> 状态源说明：本文是代码审计报告，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 `FUNCTION_TREE.md` 的状态注册表为准。

## Executive Summary

The audit execution spec was implemented as an evidence package under `docs/CODE_AUDIT_EVIDENCE/`.

The repository is not release-ready from this audit run. The release-build gate has since been reproduced as a pass, but release confidence remains open until the local formatting and factor CSV output fixes for `AUDIT-S2-010` and `AUDIT-S2-011` are committed. No S0 or S1 issue was confirmed in the sampled manual review.

## Evidence Package

| Artifact | Status |
|---|---|
| `docs/CODE_AUDIT_EVIDENCE/baseline.md` | refreshed |
| `docs/CODE_AUDIT_EVIDENCE/cargo-gates.md` | refreshed |
| `docs/CODE_AUDIT_EVIDENCE/gitnexus-queries.md` | created |
| `docs/CODE_AUDIT_EVIDENCE/pattern-scan-summary.csv` | refreshed |
| `docs/CODE_AUDIT_EVIDENCE/pattern-hotspots.md` | refreshed |
| `docs/CODE_AUDIT_EVIDENCE/manual-review-log.md` | refreshed |
| `docs/CODE_AUDIT_EVIDENCE/sampled-files.md` | created |
| `docs/CODE_AUDIT_EVIDENCE/findings.csv` | appended with current findings |
| `docs/CODE_AUDIT_EVIDENCE/graphiti-memory.md` | post-review supplement |
| `docs/CODE_AUDIT_EVIDENCE/evidence-manifest.md` | post-review supplement |
| `docs/CODE_AUDIT_EVIDENCE/logs/README.md` | post-review supplement |
| `docs/CODE_AUDIT_EVIDENCE/archive/README.md` | post-review supplement |
| `docs/CODE_AUDIT_EVIDENCE/logs/cargo-build-release-20260517T174008Z.log` | release-build gate closure evidence |

## Post-Review Supplement

After reviewing `docs/reports/IMPECCABLE_AUDIT_CODE_AUDIT_EXECUTION_SPEC_2026-05-15.md`, the audit execution spec was hardened for future reproducibility. This report now references two supplemental evidence files:

- `docs/CODE_AUDIT_EVIDENCE/graphiti-memory.md` records Graphiti read/write evidence and clarifies that Graphiti is not a code truth source.
- `docs/CODE_AUDIT_EVIDENCE/evidence-manifest.md` records evidence package actions and checksums.

These supplements do not change the original source-code findings. The release-build follow-up evidence changes `AUDIT-S3-010` from `needs-repro` to fixed.

## Baseline

| Field | Result |
|---|---|
| Branch | `master` |
| HEAD | `b30de3123cbba3ffd8a040a0caf60258140f643f` |
| Worktree | dirty, 164 status entries |
| GitNexus | ready, `gitnexus analyze` returned `Already up to date` |
| Toolchain | cargo/rustc/rustfmt 1.90.0 toolchain family |

The dirty worktree materially limits confidence. Findings describe the local workspace as audited, not a clean release candidate.

## Gate Results

| Gate | Result | Finding |
|---|---|---|
| `cargo fmt --check` | fail | `AUDIT-S2-010` |
| `cargo clippy --all-targets --all-features` | pass with 220 warning diagnostics | none |
| `cargo test --all-targets` | fail | `AUDIT-S2-011` |
| `cargo build --release` | pass in follow-up gate run | `AUDIT-S3-010` fixed |

## Findings

| ID | Severity | Status | Summary |
|---|---|---|---|
| `AUDIT-S2-010` | S2 | deferred | Formatting gate fails in `src/factor/scoring.rs:1`. |
| `AUDIT-S2-011` | S2 | deferred | Factor score CSV output test fails at `tests/factor_pipeline_test.rs:454`. |
| `AUDIT-S3-010` | S3 | fixed | Release build gate reproduced as pass; evidence is `docs/CODE_AUDIT_EVIDENCE/logs/cargo-build-release-20260517T174008Z.log`. |
| `AUDIT-S3-009` | S3 | open | `menu --tui` still advertises TUI behavior but returns success from an in-progress placeholder. |

No S0 or S1 finding was confirmed.

## Pattern Scan

The pattern scan covered 411 files. `unsafe {` had 128 raw matches, all manually classified as test or `#[cfg(test)]` context. The scan also found 2486 `.unwrap()` matches, 184 `.expect(` matches, 222 `panic!(` matches, 1190 `println!` matches, 3 `TODO[^-]` matches, and 16 `let _ =` matches.

## Manual Review

- The existing TUI command contract finding remains open.
- Factor scoring has a confirmed user-facing output test failure.
- QMT task submit payload construction maps side, quantity, price, and order type into the bridge payload in the sampled path.
- No production runtime `unsafe` block was confirmed by line-level review.
- `FUNCTION_TREE.md` remains the feature status registry; this audit report does not create a competing source of feature truth.

## Completion Criteria

| Criterion | Result |
|---|---|
| Baseline reflects current execution date and commit | met |
| Gate outcomes recorded | met |
| Pattern scan artifacts current | met |
| Manual review covers P0/P1 and sampled P2/P3 | met |
| `findings.csv` uses required schema | met |
| Final report exists | met |
| S0/S1/S2 findings have explicit status and evidence | met; S2 findings are deferred with evidence |
| Remaining S3/S4 findings listed | met; `AUDIT-S3-010` is fixed and `AUDIT-S3-009` remains open |
| Graphiti review memory completed ingestion | met; episode `c987c1b4-8b27-4fe0-b92e-10b466ab4939` completed ingestion |
| Graphiti read evidence supplement | met; `graphiti-memory.md` added after review |
| Evidence manifest supplement | met; `evidence-manifest.md` added after review |
| Documentation hygiene checks | met; artifact self-check, `git diff --check`, and `repo_hygiene_test` passed |

## Recommended Remediation Order

1. Close `AUDIT-S2-010` by applying rustfmt only to the affected formatting drift.
2. Close `AUDIT-S2-011` by reproducing the factor score CSV output and fixing either the output ordering/value contract or the test expectation with evidence.
3. Re-run `cargo test --all-targets`.
4. Decide whether `menu --tui` should launch a real TUI or return an unsupported/non-zero result until implemented.

## Final Status

Audit execution completed with open/deferred findings. Release-build reproducibility is closed, but runtime release confidence remains open until `AUDIT-S2-010` and `AUDIT-S2-011` are committed and verified. Graphiti review memory completed ingestion. Post-review evidence supplements were added without changing the original source-code findings.
