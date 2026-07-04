# Tasks: openstock-data-consumption-p0-13c

Authoritative source: `docs/superpowers/plans/2026-07-03-openstock-p0-13c-multi-day-range-plan.md`
Design rationale: `docs/superpowers/specs/2026-07-03-openstock-p0-13c-multi-day-range-design.md`

## 0. Baseline and Governance

- [x] Confirm HEAD at post-P0.13b-2 merge commit
- [x] Create `.governance/programs/project-governance/cards/P0.13c.yaml`
      with allowed_paths covering all files touched and forbidden_paths
      excluding P0.13a/P0.13b-1/P0.13b-2 symbols

## 1. DateOrRange Model (Task 1)

- [x] Add `DateOrRange` enum (`Date(NaiveDate)` | `Range { start, end }`) to `src/data/models.rs`
- [x] Add `from_cli(date, start, end)` validator enforcing mutex + rejecting
      semi-open ranges and `(None, None, None)`
- [x] Add `iter_dates_inclusive(start, end)` calendar-day iterator helper
- [x] Add unit tests U1-U7 (date-only, range, semi-open errors, conflict,
      start>end, all-none) + iterator tests

## 2. fetch_minute_klines Range (Task 2)

- [x] Extend signature: `fetch_minute_klines(code, period, DateOrRange, adjust)`
- [x] Date branch -> `params.date` (backward-compat with P0.13b-1)
- [x] Range branch -> `params.start_date` + `params.end_date`
- [x] Update existing wiremock tests to call with `DateOrRange::Date(...)`
- [x] Add wiremock W1: range sends `start_date`/`end_date`, omits `date`
- [x] Add wiremock W2: Date path omits `start_date`/`end_date`

## 3. fetch_minute_share Range (Task 3)

- [x] Extend signature: `fetch_minute_share(code, DateOrRange)`
- [x] Refactor body into `fetch_minute_share_single(code, date)` helper
- [x] Range branch loops `iter_dates_inclusive(start, end)`, aggregating results
- [x] Read `meta.trading_date` from each response envelope for record date
      (INV-2C: non-trading days return empty records, day skipped)
- [x] Update existing wiremock tests to call with `DateOrRange::Date(...)`
- [x] Add wiremock W3: range triggers N single-day requests
- [x] Add wiremock W5: range skips non-trading days

## 4. CLI Wiring (Task 4)

- [x] `FetchMinuteKlines` + `FetchMinuteShare`: `date: Option<String>` + new `--start`/`--end`
- [x] Handlers accept `(date, start, end)` triples; call `from_cli` for validation
- [x] Dispatcher arms pass new params through
- [x] Backward compat: `--date X` still works identically to P0.13b-1/2

## 5. Live Tests (Task 5)

- [x] Append L1 multi-day range test to `tests/openstock_live_minute_klines.rs`
- [x] Append L2 multi-day range test to `tests/openstock_live_minute_share.rs`
- [x] Append L3 from_cli rejection test (pure validation, CI-safe, no `#[ignore]`)

## 6. OpenSpec Change

- [x] Create `openspec/changes/openstock-data-consumption-p0-13c/` with
      proposal.md, tasks.md, design.md, specs/openstock-data-consumption/spec.md

## 7. Governance Transition

- [x] Initial state: `state: in_progress`
- [ ] Flip to `state: completed` after all verification gates pass (separate commit)

## 8. Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --workspace -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `openspec validate openstock-data-consumption-p0-13c --strict`
- [ ] `openspec validate --all --strict`
- [ ] `gitnexus detect_changes` — expect LOW risk
