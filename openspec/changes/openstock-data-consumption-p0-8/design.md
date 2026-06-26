# OpenStock Data Consumption P0.8 Design

## Context

Current repository evidence shows several market-data paths, each with a distinct boundary:

- `tdx_api` is an existing configured REST data source with separate CLI and E2E hardening work.
- `bridge_tdx`, `tdx`, `tdx_file`, `eastmoney`, and `websocket` are existing source modules.
- miniQMT market-manifest tooling is dry-run/report oriented and must not be turned into a default persistence path without separate gates.
- `FUNCTION_TREE.md` still records AKShare/TDX stock-info and kline gaps in the broader sources boundary.

OpenStock should be introduced as a separate upstream data-consumption line, not as a hidden alias for qmt_live, miniQMT, or the existing tdx-api follow-up.

## Operating Principles

1. Broker independence first.

   The OpenStock line must be usable without miniQMT, qmt_live, broker credentials, or a Windows Bridge runtime.

2. Fixture-owned development first.

   Parser, normalization, and CLI-status tests must start from committed fixtures or synthetic local artifacts. Live network calls are opt-in runtime behavior, not default CI behavior.

3. Read-only before persistence.

   The first implementation slices should parse, normalize, validate, and display. ClickHouse writes require a later explicit schema, rollback, deduplication, and dry-run gate.

4. No broad source replacement.

   OpenStock must coexist with `tdx_api`, `bridge_tdx`, `eastmoney`, and miniQMT tooling. Migration or routing changes require separate impact analysis.

5. Downstream loop alignment.

   The final target is a runnable local path: OpenStock data -> indicators -> backtest or paper/mock execution. P0.8 should sequence work toward that loop without coupling it to real broker execution.

## Slice Order

### P0.8a: Inventory And Contract Map

Produce a report that maps current types, tables, commands, and tests relevant to stock identity, daily/minute kline, quote, market foundation, indicators, backtest, and paper/mock execution. This is documentation-only unless a narrow test fixture is needed to prove current behavior.

### P0.8b: Provider Contract And Fixture Parser

Introduce the smallest OpenStock provider contract and fixture parser. The contract should avoid global data-source rewrites and expose only the data shape required by the first downstream loop, likely daily kline or stock identity.

### P0.8c: CLI Status And Fixture Validation

Add a read-only CLI/status surface that reports provider configuration and validates a local fixture/artifact. It must not perform network calls by default and must not write ClickHouse.

### P0.8d: Analysis/Backtest Fixture Loop

Wire parsed fixture/local artifact data into an existing analysis or backtest path. The slice should prove a local data -> indicator/backtest path without changing execution adapters.

### P0.8e: Persistence Or Shadow Validation

Only after the above is stable, define ClickHouse shadow validation or opt-in persistence with schema, deduplication, rollback, and failure-boundary tests.

## Risk Decisions

- `src/cli/handlers/import.rs`, `src/miniqmt_market.rs`, and existing market CLI handlers are likely higher impact than a new isolated provider module; implementation slices must run GitNexus impact before any symbol edit.
- `tdx_api` hardening is already an active OpenSpec change and should not be conflated with OpenStock.
- Any write path must be treated as a separate higher-risk persistence专项.

## Non-Goals

- No qmt_live runtime readiness restart.
- No miniQMT registry or market-manifest behavior changes.
- No global `DataSource` trait rewrite in P0.8 planning.
- No live-network CI tests.
- No production code changes in this OpenSpec planning slice.
