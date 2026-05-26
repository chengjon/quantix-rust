# Architecture Audit Report — quantix-rust

> **Status: REVIEWED BASELINE** — Cross-checked against `docs/reports/ARCHITECTURE_AUDIT_REVIEW_2026-05-23.md`; D2 evidence tables and roadmap labels have been corrected.
> Use the correction baseline and design spec resolution rules before publishing or implementing remediation issues.
> Do not create GitHub issues from the pre-review version.

> Date: 2026-05-23
> Reviewed: 2026-05-23
> Branch: master
> Method: GitNexus Graph Analysis + Pattern Scan + Test Coverage Analysis
> Scope: Module Dependencies, Code Quality, Module Cohesion, Test Coverage
> Previous Audit: 2026-05-15 (runtime gates + pattern scan)

---

## Executive Summary

**314 Rust files / 80,261 lines / 28 top-level modules** audited across four dimensions.

Key findings:

- **2 CRITICAL architectural issues**: strategy <-> execution bidirectional coupling; cli/handlers as a 69-import megahub
- **8 CRITICAL code quality issues**: file size violations, unwrap hotspots (430 production calls), mod.rs business logic
- **10 CRITICAL test coverage gaps**: large modules (risk, factor, stop, trade) with zero unit tests
- **Well-isolated modules** (factor, bridge, news, ai, fundamental, import) serve as the target architecture pattern

**Estimated remediation effort**: 3-4 focused sprints for CRITICAL items; 2-3 additional sprints for HIGH items.

---

## Finding Summary

| Severity | Count | Categories |
|----------|-------|------------|
| CRITICAL | 20 | Architecture (2), File Size (8), unwrap/expect (4), mod.rs Violation (1), Test Coverage (5) |
| HIGH | 19 | Architecture (4), File Size (8), unwrap (6), panic (1), Test Coverage (5), println (2) |
| MEDIUM | 13 | Architecture (6), mod.rs (2), expect (1), println (1), error-swallow (1), Test Coverage (2) |
| LOW | 5 | Architecture (1), Test Coverage (3), Other (1) |

---

## D1: Module Dependency & Layering

### Expected Layer Architecture

```
cli → service → provider/adapter → domain → core
```

### Actual Dependency Topology (GitNexus Graph, 5410 symbols, 13194 edges)

**Well-isolated modules** (only import core, no external deps):
factor, bridge, news, ai, fundamental, import

**Moderate coupling** (1-3 external deps):
io (→ data), sync (→ db, sources), screener (→ analysis, data, watchlist), watchlist (self-contained)

**High coupling** (4+ external deps):
strategy, execution, risk, monitoring, market, sources, cli/handlers

### Findings

#### F-ARCH-1 [CRITICAL] strategy <-> execution bidirectional coupling

**Evidence**:
- `execution/models.rs` → `strategy/trait_def.rs`
- `execution/kernel.rs` → `strategy/trait_def.rs`
- `execution/runtime_store/codec.rs` → `strategy/trait_def.rs`
- `strategy/runtime.rs` → `execution/models.rs`
- `strategy/registry.rs` → `execution/models.rs`
- `strategy/daemon.rs` → `execution/runtime_store/mod.rs`, `execution/config.rs`, `execution/models.rs`

**Impact**: Neither module can be compiled, tested, or evolved independently. Changes in either cascade unpredictably.

**Recommendation**: Extract only the shared `Signal` value type from `strategy/trait_def.rs` into a new `src/core/signal.rs`. Do NOT move the full `Strategy` async interface — it depends on `data::models::Kline` and moving it to core would make core a dumping ground. Execution, monitoring, and runtime codec mostly need `Signal`, not `Strategy`. Keep the `Strategy` interface in `strategy/` until the backtest/runtime relationship is separately designed. This also partially resolves F-ARCH-3 and F-ARCH-7.

**Blast radius**: 8+ files across 4 modules (execution, strategy, analysis, monitoring).

