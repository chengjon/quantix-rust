# Architecture Audit Review - quantix-rust

Date: 2026-05-23

> **Status: Correction baseline.** This document is preserved as the authoritative correction source for the primary audit report. The following corrections from this review have been applied to `docs/reports/ARCHITECTURE_AUDIT_2026-05-23.md`: D2 evidence table reclassification (expect/panic/println), F-ARCH-1 recommendation narrowed to Signal-only extraction, Sprint 3 action #15 wording fixed, monitoring/metrics.rs moved to panic! table. Some Phase 0 corrections described below are therefore already applied; check the latest version of all three deliverables before acting.

Reviewed source report: `docs/reports/ARCHITECTURE_AUDIT_2026-05-23.md`

## Method

This review used the mattpocock skills setup for this repo:

- `setup-matt-pocock-skills`: confirmed the existing `CLAUDE.md` agent-skills block and `docs/agents/` configuration are present.
- `improve-codebase-architecture`: reviewed the audit through Module, Interface, Implementation, Depth, Seam, Adapter, Leverage, and Locality.

Evidence sources:

- Graphiti reads from `quantix_rust_main`, `quantix_rust_docs`, and `quantix_rust_review`.
- GitNexus query/cypher/impact checks against the `quantix-rust` index.
- Local source scans for Rust file size, dependency references, test markers, and panic-prone call patterns.

## Executive Verdict

The report is directionally strong and should be kept as the architecture-audit baseline, but it is not ready to become an implementation issue batch as-is.

The D1 dependency/layering findings are mostly validated by GitNexus. The strongest findings are:

- `src/cli/handlers/mod.rs` is the largest import hub with 69 direct imports.
- `src/analysis/backtest.rs -> src/strategy/trait_def.rs` is real.
- `src/risk/volatility.rs -> strategy::{fallback_loader,runtime}` is real.
- `src/risk/industry.rs <-> src/risk/industry_store.rs` is real.
- `src/core/runtime.rs -> src/test_support.rs` is real.
- `src/db/clickhouse/models.rs -> src/market/mod.rs` is real.
- `src/market/strength.rs` imports `db/clickhouse`, `anomaly`, and `risk`.

The main problem is that the optimization plan is still too checklist-shaped. It needs to be reorganized around a small number of deeper Modules with stable Interfaces, instead of treating every large file, missing derive, and warning as equal architecture work.

## Blocking Corrections

### 1. D2 `expect`, `panic`, and `println` sections are misclassified

The original report's D2 tables appear shifted by category. A direct source scan showed:

| Original report section | Original rows | Current source evidence | Required correction |
|---|---:|---:|---|
| `expect() Hotspots` | `src/anomaly/detector.rs` 26, `src/tui/app.rs` 6, `src/core/performance_utils.rs` 5 | these are `println!` counts, not `.expect()` counts | Move to `println! in Library Modules` |
| `panic!() Hotspots` | `src/trade/service.rs` 4, `src/risk/service/state_helpers.rs` 2 | these are `.expect()` counts, not `panic!` counts | Move to `expect() Hotspots` |
| `println! in Library Modules` | `src/monitoring/metrics.rs` 3, `src/io/batch.rs` 2 | these are `panic!` counts, not `println!` counts | Move to `panic!() Hotspots` |

Impact:

- Sprint 2 action #8 is mislabeled. It says "Replace 26 expect() in `anomaly/detector.rs`", but the code has no `.expect()` in that file.
- Sprint 3 action #15 is mislabeled. `monitoring/metrics.rs` and `io/batch.rs` contain `panic!`, not `println!`.
- Any issue generation from these rows would create wrong remediation tickets.

### 2. `strategy/trait_def.rs -> core/strategy_types.rs` is too coarse

The report's first remediation says to extract all of `src/strategy/trait_def.rs` into `src/core/strategy_types.rs`.

That file currently contains two different concepts:

- `Signal`: a small shared value type used by execution, monitoring, strategy, CLI handlers, and runtime storage codec.
- `Strategy`: an async strategy Interface that depends on `crate::data::models::Kline`.

GitNexus impact for `src/strategy/trait_def.rs`:

