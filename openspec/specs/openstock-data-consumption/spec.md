# openstock-data-consumption Specification

## Purpose
TBD - created by archiving change openstock-data-consumption-p0-8. Update Purpose after archive.
## Requirements
### Requirement: OpenStock Consumption Planning Boundary

The system SHALL define OpenStock data consumption as a broker-independent market-data line before implementing provider code.

#### Scenario: Planning does not require broker runtime

- **WHEN** the OpenStock data-consumption plan is created
- **THEN** it SHALL NOT require qmt_live, miniQMT, broker credentials, or Windows Bridge runtime availability.

#### Scenario: Planning preserves existing source boundaries

- **WHEN** OpenStock slices are planned
- **THEN** they SHALL preserve existing `tdx_api`, `bridge_tdx`, `eastmoney`, and miniQMT market-manifest behavior unless a later slice explicitly authorizes a change.

### Requirement: Fixture-Owned Development

OpenStock parser and normalization work SHALL start from committed fixtures or local artifacts that are safe for CI.

#### Scenario: Tests avoid live network calls

- **WHEN** CI runs OpenStock-related tests
- **THEN** default tests SHALL NOT call live OpenStock endpoints.

#### Scenario: Fixture validation is read-only

- **WHEN** a local OpenStock fixture or artifact is validated
- **THEN** validation SHALL NOT write ClickHouse, broker state, runtime storage, or external systems.

### Requirement: Read-Only Before Persistence

The system SHALL prove read-only parsing, normalization, and downstream consumption before adding any persistence path.

#### Scenario: Persistence requires separate approval

- **WHEN** a slice proposes ClickHouse writes or other persistent storage changes
- **THEN** it SHALL include schema, deduplication, rollback, dry-run, and GitNexus impact evidence before implementation approval.

### Requirement: Downstream Quant Loop Alignment

OpenStock consumption SHALL be sequenced toward a local quant loop that can run without real broker execution.

#### Scenario: First runnable loop is local

- **WHEN** OpenStock data is connected to downstream processing
- **THEN** the first runnable loop SHOULD use indicators, backtest, or paper/mock execution without qmt_live submit/query/cancel behavior.

### Requirement: Uniform Envelope Parsing

The system SHALL parse OpenStock `/data/fetch` responses through a uniform envelope type that captures the canonical `data` array plus all metadata fields (`source`, `data_category`, `request_id`, `route_decision_id`, `quality_flags`, `cache_state`, `circuit_state`, `latency_ms`, `received_at`).

#### Scenario: Envelope metadata fields are optional

- **WHEN** a `/data/fetch` response omits any metadata field other than `data`
- **THEN** the parser SHALL accept the payload and treat the missing field as `None` / empty.

#### Scenario: Envelope always carries a data array

- **WHEN** a `/data/fetch` response is parsed for `STOCK_CODES`, `ALL_STOCKS`, `TRADE_DATES`, `WORKDAYS`, or `INDEX_KLINES`
- **THEN** the envelope SHALL expose `data` as a JSON array; parsers SHALL reject non-array shapes.

### Requirement: Uniform Error Envelope Mapping

The system SHALL map non-2xx `/data/fetch` responses through a uniform error envelope type (`code`, `message`, `request_id`, `details`) into the project's canonical error type without losing the upstream error code.

#### Scenario: HTTP error response preserves upstream error code

- **WHEN** `/data/fetch` returns non-2xx
- **THEN** the client SHALL deserialize the error envelope and surface `{code, message}` in the resulting `QuantixError` text.

### Requirement: Consumer-Side artifact_hash

The system SHALL compute `artifact_hash` as SHA-256 of the raw response body, on the consumer side, using the canonical `openstock_shadow::artifact_hash` implementation.

#### Scenario: Single source of truth for hashing

- **WHEN** any code path needs an `artifact_hash` for an OpenStock payload
- **THEN** it SHALL call `crate::sources::openstock_shadow::artifact_hash` (re-exported in `openstock_envelope`); no second SHA-256 implementation SHALL exist.

### Requirement: Fixture-Driven Parser Tests

Parsers for `STOCK_CODES`, `ALL_STOCKS`, `TRADE_DATES`, `WORKDAYS`, and `INDEX_KLINES` SHALL be covered by fixture-driven integration tests using committed JSON files.