---

#### F-ARCH-2 [CRITICAL] cli/handlers/mod.rs is a 69-import megahub

**Evidence**: `src/cli/handlers/mod.rs` imports from 16 different modules (execution 7, strategy 4, fundamental 5, market 3, monitor 2, risk, screener, sources, stop, trade, tasks, watchlist, ai 2, analysis 3, bridge, data, db/clickhouse). 54 direct callers.

**Impact**: Any change in any module potentially affects the handler hub. The `new()` constructor alone has 54 callers — highest blast radius in the entire codebase.

**Recommendation**:
1. Extract `HelperContext`, `TradeContext`, `RiskContext` from mod.rs to separate files
2. Use dependency injection instead of direct imports
3. Group handlers by domain (trading handlers, data handlers, analysis handlers)

---

#### F-ARCH-3 [HIGH] analysis → strategy upward dependency

**Evidence**:
- `src/analysis/indicator_config.rs` → `src/strategy/mod.rs`
- `src/analysis/backtest.rs` → `src/strategy/trait_def.rs`

**Impact**: The analysis layer (domain/service) imports from strategy (higher-level service). Reverses expected dependency direction.

**Recommendation**: Move shared types to `core/` or `data/models.rs`. For backtest, accept a trait object instead of importing strategy's trait_def.

---

#### F-ARCH-4 [HIGH] risk → strategy upward dependency

**Evidence**:
- `src/risk/volatility.rs` → `src/strategy/fallback_loader.rs`
- `src/risk/volatility.rs` → `src/strategy/runtime.rs`

**Impact**: Risk management (a service that strategy/execution should call into) imports strategy internals.

**Recommendation**: Invert the dependency — risk/volatility should receive inputs as plain data parameters. Strategy calls risk, not the reverse.

---

#### F-ARCH-5 [HIGH] risk/industry <-> industry_store circular import

**Evidence**: `src/risk/industry.rs` imports `src/risk/industry_store.rs` and vice versa.

**Impact**: Domain model and storage layer are mutually dependent. Violates the provider/adapter pattern.

**Recommendation**: Make `industry.rs` a pure domain type with no storage imports. Move storage logic entirely to `industry_store.rs`.

---

#### F-ARCH-6 [HIGH] execution depends on risk and trade (sideways coupling)

**Evidence**:
- `execution/daemon.rs` → `risk/{mod,service}`
- `execution/{paper,daemon}` → `trade/mod`

**Recommendation**: Inject risk service and trade service as trait parameters rather than direct imports.

---

#### F-ARCH-7 [MEDIUM] monitoring → strategy upward dependency

**Evidence**: `src/monitoring/signal_monitor.rs` → `src/strategy/trait_def.rs`

**Recommendation**: Resolved by F-ARCH-1 fix (extract trait_def to shared layer).

---

#### F-ARCH-8 [MEDIUM] core/runtime.rs → test_support.rs

**Evidence**: `src/core/runtime.rs` → `src/test_support.rs`

**Impact**: Production code in the lowest layer depends on test utilities.

**Recommendation**: Feature-gate with `#[cfg(test)]` or move shared logic into core proper.

---

#### F-ARCH-9 [MEDIUM] db/clickhouse → market upward dependency

**Evidence**: `src/db/clickhouse/models.rs` → `src/market/mod.rs`

**Recommendation**: Define DB models without importing market module types. Use parameters or shared domain types.

---

#### F-ARCH-10 [MEDIUM] cli/commands <-> cli/handlers circular

**Evidence**: `src/cli/commands/mod.rs` <-> `src/cli/handlers/mod.rs`

**Recommendation**: Extract shared types to `cli/types.rs`.

---

#### F-ARCH-11 [MEDIUM] cli/handlers/market_handler <-> market_output circular

**Evidence**: `src/cli/handlers/market_handler.rs` <-> `src/cli/handlers/market_output.rs`

