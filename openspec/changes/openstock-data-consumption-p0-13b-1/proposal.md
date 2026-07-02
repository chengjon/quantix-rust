# OpenStock Data Consumption P0.13b-1

## Why

P0.13a delivered day/week/month K-line fetching via `/data/bars`. The
HANDOFF report row 35 (corrected in this slice) tags minute-level K-line
candles as the next priority for short-timeframe signal / backtest
workloads. OpenStock's `_PERIOD_MAP` accepts `1m|5m|15m|30m|60m` on the
same `/data/bars` endpoint, so this slice is purely additive client
wiring — no server-side changes.

## What Changes

- Add `MinutePeriod` enum (strict 1m|5m|15m|30m|60m FromStr — rejects
  all aliases like `1min`/`minute`/`1h` to defend against OpenStock
  `_PERIOD_MAP` silent day-fallback for unknown tokens).
- Add `MinuteBar` struct (NaiveDateTime timestamp — distinct from P0.13a
  `Kline`'s NaiveDate; named `MinuteBar` not `MinuteKline` to avoid
  collision with `src/db/tdengine.rs:37` existing `MinuteKline` f64 type).
- Add `OpenStockClient::fetch_minute_klines(code, period, date, adjust)`
  returning `Vec<MinuteBar>` via direct `/data/bars` reqwest (no envelope,
  no retry, no breaker — matching `fetch_klines`).
- Add CLI `data openstock fetch-minute-klines` subcommand.

## Impact

- New files: `tests/openstock_live_minute_klines.rs`.
- Modified files: `src/data/models.rs`, `src/sources/openstock_client.rs`,
  `src/cli/commands/data.rs`, `src/cli/handlers/openstock_handler.rs`,
  `src/cli/handlers/mod.rs`, `src/cli/handlers/app_shell.rs`.
- No DB writes, no persistence — read-only consumption.
- No regression to P0.13a's `BarPeriod`/`Kline`/`fetch_klines`.

## Non-Goals

- Time-share point series via `/data/fetch MINUTE_DATA` (deferred to P0.13b-2).
- Multi-day range queries (single `date` param only; range is P0.13c).
- ClickHouse writes / shadow persistence (read-only).
- Retry / circuit breaker on `/data/bars` (matches `fetch_klines` P0.13a decision).
