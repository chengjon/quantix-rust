# OpenStock Data Consumption P0.10 — Live HTTP Wiring (3 P0 Categories)

## Why

P0.9 (commit `2571003`) shipped the consumer-side scaffolding: uniform envelope types, generic `OpenStockClient` skeleton, three fixture-driven parsers (`STOCK_CODES`, `TRADE_DATES`, `INDEX_KLINES`), and three `Validate*` CLI subcommands. P0.9's `design.md` Migration Path explicitly named **P0.10** as the slice that "wires `OpenStockClient` to live HTTP for the 5 P0 categories, purely additive."

The OpenStock runtime is live at `http://192.168.123.104:8040` (4 providers, 70 categories). The CLI is already async (`#[tokio::main]` in `src/main.rs:12`), the client is async, and three convenience wrappers (`fetch_stock_codes`/`fetch_trade_dates`/`fetch_index_klines`) are already implemented. This slice closes the loop: actually call the live runtime from the CLI for the three read-only P0 categories so the parsers stop being fixture-only.

Two real gaps surfaced during P0.10 exploration and are fixed in this slice:

- **G1 (correctness)**: `OpenStockClient::fetch` consumed the response body without checking HTTP status. On a non-2xx response whose body was not valid JSON, both the success and error envelope parses failed and the actual upstream error message was lost. Fixed by branching on `status.is_success()` before reading the body.
- **G2 (config)**: `OPENSTOCK_BASE_URL` was not yet wired — `OpenStockClient::new` only fell back to `OPENSTOCK_API_KEY`. Added env fallback for `base_url`, mirroring the `BridgeSettings` pattern (`src/core/runtime/settings.rs:55`).

## What Changes

- Edits `src/sources/openstock_client.rs`:
  - **G1**: branch on `resp.status().is_success()` in `fetch`; on non-2xx, parse the uniform error envelope and surface `to_summary()` (or status + body snippet if envelope itself is unparseable).
  - **G2**: `new()` now reads `OPENSTOCK_BASE_URL` env when `cfg.base_url` is empty.
  - Adds `OpenStockClient::from_env()` convenience constructor reading both env vars with default timeout.
  - Extends `OpenStockResponse<T>` with `latency_ms: Option<u64>` (additive — populated from `envelope.latency_ms`).
- Edits `src/cli/commands/data.rs`: adds 3 new `OpenStockCommands` variants — `FetchCodes`, `FetchCalendar { year }`, `FetchIndex { symbol, start?, end? }`.
- Edits `src/cli/handlers/openstock_handler.rs`: adds 3 `pub(crate) async fn` handlers — `fetch_openstock_codes`, `fetch_openstock_calendar`, `fetch_openstock_index`. Each builds `OpenStockClient::from_env()`, calls the corresponding wrapper, prints summary (count, first/last record, source, `artifact_hash`, `latency_ms`). Also drops the dead `IndexKlineParseError` import + P0.9 placeholder hack.
- Edits `src/cli/handlers/mod.rs` + `src/cli/handlers/app_shell.rs`: re-exports + 3 new dispatcher arms.
- Adds 3 `#[ignore]`-gated live integration tests under `tests/openstock_live_{codes,calendar,index}.rs` — gated by `QUANTIX_OPENSTOCK_LIVE=1`; never run in CI.
- Adds `.governance/programs/project-governance/cards/P0.10.yaml`.

## Impact

- New public API: `OpenStockClient::from_env()`, `OpenStockResponse::latency_ms`, 3 new CLI subcommands.
- Additive only — no signature changes, no deletions, no changes to `Kline`/`BacktestEngine`/`ExecutionAdapter`.
- CI regression surface unchanged (live tests are `#[ignore]`).
- Read-only: no ClickHouse writes, no shadow persistence, no state mutation.

## Non-Goals

- No ALL_STOCKS / WORKDAYS live fetchers (parse path exists from P0.9; live wiring deferred).
- No retry / circuit breaker / quality-flags surfacing (one-shot fetch only).
- No ClickHouse or shadow persistence writes (existing `PersistLive` covers the legacy daily-kline path; this slice is read-only).
- No P0.9 governance debt cleanup (P0.8i / P0.9.yaml never created) — acknowledged in `design.md` Risks, not retroactively fixed per user direction.
- No `CliRuntime` / `settings.rs` refactor — env-only configuration is sufficient for this slice.
- No replacement of the existing `Validate*` fixture-only subcommands (parallel, not superseded).