#### Scenario: No live network in parser tests

- **WHEN** the parser test suite runs in CI
- **THEN** no test SHALL make a live HTTP call to an OpenStock instance.

#### Scenario: Empty-data fixtures exercise the EmptyRecords error path

- **WHEN** a fixture with `data: []` is parsed
- **THEN** the parser SHALL return `EmptyRecords` (or equivalent) without panic.

### Requirement: Read-Only CLI Surface For Category Validation

The system SHALL expose per-category read-only CLI subcommands that parse a captured payload file (or stdin) and print a dry-run report.

#### Scenario: ValidateCodes reports source and sample

- **WHEN** `quantix data openstock validate-codes --payload <path>` is invoked with a valid STOCK_CODES or ALL_STOCKS payload
- **THEN** the CLI SHALL print `{ source, count, first sample, last sample, fields-seen }` without writing to any database or making any network call.

#### Scenario: ValidateCalendar and ValidateIndex follow the same shape

- **WHEN** the calendar or index validation subcommands are invoked
- **THEN** they SHALL produce the same report shape (source, count, sample) and SHALL NOT write to any database or make any network call.

### Requirement: Additive-Only Edits To Existing Modules

The system SHALL NOT modify the daily-kline fixture parser, the live shadow validator, the shadow persistence write path, the `Kline` model, `BacktestEngine`, or `ExecutionAdapter` in this slice.

#### Scenario: Daily-kline paths stay untouched

- **WHEN** the slice is merged
- **THEN** `parse_daily_kline_json`, `validate_live_shadow_payload`, `openstock_shadow::write_shadow_klines`, `Kline`, `BacktestEngine`, and `ExecutionAdapter` SHALL be byte-identical to `master` (except the visibility-only widen on `normalize_symbol`/`parse_live_time` to `pub(crate)`).

#### Scenario: Visibility widen does not change behavior

- **WHEN** `normalize_symbol` and `parse_live_time` become `pub(crate)`
- **THEN** their signatures, bodies, and call sites SHALL remain unchanged; only the visibility modifier changes.

### Requirement: Live HTTP Wiring For Read-Only P0 Categories

The system SHALL provide CLI subcommands (`openstock fetch-codes`, `openstock fetch-calendar --year <N>`, `openstock fetch-index --symbol <code> [--start <date>] [--end <date>]`) that perform live `POST /data/fetch` calls against an OpenStock runtime and print a summary block (records count, first/last sample, source, `artifact_hash`, `latency_ms`).

#### Scenario: FetchCodes calls STOCK_CODES

- **WHEN** the operator invokes `openstock fetch-codes` with `OPENSTOCK_BASE_URL` and `OPENSTOCK_API_KEY` set
- **THEN** the CLI SHALL build an `OpenStockClient` from env, call `fetch_stock_codes()`, parse the envelope, and print a non-empty records summary.

#### Scenario: FetchCalendar calls TRADE_DATES with year

- **WHEN** the operator invokes `openstock fetch-calendar --year 2026`
- **THEN** the CLI SHALL call `fetch_trade_dates(2026)` and print the resulting trade-date records.

#### Scenario: FetchIndex calls INDEX_KLINES with optional date range

- **WHEN** the operator invokes `openstock fetch-index --symbol sh000001`
- **THEN** the CLI SHALL call `fetch_index_klines("sh000001", None, None)` and print the resulting index kline records.

### Requirement: HTTP Status Check Before Body Parse

The `OpenStockClient::fetch` method SHALL branch on `response.status().is_success()` before parsing the response body. On non-2xx, the client SHALL attempt to deserialize the uniform error envelope and surface `to_summary()`; if the body itself is not a valid error envelope, the client SHALL surface the HTTP status code and the first 200 characters of the body.

#### Scenario: Non-2xx with valid error envelope

- **WHEN** `/data/fetch` returns HTTP 503 with a body `{"code":"provider_unavailable","message":"...","request_id":"req-1"}`
- **THEN** the resulting `QuantixError::Other` SHALL contain the upstream `code`, `message`, and `request_id`.

#### Scenario: Non-2xx with non-JSON body (e.g. proxy error)

