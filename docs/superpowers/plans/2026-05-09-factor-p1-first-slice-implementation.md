# Factor P1 First Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> 状态源说明：本文是历史实施计划，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 [`FUNCTION_TREE.md`](../../../FUNCTION_TREE.md) 的状态注册表行为准。

**Goal:** Build the first Rust-native factor research slice with a Polars-backed dataset, minimal factor operators, validation, export, and `quantix factor list/compute`.

**Architecture:** Add an isolated `src/factor/` subsystem. Data loading is async and returns a long-form Polars `DataFrame`; normalization, checks, operators, catalog compute, and export are synchronous after dataset creation. CLI exposes the first user-facing surface without touching broker, execution, risk, or strategy runtime behavior.

**Tech Stack:** Rust 2024, Polars 0.43, async-trait, chrono, clap, existing `crate::core::{Result, QuantixError}`.

---

## File Structure

- Create: `src/factor/mod.rs`
- Create: `src/factor/types.rs`
- Create: `src/factor/loader.rs`
- Create: `src/factor/dataset.rs`
- Create: `src/factor/check.rs`
- Create: `src/factor/operators.rs`
- Create: `src/factor/catalog.rs`
- Create: `src/factor/export.rs`
- Create: `src/cli/commands/factor.rs`
- Create: `src/cli/handlers/factor.rs`
- Create: `src/cli/tests/factor.rs`
- Create: `tests/factor_pipeline_test.rs`
- Modify: `src/lib.rs`
- Modify: `src/cli/commands/mod.rs`
- Modify: `src/cli/handlers/mod.rs`
- Modify: `src/cli/tests/mod.rs`
- Modify: `FUNCTION_TREE.md`

## Task 1: Add CLI Command Shape

**Files:**
- Create: `src/cli/commands/factor.rs`
- Create: `src/cli/tests/factor.rs`
- Modify: `src/cli/commands/mod.rs`
- Modify: `src/cli/tests/mod.rs`

- [ ] **Step 1: Write failing CLI parse tests**

Add this module to `src/cli/tests/mod.rs`:

```rust
mod factor;
```

Create `src/cli/tests/factor.rs`:

```rust
use super::*;

#[test]
fn parses_factor_list_command() {
    let cli = Cli::try_parse_from(["quantix", "factor", "list", "--verbose"]).unwrap();

    match cli.command {
        Commands::Factor(FactorCommands::List { category, verbose }) => {
            assert_eq!(category, None);
            assert!(verbose);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_factor_compute_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "factor",
        "compute",
        "--input",
        "/tmp/factor-input.csv",
        "--factor",
        "rank_close",
        "--factor",
        "delta_close_1",
        "--symbol",
        "000001.SZ",
        "--symbol",
        "600000.SH",
        "--start",
        "2026-01-01",
        "--end",
        "2026-01-10",
        "--format",
        "json",
        "--output",
        "/tmp/factors.json",
        "--skip-checks",
    ])
    .unwrap();

    match cli.command {
        Commands::Factor(FactorCommands::Compute {
            input,
            factors,
            symbols,
            start,
            end,
            format,
            output,
            skip_checks,
        }) => {
            assert_eq!(input, "/tmp/factor-input.csv");
            assert_eq!(factors, vec!["rank_close", "delta_close_1"]);
            assert_eq!(symbols, vec!["000001.SZ", "600000.SH"]);
            assert_eq!(start, "2026-01-01");
            assert_eq!(end, "2026-01-10");
            assert_eq!(format, FactorOutputFormat::Json);
            assert_eq!(output.as_deref(), Some("/tmp/factors.json"));
            assert!(skip_checks);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}
```

- [ ] **Step 2: Run parser tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test cli::tests::factor -- --nocapture
```

Expected: FAIL because `FactorCommands`, `FactorOutputFormat`, and `Commands::Factor` do not exist.

- [ ] **Step 3: Add command enum and wire top-level command**

Create `src/cli/commands/factor.rs`:

```rust
use clap::{Subcommand, ValueEnum};

#[derive(Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum FactorOutputFormat {
    Table,
    Csv,
    Json,
    Parquet,
}

#[derive(Subcommand, Debug)]
pub enum FactorCommands {
    /// List registered factor definitions
    List {
        /// Filter by factor category
        #[arg(long)]
        category: Option<String>,

        /// Show full factor metadata
        #[arg(long)]
        verbose: bool,
    },

