# OpenStock Data Consumption Spec Delta — P0.13d

## ADDED Requirements

### Requirement: REQ-STREAM-001 — Streaming API for Minute Klines

The system SHALL provide `OpenStockClient::fetch_minute_klines_stream` returning
`impl Stream<Item = Result<Vec<MinuteBar>, QuantixError>>`. The stream SHALL
slice the requested range into fixed 7-day chunks (D2) and yield one `Vec` per
chunk in chronological order. The Date variant yields a single batch. The first
batch error terminates the stream (D4).

#### Scenario: Weekly chunking of a multi-week range

- **WHEN** `fetch_minute_klines_stream(code, period, DateOrRange::Range { start, end }, adjust)`
  is called with `end - start >= 14 days`
- **THEN** the system emits one stream batch per 7-day sub-range
- **AND** each batch issues an HTTP request whose body carries the corresponding
  `start_date`/`end_date` sub-range and no `date` field
- **AND** concatenating all batches yields a flat `Vec<MinuteBar>` identical to
  `fetch_minute_klines` over the same range (INV-1A)

#### Scenario: Single-day range compresses to a single batch

- **WHEN** the requested range is a single calendar day (`start == end`) or the
  `Date(d)` variant is used
- **THEN** the stream yields exactly one batch
- **AND** that batch's request body carries `params.date = d` (no `start_date`/
  `end_date`)

#### Scenario: Error terminates the stream

- **WHEN** a batch HTTP request fails
- **THEN** the stream yields `Err(QuantixError)` for that batch
- **AND** subsequent `next().await` calls return `None`
- **AND** previously yielded batches remain valid consumed output

### Requirement: REQ-STREAM-002 — Streaming API for Minute Share

The system SHALL provide `OpenStockClient::fetch_minute_share_stream` returning
`impl Stream<Item = Result<Vec<MinuteShare>, QuantixError>>`. The stream SHALL
yield one `Vec<MinuteShare>` per calendar day in the requested range,
reusing `fetch_minute_share_single`. Non-trading days SHALL yield `vec![]`
(D3). The first error terminates the stream (D4).

#### Scenario: Per-day batch over a multi-day range

- **WHEN** `fetch_minute_share_stream(code, DateOrRange::Range { start, end })`
  is called
- **THEN** the stream yields one batch per calendar day in `[start, end]`
- **AND** the total batch count equals the calendar-day span of the range
- **AND** each batch issues a single HTTP request whose body carries
  `params.date = <that calendar day>`

#### Scenario: Non-trading day yields empty Vec

- **WHEN** a calendar day in the range is a non-trading day (server returns
  empty records)
- **THEN** the stream yields `Ok(vec![])` for that day
- **AND** does not skip the day (preserves day-level signal for completeness
  checks)

#### Scenario: Error terminates the stream

- **WHEN** a per-day HTTP request fails
- **THEN** the stream yields `Err(QuantixError)` for that day
- **AND** subsequent `next().await` calls return `None`

### Requirement: REQ-STREAM-003 — CLI `--stream` Flag

The CLI subcommands `fetch-minute-klines` and `fetch-minute-share` SHALL accept
a `--stream` boolean flag (default `false`). When `--stream` is absent, behavior
MUST be byte-identical to P0.13c. When `--stream` is set, the handler SHALL
invoke the streaming API and emit per-batch progress to stderr.

#### Scenario: Default behavior unchanged

- **WHEN** the user runs `fetch-minute-klines` (or `fetch-minute-share`) without
  `--stream`
- **THEN** the handler takes the existing P0.13c batch path
- **AND** produces byte-identical stdout/stderr output to P0.13c
- **AND** does not import or call the streaming API

#### Scenario: Streaming path emits per-batch progress

- **WHEN** the user passes `--stream`
- **THEN** the handler invokes `fetch_minute_klines_stream` (or
  `fetch_minute_share_stream`)
- **AND** prints one progress line per batch to stderr (e.g. batch index, record
  count)
- **AND** exits 0 only after the stream has been fully consumed

#### Scenario: Flag mutually consistent with --date / --start / --end

- **WHEN** the user passes `--stream` together with `--date X` or
  `--start X --end Y`
- **THEN** the existing `DateOrRange::from_cli` validation still applies
- **AND** `--stream` is orthogonal to date-flag validation (no new mutex)

### Requirement: REQ-STREAM-004 — Batch API Backward Compatibility

The existing batch APIs (`fetch_minute_klines`, `fetch_minute_share`) SHALL
remain unchanged in signature, wire shape, and behavior. All P0.13a/b/c
wiremock tests, live tests, and unit tests SHALL pass zero-modified.

#### Scenario: Signature unchanged

- **WHEN** callers invoke `fetch_minute_klines(code, period, DateOrRange, adjust)`
  or `fetch_minute_share(code, DateOrRange)`
- **THEN** the method signatures match P0.13c exactly
- **AND** existing call sites compile without modification

#### Scenario: Wire shape unchanged

- **WHEN** the batch API issues an HTTP request (Date or Range variant)
- **THEN** the request body is byte-identical to P0.13c
- **AND** the response parsing path is unchanged

#### Scenario: P0.13a/b/c test suite passes unmodified

- **WHEN** `cargo test --workspace` runs the existing P0.13a/b/c wiremock +
  unit + live tests
- **THEN** all of them pass with zero source modifications
- **AND** INV-4A (batch API surface frozen) holds
