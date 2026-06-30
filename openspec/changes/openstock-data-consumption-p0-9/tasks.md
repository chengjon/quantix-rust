# OpenStock Data Consumption P0.9 Tasks

## 0. Baseline And Governance

- [ ] 0.1 Confirm work starts from clean `master` after P0.8 archive.
- [ ] 0.2 Run GitNexus `overview` and `detect_changes` before edits.
- [ ] 0.3 Create FUNCTION_TREE node `P0.8i` under parent `P0.8`.
- [ ] 0.4 Transition `P0.8i` `planned → in_progress` via `ft:transition`, gate via `ft:gate`.
- [ ] 0.5 Run `openspec validate openstock-data-consumption-p0-9 --strict`.
- [ ] 0.6 Run `openspec validate --all --strict`.

## 1. Envelope Module (`openstock_envelope.rs`)

- [ ] 1.1 Add `OpenStockEnvelope<T>` raw serde struct (`data: Vec<T>` + 9 metadata fields all `Option`/`#[serde(default)]`).
- [ ] 1.2 Add `OpenStockErrorEnvelope { code, message, request_id, details }`.
- [ ] 1.3 Add `pub use crate::sources::openstock_shadow::artifact_hash;` re-export (single source of truth — no second SHA-256 implementation).
- [ ] 1.4 Add unit tests for envelope serde (data + metadata roundtrip, missing-optionals default).

## 2. Client Skeleton (`openstock_client.rs`)

- [ ] 2.1 Add `OpenStockClientConfig { base_url, api_key, timeout }`.
- [ ] 2.2 Add `OpenStockClient { base_url: Url, api_key: HeaderValue, http: reqwest::Client }`.
- [ ] 2.3 Add `OpenStockClient::new(cfg) -> Result<Self>` — reads `OPENSTOCK_API_KEY` env if `api_key` empty, builds `X-API-Key` header once.
- [ ] 2.4 Add `async fn fetch<T: DeserializeOwned>(&self, cat: &str, params: Value) -> Result<OpenStockResponse<T>>` — POST `/data/fetch` with body `{"data_category": cat, "params": params}`, deserialize envelope on 2xx, error envelope on non-2xx → `QuantixError::Other`.
- [ ] 2.5 Add `OpenStockResponse<T> { records: Vec<T>, source: String, artifact_hash: String, received_at: Option<String> }` + `from_envelope(env, raw_body) -> Self`.
- [ ] 2.6 Add convenience wrappers: `fetch_stock_codes()`, `fetch_trade_dates(year)`, `fetch_index_klines(code, start, end)`.
- [ ] 2.7 Unit tests for `OpenStockResponse::from_envelope` composition + error envelope bridge (no live HTTP).

## 3. Codes Parser (`openstock_codes.rs`)

- [ ] 3.1 Add `StockCodeRecord { code, name, #[serde(flatten)] extra }` and `StockListRecord { code, name, market, listing_date, #[serde(flatten)] extra }`.
- [ ] 3.2 Add `StockCode { code, name }` and `StockListEntry { code, name, market, listing_date }`.
- [ ] 3.3 Add `parse_stock_codes(env: OpenStockEnvelope<StockCodeRecord>) -> Result<Vec<StockCode>, StockCodeParseError>`.
- [ ] 3.4 Add `parse_all_stocks(env: OpenStockEnvelope<StockListRecord>) -> Result<Vec<StockListEntry>, StockCodeParseError>`.
- [ ] 3.5 Add `StockCodeParseError { InvalidJson, EmptyRecords, MissingField, InvalidCode }` + bridge `stock_code_error_into_quantix`.

## 4. Calendar Parser (`openstock_calendar.rs`)

- [ ] 4.1 Add `TradeDateRecord { date, #[serde(flatten)] extra }` and `WorkdayRecord { date, is_trading_day, #[serde(flatten)] extra }`.
- [ ] 4.2 Add `TradeDate { date: NaiveDate }` and `Workday { date, is_trading_day: bool }`.
- [ ] 4.3 Add `parse_calendar_date(value: &str) -> Result<NaiveDate, _>` accepting `%Y-%m-%d` and `%Y%m%d`.
- [ ] 4.4 Add `parse_trade_dates(env) -> Result<Vec<TradeDate>, CalendarParseError>`.
- [ ] 4.5 Add `parse_workdays(env) -> Result<Vec<Workday>, CalendarParseError>`.
- [ ] 4.6 Add `CalendarParseError { InvalidJson, EmptyRecords, MissingField, InvalidDate }` + bridge.

## 5. Visibility Widen (`openstock.rs`)