    /// Compute one or more factors for symbols and a date range
    Compute {
        /// CSV input file with date,symbol,open,high,low,close,volume columns
        #[arg(long)]
        input: String,

        /// Factor ID; repeat to compute multiple factors
        #[arg(long = "factor", required = true)]
        factors: Vec<String>,

        /// Stock symbol; repeat to compute multiple symbols
        #[arg(long = "symbol", required = true)]
        symbols: Vec<String>,

        /// Start date, YYYY-MM-DD
        #[arg(long)]
        start: String,

        /// End date, YYYY-MM-DD
        #[arg(long)]
        end: String,

        /// Output format; table is CLI-only display
        #[arg(long, value_enum, default_value_t = FactorOutputFormat::Table)]
        format: FactorOutputFormat,

        /// Output path for csv/json/parquet
        #[arg(long)]
        output: Option<String>,

        /// Skip first-slice factor input checks
        #[arg(long)]
        skip_checks: bool,
    },
}
```

Modify `src/cli/commands/mod.rs`:

```rust
mod factor;
pub use factor::{FactorCommands, FactorOutputFormat};
```

Add the top-level variant:

```rust
/// Factor research commands
#[command(subcommand)]
Factor(FactorCommands),
```

Temporarily route it to an unsupported error until the handler exists:

```rust
Commands::Factor(_) => {
    return Err(crate::core::QuantixError::Unsupported(
        "factor command handler is not wired yet".to_string(),
    ));
}
```

- [ ] **Step 4: Run parser tests and verify pass**

Run:

```bash
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test cli::tests::factor -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/cli/commands/factor.rs src/cli/commands/mod.rs src/cli/tests/factor.rs src/cli/tests/mod.rs
git commit -m "feat: add factor CLI command shape"
```

## Task 2: Add Factor Core Types and Loader Contract

**Files:**
- Create: `src/factor/mod.rs`
- Create: `src/factor/types.rs`
- Create: `src/factor/loader.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write compile-facing type usage test**

Create `tests/factor_pipeline_test.rs` with only the type smoke test first:

```rust
use chrono::NaiveDate;
use quantix_cli::factor::{
    FactorCategory, FactorComputeRequest, FactorLoadRequest, FactorMeta, MissingPolicy,
};

#[test]
fn factor_core_types_have_first_slice_fields() {
    let meta = FactorMeta {
        id: "rank_close".to_string(),
        category: FactorCategory::Technical,
        description: "Cross-sectional rank of close by date".to_string(),
        author: Some("quantix".to_string()),
        source: Some("p1".to_string()),
        refresh_frequency: Some("daily".to_string()),
        required_fields: vec!["close".to_string()],
        missing_policy: MissingPolicy::KeepNull,
    };

    let load = FactorLoadRequest {
        symbols: vec!["000001.SZ".to_string()],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
        required_fields: meta.required_fields.clone(),
    };

    let compute = FactorComputeRequest {
        factors: vec![meta.id.clone()],
        symbols: load.symbols.clone(),
        start: load.start,
        end: load.end,
        run_checks: true,
    };

    assert_eq!(compute.factors, vec!["rank_close"]);
    assert_eq!(load.required_fields, vec!["close"]);
}
```

- [ ] **Step 2: Run test and verify failure**

Run:

```bash
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test --test factor_pipeline_test -- --nocapture
```

Expected: FAIL because `quantix_cli::factor` does not exist.

- [ ] **Step 3: Add factor module exports and type definitions**

Create `src/factor/mod.rs`:

```rust
pub mod loader;
pub mod types;

pub use loader::FactorDataLoader;
pub use types::{
    FactorCategory, FactorComputeRequest, FactorComputeResult, FactorLoadRequest, FactorMeta,
    MissingPolicy,
};
```

Create `src/factor/types.rs`:

```rust
use chrono::NaiveDate;
use polars::prelude::DataFrame;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactorCategory {
    Technical,
    Fundamental,
    Composite,
    Experimental,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingPolicy {
    KeepNull,
    ForwardFill,
    DropRow,
    DropLeadingWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
pub struct FactorLoadRequest {
    pub symbols: Vec<String>,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub required_fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FactorComputeRequest {
    pub factors: Vec<String>,
    pub symbols: Vec<String>,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub run_checks: bool,
}

#[derive(Debug, Clone)]
pub struct FactorComputeResult {
    pub factor_id: String,
    pub frame: DataFrame,
}
```

Create `src/factor/loader.rs`:

