# Design: openstock-data-consumption-p0-13c

Full design rationale: `docs/superpowers/specs/2026-07-03-openstock-p0-13c-multi-day-range-design.md`
(R1 revision). This file is a quick-reference summary; the spec is authoritative.

## Key Decisions

- **D1**: `DateOrRange` enum (`Date` | `Range { start, end }`) — makes the
  two valid shapes unrepresentable-as-invalid at the type level. `Range`
  uses non-`Option<NaiveDate>` because `from_cli` already enforces both ends
  present (D5).
- **D2**: Single-point validation in `from_cli` — error messages name the
  offending flag(s) and include usage hints (spec §4.3).
- **D3**: `--date` becomes `Option<String>` (not removed) — preserves
  backward-compat CLI shape; callers using `--date X` see no change.
- **D4**: `fetch_minute_klines` uses server-side range (`/data/bars`
  `start_date`/`end_date`) — verified by wiremock W1.
- **D5**: `from_cli(None, None, None)`, `(None, Some, None)`, `(None, None, Some)`
  all return `Err`.
- **D6**: `fetch_minute_share` uses client-side loop — OpenStock
  MINUTE_DATA server does not support range; iterating calendar days and
  reading `meta.trading_date` per response handles non-trading-day skips
  cleanly. Switchable to server-side later without signature change.

## Risks

- **R1**: `/data/bars` field names (`start_date`/`end_date`) unverified ->
  mitigated by wiremock W1 + live L1 test
- **R2**: Range performance — handler warns when result > 10k records
  (klines) or range > 10 days (share loop)
- **R3**: Date-mode wire body must be byte-identical to P0.13b-1/2 —
  verified by wiremock W2 + unchanged existing wiremock tests
- **R4**: `from_cli` UX — error messages name flags + usage (U5/U6/U7)
- **R5**: Share loop latency warning — handler prints warning when
  range spans > 10 days
- **R6**: OpenStock server may later add MINUTE_DATA range support —
  D6 design is switchable without signature break

## Invariants

- **INV-1A**: CLI `--date` and `(--start, --end)` mutex enforced by `from_cli`
- **INV-1B**: Range inclusive on both ends; `start > end` errors; semi-open
  ranges error
- **INV-2A**: `Date(d)` wire body byte-identical to P0.13b-1/2
- **INV-2B**: Result `Vec` flat, ordered by timestamp ascending
- **INV-2C**: `fetch_minute_share` loop reads `meta.trading_date` from server
  response (does not rely on client-requested date for record timestamp)
- **INV-3**: `MinuteBar`/`MinuteShare`/`MinutePeriod`/parsers unchanged —
  only fetcher signatures extended