- 33 impacted files at depth <= 2.
- 12 direct importers.
- Direct importers include strategy implementations, `analysis/backtest.rs`, `monitoring/signal_monitor.rs`, `execution/models.rs`, `execution/kernel.rs`, `execution/runtime_store/codec.rs`, and `cli/handlers/mod.rs`.

Moving the whole file into `core` would make `core` own a strategy Interface that depends on `data::models::Kline`. That risks making `core` less deep and less local: it would become a convenient dumping ground rather than a small stable Module.

Preferred correction:

- Split the shared `Signal` value type into a stable low-level Module first.
- Keep the full `Strategy` Interface in the strategy area until the backtest and runtime use cases are deliberately separated.
- Only move the `Strategy` Interface after deciding whether backtesting should depend on strategy directly or on a separate simulation Interface.

### 3. The roadmap should put characterization before risky refactors

The report correctly identifies safety-critical gaps, especially `risk/`, `stop/`, `factor/`, and `trade/` having no local unit tests by simple marker scan. The roadmap currently places major dependency changes before enough behavior has been pinned down.

Before changing high-blast-radius Modules, add focused characterization tests around:

- `Signal` translation and execution request preparation.
- `risk/industry` resolver behavior and store persistence behavior.
- `risk/volatility` calculation edge cases.
- `stop/service` rule evaluation.
- `trade/service` error handling for the existing `.expect()` hotspots.

This is not cosmetic testing. It is the safety net that lets the architecture work proceed with smaller review scope.

## High-Confidence Architecture Findings

### A. CLI handler hub is real, but the problem is not only file size

Files:

- `src/cli/handlers/mod.rs`
- `src/cli/commands/mod.rs`
- `src/cli/handlers/*`

Problem:

`cli/handlers/mod.rs` acts as a central import and re-export hub. It imports across execution, strategy, risk, market, db, sources, stop, trade, tasks, bridge, analysis, and AI. GitNexus reports 69 direct imports.

The shallow Interface is "import this handlers module and get everything." The Implementation cost is paid globally: changing one domain can force CLI hub churn and can pull unrelated domain concepts into command wiring.

Optimization:

- Make `cli/handlers/mod.rs` a thin registry of handler Modules and public command entrypoints.
- Move domain-specific imports into each concrete handler Module.
- Remove cross-handler dependency through the root handler module.
- Break `cli/commands/mod.rs <-> cli/handlers/mod.rs` by extracting shared command-facing types into a small CLI-owned Module.

Benefits:

- Better Locality: execution changes stay near execution handlers; market output changes stay near market handlers.
- Better Leverage: the CLI root only explains the command tree, not every domain.
- Better tests: each command handler can be tested with local fixtures instead of a huge prelude.

### B. Strategy/execution coupling should be cut through the signal contract first

Files:

- `src/strategy/trait_def.rs`
- `src/execution/models.rs`
- `src/execution/kernel.rs`
- `src/execution/runtime_store/codec.rs`
- `src/monitoring/signal_monitor.rs`
- `src/analysis/backtest.rs`

Problem:

Execution and monitoring mostly need `Signal`, not the full strategy Interface. The current location makes downstream Modules import from `strategy` for a shared value type.

Optimization:

- Extract only the shared signal value type first.
- Update execution, monitoring, runtime codec, CLI, and strategy implementations to use that shared signal Module.
- Leave the higher-level strategy Interface in `strategy` until the backtest/runtime relationship is separately designed.

Benefits:

- Better Depth: a tiny signal Module gives broad callers a stable, small Interface.
- Better Locality: changes to strategy lifecycle do not affect execution signal storage.
- Lower migration risk than moving `Strategy` and `Signal` together.

### C. `risk/industry` has a genuine domain/storage cycle

Files:

- `src/risk/industry.rs`
- `src/risk/industry_store.rs`

Problem:

`industry.rs` imports `SqliteIndustryStore`, while `industry_store.rs` imports domain records and normalization from `industry.rs`. The result is a circular dependency between domain logic and storage implementation.

Optimization:

- Make industry domain records and normalization independent of storage.
- Keep SQLite schema and persistence in the store Adapter.
- Keep resolver orchestration in a separate Module that composes domain and storage.

Benefits:

- Better Locality: schema changes stay in storage; classification behavior stays in domain.
- Better tests: domain parsing/normalization can be tested without SQLite; storage can be tested with fixture DBs.
- Better Leverage: alternate storage becomes possible only after there is more than one Adapter need.

### D. `risk/volatility` depends on strategy because it borrows a bar-loader implementation

Files:

- `src/risk/volatility.rs`
- `src/strategy/fallback_loader.rs`
- `src/strategy/runtime.rs`

Problem:

`risk/volatility.rs` already defines a `RiskBarLoader` Interface, which is good. The unwanted coupling comes from the default Adapter using strategy loaders directly.

Optimization:

- Keep `RiskBarLoader` as the risk-owned Interface.
- Move the default bar-loader Adapter into a neutral market/data loading area, or inject it from the caller.
- Keep volatility calculation over plain bars and risk inputs.

Benefits:

- Better Depth: volatility evaluation remains a focused risk Module.
- Better Locality: strategy loading choices no longer affect risk rules.
- Better tests: risk volatility can use a small fake Adapter without constructing strategy runtime concerns.

### E. `market/strength.rs` combines analysis, storage access, external fetch, and risk classification

Files:

- `src/market/strength.rs`
- `src/db/clickhouse/*`
- `src/risk/industry*`
- `src/anomaly/*`

Problem:

The Module directly imports DB, risk, and anomaly types. It is large and mixes market-strength calculation with data acquisition.

Optimization:

- Keep market-strength calculation over plain input rows.
- Move ClickHouse/fundamental lookup and industry classification behind caller-provided data acquisition Modules.
- Treat EastMoney fetching as an Adapter, not part of the core calculation.

Benefits:

- Better Locality: market scoring rules can change without DB and HTTP changes.
- Better tests: deterministic rows can test strength calculation without ClickHouse or network.
- Better Leverage: the calculation can serve CLI, reports, and future jobs through one stable Interface.

### F. Execution is already partly deep; do not flatten it during cleanup

Files:

- `src/execution/kernel.rs`
- `src/execution/daemon.rs`
- `src/execution/paper.rs`
- `src/execution/runtime_store/*`

Problem:

The report lists execution as low cohesion, but the code already has useful Interfaces such as `ExecutionAdapter`, `RiskEvaluator`, and `FillDeltaApplier`. The risk is not "execution is all bad"; the risk is that daemon/runtime-store/adapter responsibilities are still tangled around a few large files.

Optimization:

- Preserve the existing execution Interfaces.
- Split large Implementations by lifecycle responsibility: request preparation, adapter submission, fill reconciliation, recovery, and persistence.
- Avoid introducing new Interfaces until a second Adapter or second caller proves the Seam is real.

Benefits:

- Better Locality without over-abstracting.
- Better tests around stable execution Interfaces.
- Less chance of replacing working depth with pass-through Modules.

## Recommended Roadmap

### Phase 0 - Fix audit evidence before publishing issues

Actions:

1. Correct D2 `expect`, `panic`, and `println` tables.
2. Refresh line/test counts in D2/D4 from the current worktree.
3. Add a short "Evidence Commands" appendix listing GitNexus queries and local scan criteria.
4. Mark the original report as "reviewed draft" until these corrections land.

Acceptance:

- D2 rows match direct source scans.
- Roadmap action labels match actual code patterns.
- No GitHub issues are created from stale or mislabeled rows.

### Phase 1 - Characterize high-risk behavior

Actions:

1. Add focused tests for signal translation and execution request preparation.
2. Add domain tests for risk industry normalization and resolution.
3. Add persistence tests for `SqliteIndustryStore`.
4. Add volatility-limit tests with fake `RiskBarLoader`.
5. Add stop/trade tests around error paths and existing `.expect()` use.

Acceptance:

- Refactor targets have tests that fail for behavior regressions.
- Tests avoid live QMT, ClickHouse, and network dependencies.

### Phase 2 - Cut low-risk dependency cycles

Actions:

1. Extract the shared signal value type without moving the full strategy Interface.
2. Break `risk/industry.rs <-> risk/industry_store.rs`.
3. Remove `core/runtime.rs -> test_support.rs` from production compilation.

