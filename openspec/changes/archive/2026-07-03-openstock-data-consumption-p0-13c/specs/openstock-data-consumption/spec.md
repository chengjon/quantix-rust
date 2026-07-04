# OpenStock Data Consumption Spec Delta — P0.13c

## MODIFIED Requirements

### Requirement: Minute-Level K-Line Fetcher

The system SHALL provide a `fetch_minute_klines` method that accepts a
`DateOrRange` parameter supporting either a single date or an inclusive
`[start, end]` range. The Date variant preserves byte-identical wire body
to P0.13b-1; the Range variant uses server-side range fields.

#### Scenario: Single-day via Date variant

- **WHEN** `fetch_minute_klines(code, period, DateOrRange::Date(d), adjust)`
  is called
- **THEN** the system sends `params.date = d` to `/data/bars`
- **AND** the request body contains no `start_date` or `end_date` field
- **AND** returns `Vec<MinuteBar>` (backward-compatible with P0.13b-1)

#### Scenario: Multi-day via Range variant

- **WHEN** `fetch_minute_klines(code, period, DateOrRange::Range { start, end }, adjust)`
  is called
- **THEN** the system sends `params.start_date` and `params.end_date` to
  `/data/bars`
- **AND** the request body contains no `date` field
- **AND** returns a flat `Vec<MinuteBar>` ordered by timestamp ascending

### Requirement: Minute-Level Time-Share Fetcher

The system SHALL provide a `fetch_minute_share` method that accepts a
`DateOrRange` parameter. Because the OpenStock MINUTE_DATA server does not
support range queries, the Range variant issues N single-day requests via
client-side loop.

#### Scenario: Single-day via Date variant

- **WHEN** `fetch_minute_share(code, DateOrRange::Date(d))` is called
- **THEN** the system sends `params.date = d` to `/data/fetch MINUTE_DATA`
- **AND** parses `meta.trading_date` for each response envelope item to
  derive per-record timestamps

#### Scenario: Multi-day via Range variant (client-side loop)

- **WHEN** `fetch_minute_share(code, DateOrRange::Range { start, end })` is
  called
- **THEN** the system iterates `iter_dates_inclusive(start, end)` issuing
  one single-day request per calendar day
- **AND** aggregates results into a flat `Vec<MinuteShare>` ordered by
  `(date, time_minutes)` ascending
- **AND** skips days where the server returns empty records (non-trading
  days) without failing the operation

### Requirement: CLI Flag Validation

The CLI SHALL validate `--date` vs `--start`/`--end` mutex via
`DateOrRange::from_cli`. Error messages SHALL name the offending flag(s)
and include usage hints.

#### Scenario: Both --date and --start provided

- **WHEN** the CLI receives `--date X --start Y` (or any overlap)
- **THEN** it returns an error naming both `--date` and `--start`/`--end`
- **AND** does not issue any HTTP request

#### Scenario: Semi-open range

- **WHEN** the CLI receives `--start X` without `--end` (or vice versa)
- **THEN** it returns an error requiring both ends to be provided together
- **AND** does not issue any HTTP request

#### Scenario: Start after end

- **WHEN** the CLI receives `--start 2026-06-30 --end 2026-06-01`
- **THEN** it returns an error stating `--start` must be on or before `--end`
- **AND** does not issue any HTTP request

#### Scenario: No date arguments

- **WHEN** the CLI receives neither `--date` nor `--start`/`--end`
- **THEN** it returns an error requiring at least one form
- **AND** does not issue any HTTP request