**Recommendation**: Define shared types in a separate file.

---

#### F-ARCH-12 [MEDIUM] market/strength.rs has 3 non-core dependencies

**Evidence**: `src/market/strength.rs` → `db/clickhouse`, `anomaly`, `risk`

**Recommendation**: Receive data through service parameters, not direct imports.

---

### Most-Connected Symbols (Highest Blast Radius)

| Symbol | File | Callers | Risk |
|--------|------|---------|------|
| `new` (constructor) | `src/cli/handlers/mod.rs` | 54 | CRITICAL |
| `insert_run` | `src/execution/runtime_store/mod.rs` | 42 | HIGH |
| `set_rule` | `src/risk/service.rs` | 38 | HIGH |
| `init_account` | `src/trade/service.rs` | 35 | HIGH |
| `Kline` | `src/data/models.rs` | 33 | HIGH |
| `insert_signal` | `src/execution/runtime_store/signals.rs` | 26 | HIGH |
| `find_order_by_client_order_id` | `src/execution/runtime_store/orders.rs` | 25 | HIGH |
| `capture` | `src/core/runtime.rs` | 24 | MEDIUM |

---

## D2: Code Quality & Technical Debt

### File Size Violations

#### Force-Split (>800 lines)

| File | Lines | Recommendation |
|------|-------|----------------|
| `src/cli/handlers/tests/strategy_requests.rs` | 1244 | Split by test group |
| `src/cli/handlers/tests/strategy_execution.rs` | 1171 | Split by test group |
| `src/cli/handlers/tests/mod.rs` | 1040 | Extract helpers, split tests |
| `src/market/strength.rs` | 948 | Types → `strength/types.rs`, logic → `strength/calculator.rs` |
| `src/monitoring/notification.rs` | 947 | Per-sender files |
| `src/core/runtime.rs` | 935 | Settings → `runtime/settings.rs`, init → `runtime/init.rs` |
| `src/execution/kernel.rs` | 879 | Recovery logic → `kernel/recovery.rs` |
| `src/cli/handlers/strategy_handler/requests.rs` | 833 | Split by request type |

#### WARN (>500 lines, top entries)

| File | Lines |
|------|-------|
| `src/cli/handlers/import.rs` | 788 |
| `src/cli/handlers/monitor_handler.rs` | 767 |
| `src/cli/handlers/app_shell.rs` | 738 |
| `src/monitoring/alert.rs` | 708 |
| `src/cli/handlers/mod.rs` | 706 |
| `src/cli/handlers/risk.rs` | 703 |
| `src/sources/tdx_file.rs` | 685 |
| `src/cli/handlers/account.rs` | 683 |
| `src/execution/algo/vwap.rs` | 606 |
| `src/analysis/performance.rs` | 600 |
| `src/cli/handlers/data_handler.rs` | 599 |
| `src/cli/handlers/market_output.rs` | 582 |
| `src/monitoring/performance_monitor.rs` | 560 |
| `src/execution/runtime_store/orders.rs` | 545 |
| `src/execution/models.rs` | 536 |
| `src/anomaly/features.rs` | 529 |
| `src/sources/websocket.rs` | 524 |
| `src/account/router.rs` | 520 |
| `src/risk/models.rs` | 510 |
| `src/tasks/scheduler.rs` | 501 |
| `src/stop/service.rs` | 501 |
| `src/risk/import_store.rs` | 501 |

### unwrap() Hotspots (Production Code)

**~430 production unwrap() calls** (970 total including tests)

| File | Count | Severity |
|------|-------|----------|
| `src/cli/handlers/market_output.rs` | 51 | CRITICAL |
| `src/monitor/storage.rs` | 29 | CRITICAL |
| `src/tasks/cron.rs` | 27 | CRITICAL |
| `src/io/exporter.rs` | 18 | HIGH |
| `src/cli/handlers/risk.rs` | 16 | HIGH |
| `src/account/router.rs` | 13 | HIGH |
| `src/account/registry.rs` | 13 | HIGH |
| `src/monitoring/metrics.rs` | 11 | HIGH |
| `src/io/importer.rs` | 11 | HIGH |
| `src/market/strength.rs` | 9 | MEDIUM |

