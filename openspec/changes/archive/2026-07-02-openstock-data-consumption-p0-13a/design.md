# Design — OpenStock Data Consumption P0.13a (Multi-period K-line Fetch)

Source: `docs/superpowers/specs/2026-07-02-openstock-p0-13a-multi-period-klines-design.md`

## Decisions

| Decision | Choice | Rationale |
|---|---|---|
| **D1 Scope baseline** | C — day/week/month + qfq/hfq | Covers all three P1 items from HANDOFF §四 (`day` is included as default; P0.13a normalizes it through the new path) |
| **D2 Adjust type source** | A — request-driven | OpenStock runtime does not echo `adjust_type` in response; shadow persistence chain is already request-driven |
| **D3 Client API shape** | C — add `fetch_klines`, leave `fetch_daily_klines` unchanged | Zero disruption to existing market/backtest callers (P0.11) |
| **D4 CLI shape** | C — single `FetchKlines` with `--period day\|week\|month` strict enum | Matches OpenStock `/data/bars` shape; P0.13b only needs to widen enum |
| **D5 Test matrix** | C — full (5 unit/wiremock + 3 live = 8 tests across 3 layers) | Covers request construction, response parsing, and end-to-end live paths |
| **D6 Period enum strictness** | Reject `daily`/`weekly`/`monthly`/`minute*` aliases; case-insensitive on input | Surface predictable error rather than let OpenStock silently map them; case-insensitivity matches `AdjustType::FromStr` (D8) |
| **D7 OpenSpec approach** | C — single OpenSpec change, phased commits | One governance card; 3 commits map to Phase 1/2/3 |
| **D8 Type naming** | Name the new type `BarPeriod` (NOT `KlinePeriod`) | Existing `KlinePeriod` in `src/sources/kline_aggregator.rs:14` represents aggregator time-windows (1m/5m/1d); OpenStock `/data/bars` period is a different semantic domain (day/week/month API param). Name collision would compile-fail and confuse consumers |

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

1. **Response schema unchanged**: `/data/bars` returns `{data: [{time, open, high, low, close, volume, amount}, ...]}` for all periods and adjust types.
2. **`Kline` data model unchanged**: existing `Kline { code, date, open, high, low, close, volume, amount, adjust_type }` covers all 3 periods × 3 adjust types.
3. **No new DB writes**: read-only fetch only. ClickHouse / shadow persistence integration is out of scope.
4. **No new parser**: the inline JSON parsing in `fetch_daily_klines` shape (using local `BarsResponse` / `BarRecord` structs) is reused.
5. **Symbol prefix behavior**: `/data/bars` does NOT call `normalize_symbol` — `Kline.code` retains whatever prefix the caller passed. This differs from `/data/fetch` paths (`INDEX_KLINES`) which strip the prefix. Existing behavior preserved.
6. **`f64` → `Decimal` precision loss**: `BarRecord` deserializes OHLCV as `f64`, then `format!("{}", x)` → `Decimal::from_str`. Existing tech debt in `fetch_daily_klines`, preserved unchanged in P0.13a (precision adequate for A-share prices).

## Risks

- **R1 — OpenStock `/data/bars` schema drift**: if the runtime changes the response envelope (e.g. adds a wrapper), the local `BarsResponse` parser breaks. Mitigation: 3 wiremock tests pin the wire shape; Phase 3 live tests will catch drift at integration time.
- **R2 — Adjust type not echoed**: decision D2 means `Kline.adjust_type` is stamped from the request, not the response. If a future OpenStock build starts echoing adjust_type, the stamp remains request-driven (documented invariant; not a bug). Mitigation: in-code comment links decision D2.
- **R3 — Strict period enum blocks legitimate uses**: rejecting `daily`/`weekly`/`monthly` aliases may surprise users who copy-paste from `_eltdx_timeseries.py:_PERIOD_MAP`. Mitigation: error message names the accepted values explicitly.
- **R4 — `f64` precision loss on close prices**: invariant 6 preserves a known tech debt. Acceptable for A-share (≤2 decimal places); revisit if cross-market support is added.
- **R5 — Live smoke unreachable**: `192.168.123.104:8040` may be offline during execution. Mitigation: live tests are `#[ignore]`-gated; quality gates (fmt/clippy/test/openspec validate) still gate the commit.
