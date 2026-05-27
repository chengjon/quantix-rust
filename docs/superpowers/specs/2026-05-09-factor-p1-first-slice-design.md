# 2026-05-09 Factor P1 First Slice Design

## Goal

Build the first Rust-native factor research slice for `quantix-rust` without expanding into broker, strategy execution, or GUI scope.

This slice must prove the full factor dataflow:

1. async loader fetches input data
2. loader output is normalized into a Polars-backed factor dataset
3. group-aware factor operators run synchronously
4. results can be checked and exported
5. CLI users can list and compute factors

The first slice is intentionally small. It is not trying to deliver Alpha101/191 yet. It is building the platform that those later capabilities will run on.

## Scope

### In Scope

- new `factor` module family under `src/factor/`
- Polars-first long-form factor dataset
- async loader contract with sync compute boundary
- minimal operator set:
  - `cs_rank`
  - `ts_delay`
  - `ts_delta`
- factor metadata and registry
- factor validation helpers:
  - time alignment
  - duplicate row detection
  - basic anti-lookahead structural checks
- result export wiring for factor compute output
- CLI:
  - `quantix factor list`
  - `quantix factor compute`
- end-to-end integration test using local mock data only

### Out of Scope

- Alpha101
- Alpha191
- IC/IR
- factor correlation
- factor neutralization
- layered backtest
- database-backed factor result cache
- live broker or bridge integration changes

## Chosen Approach

Three implementation shapes were considered:

1. Extend `analysis/` directly
2. Add a dedicated `factor/` subsystem and reuse selected analysis/fundamental pieces
3. Add factor commands first, then retrofit internals later

Recommendation: **Option 2**

Reasoning:

- `factor` has different semantics from existing indicator helpers
- Alpha, neutralization, IC/IR, and layered backtests need a dedicated vocabulary and contract
- keeping `factor/` isolated avoids turning `analysis/` into a mixed indicator-and-research bucket
- CLI and tests become easier to grow without leaking research concerns into execution/risk modules
- the existing analysis code often uses Vec-backed helper structures, while the factor subsystem is intentionally Polars DataFrame-first so later all-market factor workloads can use columnar and grouped execution directly

## Architecture

### Module Layout

```text
src/factor/
├── mod.rs
├── types.rs
├── loader.rs
├── dataset.rs
├── operators.rs
├── catalog.rs
├── export.rs
└── check.rs
```

The first implementation slice should also add:

```text
src/cli/commands/factor.rs
src/cli/handlers/factor.rs
src/cli/tests/factor.rs
tests/factor_pipeline_test.rs
```

And update:

- [src/lib.rs](/opt/claude/quantix-rust/src/lib.rs)
- [src/cli/commands/mod.rs](/opt/claude/quantix-rust/src/cli/commands/mod.rs)
- [src/cli/handlers/mod.rs](/opt/claude/quantix-rust/src/cli/handlers/mod.rs)

### Responsibility Split

- `types.rs`
  - shared enums and request/result structs
  - factor metadata model
  - missing-value policy model
- `loader.rs`
  - async input loading contract
  - request types for universe/date range/required fields
- `dataset.rs`
  - converts loader output into normalized long-form Polars dataset
  - enforces sorting, uniqueness, and schema checks
- `operators.rs`
  - reusable Polars-first operator helpers
  - no source I/O, no CLI knowledge
- `catalog.rs`
  - factor registry and metadata lookup
  - compute entrypoint for named factors
- `export.rs`
  - factor result file output shaping for CSV/JSON/Parquet
  - does not own CLI-only table rendering
- `check.rs`
  - alignment validation
  - duplicate/ordering checks
  - basic anti-lookahead structural checks

## Data Contract

### Loader Boundary

The loader is async. The factor engine is sync after loading.

Recommended contract:

```rust
use crate::core::{QuantixError, Result};
use polars::prelude::DataFrame;

#[async_trait]
pub trait FactorDataLoader {
    async fn load_bars(&self, request: &FactorLoadRequest) -> Result<DataFrame>;
}
```

Design rule:

- loader implementations finish all async work internally
- loader returns a long-form Polars `DataFrame`
- `FactorDataset::from_loader(...).await` waits for load completion, then normalizes the frame
- all operator execution after dataset creation is synchronous
- bare `Result<T>` in factor APIs means `crate::core::Result<T>`, backed by `QuantixError`