- [ ] 5.1 Widen `normalize_symbol` (L517) from `fn` to `pub(crate) fn` — signature unchanged.
- [ ] 5.2 Widen `parse_live_time` (L531) from `fn` to `pub(crate) fn` — signature unchanged.

## 6. Index Parser (`openstock_index.rs`)

- [ ] 6.1 Add `IndexKlineRecord { symbol, time, open, high, low, close, volume, amount }` (shape identical to existing `LiveShadowRecord`).
- [ ] 6.2 Add `parse_index_klines(env: OpenStockEnvelope<IndexKlineRecord>) -> Result<Vec<Kline>, IndexKlineParseError>` — reuse `normalize_symbol`, `parse_live_time`, canonical `Kline` with `AdjustType::None`.
- [ ] 6.3 Add `IndexKlineParseError` mirroring `OpenStockKlineParseError` family + bridge `index_kline_error_into_quantix`.

## 7. Module Wiring (`src/sources/mod.rs`)

- [ ] 7.1 Add `pub mod openstock_envelope; pub mod openstock_client; pub mod openstock_codes; pub mod openstock_calendar; pub mod openstock_index;`.
- [ ] 7.2 Add `pub use` re-exports for public types: `OpenStockEnvelope`, `OpenStockErrorEnvelope`, `OpenStockClient`, `OpenStockClientConfig`, `OpenStockResponse`, and parser entry points.
- [ ] 7.3 Add `pub use openstock_shadow::artifact_hash as openstock_artifact_hash;` (disambiguated re-export).

## 8. CLI Subcommands

- [ ] 8.1 Add `OpenStockCommands::ValidateCodes { payload, kind: Option<codes|all_stocks> }` to `src/cli/commands/data.rs`.
- [ ] 8.2 Add `OpenStockCommands::ValidateCalendar { payload, kind: trade_dates|workdays }`.
- [ ] 8.3 Add `OpenStockCommands::ValidateIndex { payload, symbol, start?, end? }`.
- [ ] 8.4 Add 3 sync handlers in `openstock_handler.rs`: `validate_openstock_codes`, `validate_openstock_calendar`, `validate_openstock_index` — each `read_payload` → parse → print `{ source, count, first/last sample, fields-seen }` → `Ok(())`.
- [ ] 8.5 Re-export 3 handlers in `src/cli/handlers/mod.rs:129-132`.
- [ ] 8.6 Add 3 dispatcher arms in `src/cli/handlers/app_shell.rs:307-339`.

## 9. Fixtures

- [ ] 9.1 `tests/fixtures/openstock/codes.json` + `codes_empty.json`.
- [ ] 9.2 `tests/fixtures/openstock/all_stocks.json`.
- [ ] 9.3 `tests/fixtures/openstock/trade_dates.json` + `trade_dates_empty.json`.
- [ ] 9.4 `tests/fixtures/openstock/workdays.json`.
- [ ] 9.5 `tests/fixtures/openstock/index_klines.json` + `index_klines_empty.json`.

## 10. Integration Tests

- [ ] 10.1 `tests/openstock_codes.rs` — happy path via `include_str!`, empty → error, missing-field → error.
- [ ] 10.2 `tests/openstock_calendar.rs` — happy + empty + invalid-date.
- [ ] 10.3 `tests/openstock_index.rs` — happy + empty + HighBelowLow + MixedCode.
- [ ] 10.4 `tests/openstock_client.rs` — `OpenStockResponse::from_envelope` composition + error envelope serde (inline JSON, no HTTP).

## 11. Verification

- [ ] 11.1 `cargo fmt --all -- --check`.
- [ ] 11.2 `cargo clippy --all-targets --workspace -- -D warnings`.
- [ ] 11.3 `cargo test --lib --package quantix-cli openstock`.
- [ ] 11.4 `cargo test --test openstock_codes openstock_calendar openstock_index openstock_client`.
- [ ] 11.5 `cargo test --doc`.
- [ ] 11.6 `cargo test --workspace` (regression).
- [ ] 11.7 `openspec validate openstock-data-consumption-p0-9 --strict`.
- [ ] 11.8 `openspec validate --all --strict`.
- [ ] 11.9 GitNexus `detect_changes` — expect LOW on `DataCommands`/`run_data_command`, no `Kline`/`BacktestEngine`/`ExecutionAdapter` touches.
- [ ] 11.10 FUNCTION_TREE: `ft:status --scope openstock-data-consumption-p0-9`, `ft:transition --node P0.8i --from in_progress --to review`, `ft:gate --node P0.8i --at review`.
- [ ] 11.11 `git diff --check`.
