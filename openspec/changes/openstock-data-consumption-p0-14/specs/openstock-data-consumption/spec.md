# OpenStock Data Consumption Spec Delta — P0.14

## ADDED Requirements

### Requirement: REQ-PERSIST-001 — ClickHouse Minute-Level Kline Persistence

The system SHALL provide
`stream_minute_klines_to_clickhouse<S: MinuteSink<MinuteKlineCH>>` that consumes
`OpenStockClient::fetch_minute_klines_stream` and inserts each yielded batch into
the ClickHouse `minute_klines` table. The function SHALL return
`Result<StreamStats, QuantixError>` where `StreamStats` records `batches`,
`input_records`, and `inserted_records` counts. The first stream error or sink
error SHALL short-circuit the function and propagate as
`Err(QuantixError)` (INV-3A/3C).

#### Scenario: Stream klines round-trip

- **WHEN** `stream_minute_klines_to_clickhouse` is called for code `sh600000`,
  period `1m`, date range `[2026-06-23, 2026-06-24]`, adjust `none`
- **THEN** the function returns
  `StreamStats { batches >= 1, inserted_records > 0 }`
- **AND** querying `minute_klines WHERE code = 'sh600000'` returns rows whose
  count equals `stats.inserted_records`

#### Scenario: Stream consumer short-circuits on first sink error

- **WHEN** a `MinuteSink<MinuteKlineCH>` returns `Err` on the second batch
- **THEN** the consumer returns
  `Err(QuantixError::DatabaseQuery(...))` immediately (INV-3A)
- **AND** does not consume further batches from the stream (INV-3C)

### Requirement: REQ-PERSIST-002 — ClickHouse Minute-Level Share Persistence

The system SHALL provide
`stream_minute_shares_to_clickhouse<S: MinuteSink<MinuteShareCH>>` that consumes
`OpenStockClient::fetch_minute_share_stream` and inserts each yielded batch into
the ClickHouse `minute_shares` table. Semantics mirror REQ-PERSIST-001
(short-circuit on first error, return `StreamStats`).

#### Scenario: Stream shares round-trip

- **WHEN** `stream_minute_shares_to_clickhouse` is called for code `sh600000`
  and date range `[2026-06-23, 2026-06-24]`
- **THEN** the function returns `StreamStats { batches >= 1, inserted_records > 0 }`
- **AND** querying `minute_shares WHERE code = 'sh600000'` returns rows whose
  count equals `stats.inserted_records`

### Requirement: REQ-PERSIST-003 — Type Alignment With kline_data

The new `minute_klines` and `minute_shares` tables SHALL use the same
column-type conventions as the existing `kline_data` table:

- `timestamp DateTime` (no timezone annotation)
- `period String`, `adjust String` (no Enum8)
- OHLCV / amount columns `Float64`

The Rust row structs `MinuteKlineCH` / `MinuteShareCH` SHALL derive
`clickhouse::Row` and use `DateTime<Utc>` for `timestamp`, matching
`KlineDataCH`.

#### Scenario: Reverse query returns expected literal values

- **WHEN** rows written via `stream_minute_klines_to_clickhouse` are queried
  back via `SELECT * FROM minute_klines`
- **THEN** the `period` column values are exactly one of
  `"1m"`, `"5m"`, `"15m"`, `"30m"`, `"60m"`
- **AND** the `adjust` column values are exactly one of
  `"none"`, `"qfq"`, `"hfq"`

### Requirement: REQ-PERSIST-004 — DDL Registration in init_database

The `ClickHouseClient::init_database()` function SHALL call
`create_minute_klines_table()` and `create_minute_shares_table()` before
returning success. Both tables SHALL be created with `ENGINE = MergeTree()` and
`ON CLUSTER '{cluster}'` clause, consistent with the existing `kline_data`
table DDL (INV-1A/1B, INV-5A/5B).

#### Scenario: Tables created after init_database succeeds

- **WHEN** `init_database()` returns `Ok(())`
- **THEN** both `minute_klines` and `minute_shares` tables exist
- **AND** both tables report `ENGINE = MergeTree()`
- **AND** the DDL was issued with `ON CLUSTER 'single_cluster'` after runtime
  replacement

### Requirement: REQ-PERSIST-005 — Sink Trait Visibility (Internal Only)

The sink trait and concrete sinks SHALL be declared `pub(crate)`. Specifically,
`MinuteSink<T>`, `ClickHouseMinuteKlineSink`, and `ClickHouseMinuteShareSink`
MUST all have visibility `pub(crate)`. The public stream consumer functions
MUST be generic over `<S: MinuteSink<...>>` so that, because the trait itself
is `pub(crate)`, external crates cannot construct a satisfying type —
effectively making the consumers internal-only (INV-4A/4B/4C/4D).

#### Scenario: External crate cannot construct a satisfying sink type

- **WHEN** an external crate attempts to call
  `stream_minute_klines_to_clickhouse(...)`
- **THEN** compilation fails because both the trait and the concrete sinks are
  not visible outside the crate
