# miniQMT Direct ClickHouse Read-Only Comparison Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

本文是实施计划，不是功能状态注册表；当前功能状态、证据和边界以根目录 `FUNCTION_TREE.md` 为准。

**Goal:** Add an explicit opt-in ClickHouse read-only double-read comparison path for `quantix import market-manifest`.

**Architecture:** The CLI will keep its default dry-run behavior unchanged. When the operator passes ClickHouse comparison flags, Quantix will query row-count and sample identity from ClickHouse using `ClickHouseClient::query_json`, then convert that read-only result into the existing `QuantixRegressionComparison` block for report/evidence generation.

**Tech Stack:** Rust, Clap, existing `ClickHouseClient::query_json`, serde JSON, wiremock integration tests, existing miniQMT market regression report/evidence types.

---

### Task 1: CLI RED Test

**Files:**
- Modify: `tests/miniqmt_market_import_handler_test.rs`

- [ ] **Step 1: Write the failing test**

Add `market_manifest_cli_records_direct_clickhouse_read_only_comparison`.

The test should:
- create a local Parquet artifact with 3 rows;
- create a miniQMT manifest pointing at that artifact;
- start a `wiremock::MockServer`;
- make the mock server return JSONEachRow-compatible responses for row-count, symbols, and dates;
- run `quantix import market-manifest` with:
  - `--verify-artifact-file`
  - `--comparison-clickhouse-url <mock-server-uri>`
  - `--comparison-clickhouse-database quantix`
  - `--comparison-clickhouse-table miniqmt_shadow_kline_daily`
  - `--comparison-clickhouse-dataset-version-column dataset_version`
  - `--comparison-clickhouse-symbol-column symbol`
  - `--comparison-clickhouse-date-column date`
  - report/evidence outputs;
- assert report comparison type is `direct_clickhouse_read_only`;
- assert comparison summary is `direct_clickhouse_read_only_matched`;
- assert writes remain false.

- [ ] **Step 2: Run RED**

Run:

```bash
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_import_handler_test market_manifest_cli_records_direct_clickhouse_read_only_comparison
```

Expected: FAIL because the CLI does not recognize the ClickHouse comparison flags.

### Task 2: Minimal Implementation

**Files:**
- Modify: `src/cli/commands/info.rs`
- Modify: `src/cli/handlers/import.rs`
- Modify: `src/miniqmt_market.rs`
- Modify: `tests/miniqmt_market_import_handler_test.rs`

- [ ] **Step 1: Add Clap fields**

Add optional fields to `ImportCommands::MarketManifest`:
- `comparison_clickhouse_url`
- `comparison_clickhouse_database`
- `comparison_clickhouse_user`
- `comparison_clickhouse_password`
- `comparison_clickhouse_table`
- `comparison_clickhouse_dataset_version_column`
- `comparison_clickhouse_symbol_column`
- `comparison_clickhouse_date_column`

Defaults:
- user: `default`
- password: empty
- database: `quantix`
- dataset version column: `dataset_version`
- symbol column: `symbol`
- date column: `date`

- [ ] **Step 2: Add comparison options struct**

Add an import-handler local struct to group the ClickHouse options so `run_import_market_manifest` does not grow further.

- [ ] **Step 3: Add read-only summary generation**

Add a function that builds three read-only `SELECT` statements:
- row count by explicit dataset version column;
- sorted distinct sample symbols;
- sorted distinct sample dates.

Reject unsafe identifiers unless every table/column segment is ASCII alphanumeric or underscore, with dot allowed only between table segments.

- [ ] **Step 4: Convert query results into comparison**

Add `QuantixRegressionComparison::from_clickhouse_read_only_summary` or equivalent. It must set:
- `comparison_type = "direct_clickhouse_read_only"`
- `reference_source_system = "clickhouse"`
- `reference_source_uri = "clickhouse://<database>.<table>?dataset_version=<dataset_version>"`
- matched flags based on row-count/samples.

- [ ] **Step 5: Preserve boundaries**

Reject combinations where more than one comparison source is provided:
- local reference artifact;
- source-of-truth summary JSON;
- direct ClickHouse read-only comparison.

Require `--verify-artifact-file` for direct ClickHouse comparison.

Never set `writes_performed=true`.

### Task 3: Docs And Gates

**Files:**
- Modify: `FUNCTION_TREE.md`
- Modify: `CHANGELOG.md`
- Create: `docs/reports/MINIQMT_DIRECT_CLICKHOUSE_READ_ONLY_COMPARISON_CLOSEOUT_2026-05-21.md`

- [ ] **Step 1: Update FUNCTION_TREE**

Move direct ClickHouse read-only comparison from `[已设计/待实现]` to `[部分实现]`.

Keep ClickHouse shadow import as `[已设计/待实现]`.

- [ ] **Step 2: Update CHANGELOG**

Add the direct ClickHouse read-only comparison to the existing 2026-05-21 miniQMT entry.

- [ ] **Step 3: Add closeout**

Record RED/GREEN, gates, GitNexus impact, and Graphiti fallback if needed.

- [ ] **Step 4: Run gates**

Run:

```bash
cargo fmt --manifest-path /opt/claude/quantix-rust/Cargo.toml --check
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test --test miniqmt_market_import_handler_test
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test repo_hygiene_test --quiet
cargo clippy --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test --test miniqmt_market_import_handler_test --quiet
```

Expected: all commands exit 0; existing non-fatal warnings may remain.
