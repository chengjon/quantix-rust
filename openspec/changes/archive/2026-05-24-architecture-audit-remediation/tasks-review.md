# Review: tasks.md

**Type**: md / plan | **Perspective**: completeness, feasibility | **Date**: 2026-05-23

## Summary

The task list is well-structured with correct phase ordering (characterization tests before seams, seams before splits) and all 19 referenced source files exist. One task (4.6) describes code conditions that don't match the live codebase, and several tasks lack individual acceptance criteria. Task 3.1 references three context types that don't exist as named symbols anywhere.

## Verified

- **C1 (Required sections)**: Plan has 7 phases (baseline, characterization, seams, CLI, safety, splits, closure) with issue linkage and dependency ordering. Complete for a task-list document.
- **C4 (partial)**: Section 6 (Closure Gates) defines validation gates per slice: focused tests, fmt/clippy, GitNexus detect_changes, GitHub issue updates, OpenSpec validation, and archival.
- **F1 (Technical risk)**: Section 5 large-file splits are correctly gated on "after seams stabilize" (#69, #70). Section 2 correctly sequences characterization tests (#64) before architecture seam changes.
- **F2 (Dependency availability)**: All 19 referenced source files verified present: `strategy/trait_def.rs`, `trade/service.rs`, `cli/handlers/market_output.rs`, `monitor/storage.rs`, `tasks/cron.rs`, `anomaly/detector.rs`, `monitoring/metrics.rs`, `io/batch.rs`, `market/strength.rs`, `monitoring/notification.rs`, `core/runtime.rs`, `execution/kernel.rs`, `miniqmt_market.rs`, `cli/handlers/strategy_handler/requests.rs`, and 11 handler test files.
- **L2 cross-doc**: All 3 referenced design documents exist: `docs/reports/ARCHITECTURE_AUDIT_2026-05-23.md`, `docs/reports/ARCHITECTURE_AUDIT_REVIEW_2026-05-23.md`, `docs/superpowers/specs/2026-05-23-architecture-audit-design.md`.
- **L1 task 4.1 completion verified**: `src/trade/service.rs` has 0 `.expect()` calls remaining, consistent with the `[x]` completion mark.
- **L1 task 2.1 target verified**: `pub enum Signal` found at `src/strategy/trait_def.rs:33`; target file `src/core/signal.rs` does not yet exist (expected, task is unchecked).
- **L1 task 2.5 coupling confirmed**: `src/risk/volatility.rs:8-9` imports `strategy::runtime::StrategyBarLoader` and `strategy::fallback_loader::FallbackStrategyBarLoader`.
- **L1 task 2.6 coupling confirmed**: `src/market/strength.rs` imports from `crate::anomaly`, `crate::db::clickhouse`, and `crate::risk`.
- **L1 task 2.7 upward dependency confirmed**: `src/db/clickhouse/models.rs:5` imports from `crate::market`.
- **L1 task 2.8 test dependency confirmed**: `src/core/runtime.rs:311` imports `crate::test_support::env_lock` (inside a test module).
- **L1 task 3.4 cycle confirmed**: `src/cli/commands/mod.rs:45` imports `crate::cli::handlers`; `src/cli/handlers/mod.rs:22` imports `crate::cli::commands::BacktestCommands`.
- **L1 safety targets confirmed**: `market_output.rs` has 26 `.unwrap()`, `anomaly/detector.rs` has 29 `println!` calls, `monitor/storage.rs` has 3 `.unwrap()`, `tasks/cron.rs` has 2 `.unwrap()`.
- **L1 split targets confirmed**: `miniqmt_market.rs` (1289 lines) exceeds the 800-line force-split threshold; `market/strength.rs` (948), `monitoring/notification.rs` (947), `core/runtime.rs` (935), `execution/kernel.rs` (879), and `strategy_handler/requests.rs` (833) all exceed the 500-line warn threshold.

## Issues

- [ ] **[HIGH]** Task 4.6 describes "production-risk `panic!` paths" in `src/monitoring/metrics.rs` and `src/io/batch.rs`, but all 5 `panic!` calls are inside test match arms (`metrics.rs:355,369,392` and `batch.rs:377,392`). None are in production code paths. — Section 4, task 4.6
      Evidence: Grepped for `panic!` in both files. All 5 hits are within `#[cfg(test)]` test assertion patterns (`_ => panic!("Expected counter")`, `Ok(_) => panic!("expected zero batch_size to be rejected")`). Document was checked for any mention of "test-only" scoping for this task — none found. Task should be rescoped or removed.

- [ ] **[MED]** Task 3.1 says "Extract `HelperContext`, `TradeContext`, and `RiskContext`" from `cli/handlers/mod.rs`, but none of these type names exist anywhere in the codebase (0 hits across all `.rs` files). — Section 3, task 3.1
      Evidence: Grepped for `HelperContext`, `TradeContext`, and `RiskContext` across the entire `src/` tree — no matches. The task should clarify whether these are new abstractions to invent or whether existing code blocks should be identified by a different name.

- [ ] **[MED]** Individual tasks (1.1-1.5, 2.1-2.8, 3.1-3.5, 4.2-4.7, 5.1-5.7) lack specific acceptance criteria. Section 6 defines gates but at the group level, not per-task. — Sections 1-5
      Evidence: Reviewed all task descriptions. Each is a single imperative sentence with no "done when" condition. Document was checked for a shared acceptance-criteria definition — none found. Without per-task criteria, a task like 4.7 ("Add serialization derives only where a real persistence... path requires them") has no objective pass/fail test.

- [ ] **[MED]** Task 4.6 covers `panic!` remediation, but the actual production-risk constructs in `monitoring/metrics.rs` are `.unwrap()` (20 occurrences) and `.expect()` (3 occurrences), not `panic!`. Similarly, `io/batch.rs` has `.unwrap()` (17 occurrences) and `.expect()` (2 occurrences). — Section 4, task 4.6
      Evidence: Grepped for `panic!|\.expect\(|\.unwrap\(\)|assert!` in both files. The panic-specific claim is wrong, but the files do contain real unwrap/expect calls that may include production-risk instances. Document checked — task 4.6 does not mention unwraps or expects for these files.

- [ ] **[LOW]** Task 4.2 targets "production-risk unwraps" in `market_output.rs` (26 total) but doesn't distinguish test-only unwraps from production ones. The task would benefit from a count or scope qualifier. — Section 4, task 4.2
      Evidence: The 26 `.unwrap()` calls in `market_output.rs` were counted without filtering for test modules. Document does not specify how to classify which unwraps are production-risk.

## Suggestions

- Rewrite task 4.6 to target `.unwrap()` and `.expect()` in `metrics.rs` and `batch.rs` (the actual production-risk constructs), or confirm the panics are the only target and close the task as already-safe.
- For task 3.1, add a precondition step: audit `cli/handlers/mod.rs` to identify the actual code blocks that correspond to the three proposed context types, then name them by their existing patterns rather than aspirational names.
- Add a one-line acceptance criterion to each task (e.g., "Done when: `grep -c '\.unwrap()' src/cli/handlers/market_output.rs` shows 0 production-path hits"). This enables objective verification per slice.
- Consider merging tasks 4.2-4.4 and 4.5-4.6 into two focused sub-phases (unwrap remediation, output hygiene) to reduce task count and align with the design doc's "behavior-risk remediation" framing.

## Verdict

APPROVE_WITH_NOTES — Task sequencing and dependency ordering are sound, and all codebase references check out. Task 4.6's factual claim about `panic!` locations is incorrect (all are test-only), and task 3.1 references types that don't exist. Fix these two items and add per-task acceptance criteria to strengthen execution confidence.
