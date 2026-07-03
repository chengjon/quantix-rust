# Tasks: openstock-data-consumption-p0-13b-2

## 0. Baseline and Governance

- [x] Confirm HEAD is at the post-P0.13b-1 merge commit
- [x] Create `.governance/programs/project-governance/cards/P0.13b-2.yaml`
      with allowed_paths covering all files touched in this slice and
      forbidden_paths excluding P0.13b-1 symbols

## 1. MinuteShare Model (Task 1)

- [x] Add `MinuteShare` struct to `src/data/models.rs` with Option-wrapped
      business fields
- [x] Add unit test `minute_share_round_trip_serde`
- [x] Add unit test `minute_share_allows_missing_optional_fields`

## 2. Client Method (Task 2)

- [x] Add `RawMinuteRecord` struct with `Option<Decimal>` business fields
- [x] Add `fetch_minute_share` method calling
      `self.fetch::<RawMinuteRecord>("MINUTE_DATA", params)`
- [x] Add `parse_minute_share` returning `Option<MinuteShare>`
- [x] Add `parse_time_minutes` accepting "0930" and "09:30" formats
- [x] Add wiremock test `fetch_minute_share_sends_minute_data_category_and_date`
- [x] Add wiremock test `fetch_minute_share_skips_records_with_missing_required_field`
- [x] Add wiremock test `fetch_minute_share_propagates_4xx`
- [x] Add unit tests for `parse_time_minutes`

## 3. CLI Wiring (Task 3)

- [x] Add `FetchMinuteShare` variant to `OpenStockCommands` enum
- [x] Add `fetch_openstock_minute_share` handler
- [x] Re-export in `src/cli/handlers/mod.rs`
- [x] Add dispatcher arm in `app_shell.rs`

## 4. Live Tests (Task 4)

- [x] Create `tests/openstock_live_minute_share.rs` with L1/L2/L3 tests
- [x] All tests `#[ignore]` + `QUANTIX_OPENSTOCK_LIVE=1` env gate

## 5. OpenSpec Change

- [x] proposal.md, tasks.md, design.md, spec deltas

## 6. Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --workspace -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `openspec validate openstock-data-consumption-p0-13b-2 --strict`
- [ ] `openspec validate --all --strict`
- [ ] `gitnexus detect_changes` — expect LOW risk