```rust
use async_trait::async_trait;
use polars::prelude::DataFrame;

use crate::core::Result;
use crate::factor::types::FactorLoadRequest;

#[async_trait]
pub trait FactorDataLoader: Send + Sync {
    async fn load_bars(&self, request: &FactorLoadRequest) -> Result<DataFrame>;
}
```

Modify `src/lib.rs`:

```rust
pub mod factor;
```

- [ ] **Step 4: Run test and verify pass**

Run:

```bash
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test --test factor_pipeline_test -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/factor/mod.rs src/factor/types.rs src/factor/loader.rs src/lib.rs tests/factor_pipeline_test.rs
git commit -m "feat: add factor core contracts"
```

## Task 3: Add Dataset Normalization and Checks

**Files:**
- Create: `src/factor/dataset.rs`
- Create: `src/factor/check.rs`
- Modify: `src/factor/mod.rs`
- Test: `tests/factor_pipeline_test.rs`

- [ ] **Step 1: Add failing dataset/check tests**

Append to `tests/factor_pipeline_test.rs`:

```rust
use async_trait::async_trait;
use polars::prelude::*;
use quantix_cli::core::Result;
use quantix_cli::factor::{FactorDataLoader, FactorDataset};

struct MockFactorLoader {
    frame: DataFrame,
}

#[async_trait]
impl FactorDataLoader for MockFactorLoader {
    async fn load_bars(&self, _request: &FactorLoadRequest) -> Result<DataFrame> {
        Ok(self.frame.clone())
    }
}

fn mock_factor_frame() -> DataFrame {
    df!(
        "date" => &[
            "2026-01-01", "2026-01-01", "2026-01-01",
            "2026-01-02", "2026-01-02", "2026-01-02",
        ],
        "symbol" => &[
            "000001.SZ", "600000.SH", "000002.SZ",
            "000001.SZ", "600000.SH", "000002.SZ",
        ],
        "open" => &[10.0, 20.0, 30.0, 11.0, 21.0, 31.0],
        "high" => &[10.5, 20.5, 30.5, 11.5, 21.5, 31.5],
        "low" => &[9.5, 19.5, 29.5, 10.5, 20.5, 30.5],
        "close" => &[10.2, 20.2, 30.2, 11.2, 21.2, 31.2],
        "volume" => &[1000i64, 2000, 3000, 1100, 2100, 3100],
    )
    .unwrap()
}

#[tokio::test]
async fn dataset_from_loader_normalizes_and_checks_schema() {
    let loader = MockFactorLoader {
        frame: mock_factor_frame(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        required_fields: vec!["close".to_string()],
    };

    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    assert_eq!(dataset.frame().height(), 6);
    dataset.ensure_time_aligned().unwrap();
    dataset.validate_no_lookahead_basic().unwrap();
}
```

- [ ] **Step 2: Run test and verify failure**

Run:

```bash
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test --test factor_pipeline_test dataset_from_loader_normalizes_and_checks_schema -- --nocapture
```

Expected: FAIL because `FactorDataset` does not exist.

- [ ] **Step 3: Implement dataset and check modules**

Create `src/factor/dataset.rs`:

```rust
use polars::prelude::*;

use crate::core::{QuantixError, Result};
use crate::factor::loader::FactorDataLoader;
use crate::factor::types::FactorLoadRequest;

#[derive(Debug, Clone)]
pub struct FactorDataset {
    frame: DataFrame,
}

impl FactorDataset {
    pub async fn from_loader<L>(loader: &L, request: &FactorLoadRequest) -> Result<Self>
    where
        L: FactorDataLoader + ?Sized,
    {
        let frame = loader.load_bars(request).await?;
        let dataset = Self::new(frame)?;
        dataset.ensure_required_columns(&request.required_fields)?;
        Ok(dataset)
    }

    pub fn new(frame: DataFrame) -> Result<Self> {
        let frame = crate::factor::check::normalize_factor_frame(frame)?;
        let dataset = Self { frame };
        dataset.ensure_required_columns(&[])?;
        dataset.ensure_time_aligned()?;
        Ok(dataset)
    }

    pub fn frame(&self) -> &DataFrame {
        &self.frame
    }

    pub fn ensure_required_columns(&self, fields: &[String]) -> Result<()> {
        crate::factor::check::ensure_required_columns(&self.frame, fields)
    }

    pub fn ensure_time_aligned(&self) -> Result<()> {
        crate::factor::check::ensure_symbol_date_sorted(&self.frame)?;
        crate::factor::check::ensure_unique_symbol_date(&self.frame)
    }

    pub fn validate_no_lookahead_basic(&self) -> Result<()> {
        crate::factor::check::validate_no_lookahead_basic(&self.frame)
    }
}
```

