# OpenStock Data Consumption P0.8 OpenSpec

Date: 2026-06-26

Status: planning scope established

## Summary

P0.8 establishes OpenStock data consumption as the next primary broker-independent development line after qmt_live P0.6 was archived as `blocked_by_environment` and ExecutionCapabilities P0.7 was closed.

This slice creates the OpenSpec scope only. It does not change production Rust code.

## Current Context

Graphiti and repository scans show:

- miniQMT market-manifest tooling exists and remains dry-run/report oriented.
- `tdx_api` exists as a separate configured data-source line with its own OpenSpec hardening work.
- `bridge_tdx`, `eastmoney`, `tdx`, `tdx_file`, and `websocket` source modules already exist.
- AKShare/TDX stock-info and kline gaps remain documented in `FUNCTION_TREE.md`.
- There is no explicit OpenStock provider contract yet.

## Decision

OpenStock will be introduced as a separate upstream data-consumption line, not as a hidden alias for qmt_live, miniQMT, or tdx-api.

The first implementation work after P0.8 should be P0.8a inventory/contract mapping, followed by a fixture-owned provider parser. This keeps the path independent from broker runtime availability and avoids accidental ClickHouse writes.

## Slice Plan

- P0.8a: inventory current models, source modules, storage boundaries, CLI consumers, and first implementation candidate.
- P0.8b: define a minimal OpenStock provider contract and fixture parser.
- P0.8c: add read-only provider status/local fixture validation CLI.
- P0.8d: connect fixture/local artifact data to one analysis/backtest path.
- P0.8e: design opt-in persistence or ClickHouse shadow validation only after schema/rollback gates exist.

## Non-Goals

- No production Rust code changes in this slice.
- No live OpenStock calls in CI.
- No ClickHouse writes.
- No qmt_live runtime probing, canary, submit/query/cancel, bridge protocol, storage schema, `ExecutionAdapter`, or `OrderStatus` changes.
- No miniQMT registry or market-manifest behavior changes.
- No broad replacement of existing data-source modules.
- No `.unwrap()` cleanup.

## Verification

Required gates:

- `openspec validate openstock-data-consumption-p0-8 --strict`
- `openspec validate --all --strict`
- `git diff --check`
- FUNCTION_TREE `scope-check`
- FUNCTION_TREE `validate`
- FUNCTION_TREE `gate --verbose`
- GitNexus `detect_changes`
