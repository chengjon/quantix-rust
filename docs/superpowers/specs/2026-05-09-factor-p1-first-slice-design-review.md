# Review: 2026-05-09-factor-p1-first-slice-design.md

**Type**: .md / spec | **Perspective**: auto (completeness + architecture + feasibility + consistency) | **Date**: 2026-05-09 | **Reviewer**: Claude

---

## Executive Summary

A well-structured first-slice design for a Rust-native factor research module. The document correctly identifies isolation from existing indicator code, specifies a clear async/sync boundary, and lists concrete files, types, and acceptance criteria. The main gap is missing dtype specifications for the Polars dataset schema and absent operator function signatures -- both are implementation-blocking. Existing codebase dependencies (polars, async-trait, chrono) and CLI conventions all align with the proposal.

## Document Metadata

| Field | Value |
|-------|-------|
| Source | docs/superpowers/specs/2026-05-09-factor-p1-first-slice-design.md |
| File Type | .md |
| Doc Type | spec |
| Sections | 13 |
| Referenced Files | 6 found / 0 missing (4 existing to update, 1 existing to reuse, 5 new expected absent) |
| Referenced Symbols | 0 found / 8 missing (all are proposed new types, expected absent) |

## Evidence Verification

### Files Referenced

| File | Exists? | Location |
|------|---------|----------|
| `src/lib.rs` | yes | `/opt/claude/quantix-rust/src/lib.rs` |
| `src/cli/commands/mod.rs` | yes | `/opt/claude/quantix-rust/src/cli/commands/mod.rs` |
| `src/cli/handlers/mod.rs` | yes | `/opt/claude/quantix-rust/src/cli/handlers/mod.rs` |
| `src/analysis/polars_adapter.rs` | yes | `/opt/claude/quantix-rust/src/analysis/polars_adapter.rs` |
| `src/factor/` | no (expected new) | -- |
| `src/cli/commands/factor.rs` | no (expected new) | -- |
| `src/cli/handlers/factor.rs` | no (expected new) | -- |
| `src/cli/tests/factor.rs` | no (expected new) | -- |
| `tests/factor_pipeline_test.rs` | no (expected new) | -- |

### Functions/Classes Referenced

| Symbol | Found? | Location |
|--------|--------|----------|
| `FactorCategory` | no (proposed) | -- |
| `FactorMeta` | no (proposed) | -- |
| `MissingPolicy` | no (proposed) | -- |
| `FactorLoadRequest` | no (proposed) | -- |
| `FactorComputeRequest` | no (proposed) | -- |
| `FactorComputeResult` | no (proposed) | -- |
| `FactorDataLoader` | no (proposed) | -- |
| `cs_rank` / `ts_delay` / `ts_delta` | no (proposed) | -- |

### Claims Verified

| Claim | Status | Evidence |
|-------|--------|----------|
| polars crate available | confirmed | `Cargo.toml:66`: `polars = { version = "0.43", features = ["lazy", "rolling_window", "dtype-datetime"] }` |
| async-trait crate available | confirmed | `Cargo.toml:90`: `async-trait = "0.1"` |
| chrono / NaiveDate available | confirmed | `Cargo.toml:49`: `chrono = { version = "0.4", features = ["serde"] }` |
| `init_polars` exists for reuse | confirmed | `src/analysis/polars_adapter.rs:15`: `pub fn init_polars() -> Result<()>` |
| `src/io/exporter.rs` has CSV/JSON/Parquet | confirmed | `src/io/exporter.rs:14-18`: `ExportFormat` enum with CSV, JSON, Parquet variants |
| existing codebase uses `async_trait` | confirmed | Found in 10 files: execution, risk, monitoring, handlers, etc. |
| existing CLI pattern uses clap Subcommand enum | confirmed | `src/cli/commands/mod.rs:57`: `Commands` enum with `#[derive(clap::Subcommand)]` |
| `DataFrame` used in codebase | partially | Only `src/analysis/polars_adapter.rs` references `DataFrame`; existing data uses Vec-based `BatchKlineData` |
| `ExportFormat` includes `table` | contradicted | `src/io/exporter.rs:14-18` only has CSV, JSON, Parquet -- no `Table` variant |