This keeps async concerns out of operator logic and out of later Alpha implementations.

### Dataset Schema

The normalized first-slice dataset is long-form.

Required columns:

- `date`
- `symbol`
- `open`
- `high`
- `low`
- `close`
- `volume`

Optional first-slice support:

- `amount`

Required dtype contract:

| Column | Polars dtype | Notes |
|--------|--------------|-------|
| `date` | `Date` | Calendar trading date. P1 should normalize accepted date-like input into Polars `Date`. |
| `symbol` | `String` / `Utf8` | Stock symbol in the project canonical display form. |
| `open` | `Float64` | Price column. |
| `high` | `Float64` | Price column. |
| `low` | `Float64` | Price column. |
| `close` | `Float64` | Price column and first-slice primary input. |
| `volume` | `Int64` | Share volume. |
| `amount` | `Float64` | Optional turnover amount. |

P1 uses `Float64` for factor math even though some existing modules expose `Decimal`. The factor engine is a research computation layer; decimal-precision accounting semantics stay in trade, risk, and reporting modules.

Normalization rules:

- dataset must be sorted by `symbol`, then `date` ascending
- each `(symbol, date)` pair must be unique
- required columns must exist with the dtype contract above or be explicitly castable during normalization
- output remains long-form; no default wide full-market panel is introduced in P1

### Operator Semantics

The engine must document these defaults explicitly.

- all `ts_*` operators default to:
  - partition by `symbol`
  - order by `date asc`
- all `cs_*` operators default to:
  - partition by `date`

Examples:

- `ts_delay(close, 1)` means per-symbol lag by one row after symbol/date sorting
- `ts_delta(close, 1)` means `close - ts_delay(close, 1)` per symbol
- `cs_rank(close)` means cross-sectional rank across symbols on each date

The dataset layer is responsible for ensuring the sort precondition before operators run.

First-slice operator signatures:

```rust
use polars::prelude::{DataFrame, PolarsResult, Series};

pub fn cs_rank(df: &DataFrame, col: &str) -> PolarsResult<Series>;

pub fn ts_delay(df: &DataFrame, col: &str, periods: usize) -> PolarsResult<Series>;

pub fn ts_delta(df: &DataFrame, col: &str, periods: usize) -> PolarsResult<Series>;
```

These functions return a `Series` aligned with the input frame's row order. The caller is responsible for attaching the returned series to an output frame under the requested factor name. Negative time shifts are not representable in the P1 API; future variants that accept signed offsets must reject negative values through `check.rs`.

## Core Types

### Metadata and Policies

```rust
pub enum FactorCategory {
    Technical,
    Fundamental,
    Composite,
    Experimental,
}

pub enum MissingPolicy {
    KeepNull,
    ForwardFill,
    DropRow,
    DropLeadingWindow,
}

pub struct FactorMeta {
    pub id: String,
    pub category: FactorCategory,
    pub description: String,
    pub author: Option<String>,
    pub source: Option<String>,
    pub refresh_frequency: Option<String>,
    pub required_fields: Vec<String>,
    pub missing_policy: MissingPolicy,
}
```

The `missing_policy` lives in metadata so missing handling is consistent and not reimplemented ad hoc in every factor.

### Requests and Results

```rust
pub struct FactorLoadRequest {
    pub symbols: Vec<String>,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub required_fields: Vec<String>,
}

pub struct FactorComputeRequest {
    pub factors: Vec<String>,
    pub symbols: Vec<String>,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub run_checks: bool,
}

pub struct FactorComputeResult {
    pub factor_id: String,
    pub frame: DataFrame,
}
```

For the first slice, each computed factor may return a long-form frame containing:

- `date`
- `symbol`
- `value`

Result dtype contract:

| Column | Polars dtype | Notes |
|--------|--------------|-------|
| `date` | `Date` | Copied from normalized dataset. |
| `symbol` | `String` / `Utf8` | Copied from normalized dataset. |
| `value` | `Float64` | Numeric factor value; nulls are allowed according to `MissingPolicy`. |

## Factor Registry

The catalog should be able to:

- list registered factors and metadata
- compute a named factor
- accept multiple factors in one request even if execution reuse is not optimized yet

First-slice factor set:

- `rank_close`
- `delay_close_1`
- `delta_close_1`

