# OpenStock Data Consumption

## ADDED Requirements

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
