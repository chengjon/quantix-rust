# OpenStock Data Consumption P0.8

## Why

The project goal is to make the A-share quant workflow genuinely runnable without waiting on qmt_live runtime availability. P0.6 qmt_live readiness is archived as `blocked_by_environment`, while ExecutionCapabilities P0.7 is closed. The next primary line should therefore focus on a broker-independent market-data path that can support:

- market data ingestion planning;
- indicator and strategy calculation;
- backtest and paper/mock execution loops;
- later qmt_live integration only after its external runtime exists.

OpenStock is the intended upstream data provider for this line, but the repository currently has no explicit OpenStock consumption contract. Existing data-source code includes `tdx_api`, `bridge_tdx`, `eastmoney`, miniQMT market manifest dry-run tooling, and documented AKShare/TDX kline gaps. P0.8 formalizes how OpenStock will fit into those boundaries before implementation begins.

## What Changes

P0.8 adds an OpenSpec change that defines the OpenStock data-consumption roadmap, slice order, acceptance gates, and non-goals.

The implementation roadmap is split into independently reviewable slices:

- P0.8a: inventory current data models, kline/quote abstractions, storage tables, and CLI consumers.
- P0.8b: define an OpenStock provider contract and fixture-owned parser tests without live network calls.
- P0.8c: add a read-only CLI/status surface for provider configuration and sample fixture validation.
- P0.8d: connect parsed data into the existing analysis/backtest path using fixture or local artifact data only.
- P0.8e: add opt-in persistence or ClickHouse shadow validation only after schema and rollback gates are approved.

## Impact

- Adds OpenSpec proposal, design, task list, and spec delta files.
- Updates project status documentation to mark OpenStock data consumption as the next primary line.
- Adds no production Rust behavior in this planning slice.
- Establishes that qmt_live runtime readiness stays archived/maintenance-only until an operator provides the miniQMT bridge runtime.

## Non-Goals

- No qmt_live canary, runtime probing, submit/query/cancel changes, bridge protocol changes, or broker-state writes.
- No `ExecutionAdapter`, `OrderStatus`, or runtime storage schema changes.
- No ClickHouse writes in the planning slice.
- No live OpenStock HTTP calls in tests.
- No replacement of existing `tdx_api`, `bridge_tdx`, `eastmoney`, or miniQMT market-manifest paths in one broad rewrite.
- No `.unwrap()` cleanup.