The first slice should allow repeated `--factor` flags in CLI even if initial execution computes them one by one. This preserves a forward-compatible command shape for future batching and shared intermediate reuse.

## Validation and Anti-Lookahead

`check.rs` is not a late-stage add-on. It belongs in the first slice.

First-slice checks:

- `ensure_required_columns`
- `ensure_unique_symbol_date`
- `ensure_symbol_date_sorted`
- `validate_time_alignment`
- `validate_no_negative_time_shift`
- `validate_no_lookahead_basic`

`validate_no_lookahead_basic` in P1 is structural, not full economic leakage detection.

It should at least enforce:

- time-series operators cannot use negative delays
- dataset order is strictly symbol/date ascending before `ts_*` execution
- no duplicated future-position rows exist after normalization

More advanced leakage checks can be added later when forward returns and labels are introduced.

## CLI Design

Add a new top-level command family:

```text
quantix factor ...
```

### `quantix factor list`

Purpose:

- show available factor IDs and metadata

Suggested flags:

- `--category <technical|fundamental|composite|experimental>` optional
- `--verbose` optional

### `quantix factor compute`

Purpose:

- compute one or more factors over a symbol/date range

Suggested flags:

- `--input <CSV_PATH>` required for P1 local-file loading
- `--factor <FACTOR_ID>` repeatable, required
- `--symbol <CODE>` repeatable, required for P1
- `--start <YYYY-MM-DD>` required
- `--end <YYYY-MM-DD>` required
- `--format <table|csv|json|parquet>` optional, default `table`
- `--output <PATH>` optional for file formats
- `--skip-checks` optional

`table` is a CLI display mode only. It should be rendered by the factor CLI handler and must not be added to the shared `io::exporter::ExportFormat` enum. `export.rs` handles file outputs only: CSV, JSON, and Parquet.

P1 `factor compute` uses a local CSV input file to avoid hidden mock data and avoid requiring a database connection in the first slice. Database-backed loaders, universe presets, batch execution planning, and caching flags are not part of this slice.

## Testing

### Unit Coverage

- `src/cli/tests/factor.rs`
  - parse `factor list`
  - parse `factor compute`
- `src/cli/handlers/tests/`
  - handler-level factor command tests should live here if the implementation adds handler-specific branch coverage, following existing CLI handler test patterns
- operator tests
  - `cs_rank`
  - `ts_delay`
  - `ts_delta`
- dataset tests
  - sort normalization
  - duplicate row rejection
  - missing required column rejection
- check tests
  - basic lookahead and ordering validation

### Integration Coverage

Add [tests/factor_pipeline_test.rs](/opt/claude/quantix-rust/tests/factor_pipeline_test.rs).

This test must be environment-independent and use local mock data only.

Mock dataset requirements:

- 3 stocks
- 10 trading days
- long-form rows
- columns: `date`, `symbol`, `open`, `high`, `low`, `close`, `volume`
- include deliberate missing values

The integration path must verify:

1. async mock loader returns test frame
2. dataset normalization succeeds
3. checks run successfully
4. `rank_close` computes expected values
5. export path produces a stable result shape

This integration test is the first gate before any Alpha implementation starts.

## Reuse and Non-Reuse Rules

Reuse:

- [src/analysis/polars_adapter.rs](/opt/claude/quantix-rust/src/analysis/polars_adapter.rs) only for Polars initialization patterns and limited utility ideas
- existing `fundamental` providers as later factor input sources
- existing `io` export surfaces where practical

Do not reuse as first-slice engine core:

- loop-heavy indicator implementations as the factor execution backbone
- `risk::volatility` semantics for research volatility factors

Research-side volatility and risk-side trading volatility must stay separate. When the research-side version is added later, prefer a distinct name such as `research_specific_vol`.

## Acceptance Criteria

The first slice is complete when all of the following are true:

1. `src/factor/` exists with the planned first-slice files
2. `quantix factor list` works
3. `quantix factor compute` works for at least `rank_close`
4. factor dataset normalization is Polars-backed and long-form
5. time-series and cross-sectional semantics are explicitly encoded
6. missing-value policy exists in factor metadata
7. first-slice checks run by default
8. the environment-independent integration test passes

## Follow-Up After P1

After this slice is green, the next implementation plan should cover:

1. `technical`
2. `fundamental`
3. `composite`
4. `factor score`
5. `alpha101`

This keeps the project on the approved platform-first migration path.
