# Clippy Cleanup Integrated Plan

**Date**: 2026-06-08
**Inputs**:
- `docs/reports/CLIPPY_DIAGNOSIS_2026-06-07.md`
- `docs/reports/CLIPPY_RECHECK_RECOMMENDATIONS_2026-06-08.md`
- `docs/reports/HANDOFF_CLIPPY_CLEANUP_2026-06-08.md`

## Current State

The clippy cleanup itself should be treated as closed in the current workspace.

Fresh verification on 2026-06-08:

| Gate | Result |
|---|---|
| `cargo clippy --lib -p quantix-cli --message-format short -- -D warnings` | status 0, 0 diagnostics |
| `cargo clippy --workspace --all-targets --all-features --message-format short -- -D warnings` | status 0, 0 diagnostics |
| `cargo fmt --all -- --check` | status 0, clean |
| `cargo test --lib` | status 0, 695 passed |

The handoff document's clippy closure claim is therefore valid, but its "next steps" are not clippy-remediation tasks. They are post-clippy quality-debt tasks and should be managed separately from the completed warning cleanup.

Execution update in this line:

- P1 CI gate hardening has been applied in `.github/workflows/ci.yml`.
- P2 first slice removed production `.unwrap()` calls from `src/core/trading_calendar.rs`.
- P2 second slice removed production `.unwrap()` calls from `src/core/trading_time.rs`.
- P2 third slice removed production `.unwrap()` calls from `src/sources/tdx_file.rs`.
- P2 fourth slice removed production `.unwrap()` calls from `src/sources/tdx_file/fuquan.rs`.
- P2 fifth slice removed production `.unwrap()` calls from `src/anomaly/forest.rs`.
- P2 sixth slice removed production `.unwrap()` calls from `src/analysis/indicators.rs`.
- P2 seventh slice removed production `.unwrap()` calls from `src/analysis/indicators/momentum.rs`.
- P2 eighth slice removed production `.unwrap()` calls from `src/ai/providers/openai_compat.rs`.
- P2 ninth slice removed production `.unwrap()` calls from `src/strategy/fallback_loader.rs`.
- P2 tenth slice removed production `.unwrap()` calls from `src/import/types.rs`.
- P2 eleventh slice removed production `.unwrap()` calls from `src/io/importer.rs`.
- P2 twelfth slice removed production `.unwrap()` calls from `src/tasks/scheduler.rs`.
- P2 thirteenth slice removed production `.unwrap()` calls from `src/watchlist/service.rs`.
- P2 fourteenth slice removed production `.unwrap()` calls from `src/anomaly/detector.rs`.
- P2 fifteenth slice removed production `.unwrap()` calls from `src/io/exporter.rs`.
- P2 sixteenth slice removed production `.unwrap()` calls from `src/db/clickhouse/kline.rs`.
- P2 seventeenth slice removed production `.unwrap()` calls from `src/db/tdengine.rs`.
- P2 eighteenth slice removed production `.unwrap()` calls from `src/execution/kernel/recovery.rs`.
- P2 nineteenth slice removed production `.unwrap()` calls from `src/strategy/momentum.rs`.
- P2 twentieth slice removed production `.unwrap()` calls from `src/strategy/grid.rs`.
- P2 twenty-first slice removed production `.unwrap()` calls from `src/strategy/breakout.rs::calculate_high_low`.
- P2 twenty-second slice removed the remaining production `.unwrap()` call from `src/strategy/breakout.rs::on_bar`.
- P2 twenty-third slice removed production `.unwrap()` calls from `src/strategy/mean_reversion.rs::on_bar`.
- P2 twenty-fourth slice removed a production `.unwrap()` call from `src/analysis/performance.rs::calculate_total_return`.
- P2 LOW-only continuation audit after the twenty-fourth slice found no remaining clear production target that satisfies both the current pre-edit GitNexus rule and the final `detect_changes` LOW closure rule.
- P3 first slice removed two production `.unwrap()` calls from `src/strategy/ma_cross.rs::MACrossStrategy::on_bar` after official approval to leave the P2 LOW-only limit and accept the expected MEDIUM final GitNexus closure.
- P3 second slice removed the production `.unwrap()` call from `src/analysis/indicators_benches.rs::calculate_ema`; pre-edit GitNexus impact was MEDIUM with no affected execution processes.
- `src/cli/handlers/strategy_handler/catalog.rs` was checked as a candidate, but the actual edit target `run_ma_cross_backtest` reported CRITICAL GitNexus impact and remains skipped unless CRITICAL risk is explicitly approved.
- `src/tasks/cron.rs` was checked and its 20 `.unwrap()` calls are test-only, so it was not changed in this slice.

