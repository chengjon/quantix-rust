# Tasks — OpenStock Data Consumption P0.10 (Live HTTP Wiring)

## 0. Baseline And Governance

- [x] 0.1 Baseline: P0.9 shipped at commit `2571003`; `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`, `openspec validate --all --strict` all green on master.
- [x] 0.2 Create `.governance/programs/project-governance/cards/P0.10.yaml` scoped to `openstock-data-consumption-p0-10/*` + the 5 in-tree paths being edited + the 3 new test files.
- [ ] 0.3 `openspec validate openstock-data-consumption-p0-10 --strict` passes.
- [ ] 0.4 (Deferred) `ft:new-node`/`ft:transition` for P0.10 — governance flow invocation deferred to closeout per P0.9 precedent.

## 1. Client G1 — HTTP Status Check In `fetch`

- [x] 1.1 In `src/sources/openstock_client.rs::fetch`, split `.send().await?.text().await?` into two steps; capture `status` before consuming body.
- [x] 1.2 On `!status.is_success()`: attempt `serde_json::from_str::<OpenStockErrorEnvelope>(&raw)`; on success surface `to_summary()`, on parse failure surface `HTTP {status} | body: {first 200 chars}`.
- [x] 1.3 On success: parse envelope directly (no more "try success then try error" double-attempt).

## 2. Client G2 — `OPENSTOCK_BASE_URL` Env Fallback

- [x] 2.1 In `OpenStockClient::new`, if `cfg.base_url.is_empty()` read `OPENSTOCK_BASE_URL` env; missing → `QuantixError::Config`.
- [x] 2.2 Mirror the existing `OPENSTOCK_API_KEY` fallback structure exactly.

## 3. Client — `from_env()` Constructor

- [x] 3.1 Add `pub fn from_env() -> Result<Self>` that calls `Self::new(OpenStockClientConfig::default())` (which now reads both env vars).
- [x] 3.2 Document that explicit `new(cfg)` API is unchanged.

## 4. Client — `OpenStockResponse::latency_ms`

- [x] 4.1 Add `pub latency_ms: Option<u64>` to `OpenStockResponse<T>`.
- [x] 4.2 Populate it from `envelope.latency_ms` in `from_envelope`.

## 5. CLI — 3 New `Fetch*` Variants

- [x] 5.1 In `src/cli/commands/data.rs` after `ValidateIndex`, add `FetchCodes` (no args).
- [x] 5.2 Add `FetchCalendar { year: u32 }`.
- [x] 5.3 Add `FetchIndex { symbol: String, start: Option<String>, end: Option<String> }`.

## 6. CLI — 3 Async Handlers

- [x] 6.1 Add `pub(crate) async fn fetch_openstock_codes()` to `openstock_handler.rs`.
- [x] 6.2 Add `pub(crate) async fn fetch_openstock_calendar(year: u32)`.
- [x] 6.3 Add `pub(crate) async fn fetch_openstock_index(symbol, start, end)`.
- [x] 6.4 Each handler: `OpenStockClient::from_env()? → fetch_*().await? → println! summary`.
- [x] 6.5 Drop dead `IndexKlineParseError` import + P0.9 placeholder `let _ = (IndexKlineParseError::EmptyRecords,)`.

## 7. CLI — Dispatcher + Re-exports

- [x] 7.1 In `src/cli/handlers/mod.rs`, add 3 imports to the `openstock_handler::{...}` block.
- [x] 7.2 In `src/cli/handlers/app_shell.rs`, add 3 dispatcher arms after `ValidateIndex` (`.await?` calls).

## 8. Live Integration Tests (`#[ignore]`)

- [x] 8.1 `tests/openstock_live_codes.rs` — `#[tokio::test] #[ignore]`, gated by `QUANTIX_OPENSTOCK_LIVE=1`, asserts `records` non-empty + `artifact_hash.len()==64`.
- [x] 8.2 `tests/openstock_live_calendar.rs` — same shape, uses `OPENSTOCK_LIVE_YEAR` (default 2026).
- [x] 8.3 `tests/openstock_live_index.rs` — same shape, uses `OPENSTOCK_LIVE_SYMBOL` (default `sh000001`).

## 9. Verification

- [ ] 9.1 `cargo fmt --all -- --check`.
- [ ] 9.2 `cargo clippy --all-targets --workspace -- -D warnings`.
- [ ] 9.3 `cargo test --lib --package quantix-cli openstock`.
- [ ] 9.4 `cargo test --test openstock_codes --test openstock_calendar --test openstock_index --test openstock_client`.
- [ ] 9.5 `cargo test --workspace` (new live tests skipped via `#[ignore]`).
- [ ] 9.6 `openspec validate openstock-data-consumption-p0-10 --strict`.
- [ ] 9.7 `openspec validate --all --strict`.
- [ ] 9.8 `gitnexus detect_changes` — expect LOW risk on edited paths.
- [ ] 9.9 `git diff --check`.
- [ ] 9.10 Manual live smoke (when runtime is reachable):
  ```
  OPENSTOCK_BASE_URL=http://192.168.123.104:8040 OPENSTOCK_API_KEY=<key> \
    cargo run -q -- openstock fetch-calendar --year 2026
  ```
- [ ] 9.11 Manual live tests:
  ```
  QUANTIX_OPENSTOCK_LIVE=1 OPENSTOCK_BASE_URL=... OPENSTOCK_API_KEY=... \
    cargo test --test openstock_live_codes --test openstock_live_calendar --test openstock_live_index -- --ignored
  ```
