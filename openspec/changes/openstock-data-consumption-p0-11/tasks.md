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
- [ ] 2b.10 `cargo test --test openstock_tick_data_live_test -- --ignored` → passes.

---

## 3. P0.11c — Remove `TdxApiClient` + 18 CLI subcommands (blocked on 1 + 2)

### 3c. Pre-flight

- [ ] 3c.1 P0.11a + P0.11b both merged and verified.
- [ ] 3c.2 Decide collect_scheduler fallback: Option A (rewire to `OpenStockClient::fetch_realtime_quotes`) vs Option B (delete the fallback). Default A unless `collect_scheduler` itself is unused in production.

### 3c. Code (Option A path)

- [ ] 3c.3 Add `OpenStockClient::fetch_realtime_quotes(&self, codes: &[&str])` wrapper.
- [ ] 3c.4 Add `parse_realtime_quotes` in `src/sources/openstock_quotes.rs` (new file).
- [ ] 3c.5 Rewire `src/tasks/collect_scheduler.rs:83,136` from `tdx_api::TdxApiClient` to `OpenStockClient`. Field rename `tdx_api_fallback` → `openstock_fallback`. Method rename `set_tdx_api_fallback` → `set_openstock_fallback`.
- [ ] 3c.6 Reroute `src/cli/handlers/data_handler.rs:348` `DataSourceKind::TdxApi` arm. Remove the variant or alias it.
- [ ] 3c.7 Delete `src/sources/tdx_api.rs`.
- [ ] 3c.8 Delete `src/cli/handlers/tdx_api_handler.rs`.
- [ ] 3c.9 Remove `TdxApi(TdxApiCommands)` arm from `src/cli/commands/data.rs:14-15` and the entire `TdxApiCommands` enum at L64.
- [ ] 3c.10 Remove `pub mod tdx_api_handler;` and `pub mod tdx_api;` re-exports.

### 3c. Verify

- [ ] 3c.11 `cargo build --workspace` — no remaining references to `TdxApiClient`, `TdxApiCommands`, `tdx_api_handler`, `tdx-api` service.
- [ ] 3c.12 `grep -rn "tdx_api\|TdxApi\|tdx-api" src/ tests/` — only doc comments remain (no live code).
- [ ] 3c.13 `cargo fmt --check && cargo clippy -D warnings && cargo test --workspace` — clean.
- [ ] 3c.14 `gitnexus detect_changes` — expected scope: deleted `tdx_api.rs`, deleted `tdx_api_handler.rs`, modified `collect_scheduler.rs`, `data_handler.rs`, `data.rs`, `app_shell.rs`. No surprise file.

### 3c. Ecosystem cleanup

- [ ] 3c.15 `docker-compose.yml`: remove or comment out `tdx-api` service block.
- [ ] 3c.16 FUNCTION_TREE.md: L95 (`quantix data` row), L212 (tree), L658 (`tdx-api fallback`), L781 (subcommand list), L1126 (`tdx-api bridge` row) — mark `[deprecated, removed in P0.11c]` or delete as appropriate.
- [ ] 3c.17 Update README.md, CHANGELOG.md, `docs/guides/TDX_API_BRIDGE_GUIDE.md` — add deprecation banner pointing to OpenStock.

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
- **Risk R3 (P0.11c)**: hidden `TdxApiClient` consumer not caught by grep (e.g., dynamically constructed via cfg, conditionally compiled). Mitigation: 3c.12 grep audit + 3c.13 build clean + `gitnexus detect_changes` post-check.
- **Risk R4 (P0.11c)**: `collect_scheduler` fallback rewire to openstock breaks an unrelated execution flow. Mitigation: 3c.5 is the only scheduler edit, run full scheduler-related test suite before merge.
- **Rollback**: each sub-slice is one commit. Revert the offending commit; tdx-api path returns automatically (it's only deleted in P0.11c, not before).

---

## Reference

- Handoff: `docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md` (especially 2026-07-01 Status section)
- Live findings: `docs/proposals/openstock-live-integration-findings.md`
- openstock-side issue log: `/opt/claude/openstock/docs/operations/REMOTE_TEST_2026-07-01_ISSUES.md`
- Prior slice (P0.10): `openspec/changes/archive/2026-06-30-openstock-data-consumption-p0-10/`
- Shadow persistence (P0.8g-impl) — ClickHouse write path to mirror: `src/sources/openstock_shadow.rs`
