# Spec — OpenStock Data Consumption (P0.11 TDX-API Cleanup Delta)

## ADDED Requirements

### Requirement: `import-klines` defaults to OpenStock source

The `quantix data tdx-api import-klines` subcommand SHALL accept a `--source <openstock|tdx-api>` flag with default `openstock`. When `--source openstock`, the command SHALL fetch K-line data via `OpenStockClient::fetch_index_klines` (or a sibling wrapper) and write through the existing ClickHouse client to the main `kline_data` table with `source = "OPENSTOCK"`. When `--source tdx-api`, the legacy path SHALL be preserved with a deprecation warning on stderr.

#### Scenario: default source is openstock

- **WHEN** the user runs `quantix data tdx-api import-klines --code 600000` (no `--source`)
- **THEN** the command fetches via `OpenStockClient::fetch_index_klines` or `fetch_historical_klines` (per code prefix)
- **AND** writes to ClickHouse main `kline_data` table (source column = "OPENSTOCK") after dry-run gate passes

#### Scenario: explicit legacy source emits deprecation warning

- **WHEN** the user runs `quantix data tdx-api import-klines --code 600000 --source tdx-api`
- **THEN** the command uses the legacy `TdxApiClient` path
- **AND** prints `⚠️ tdx-api legacy path, scheduled for removal in P0.11c` to stderr

#### Scenario: dry-run gate default

- **WHEN** the user runs `import-klines --source openstock` without `--apply`
- **THEN** the command prints a dry-run report (record count, sample rows, drift check)
- **AND** does NOT write to ClickHouse

#### Scenario: apply requires explicit confirmation

- **WHEN** the user runs `import-klines --source openstock --apply`
- **AND** env var `QUANTIX_OPENSTOCK_KLINE_APPLY` is NOT set to `yes`
- **THEN** the command refuses to write to `kline_data` and exits with non-zero status

### Requirement: `import-ticks` accepts OpenStock source after live verification

The `quantix data tdx-api import-ticks` subcommand SHALL accept a `--source <openstock|tdx-api>` flag with default `openstock`. The openstock branch SHALL use `OpenStockClient::fetch_tick_data(symbol, date)` where the JSON request parameter name is `symbol` (NOT `code`) per the eltdx adapter contract — sending `code` returns HTTP 422. The openstock branch SHALL be dry-run by default and write to TDengine only when `--apply` is set AND env var `QUANTIX_OPENSTOCK_TICK_APPLY=yes`.

#### Scenario: TICK_DATA request uses `symbol` parameter

- **WHEN** the openstock branch issues a `TICK_DATA` fetch
- **THEN** the JSON body params object contains key `symbol` (not `code`)
- **AND** the value is the stock code string (e.g. `"600000"`)

#### Scenario: TICK_DATA live smoke passed

- **GIVEN** the `TICK_DATA` smoke gate (task 2b.2) has passed
- **WHEN** the user runs `quantix data tdx-api import-ticks --code 600000`
- **THEN** the command fetches via `OpenStockClient::fetch_tick_data`
- **AND** the response envelope shape `data: [{meta, ticks}]` is flattened by `parse_tick_data` into `(TickMeta, Vec<Tick>)`

#### Scenario: TICK_DATA dry-run default

- **WHEN** the user runs `import-ticks --code 600000 --source openstock` without `--apply`
- **THEN** the command prints a dry-run report (tick count, first/last sample, artifact_hash, latency_ms)
- **AND** does NOT write to TDengine

#### Scenario: TICK_DATA apply requires explicit confirmation

- **WHEN** the user runs `import-ticks --code 600000 --source openstock --apply`
- **AND** env var `QUANTIX_OPENSTOCK_TICK_APPLY` is NOT set to `yes`
- **THEN** the command refuses to write to TDengine and exits with non-zero status

#### Scenario: TICK_DATA apply writes TDengine

