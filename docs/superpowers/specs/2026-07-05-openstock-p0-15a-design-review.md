# Review: docs/superpowers/specs/2026-07-05-openstock-p0-15a-minute-cli-persistence-design.md

**Date**: 2026-07-05
**Scope**: docs/superpowers/specs/2026-07-05-openstock-p0-15a-minute-cli-persistence-design.md
**File type**: md
**Doc type**: proposal (path 含 `review`，内容含 recommendation / scoring / verdict；自判 spec 不准确，按 meta-review L1 修正为 proposal)
**Perspectives**: Completeness (2x), Codebase Alignment (2x), Consistency

---

## Evidence Verification

### Files Referenced
| Claimed path | Exists? | Actual location |
|------|---------|----------|
| `src/cli/commands/data.rs:74` (ImportKlines) | yes | `data.rs:74` |
| `src/cli/commands/data.rs:176` (OpenStockCommands) | yes | `data.rs:176` |
| `src/cli/commands/data.rs:418` (FetchMinuteShare) | yes | `data.rs:418` |
| `src/cli/handlers/openstock_handler.rs:520` (fetch_openstock_minute_share) | yes | L520-608 |
| `src/cli/handlers/openstock_handler.rs:610` (fetch_openstock_all_stocks) | yes | L610 |
| `src/cli/handlers/app_shell.rs:407` (FetchMinuteShare arm) | yes | L407-420 |
| `src/data/models.rs:340-342` (from_cli error) | yes | L341: `"at least one of --date or (--start, --end) is required"` |
| `src/db/clickhouse/minute.rs` | yes | 269 lines, shipped in P0.14 |
| `src/db/clickhouse/mod.rs:12-14` (stream re-exports) | yes | `stream_minute_klines_to_clickhouse` + `stream_minute_shares_to_clickhouse` |
| `src/db/clickhouse/mod.rs:89-94` (ClickHouseClient) | yes | `with_default_config()` at L89, `client()` at L94 |

### Functions/Classes Referenced
| Symbol | Found? | Location |
|--------|--------|----------|
| `ImportKlines` | yes | `data.rs:74` |
| `OpenStockCommands` | yes | `data.rs:176` |
| `FetchMinuteShare` | yes | `data.rs:418` |
| `fetch_openstock_minute_share` | yes | `openstock_handler.rs:520` |
| `fetch_openstock_all_stocks` | yes | `openstock_handler.rs:610` |
| `DateOrRange::from_cli` | yes | `models.rs:297-342` |
| `ClickHouseClient::with_default_config` | yes | `mod.rs:89` |
| `ClickHouseClient::client` | yes | `mod.rs:94` (returns `&Client`) |
| `ClickHouseMinuteKlineSink` | yes | `minute.rs:136-138` — **has lifetime `'a`** (`ClickHouseMinuteKlineSink<'a>`) |
| `ClickHouseMinuteShareSink` | yes | `minute.rs:141-143` — **has lifetime `'a`** (`ClickHouseMinuteShareSink<'a>`) |
| `stream_minute_klines_to_clickhouse` | yes | `minute.rs:210` — **generic over `S: MinuteSink<MinuteKlineCH>`** |
| `stream_minute_shares_to_clickhouse` | yes | `minute.rs:244` — **generic over `S: MinuteSink<MinuteShareCH>`** |
| `MinuteSink<T>` trait | yes | `minute.rs:131` — `pub trait` (not pub(crate)) with `#[async_trait]` |
| `compute_apply` | no | not yet created |

### Claims Verified
| Claim | Status | Evidence |
|-------|--------|----------|
| `ImportKlines` exists at `data.rs:74` | confirmed | exact match |
| `from_cli(None, None, None)` returns error | confirmed | `models.rs:341`: `"at least one of --date or (--start, --end) is required"` |
| P0.14 consumers `stream_minute_*_to_clickhouse` shipped | confirmed | `minute.rs:210,244`, re-exported via `mod.rs:12-14` |
| `ClickHouseClient` has `with_default_config()` + `client()` | confirmed | `mod.rs:89,94` |
| `ClickHouseMinuteKlineSink { client: ch.client() }` construction (L162) | contradicted | actual struct is `ClickHouseMinuteKlineSink<'a>` with explicit lifetime param (`minute.rs:136`) |
| `stream_minute_klines_to_clickhouse` signature accepts `&sink` (L166-168) | confirmed | signature `sink: &S` where `S: MinuteSink<...>`, `ClickHouseMinuteKlineSink<'a>` implements `MinuteSink<MinuteKlineCH>` (`minute.rs:146`) |
| Handler insertion point after `fetch_openstock_minute_share` at L608, before `fetch_openstock_all_stocks` at L610 | confirmed | exact match |
| `FetchMinuteShare` at L418 for enum insertion | confirmed | exact match |
| Dispatcher insertion after L417 `FetchMinuteShare` | confirmed | `app_shell.rs:407-420` |