## Handoff Items Rechecked

### `.unwrap()` Debt

Still valid.

The handoff count used a path-only test exclusion. That overcounts files with inline `#[cfg(test)] mod tests` blocks. Use the stricter path + cfg(test)-aware count for production planning.

| Metric | Count |
|---|---:|
| `.unwrap()` in `src/**/*.rs` after P3 second slice | 1,053 |
| Path + cfg(test)-aware production `.unwrap()` count | 16 |
| Test/cfg-test `.unwrap()` count | 1,037 |

Top current path + cfg(test)-aware production hotspots:

| File | Count |
|---|---:|
| `src/strategy/test_utils.rs` | 7 |
| `src/analysis/performance.rs` | 3 |
| `src/analysis/backtest.rs` | 1 |
| `src/analysis/polars_adapter.rs` | 1 |
| `src/cli/handlers/data_handler.rs` | 1 |
| `src/cli/handlers/strategy_handler.rs` | 1 |
| `src/cli/handlers/strategy_handler/catalog.rs` | 1 |
| `src/strategy/daemon.rs` | 1 |

The handoff count of 380 is stale. Use 16 as the current production planning baseline when excluding both test paths and inline cfg-test modules.

Current P3 unwrap-cleanup state:

- `src/strategy/test_utils.rs`: 7 counted production `.unwrap()` calls, but the file is a semantic test utility and remains excluded under the current "avoid ambiguous files such as test_utils.rs" rule.
- `src/analysis/performance.rs`: 3 remaining production `.unwrap()` calls are in `PerformanceCalculator::calculate` / `calculate_annual_return`, both previously assessed as CRITICAL paths.
- `src/strategy/ma_cross.rs`: P3 first slice completed; pre-edit impact for `MACrossStrategy::on_bar` reported LOW, and the expected final MEDIUM `detect_changes(scope=all)` closure is accepted under the official P3 approval.
- `src/analysis/backtest.rs`: `BacktestEngine::new` assessed CRITICAL.
- `src/analysis/indicators_benches.rs`: P3 second slice completed; `calculate_ema` production `.unwrap()` count is now 0 in this file.
- `src/analysis/polars_adapter.rs`: `from_kline_vec` assessed HIGH.
- `src/cli/handlers/data_handler.rs`: `export_data` assessed HIGH.
- `src/cli/handlers/strategy_handler.rs`: `execute_strategy_run_with_risk_service_and_kill_switch` prechecked as LOW, but final `detect_changes(scope=all)` reported HIGH with eight affected execution processes; skip unless HIGH risk is explicitly approved.
- `src/cli/handlers/strategy_handler/catalog.rs`: `run_strategy` prechecked as LOW, but already reports affected processes and is treated as likely final-detect MEDIUM under the current conservative rule; P3 can consider it only after fresh impact analysis.
- `src/strategy/daemon.rs`: `normalize_daily_bar_end` assessed HIGH.

### Large Files

Still valid, and one additional production file should be included in the split backlog.

Current line counts for handoff-listed files:

| File | Current lines |
|---|---:|
| `src/cli/handlers/tests/strategy_execution.rs` | 1,517 |
| `src/miniqmt_market.rs` | 1,459 |
| `src/sources/tdx_api.rs` | 1,310 |
| `src/cli/handlers/import.rs` | 861 |
| `src/cli/handlers/execution_handler.rs` | 836 |
| `src/cli/handlers/monitor_handler.rs` | 814 |

Additional current large production file:

| File | Current lines | Note |
|---|---:|---|
| `src/execution/reconciliation.rs` | 804 | Over the 800-line force-split threshold for ordinary Rust modules |

### CI Gate Hardening

Still valid, but narrower than the handoff suggested.

