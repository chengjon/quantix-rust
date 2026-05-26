# miniQMT Source-Of-Truth Summary Comparison Closeout - 2026-05-21

本文是本切片的本地收尾证据，不作为功能状态注册表；当前功能状态、证据和边界以根目录 `FUNCTION_TREE.md` 为准。

## Status

- Graphiti backfill required
- Graphiti episode UUID: `4872d0d0-ca94-4a0b-bbed-f93151d4ddf9`
- Graphiti ingest result: failed with `Request timed out.`
- Commit status: not committed

## Implemented Slice

Quantix now supports an opt-in source-of-truth summary comparison path for `quantix import market-manifest`.

New CLI input:

```text
--comparison-source-of-truth-summary <json-path-or-file-uri>
```

Behavior:

- Requires `--verify-artifact-file`.
- Mutually exclusive with `--comparison-reference-artifact`.
- Reads a local JSON source-of-truth summary file.
- Fails closed if `dataset_version` does not match the miniQMT manifest identity.
- Fails closed if optional `lineage_id` or `payload_hash` are present and do not match the miniQMT manifest identity.
- Compares row-count, sample symbols, and sample dates.
- Writes `source_of_truth_summary_matched` / `source_of_truth_summary_mismatch` into the raw regression report and controlled evidence.
- Records source summary file SHA-256, `source_system`, and `source_uri` in the comparison block.

## Boundary

This is not direct ClickHouse read-only comparison.

This is not ClickHouse shadow import.

This does not write ClickHouse.

This does not own miniQMT registry promotion, validator, preview, apply, manual promote, rollback readiness, or maturity gate decisions.

The new path only lets Quantix consume a controlled source-of-truth summary artifact and bind it into `quantix_regression` evidence.

## Files Touched

- `CHANGELOG.md`
- `FUNCTION_TREE.md`
- `src/cli/commands/info.rs`
- `src/cli/handlers/import.rs`
- `src/miniqmt_market.rs`
- `tests/miniqmt_market_import_handler_test.rs`

## Verification

RED:

```text
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_import_handler_test market_manifest_cli_records_source_of_truth_summary_comparison
```

Initial result: failed because the CLI did not recognize `--comparison-source-of-truth-summary`.

GREEN:

```text
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_import_handler_test market_manifest_cli_records_source_of_truth_summary_comparison
```

Result: 1 passed, 0 failed.

Final gates:

```text
cargo fmt --manifest-path /opt/claude/quantix-rust/Cargo.toml --check
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test --test miniqmt_market_import_handler_test
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test repo_hygiene_test --quiet
cargo clippy --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test --test miniqmt_market_import_handler_test --quiet
git -C /opt/claude/quantix-rust diff --check -- CHANGELOG.md FUNCTION_TREE.md src/cli/commands/info.rs src/cli/handlers/import.rs src/miniqmt_market.rs tests/miniqmt_market_import_handler_test.rs tests/miniqmt_market_manifest_test.rs
```

Results:

- `cargo fmt --check`: passed.
- miniQMT tests: 7 + 27 passed, 0 failed.
- `repo_hygiene_test`: 32 passed, 0 failed.
- targeted `clippy`: exit status 0; existing warning set remains, including broader repository warnings and import-handler `too_many_arguments`.
- scoped `diff --check`: passed.

## GitNexus

Pre-edit impact:

- `run_import_market_manifest`: CRITICAL.
- Direct caller: `run_import_command`.
- Affected processes: 20.
- Mitigation: change is explicit opt-in; default `market-manifest` behavior remains dry-run artifact resolution and existing report/evidence behavior.

Other touched helper symbols were not found by GitNexus because parts of the miniQMT implementation are still untracked or stale relative to the current index.

Post-change `detect_changes(scope=unstaged)`:

- Risk: CRITICAL.
- Changed files: 143.
- Changed symbols: 1515.
- Affected symbols: 140.

This is the whole shared dirty worktree risk, not a clean miniQMT-only scope report.

## Remaining Work

- Direct ClickHouse read-only double-read comparison remains [已设计/待实现] in `FUNCTION_TREE.md`.
- ClickHouse shadow import remains [已设计/待实现] in `FUNCTION_TREE.md`.
- Production ClickHouse writes remain out of scope.
- miniQMT registry ownership remains a non-goal for Quantix.