---

## Checklist Results

### Completeness
| # | Check | Result | Notes |
|---|-------|--------|-------|
| CP1 | All referenced files/symbols/paths exist | PASS | All 15 symbols verified |
| CP2 | Error states covered | PASS | INV-CLI-1 (gate refusal), INV-CLI-2 (no CH on dry-run), INV-FLOW-1 (partial failure), R2 (silent dry-run warning) |
| CP3 | Edge cases mentioned | PASS | `from_cli(None, None, None)` error (L100), `Date` variant defensive match (L126) |
| CP4 | Rollback / recovery described | PASS | INV-FLOW-1 explicitly documents no-rollback semantics + MergeTree dedup path |
| CP5 | Migration path for existing users | PASS | Non-goals: no existing user migration needed; additive CLI only |

### Codebase Alignment
| # | Check | Result | Notes |
|---|-------|--------|-------|
| CA1 | Referenced APIs exist with claimed signatures | FAIL | `ClickHouseMinuteKlineSink` constructor at L162 missing `<'a>` lifetime param (minute.rs:136). `stream_minute_*` function is generic over `S: MinuteSink`, not a concrete type — this works but the design doesn't document the generic constraint |
| CA2 | Referenced types exist with claimed fields | PASS | `ClickHouseMinuteKlineSink { client }` field validated at minute.rs:137 |
| CA3 | Claimed file paths are correct | PASS | All 10 file paths verified |
| CA4 | No name collisions | PASS | `import-minute-klines` / `import-minute-share` are new subcommand names |
| CA5 | Follows existing conventions | PASS | `import-` prefix (D1), double-key gate (INV-CLI-1), stdout/stderr split (D4) all match existing patterns |

### Consistency
| # | Check | Result | Notes |
|---|-------|--------|-------|
| CS1 | Terms match codebase definitions | PASS | `MinutePeriod`, `AdjustType`, `DateOrRange`, `OpenStockClient`, `ClickHouseClient` all consistent |
| CS2 | No internal contradictions | PASS | INV-CLI-1 through INV-CLI-5 and INV-FLOW-1 are internally consistent |
| CS3 | Dependencies between components correct | PASS | P0.14 → P0.15a dependency chain correct; P0.15a does not modify P0.14 |

15 items PASS (CP1-CP5, CA2-CA5, CS1-CS3).

---

## Findings

### Critical Issues
| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| C1 | §3.1 L162 | `ClickHouseMinuteKlineSink { client: ch.client() }` missing lifetime parameter | Compile error in apply branch: struct requires `ClickHouseMinuteKlineSink<'a>` | `minute.rs:136`: `pub(crate) struct ClickHouseMinuteKlineSink<'a>` | Add explicit lifetime: `let sink = ClickHouseMinuteKlineSink { client: ch.client() };` → `let sink = ClickHouseMinuteKlineSink::<'_> { client: ch.client() };` or let compiler infer via type annotation. R1 already flags this risk but doesn't give the fix. |

### Medium Issues
| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| M1 | §3.1 L162 | Apply branch uses bare `ClickHouseMinuteKlineSink` constructor without import path documentation | Handler won't compile without correct `use` — sink type is `pub(crate)` in `minute.rs`, accessible as `crate::db::clickhouse::minute::ClickHouseMinuteKlineSink` | `minute.rs:136` defines the struct; no re-export in `mod.rs` | Add explicit `use crate::db::clickhouse::minute::ClickHouseMinuteKlineSink;` (or `ClickHouseMinuteShareSink`) in the handler's inline import block. **[2026-07-06 更新]** re-export 已在实施提交 `9172648` 中添加（`src/db/clickhouse/mod.rs:19`: `pub(crate) use self::minute::{ClickHouseMinuteKlineSink, ClickHouseMinuteShareSink};`），此建议已被采纳。 |
| M2 | §7 D3 | `compute_apply(apply: bool, env: Option<&str>)` signature — param `env` shifts env-var reading from handle r to test caller | Test must hardcode `compute_apply(true, Some("yes"))` which doesn't verify the env var name is correct; actual env var reading in handler must still be tested separately | D3 says "Makes the gate logic testable" but U2/U3 testing `compute_apply(true, Some("yes"))` only tests `apply && env == "yes"` — a unit-test tautology | Consider `fn check_env_apply() -> bool` that reads the env var internally; U2/U3 would set env in `std::env::set_var` (with `serial_test` or `EnvVarGuard` if available) for real integration, or accept that `compute_apply` is a thin wrapper tested by handler integration |

