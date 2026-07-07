## ADDED Requirements

### Requirement: REQ-PERSIST-006 — import-minute-klines subcommand

The system SHALL expose a `data openstock import-minute-klines` subcommand that
accepts `--code <SYMBOL>`, `--period <PERIOD>`, `--adjust <ADJUST>`, `--start
<YYYY-MM-DD>`, `--end <YYYY-MM-DD>`, and `--apply` flags. The subcommand SHALL
parse `period` via `MinutePeriod::from_str` and `adjust` via
`AdjustType::from_str`, returning a `Config` error on invalid values. The
date range SHALL be parsed via `DateOrRange::from_cli(None, start, end)`; both
`--start` and `--end` are required (no `--date` single-day shortform). The
handler signature SHALL be
`import_openstock_minute_klines(settings, code, period, adjust, start, end, apply)`.

#### Scenario: Valid invocation enters the handler

- **WHEN** `quantix data openstock import-minute-klines --code sh600000 --period
  5m --adjust qfq --start 2026-01-01 --end 2026-01-31` is invoked
- **THEN** the handler parses `period` as `MinutePeriod::M5` and `adjust` as
  `AdjustType::Qfq`
- **AND** constructs a `DateOrRange::Range { start: 2026-01-01, end: 2026-01-31 }`
- **AND** dispatches to either the dry-run or apply branch based on
  `compute_apply(apply)` (REQ-PERSIST-008)

#### Scenario: Invalid period is rejected with a Config error

- **WHEN** `--period 7m` (unsupported value) is passed
- **THEN** the handler returns `Err(QuantixError::Config("--period: ..."))`
- **AND** no `OpenStockClient` is constructed

#### Scenario: Missing both --start and --end is rejected

- **WHEN** neither `--start` nor `--end` is provided
- **THEN** `DateOrRange::from_cli(None, None, None)` returns an error
- **AND** the handler propagates that error without constructing any client

### Requirement: REQ-PERSIST-007 — import-minute-share subcommand

The system SHALL expose a `data openstock import-minute-share` subcommand that
accepts `--code <SYMBOL>`, `--start <YYYY-MM-DD>`, `--end <YYYY-MM-DD>`, and
`--apply` flags. It SHALL NOT accept `--period` or `--adjust` (minute share
ticks have no concept of period or adjustment). The handler signature SHALL be
`import_openstock_minute_share(settings, code, start, end, apply)` and SHALL be
symmetric to REQ-PERSIST-006 except for the absence of period/adjust parsing.

#### Scenario: Valid invocation enters the handler

- **WHEN** `quantix data openstock import-minute-share --code sh600000 --start
  2026-01-01 --end 2026-01-31` is invoked
- **THEN** the handler constructs a `DateOrRange::Range` from `--start`/`--end`
- **AND** dispatches to either the dry-run or apply branch based on
  `compute_apply(apply)` (REQ-PERSIST-008)

#### Scenario: No period or adjust flags are exposed

- **WHEN** the `ImportMinuteShare` clap variant is parsed
- **THEN** the variant contains only `code`, `start`, `end`, `apply` fields
- **AND** any user-supplied `--period` or `--adjust` is rejected by clap as an
  unknown argument

### Requirement: REQ-PERSIST-008 — Double-key apply gate

The system SHALL gate ClickHouse writes on the conjunction of `--apply == true`
AND `env QUANTIX_OPENSTOCK_MINUTE_APPLY == "yes"` (verbatim). The boolean is
computed by the `compute_apply(apply: bool) -> bool` helper which reads the env
var internally. If either condition is false, the invocation SHALL execute the
dry-run branch and SHALL NOT write to ClickHouse. This double-key pattern
mirrors `ImportKlines`'s `QUANTIX_OPENSTOCK_KLINE_APPLY` semantics and protects
against misconfigured aliases or stale shell history triggering writes
(INV-CLI-1).

#### Scenario: Both keys set enters apply branch

- **WHEN** `--apply` is passed AND `QUANTIX_OPENSTOCK_MINUTE_APPLY=yes` is set
  in the process environment
