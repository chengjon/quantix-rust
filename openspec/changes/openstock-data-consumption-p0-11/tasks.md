# Tasks — OpenStock Data Consumption P0.11 (TDX-API Cleanup)

Sub-slice status legend:
- 🔲 not started
- 🟡 in-flight
- ✅ done
- ⛔ blocked (see Notes)

---

## 0. Baseline And Governance

- [ ] 0.1 Baseline: master HEAD is clean; `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`, `openspec validate --all --strict` all green.
- [ ] 0.2 Read `docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md` end-to-end, especially the 2026-07-01 Status section.
- [ ] 0.3 Grep-grounded consumer inventory (gitnexus impact may be stale): `TdxApiClient` callers in `src/` = `tdx_api_handler.rs`, `data_handler.rs:348`, `collect_scheduler.rs:83,136`. Document in `design.md` Consumer Map.
- [ ] 0.4 Create `.governance/programs/project-governance/cards/P0.11.yaml` scoped to `openstock-data-consumption-p0-11/*` + every `src/` path being edited or deleted.
- [ ] 0.5 `openspec validate openstock-data-consumption-p0-11 --strict` passes.
- [ ] 0.6 (Deferred) `ft:new-node`/`ft:transition` for P0.11 — governance flow invocation deferred to closeout per P0.9/P0.10 precedent.

---

## 1. P0.11a — `import-klines` Source Switch (openstock ready)

### 1a. Design

- [ ] 1a.1 Decide: `INDEX_KLINES` vs `HISTORICAL_KLINES` for `import-klines`. Default `INDEX_KLINES` for index codes, `HISTORICAL_KLINES` for stock codes. Document in `design.md` §1a.
- [ ] 1a.2 Decide: ClickHouse write target. Reuse existing `ClickHouseClient::insert_klines` (main `klines` table, not `quantix_shadow`). Dry-run first, `--apply` gate mirrors P0.8g-impl.

### 1a. Code

- [ ] 1a.3 Add `OpenStockClient::fetch_historical_klines(&self, code: &str, period: &str, start: Option<&str>, end: Option<&str>)` wrapper in `src/sources/openstock_client.rs` (mirror `fetch_index_klines` shape; use `start_date`/`end_date`).
- [ ] 1a.4 Edit `src/cli/commands/data.rs::ImportKlines`: add `#[arg(long, default_value = "openstock")] source: String` + `#[arg(long)] start: Option<String>` + `#[arg(long)] end: Option<String>`.
- [ ] 1a.5 Edit `src/cli/handlers/tdx_api_handler.rs::import_klines`: branch on `source`; openstock branch calls `fetch_historical_klines` → existing ClickHouse write path; tdx-api branch unchanged + `eprintln!("⚠️ tdx-api legacy path, scheduled for removal in P0.11c")`.
- [ ] 1a.6 Add `#[ignore]` live test `tests/openstock_import_klines_live_test.rs` gated by `QUANTIX_OPENSTOCK_LIVE=1`. Single symbol, dry-run only. Assert: OpenStock fetch returns ≥1 record, ClickHouse dry-run report maps records without drift.

### 1a. Verify

- [ ] 1a.7 `cargo test --workspace` (live tests skip).
- [ ] 1a.8 `cargo test --test openstock_import_klines_live_test -- --ignored` with live env → passes.
- [ ] 1a.9 Manual smoke: `cargo run -q -- data tdx-api import-klines --code 600000 --source openstock` (dry-run) prints sensible report.

---

## 2. P0.11b — `import-ticks` Source Switch (blocked on TICK_DATA smoke)

### 2b. Unblock

- [x] 2b.1 Live smoke `TICK_DATA`: `curl -X POST $OPENSTOCK_BASE_URL/data/fetch -H "X-API-Key: $KEY" -d '{"data_category":"TICK_DATA","params":{"symbol":"600000","date":"20260630"}}'`. Sample shape captured in design.md §D4.1. **Note: parameter is `symbol`, not `code`** (sending `code` returns HTTP 422).
- [x] 2b.2 TICK_DATA returns usable data (HTTP 200, 456 KB, 1800 ticks for sh600000 on 2026-06-30). Proceed to 2b.3. P0.11b is **not split out**.

### 2b. Code (only if 2b.2 passes)

