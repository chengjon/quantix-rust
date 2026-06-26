# OpenStock Data Consumption P0.8a Inventory

Date: 2026-06-26

Status: inventory complete

Branch: `docs/openstock-p0-8a-inventory`

Base commit: `543ac7a7b6ea1be8ce2e1e6ab332f559bb1ca368`

## Scope

P0.8a is a documentation and governance inventory slice for the broker-independent OpenStock data consumption line.

This slice does not implement an OpenStock provider, parser, CLI command, persistence path, or data-source routing change. It maps the current Quantix data surface so P0.8b can start from a narrow fixture-owned parser contract.

## Inputs Reviewed

- OpenSpec P0.8 proposal/design/tasks/spec under `openspec/changes/openstock-data-consumption-p0-8/`
- GitNexus overview and query results for Sources, Import, Analysis, Strategy, Io, and CLI handler flows
- Graphiti reads for prior OpenStock, TDX, miniQMT manifest, and data-plane decisions
- Current code inventory anchors in:
  - `src/data/models.rs`
  - `src/sources/mod.rs`
  - `src/sources/tdx.rs`
  - `src/sources/tdx_api.rs`
  - `src/sources/bridge_tdx.rs`
  - `src/sources/eastmoney.rs`
  - `src/miniqmt_market.rs`
  - `src/cli/commands/data.rs`
  - `src/cli/handlers/data_handler.rs`
  - `src/cli/handlers/tdx_api_handler.rs`
  - `src/cli/handlers/import.rs`
  - `src/db/clickhouse/kline.rs`
  - `src/analysis/backtest.rs`
  - `src/analysis/polars_adapter.rs`
  - `src/cli/handlers/backtest_handler.rs`
  - `src/cli/handlers/strategy_handler/catalog.rs`

## Current Canonical Data Shapes

The existing shared data model surface is small and should be reused rather than bypassed.

| Shape | Location | Current Role | OpenStock Relevance |
|---|---|---|---|
| `Kline` | `src/data/models.rs` | Canonical OHLCV bar model used by sources, ClickHouse reads/writes, and backtest conversion paths | Primary target normalization shape for daily/minute bars |
| `Tick` | `src/data/models.rs` | Tick-level model | Out of P0.8b unless OpenStock fixture explicitly includes ticks |
| `StockInfo` | `src/data/models.rs` | Stock identity and metadata model | Candidate target for identity/basic-info normalization |
| `Market` | `src/data/models.rs` | Market enum for stock identity | Useful for exchange/market inference, but P0.8b should not expand routing semantics |
| `StockQuote` | `src/sources/tdx.rs` | Quote shape currently reused by TDX and Bridge TDX quote paths | Candidate quote normalization target if OpenStock fixture includes quote snapshots |

Important current behavior:

- `Kline.amount` is optional in the shared model and ClickHouse writer defaults missing amount in current TDX insert code.
- Backtest paths ultimately need `Vec<Kline>` keyed by stock code.
- Quote paths are less central to the first backtest loop than `Kline`, but the inventory should preserve `StockQuote` as a later target shape.

## Current Source Modules

| Module | Current Category | Evidence | P0.8 Boundary |
|---|---|---|---|
| `src/sources/tdx_api.rs` | Existing configured REST source | `TdxApiClient`, `TdxApiConfig`, `get_quote`, `get_kline_raw`, `get_kline_ths_qfq`, `get_kline_history`, CLI coverage through `data tdx-api` | Do not replace or route through OpenStock in P0.8b |
| `src/sources/bridge_tdx.rs` | External-runtime-dependent bridge source | `BridgeTdxSource`, `fetch_quotes_batch`, `Fetcher::get_kline`; depends on Windows Bridge runtime | Keep separate from OpenStock; no Windows Bridge dependency for OpenStock |
| `src/sources/tdx.rs` | Existing TDX source and quote model owner | `StockQuote`, `TdxSource` | Reuse `StockQuote` shape only if useful; do not modify source behavior |
| `src/sources/eastmoney.rs` | Existing EastMoney source | `EastMoneySource`, EastMoney-specific `StockInfo` and `Quote` structs | Keep independent; no hidden fallback to EastMoney |
| `src/sources/tdx_file.rs` | Local file source | TDX day-file parsing path recorded in source registry | Useful as fixture-first precedent, but not a direct OpenStock contract |
| `src/miniqmt_market.rs` | miniQMT manifest/artifact tooling | `ResolvedMarketArtifact`, manifest resolution, local artifact hash and report behavior | Must not be conflated with OpenStock; remains dry-run/report oriented and not a default persistence path |

## Storage And Persistence Paths

| Path | Location | Current Behavior | P0.8 Rule |
|---|---|---|---|
| ClickHouse kline read | `src/db/clickhouse/kline.rs::get_kline_data` | Reads `kline_data` into `crate::data::models::Kline` | P0.8a only records; P0.8b/P0.8c must not write |
| ClickHouse kline write | `insert_kline_data`, `insert_kline_data_batch`, `insert_kline_data_batch_with_source` | Writes `Kline` rows into `kline_data` with source metadata | Deferred to P0.8e or later schema/rollback gate |
| ClickHouse quote write | `insert_stock_quote`, `insert_stock_quotes_batch` | Quote persistence exists in the same kline extension impl | Out of first OpenStock parser slice |
| TDengine tick write | `src/db/tdengine.rs` | Used by tdx-api tick import path | Out of P0.8b/P0.8c |
| miniQMT manifest direct comparison | `src/cli/handlers/import.rs` | Opt-in read-only/direct comparison and dry-run report behavior | Not a persistence precedent for OpenStock |

