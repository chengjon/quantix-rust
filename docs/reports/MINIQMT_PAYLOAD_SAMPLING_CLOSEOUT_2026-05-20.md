# miniQMT Payload Sampling Closeout

Date: 2026-05-20

Graphiti backfill required

## Summary

Quantix now performs best-effort local Parquet payload sampling for the miniQMT controlled evidence path.

After `--verify-artifact-file` resolves a local path / `file://` artifact and verifies the artifact SHA-256, `ResolvedMarketArtifact` can capture deduplicated:

- `sample_symbols`
- `sample_dates`

The samples are copied into the raw `QuantixRegressionReport` and miniQMT-shaped `quantix_regression` evidence. When sampling succeeds, the regression checks include:

```text
artifact_payload_sampled
```

Sampling does not replace the required dataset identity or artifact hash gates.

## Boundary

Implemented:

- local Parquet sampling for recognizable symbol columns: `symbol`, `code`, `ts_code`, `ticker`
- local Parquet sampling for recognizable date columns: `date`, `trade_date`, `datetime`, `timestamp`
- sample propagation into report/evidence JSON
- `FUNCTION_TREE.md`, README, alignment spec, and operator runbook status updates
- miniQMT runbook mirror refresh under `/mnt/d/MyCode3/miniQMT/DOCS/quantix/`

Still not implemented:

- real double-read comparison against a Quantix source of truth
- ClickHouse shadow import
- production ClickHouse writes
- Quantix ownership of miniQMT registry apply / maturity promotion

## Verification

RED:

- `cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test quantix_regression_report_samples_symbols_and_dates_from_verified_parquet_payload`
- initial failure: `ResolvedMarketArtifact` had no `sample_symbols` / `sample_dates` fields

GREEN / closure:

- `cargo fmt --manifest-path /opt/claude/quantix-rust/Cargo.toml --check`: passed
- `git diff --check` for the related Quantix files: passed
- `git -C /mnt/d/MyCode3/miniQMT diff --check -- DOCS/quantix/MINIQMT_QUANTIX_REGRESSION_OPERATOR_RUNBOOK_2026-05-20.md`: passed
- `cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test --test miniqmt_market_import_handler_test`: 26 + 5 passed
- `cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test repo_hygiene_test --quiet`: 32 passed
- targeted clippy wrapper: `exit=0`, `miniqmt_related_warning_lines=0`

## Notes

GitNexus impact lookup for the new untracked miniQMT symbol path could not resolve the target because `src/miniqmt_market.rs` remains untracked / unindexed in the shared dirty workspace.

`gitnexus detect_changes(scope=unstaged)` reported a critical whole-workspace state because the shared worktree already contains 143 changed files and 1515 changed symbols. That result is a workspace risk indicator, not a clean scope report for this payload sampling slice.
