# miniQMT Payload Row Count Verification Closeout

Date: 2026-05-20

Graphiti backfill required

Graphiti episode `42df5aa2-7913-41ba-b71e-ed8795ad177e` was queued for `quantix_rust_debug` but ingest failed with `Request timed out.`

## Summary

Quantix now records local Parquet payload row-count evidence for the miniQMT controlled evidence path.

When `quantix import market-manifest --verify-artifact-file` resolves a readable local Parquet artifact, `src/miniqmt_market.rs` reads the Parquet metadata row count and stores it as `computed_row_count` on the resolved artifact and regression artifact.

The raw `QuantixRegressionReport` compares that computed value with the manifest artifact `row_count`:

```text
artifact_payload_row_count_verified
```

is added when the values match. If they differ, the report fails closed with:

```text
artifact_payload_row_count_mismatch
```

This is payload-shape verification, not source-of-truth double-read. It does not write ClickHouse, does not read miniQMT raw/candidate/job files, and does not make Quantix the miniQMT registry owner.

## Files Updated

- `src/miniqmt_market.rs`
- `tests/miniqmt_market_manifest_test.rs`
- `FUNCTION_TREE.md`
- `README.md`
- `docs/superpowers/specs/2026-05-18-miniqmt-controlled-evidence-alignment-spec.md`

## Verification

- RED: `cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test`
  - Failed as expected because `computed_row_count` did not yet exist on `ResolvedMarketArtifact` / `QuantixRegressionArtifact`.
- GREEN: `cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test quantix_regression_report_fails_closed_when_parquet_payload_row_count_differs`
  - Passed.
- Companion check: `cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test quantix_regression_report_samples_symbols_and_dates_from_verified_parquet_payload`
  - Passed.

Existing project warnings remain unrelated to this miniQMT slice.

## Remaining Boundary

Still not implemented:

- real double-read comparison against a Quantix source of truth
- ClickHouse shadow import
- production ClickHouse writes
- Quantix ownership of miniQMT evidence apply, registry mutation, or maturity promotion

Authoritative feature status remains `FUNCTION_TREE.md`.