### expect() Hotspots

| File | Count | Severity |
|------|-------|----------|
| `src/trade/service.rs` | 4 | HIGH |
| `src/risk/service/state_helpers.rs` | 2 | MEDIUM |

### panic!() Hotspots

| File | Count | Severity |
|------|-------|----------|
| `src/monitoring/metrics.rs` | 3 | HIGH |
| `src/io/batch.rs` | 2 | MEDIUM |

> Note: Production `panic!()` is rare. Most 206 total matches are in test code or macro-generated code.

### println!/eprintln! in Library Modules (Forbidden)

| File | Count | Severity |
|------|-------|----------|
| `src/anomaly/detector.rs` | 25 `println!` + 1 `eprintln!` | CRITICAL |
| `src/tui/app.rs` | 6 | MEDIUM |
| `src/core/performance_utils.rs` | 5 | MEDIUM |

### mod.rs Business Logic Violations

| File | Lines | Functions | impl Blocks | Severity |
|------|-------|-----------|-------------|----------|
| `src/cli/handlers/mod.rs` | 706 | 5+ | 3 | CRITICAL |
| `src/execution/runtime_store/mod.rs` | 475 | 4 | 1 | HIGH |
| `src/db/clickhouse/mod.rs` | 195 | 4 | 3 | HIGH |
| `src/cli/commands/mod.rs` | 267 | 1 | 1 | MEDIUM |
| `src/execution/algo/mod.rs` | 60 | 2 | 2 | MEDIUM |

### Missing Derives (Serialize/Deserialize)

~25-30 data transfer structs across `factor/`, `io/`, `execution/` are missing `Serialize, Deserialize` derives. Key examples:

- `factor/`: `FactorLoadRequest`, `FactorComputeRequest`, `FactorComputeResult`, `FactorScoreResult`, `FactorIcResult`, `FactorDataset`, `NeutralizationRequest`
- `io/`: `ExportResult`, `BatchConfig`, `BatchProgress`, `ImportConfig`, `ImportResult`, `ValidationConfig`, `ValidationResult`, `DataQualityReport`
- `execution/`: 6 record/struct types

---

## D3: Module Responsibility & Cohesion

GitNexus identified 44 functional communities across the codebase.

### Well-Cohesive Modules (1-2 communities)

| Module | Assessment |
|--------|------------|
| factor | Excellent — purely focused |
| io | Excellent |
| bridge | Excellent |
| screener | Good |
| trade | Good |
| anomaly | Good |
| account | Good |
| news | Good |
| ai | Good |

### Low-Cohesion Modules (3+ communities, mixed concerns)

| Module | Communities | Issue |
|--------|-------------|-------|
| strategy | Strategy (21 files), Strategy_handler (11 files), partly in Analysis | Bridges execution, analysis, and CLI handler concerns |
| execution | Execution (18 files), Runtime_store (4), Daemon (2), Algo (7) | Spans runtime storage, algorithmic trading, kernel, daemon, bridge |
| cli/handlers | Handlers (57+ files), Strategy_handler (11 files) | Hub connects to virtually every module |
| market | Market, Clickhouse, Sentiment | Combines strength analysis, sentiment, and DB queries |
| risk | Risk (11 files), Import_store (3), Market (2) | Mixes industry analysis, volatility, and import/export |
| monitoring | Monitoring core, Alert, Notification, Performance | Multiple notification channels + performance monitoring |

### Cross-Module Community Leaks

The **Strategy** community spans: strategy (13 files), risk (3 files), analysis (3 files), execution (runtime_store/signals.rs), and CLI handlers — confirming the bidirectional coupling issues from D1.

### Confusing Module Pairs

