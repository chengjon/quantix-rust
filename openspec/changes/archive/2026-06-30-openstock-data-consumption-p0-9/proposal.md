# OpenStock Data Consumption P0.9 — Consumer-Side Parsers + Generic Client Skeleton

## Why

P0.8 archived the daily-kline fixture/live-shadow/persist line. OpenStock runtime is now live at `http://192.168.123.104:8040` with 70 categories across 4 providers. Two authoritative docs (`docs/CONNECTION_GUIDE.md`, `docs/DATA_CAPABILITY_SCOPE.md`) confirm the real contract is a uniform `POST /data/fetch` endpoint with `data_category` routing and a uniform response envelope — not the RESTful shape suggested in earlier handoffs.

The three P0 categories quantix-rust needs next are: stock codes (`STOCK_CODES`/`ALL_STOCKS`), trading calendar (`TRADE_DATES`/`WORKDAYS`), and index klines (`INDEX_KLINES`). None of these are covered by the daily-kline parser from P0.8. Before any live network wiring lands, the consumer side needs:

1. A generic client skeleton that knows the `/data/fetch` envelope shape and `X-API-Key` auth.
2. Three read-only parsers that turn category payloads into typed Rust structs.
3. Fixture-driven CLI subcommands so operators can validate captured payloads offline.

This slice establishes the type scaffolding so live wiring in a later slice is purely additive. No live network in CI, no ClickHouse writes, no modification to `Kline`, `BacktestEngine`, or `ExecutionAdapter`.

## What Changes

- Adds `src/sources/openstock_envelope.rs` — raw `OpenStockEnvelope<T>` serde target + `OpenStockErrorEnvelope` + `pub use` re-export of `openstock_shadow::artifact_hash` (single source of truth).
- Adds `src/sources/openstock_client.rs` — generic `OpenStockClient` (reqwest-based, `X-API-Key` from `OPENSTOCK_API_KEY` env) + `OpenStockResponse<T>` public post-parse view with computed `artifact_hash`.
- Adds `src/sources/openstock_codes.rs` — `parse_stock_codes` + `parse_all_stocks` parsers.
- Adds `src/sources/openstock_calendar.rs` — `parse_trade_dates` + `parse_workdays` parsers.
- Adds `src/sources/openstock_index.rs` — `parse_index_klines` parser reusing canonical `Kline`.
- Adds 3 new `OpenStockCommands` CLI variants: `ValidateCodes`, `ValidateCalendar`, `ValidateIndex`.
- Adds 8 fixture files + 4 integration test files (flat under `tests/`).
- Visibility-only widen: `normalize_symbol` and `parse_live_time` → `pub(crate)` in `openstock.rs` for INDEX_KLINES reuse.

## Impact

- New public API surface in `crate::sources` (5 new modules, 3 new CLI subcommands).
- Zero changes to `Kline`, `BacktestEngine`, `ExecutionAdapter`, `ControlledPersistencePolicy`.
- No live network in tests; no ClickHouse writes anywhere.
- Establishes the type scaffolding so P0.10+ live wiring slices are purely additive (no parser rewrite).

## Non-Goals

- No live `/data/fetch` calls in CI or default CLI behavior.
- No ClickHouse or shadow persistence writes.
- No replacement of existing `parse_daily_kline_json` (legacy local-fixture parser stays untouched).
- No refactor of `openstock_shadow.rs` (only re-exports its `artifact_hash`).
- No change to existing `validate-live`/`persist-live` daily-kline paths.
- No new traits — `OpenStockClient` is a struct (single backend expected).
- No `cargo test --doc` regression expansion beyond existing state (added to verification as polish).