Create `src/factor/check.rs`:

```rust
use polars::prelude::*;

use crate::core::{QuantixError, Result};

const BASE_COLUMNS: &[&str] = &["date", "symbol", "open", "high", "low", "close", "volume"];

pub fn normalize_factor_frame(mut frame: DataFrame) -> Result<DataFrame> {
    ensure_base_columns(&frame)?;
    cast_required_types(&mut frame)?;
    frame
        .sort(["symbol", "date"], Default::default())
        .map_err(|e| QuantixError::DataParse(format!("factor dataset sort failed: {}", e)))
}

pub fn ensure_required_columns(frame: &DataFrame, fields: &[String]) -> Result<()> {
    ensure_base_columns(frame)?;
    for field in fields {
        if frame.column(field).is_err() {
            return Err(QuantixError::DataParse(format!(
                "factor dataset missing required field `{}`",
                field
            )));
        }
    }
    Ok(())
}

pub fn ensure_unique_symbol_date(frame: &DataFrame) -> Result<()> {
    let unique = frame
        .select(["symbol", "date"])
        .and_then(|df| df.unique(None, UniqueKeepStrategy::First, None))
        .map_err(|e| QuantixError::DataParse(format!("factor uniqueness check failed: {}", e)))?;
    if unique.height() != frame.height() {
        return Err(QuantixError::DataParse(
            "factor dataset contains duplicate (symbol, date) rows".to_string(),
        ));
    }
    Ok(())
}

pub fn ensure_symbol_date_sorted(frame: &DataFrame) -> Result<()> {
    let sorted = frame
        .sort(["symbol", "date"], Default::default())
        .map_err(|e| QuantixError::DataParse(format!("factor sort check failed: {}", e)))?;
    if sorted.equals(frame) {
        Ok(())
    } else {
        Err(QuantixError::DataParse(
            "factor dataset must be sorted by symbol,date ascending".to_string(),
        ))
    }
}

pub fn validate_no_lookahead_basic(frame: &DataFrame) -> Result<()> {
    ensure_symbol_date_sorted(frame)?;
    ensure_unique_symbol_date(frame)
}

fn ensure_base_columns(frame: &DataFrame) -> Result<()> {
    for column in BASE_COLUMNS {
        if frame.column(column).is_err() {
            return Err(QuantixError::DataParse(format!(
                "factor dataset missing base column `{}`",
                column
            )));
        }
    }
    Ok(())
}

fn cast_required_types(frame: &mut DataFrame) -> Result<()> {
    cast_column(frame, "date", &DataType::Date)?;
    cast_column(frame, "symbol", &DataType::String)?;
    for column in ["open", "high", "low", "close"] {
        cast_column(frame, column, &DataType::Float64)?;
    }
    cast_column(frame, "volume", &DataType::Int64)?;
    if frame.column("amount").is_ok() {
        cast_column(frame, "amount", &DataType::Float64)?;
    }
    Ok(())
}

fn cast_column(frame: &mut DataFrame, column: &str, dtype: &DataType) -> Result<()> {
    let casted = frame
        .column(column)
        .and_then(|s| s.cast(dtype))
        .map_err(|e| QuantixError::DataParse(format!("factor column `{}` cast failed: {}", column, e)))?;
    frame
        .replace(column, casted)
        .map_err(|e| QuantixError::DataParse(format!("factor column `{}` replace failed: {}", column, e)))?;
    Ok(())
}
```

Modify `src/factor/mod.rs`:

```rust
pub mod check;
pub mod dataset;
pub mod loader;
pub mod types;

pub use dataset::FactorDataset;
pub use loader::FactorDataLoader;
pub use types::{
    FactorCategory, FactorComputeRequest, FactorComputeResult, FactorLoadRequest, FactorMeta,
    MissingPolicy,
};
```

- [ ] **Step 4: Run test and verify pass**

Run:

```bash
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test --test factor_pipeline_test dataset_from_loader_normalizes_and_checks_schema -- --nocapture
```

Expected: PASS. If Polars 0.43 requires small API adjustments, keep the public behavior and test assertions unchanged.

- [ ] **Step 5: Commit**

```bash
git add src/factor/mod.rs src/factor/dataset.rs src/factor/check.rs tests/factor_pipeline_test.rs
git commit -m "feat: add factor dataset checks"
```

