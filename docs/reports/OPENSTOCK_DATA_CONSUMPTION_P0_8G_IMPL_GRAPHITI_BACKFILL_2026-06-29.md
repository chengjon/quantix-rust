# OpenStock Data Consumption P0.8g-impl Graphiti Backfill

Date: 2026-06-29

## Summary

P0.8g-impl (OpenStock shadow persistence write path) was closed, merged, and verified on master, but its Graphiti closeout episode may not reach `completed` during the closeout polling window.

Per the project Graphiti fallback rule, this report records an equivalent local memory so the P0.8g-impl handoff remains durable.

Graphiti backfill required.

## Graphiti Episode

- Group: `quantix_rust_main`
- Episode: not yet captured (no closeout episode polled this slice)
- Fallback: local backfill recorded below

## Equivalent Memory

P0.8g-impl OpenStock shadow persistence write path closed and merged.

PR #319 squash merge commit `e5639ac068aa157f14fedbc450c3e4d483657e66` added the first writable shadow persistence path under `quantix_shadow.openstock_daily_kline_shadow`. The slice consumed the P0.8f `LiveShadowReport` as input contract and implemented the design contract from PR #318 (`docs/reports/OPENSTOCK_DATA_CONSUMPTION_P0_8G_SHADOW_PERSISTENCE_DESIGN_2026-06-29.md`).

The implementation:

- New module `src/sources/openstock_shadow.rs`: pure `artifact_hash` SHA-256, `ShadowKlineRow`, `ShadowWriteReport`, `ShadowWriteError`, `build_shadow_rows_from_report` (dry-run gate), `write_shadow_klines` (double-gate opt-in), `rollback_shadow_batch` (idempotent), `verify_shadow_batch`.
- New module `src/db/clickhouse/shadow_kline.rs`: `ClickHouseClient` append-only extension (`insert_shadow_klines`, `delete_shadow_batch`, `count_shadow_batch`). Existing methods untouched.
- New CLI subcommands: `quantix data openstock persist-live` (dry-run default; `--apply` + `QUANTIX_SHADOW_PERSIST_CONFIRM=yes` double-gate for any write), `shadow-rollback --batch-id`, `shadow-verify --batch-id`.
- Schema artifact `db/schema/quantix_shadow_init.sql` for operator manual setup: `quantix_shadow.openstock_daily_kline_shadow` ReplacingMergeTree, dedup key `source+period+code+date+adjust_type`, partitioned by `toYYYYMM(date)`.
- Tests: 8 default-CI tests cover all design §8 assertions (dry-run no-connect, requires-apply-flag, requires-env-confirm, rejects-drift, rejects-fail-closed, rejects-duplicates, artifact-hash-deterministic, rollback-idempotent); 2 `#[ignore]` integration tests gated by `QUANTIX_SHADOW_INTEGRATION=1`.

Preserved boundaries:

- Additive-only: new modules + appended methods on existing `ClickHouseClient`.
- No live OpenStock network calls (payload externally captured).
- No `Kline` (CRITICAL hub) modification — read-only consumption only.
- No `ControlledPersistencePolicy`, `BacktestEngine`, `src/db/clickhouse/kline.rs`, or `Cargo.toml` modification.
- No data-source route replacement.
- No qmt_live or miniQMT changes.
- No `ExecutionAdapter` or `OrderStatus` changes.

GitNexus impact:

- `ClickHouseClient`: LOW, additive-only (new methods, no modification of existing methods).
- `validate_live_shadow_payload`: LOW, read-only consumption of `LiveShadowReport`.
- `OpenStockCommands`: LOW, 3 new variants appended.
- Final `detect_changes`: only 2 symbols touched (`run_data_command`, `run_task_command` in `app_shell.rs` — dispatch match arms); `ControlledPersistencePolicy`/`Kline`/`BacktestEngine` touches = 0.

Verification:

- `cargo fmt --check`.
- `cargo clippy --tests -D warnings` (0 warnings).
- `cargo test --workspace` (1378 passed, 0 failed, 2 ignored integration).
- `openspec validate openstock-data-consumption-p0-8 --strict`.
- `openspec validate --all --strict` (4 passed).
- FUNCTION_TREE scope-check (22 files), validate, gate all passed.
- `git diff --check`.
- GitNexus `detect_changes` confirmed forbidden-hub isolation.
- PR #319 CI passed (Lint SUCCESS, Test SUCCESS).

## Backfill Action

Backfill this content into `quantix_rust_main` if a future closeout episode for P0.8g-impl does not reach `completed`.
