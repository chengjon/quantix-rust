## Why

P0.14 shipped `stream_minute_klines_to_clickhouse` / `stream_minute_shares_to_clickhouse` consumers and the `minute_klines` / `minute_shares` ClickHouse tables, but zero callers exist anywhere in the codebase. The library is built but unreachable from the CLI. P0.15a wires the P0.14 consumers to two user-invokable CLI subcommands so a human (or a future P0.15b scheduler) can persist minute bars and minute shares to ClickHouse by code + date range.

## What Changes

- Two new CLI subcommands: `data openstock import-minute-klines` and `data openstock import-minute-share`.
- Both gated by a double-key: `--apply == true` AND `env QUANTIX_OPENSTOCK_MINUTE_APPLY == "yes"`.
- Dry-run path streams + counts; never constructs ClickHouse client.
- Apply path consumes the P0.14 `stream_minute_*_to_clickhouse` consumer.
- 5 new requirements REQ-PERSIST-006 through REQ-PERSIST-010.

## Impact

- New surface: 2 subcommands on `OpenStockCommands`. No existing CLI behavior changes.
- New env var: `QUANTIX_OPENSTOCK_MINUTE_APPLY`.
- No database migrations, no schema changes, no new dependencies.

## Non-Goals

- Scheduler / cron triggers (P0.15b).
- Multi-code orchestration per invocation (P0.15b).
- `--date` single-day shortform.
- Idempotent rollback / ReplacingMergeTree migration.
- Real-time / live-tick import.
- `assert_cmd` subprocess tests.
