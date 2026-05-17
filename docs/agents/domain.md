# Domain Docs

This file tells Matt Pocock skills where to read project context before creating issues, triaging work, diagnosing bugs, using TDD, or proposing architecture improvements.

## Current Layout

This repository does not currently have a root `CONTEXT.md`, `CONTEXT-MAP.md`, or `docs/adr/` directory. Until those are added, agents must use the existing governance, audit, and standards documents listed here.

## Required Reading

Always read these before code-audit work:

- `FUNCTION_TREE.md`
- `docs/standards/CODE_AUDIT_METHODOLOGY.md`
- `docs/standards/MOCK_USAGE_POLICY.md`
- `docs/reports/CODE_AUDIT_FINAL_2026-05-15.md`
- `docs/reports/IMPECCABLE_AUDIT_CODE_AUDIT_EXECUTION_SPEC_2026-05-15.md`

For audit evidence, also read:

- `docs/CODE_AUDIT_EVIDENCE/baseline.md`
- `docs/CODE_AUDIT_EVIDENCE/cargo-gates.md`
- `docs/CODE_AUDIT_EVIDENCE/findings.csv`
- `docs/CODE_AUDIT_EVIDENCE/manual-review-log.md`

## Project-Specific Rules

- `FUNCTION_TREE.md` is the sole feature status registry.
- Audit reports and issues must not become competing feature status sources.
- Use GitNexus for code structure, process flow, impact analysis, and change-scope evidence.
- Use Graphiti reads before review conclusions and Graphiti writes after conclusions converge.
- Use context-mode for broad scans, large outputs, and evidence summarization.
- Do not use Graphiti as current code truth.
- Do not edit production code during audit-spec work unless the task explicitly shifts to remediation.
- Do not revert unrelated dirty worktree changes.

## Closure-Stage Priority

When a task is in closure stage, prioritize gate closure before cleanup. For the current audit finding set, this means resolving or explicitly reclassifying:

- `cargo fmt --check` failure, currently tracked as `AUDIT-S2-010`
- `cargo test --all-targets` failure, currently tracked as `AUDIT-S2-011`
- `cargo build --release` `NEEDS-REPRO`, currently tracked as `AUDIT-S3-010`

Do not start broad architecture cleanup until those gate outcomes are clear.

## Architecture Review Vocabulary

When using `improve-codebase-architecture`, use the Matt Pocock terms from that skill:

- Module
- Interface
- Implementation
- Depth
- Seam
- Adapter
- Leverage
- Locality

Apply those terms to this repo's high-risk domains:

- execution
- strategy
- risk
- bridge and QMT integration
- CLI command and handler wiring
- persistence and storage
- notification paths

If a future architecture discussion introduces stable domain vocabulary or accepted design decisions, add a root `CONTEXT.md` and `docs/adr/` before relying on those terms in new issue batches.
