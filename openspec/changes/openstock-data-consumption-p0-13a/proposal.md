# OpenStock Data Consumption P0.13a — Multi-period K-line Fetch

## Why

The HANDOFF report `docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md`
lists 8 ❌ rows on the quantix-rust consumer side. P0.13a-d decompose
them into 4 slices; this change covers the B-group row (multi-period
K-line + adjust type — 3 categories: `KLINES`, `ADJUSTED_KLINES`,
`HISTORICAL_KLINES`) which OpenStock `/data/bars` already serves
transparently.

## What Changes

- Add `BarPeriod` enum (`Day`/`Week`/`Month`) in `src/data/models.rs`
  with `as_str()` and strict case-insensitive `FromStr` (rejects
  `daily`/`weekly`/`monthly`/`minute*` aliases).
- Extend `AdjustType` with `as_openstock_param() -> Option<&'static str>`
  and a case-insensitive `FromStr` (`none|qfq|hfq`).
- Add `OpenStockClient::fetch_klines(code, period, adjust, start, end)`
  in `src/sources/openstock_client.rs` — generalizes
  `fetch_daily_klines` (preserved unchanged) to week/month periods and
  qfq/hfq adjust. Stamps each `Kline` with the requested `AdjustType`.
- Add `data openstock fetch-klines` CLI subcommand with `--symbol`,
  `--period`, `--adjust`, `--start`, `--end`.
- 8 tests across 3 layers: 5 unit/wiremock + 3 live `#[ignore]`.

## Impact

**Files added:** 5 (1 live test file, 4 OpenSpec files).
**Files modified:** 6 (`data/models.rs`, `openstock_client.rs`,
`commands/data.rs`, `openstock_handler.rs`, `app_shell.rs`,
`handlers/mod.rs`).
**Public API:** new `BarPeriod`, new method on `AdjustType`, new
`fetch_klines` method on `OpenStockClient`, new CLI subcommand. No
breaking changes.

## Non-Goals

- Minute-level periods (`MINUTE_DATA`) — P0.13b.
- `ADJUST_FACTOR` raw factor exposure — P0.13d+.
- ClickHouse / shadow persistence integration for new periods — later slice.
- Refactor `fetch_daily_klines` to call `fetch_klines` — later slice.
- Retry / circuit breaker for `/data/bars` path — P0.10 design decision preserved.
