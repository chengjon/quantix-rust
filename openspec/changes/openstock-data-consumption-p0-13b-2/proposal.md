# Proposal: openstock-data-consumption-p0-13b-2

## Why

P0.13b-1 added minute OHLC candles via `/data/bars` (KLINES minute periods).
P0.13b-2 completes the minute-level story by adding intraday time-share ticks
via the `MINUTE_DATA` category. Time-share ticks (price + avg_price per
minute, no OHLC) are required to render intraday 分时图 charts in MyStocks
frontend.

The two paths are architecturally orthogonal: P0.13b-1 uses direct reqwest
(no retry); P0.13b-2 uses the `/data/fetch` envelope path (with retry +
circuit breaker, same path as fetch_stock_codes). Each fits in its own
slice for risk isolation.

## What Changes

- Add `MinuteShare` struct in `src/data/models.rs` (Option-wrapped fields
  for INV-2C skip semantics)
- Add `fetch_minute_share(code, date)` client method in
  `src/sources/openstock_client.rs` — calls `self.fetch::<T>("MINUTE_DATA", params)`
- Add `parse_minute_share` + `parse_time_minutes` inline helpers
- Add `FetchMinuteShare { --symbol, --date }` CLI subcommand
- Add `fetch_openstock_minute_share` handler
- Add 3 `#[ignore]` live tests (L1/L2/L3)
- Add governance card `P0.13b-2.yaml`

## Impact

| Area | Change |
|------|--------|
| `src/data/models.rs` | +60 lines (struct + tests) |
| `src/sources/openstock_client.rs` | +180 lines (method + helpers + tests) |
| `src/cli/commands/data.rs` | +6 lines (enum variant) |
| `src/cli/handlers/openstock_handler.rs` | +30 lines (handler) |
| `src/cli/handlers/mod.rs` | +1 line (re-export) |
| `src/cli/handlers/app_shell.rs` | +3 lines (dispatcher arm) |
| `tests/openstock_live_minute_share.rs` | +60 lines (new file) |

Total: ~340 lines added, 0 deleted. No P0.13b-1 code modified.

## Non-Goals

- Multi-day range queries (P0.13c)
- ClickHouse writes / shadow persistence integration
- Other categories (REALTIME_QUOTES, depth, etc.)
- Refactoring envelope retry/circuit breaker
- Migrating existing parsers to a dedicated module (cross-slice refactor)
