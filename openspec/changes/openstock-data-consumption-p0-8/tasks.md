# OpenStock Data Consumption P0.8 Tasks

## 0. Baseline And Governance

- [x] 0.1 Confirm work starts from clean `master` after P0.7d backfill merge.
- [x] 0.2 Run Graphiti reads for `quantix_rust_main` and `quantix_rust_docs`.
- [x] 0.3 Run GitNexus overview/detect_changes before edits.
- [x] 0.4 Create a dedicated FUNCTION_TREE P0.8 node before editing planning files.
- [ ] 0.5 Create this OpenSpec change as the governing scope for OpenStock data consumption.
- [ ] 0.6 Run `openspec validate openstock-data-consumption-p0-8 --strict`.
- [ ] 0.7 Run `openspec validate --all --strict`.
- [ ] 0.8 Run FUNCTION_TREE scope-check, validate, and gate.
- [ ] 0.9 Run GitNexus detect_changes before committing.

## 1. P0.8a Inventory And Contract Map

- [x] 1.1 Map current stock identity, kline, quote, market foundation, analysis, backtest, and paper/mock data consumers.
- [x] 1.2 Identify current source modules and classify each as existing, external-runtime-dependent, fixture-testable, or persistence-related.
- [x] 1.3 Produce a report with the first implementation candidate and GitNexus impact targets.
- [x] 1.4 Do not change production code unless a separate P0.8a implementation node authorizes it.

## 2. P0.8b Provider Contract And Fixture Parser

- [x] 2.1 Define the smallest OpenStock fixture-owned input shape.
- [x] 2.2 Add RED parser/normalization tests from committed fixtures.
- [x] 2.3 Implement only the minimal parser/normalizer required for GREEN.
- [x] 2.4 Preserve existing `tdx_api`, `bridge_tdx`, `eastmoney`, and miniQMT behavior.

## 3. P0.8c CLI Status And Fixture Validation

- [x] 3.1 Add read-only CLI/status design for OpenStock configuration and local fixture validation.
- [x] 3.2 Fail closed when no fixture/config is supplied.
- [x] 3.3 Do not call live OpenStock endpoints in CI.
- [x] 3.4 Do not write ClickHouse.

## 4. P0.8d Analysis/Backtest Fixture Loop

- [x] 4.1 Select one downstream path: indicator calculation, backtest, or paper/mock simulation input.
- [x] 4.2 Use fixture/local artifact data to prove an end-to-end local loop.
- [x] 4.3 Keep execution adapters unchanged.

## 5. P0.8e Persistence Or Shadow Validation

- [x] 5.1 Design ClickHouse shadow validation or opt-in persistence separately.
- [x] 5.2 Include schema, deduplication, rollback, and dry-run gates before any write path.
- [x] 5.3 Require fresh GitNexus impact and explicit approval.

## 5f. P0.8f Executable Live Shadow Validation

- [x] 5f.1 Authorize slice via FUNCTION_TREE P0.8f node (status: approved-for-implementation).
- [x] 5f.2 Capture live baseline evidence: service address, X-API-Key requirement, /data/bars 100-row return shape, start/end/limit not honored by service.
- [x] 5f.3 Run GitNexus impact on Kline (CRITICAL) and confirm slice only reads it.
- [x] 5f.4 TDD RED tests for validate_live_shadow_payload (valid mapping, limit drift, out-of-window drift, missing symbol, bad time, non-daily period, mixed symbol, invalid envelope, empty envelope, Display impl).
- [x] 5f.5 Implement read-only validator and report type; never call network, never write ClickHouse.
- [x] 5f.6 Wire `quantix data openstock validate-live` CLI reading captured payload from file or stdin.
- [x] 5f.7 CLI integration tests: valid, drift, fail-closed, missing-file failure.
- [x] 5f.8 Run cargo fmt --check, clippy -D warnings, full test suite, git diff --check, GitNexus detect_changes.

## 5g. P0.8g Shadow Persistence Opt-in Design Gate

- [x] 5g.1 Consume P0.8f `LiveShadowReport` as input contract; reject persistence on any drift or fail-closed.
- [x] 5g.2 Document shadow namespace (`quantix_shadow`), table (`openstock_daily_kline_shadow`), full schema, batch identity (`batch_id` + `artifact_hash`), and deduplication key (`source + period + code + date + adjust_type`).
- [x] 5g.3 Document two-stage write model (dry-run preview + `--apply` with `QUANTIX_SHADOW_PERSIST_CONFIRM=yes`), rollback command (`shadow-rollback --batch-id`), partial-write failure behavior, and operator runbook.
- [x] 5g.4 Document CI non-write proof obligations (default tests do not touch ClickHouse; opt-in gates).
- [x] 5g.5 Record GitNexus impact targets for the future implementation slice; explicitly forbid reuse of `ControlledPersistencePolicy` (HIGH) and modification of `Kline` (CRITICAL hub, read-only).
- [x] 5g.6 Pure design gate — no production Rust source changes, no ClickHouse writes, no live network.

## 5h. P0.8h Analysis Wider Fixture Loop (Indicators + Strategy)

- [ ] 5h.1 Authorize slice via FUNCTION_TREE P0.8h node (status: approved-for-implementation, test-only).
- [ ] 5h.2 Commit a larger fixture `tests/fixtures/openstock/daily_kline_30d.json` (~30 trading days, code 600000) that parses via `parse_daily_kline_json`.
- [ ] 5h.3 Indicator loop test: extract close/high/low/volume from fixture `Vec<Kline>` and fan through `sma`, `ema`, `wma`, `bollinger_bands`, `atr`, `obv`, `cci`, `williams_r`; assert no panic and `Some(...)` present in last-window outputs.
- [ ] 5h.4 Strategy loop test: feed each `Kline` into `MACrossStrategy::new(short, long)` via `Strategy::on_bar`; assert non-empty signal sequence from `{Buy, Sell, Hold}`.
- [ ] 5h.5 Run `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --test openstock_analysis_wider_loop_test`, full test suite.
- [ ] 5h.6 Run OpenSpec single/all strict, FUNCTION_TREE validate/gate, `git diff --check`, GitNexus `detect_changes` (confirm test-only scope, 0 src/ symbols touched).
- [ ] 5h.7 Backtest option explicitly deferred: `BacktestEngine` has no public `run`/`feed` method; will require a separate production-code slice with its own governance card.

## 6. Closure

- [ ] 6.1 Update README, CHANGELOG, and FUNCTION_TREE for any completed slices.
- [ ] 6.2 Run full slice gates and PR CI.
- [ ] 6.3 Write Graphiti memory or local backfill if ingest does not complete.