| Pair | Overlap | Recommendation |
|------|---------|----------------|
| `monitor/` vs `monitoring/` | Both deal with monitoring concerns | Clarify: `monitor` = runtime monitor service, `monitoring` = alert/notification infrastructure. Document the distinction. |
| `strategy/` vs `execution/` | Tightly coupled (see F-ARCH-1) | Extract shared types to core layer |

---

## D4: Test Coverage & Quality

### Overall Statistics

| Metric | Count |
|--------|-------|
| `#[cfg(test)]` blocks | 103 |
| `#[test]` functions (unit) | 468 |
| `#[test]` functions (integration) | 246 |
| Total test functions | **714** |
| Integration test files | 75 |
| Assert macros (unit) | ~620 |
| Assert macros (integration) | ~715 |

### Module Coverage Map

| Module | Lines | Unit Tests | Int Tests | Coverage | Risk |
|--------|-------|------------|-----------|----------|------|
| cli | 25,727 | 169 | ~200 | GOOD | — |
| monitoring | 3,968 | 59 | — | GOOD | — |
| analysis | 4,204 | 29 | 25 | GOOD | — |
| execution | 6,114 | 15 | ~37 | GOOD | — |
| strategy | 2,887 | 14 | ~19 | GOOD | — |
| sources | 3,083 | 21 | — | GOOD | — |
| anomaly | 2,628 | 27 | — | GOOD | — |
| io | 1,774 | 24 | — | GOOD | — |
| core | 1,890 | 29 | — | GOOD* | — |
| **risk** | **3,103** | **0** | ~16 | **NONE** | **CRITICAL** |
| **factor** | **2,330** | **0** | 3 | **NONE** | **CRITICAL** |
| **stop** | **1,083** | **0** | ~20 | **NONE** | **CRITICAL** |
| **trade** | **871** | **0** | ~27 | **NONE** | **CRITICAL** |
| **news** | **730** | **0** | **0** | **NONE** | **CRITICAL** |
| account | 1,466 | 4 | 2 | PARTIAL | HIGH |
| bridge | 629 | 0 | ~9 | PARTIAL | HIGH |
| db | 1,659 | 9 | — | PARTIAL | HIGH |
| market | 1,891 | 7 | ~2 | PARTIAL | HIGH |
| monitor | 1,522 | 0 | ~22 | PARTIAL | HIGH |
| screener | 468 | 0 | ~20 | PARTIAL | HIGH |
| watchlist | 683 | 0 | ~18 | PARTIAL | HIGH |
| fundamental | 1,278 | 14 | — | PARTIAL* | MEDIUM |
| import | 1,129 | 13 | — | PARTIAL* | MEDIUM |
| tasks | 1,184 | 7 | — | PARTIAL | MEDIUM |
| miniqmt_market | 1,289 | 1 | ~34 | PARTIAL | CRITICAL |
| data | 163 | 0 | 0 | NONE | LOW |
| tui | 22 | 0 | 0 | NONE | NEGLIGIBLE |

*Good test count but low assert density — tests may only verify compilation.

### Critical Large Files Without Tests

| File | Lines | Tests | Issue |
|------|-------|-------|-------|
| `src/miniqmt_market.rs` | 1,289 | 1 | Largest file, 1 test for URI parsing only |
| `src/cli/handlers/import.rs` | 788 | 0 | Import handler completely untested |
| `src/cli/handlers/monitor_handler.rs` | 767 | 1 | Handler near-untested |
| `src/cli/handlers/risk.rs` | 703 | 1 | Handler near-untested |
| `src/cli/handlers/account.rs` | 683 | 2 | Handler near-untested |
| `src/risk/models.rs` | 510 | 0 | Risk models untested |
| `src/risk/import_store.rs` | 501 | 0 | Risk storage untested |
| `src/risk/service.rs` | 479 | 0 | Risk service untested |
| `src/stop/service.rs` | 501 | 0 | Stop-loss logic untested |
| `src/monitor/storage.rs` | 489 | 0 | Monitor storage untested |
| `src/factor/evaluation.rs` | 308 | 0 | Factor evaluation untested |
| `src/factor/scoring.rs` | 283 | 0 | Factor scoring untested |
| `src/factor/catalog.rs` | 285 | 0 | Factor catalog untested |
| `src/trade/service.rs` | 243 | 0 | Trade service untested |
| `src/trade/models.rs` | 296 | 0 | Trade models untested |