## Task 4: Add Operators and Built-In Catalog

**Files:**
- Create: `src/factor/operators.rs`
- Create: `src/factor/catalog.rs`
- Modify: `src/factor/mod.rs`
- Test: `tests/factor_pipeline_test.rs`

- [ ] **Step 1: Add failing operator/catalog tests**

Append to `tests/factor_pipeline_test.rs`:

```rust
use quantix_cli::factor::{builtin_factor_catalog, cs_rank, ts_delta, ts_delay};

#[test]
fn operators_compute_aligned_series() {
    let df = mock_factor_frame();

    let rank = cs_rank(&df, "close").unwrap();
    assert_eq!(rank.len(), df.height());

    let delay = ts_delay(&df, "close", 1).unwrap();
    assert_eq!(delay.len(), df.height());

    let delta = ts_delta(&df, "close", 1).unwrap();
    assert_eq!(delta.len(), df.height());
}

#[tokio::test]
async fn catalog_lists_and_computes_rank_close() {
    let loader = MockFactorLoader {
        frame: mock_factor_frame(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        required_fields: vec!["close".to_string()],
    };
    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    let catalog = builtin_factor_catalog();

    assert!(catalog.list().iter().any(|meta| meta.id == "rank_close"));
    let result = catalog.compute("rank_close", &dataset).unwrap();
    assert_eq!(result.factor_id, "rank_close");
    assert_eq!(result.frame.height(), dataset.frame().height());
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test --test factor_pipeline_test catalog_lists_and_computes_rank_close operators_compute_aligned_series -- --nocapture
```

Expected: FAIL because operators and catalog do not exist.

- [ ] **Step 3: Implement operators and catalog**

Create `src/factor/operators.rs`:

```rust
use polars::prelude::*;

pub fn cs_rank(df: &DataFrame, col: &str) -> PolarsResult<Series> {
    df.clone()
        .lazy()
        .with_columns([col(col).rank(Default::default(), None).over([col("date")]).alias("__factor_value")])
        .collect()?
        .column("__factor_value")
        .cloned()
}

pub fn ts_delay(df: &DataFrame, col_name: &str, periods: usize) -> PolarsResult<Series> {
    df.clone()
        .lazy()
        .with_columns([col(col_name).shift(periods as i64).over([col("symbol")]).alias("__factor_value")])
        .collect()?
        .column("__factor_value")
        .cloned()
}

pub fn ts_delta(df: &DataFrame, col_name: &str, periods: usize) -> PolarsResult<Series> {
    df.clone()
        .lazy()
        .with_columns([
            (col(col_name) - col(col_name).shift(periods as i64).over([col("symbol")]))
                .alias("__factor_value"),
        ])
        .collect()?
        .column("__factor_value")
        .cloned()
}
```

Create `src/factor/catalog.rs`:

```rust
use polars::prelude::*;

use crate::core::{QuantixError, Result};
use crate::factor::dataset::FactorDataset;
use crate::factor::operators::{cs_rank, ts_delta, ts_delay};
use crate::factor::types::{FactorCategory, FactorComputeResult, FactorMeta, MissingPolicy};

pub struct FactorCatalog {
    metas: Vec<FactorMeta>,
}

pub fn builtin_factor_catalog() -> FactorCatalog {
    FactorCatalog {
        metas: vec![
            FactorMeta {
                id: "rank_close".to_string(),
                category: FactorCategory::Technical,
                description: "Cross-sectional rank of close within each date".to_string(),
                author: Some("quantix".to_string()),
                source: Some("p1".to_string()),
                refresh_frequency: Some("daily".to_string()),
                required_fields: vec!["close".to_string()],
                missing_policy: MissingPolicy::KeepNull,
            },
            FactorMeta {
                id: "delay_close_1".to_string(),
                category: FactorCategory::Technical,
                description: "One-bar per-symbol delayed close".to_string(),
                author: Some("quantix".to_string()),
                source: Some("p1".to_string()),
                refresh_frequency: Some("daily".to_string()),
                required_fields: vec!["close".to_string()],
                missing_policy: MissingPolicy::KeepNull,
            },
            FactorMeta {
                id: "delta_close_1".to_string(),
                category: FactorCategory::Technical,
                description: "One-bar per-symbol close delta".to_string(),
                author: Some("quantix".to_string()),
                source: Some("p1".to_string()),
                refresh_frequency: Some("daily".to_string()),
                required_fields: vec!["close".to_string()],
                missing_policy: MissingPolicy::KeepNull,
            },
        ],
    }
}

impl FactorCatalog {
    pub fn list(&self) -> &[FactorMeta] {
        &self.metas
    }

    pub fn compute(&self, factor_id: &str, dataset: &FactorDataset) -> Result<FactorComputeResult> {
        let values = match factor_id {
            "rank_close" => cs_rank(dataset.frame(), "close"),
            "delay_close_1" => ts_delay(dataset.frame(), "close", 1),
            "delta_close_1" => ts_delta(dataset.frame(), "close", 1),
            other => {
                return Err(QuantixError::Unsupported(format!(
                    "unknown factor `{}`",
                    other
                )));
            }
        }
        .map_err(|e| QuantixError::DataParse(format!("factor `{}` compute failed: {}", factor_id, e)))?;

        let mut frame = dataset
            .frame()
            .select(["date", "symbol"])
            .map_err(|e| QuantixError::DataParse(format!("factor output select failed: {}", e)))?;
        let mut values = values.clone();
        values.rename("value".into());
        frame
            .with_column(values)
            .map_err(|e| QuantixError::DataParse(format!("factor output value attach failed: {}", e)))?;

        Ok(FactorComputeResult {
            factor_id: factor_id.to_string(),
            frame,
        })
    }
}
```

