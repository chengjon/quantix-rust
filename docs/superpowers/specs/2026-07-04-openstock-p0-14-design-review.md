# Review: docs/superpowers/specs/2026-07-04-openstock-p0-14-clickhouse-minute-persistence-design.md

**Date**: 2026-07-05
**Scope**: docs/superpowers/specs/2026-07-04-openstock-p0-14-clickhouse-minute-persistence-design.md
**File type**: md
**Doc type**: arch (path: design, content: component boundaries, DDL schema, data flow)
**Perspectives**: Codebase Alignment (2x), Terminology Consistency (2x), Feasibility

---

## Evidence Verification

### Files Referenced
| Claimed path | Exists? | Actual location |
|------|---------|----------|
| `src/sources/openstock_client.rs::fetch_minute_klines_stream` | yes | `src/sources/openstock_client.rs:721` |
| `src/sources/openstock_client.rs::fetch_minute_share_stream` | yes | `src/sources/openstock_client.rs:900` |
| `src/db/clickhouse/models.rs` | yes | `src/db/clickhouse/models.rs` (210 lines) |
| `src/db/clickhouse/schema.rs` | yes | `src/db/clickhouse/schema.rs` (218 lines) |
| `src/db/clickhouse/kline.rs` | yes | `src/db/clickhouse/kline.rs` (360 lines) |
| `src/db/clickhouse/fundamentals.rs` | yes | `src/db/clickhouse/fundamentals.rs` (80 lines) |
| `src/data/models.rs::MinuteBar` | yes | `src/data/models.rs:138-148` |
| `src/data/models.rs::MinuteShare` | yes | `src/data/models.rs:161-168` |
| `src/db/clickhouse/minute.rs` (proposed new) | no | not yet created |
| `tests/clickhouse_live_minute_klines.rs` (proposed new) | no | not yet created |

### Functions/Classes Referenced
| Symbol | Found? | Location |
|--------|--------|----------|
| `MinuteBar` | yes | `src/data/models.rs:138` |
| `MinuteShare` | yes | `src/data/models.rs:161` |
| `MinutePeriod` | yes | `src/data/models.rs:108` |
| `AdjustType` | yes | `src/data/models.rs:25` |
| `DateOrRange` | yes | `src/data/models.rs:287` |
| `iter_dates_inclusive` | yes | `src/data/models.rs:360` |
| `OpenStockClient` | yes | `src/sources/openstock_client.rs:87` (no generic lifetime) |
| `ClickHouseClient` | yes | `src/db/clickhouse/` (mod.rs) |
| `MinuteKlineCH` | no | not yet created |
| `MinuteShareCH` | no | not yet created |
| `Decimal::try_into` (for f64) | yes | rust_decimal 1.33; `use rust_decimal::prelude::TryInto as _` |
| `clickhouse::Row` derive | yes | `src/db/clickhouse/models.rs` (9 existing structs) |

### Claims Verified
| Claim | Status | Evidence |
|-------|--------|----------|
| P0.13d stream API exists | confirmed | `fetch_minute_klines_stream` at L721, `fetch_minute_share_stream` at L900 |
| `async_insert=1` + `wait_for_async_insert=1` existing pattern | confirmed | `kline.rs:204-205`, `fundamentals.rs:23-24`, `gbbq.rs:61-62` |
| `ON CLUSTER '{cluster}'` existing pattern | confirmed | `schema.rs:35,61,99,134,172` (5 tables) |
| `MinuteBar.volume` is `Decimal` (design L66) | contradicted | actual `MinuteBar.volume` is `i64` (`models.rs:145`), not `Decimal` |
| `MinuteBar.amount` is `Decimal` (design L67) | contradicted | actual `MinuteBar.amount` is `Option<Decimal>` (`models.rs:146`) |
| `MinuteShare` fields are non-Option (design L102-106) | contradicted | all 5 business fields are `Option<_>` (`models.rs:164-167`), parser guarantees non-None at write time |
| `OpenStockClient<'_>` generic lifetime (design L255) | contradicted | `OpenStockClient` has no generic params (`openstock_client.rs:87`); should be `&OpenStockClient` |
| `futures` crate needed (design R2 note for P0.13d pattern) | unverified | design does not claim new `futures` dep; `futures = "0.3"` already in `Cargo.toml:38` |
| A-share numeric range safe for f64 | confirmed | prices [0.01, 9999.99] << 2^53 mantissa |
| China no DST, +08:00 == Asia/Shanghai | confirmed | historical truth |