## Checklist Results

### Architecture

| # | Check | Result | Notes |
|---|-------|--------|-------|
| A1 | Component boundaries | PASS | Clear module layout with 7 files, each with explicit responsibility (lines 70-122) |
| A2 | Data flow | PASS | Async loader -> sync compute boundary documented with design rationale (lines 127-145) |
| A3 | Coupling | PASS | Factor module isolated; reuse/non-reuse rules explicit (lines 371-384) |
| A4 | Interface contracts | FAIL | Loader trait shown (line 132-135) but operator function signatures not specified |
| A5 | Scalability | N/A | First slice, explicitly scoped small |
| A6 | Terminology consistency | PASS | ts_*/cs_* naming consistent throughout; long-form, cross-sectional, time-series used uniformly |
| A7 | Backward compatibility | PASS | New isolated module; no existing behavior modified |
| A8 | Implementation surface precision | PASS | Exact files listed for creation (lines 72-90) and update (lines 94-96) |
| A9 | Named entities verified | PASS | All 4 existing files confirmed present; 5 new files correctly absent |

### Completeness

| # | Check | Result | Notes |
|---|-------|--------|-------|
| C1 | Required sections | PASS | Goal, Scope, Approach, Architecture, Data Contract, Core Types, Registry, Validation, CLI, Testing, Reuse, Acceptance, Follow-Up |
| C2 | Edge cases | FAIL | Missing-value policy defined but dtype edge cases (e.g., integer vs float close, null handling in Polars) not addressed |
| C3 | Implicit assumptions | FAIL | Three unstated: (1) which Polars dtypes for each column, (2) which error type for `Result<DataFrame>`, (3) `table` format is CLI-only vs export concern |
| C4 | Acceptance criteria | PASS | 8 clear, testable criteria (lines 388-397) |
| C5 | Missing roles/stakeholders | N/A | Solo/small team project |

### Consistency

| # | Check | Result | Notes |
|---|-------|--------|-------|
| N1 | Terminology | PASS | Same terms used throughout; no synonyms for key concepts |
| N2 | Naming conventions | PASS | Rust snake_case for functions, CamelCase for types; matches codebase |
| N3 | Formatting | PASS | Uniform heading hierarchy, code blocks, and list style |
| N4 | Cross-references | PASS | File paths are valid; links resolve to existing files |
| N5 | Style consistency | PASS | Technical but readable; uniform tone |

### Feasibility

| # | Check | Result | Notes |
|---|-------|--------|-------|
| F1 | Technical risk | FAIL | Polars DataFrame-first approach is a departure from existing Vec-based patterns; risk not acknowledged |
| F2 | Dependency availability | PASS | polars 0.43, async-trait 0.1, chrono 0.4 all in Cargo.toml |
| F3 | Timeline realism | N/A | No timeline given |
| F4 | Resource constraints | N/A | Not specified |
| F5 | Rollback plan | PASS | New isolated module under `src/factor/`; can be removed without affecting existing code |

## Findings

### High Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| 1 | Dataset Schema (lines 148-171) | Column dtypes not specified. Doc says "required columns must exist with compatible dtypes" (line 169) but never defines what "compatible" means for each column. The codebase uses `f64` for prices in `BatchKlineData` (`src/analysis/polars_adapter.rs:33-39`) but `Decimal` in indicator calculations (`src/analysis/indicator_registry.rs:44`). Polars is dtype-strict; implementers cannot proceed without this. | Implementation-blocking. Polars operations like `cs_rank` and `ts_delay` produce different results on Float64 vs String columns. Incorrect dtypes will cause runtime panics or silent type coercion. | Codebase check: `BatchKlineData` uses `Vec<f64>` for OHLCV, `Vec<i64>` for volume. Doc check: line 169 says "compatible dtypes" but no dtype table follows. The schema section lists column names only. | Add a dtype table: `date -> Date`, `symbol -> Utf8/String`, `open/high/low/close -> Float64`, `volume -> Int64`, `amount -> Float64 (optional)`. State whether Polars `Date` or `Datetime` is used. |

