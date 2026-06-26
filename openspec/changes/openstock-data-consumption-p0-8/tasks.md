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

- [ ] 3.1 Add read-only CLI/status design for OpenStock configuration and local fixture validation.
- [ ] 3.2 Fail closed when no fixture/config is supplied.
- [ ] 3.3 Do not call live OpenStock endpoints in CI.
- [ ] 3.4 Do not write ClickHouse.

## 4. P0.8d Analysis/Backtest Fixture Loop

- [ ] 4.1 Select one downstream path: indicator calculation, backtest, or paper/mock simulation input.
- [ ] 4.2 Use fixture/local artifact data to prove an end-to-end local loop.
- [ ] 4.3 Keep execution adapters unchanged.

## 5. P0.8e Persistence Or Shadow Validation

- [ ] 5.1 Design ClickHouse shadow validation or opt-in persistence separately.
- [ ] 5.2 Include schema, deduplication, rollback, and dry-run gates before any write path.
- [ ] 5.3 Require fresh GitNexus impact and explicit approval.

## 6. Closure

- [ ] 6.1 Update README, CHANGELOG, and FUNCTION_TREE for any completed slices.
- [ ] 6.2 Run full slice gates and PR CI.
- [ ] 6.3 Write Graphiti memory or local backfill if ingest does not complete.