- **WHEN** `/data/fetch` returns HTTP 502 with an HTML body from an intermediate proxy
- **THEN** the resulting `QuantixError::Other` SHALL contain the literal `HTTP 502` string and the first 200 characters of the body so the operator can diagnose the proxy failure.

### Requirement: Environment-Driven Client Configuration

`OpenStockClient::new` SHALL fall back to the `OPENSTOCK_BASE_URL` environment variable when `cfg.base_url` is empty, mirroring the existing `OPENSTOCK_API_KEY` fallback for `cfg.api_key`. A `from_env()` convenience constructor SHALL build a client entirely from these two env vars with default timeout.

#### Scenario: Both env vars set, default config used

- **WHEN** the operator sets `OPENSTOCK_BASE_URL=http://...:8040` and `OPENSTOCK_API_KEY=<key>` and calls `OpenStockClient::from_env()`
- **THEN** the client SHALL construct successfully with no explicit config.

#### Scenario: Missing base_url env

- **WHEN** `OPENSTOCK_BASE_URL` is unset and `cfg.base_url` is empty
- **THEN** `OpenStockClient::new` SHALL return `QuantixError::Config` with a message naming the missing env var.

### Requirement: Live Network Tests Are Gated

Live HTTP integration tests for the three P0 categories SHALL be marked `#[ignore]` and SHALL additionally early-return when `QUANTIX_OPENSTOCK_LIVE != "1"`. CI runs of `cargo test --workspace` SHALL execute zero live network calls.

#### Scenario: CI skips live tests

- **WHEN** `cargo test --workspace` runs without `--ignored` and without `QUANTIX_OPENSTOCK_LIVE=1`
- **THEN** no live HTTP request is issued; the live tests are reported as ignored.

#### Scenario: Manual live smoke

- **WHEN** the operator runs `QUANTIX_OPENSTOCK_LIVE=1 OPENSTOCK_BASE_URL=... OPENSTOCK_API_KEY=... cargo test --test openstock_live_codes -- --ignored`
- **THEN** the live test SHALL issue a real HTTP request and assert non-empty records + 64-character `artifact_hash`.

### Requirement: Latency Surface In Response View

`OpenStockResponse<T>` SHALL expose `latency_ms: Option<u64>` populated from the envelope's `latency_ms` field, so CLI handlers and downstream consumers can report provider-side latency without re-parsing the raw body.

#### Scenario: Provider reports latency

- **WHEN** `/data/fetch` returns `"latency_ms": 42` in the success envelope
- **THEN** `OpenStockResponse::latency_ms` SHALL equal `Some(42)`.

#### Scenario: Provider omits latency

- **WHEN** `/data/fetch` omits the `latency_ms` field
- **THEN** `OpenStockResponse::latency_ms` SHALL equal `None`; handlers SHALL render it as "(not reported)".

### Requirement: Multi-period K-line Fetch

The system SHALL support fetching K-lines from OpenStock `/data/bars`
with `period ∈ {day, week, month}` and `adjust_type ∈ {None, QFQ, HFQ}`
through a unified `OpenStockClient::fetch_klines` API.

#### Scenario: day period without adjust

- **WHEN** the caller invokes `fetch_klines("600000", BarPeriod::Day, AdjustType::None, None, None)`
- **THEN** the request body to `/data/bars` contains `{"symbol":"600000","period":"day"}` with NO `adjust` field
- **AND** each returned `Kline` has `adjust_type = None`

#### Scenario: week period with qfq

- **WHEN** the caller invokes `fetch_klines("600000", BarPeriod::Week, AdjustType::QFQ, None, None)`
- **THEN** the request body contains `"period":"week"` and `"adjust":"qfq"`
- **AND** each returned `Kline` has `adjust_type = QFQ` (request-driven — runtime does not echo)

#### Scenario: month period with hfq

- **WHEN** the caller invokes `fetch_klines("600000", BarPeriod::Month, AdjustType::HFQ, None, None)`
- **THEN** the request body contains `"period":"month"` and `"adjust":"hfq"`
- **AND** each returned `Kline` has `adjust_type = HFQ`

### Requirement: Strict Period Parsing