### Test Quality Flags

| Module | Test Count | Assert Count | Asserts/Test | Assessment |
|--------|-----------|-------------|--------------|------------|
| fundamental | 14 | 4 | 0.29 | WEAK — likely smoke/compilation tests |
| import | 13 | 4 | 0.31 | WEAK — likely smoke/compilation tests |
| market/strength.rs | 7 | 1 | 0.14 | WEAK — near-zero behavioral testing |
| core/runtime.rs | 24 | ~8 | 0.33 | MODERATE — low assert ratio |

---

## Optimization Roadmap

### Sprint 1: Architecture Critical Fixes

| # | Action | Resolves | Effort | Impact |
|---|--------|----------|--------|--------|
| 1 | Extract shared `Signal` value type from `strategy/trait_def.rs` → `src/core/signal.rs` (NOT the full Strategy interface) | F-ARCH-1, F-ARCH-3(partial), F-ARCH-7 | M | Breaks strategy-execution deadlock for Signal consumers |
| 2 | Decouple `risk/volatility.rs` from strategy internals | F-ARCH-4 | S | Risk becomes independent |
| 3 | Break `risk/industry.rs` ↔ `industry_store.rs` cycle | F-ARCH-5 | S | Clean domain/storage boundary |
| 4 | Remove `core/runtime.rs` → `test_support.rs` dependency | F-ARCH-8 | S | Production code no longer depends on test helpers |

### Sprint 2: Code Quality Critical Fixes

| # | Action | Resolves | Effort | Impact |
|---|--------|----------|--------|--------|
| 5 | Replace 51 unwrap() in `market_output.rs` | unwrap hotspot #1 | M | Prevents CLI output panics |
| 6 | Replace 29 unwrap() in `monitor/storage.rs` | unwrap hotspot #2 | M | Prevents data loss on storage errors |
| 7 | Replace 27 unwrap() in `tasks/cron.rs` | unwrap hotspot #3 | M | Prevents scheduler crashes |
| 8 | Replace 25 `println!` + 1 `eprintln!` in `anomaly/detector.rs` with tracing | print macro hotspot #1 | M | Prevents library stdout/stderr noise in anomaly pipeline |
| 9 | Extract business logic from `cli/handlers/mod.rs` | mod.rs violation | L | Enables independent compilation |

### Sprint 3: File Size Reduction & Module Cleanup

| # | Action | Resolves | Effort | Impact |
|---|--------|----------|--------|--------|
| 10 | Split `market/strength.rs` (948 lines) | File size | M | Maintainability |
| 11 | Split `monitoring/notification.rs` (947 lines) | File size | M | Maintainability |
| 12 | Split `core/runtime.rs` (935 lines) | File size | M | Maintainability |
| 13 | Split `execution/kernel.rs` (879 lines) | File size | M | Maintainability |
| 14 | Fix `db/clickhouse` → `market` upward dependency | F-ARCH-9 | S | Clean DB layer |
| 15 | Replace panic! in `monitoring/metrics.rs` and `io/batch.rs` | Safety violations | S | Proper error returns |

### Sprint 4: Test Coverage Expansion