Acceptance:

- GitNexus no longer reports execution/monitoring importing `strategy/trait_def.rs` just to use `Signal`.
- GitNexus no longer reports the `risk/industry` storage cycle.
- Production core no longer imports test support.

### Phase 3 - Make the CLI root deep instead of broad

Actions:

1. Move domain imports out of `cli/handlers/mod.rs`.
2. Keep only module declarations and stable command entrypoints in the root.
3. Extract command/handler shared types into a CLI-owned Module.
4. Split command handler tests so they import the concrete handler being tested.

Acceptance:

- `cli/handlers/mod.rs` import count drops sharply.
- `cli/commands/mod.rs <-> cli/handlers/mod.rs` cycle is gone.
- Handler tests compile without relying on a large root prelude.

### Phase 4 - Move data acquisition behind focused Adapters

Actions:

1. Remove strategy-loader dependency from `risk/volatility.rs`.
2. Move market-strength calculation away from direct ClickHouse/risk/anomaly imports.
3. Remove `db/clickhouse/models.rs -> market/mod.rs` by using shared or DB-local data transfer types.

Acceptance:

- Risk volatility depends on risk inputs, bars, and its own loader Interface.
- Market strength can be tested from in-memory rows.
- DB models no longer import the market Module.

### Phase 5 - Split large Implementations after the Seams are stable

Actions:

1. Split `src/market/strength.rs`.
2. Split `src/monitoring/notification.rs`.
3. Split `src/core/runtime.rs`.
4. Split `src/execution/kernel.rs` by lifecycle responsibility.
5. Split `src/miniqmt_market.rs` only after its current role is documented and covered by tests.

Acceptance:

- Splits reduce file size and improve Locality without adding pass-through Modules.
- New Modules have meaningful Interfaces and focused tests.

### Phase 6 - Clean safety and hygiene issues

Actions:

1. Replace actual `.unwrap()` hotspots by risk order: CLI output panics, storage/scheduler panics, then lower-risk conversions.
2. Replace corrected `.expect()` hotspots in trade/risk.
3. Replace corrected `panic!` hotspots in monitoring/io.
4. Replace corrected `println!` library output in anomaly/TUI/performance utility paths where inappropriate.
5. Add missing derives only where a real persistence/API/serialization path needs them.

Acceptance:

- No remediation issue is based only on a pattern count without a behavior risk.
- Safety cleanup does not expand into unrelated formatting or cosmetic churn.

## Suggested Issue Batch

Do not publish these until Phase 0 corrections are made.

1. Correct architecture audit evidence tables and refresh counts.
2. Add characterization tests for signal/execution/risk-industry/risk-volatility.
3. Extract shared signal type away from `strategy/trait_def.rs`.
4. Split risk industry domain, resolver, and SQLite store responsibilities.
5. Thin `cli/handlers/mod.rs` into a command registry.
6. Decouple `risk/volatility.rs` default loader from strategy runtime.
7. Decouple `market/strength.rs` calculation from DB/risk/anomaly acquisition.
8. Remove `db/clickhouse -> market` dependency.
9. Remove production `core/runtime -> test_support` dependency.
10. Split large Implementations only after their behavior is covered.

## Not Recommended

- Do not move all of `strategy/trait_def.rs` into `core` in one step.
- Do not turn every large file into a split task before dependency direction is corrected.
- Do not add generic Interfaces before there are at least two real Adapters or callers.
- Do not use this audit as a competing feature-status registry; `FUNCTION_TREE.md` remains the feature-status source.
- Do not pursue the older plugin-oriented multi-crate architecture as the default target. Current memory and code evidence say this project is a Rust CLI/TUI execution and analysis engine.

## Final Assessment

Use the original audit as a strong draft, not as a final remediation backlog.

The most valuable next move is to correct the evidence tables and then turn the audit into a smaller architecture program centered on five deepening opportunities:

1. Shared signal contract.
2. CLI command registry.
3. Risk industry domain/storage split.
4. Risk volatility loader Adapter.
5. Market strength calculation Module.

Those five give the best ratio of Leverage to change surface. File-size cleanup and panic cleanup should follow once these Modules are easier to test and reason about.