### Medium Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| 2 | Operator Semantics (lines 172-188) | Operator function signatures not shown. The doc describes `ts_delay`, `ts_delta`, `cs_rank` semantics and gives expression examples (lines 184-187) but never shows Rust function signatures. The loader trait signature is shown (line 134), but operators are equally important since they are the core computation surface. | Implementers must guess function signatures. Likely to cause inconsistency: one dev might write `fn ts_delay(col: &Series, period: usize) -> Series` while another writes `fn ts_delay(df: &DataFrame, col: &str, period: usize) -> DataFrame`. | Codebase check: no existing operator signatures to reference. Doc check: lines 110-112 describe `operators.rs` as "reusable Polars-first operator helpers" but the Core Types section (lines 190-245) only shows metadata/request/result types, not operator signatures. | Add a function signature block for each operator, e.g.: `pub fn ts_delay(df: &DataFrame, col: &str, period: usize) -> PolarsResult<Series>` or whatever the chosen API surface is. |
| 3 | CLI Design (lines 316-327) | `table` format undefined. CLI proposes `--format <table\|csv\|json\|parquet>` with default `table` (line 323). The `export.rs` description also lists "table/CSV/JSON/Parquet" (line 117). But existing `src/io/exporter.rs:14-18` `ExportFormat` only has CSV, JSON, Parquet. The doc does not clarify whether `table` is a CLI display mode (like `prettytable`) or an export format. | If implementers treat `table` as an `ExportFormat` variant, they will modify the shared `io::exporter` module. If it is CLI-only, it should not appear in `export.rs` scope. | Codebase check: `src/io/exporter.rs:14-18` has `enum ExportFormat { CSV, JSON, Parquet }` -- no Table variant. Doc check: line 117 lists "table/CSV/JSON/Parquet" as `export.rs` responsibility; line 323 lists `table` as a CLI format option. Neither section clarifies the distinction. | State explicitly: "`table` is a CLI display format rendered via `comfy-table`/`prettytable` and is not part of `ExportFormat`. The `export.rs` module handles CSV/JSON/Parquet file output only." |
| 4 | Data Contract (line 134) | Error type unspecified. `FactorDataLoader::load_bars` returns `Result<DataFrame>` but the error type is not specified. The codebase uses `crate::core::QuantixError` as its canonical error type. | Implementers will either (a) use `QuantixError` directly, (b) create a `FactorError`, or (c) use `polars::error::PolarsError`. This decision affects the error chain and how factor errors propagate to CLI. | Codebase check: `src/core/mod.rs` defines `QuantixError` and `Result`; used throughout handlers and services. Doc check: line 134 uses bare `Result<DataFrame>` with no import or type alias shown. | Specify: `type FactorResult<T> = std::result::Result<T, QuantixError>` or add a `FactorError` variant to `QuantixError`. |

### Low Issues

| # | Section | Issue | Evidence | Recommendation |
|---|---------|-------|----------|----------------|
| 5 | Core Types (line 250) | `value` column dtype in `FactorComputeResult` not specified. The doc says compute results contain `date`, `symbol`, `value` columns but does not specify that `value` should be `Float64`. | Codebase check: factor values are numeric by nature; `cs_rank` and `ts_delta` both produce floats. Doc check: line 250 only says "value" without dtype. | Add: `value -> Float64`. |
| 6 | Testing (line 89) | Test location ambiguity. Spec says `src/cli/tests/factor.rs` for CLI parsing tests, but the codebase also has `src/cli/handlers/tests/` for handler-level tests. The spec does not clarify where handler tests for `factor` go. | Codebase check: 15 test files in `src/cli/tests/` (e.g., `execution.rs`), plus `src/cli/handlers/tests/` with handler-specific tests. Doc check: line 89 lists `src/cli/tests/factor.rs` only. | Add a note: "Handler tests for factor commands go in `src/cli/handlers/tests/` following the existing pattern. `src/cli/tests/factor.rs` covers CLI parsing only." |