Modify `src/factor/mod.rs`:

```rust
pub mod catalog;
pub mod check;
pub mod dataset;
pub mod loader;
pub mod operators;
pub mod types;

pub use catalog::{FactorCatalog, builtin_factor_catalog};
pub use dataset::FactorDataset;
pub use loader::FactorDataLoader;
pub use operators::{cs_rank, ts_delta, ts_delay};
pub use types::{
    FactorCategory, FactorComputeRequest, FactorComputeResult, FactorLoadRequest, FactorMeta,
    MissingPolicy,
};
```

- [ ] **Step 4: Run tests and verify pass**

Run:

```bash
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test --test factor_pipeline_test -- --nocapture
```

Expected: PASS. If Polars rank/over syntax differs in 0.43, preserve the public signatures and test behavior while adjusting internals.

- [ ] **Step 5: Commit**

```bash
git add src/factor/mod.rs src/factor/operators.rs src/factor/catalog.rs tests/factor_pipeline_test.rs
git commit -m "feat: add factor operators and catalog"
```

## Task 5: Add Factor Export and CLI Handler

**Files:**
- Create: `src/factor/export.rs`
- Create: `src/cli/handlers/factor.rs`
- Modify: `src/factor/mod.rs`
- Modify: `src/cli/handlers/mod.rs`
- Modify: `src/cli/commands/mod.rs`

- [ ] **Step 1: Add failing handler smoke tests**

Add handler-specific tests only if the repo's existing handler test harness can be reused without expanding scope. Otherwise use parser tests from Task 1 and a direct export test in `tests/factor_pipeline_test.rs`.

Append this export test to `tests/factor_pipeline_test.rs`:

```rust
use quantix_cli::factor::{factor_result_to_csv_string, factor_result_to_json_string};

#[tokio::test]
async fn factor_result_exports_csv_and_json_strings() {
    let loader = MockFactorLoader {
        frame: mock_factor_frame(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        required_fields: vec!["close".to_string()],
    };
    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    let result = builtin_factor_catalog()
        .compute("rank_close", &dataset)
        .unwrap();

    let csv = factor_result_to_csv_string(&result).unwrap();
    assert!(csv.contains("date,symbol,value"));

    let json = factor_result_to_json_string(&result).unwrap();
    assert!(json.contains("rank_close"));
}
```

- [ ] **Step 2: Run test and verify failure**

Run:

```bash
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test --test factor_pipeline_test factor_result_exports_csv_and_json_strings -- --nocapture
```

Expected: FAIL because export helpers do not exist.

- [ ] **Step 3: Implement export helpers and handler**

Create `src/factor/export.rs`:

```rust
use crate::core::{QuantixError, Result};
use crate::factor::types::FactorComputeResult;

pub fn factor_result_to_csv_string(result: &FactorComputeResult) -> Result<String> {
    let mut bytes = Vec::new();
    CsvWriter::new(&mut bytes)
        .finish(&mut result.frame.clone())
        .map_err(|e| QuantixError::Other(format!("factor csv export failed: {}", e)))?;
    String::from_utf8(bytes)
        .map_err(|e| QuantixError::Other(format!("factor csv export produced invalid utf8: {}", e)))
}

pub fn factor_result_to_json_string(result: &FactorComputeResult) -> Result<String> {
    let rows = result.frame.height();
    Ok(format!(
        "{{\"factor_id\":\"{}\",\"rows\":{},\"columns\":{:?}}}",
        result.factor_id,
        rows,
        result.frame.get_column_names()
    ))
}
```

