# Proposal: openstock-data-consumption-p0-13c

## Why

P0.13b-1/2 fetchers (`fetch_minute_klines` + `fetch_minute_share`) accept only
single-day `date` parameters, forcing callers to loop manually for backfills.
Parent design (P0.13b) deferred range support to P0.13c. This slice adds
multi-day range queries with an asymmetric design driven by OpenStock server
capabilities (R1-revised).

## What Changes

- New `DateOrRange` enum in `src/data/models.rs` (mutex: `Date` | `Range`)
  with `from_cli` validator and `iter_dates_inclusive` helper
- `fetch_minute_klines` accepts `DateOrRange`; server-side range via
  `/data/bars` `start_date`/`end_date` body fields
- `fetch_minute_share` accepts `DateOrRange`; client-side per-day loop
  (OpenStock MINUTE_DATA server does not support range; reads
  `meta.trading_date` per response envelope for the correct record date)
- CLI `fetch-minute-klines` + `fetch-minute-share` add `--start`/`--end`;
  `--date` becomes `Option<String>` (backward-compatible superset)
- `from_cli` validates mutex + rejects `(None, None, None)` and semi-open
  ranges (errors name the offending flags per spec §4.3)

## Impact

| Area | Change |
|------|--------|
| `src/data/models.rs` | +90 lines (enum + `from_cli` + helper + tests) |
| `src/sources/openstock_client.rs` | Modified `fetch_minute_klines` signature; refactored `fetch_minute_share` into dispatcher + `fetch_minute_share_single` helper; new wiremock tests W1/W3/W5 |
| `src/cli/commands/data.rs` | `--date` -> `Option<String>` + `--start`/`--end` flags on 2 subcommands |
| `src/cli/handlers/openstock_handler.rs` | 2 handlers extended with start/end params + `from_cli` validation |
| `src/cli/handlers/app_shell.rs` | 2 dispatcher arms pass new params |
| `tests/openstock_live_minute_klines.rs` | +L1 multi-day range test |
| `tests/openstock_live_minute_share.rs` | +L2 multi-day range test + L3 from_cli rejection test |

Backward compatibility: P0.13b-1/2 wiremock tests pass unchanged (Date mode
wire body is identical). Existing live tests wrapped in `DateOrRange::Date`.

## Non-Goals

- ClickHouse writes / shadow persistence for multi-day data
- Pagination / streaming for huge ranges
- Cross-period candle merge (different period candles stay separate)
- Other categories' range extension (REALTIME_QUOTES, depth, etc.)
- Awaiting OpenStock server-side MINUTE_DATA range support (deferred per D6;
  switchable later without signature change)