The system SHALL reject period aliases (`daily`/`weekly`/`monthly`,
any case) and any minute-level value via `BarPeriod::from_str`,
returning `QuantixError::Config`. Only `day|week|month` (case-insensitive)
are accepted.

#### Scenario: invalid period surfaces config error

- **WHEN** the CLI parses `--period daily`
- **THEN** the handler returns `QuantixError::Config` mentioning "unsupported BarPeriod"
- **AND** no HTTP request is made

### Requirement: CLI Multi-period Subcommand

The system SHALL expose `data openstock fetch-klines` with `--symbol`
(required), `--period` (default `day`), `--adjust` (default `none`),
`--start`, `--end` (both optional).

#### Scenario: default invocation

- **WHEN** the user runs `data openstock fetch-klines --symbol 600000`
- **THEN** the system fetches day-period unadjusted bars for symbol 600000

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

#### Scenario: Strict period whitelist

- **WHEN** `MinutePeriod::from_str("1min")` is called
- **THEN** the result SHALL be `Err(QuantixError::Config)` with a message
  listing `1m|5m|15m|30m|60m` as the only accepted tokens

#### Scenario: Adjust field omission on None

- **WHEN** `fetch_minute_klines(code, period, date, AdjustType::None)` is called
- **THEN** the request body SHALL NOT contain the `adjust` key

#### Scenario: 4xx propagation without retry

- **WHEN** `/data/bars` returns HTTP 400
- **THEN** the method SHALL return `Err(QuantixError::Other)` containing
  "/data/bars returned 400" on the first attempt, without retrying

#### Scenario: Minute-precision timestamp preserved

- **WHEN** the wire response contains `"time": "2026-07-02T09:31:00+08:00"`
- **THEN** the returned `MinuteBar.timestamp` SHALL equal the parsed
  `NaiveDateTime` for `2026-07-02T09:31:00`

### Requirement: fetch-minute-klines CLI subcommand

The system SHALL provide a `data openstock fetch-minute-klines` subcommand
accepting `--symbol`, `--period` (default `1m`), `--date` (required,
`YYYY-MM-DD`), and `--adjust` (default `none`).

#### Scenario: Bad --period surfaces as Config error

- **WHEN** the user runs `data openstock fetch-minute-klines --symbol sh600000 --period 1min --date 2026-07-02`
- **THEN** the CLI SHALL exit with a `QuantixError::Config` whose message
  contains "--period:" and "expected one of 1m|5m|15m|30m|60m"

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

#### Scenario: Successful fetch with complete records

- **WHEN** `fetch_minute_share` is called with a valid code and trading day
- **THEN** the system issues `POST /data/fetch` with body
  `{data_category: "MINUTE_DATA", params: {code, date}}`
- **AND** returns `Vec<MinuteShare>` containing all complete records

#### Scenario: Records with missing fields are skipped

- **WHEN** the envelope contains records where one or more required fields
  (price, volume, amount, avg_price) are missing
- **THEN** the system emits a `tracing::warn!` for each skipped record
- **AND** returns `Vec<MinuteShare>` containing only the complete records
- **AND** does NOT fail the whole operation

#### Scenario: 4xx HTTP response

- **WHEN** the OpenStock runtime returns HTTP 4xx
- **THEN** the system fails fast (no retry) and propagates the error

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

### Requirement: MinuteShare Model

The system SHALL provide a `MinuteShare` struct with fields:
`code: String`, `timestamp: NaiveDateTime`, `price: Option<Decimal>`,
`volume: Option<i64>`, `amount: Option<Decimal>`,
`avg_price: Option<Decimal>`.

#### Scenario: Serialization round-trip

- **WHEN** a `MinuteShare` is serialized and deserialized
- **THEN** all fields are preserved

#### Scenario: Missing optional fields deserialize as None

- **WHEN** a JSON record omits one or more optional fields
- **THEN** serde deserialization succeeds with the missing fields as `None`

### Requirement: CLI Subcommand fetch-minute-share

The system SHALL provide a `data openstock fetch-minute-share` CLI
subcommand accepting `--symbol` and `--date` (YYYY-MM-DD) arguments.

#### Scenario: CLI smoke

- **WHEN** invoked as `data openstock fetch-minute-share --symbol sh600000 --date 2026-06-30`
- **THEN** the system fetches and prints the time-share ticks summary

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