- [x] 2b.3 Add `OpenStockClient::fetch_tick_data(&self, symbol: &str, date: Option<&str>)` wrapper. Parameter name MUST be `symbol` (NOT `code`) — see design.md §D4.1.
- [x] 2b.4 Create `src/sources/openstock_ticks.rs` with `TickRecord`, `parse_tick_data(envelope) -> Result<Vec<TickEntry>, TickParseError>`, and `normalize_*` helpers mirroring `openstock_index.rs` layout.
- [x] 2b.5 Add fixture-driven unit tests `tests/openstock_ticks.rs` covering: happy path, empty records, missing required field, malformed numeric (string vs number, mirroring `IndexKlineRecord` learnings).
- [x] 2b.6 Edit `ImportTicks` command: add `--source` flag (default `openstock`).
- [x] 2b.7 Edit `import_ticks` handler: branch on `source`; openstock branch → TDengine write via existing `src/db/tdengine.rs` client.
- [x] 2b.8 Add `#[ignore]` live test `tests/openstock_tick_data_live_test.rs`.

### 2b. Verify

- [x] 2b.9 `cargo test --workspace` (live tests skip).
- [x] 2b.10 `cargo test --test openstock_tick_data_live_test -- --ignored` → passes. **Live-verified 2026-07-01** against `http://192.168.123.104:8040`, symbol=600000 date=20260630, parser produced non-empty `Vec<Tick>`, TDengine untouched (dry-run).

---

## 3. P0.11c — Remove `TdxApiClient` + 18 CLI subcommands (blocked on 1 + 2)

> **编排依据**: `docs/reports/P0_11C_PREFLIGHT_AUDIT_2026-07-01.md` §四（r2，已应用审核反馈）。本节按五阶段（Phase 1-5）展开，task 编号 3c.1 → 3c.23。
> **硬约束**: Phase 1 必须先于 Phase 4；Phase 4 步骤 3c.16（删 `config.rs::TdxApiConfig`）必须与 3c.10（删 `tdx_api.rs`）同步，否则 `DataSourceConfig.tdx_api` 字段引用悬空，编译断裂。
> **Decision 默认推荐**: Decision 1=A / 2=A / 3=B / 4=A（全部采纳默认时总估时 5.5-7.5 天，净删 ~2 020 行）。

### 3c. Pre-flight

- [ ] 3c.1 P0.11a + P0.11b both merged and verified (P0.11a = commit `d5e9b75` ✅; P0.11b = commit `47747c5` + `c16ea8e` ✅).
- [x] 3c.2 **2b.10 live smoke passed** (`cargo test --test openstock_tick_data_live_test -- --ignored`) — ✅ 2026-07-01, openstock:8040 reachable, TICK_DATA parser verified end-to-end.
- [x] 3c.3 Decision 1 confirmed: **A** (rewire to OpenStock, 3-5 天, P0.11d 规模独立切片). 确认时间 2026-07-01.
- [x] 3c.4 Decision 2 confirmed: **A** (迁出到 `openstock_handler.rs`).
- [x] 3c.5 Decision 3 confirmed: **B** (拆 direction+status 双列, ~0.5 天).
- [x] 3c.6 Decision 4 confirmed: **A** (顶层 promote 到 `quantix data import-*`).

### 3c. Phase 1 — OpenStock 分支迁出（safety-critical，必须先于 Phase 4）

> 删除 `tdx_api_handler.rs` 前，P0.11a/b 写入该文件的 openstock 分支必须先迁出，否则删除时一并丢失。

- [x] 3c.7 把 `tdx_api_handler.rs::import_ticks` 的 openstock 分支迁到 `openstock_handler.rs`（Decision 2 = A 时）. ✅ commit `d73f860` — 新增 `import_openstock_ticks()` (screener_handler.rs:596).
- [x] 3c.8 把 `tdx_api_handler.rs::import_klines` 的 openstock 分支迁到 `openstock_handler.rs`（Decision 2 = A 时）. ✅ commit `d73f860` — 新增 `import_openstock_klines()` (screener_handler.rs:723).
- [x] 3c.9 `app_shell.rs` dispatcher 重路由到新位置；如 Decision 4 = A，新增 `DataCommands::ImportTicks` / `ImportKlines` 顶层 variant. ✅ commit `d73f860` — DataCommands 加 ImportTicks/ImportKlines 顶层 variant, app_shell.rs 加 dispatcher 分支.
- [x] 3c.10 `cargo build + cargo test --workspace` 全绿（验证迁出无误）. ✅ commit `d73f860` — fmt clean, clippy -D warnings clean, 765 lib tests + 681 integration tests + 14 ignored (live) all pass.

### 3c. Phase 2 — TDengine schema 准备（按 Decision 3 = B 展开）

> 如选 A 或 C，本 phase 步骤和估时需要调整：A 膨胀到 ~2 天（需 backfill），C 改加 `source VARCHAR` tag 列。