Current `.github/workflows/ci.yml` already has:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features`
- test jobs

The missing hardening is `-- -D warnings` on the clippy step. Local verification currently passes with `-D warnings`, so this is the lowest-risk next change.

## Recommended Execution Order

### P0 — Preserve Clippy Closure Evidence

Status: already substantially done.

Actions:

1. Keep `docs/reports/CLIPPY_RECHECK_RECOMMENDATIONS_2026-06-08.md` as the current closure note.
2. Treat `docs/reports/CLIPPY_DIAGNOSIS_2026-06-07.md` as historical context, not as an active checklist.
3. If editing that diagnosis report later, add a superseded banner that points to the 2026-06-08 clean gate result.

Do not reopen clippy-cleanup batches unless a live clippy command regresses.

### P1 — CI Gate Hardening

Status: applied in this line.

Change:

```yaml
run: cargo clippy --all-targets --all-features -- -D warnings
```

Why first:

- It is small and contained to `.github/workflows/ci.yml`.
- It prevents warning regressions from reintroducing the completed clippy cleanup.
- It is supported by the current local all-targets/all-features `-D warnings` pass.

Verification:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --message-format short -- -D warnings
cargo test --lib
```

### P2/P3 — `.unwrap()` Removal Program

Second implementation stream. P2 LOW-only cleanup is closed. P3 now allows explicitly selected MEDIUM-risk closure slices, while HIGH and CRITICAL targets still require a separate risk decision. Do not attempt all remaining production `.unwrap()` calls in one change.

Recommended slicing:

1. `core` and scheduler/runtime utilities
   - `src/core/trading_calendar.rs`: first slice completed; production `.unwrap()` count is now 0 in this file.
   - `src/core/trading_time.rs`: second slice completed; production `.unwrap()` count is now 0 in this file.
   - `src/tasks/cron.rs`: checked; current `.unwrap()` calls are test-only and were skipped.
   - `src/tasks/scheduler.rs`: twelfth slice completed; production `.unwrap()` count is now 0 in this file.
   - Replace fallible paths with `?`, typed errors, or `.expect("invariant: ...")` where the invariant is local and defensible.
2. Storage and IO
   - Recheck with cfg-aware counting before editing; many handoff hotspots were test or support code.
   - `src/sources/tdx_file.rs`: third slice completed; production `.unwrap()` count is now 0 in this file.
   - `src/sources/tdx_file/fuquan.rs`: fourth slice completed; production `.unwrap()` count is now 0 in this file.
   - `src/io/exporter.rs`: fifteenth slice completed; production `.unwrap()` count is now 0 in this file.
   - `src/db/clickhouse/kline.rs`: sixteenth slice completed; production `.unwrap()` count is now 0 in this file.
   - `src/db/tdengine.rs`: seventeenth slice completed; production `.unwrap()` count is now 0 in this file.
3. Anomaly and analysis
   - `src/anomaly/forest.rs`: fifth slice completed; production `.unwrap()` count is now 0 in this file.
   - `src/anomaly/detector.rs`: fourteenth slice completed; production `.unwrap()` count is now 0 in this file.
   - `src/analysis/indicators.rs`: sixth slice completed; production `.unwrap()` count is now 0 in this file.
   - `src/analysis/indicators/momentum.rs`: seventh slice completed; production `.unwrap()` count is now 0 in this file.
   - `src/analysis/indicators_benches.rs`: P3 second slice removed the `calculate_ema` previous-value unwrap; production `.unwrap()` count is now 0 in this file.
   - `src/analysis/performance.rs`: twenty-fourth slice removed the LOW-risk `calculate_total_return` unwrap. The remaining production unwraps are still in `PerformanceCalculator::calculate` / `calculate_annual_return`, which reported CRITICAL GitNexus impact; do not use those as casual unwrap-cleanup slices without an explicit risk decision.
4. Account and CLI boundary
   - Recheck with cfg-aware counting before editing; do not rely on the old path-only count.
   - `src/ai/providers/openai_compat.rs`: eighth slice completed; production `.unwrap()` count is now 0 in this file.
   - `src/cli/handlers/strategy_handler.rs`: checked as a P3 candidate, but final `detect_changes(scope=all)` reported HIGH; skip unless HIGH risk is explicitly approved.
   - `src/cli/handlers/strategy_handler/catalog.rs`: checked as a candidate, but `run_ma_cross_backtest` reported CRITICAL GitNexus impact; skip unless CRITICAL risk is explicitly approved.
