# OpenStock Data Consumption Spec Delta — P0.13b-1

## ADDED Requirements

### Requirement: Minute-level K-line candles via /data/bars

The system SHALL provide a `fetch_minute_klines(code, period, date, adjust)`
method on `OpenStockClient` that fetches OHLCV candles at minute granularity
(1m|5m|15m|30m|60m) via POST to `/data/bars` with JSON body
`{symbol, period, date, adjust?}`.

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