### Low Issues
| # | Section | Issue | Evidence | Recommendation |
|---|---------|-------|----------|----------------|
| L1 | §2.1 L58-76 | `ImportMinuteKlines` uses `short = 'c'` for `--code` | Existing `ImportKlines` at `data.rs:74` uses `short` — verify exact attribute matches to avoid clap parse conflicts | Grep `ImportKlines` definition at `data.rs:74-92` to confirm `short` usage for `--code` |
| L2 | §3.1 L126 | `DateOrRange::Date(_) => return Err(...)` defensive arm uses Rust 2024 `let-else` pattern implicitly | Not an error but could be simplified with `let DateOrRange::Range { start, end } = dor else { return Err(...) };` | Consider `let-else` for conciseness |

---

## Strengths

- INV set is tight and testable: INV-CLI-1 (double-key gate), INV-CLI-2 (no CH on dry-run), INV-CLI-3 (stream-only), INV-CLI-4 (single code), INV-CLI-5 (range-only), INV-FLOW-1 (partial-failure semantics) — each maps to a concrete code location and can be verified without mocking
- D1 `import-` prefix decision correctly distinguishes three verb families: `fetch-*` (read-only), `persist-*` (shadow), `import-*` (canonical write) — matches `ImportKlines` exactly
- D2 single env var `QUANTIX_OPENSTOCK_MINUTE_APPLY` for both subcommands is operator-friendly — P0.15b scheduler will call both for each code
- INV-FLOW-1's partial-failure documentation is honest: no implicit rollback, duplicates filterable via `ORDER BY` — operator knows exactly what to do after a failed mid-range run
- D4 stdout/stderr split mirrors existing `--stream` behavior, preserving the operator's redirection pipeline
- D6 separate live test file correctly isolates fetch (P0.13d) from import (P0.15a) concerns
- R1 proactively anticipates lifetime friction and delegates resolution to the implementation task — appropriate for a design doc

---

## Recommendations

1. Fix `ClickHouseMinuteKlineSink` construction with explicit lifetime or type annotation (C1)
2. Add import paths for `ClickHouseMinuteKlineSink` / `ClickHouseMinuteShareSink` in handler code (M1)
3. Clarify `compute_apply`'s relationship to `std::env::var` — either make it env-aware internally or accept it's a thin wrapper tested only through handler integration (M2)
4. Verify `ImportKlines` CLI attributes for exact `short` usage before implementing (L1)

---

## Scoring

| Dimension | Score (1-5) | Evidence |
|-----------|-------------|----------|
| Technical Accuracy | 4 | One compile-blocker (C1: lifetime); otherwise accurate on all API signatures and file locations |
| Completeness (2x) | 5 | Error states, edge cases, rollback, migration path, non-goals all covered |
| Codebase Alignment (2x) | 4 | Strong on conventions; C1/M1 are minor construction/import gaps |
| Actionability | 5 | File list, LOC estimates, acceptance gates, CLI smoke commands all concrete |
| Terminology Consistency | 5 | Consistent with D1 verb taxonomy, P0.14 types, existing handler patterns |
| **Overall** | **4.6** | Weighted: (4+5×2+4×2+5+5)/7 = 4.6 |

---

## Verdict

APPROVE_WITH_NOTES

Design is well-structured — INV set is tight, D1-D6 decisions are clear, and the dual-path (dry-run / apply) handler design faithfully mirrors `ImportKlines`. One compile-blocker (C1: `ClickHouseMinuteKlineSink` constructor missing lifetime) identified in R1 needs explicit resolution in the apply branch. Two handler import/documentation gaps (M1, M2) should be addressed before implementation. No design-level rework needed.
