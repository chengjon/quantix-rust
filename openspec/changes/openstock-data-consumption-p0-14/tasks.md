# Tasks: openstock-data-consumption-p0-14

Authoritative source: `docs/superpowers/plans/2026-07-05-openstock-p0-14-clickhouse-minute-persistence-plan.md`
Design rationale: `docs/superpowers/specs/2026-07-04-openstock-p0-14-clickhouse-minute-persistence-design.md`

## 0. Baseline And Governance

- [ ] Create `.governance/programs/project-governance/cards/P0.14.yaml` (scope: db/clickhouse subtree + openspec change + spec/plan docs)
- [ ] Confirm HEAD = master tip with P0.13d merged
- [ ] `cargo fmt --all -- --check && cargo clippy --all-targets --workspace -- -D warnings && cargo test --workspace` baseline green

## 1. DDL + Models (T1)

- [ ] Add `MinuteKlineCH` / `MinuteShareCH` to `src/db/clickhouse/models.rs`
- [ ] Add `create_minute_klines_table` / `create_minute_shares_table` to `src/db/clickhouse/schema.rs`
- [ ] Append both calls in `init_database()` (INV-1A)
- [ ] Verify existing clickhouse tests still pass

## 2. Conversion helpers + Sink trait + Stream consumers (T2)

- [ ] Create `src/db/clickhouse/minute.rs` with `decimal_to_f64` / `naive_to_utc` / `period_as_str` / `adjust_as_str` / `bar_to_row` / `share_to_row`
- [ ] Add `pub(crate) trait MinuteSink<T>` + `ClickHouseMinuteKlineSink` / `ClickHouseMinuteShareSink`
- [ ] Add `pub struct StreamStats` + `pub stream_minute_{klines,shares}_to_clickhouse`
- [ ] Add `_for_test` test-only exposure for private helpers
- [ ] Register `pub mod minute` + `pub use` in `mod.rs`

## 3. Unit tests U1–U8

- [ ] U1 decimal_to_f64 normal range
- [ ] U2 decimal_to_f64 extreme fallback
- [ ] U3 naive_to_utc wall-clock preservation
- [ ] U4 bar_to_row field mapping
- [ ] U5 share_to_row field mapping
- [ ] U6 mock sink compiles + constructs
- [ ] U7 share sink same
- [ ] U8 short-circuit on sink failure (INV-3A/3C)

## 4. Live tests L1 + L2

- [ ] L1 live_stream_minute_klines_to_clickhouse_round_trip (`#[ignore]` + env gate)
- [ ] L2 live_stream_minute_shares_to_clickhouse_round_trip (`#[ignore]` + env gate)

## 5. OpenSpec change

- [ ] `proposal.md` / `tasks.md` / `design.md` / `specs/openstock-data-consumption/spec.md`
- [ ] `openspec validate openstock-data-consumption-p0-14 --strict`

## 6. Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --workspace -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `openspec validate --all --strict`
- [ ] `gitnexus detect_changes` (expect LOW, only db/clickhouse touched)
