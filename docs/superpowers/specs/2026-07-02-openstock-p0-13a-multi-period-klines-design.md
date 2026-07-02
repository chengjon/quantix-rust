# OpenStock P0.13a — Multi-period K-line Fetch (day/week/month + qfq/hfq)

Date: 2026-07-02
Status: Design (brainstorming output)
Scope: quantix-rust only (no openstock runtime changes)
Slice reference: P0.13a (first of 4 sub-slices covering the 8 ❌ gaps in HANDOFF report)

## Background

P0.11c removed `TdxApiClient` (commit `d1dda04`, 2,656 lines deleted). P0.12
landed the non-blocking follow-ups (commit `49d1136`). The
`HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md` §"openstock
侧已就绪但 quantix-rust 未接入" still lists 8 categories with ❌ on the
quantix-rust consumer side. P0.13a-d decompose those gaps into 4 slices;
this document covers P0.13a only.

**P0.13 slice plan** (decided 2026-07-02):

| Slice | Group | Categories | Reuses existing parser? |
|---|---|---|---|
| **P0.13a (this spec)** | B — multi-period K + adjust | `KLINES` / `ADJUSTED_KLINES` / `HISTORICAL_KLINES` (transparent via `/data/bars`) | Yes — `/data/bars` schema stable |
| P0.13b | A — minute-level | `MINUTE_DATA` | No — new parser |
| P0.13c | C — realtime quotes | `REALTIME_QUOTES` | No — new parser |
| P0.13d | D + E — tick read-only + financials/stats/search | `TICK_DATA` read-only + others | Mixed |

## Slice Goal

Enable quantix-rust to fetch A-share K-lines from OpenStock `/data/bars`
with `period ∈ {day, week, month}` and `adjust_type ∈ {None, Qfq, Hfq}`,
read-only (no DB writes). This covers the B-group row of the HANDOFF
table (周/月 K 线 + 不复权 — `不复权` is satisfied by `adjust_type=None`
which is already the default).

## Decisions (from brainstorming Q1-Q5 + approach C)

| Decision | Choice | Rationale |
|---|---|---|
| **Q1 Scope baseline** | C — week/month + qfq/hfq | Covers all three P1 items from HANDOFF §四 |
| **Q2 Adjust type source** | A — request-driven | OpenStock runtime does not echo `adjust_type` in response; shadow persistence chain is already request-driven |
| **Q3 Client API shape** | C — add `fetch_klines`, leave `fetch_daily_klines` unchanged | Zero disruption to existing market/backtest callers (P0.11) |
| **Q4 CLI shape** | C — single `FetchKlines` with `--period day\|week\|month` enum | Matches OpenStock `/data/bars` shape; P0.13b only needs to widen enum |
| **Q5 Test matrix** | C — full (3 fixture + 3 live + 1 wiremock + 1 qfq unit) | Covers request construction, response parsing, and end-to-end live paths |
| **Approach** | C — single OpenSpec change, phased commits | One governance card; 3 commits map to Phase 1/2/3 |

## Architecture

### Layer overview

```
CLI                          Handler                      Client                       OpenStock runtime
─────────────────────────    ─────────────────────────    ─────────────────────────    ─────────────────
data openstock fetch-klines  fetch_openstock_klines()     OpenStockClient::            POST /data/bars
  --symbol 600000     ──►     parse period/adjust   ──►    fetch_klines(          ──►   body: {
  --period week                OpenStockClient::from_env()   code,                          symbol: ...,
  --adjust qfq                 client.fetch_klines(...)      period,                        period: "week",
  --start 2026-01-01                                         adjust,                        adjust: "qfq",
  --end   2026-06-30                                         start,                         start_date: ...,
                                                             end)                           end_date: ...
                                                            )                            }
                                                           ◄──    Vec<Kline>          ◄── HTTP 200 + JSON
```

The new `fetch_klines` path mirrors `fetch_daily_klines` shape (direct
reqwest, **not** the generic `fetch<T>()` envelope path). This is the
P0.10 established design: `/data/bars` is a special endpoint with its
own response shape, distinct from `/data/fetch`.

