# Tasks: openstock-data-consumption-p0-13d

Authoritative source: `docs/superpowers/plans/2026-07-03-openstock-p0-13d-streaming-plan.md`
Design rationale: `docs/superpowers/specs/2026-07-03-openstock-p0-13d-streaming-design.md`

## 0. Baseline and Governance

- [ ] Confirm HEAD at post-P0.13c merge commit (P0.13c shipped & archived)
- [ ] Create `.governance/programs/project-governance/cards/P0.13d.yaml`
      with `allowed_paths` covering all files touched and `forbidden_paths`
      excluding unrelated modules (db / backtest / execution / etc.)
- [ ] If `ft:new-node` governance CLI is available, register P0.13d node;
      otherwise note "manual yaml placement" (this slice uses direct file write)

## 1. chunk_range_weekly Pure Function (Task 1)

- [ ] Add private `fn chunk_range_weekly(start: NaiveDate, end: NaiveDate) -> Vec<(NaiveDate, NaiveDate)>`
      to `src/sources/openstock_client.rs`
- [ ] Iterate `start + 7 days` per chunk (not natural-week; uniform ≤ 7 days each)
- [ ] Final chunk covers remaining days (≤ 7)
- [ ] Add unit tests S1 (single day → 1 chunk), S2 (exact 7 days → 1 chunk),
      S3 (8 days → 2 chunks), S4 (long-range endpoint coverage + total span)

## 2. fetch_minute_klines_range Helper Extraction (Task 2)

- [ ] Extract private `fetch_minute_klines_range(code, period, start, end, adjust)`
      from existing `fetch_minute_klines` Range-branch body
- [ ] Existing `fetch_minute_klines` calls the helper in its Range arm (no behavior change)
- [ ] Date arm of `fetch_minute_klines` unchanged (P0.13b-1/c wire body preserved)

## 3. Stream API Methods (Task 3)

- [ ] Add `fetch_minute_klines_stream(code, period, DateOrRange, adjust) -> impl Stream<Item = Result<Vec<MinuteBar>, QuantixError>>`
      (weekly chunks via `chunk_range_weekly` + `fetch_minute_klines_range`)
- [ ] Add `fetch_minute_share_stream(code, DateOrRange) -> impl Stream<Item = Result<Vec<MinuteShare>, QuantixError>>`
      (per-calendar-day batches via `fetch_minute_share_single`; non-trading days yield `vec![]`)
- [ ] Date variant yields a single batch (size == 1)
- [ ] First batch error terminates the stream (subsequent `next()` returns `None`)

## 4. Stream Unit Tests (Task 4)

- [ ] S5 `fetch_minute_klines_stream_collects_same_as_batch` (INV-1A equivalence)
- [ ] S6 `stream_terminates_on_first_batch_error` (INV-5A)
- [ ] S7 `share_stream_yields_empty_vec_for_non_trading_days` (INV-5B)
- [ ] All stream unit tests use injected mock helper (no live HTTP)

## 5. Stream Wiremock Tests (Task 5)

- [ ] W1 `fetch_minute_klines_stream_emits_weekly_subrange_body`
      (multi-week range → N requests; each body has sub_start/sub_end, no `date`)
- [ ] W2 `fetch_minute_share_stream_emits_one_request_per_calendar_day`
      (7-day range → 7 requests)
- [ ] W3 `fetch_minute_klines_stream_date_mode_emits_single_batch`
      (`Date(d)` → 1 request with `date` field)

## 6. CLI --stream Flag (Task 6)

- [ ] Add `stream: bool` field to `FetchMinuteKlines` and `FetchMinuteShare` in
      `src/cli/commands/data.rs` (default `false`)
- [ ] Handlers in `src/cli/handlers/openstock_handler.rs`: when `stream == true`,
      take the streaming branch and print per-batch progress to stderr
- [ ] `src/cli/handlers/app_shell.rs`: destructure `stream` in the 2 dispatcher arms
- [ ] When `stream == false` (default), behavior is byte-identical to P0.13c

## 7. Live Tests (Task 7)

- [ ] Append L1 `live_fetch_minute_klines_stream_multi_week_range` (#[ignore])
      to `tests/openstock_live_minute_klines.rs` — asserts stream == batch result
- [ ] Append L2 `live_fetch_minute_share_stream_one_day_per_batch` (#[ignore])
      to `tests/openstock_live_minute_share.rs` — asserts batch count == calendar days

## 8. Final Validation (Task 8)

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --workspace -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `openspec validate openstock-data-consumption-p0-13d --strict`
- [ ] `openspec validate --all --strict`
- [ ] `gitnexus detect_changes` — expect LOW risk

## 9. Verification

- [ ] All P0.13a/b/c wiremock + live + unit tests pass zero-modified
- [ ] INV-1A (stream/batch equivalence) verified by S5 + L1
- [ ] INV-2A/2B (wire shape per batch) verified by W1/W2/W3
- [ ] INV-4A (batch API unchanged) verified by zero-modified P0.13a/b/c tests
- [ ] INV-5A/5B (error termination / non-trading-day empty Vec) verified by S6/S7
- [ ] Governance card `P0.13d.yaml` `state: in_progress` -> flip to `completed`
      in a separate follow-up commit after all gates pass