## Strengths

- **Clear scope boundary**: The In Scope / Out of Scope sections (lines 18-49) are precise. Alpha101, IC/IR, neutralization, and backtest are explicitly excluded, preventing scope creep.
- **Async/sync boundary design**: The loader-is-async, engine-is-sync rule (lines 127-145) with explicit rationale is a strong architectural decision that will simplify all downstream operator code.
- **Reuse/non-reuse rules**: Lines 371-384 explicitly call out what to reuse (`polars_adapter` init patterns) and what not to reuse (loop-heavy indicators, `risk::volatility`), with named files. This prevents the common mistake of copying inappropriate patterns.
- **Forward-compatible CLI shape**: Allowing repeated `--factor` flags even when initial execution is sequential (line 267) is a good API design that avoids CLI breakage when batching is added later.
- **Validation in first slice**: `check.rs` is included from the start (lines 269-290), not deferred. This prevents the common pattern where validation is an afterthought.
- **Acceptance criteria are testable**: All 8 criteria (lines 388-397) can be objectively verified -- `quantix factor list` works, integration test passes, etc.

## Detailed Recommendations

1. **Add a dtype table** after the column list (after line 170). This is the single highest-impact addition. Without it, each implementer will make independent dtype choices that may conflict at integration time.

2. **Add operator function signatures** in the Operator Semantics section (after line 187). Even one example signature establishes the pattern:
   ```rust
   pub fn ts_delay(df: &DataFrame, col: &str, period: usize) -> PolarsResult<Series>;
   pub fn ts_delta(df: &DataFrame, col: &str, period: usize) -> PolarsResult<Series>;
   pub fn cs_rank(df: &DataFrame, col: &str) -> PolarsResult<Series>;
   ```

3. **Clarify `table` format scope** in both the CLI Design and export.rs responsibility sections. One sentence in each location is sufficient.

4. **Specify the error type** in the Data Contract section. Either reference `QuantixError` explicitly or define a type alias.

5. **Acknowledge the DataFrame paradigm shift** in the Chosen Approach or Reuse sections. The existing codebase is Vec-based (`BatchKlineData`); the factor module is Polars-DataFrame-first. This is a deliberate and correct decision, but stating it explicitly helps future readers understand why the patterns differ.

## Scoring

| Dimension | Score (1-5) | Evidence |
|-----------|-------------|----------|
| Technical Accuracy | 4 | All crate dependencies verified; async/sync boundary well-reasoned; loader contract correct |
| Completeness | 3 | Missing dtype specs and operator signatures block implementation; error type unspecified |
| Codebase Alignment | 5 | CLI pattern matches existing Commands enum; reuse targets exist; new module follows `src/lib.rs` registration pattern |
| Actionability | 4 | Files, types, and acceptance criteria are concrete; only dtype and signature gaps reduce score |
| Terminology Consistency | 5 | ts_*/cs_* used consistently; long-form, cross-sectional defined and reused throughout |
| **Overall** | **4.2** | |

## Verdict

APPROVE_WITH_NOTES

The spec is well-structured with correct codebase alignment and clear scope boundaries. The four medium issues (dtype table, operator signatures, `table` format scope, error type) should be addressed before implementation begins -- they are cheap to fix now and expensive to retrofit later. None of the findings are architectural defects; they are specification gaps that will cause ambiguity during implementation.