### Invariants

1. **Response schema unchanged**: `/data/bars` returns `{data: [{time, open, high, low, close, volume, amount}, ...]}` for all periods and adjust types — verified against `_eltdx_timeseries.py:_PERIOD_MAP` (week/month handled same as day).
2. **`Kline` data model unchanged**: existing `Kline { code, date, open, high, low, close, volume, amount, adjust_type }` covers all 3 periods × 3 adjust types.
3. **No new DB writes**: read-only fetch only. ClickHouse / shadow persistence integration is out of scope (deferred to a later slice if needed).
4. **No new parser**: the inline JSON parsing in `fetch_daily_klines` shape (using local `BarsResponse` / `BarRecord` structs) is reused.

## Components

### 1. `KlinePeriod` enum

New type — placed in `src/data/models.rs` alongside `AdjustType` (decision
D5, to be confirmed at implementation time):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KlinePeriod {
    Day,
    Week,
    Month,
}

impl KlinePeriod {
    pub fn as_str(&self) -> &'static str { /* "day" | "week" | "month" */ }
}

impl FromStr for KlinePeriod {
    type Err = QuantixError;
    // Strict — rejects "daily"/"weekly"/"monthly" aliases and "minute*" (P0.13b).
}
```

**Strict enum rationale (decision D3)**: rejecting aliases prevents
confusion when CLI users try `--period daily` (which OpenStock silently
accepts via `_PERIOD_MAP` but quantix-rust should surface as a config
error to keep the surface predictable).

### 2. `AdjustType::as_openstock_param` helper

Added to existing `AdjustType` enum in `src/data/models.rs`:

```rust
impl AdjustType {
    pub fn as_openstock_param(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Qfq => "qfq",
            Self::Hfq => "hfq",
        }
    }
}
```

### 3. `OpenStockClient::fetch_klines` method

In `src/sources/openstock_client.rs`, after `fetch_daily_klines`:

```rust
/// Fetch OHLCV bars with explicit period and adjust type. Generalizes
/// `fetch_daily_klines` to week/month periods and qfq/hfq adjustment.
///
/// New CLI paths use this; `fetch_daily_klines` is preserved unchanged
/// for existing market/backtest callers.
///
/// `period` accepts `day` | `week` | `month` (P0.13a scope).
/// `adjust` is request-driven (runtime does not echo it; we stamp each
/// `Kline` with the requested `AdjustType`).
pub async fn fetch_klines(
    &self,
    code: &str,
    period: KlinePeriod,
    adjust: AdjustType,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<Vec<Kline>> {
    // Same shape as fetch_daily_klines, plus:
    //   body["period"] = period.as_str()
    //   body["adjust"] = adjust.as_openstock_param()
    //   each Kline gets `adjust_type: adjust` (not hardcoded None)
}
```

### 4. `FetchKlines` CLI subcommand

In `src/cli/commands/data.rs`, append to `OpenStockCommands`:

```rust
FetchKlines {
    #[arg(long)]
    symbol: String,
    #[arg(long, default_value = "day")]
    period: String,           // parsed to KlinePeriod in handler
    #[arg(long, default_value = "none")]
    adjust: String,           // parsed to AdjustType in handler
    #[arg(long)]
    start: Option<String>,
    #[arg(long)]
    end: Option<String>,
},
```

### 5. `fetch_openstock_klines` handler

In `src/cli/handlers/openstock_handler.rs`, mirror `fetch_openstock_index`
shape with extra `Period` / `Adjust` output lines.

### 6. Dispatcher + re-export

`src/cli/handlers/app_shell.rs`: new match arm.
`src/cli/handlers/mod.rs`: `pub(crate) use ... fetch_openstock_klines;`

## Data Flow

(See architecture diagram above. No DB writes; no caches; no side
effects beyond HTTP egress and stdout.)

## Error Handling

| Error source | Path | Outcome |
|---|---|---|
| Invalid `--period` string | Handler parses with `KlinePeriod::from_str` | `QuantixError::Config` — fail-fast, no HTTP |
| Invalid `--adjust` string | Handler parses (existing `AdjustType::from_str` if exists, else new helper) | `QuantixError::Config` — fail-fast, no HTTP |
| `OPENSTOCK_BASE_URL` / `OPENSTOCK_API_KEY` missing | `OpenStockClient::from_env` | `QuantixError::Config` — fail-fast |
| HTTP 4xx from `/data/bars` | Direct from reqwest, no retry (matches `fetch_daily_klines`) | `QuantixError::Other` — surfaced to CLI |
| HTTP 5xx from `/data/bars` | Same — no retry (matches `fetch_daily_klines`) | `QuantixError::Other` |
| Body JSON parse failure | Local `BarsResponse` deserialize | `QuantixError::DataParse` |
| Empty `data` array | Returns `Vec::new()` (matches `fetch_daily_klines` contract) | Empty result, no error |

**Note**: the new `fetch_klines` does NOT participate in circuit breaker
or retry semantics (matches `fetch_daily_klines`). This is the P0.10
design decision for `/data/bars` paths; the generic `fetch<T>()` envelope
path (with retry + breaker) is reserved for `/data/fetch` categories.
Widening this is out of scope for P0.13a.

## Testing

### Test matrix (8 tests across 3 layers)

| ID | Layer | File | Purpose |
|---|---|---|---|
| T1 | Unit | `src/sources/openstock_client.rs` `#[cfg(test)]` | `KlinePeriod` round-trip + strict rejection of aliases/minute |
| T2 | Unit | `src/data/models.rs` `#[cfg(test)]` or same file | `AdjustType::as_openstock_param` mapping for None/Qfq/Hfq |
| T3 | Wiremock | `src/sources/openstock_client.rs` `#[cfg(test)]` | day+None request body construction + response parsing |
| T4 | Wiremock | same | week+qfq request body construction + `adjust_type` stamping on each `Kline` |
| T5 | Wiremock | same | HTTP 4xx propagation |
| T6 | Live `#[ignore]` | `tests/openstock_live_klines.rs` | day+None end-to-end |
| T7 | Live `#[ignore]` | same | week+qfq end-to-end |
| T8 | Live `#[ignore]` | same | month+hfq end-to-end |

Live tests gated by `QUANTIX_OPENSTOCK_LIVE=1` env (matches P0.10/P0.11
convention).

### Quality gates (per Phase)

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --lib --package quantix-cli openstock
cargo test --test openstock_live_klines   # default skips via #[ignore]
cargo test --workspace                    # regression incl. new tests (skipped)
openspec validate openstock-data-consumption-p0-13a --strict
openspec validate --all --strict
gitnexus detect_changes                   # expect LOW on client + handlers
```

## Implementation Plan (3 phases, 3 commits)

### Phase 1 — Client method (commit 1)
- `KlinePeriod` enum + `FromStr` + tests (T1)
- `AdjustType::as_openstock_param` + test (T2)
- `OpenStockClient::fetch_klines` + wiremock tests (T3, T4, T5)
- `cargo fmt` / `clippy` / `cargo test --lib` pass

### Phase 2 — CLI wiring (commit 2)
- `FetchKlines` variant in `OpenStockCommands`
- `fetch_openstock_klines` handler
- `mod.rs` re-export + `app_shell.rs` dispatcher arm
- `cargo build` + `cargo test --workspace` pass (no new live tests yet)

### Phase 3 — Live verification + spec finalization (commit 3)
- `tests/openstock_live_klines.rs` with T6/T7/T8
- Manual live smoke against `http://192.168.123.104:8040` (when reachable)
- HANDOFF report table update (3 rows → ✅)
- `openspec validate ... --strict` passes
- OpenSpec change archived

## OpenSpec change layout

```
openspec/changes/openstock-data-consumption-p0-13a/
├── proposal.md       # Why / What Changes / Impact / Non-Goals
├── tasks.md          # Sections 0-4 mirroring the 3 phases above
├── design.md         # D1-D5 decisions (see "Decisions" table above)
└── specs/openstock-data-consumption/spec.md
                      # ### ADDED Requirements for multi-period + adjust passthrough
```

## Governance

### Card `.governance/programs/project-governance/cards/P0.13a.yaml`

Mirrors P0.10/P0.11.yaml structure (no `status:` field — retrospective-
friendly per P0.11 convention).

- `scope.allowed_paths` lists every file P0.13a may touch
- `scope.forbidden_paths` excludes `src/sources/openstock.rs`,
  `src/sources/openstock_index.rs`, `src/db/**`, `src/backtest/**`,
  `src/execution/**`
- `non_goals` enumerates minute period, ADJUST_FACTOR, eltdx KLINES
  direct, DB integration, fetch_daily_klines refactor
- `acceptance.commit_gate` lists `cargo fmt` / `clippy` / `test` /
  `openspec validate` + grep confirming `fetch_klines` exists
- `acceptance.closeout_gate` lists HANDOFF report table update +
  OpenSpec archive
- `evidence.current_head` filled at execution time

### Function tree (FUNCTION_TREE.md)

Updated in Phase 3 closeout: P0.13a entry added under OpenStock
consumer-side, marking 3 B-group categories as ✅.

## Out-of-Scope (deferred)

| Item | Defer to |
|---|---|
| minute-level periods (`MINUTE_DATA`) | P0.13b |
| `ADJUST_FACTOR` raw factor exposure | P0.13d+ |
| eltdx `KLINES` direct category wiring | not needed (covered via `/data/bars` transparently) |
| ClickHouse / shadow persistence integration for new periods | later slice |
| Refactor `fetch_daily_klines` to call `fetch_klines(code, Day, None, ...)` | later slice (low-value, risk-only) |
| Live shadow drift detection for adjust_type | later slice (Q2=A accepted risk; audit via `artifact_hash`) |
| Retry / circuit breaker for `/data/bars` path | later slice (P0.10 design; not regression) |

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| OpenStock `/data/bars` returns different schema for week/month | Low — `_eltdx_timeseries.py:_PERIOD_MAP` shows uniform handling | T7/T8 live tests will catch any drift |
| `adjust_type` request/response drift undetectable | Medium — accepted per Q2=A | Mitigated via `artifact_hash` on raw body (recorded but not acted on this slice) |
| `KlinePeriod::from_str` rejecting aliases breaks user expectation | Low — `--period daily` users will get clear error message | Document `day\|week\|month` in `--help` (default is `day`) |
| Governance card scope too narrow (forgot a file) | Low — based on P0.10/P0.11 patterns | `ft:new-node` will surface missing paths at execution |

## Success Criteria

1. `data openstock fetch-klines --symbol sh000001 --period week --adjust qfq --start 2026-01-01 --end 2026-06-30` returns ≥1 Kline with `adjust_type=Qfq` when OpenStock runtime is reachable.
2. All 8 tests pass (3 layers).
3. `cargo fmt` / `clippy` / `cargo test --workspace` / `openspec validate ... --strict` all green.
4. HANDOFF report B-group rows flip ❌ → ✅.
5. OpenSpec change `openstock-data-consumption-p0-13a` archived.

## Cross-references

- Source plan: `docs/superpowers/plans/2026-07-02-openstock-p0-13a-multi-period-klines-plan.md` (created by writing-plans skill — next step)
- HANDOFF report: `docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md`
- P0.11 closeout: `docs/reports/OPENSTOCK_DATA_CONSUMPTION_P0_11_CLOSEOUT_2026-07-02.md`
- Prior slice reference: `openspec/changes/openstock-data-consumption-p0-10/design.md` (shape mirror)
- OpenStock categories: `/opt/claude/openstock/docs/DATA_CAPABILITY_SCOPE.md`
- OpenStock `/data/bars` period map: `/opt/claude/openstock/openstock/adapters/_eltdx_timeseries.py:12`
