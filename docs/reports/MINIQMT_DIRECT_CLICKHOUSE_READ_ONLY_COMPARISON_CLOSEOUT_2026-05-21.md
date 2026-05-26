# miniQMT Direct ClickHouse Read-Only Comparison Closeout - 2026-05-21

本文是本切片的本地收尾证据，不作为功能状态注册表；当前功能状态、证据和边界以根目录 `FUNCTION_TREE.md` 为准。

## Status

- Graphiti backfill required
- Graphiti episode `1d2ace2d-9b66-4568-adb2-cbb8044f9e41` queued, then ingest failed with `Request timed out.`
- Commit status: not committed

## Implemented Slice

Quantix now supports an opt-in direct ClickHouse read-only comparison path for `quantix import market-manifest`.

New CLI inputs:

```text
--comparison-clickhouse-url <url>
--comparison-clickhouse-database <database>
--comparison-clickhouse-user <user>
--comparison-clickhouse-password <password>
--comparison-clickhouse-table <table>
--comparison-clickhouse-dataset-version-column <column>
--comparison-clickhouse-symbol-column <column>
--comparison-clickhouse-date-column <column>
```

Behavior:

- Requires `--verify-artifact-file`.
- Mutually exclusive with `--comparison-reference-artifact` and `--comparison-source-of-truth-summary`.
- Uses existing `ClickHouseClient::query_json`.
- Executes only `SELECT` statements for row-count, sample symbols, and sample dates.
- Validates table and column identifiers with a conservative ASCII identifier allowlist.
- Writes `direct_clickhouse_read_only_matched` / `direct_clickhouse_read_only_mismatch` into the raw regression report and controlled evidence.
- Keeps `writes_performed=false`.
- Redacts the ClickHouse password from `source_command`.

## Boundary

This is read-only ClickHouse comparison.

This is not ClickHouse shadow import.

This does not write ClickHouse.

This does not create or migrate tables.

This does not own miniQMT registry promotion, validator, preview, apply, manual promote, rollback readiness, or maturity gate decisions.

## Files Touched

- `CHANGELOG.md`
- `FUNCTION_TREE.md`
- `docs/superpowers/plans/2026-05-21-miniqmt-direct-clickhouse-read-only-comparison.md`
- `src/cli/commands/info.rs`
- `src/cli/handlers/import.rs`
- `src/miniqmt_market.rs`
- `tests/miniqmt_market_import_handler_test.rs`

## Verification

RED:

```text
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_import_handler_test market_manifest_cli_records_direct_clickhouse_read_only_comparison
```

Initial result: failed because the CLI did not recognize `--comparison-clickhouse-url`.

GREEN:

```text
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_import_handler_test market_manifest_cli_records_direct_clickhouse_read_only_comparison
```

Result: 1 passed, 0 failed.

Closure gates:

```text
cargo fmt --manifest-path /opt/claude/quantix-rust/Cargo.toml --check
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test --test miniqmt_market_import_handler_test
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test repo_hygiene_test --quiet
cargo clippy --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test --test miniqmt_market_import_handler_test --quiet
git -C /opt/claude/quantix-rust diff --check -- CHANGELOG.md FUNCTION_TREE.md docs/reports/MINIQMT_DIRECT_CLICKHOUSE_READ_ONLY_COMPARISON_CLOSEOUT_2026-05-21.md docs/superpowers/plans/2026-05-21-miniqmt-direct-clickhouse-read-only-comparison.md src/cli/commands/info.rs src/cli/handlers/import.rs src/miniqmt_market.rs tests/miniqmt_market_import_handler_test.rs
```

Result: all commands exited 0. The two miniQMT test files reported 8 passed and 27 passed. Repo hygiene reported 32 passed. Clippy still reports existing warning debt, including large enum / too many arguments in the CLI path; those are not remediated in this slice.

## GitNexus

Pre-edit impact:

- `run_import_market_manifest`: CRITICAL.
- `ClickHouseClient`: LOW.
- `query_json`: CRITICAL.

Mitigation:

- Did not modify `query_json`.
- New behavior is explicit opt-in.
- Default `market-manifest` behavior remains unchanged.
- New ClickHouse path performs read-only `SELECT` queries and keeps `writes_performed=false`.

Post-change `detect_changes(scope=unstaged)` returned CRITICAL because the shared worktree has 143 changed files and 1515 changed symbols. Treat that as a whole-worktree dirty-state warning, not a clean slice report for this implementation.

## Remaining Work

- ClickHouse shadow import remains [已设计/待实现] in `FUNCTION_TREE.md`.
- Production ClickHouse writes remain out of scope.
- miniQMT registry ownership remains a non-goal for Quantix.
