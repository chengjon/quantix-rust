# openstock-data-consumption Specification Delta — P0.9

## ADDED Requirements

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
