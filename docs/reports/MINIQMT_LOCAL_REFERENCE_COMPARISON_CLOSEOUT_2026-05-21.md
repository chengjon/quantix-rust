# miniQMT Local Reference Comparison Closeout

Date: 2026-05-21

Graphiti backfill required

Graphiti episode `0f64dea8-1f9b-4693-a66c-b71ef1b1515c` was queued for `quantix_rust_debug` but ingest failed with `Request timed out.`

## Summary

Quantix now supports an opt-in local reference artifact comparison for the miniQMT controlled evidence path.

`quantix import market-manifest` accepts:

```text
--comparison-reference-artifact <local-path-or-file-uri>
```

When this flag is used together with `--verify-artifact-file`, Quantix reads the local reference artifact, computes its SHA-256 hash, samples its local Parquet payload, compares row count and samples against the miniQMT artifact, and writes the comparison object into the raw regression report and miniQMT-shaped evidence.

Successful comparison removes the default `double_read_comparison_not_yet_implemented` warning and records:

```text
double_read_comparison_performed
double_read_row_count_matched
double_read_sample_symbols_matched
double_read_sample_dates_matched
```

The comparison summary becomes:

```text
local_reference_artifact_matched
```

## Boundary

This is an opt-in local reference artifact comparison. It is not a ClickHouse/source-of-truth double-read and does not write ClickHouse.

Still not implemented:

- ClickHouse/source-of-truth double-read comparison
- ClickHouse shadow import
- production ClickHouse writes
- Quantix ownership of miniQMT registry apply, registry mutation, or maturity promotion

Authoritative feature status remains `FUNCTION_TREE.md`.

## Verification

- RED: `cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_import_handler_test market_manifest_cli_records_local_reference_double_read_comparison`
  - Failed as expected because `--comparison-reference-artifact` was not recognized by the CLI.
- GREEN: the same command passed after adding the opt-in CLI parameter and local reference comparison report path.
- `cargo fmt --manifest-path /opt/claude/quantix-rust/Cargo.toml --check`: passed.
- `cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test --test miniqmt_market_import_handler_test`: passed, 27 + 6 tests.
- `cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test repo_hygiene_test --quiet`: passed, 32 tests.
- `git -C /opt/claude/quantix-rust diff --check -- <miniQMT file scope>`: passed.
- `cargo clippy --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test --test miniqmt_market_import_handler_test --quiet`: exited 0 with existing warning output.

Existing project warnings remain unrelated to this miniQMT slice.

## GitNexus

- `ResolvedMarketArtifact` impact: LOW.
- `run_import_market_manifest` impact: CRITICAL because it is part of the broad CLI import handler and upstream `run` flow. The implemented change is opt-in only and preserves the default command behavior.
- `QuantixRegressionContext` and some new miniQMT symbols were not found in the current index.
- `detect_changes(scope=unstaged)` reported CRITICAL for the shared dirty worktree: 143 changed files, 1515 changed symbols, 140 affected symbols. This is a whole-workspace risk indicator, not a clean scope report for this miniQMT slice.
