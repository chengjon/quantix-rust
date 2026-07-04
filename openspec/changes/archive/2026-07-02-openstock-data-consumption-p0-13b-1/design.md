# OpenStock P0.13b-1 Design

**Canonical spec:** `docs/superpowers/specs/2026-07-02-openstock-p0-13b-minute-level-data-design.md` (R1 revisions applied per `docs/superpowers/specs/2026-07-02-openstock-p0-13b-design-review.md`).

This OpenSpec change implements only the P0.13b-1 sub-slice (minute candles
via `/data/bars`). P0.13b-2 (time-share via `/data/fetch MINUTE_DATA`) is a
separate OpenSpec change to be opened after P0.13b-1 archives.

## Key Decisions (subset of spec D1-D8)

- **D3**: New `MinutePeriod` enum (not extending P0.13a `BarPeriod`);
  new `MinuteBar` struct (not reusing `Kline`, not colliding with
  `src/db/tdengine.rs:37` `MinuteKline`).
- **D4**: Strict `FromStr` whitelist — only `1m|5m|15m|30m|60m`.
  Defends against OpenStock `_PERIOD_MAP.get(period, "day")` silent
  day-fallback for unknown tokens.
- **D5**: `MinuteBar.timestamp: NaiveDateTime` (vs `Kline.date: NaiveDate`)
  to preserve minute-level precision.
- **Reuse**: Same `/data/bars` endpoint as P0.13a `fetch_klines`; same
  direct-reqwest pattern (no envelope/retry/breaker).

## Risks

- **R1** (silent day-fallback) — mitigated by D4 strict whitelist.
- **R2** (time field format) — verified via wiremock tests using ISO format
  `"2026-07-02T09:31:00+08:00"`; live tests confirm real wire shape.