Persistence conclusion:

OpenStock should first parse and normalize to local in-memory `Kline` or `StockInfo`/`StockQuote` candidates. Any ClickHouse write path requires a later explicit persistence slice with schema compatibility, deduplication, rollback, and dry-run gates.

## CLI And Consumer Surface

| Consumer | Location | Current Input Expectation | P0.8 Implication |
|---|---|---|---|
| `quantix data tdx-api ...` | `src/cli/commands/data.rs`, `src/cli/handlers/tdx_api_handler.rs` | Configured tdx-api REST source, environment/config dependent | OpenStock should not be hidden under `tdx-api`; use separate future command/status surface |
| `quantix data source ...` | `src/cli/handlers/data_handler.rs` | Data source registry/config display and default source resolution | P0.8c can add read-only status only after P0.8b contract exists |
| `quantix import market-manifest` | `src/cli/handlers/import.rs`, `src/miniqmt_market.rs` | miniQMT manifest/artifact dry-run/report and comparisons | Do not reuse for OpenStock; naming and semantics differ |
| Strategy `ma_cross` backtest | `src/cli/handlers/strategy_handler/catalog.rs` | Reads ClickHouse `get_kline_data`, converts to `Vec<Kline>`, then feeds `BacktestEngine` | Best later P0.8d local loop target after fixture parser |
| Backtest CLI | `src/cli/handlers/backtest_handler.rs`, `src/analysis/backtest.rs` | Backtest engine consumes kline-like data maps | Candidate downstream path, but P0.8a does not alter it |
| Paper/mock execution | `src/execution/`, strategy daemon tests | Runs after strategy signals; not a data provider boundary | Out of P0.8b; only P0.8d may consider paper/mock fixture loop without changing adapters |

## First Implementation Candidate

Recommended P0.8b target:

Create a fixture-owned OpenStock daily-kline parser/normalizer that accepts a committed JSON or CSV fixture and converts it into `Vec<crate::data::models::Kline>`.

Rationale:

- `Kline` is already the common downstream shape for ClickHouse and backtest.
- Daily kline avoids live quote freshness, tick volume, async task, and external runtime complexity.
- A fixture-owned parser can be tested without network, credentials, broker runtime, or ClickHouse.
- It gives P0.8d a direct local loop candidate: fixture data -> `Vec<Kline>` -> indicator/backtest path.

Suggested minimal P0.8b fixture fields:

| Field | Required | Target |
|---|---:|---|
| `code` | yes | `Kline.code` |
| `date` | yes | `Kline.date` |
| `open` | yes | `Kline.open` |
| `high` | yes | `Kline.high` |
| `low` | yes | `Kline.low` |
| `close` | yes | `Kline.close` |
| `volume` | yes | `Kline.volume` |
| `amount` | no | `Kline.amount` |
| `source` | optional metadata | not a model field today; preserve only in parser report until persistence design |

P0.8b should include fail-closed tests for:

- missing `code`;
- unparsable `date`;
- non-finite numeric values;
- `high < low`;
- empty fixture;
- mixed or unsupported period if period metadata is present.

## GitNexus Impact Targets For P0.8b

Before any P0.8b code edit, run GitNexus impact on the exact symbols selected by the implementation plan. Likely initial targets:

- New parser module/function if created under `src/sources/`.
- `src/sources/mod.rs` export if a new module is added.
- Existing `crate::data::models::Kline` if parser construction depends on model semantics.
- Future CLI status function only if P0.8c starts; do not include it in P0.8b unless explicitly approved.

Do not start from these higher-impact surfaces in P0.8b:

- `src/cli/handlers/data_handler.rs::list_data_sources`
- `src/cli/handlers/tdx_api_handler.rs`
- `src/db/clickhouse/kline.rs`
- `src/cli/handlers/strategy_handler/catalog.rs::run_ma_cross_backtest`
- any execution adapter or qmt_live handler

## Explicit Non-Goals Preserved

P0.8a and the recommended P0.8b path do not:

- call live OpenStock endpoints in CI;
- write ClickHouse or TDengine;
- change `tdx_api`, `bridge_tdx`, `tdx`, `tdx_file`, `eastmoney`, `websocket`, or miniQMT behavior;
- alter miniQMT market-manifest dry-run/report semantics;
- change qmt_live runtime readiness, canary, submit, query, or cancel behavior;
- change `ExecutionAdapter` or `OrderStatus`;
- route existing data-source commands to OpenStock;
- perform `.unwrap()` cleanup.

## P0.8a Closeout

P0.8a satisfies OpenSpec tasks 1.1 through 1.4:

- mapped stock identity, kline, quote, market foundation, analysis, backtest, and paper/mock-adjacent consumers;
- classified existing source modules;
- selected a first implementation candidate;
- preserved the no-production-code-change boundary for this inventory slice.

Next slice: P0.8b provider contract and fixture parser, with TDD and fresh GitNexus impact before code edits.