Make sure `src/factor/export.rs` imports `polars::prelude::CsvWriter`.

Create `src/cli/handlers/factor.rs`:

```rust
use super::*;

pub async fn run_factor_command(cmd: FactorCommands) -> Result<()> {
    match cmd {
        FactorCommands::List { category, verbose } => {
            let catalog = crate::factor::builtin_factor_catalog();
            for meta in catalog.list() {
                if let Some(category_filter) = &category {
                    if format!("{:?}", meta.category).to_lowercase()
                        != category_filter.to_lowercase()
                    {
                        continue;
                    }
                }
                if verbose {
                    println!(
                        "{}\t{:?}\t{}\tfields={:?}\tmissing={:?}",
                        meta.id, meta.category, meta.description, meta.required_fields, meta.missing_policy
                    );
                } else {
                    println!("{}\t{:?}\t{}", meta.id, meta.category, meta.description);
                }
            }
            Ok(())
        }
        FactorCommands::Compute { .. } => Err(QuantixError::Unsupported(
            "factor compute requires a data loader; P1 CLI data-loader wiring is implemented in the next step".to_string(),
        )),
    }
}
```

Modify `src/cli/handlers/mod.rs`:

```rust
FactorCommands,
```

Add:

```rust
mod factor;
pub use self::factor::run_factor_command;
```

Modify `src/cli/commands/mod.rs` routing:

```rust
Commands::Factor(cmd) => {
    handlers::run_factor_command(cmd).await?;
}
```

Modify `src/factor/mod.rs`:

```rust
pub mod export;
pub use export::{factor_result_to_csv_string, factor_result_to_json_string};
```

- [ ] **Step 4: Run tests and verify pass**

Run:

```bash
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test --test factor_pipeline_test factor_result_exports_csv_and_json_strings -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test cli::tests::factor -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/factor/mod.rs src/factor/export.rs src/cli/handlers/factor.rs src/cli/handlers/mod.rs src/cli/commands/mod.rs tests/factor_pipeline_test.rs
git commit -m "feat: add factor export and list handler"
```

## Task 6: Wire Compute With a First-Slice CSV Loader

**Files:**
- Modify: `src/factor/loader.rs`
- Modify: `src/cli/handlers/factor.rs`
- Test: `tests/factor_pipeline_test.rs`

- [ ] **Step 1: Add a direct compute pipeline integration test with 3 stocks and 10 days**

Replace or extend `mock_factor_frame()` so it has 3 stocks and 10 dates, with at least one null close value. Keep expected assertions based on row count and stable output schema.

Add:

```rust
#[tokio::test]
async fn p1_pipeline_computes_rank_close_with_mock_loader() {
    let loader = MockFactorLoader {
        frame: mock_factor_frame(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
        required_fields: vec!["close".to_string()],
    };

    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    dataset.ensure_time_aligned().unwrap();
    dataset.validate_no_lookahead_basic().unwrap();

    let result = builtin_factor_catalog()
        .compute("rank_close", &dataset)
        .unwrap();
    assert_eq!(result.factor_id, "rank_close");
    assert_eq!(result.frame.height(), 30);
    assert_eq!(result.frame.get_column_names(), vec!["date", "symbol", "value"]);
}
```

- [ ] **Step 2: Run test and verify pass or targeted failure**

Run:

```bash
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test --test factor_pipeline_test p1_pipeline_computes_rank_close_with_mock_loader -- --nocapture
```

Expected: PASS if prior tasks already support it, or FAIL only on the mock data size/null handling that this task resolves.

- [ ] **Step 3: Add a P1 CSV loader and wire `factor compute`**

Implement P1 CLI compute with a local CSV loader. The command must never silently use generated or mock market data.

In `src/factor/loader.rs`, add a `CsvFactorDataLoader` that reads a local CSV file with this schema:

```text
date,symbol,open,high,low,close,volume
2026-01-01,000001.SZ,10.0,10.5,9.8,10.2,1000
```

The loader should:

- read the file path supplied by `--input`
- parse into a Polars `DataFrame`
- filter to requested `symbols`
- filter `date >= start` and `date <= end`
- return the long-form frame to `FactorDataset::from_loader`

In `src/cli/handlers/factor.rs`, add:

```rust
fn parse_factor_date(value: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|e| QuantixError::DataParse(format!("invalid factor date `{}`: {}", value, e)))
}
```

`Compute` should:

- parse dates
- build `FactorLoadRequest`
- instantiate `CsvFactorDataLoader` from `input`
- build `FactorDataset`
- run checks unless `skip_checks`
- compute every requested factor via `builtin_factor_catalog`
- print table output for `FactorOutputFormat::Table`
- require `--output` for csv/json/parquet

- [ ] **Step 4: Run CLI smoke commands**

Run:

```bash
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo run -- factor list
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo run -- factor compute --input /tmp/quantix-factor-p1-input.csv --factor rank_close --symbol 000001.SZ --symbol 600000.SH --start 2026-01-01 --end 2026-01-10
```

Expected:

- first command prints `rank_close`, `delay_close_1`, and `delta_close_1`
- second command prints table-shaped factor output and exits successfully after `/tmp/quantix-factor-p1-input.csv` is created with the schema above

- [ ] **Step 5: Commit**

```bash
git add src/cli/handlers/factor.rs tests/factor_pipeline_test.rs
git commit -m "feat: wire factor compute first slice"
```

## Task 7: Documentation, FUNCTION_TREE, and Final Verification

**Files:**
- Modify: `FUNCTION_TREE.md`
- Optionally modify: `README.md` or `docs/CLI_COMMAND_MANUAL.html` only if repo policy or current workflow requires documenting newly shipped CLI commands in the same slice

- [ ] **Step 1: Update FUNCTION_TREE**

Add a new factor/research section or extend the analysis section with:

```text
├── 🧮 因子研究 (factor/) [部分实现]
│   ├── FactorDataset - Polars long-form 因子面板 [已实现]
│   ├── FactorDataLoader trait - 异步数据加载边界 [已实现]
│   ├── 因子算子 (operators) [部分实现]
│   │   ├── cs_rank [已实现]
│   │   ├── ts_delay [已实现]
│   │   └── ts_delta [已实现]
│   ├── 因子目录 (catalog) [部分实现]
│   │   ├── rank_close [已实现]
│   │   ├── delay_close_1 [已实现]
│   │   └── delta_close_1 [已实现]
│   ├── 因子检查 (check) [部分实现]
│   │   ├── required columns / dtype / uniqueness [已实现]
│   │   └── basic no-lookahead structure check [已实现]
│   ├── Alpha101 / Alpha191 [待实现]
│   ├── IC/IR / correlation / neutralization [待实现]
│   └── layered factor backtest [待实现]
```

- [ ] **Step 2: Run focused tests**

Run:

```bash
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test --test factor_pipeline_test -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test cli::tests::factor -- --nocapture
```

Expected: PASS.

- [ ] **Step 3: Run broader compile/test gate**

Run:

```bash
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo test --lib -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-factor-p1 cargo check
```

Expected: PASS. If unrelated pre-existing failures appear, record them with exact test names and error summaries.

- [ ] **Step 4: Run GitNexus change detection**

Run:

```bash
gitnexus_detect_changes({scope: "all"})
```

Expected: changed symbols are limited to factor, factor CLI, tests, and FUNCTION_TREE documentation.

- [ ] **Step 5: Commit**

```bash
git add FUNCTION_TREE.md
git commit -m "docs: record factor first slice"
```

## Self-Review Checklist

- Spec coverage:
  - dedicated `src/factor/` subsystem: Tasks 2-6
  - dtype table and long-form dataset: Task 3
  - async loader and sync compute boundary: Task 2 and Task 3
  - `cs_rank`, `ts_delay`, `ts_delta`: Task 4
  - first-slice checks: Task 3
  - `factor list` / `factor compute`: Task 1, Task 5, Task 6
  - mock-data integration test: Task 3, Task 4, Task 6
  - FUNCTION_TREE update: Task 7
- Type consistency:
  - `FactorOutputFormat` is CLI-only
  - file export helpers live under `factor::export`
  - factor APIs use `crate::core::Result<T>`
  - operator APIs return `PolarsResult<Series>` aligned to input rows
- Execution note:
  - Before editing Rust symbols, run GitNexus impact analysis for each modified existing symbol such as `Commands`, `Cli::run`, and `run_factor_command` once it exists.