---

## Checklist Results

### Codebase Alignment
| # | Check | Result | Notes |
|---|-------|--------|-------|
| CA1 | Referenced APIs exist with claimed signatures | FAIL | `fetch_minute_klines_stream` exists at L721; `fetch_minute_share_stream` at L900 — but `MinuteSink<T>` trait, `ClickHouseMinuteKlineSink`, `MinuteKlineCH`, `MinuteShareCH` do not exist yet (new code) |
| CA2 | Referenced types exist with claimed fields | FAIL | `MinuteBar.volume` claimed as `Decimal`, actually `i64` (L66). `MinuteBar.amount` claimed as `Decimal`, actually `Option<Decimal>` (L67). `MinuteShare` price/volume/amount/avg_price claimed as non-Option, actually `Option<_>` (L102-106) |
| CA3 | Claimed file paths are correct | PASS | `src/sources/openstock_client.rs`, `src/db/clickhouse/models.rs`, `src/db/clickhouse/schema.rs`, `src/data/models.rs` all verified |
| CA4 | No name collisions with existing types/functions | PASS | `MinuteKlineCH`, `MinuteShareCH`, `MinuteSink` are all new names; no existing conflicts found (grep: `src/db/clickhouse/`)  |
| CA5 | Follows existing codebase conventions | PASS | `async_insert=1` insert pattern + `ON CLUSTER` DDL + `clickhouse::Row` derive all matched to existing code |

10 items PASS (CA3-CA5 + all Terminology + 5 Feasibility items).

### Terminology Consistency
| # | Check | Result | Notes |
|---|-------|--------|-------|
| TC1 | Terms match codebase definitions | FAIL | `volume: Decimal` should be `volume: i64` (L66); `amount: Decimal` should be `amount: Option<Decimal>` (L67); `price/volume/amount/avg_price` should be `Option<...>` (L102-106) |
| TC2 | No internal contradictions | PASS | Table naming (`minute_klines`/`minute_shares`) consistent throughout |
| TC3 | Abbreviations expanded on first use | PASS | DDL, Rust structs, CH (ClickHouse) all expanded |
| TC4 | Naming follows project conventions | PASS | `MinuteKlineCH` matches existing `KlineDataCH`, `StockInfoCH` pattern |

### Feasibility
| # | Check | Result | Notes |
|---|-------|--------|-------|
| F1 | Dependencies exist | PASS | `clickhouse = "0.12"`, `futures = "0.3"`, `rust_decimal = "1.33"`, `chrono` all in `Cargo.toml` |
| F2 | No impossible claims | PASS | No frozen API mutations claimed |
| F3 | Resource estimates in-bounds | PASS | +560 loc across 7 files, well within limits |
| F4 | Timeline realistic | PASS | Single-slice scope, additive only |
| F5 | Prerequisites explicitly listed | PASS | P0.13d merged, DDL pattern established |

---

## Findings

### Critical Issues
| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| C1 | §1.1 L66 | `MinuteBar.volume` claimed as `Decimal` — actual type is `i64` | `bar_to_row` would fail to compile using `decimal_to_f64(bar.volume)` | `models.rs:145`: `pub volume: i64` | Change field mapping to `volume: i64 → bar.volume as f64`, with comment noting `i64→f64` cast is lossless for A-share range (≤ 10^9) |

### Medium Issues
| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| M1 | §1.1 L67 + §1.2 L102-106 | 5 fields shown as non-Option types; actual types are `Option<Decimal>` / `Option<i64>` | `bar_to_row`/`share_to_row` signatures ambiguous — implementer may write wrong unwrap path | `models.rs:146` (`Option<Decimal>`), `models.rs:164-167` (all `Option<_>`) | Annotate field mapping table: add `Option` column, note "parser guarantees non-None at write time; unwrap in bar_to_row/share_to_row" |
| M2 | §2.3 L255 | `client: &OpenStockClient<'_>` uses spurious generic lifetime | Compile error: `OpenStockClient` has no generic params | `openstock_client.rs:87`: `pub struct OpenStockClient {` (no generics) | Change to `client: &OpenStockClient` |
| M3 | §5.1 schema.rs | States new `create_minute_*_table()` functions but does not explicitly say to register them in `init_database()` | `init_database()` callsite could be missed in implementation | Existing pattern: `init_database()` calls multiple `create_*_table()` methods (`schema.rs:10`) | Add step: "Register `create_minute_klines_table()` and `create_minute_shares_table()` in `init_database()`" |
| M4 | §2.1 | `MinuteKlineCH`/`MinuteShareCH` Row derive for Enum8 columns not expanded | `clickhouse::Row` for `Enum8('1m'=1, ...)` requires Rust enum + Row derive — design does not specify the Rust-side enum types | `clickhouse = "0.12"` requires explicit enum mapping | Add two Rust enums (`PeriodCH` / `AdjustCH`) with Row derive in the models.rs section |

