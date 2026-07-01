# OpenStock Data Consumption P0.11 — TDX-API Cleanup

## Why

`quantix-rust` declared in the 2026-06-30 handoff (`docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md`) that "本项目 `quantix-rust` 仅作数据消费者，所有数据来自容器化部署的 `openstock`". The 2026-07-01 status复审 of that handoff recorded:

- ✅ openstock 侧 P0 三项能力（codes / calendar / index klines）已就绪并 live 验通。
- ✅ `openspec/changes/tdx-api-import-e2e-hardening/` 已归档（方案 A）。
- ❌ **quantix-rust 侧 `TdxApiClient` + 18 个 CLI 子命令 + `import-ticks`/`import-klines` 直写路径完全未动**；`src/sources/tdx_api.rs` 仍在 1309 行，`src/cli/handlers/tdx_api_handler.rs` 仍在 476 行。

P0.11 closes that gap. The slice is **subtractive** (delete + reroute), not additive — the inverse of P0.9/P0.10. It removes the historical TDX-API REST bridge now that OpenStock provides equivalent live capabilities for the P0 categories, and reroutes the two write paths that today go directly to `tdx-api` → ClickHouse / TDengine.

This is explicitly a multi-step slice (`P0.11a` / `P0.11b` / `P0.11c`) because the three sub-targets have different readiness:

- **P0.11a** — `import-klines` → openstock `INDEX_KLINES`/`HISTORICAL_KLINES`: openstock side ready now (live-verified 2026-07-01).
- **P0.11b** — `import-ticks` → openstock `TICK_DATA`: openstock category exists in `DATA_CAPABILITY_SCOPE.md` but quantix has not yet live-verified it. Slice blocked on a live smoke.
- **P0.11c** — remove `TdxApiClient` + 18 CLI subcommands + scheduler fallback: blocked on P0.11a + P0.11b completing without regression.

Each sub-slice is independently releasable. The OpenSpec change is **created as a single unit** so reviewers see the full arc; sub-tasks checkboxes track per-sub-slice readiness.

## What Changes

Three sub-slices, in dependency order:

### P0.11a — `import-klines` data source switch (openstock side ready)

- Edit `src/cli/handlers/tdx_api_handler.rs::ImportKlines`: add a new `--source <openstock|tdx-api>` flag with default `openstock`. When `openstock`, fetch via `OpenStockClient::fetch_index_klines` (or a new `fetch_historical_klines` wrapper if `HISTORICAL_KLINES` proves necessary) and write through the **existing** `ClickHouseClient::insert_klines` path used by P0.8g-impl shadow persistence (minus the `quantix_shadow` namespace).
- Keep `--source tdx-api` as explicit legacy path. Print deprecation warning when used.
- Live-gated integration test `tests/openstock_import_klines_live_test.rs` (new): fetch a single symbol via openstock, assert ClickHouse write succeeded, rollback via existing shadow-rollback pattern adapted for the main `klines` table.
- FUNCTION_TREE.md L95 update: `import-klines` notes "default source = openstock since P0.11a; tdx-api legacy path removal tracked in P0.11c".

### P0.11b — `import-ticks` data source switch (blocked on TICK_DATA live smoke)

- Add `OpenStockClient::fetch_tick_data(&self, code: &str, date: Option<&str>)` wrapper in `src/sources/openstock_client.rs` mirroring `fetch_index_klines` shape.
- Add a fixture-driven parser `parse_tick_data` in new `src/sources/openstock_ticks.rs` (mirrors `openstock_index.rs` layout) plus tests.
- Edit `src/cli/handlers/tdx_api_handler.rs::ImportTicks`: add `--source <openstock|tdx-api>` flag, default `openstock`. When openstock, fetch + parse → existing TDengine write path (`src/db/tdengine.rs` retained — storage backend choice is internal per handoff §三.1).
- New `#[ignore]` live test `tests/openstock_tick_data_live_test.rs` — must pass before P0.11b is marked done.
- If openstock `TICK_DATA` proves non-functional in live smoke, **fall back**: keep `tdx-api` as default for `ImportTicks`, document in proposal, split P0.11b out as a separate OpenSpec change.

### P0.11c — Remove `TdxApiClient` and 18 CLI subcommands

- Delete `src/sources/tdx_api.rs` (1309 lines).
- Delete `src/cli/handlers/tdx_api_handler.rs` (476 lines) and **all 18 subcommands** in `TdxApiCommands` enum (`src/cli/commands/data.rs:64`).
- Reroute `src/tasks/collect_scheduler.rs:83,136` — `tdx_api_fallback` collector. Either:
  - **Option A (preferred)**: replace with `openstock_fallback` using `OpenStockClient::fetch_realtime_quotes` (P2 capability — openstock category exists, quantix client method does not; add as part of P0.11c).
  - **Option B**: delete the fallback path entirely if no production consumer relies on the scheduler today.
- Reroute `src/cli/handlers/data_handler.rs:348` `DataSourceKind::TdxApi` arm — remove the variant or alias it to `OpenStock`.
- Update FUNCTION_TREE.md: L95, L781, L1126 tdx-api entries → `[deprecated, removed in P0.11c]`. Update L212, L658 tree nodes.
- Update `docker-compose.yml`: remove or comment out `tdx-api` service (openstock remains).

## Impact

- **Public API removed**: `crate::sources::tdx_api::TdxApiClient` and its 33 methods. No external consumers expected (this is an application, not a library).
- **CLI surface removed**: 18 `quantix data tdx-api ...` subcommands become unavailable. Users running those commands today must switch to `quantix data openstock ...` equivalents (added in P0.10 + P0.11a/b).
- **Internal reroutes**: `collect_scheduler.rs` fallback collector + `data_handler.rs` health check branch.
- **Runtime**: `tdx-api` Docker service becomes optional then removed; `openstock` is the sole data source.
- **Risk**: per `gitnexus_impact` (P0.11 planning, 2026-07-01) on `TdxApiClient` struct: nominal LOW because the index is stale; **real consumers verified by grep**: `tdx_api_handler.rs` (the file being deleted), `data_handler.rs:348`, `collect_scheduler.rs:83,136`. The grep-grounded consumer count is 3, not 0 — Option A in P0.11c must be implemented before the delete lands.

## Non-Goals

- No new data category on openstock side — this slice consumes existing categories.
- No ClickHouse / TDengine storage backend change. Storage selection is internal to quantix-rust per handoff §三.1.
- No removal of `bridge_tdx` (the TCP bridge is separate from `tdx-api` REST; judged out of scope).
- No removal of `miniqmt_market.rs` (Windows-client-dependent, already declared out-of-scope in handoff §六).
- No replacement of the existing P0.10 `fetch-*` CLI subcommands — they remain the live-ingest surface.
- No `Kline` / `BacktestEngine` / `ExecutionAdapter` / `OrderStatus` / `ControlledPersistencePolicy` modification — CRITICAL hubs are read-only consumers if they participate at all.
- No retry-policy / circuit-breaker tuning — P0.11 reroutes call sites, the OpenStockClient retry/breaker from commit `440c79f` is reused as-is.
