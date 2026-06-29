# tdx-api Import E2E Hardening

## Why

The tdx-api bridge is integrated into quantix-rust and package tests pass, but the remaining release risk is the real dependency path:

- tdx-api to ClickHouse K-line import through `quantix data tdx-api import-klines`.
- tdx-api to TDengine tick import through `quantix data tdx-api import-ticks`.

Unit and command tests prove wiring, but they do not prove that real tdx-api, ClickHouse, and TDengine deployments accept the imported data, preserve the expected source metadata, or remain safe on incremental reruns. This change makes those import paths explicitly acceptable, releasable, and rerunnable.

## What Changes

- Establish an OpenSpec-governed E2E hardening change for tdx-api import release readiness.
- Require dependency preflight evidence for tdx-api, ClickHouse, and TDengine before E2E gates run.
- Require ClickHouse K-line E2E evidence that exercises the `import-klines --all` path, with an explicit approved market bound such as `--exchange sh` when full-market execution is too large for a repeatable release gate.
- Require TDengine tick E2E evidence for `import-ticks --code <code> --date <YYYYMMDD>`.
- Require post-import quality checks for row count, date/time ordering, source tagging, OHLCV or tick-field sanity, and incremental rerun behavior.
- Treat post-cleanup release build verification as a closure gate because the previous session did not capture a current release-build exit code.
- Keep repository-wide warning cleanup, logging hygiene, DataSource abstraction changes, streaming, notification, scheduling, factor, and backtest work outside this change.

## Capabilities

### New Capabilities

- `tdx-api-import-e2e`: governs real-dependency acceptance for tdx-api ClickHouse K-line import and TDengine tick import.

### Modified Capabilities

- None. This change does not modify an existing OpenSpec capability by itself.

## Impact

- Adds an active OpenSpec change under `openspec/changes/tdx-api-import-e2e-hardening/`.
- Adds a value analysis report under `docs/reports/TDX_API_FOLLOWUP_VALUE_ANALYSIS_2026-06-06.md`.
- Does not change runtime code by itself.
- Future implementation may add E2E scripts, evidence templates, tests, or command hardening around:
  - `src/cli/handlers/tdx_api_handler.rs`
  - `src/sources/tdx_api.rs`
  - `src/db/clickhouse/kline.rs`
  - `src/db/tdengine.rs`
  - `scripts/`
  - `docs/reports/evidence/`
- Requires live or test-owned instances of tdx-api, ClickHouse, and TDengine. Secrets and host-specific credentials must not be committed.

## Non-Goals

- Do not vendor the tdx-api service source into this repository.
- Do not perform broad clippy cleanup.
- Do not convert unrelated `println!` calls to `tracing`.
- Do not introduce a unified `DataSource` abstraction.
- Do not add WebSocket streaming, factor computation, backtest auto-preparation, progress UI, notification, or systemd packaging in this change.