- [ ] 3c.11 TDengine migration script: 加 `direction TINYINT` 列（与现有 `status TINYINT` 物理隔离）。
- [ ] 3c.12 修改 `import_ticks` openstock 分支：写入 `direction` 列而非 `status`（保留 legacy `status` 字段不动，承载 tdx-api 历史字节）。
- [ ] 3c.13 `cargo test --workspace` 全绿。

### 3c. Phase 3 — scheduler reroute（Decision 1 = A 时；实为 P0.11d 规模独立切片）

> ⚠️ 审计文档 §二 Decision 1 标注：Option A 工作量「高（需 P0.11d 规模新 parser 切片）」，3-5 天。如 Decision 1 = B，跳过本 phase，直接进 Phase 4；总计估时降到 2.5 天，但失去 fallback 兜底（见风险 R7）。

- [ ] 3c.14 Live-verify OpenStock `REALTIME_QUOTES` category（design.md R5 标注未 live-verified）。
- [ ] 3c.15 新增 `OpenStockClient::fetch_realtime_quotes(&self, codes: &[&str])` wrapper。
- [ ] 3c.16 新建 `src/sources/openstock_quotes.rs` parser 模块（对标 `openstock_ticks.rs` 297 行规模），含 `parse_realtime_quotes` + fixture 单元测试。
- [ ] 3c.17 Rewire `collect_scheduler.rs:83,136` 从 `TdxApiClient` 改持有 `OpenStockClient`。字段改名 `tdx_api_fallback` → `openstock_fallback`；方法改名 `set_tdx_api_fallback` → `set_openstock_fallback`。需要 adapter 层转换 `TdxApiClient::collect_all_quotes` 返回类型为 OpenStock category-based fetch 语义。
- [ ] 3c.18 `cargo test --workspace` 全绿（含 scheduler 测试套件）。

### 3c. Phase 4 — 删除（此时 legacy 已无生产引用）

- [ ] 3c.19 删 `src/sources/tdx_api.rs`（1 309 行）+ `src/sources/mod.rs` 的 `pub mod tdx_api;`。
- [ ] 3c.20 删 `src/cli/handlers/tdx_api_handler.rs`（726 行）+ `src/cli/handlers/mod.rs` 的 `pub mod tdx_api_handler;`。⚠️ 必须确认 Phase 1 已完成，openstock 分支已迁出。
- [ ] 3c.21 删 `src/cli/commands/data.rs` 的 `TdxApiCommands` 枚举 + `TdxApi` parent variant（保留 `ImportTicks` / `ImportKlines`，按 Decision 4 = A promote 到 `DataCommands` 顶层）。
- [ ] 3c.22 删 `src/cli/command_types.rs` 的 `DataSourceKind::TdxApi` 变体。
- [ ] 3c.23 删 `src/cli/handlers/app_shell.rs` 的 `TdxApi(cmd) => ...` dispatcher 分支（Phase 1 已重路由后，本步骤仅清空死分支）。
- [ ] 3c.24 删 `src/cli/handlers/data_handler.rs` 的全部 `tdx_api_*` helper（L42-47 `PersistedTdxApiConfig`、L543-605 helper 函数、L279-281 / L300 / L347-351 分支、L678-712 测试）。
- [ ] 3c.25 ⚠️ **CRITICAL**：删 `src/core/config.rs` 的 `TdxApiConfig` 结构体 + 6 个 `default_tdx_api_*` 函数 + `DataSourceConfig.tdx_api` 字段。**必须与 3c.19 同 commit 或紧邻 commit**，否则 `DataSourceConfig.tdx_api` 引用悬空，编译断裂（审核意见 1）。
- [ ] 3c.26 清理 §1.4 仅注释/doc-comment 文件：`src/core/trading_calendar.rs:387` 注释、`tests/openstock_tick_data_live_test.rs` doc、`tests/openstock_import_klines_live_test.rs` doc。
- [ ] 3c.27 `grep -rn "tdx_api\|TdxApi\|tdx-api" src/ tests/ --include="*.rs"` → 应为空（验证全部清理）。
- [ ] 3c.28 `cargo build --workspace + cargo test --workspace` 全绿。
- [ ] 3c.29 `gitnexus detect_changes` — expected scope: deleted `tdx_api.rs`, deleted `tdx_api_handler.rs`, modified `collect_scheduler.rs`, `data_handler.rs`, `data.rs`, `app_shell.rs`, `config.rs`, `command_types.rs`. No surprise file.

### 3c. Phase 5 — Ecosystem cleanup（main 已稳定后）