| # | Action | Resolves | Effort | Impact |
|---|--------|----------|--------|--------|
| 16 | Add unit tests to `risk/` (3,103 lines, 0 tests) | P0 coverage | L | Safety-critical module |
| 17 | Add unit tests to `stop/` (1,083 lines, 0 tests) | P0 coverage | M | Capital protection |
| 18 | Add tests to `miniqmt_market.rs` (1,289 lines, 1 test) | P0 coverage | M | Largest file |
| 19 | Add unit tests to `factor/` (2,330 lines, 0 tests) | P1 coverage | L | Quantitative correctness |
| 20 | Add tests to CLI handlers: import, monitor, risk, account | P1 coverage | L | User-facing correctness |

### Sprint 5: Polish & Hygiene

| # | Action | Resolves | Effort | Impact |
|---|--------|----------|--------|--------|
| 21 | Add Serialize/Deserialize to ~25-30 data transfer structs | Derive compliance | M | API/persistence consistency |
| 22 | Fix error swallowing in systemd modules | Error hygiene | S | Proper error logging |
| 23 | Document `monitor/` vs `monitoring/` distinction | Cohesion clarity | S | Developer clarity |
| 24 | Improve assert density in fundamental/ and import/ tests | Test quality | M | Behavioral correctness |
| 25 | Clean up remaining mod.rs violations (clickhouse, algo, commands) | mod.rs hygiene | S | Architecture compliance |

---

## Appendix A: Well-Isolated Module Pattern

The following modules represent the **target architecture pattern** for the codebase:

- **factor**: Zero external imports (only core). Single GitNexus community. Clean module boundary.
- **bridge**: Zero external imports. Single community. Self-contained adapter.
- **news**: Zero external imports. Single community. Clean provider pattern.
- **ai**: Zero external imports. Single community. Isolated decision engine.
- **fundamental**: Zero external imports. Single community. Isolated data provider.
- **import**: Zero external imports. Single community. Isolated import pipeline.

When refactoring coupled modules, aim for this pattern: **depend only on core, form a single functional cluster, no cross-module imports beyond core**.

---

## Appendix B: Module Dependency Graph (Simplified)

```
cli/handlers (hub, 69 imports)
├── execution ←→ strategy (BIDIRECTIONAL, CRITICAL)
│   ├── execution → risk, trade (sideways)
│   └── strategy → execution/models (upward)
├── analysis → strategy (upward, violation)
├── risk → strategy (upward, violation)
├── monitoring → strategy (upward, violation)
├── market → db/clickhouse, anomaly, risk
├── sources → data, bridge
├── screener → analysis, data, watchlist
├── sync → db, sources
├── monitor → watchlist, stop, trade
└── [well-isolated: factor, bridge, news, ai, fundamental, import]
        └── all depend only on core
```

---

## Appendix C: Evidence Commands

Commands used to generate this audit. Re-run to verify or refresh:

```bash
# File sizes
find src/ -name "*.rs" | while read f; do lines=$(wc -l < "$f"); echo "$lines $f"; done | sort -rn | head -50

# unwrap() hotspot (production only)
grep -rn "\.unwrap()" src/ --include="*.rs" | grep -v "/tests/" | grep -v "#\[cfg(test)\]" | cut -d: -f1 | sort | uniq -c | sort -rn | head -20

# print macros in library modules
rg -n "\b(e)?println!\s*\(" src/ --glob "*.rs" --glob "!**/cli/**" --glob "!**/tests/**" --glob "!main.rs"

# Test counts per module
for mod in src/*/; do name=$(basename "$mod"); unit=$(grep -rn "#\[test\]" "$mod" --include="*.rs" 2>/dev/null | wc -l); echo "$unit $name"; done | sort -rn

# GitNexus circular dependency query
# gitnexus cypher: MATCH (a)-[:CodeRelation {type:'IMPORTS'}]->(b)-[:CodeRelation {type:'IMPORTS'}]->(a) RETURN a.filePath, b.filePath

# GitNexus module cohesion
# gitnexus cypher: MATCH (f)-[r:CodeRelation {type:'MEMBER_OF'}]->(c:Community) RETURN c.heuristicLabel, count(f) ORDER BY count(f) DESC
```