5. Strategy loaders
    - `src/strategy/fallback_loader.rs`: ninth slice completed; production `.unwrap()` count is now 0 in this file.
    - `src/strategy/momentum.rs`: nineteenth slice completed; production `.unwrap()` count is now 0 in this file.
    - `src/strategy/grid.rs`: twentieth slice completed; production `.unwrap()` count is now 0 in this file.
   - `src/strategy/breakout.rs`: twenty-first and twenty-second slices removed the `calculate_high_low` comparison unwraps and the `on_bar` ATR unwrap; production `.unwrap()` count is now 0 in this file.
   - `src/strategy/mean_reversion.rs`: twenty-third slice removed the `on_bar` RSI and Bollinger Band unwraps; production `.unwrap()` count is now 0 in this file.
   - `src/strategy/ma_cross.rs`: P3 first slice removed the current MA unwraps in `MACrossStrategy::on_bar`; production `.unwrap()` count is now 0 in this file.
6. Import parsing
   - `src/import/types.rs`: tenth slice completed; production `.unwrap()` count is now 0 in this file.
7. IO import
   - `src/io/importer.rs`: eleventh slice completed; production `.unwrap()` count is now 0 in this file.
8. Watchlist service
   - `src/watchlist/service.rs`: thirteenth slice completed; production `.unwrap()` count is now 0 in this file.
9. Execution recovery
   - `src/execution/kernel/recovery.rs`: eighteenth slice completed; production `.unwrap()` count is now 0 in this file.

Rules:

- Before editing functions/classes/methods, run GitNexus impact analysis for the target symbols.
- Prefer propagating typed errors over converting panics into generic strings.
- Keep tests close to the changed module.
- Use `.expect(...)` only for true invariants, with a reason that would help a maintainer.

Verification per slice:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --message-format short -- -D warnings
cargo test --lib
```

Add targeted tests when an `.unwrap()` removal changes error behavior.

### P4 — Large File Split Backlog

Later implementation stream. Each split should be its own PR or isolated branch because these are refactors with larger blast radius.

Recommended order:

1. Test-only split: `src/cli/handlers/tests/strategy_execution.rs`
   - Lower product risk.
   - Good candidate for establishing module naming and test organization conventions.
2. CLI handler split: `src/cli/handlers/import.rs`
   - Extract manifest/import submodules first.
3. Source/API split: `src/sources/tdx_api.rs`
   - Split models/client/endpoints only after impact analysis.
4. miniQMT split: `src/miniqmt_market.rs`
   - Treat as domain-sensitive; preserve parsing and manifest behavior with tests.
5. Execution/monitoring split:
   - `src/cli/handlers/execution_handler.rs`
   - `src/cli/handlers/monitor_handler.rs`
   - `src/execution/reconciliation.rs`

Rules:

- Run GitNexus context and impact before extracting or moving symbols.
- Avoid behavior changes in the same PR as a file split.
- Keep public module exports stable unless the impact report proves all callers are updated.

Verification per split:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --message-format short -- -D warnings
cargo test --lib
```

Use broader targeted tests for each affected domain.

### P5 — `#[allow]` Audit

Later stream, after CI hardening.

The handoff notes intentional allows for:

- `clippy::too_many_arguments`
- `clippy::large_enum_variant`
- `clippy::await_holding_lock`

Action:

1. Inventory all new clippy `#[allow]` annotations from the cleanup period.
2. Confirm each has a local justification.
3. Convert broad file-level allows to narrower symbol-level allows where practical.
4. Do not remove allows merely for cosmetics if that would force disproportionate refactors before the gate loop is stable.

## Immediate Next Action

The cleanup line is closed. P1 CI gate hardening, P2 LOW-only slices, and P3 MEDIUM-closure slices for `src/strategy/ma_cross.rs` and `src/analysis/indicators_benches.rs` have been applied in this line.

Final closure is recorded in `docs/reports/CLIPPY_CLEANUP_FINAL_CLOSURE_2026-06-10.md`.

No remaining production `.unwrap()` cleanup is authorized in this line. The remaining items are officially retained as test-support, HIGH, or CRITICAL risk.

## Operational Guardrails

- Do not run `cargo clippy --fix` directly on a dirty worktree. The handoff records that it can remove imports needed under `#[cfg(test)]`.
- Use live clippy output as the source of truth. Historical warning counts are not current work.
- Treat clippy closure as complete unless a fresh gate command fails.
- Run `gitnexus_detect_changes(scope: "all")` before committing implementation changes.
