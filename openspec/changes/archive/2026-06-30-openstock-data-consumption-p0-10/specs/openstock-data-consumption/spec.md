# openstock-data-consumption Specification Delta — P0.10

## ADDED Requirements

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
