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