- [ ] 3c.30 `docker-compose.yml`: 注释或删除 `tdx-api` 服务块（design.md R6 建议注释保留以便回滚）。
- [ ] 3c.31 `FUNCTION_TREE.md` L95 / L212 / L658 / L781 / L1126 五处更新（design.md D7）— 标 `[deprecated, removed in P0.11c]` 或删除。
- [ ] 3c.32 `docs/CLI_COMMAND_MANUAL.html`：删除 `cmd-data-tdx-api-import-ticks` / `cmd-data-tdx-api-import-klines` section，新增 `cmd-data-import-ticks` / `cmd-data-import-klines`（Decision 4 = A 时）；同步侧边导航、目录条目。
- [ ] 3c.33 Update README.md, CHANGELOG.md, `docs/guides/TDX_API_BRIDGE_GUIDE.md` — 加 deprecation banner 指向 OpenStock。Decision 4 = A 时考虑保留 `tdx-api` parent 作为隐藏 alias 一个 release（风险 R9 缓解）。

---

## 4. Closeout

- [ ] 4.1 `docs/reports/OPENSTOCK_DATA_CONSUMPTION_P0_11_CLOSEOUT_<date>.md` — record what shipped (per sub-slice), consumer reroutes, any residual `tdx-api` references (should be doc-only).
- [ ] 4.2 Update handoff doc `HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md` Status section: §三.1 / §三.2 / §七 lines move from ❌ to ✅.
- [ ] 4.3 Archive this OpenSpec change to `openspec/changes/archive/<YYYY-MM-DD>-openstock-data-consumption-p0-11/`.
- [ ] 4.4 Governance card `P0.11.yaml` transition to `completed`.

---

## Risks And Rollback

- **Risk R1 (P0.11a)**: openstock `HISTORICAL_KLINES` field shape differs from `INDEX_KLINES` (likely — different providers, baostock vs baostock/akshare). Mitigation: fixture-driven parser tests before live wiring, mirroring P0.9's pattern.
- **Risk R2 (P0.11b)**: openstock `TICK_DATA` may not be live-functional. Mitigation: explicit unblock gate at task 2b.2; fall back to keeping `import-ticks` on tdx-api if needed (slice boundary adapts).
- **Risk R3 (P0.11c)**: hidden `TdxApiClient` consumer not caught by grep (e.g., dynamically constructed via cfg, conditionally compiled). Mitigation: 3c.27 grep audit + 3c.28 build clean + 3c.29 `gitnexus detect_changes` post-check.
- **Risk R4 (P0.11c)**: `collect_scheduler` fallback rewire to openstock breaks an unrelated execution flow. Mitigation: 3c.17 is the only scheduler edit, run full scheduler-related test suite before merge.
- **Risk R7 (P0.11c, audit-r2)**: scheduler fallback 删除后（Decision 1 = B）主采集器失败无兜底。Mitigation: 选 Decision 1 = A；若选 B，先确认上层有重试机制，且增加采集失败告警。
- **Risk R8 (P0.11c, audit-r2)**: TDengine schema migration 不可逆（Decision 3 = B 加 `direction` 列后，回滚不能删列）。Mitigation: 先在 staging 环境验证；准备 backfill 脚本以反向兼容。
- **Risk R9 (P0.11c, audit-r2)**: CLI 路径变更破坏外部脚本（Decision 4 = A 后 `quantix data tdx-api import-*` 不存在）。Mitigation: 在 CHANGELOG 显式标注；保留一个 release 的 deprecation warning；考虑保持 `tdx-api` parent 作为隐藏 alias 一个版本。
- **Risk R10 (P0.11c, audit-r2 CRITICAL)**: 删除 `tdx_api.rs` 时漏删 `config.rs::TdxApiConfig` → `DataSourceConfig.tdx_api` 字段引用悬空 → 编译断裂。Mitigation: 3c.25 必须与 3c.19 同步或紧邻；3c.27 grep 审计全空后才能进 Phase 5。
- **Rollback**: each sub-slice is one commit. Revert the offending commit; tdx-api path returns automatically (it's only deleted in P0.11c, not before). Phase 2 schema migration（R8）不可逆，需要专门回滚脚本。

---

## Reference

- Handoff: `docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md` (especially 2026-07-01 Status section)
- Live findings: `docs/proposals/openstock-live-integration-findings.md`
- openstock-side issue log: `/opt/claude/openstock/docs/operations/REMOTE_TEST_2026-07-01_ISSUES.md`
- Prior slice (P0.10): `openspec/changes/archive/2026-06-30-openstock-data-consumption-p0-10/`
- Shadow persistence (P0.8g-impl) — ClickHouse write path to mirror: `src/sources/openstock_shadow.rs`