- **GIVEN** `--apply` is set AND `QUANTIX_OPENSTOCK_TICK_APPLY=yes`
- **WHEN** the openstock branch parses the response
- **THEN** each `TickEntry` is mapped to `(timestamp_ms, price_f64, volume_i32, amount_f64, direction_status_i32)`
- **AND** `direction_status_i32` is `1` for `Buy`, `-1` for `Sell`, `0` for `Neutral`
- **AND** `decimal_to_f64` returns `Err` on out-of-range conversion (does NOT silently substitute `0.0`)
- **AND** any conversion error aborts the entire batch before the TDengine write

#### Scenario: TICK_DATA response fields are forward-compatible

- **WHEN** the openstock response contains fields not modeled in `TickEntry` (e.g. `price_milli`, `order_count`, `status`, `price_delta_raw`)
- **THEN** they are captured in `TickEntry.extra: HashMap<String, Value>` via `#[serde(flatten)]`
- **AND** `TickMeta` similarly captures unmapped meta fields
- **AND** dry-run output MAY surface selected extra fields for operator visibility

#### Scenario: TICK_DATA live smoke failed

- **GIVEN** the `TICK_DATA` smoke gate (task 2b.2) has failed
- **WHEN** the user runs `import-ticks`
- **THEN** the default source remains `tdx-api`
- **AND** P0.11b splits out as a separate OpenSpec change (slice boundary adapts)

#### Scenario: status byte semantic mismatch with legacy path

- **GIVEN** P0.11b is merged but P0.11c has not yet removed the legacy tdx-api path
- **WHEN** both paths write to the same TDengine `tick` table's status/direction column
- **THEN** the openstock path writes `TradeDirection` mapping (Buy=1, Sell=-1, Neutral=0)
- **AND** the legacy path writes raw tdx-api protocol bytes (semantics undefined in quantix)
- **AND** downstream consumers MUST NOT compare status values across the two paths
- **AND** P0.11c SHALL resolve this via one of: (a) unified mapping, (b) split columns, or (c) source tag column

### Requirement: `TdxApiClient` removal

After P0.11c completes, the codebase SHALL contain zero references to `TdxApiClient`, `TdxApiCommands`, `tdx_api_handler`, or `tdx-api` in any non-documentation file under `src/` or `tests/`.

#### Scenario: production code has no TdxApi references

- **WHEN** P0.11c task 3c.12 runs `grep -rn "tdx_api\|TdxApi\|tdx-api" src/ tests/`
- **THEN** the output is empty

#### Scenario: scheduler fallback rewired

- **GIVEN** P0.11c Option A is chosen (rewire, not delete)
- **WHEN** the diff is reviewed
- **THEN** `src/tasks/collect_scheduler.rs` no longer references `TdxApiClient`
- **AND** the `tdx_api_fallback` field is renamed to `openstock_fallback`
- **AND** the `set_tdx_api_fallback` method is renamed to `set_openstock_fallback`

#### Scenario: Docker compose no longer requires tdx-api

- **WHEN** P0.11c task 3c.15 completes
- **THEN** `docker-compose.yml` has the `tdx-api` service block commented out or removed
- **AND** `quantix` starts successfully without `tdx-api` running

### Requirement: FUNCTION_TREE.md reflects removal

After P0.11c, FUNCTION_TREE.md SHALL mark all tdx-api-related rows as deprecated or removed, with a pointer to the OpenStock replacement.

#### Scenario: status registry updated

- **WHEN** P0.11c task 3c.16 completes
- **THEN** FUNCTION_TREE.md lines L95, L212, L658, L781, L1126 reflect the removal
- **AND** no row claims tdx-api is `[部分实现]`

## MODIFIED Requirements

### Requirement: OpenStock is the sole data source

Prior to P0.11, the FUNCTION_TREE registry listed both `tdx-api` and `openstock` as data sources. After P0.11c, OpenStock SHALL be the sole data source for `quantix-rust`; tdx-api references are doc-only historical context.

#### Scenario: registry declares OpenStock as sole source

- **WHEN** P0.11c is archived
- **THEN** FUNCTION_TREE.md `sources/` row lists only OpenStock-related entries as live data sources
- **AND** tdx-api rows are either removed or marked `[deprecated, historical]`
- **AND** a changelog entry under FUNCTION_TREE.md cites P0.11 as the closure slice for the 2026-06-30 handoff