- **THEN** `compute_apply(true)` returns `true`
- **AND** the handler executes the apply branch (constructs ClickHouse sink,
  calls `stream_minute_*_to_clickhouse`)

#### Scenario: --apply alone is not sufficient

- **WHEN** `--apply` is passed AND `QUANTIX_OPENSTOCK_MINUTE_APPLY` is unset or
  set to any value other than exactly `"yes"`
- **THEN** `compute_apply(true)` returns `false`
- **AND** the handler executes the dry-run branch
- **AND** the stdout summary includes a hint: `set QUANTIX_OPENSTOCK_MINUTE_APPLY=yes
  to actually insert`

#### Scenario: Env var alone is not sufficient

- **WHEN** `QUANTIX_OPENSTOCK_MINUTE_APPLY=yes` is set BUT `--apply` is omitted
  (defaults to `false`)
- **THEN** `compute_apply(false)` returns `false`
- **AND** the handler executes the dry-run branch
- **AND** no hint about the env var is printed (because `--apply` itself was
  missing, not the env var)

### Requirement: REQ-PERSIST-009 — Dry-run never constructs ClickHouse

When `compute_apply(apply)` returns `false`, the handler SHALL NOT construct a
`ClickHouseClient`, SHALL NOT instantiate any `ClickHouseMinuteKlineSink` or
`ClickHouseMinuteShareSink`, and SHALL NOT call any `stream_minute_*_to_clickhouse`
consumer. The dry-run branch's only external dependency SHALL be
`OpenStockClient` (used to consume the P0.13d stream and count rows). This
allows operators to validate OpenStock connectivity and range sizing without
needing ClickHouse credentials (INV-CLI-2).

#### Scenario: Dry-run consumes the stream and prints a count

- **WHEN** the handler runs with `will_apply == false`
- **THEN** the handler constructs `OpenStockClient::from_settings(settings)`
- **AND** calls `client.fetch_minute_klines_stream(...)` (or
  `fetch_minute_share_stream(...)`)
- **AND** iterates the stream, counting batches and total rows
- **AND** prints `dry_run: true, applied: false` plus `would_insert_total: <N>`
  to stdout
- **AND** no `ClickHouseClient` construction is attempted

#### Scenario: No ClickHouse credentials required for dry-run

- **WHEN** the process environment has no ClickHouse credentials configured
- **AND** the handler runs in dry-run mode
- **THEN** the handler completes successfully
- **AND** returns `Ok(())` without any ClickHouse-related error

### Requirement: REQ-PERSIST-010 — Stream API only (no batch API)

Both handlers SHALL consume the P0.13d streaming API
(`fetch_minute_klines_stream` / `fetch_minute_share_stream`) exclusively. The
batch APIs (`fetch_minute_klines` / `fetch_minute_share`) SHALL NOT be called
from either handler. The apply branch SHALL pipe the stream directly into the
P0.14 consumer (`stream_minute_klines_to_clickhouse` /
`stream_minute_shares_to_clickhouse`) without collecting to an intermediate
`Vec`. This invariant unifies the codepath and leverages P0.13d's weekly
chunking + per-batch progress for partial-failure telemetry (INV-CLI-3,
INV-FLOW-1).

#### Scenario: Apply branch pipes stream into ClickHouse consumer

- **WHEN** the handler runs with `will_apply == true`
- **THEN** the handler calls
  `stream_minute_klines_to_clickhouse(&client, &sink, &code, period_enum,
   adjust_enum, start_date, end_date)` (or the share-equivalent)
- **AND** the P0.13d stream is consumed lazily by the P0.14 consumer
- **AND** no intermediate `Vec<MinuteBar>` or `Vec<MinuteShare>` is materialized
  in handler memory

#### Scenario: Batch API is unreachable from the new handlers

- **WHEN** the source of `import_openstock_minute_klines` or
  `import_openstock_minute_share` is inspected
- **THEN** neither function contains a call to `fetch_minute_klines` or
  `fetch_minute_share` (the batch APIs)
- **AND** only their `_stream` variants appear
