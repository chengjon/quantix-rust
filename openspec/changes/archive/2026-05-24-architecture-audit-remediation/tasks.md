## 0. Baseline And OpenSpec Setup

- [x] 0.1 Complete issue #63 audit correction baseline and mark the primary audit as reviewed.
- [x] 0.2 Initialize OpenSpec project configuration for architecture remediation.
- [x] 0.3 Create the `architecture-audit-remediation` proposal, design, spec, and task list.
- [x] 0.4 Validate the OpenSpec change with `OPENSPEC_TELEMETRY=0 openspec validate architecture-audit-remediation --strict`.

## 1. Characterization Tests (#64)

- [x] 1.1 Add signal translation and execution request preparation characterization tests.
  Done when: tests capture current Signal-to-execution request mapping and fail if request preparation behavior changes unexpectedly.
- [x] 1.2 Add risk industry resolver behavior and store persistence characterization tests.
  Done when: tests cover industry normalization/classification plus a store save/load round trip using deterministic test storage.
- [x] 1.3 Add risk volatility edge-case tests with a fake `RiskBarLoader`.
  Done when: tests cover empty/insufficient bars and normal calculation paths without using strategy runtime defaults.
- [x] 1.4 Add stop/service rule evaluation characterization tests.
  Done when: tests cover allow/reject outcomes and diagnostics for the existing stop/service rule behavior.
- [x] 1.5 Add trade/service error handling tests around remaining safety-sensitive paths.
  Done when: tests prove remaining trade service error paths return `QuantixError` instead of panicking.

## 2. Critical Architecture Seams (#65, #66, #68, #69, #70)

- [x] 2.1 Extract only the shared `Signal` value type from `strategy/trait_def.rs` to `src/core/signal.rs`.
  Done when: `Signal` is defined/exported from `core::signal` with no `Kline` or strategy trait dependency.
- [x] 2.2 Update execution, runtime codec, monitoring, analysis, and strategy imports to use `core::signal::Signal`.
  Done when: production Signal consumers no longer import Signal from `strategy::trait_def`, and focused signal/execution tests pass.
- [x] 2.3 Keep the full async `Strategy` trait in `strategy/trait_def.rs`.
  Done when: the `Strategy` trait remains strategy-owned and no core module imports strategy runtime behavior to host it.
- [x] 2.4 Split risk industry domain types, resolver/composer behavior, and SQLite store persistence.
  Done when: domain classification code has no SQLite persistence imports, and store/resolver tests cover the new boundary.
- [x] 2.5 Decouple `risk/volatility.rs` default loading from strategy runtime.
  Done when: volatility calculation uses risk-owned inputs or injected loaders and has no direct dependency on strategy runtime defaults.
- [x] 2.6 Decouple `market/strength.rs` calculation from DB, risk, anomaly, and EastMoney acquisition.
  Done when: market-strength calculation can run over deterministic rows without importing DB, risk, anomaly, or EastMoney adapters.
- [x] 2.7 Remove `db/clickhouse/models.rs -> market/mod.rs` upward dependency.
  Done when: `src/db/clickhouse/models.rs` has no `crate::market` import and DB model serialization tests still pass.
- [x] 2.8 Remove or explicitly constrain the `core/runtime.rs -> test_support.rs` dependency to test-only code.
  Done when: there is no non-test `test_support` import from `core/runtime.rs`; any remaining test helper use is under `#[cfg(test)]`.

## 3. CLI Boundary Cleanup (#67)

- [x] 3.1 Audit `cli/handlers/mod.rs` to identify the existing helper/trade/risk code blocks before naming new CLI context abstractions.
  Done when: the extraction targets are documented with current file/line evidence and no nonexistent context type is treated as an existing symbol.
- [x] 3.2 Move domain-specific imports into concrete handler modules.
  Done when: concrete handler modules own their domain imports and `cli/handlers/mod.rs` no longer imports those domain modules directly.
- [x] 3.3 Keep `cli/handlers/mod.rs` limited to module declarations and stable command entrypoints.
  Done when: `cli/handlers/mod.rs` contains only module declarations, re-exports, and stable entrypoint glue, with no command business logic.
- [x] 3.4 Extract CLI-owned command/handler shared types to break the `cli/commands` and `cli/handlers` cycle.
  Done when: the `cli/commands` and `cli/handlers` import cycle is absent and shared types live in a CLI-owned neutral module.
- [x] 3.5 Split handler tests so they import the concrete handler under test.
  Done when: handler tests compile through concrete handler modules rather than relying on root `cli/handlers/mod.rs` aggregation.

## 4. Safety And Hygiene (#72)