### Low Issues
| # | Section | Issue | Evidence | Recommendation |
|---|---------|-------|----------|----------------|
| L1 | §2.1 L172 | `use chrono::{DateTime, FixedOffset, NaiveDate}` may trigger clippy `unused_import` warning for `NaiveDate` if only used through `DateOrRange` | NaiveDate indirectly available via DateOrRange | Audit the use block; remove `NaiveDate` if unused directly |

---

## Strengths

- DDL design aligns perfectly with existing ClickHouse conventions — `ON CLUSTER '{cluster}'`, MergeTree engine, `async_insert=1` + `wait_for_async_insert=1` all match the 5 existing tables in `schema.rs` and the insert patterns in `kline.rs`/`fundamentals.rs`/`gbbq.rs`
- P0.13 series boundary is cleanly respected — no modifications to `src/sources/**` or `src/cli/**`; all 6 upstream types (`MinuteBar`, `MinuteShare`, `DateOrRange`, `MinutePeriod`, `AdjustType`, `iter_dates_inclusive`) are consumed read-only
- D2 timezone decision (FixedOffset +08:00, no chrono-tz) is well-reasoned — China has no DST history, +08:00 is always equivalent to Asia/Shanghai
- `decimal_to_f64` overflow handling with `try_into` + `tracing::warn!` + `StreamStats::skipped_records` is a thoughtful defense-in-depth pattern — single bad value won't block the entire stream
- INV-4 (Sink trait pub(crate) only) correctly scopes the mock injection boundary — trait exists only for tests, public API stays simple
- Test matrix (8 unit + 2 live) covers every INV explicitly mapped in §4.3

---

## Recommendations

1. Fix `MinuteBar.volume` type in field mapping (i64, not Decimal) — this is a compile-breaking error in `bar_to_row`
2. Add `Option` annotations to all 5 MinuteShare fields + `MinuteBar.amount` in the mapping tables, with a one-line note that parser guarantees non-None
3. Remove `<'_>` from `&OpenStockClient<'_>` — pure syntax error
4. Specify the Rust-side Enum8 types needed for `MinuteKlineCH.period` / `adjust` Row derive
5. After fixes, the implementation plan can follow the existing `kline.rs` + `schema.rs` patterns directly

---

## Scoring

| Dimension | Score (1-5) | Evidence |
|-----------|-------------|----------|
| Technical Accuracy | 3 | DDL/insert patterns accurate; 3 field type mapping errors (C1, M1, M2) |
| Completeness | 4 | INV/DDL/test matrix full coverage; missing Enum8 Row types (M4) and init_database wiring (M3) |
| Codebase Alignment (2x) | 3 | Strong on conventions (DDL, insert); weak on upstream type fidelity (C1, M1) |
| Actionability | 4 | File list, loc estimates, phase structure all concrete |
| Terminology Consistency (2x) | 3 | Naming conventions consistent; field type terms inaccurate (C1, M1) |
| **Overall** | **3.3** | Weighted: (3+4+3×2+4+3×2)/7 = 3.3 |

---

## Verdict

NEEDS_REVISION

The DDL and insert-path design is solid and aligns perfectly with existing ClickHouse conventions. However, three field type mapping errors exist in §1.1/§1.2 — `MinuteBar.volume` is `i64` not `Decimal`, both `MinuteBar.amount` and all `MinuteShare` numeric fields are `Option<_>` not bare types. Additionally, `OpenStockClient<'_>` has a spurious generic lifetime. These are blocking for implementation — `bar_to_row` would fail to compile as written. Fix the type mappings and the review moves to APPROVE_WITH_NOTES (remaining M3/M4 are low-risk clarifications).
