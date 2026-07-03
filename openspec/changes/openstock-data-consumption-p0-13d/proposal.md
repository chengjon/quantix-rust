# OpenStock Data Consumption P0.13d — Streaming Fetch

## Why

P0.13c's batch API accumulates an entire range into one `Vec<MinuteBar>` / `Vec<MinuteShare>`. A 6-month 1m kline range ≈ 36,000 records held in memory at once; CLI handlers further fan this out by printing every record. There is no server-side pagination primitive in OpenStock (`fetching.py:232` `execute_bars_payload` has no `limit/offset/cursor`). P0.13d adds client-side range chunking and a streaming API so callers can process large ranges batch-by-batch.

## What Changes

- New `OpenStockClient::fetch_minute_klines_stream` returning `impl Stream<Item = Result<Vec<MinuteBar>, QuantixError>>` (weekly chunks)
- New `OpenStockClient::fetch_minute_share_stream` returning `impl Stream<Item = Result<Vec<MinuteShare>, QuantixError>>` (daily batches; non-trading days yield empty Vec)
- Private helper `fetch_minute_klines_range(start, end)` extracted from existing `fetch_minute_klines` body
- Private pure function `chunk_range_weekly(start, end) -> Vec<(NaiveDate, NaiveDate)>`
- CLI `--stream` flag (default `false`) on `fetch-minute-klines` and `fetch-minute-share`; prints per-batch progress to stderr when set
- Existing batch APIs (`fetch_minute_klines` / `fetch_minute_share`) unchanged in signature, wire shape, and behavior

## Impact

- `src/sources/openstock_client.rs`: +180 lines (2 stream methods, 1 helper extract, 1 chunk fn, 7 unit tests)
- `src/cli/commands/data.rs`: +6 lines (2 × `stream: bool` field)
- `src/cli/handlers/openstock_handler.rs`: +50 lines (2 streaming branches + warning text)
- `src/cli/handlers/app_shell.rs`: +6 lines (destructure `stream` in 2 arms)
- `tests/openstock_live_minute_klines.rs`: +60 lines (1 live test)
- `tests/openstock_live_minute_share.rs`: +50 lines (1 live test)
- OpenSpec change (4 files) + governance card: +230 lines

## Non-Goals

- Server-side pagination protocol (none exists; this slice does not invent one)
- ClickHouse batch inserts from stream (separate slice)
- Backpressure / flow control (`Vec<T>` per batch is the natural unit)
- Refactor batch API to `stream.collect` (D6 rejected — causes P0.13a/b/c churn)
- Streaming for other fetchers (daily klines, index, etc.) — separate slice
- klines retry (R4 follow-up; candidate for P0.13e)
- P0.13c §13 R6 (server-side MINUTE_DATA range) — blocked on OpenStock server
