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

The `quantix data tdx-api import-ticks` subcommand SHALL accept a `--source <openstock|tdx-api>` flag. The default value is `openstock` IF AND ONLY IF task 2b.2 (`TICK_DATA` live smoke) has passed; otherwise the default is `tdx-api` and the openstock branch is hidden behind the flag.

#### Scenario: TICK_DATA live smoke passed

- **GIVEN** the `TICK_DATA` smoke gate (task 2b.2) has passed
- **WHEN** the user runs `quantix data tdx-api import-ticks --code 600000`
- **THEN** the command fetches via `OpenStockClient::fetch_tick_data`
- **AND** writes through the existing TDengine client

#### Scenario: TICK_DATA live smoke failed

- **GIVEN** the `TICK_DATA` smoke gate (task 2b.2) has failed
- **WHEN** the user runs `import-ticks`
- **THEN** the default source remains `tdx-api`
- **AND** P0.11b splits out as a separate OpenSpec change (slice boundary adapts)

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