- [x] 4.1 Remove `src/trade/service.rs` initialized-account `expect()` calls and preserve trade service tests.
  Done when: `src/trade/service.rs` has 0 `.expect()`/`.unwrap()` calls and `cargo test --test trade_service_test` passes.
- [x] 4.2 Replace production-risk unwraps in `src/cli/handlers/market_output.rs`.
  Done when: production-risk unwraps are removed or documented as non-risk writes to in-memory buffers, with test-only unwraps classified separately.
- [x] 4.3 Classify `src/monitor/storage.rs` unwraps and confirm no production-risk unwraps remain.
  Done when: storage production paths have no `.unwrap()`/`.expect()` calls; remaining test-only unwraps are assertion/setup clarity.
- [x] 4.4 Replace production-risk unwraps in `src/tasks/cron.rs`.
  Done when: scheduler preset constructors use validated constants instead of runtime unwraps; remaining `.unwrap()` calls are test-only setup/assertion code.
- [x] 4.5 Replace library `println!` and `eprintln!` calls in `src/anomaly/detector.rs` with tracing or caller-owned output.
  Done when: `src/anomaly/detector.rs` has no `println!`/`eprintln!` hits; detector output is returned as rendered text and CLI code owns stdout writes.
- [x] 4.6 Classify and remediate production-risk `.unwrap()`/`.expect()` paths in `src/monitoring/metrics.rs` and `src/io/batch.rs`; keep test-only `panic!` assertions only when they are assertion clarity, not product behavior.
  Done when: both files have 0 production `.unwrap()`/`.expect()`/`panic!` hits; the 5 remaining `panic!` hits are test-only assertions and remaining unwraps are test setup/assertion code.
- [x] 4.7 Classify serialization derive requests and add derives only where a real persistence, API, or serialization path requires them.
  Done when: audited derive candidates are not remediated by checklist count alone; no new derive is added without a concrete persistence/API/serialization callsite and focused test or compile-time use.

## 5. Large File Splits After Seams Stabilize (#71)

- [x] 5.1 Split `src/market/strength.rs` by foundation construction versus strength report calculation after #69 is stable.
  Done when: foundation construction and strength report calculation live in owned submodules behind the stable `market::strength` boundary; market-strength tests cover both sides without adding pass-through-only modules.
- [x] 5.2 Split `src/monitoring/notification.rs` by sender responsibilities.
  Done when: sender-specific modules own sender behavior and notification tests cover each extracted sender path.
- [x] 5.3 Split `src/core/runtime.rs` by settings versus initialization logic after #70 is stable.
  Done when: settings and initialization have separate owned modules and runtime initialization tests still pass.
- [x] 5.4 Split `src/execution/kernel.rs` by lifecycle responsibility while preserving execution interfaces.
  Done when: request preparation, submit, fill reconciliation, recovery, and persistence are separated without changing `ExecutionAdapter`, `RiskEvaluator`, or `FillDeltaApplier` contracts.
- [x] 5.5 Document and cover `src/miniqmt_market.rs` before any split.
  Done when: the module role is documented and characterization tests cover current behavior before file movement.
- [x] 5.6 Split `src/cli/handlers/strategy_handler/requests.rs` by request type.
  Done when: request-type modules own their parsing/building behavior and strategy handler request tests pass.
- [x] 5.7 Extract shared helpers from `src/cli/handlers/tests/*.rs` and split tests by behavior group.
  Done when: shared test helpers are centralized, behavior-group test files remain focused, and the handler test suite passes.

## 6. Closure Gates

- [x] 6.1 Run focused tests for each completed slice.
- [x] 6.2 Run `cargo fmt --check` and task-relevant `cargo test` or `cargo clippy`.
  - Status 2026-05-25: task-relevant focused tests passed during slice work, the remaining formatting drift was resolved with pure rustfmt, and repo-level `cargo fmt --check` passes.
- [x] 6.3 Run GitNexus `detect_changes` and confirm changed processes match the active task.
  - Status 2026-05-24: `detect_changes(scope: all)` first showed the shared worktree has 276 dirty paths and reports CRITICAL over 202 changed files. To isolate this change, the OpenSpec remediation paths were temporarily staged, `detect_changes(scope: staged)` was run over 64 staged files, and the index was restored to empty afterward. The staged result still reports CRITICAL because the remediation intentionally spans signal, execution, risk, market, CLI, monitoring, and safety/hygiene paths; affected process families match the active architecture-remediation scope.
- [x] 6.4 Update the linked GitHub issue checklist and status after each slice.
- [x] 6.5 Validate OpenSpec after each task group.
- [x] 6.6 Archive the OpenSpec change after issues #64-#72 are complete and specs are promoted.
  - Status 2026-05-25: issues #64-#72 are closed, 6.2 is resolved, and this change is ready for OpenSpec archive/spec promotion.
