# OpenStock Data Consumption Spec Delta — P0.13b-2

## ADDED Requirements

### Requirement: Consume MINUTE_DATA Category

The system SHALL provide a `fetch_minute_share(code, date)` client method
that consumes the OpenStock `MINUTE_DATA` category via the `/data/fetch`
envelope path with retry and circuit breaker.

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
