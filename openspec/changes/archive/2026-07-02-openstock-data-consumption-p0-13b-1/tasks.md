# OpenStock Data Consumption P0.13b-1 — Tasks

## 1. Baseline And Governance
- [x] Create `.governance/programs/project-governance/cards/P0.13b-1.yaml`
- [x] Confirm clean working tree (P0.13a merged, R1 spec revisions committed)

## 2. Data Models (`src/data/models.rs`)
- [x] Add `MinutePeriod` enum (Minute1/5/15/30/60) with `as_str()` + strict `FromStr`
- [x] Add `MinuteBar` struct with `NaiveDateTime` timestamp
- [x] Add 3 unit tests (as_str round trip, canonical accept, alias reject)

## 3. Client Method (`src/sources/openstock_client.rs`)
- [x] Add `fetch_minute_klines(code, period, date, adjust) -> Vec<MinuteBar>`
- [x] 3 wiremock tests (1m+none, 5m+qfq, 15m+4xx-no-retry)

## 4. CLI Wiring
- [x] Add `FetchMinuteKlines` variant to `OpenStockCommands` (`src/cli/commands/data.rs`)
- [x] Add `fetch_openstock_minute_klines` handler (`src/cli/handlers/openstock_handler.rs`)
- [x] Re-export in `src/cli/handlers/mod.rs`
- [x] Add dispatcher arm (`src/cli/handlers/app_shell.rs`)

## 5. Live Tests
- [x] Create `tests/openstock_live_minute_klines.rs` with 3 `#[ignore]` tests

## 6. Quality Gates
- [x] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --workspace`
- [x] `openspec validate openstock-data-consumption-p0-13b-1 --strict`

## 7. HANDOFF Report Correction
- [ ] Update `docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md`
      row 35: correct `MINUTE_DATA` mislabel (clarify minute candles go via KLINES/`/data/bars`,
      not MINUTE_DATA). Mark B-group minute-candles row as ✅ P0.13b-1.

## 8. Archive
- [ ] `openspec archive openstock-data-consumption-p0-13b-1` (after merge)
- [ ] Governance: mark P0.13b-1 card state as `completed`
