# Design — OpenStock Data Consumption P0.10 (Live HTTP Wiring)

## Context

P0.9 left two deliberate gaps that P0.10 closes:

1. **No live HTTP wiring** — `OpenStockClient` was a skeleton with three async wrappers but no CLI subcommand invoked them.
2. **Two real defects in the skeleton** — discovered during P0.10 exploration:
   - **G1**: `fetch` consumed the response body without checking HTTP status. On a non-2xx response whose body wasn't valid JSON, both success and error envelope parses failed and the actual upstream error was masked behind a generic "cannot parse" message.
   - **G2**: `OPENSTOCK_BASE_URL` was not wired as an env fallback. Only `OPENSTOCK_API_KEY` had a fallback; base_url required an explicit `OpenStockClientConfig`, which doesn't match the BridgeSettings convention.

The CLI is already async (`src/main.rs:12` `#[tokio::main]`, dispatcher in `app_shell.rs` already `.await`s async handlers like `persist_openstock_live` at L330). So adding 3 new `async fn fetch_*` handlers and 3 dispatcher arms is purely additive — no runtime refactor.

The three convenience wrappers (`fetch_stock_codes`/`fetch_trade_dates`/`fetch_index_klines`) are already implemented in `openstock_client.rs:128/135/144` from P0.9, returning `OpenStockResponse<XxxRecord>`. Handlers print summaries from `resp.records` (raw record types) — sufficient for live smoke output.

## Decisions

### D1. Fix G1 by branching on `status.is_success()` before consuming body

**Why:** The original P0.9 logic was "try success envelope, on parse failure try error envelope." This loses information when the body isn't JSON at all (e.g. HTML 502 from a proxy between client and OpenStock). Worse, on a real OpenStock error envelope returned with a non-2xx status, the success envelope parse *also* would have failed (`data` field absent), so the original fallback happened to work — but only by accident, and only when the body was valid JSON.

**Alternative considered:** Trust OpenStock to always return JSON (success or error envelope). Rejected: we cannot trust intermediate proxies or future OpenStock versions to preserve this invariant, and the cost of one status check is negligible.

**Decision:** Capture `let status = resp.status();` before `let raw = resp.text().await?;`. If `!status.is_success()`, attempt `OpenStockErrorEnvelope` parse; on success use `to_summary()`, on parse failure surface `HTTP {status} | body: {first 200 chars}` so the operator sees the actual upstream response.

### D2. Fix G2 by mirroring BridgeSettings pattern: env fallback for `base_url`

**Why:** `BridgeRuntimeSettings` (`src/core/runtime/settings.rs:55`) reads `BRIDGE_BASE_URL` / `BRIDGE_API_KEY` from env via `core/runtime/init.rs`. The P0.9 `OpenStockClient::new` only fell back for the API key — the base_url required an explicit `OpenStockClientConfig` struct, which doesn't match the rest of the codebase and forces callers to construct config boilerplate.

**Decision:** In `new()`, after the existing `OPENSTOCK_API_KEY` fallback for `api_key`, add a parallel `OPENSTOCK_BASE_URL` fallback for `base_url`. Add `pub fn from_env() -> Result<Self>` calling `Self::new(OpenStockClientConfig::default())` for the common case where both come from env.

**Alternative considered:** Refactor `CliRuntime` to include `OpenStockSettings`. Rejected for this slice — out of scope, deferred to Non-Goals; env-only is sufficient.

### D3. Three new `Fetch*` subcommands instead of unifying under one `FetchLive`

**Why:** Mirrors the granularity of the P0.9 `Validate*` family (3 subcommands). Each category has different parameters (codes has none, calendar takes `--year`, index takes `--symbol` + optional date range), so a single unified subcommand would need a `--kind` discriminator plus a variadic arg shape — strictly worse ergonomically.

**Decision:** `FetchCodes` (no args), `FetchCalendar { year }`, `FetchIndex { symbol, start?, end? }`. Each handler prints a uniform summary block (records count, first/last sample, source, `artifact_hash`, `latency_ms`).

**Alternative considered:** Single `FetchLive { category, params_json }`. Rejected: poor CLI ergonomics; loses `--year`/`--symbol` validation at the clap layer.

### D4. ALL_STOCKS / WORKDAYS live fetchers explicitly deferred

**Why:** P0.9 shipped parsers for all 5 P0 categories but convenience fetchers only for 3 (`fetch_stock_codes`/`fetch_trade_dates`/`fetch_index_klines`). Wiring live fetchers for ALL_STOCKS and WORKDAYS would require 2 new client wrappers + 2 new CLI subcommands + 2 new live tests — pure linear scale-up with no new design content.

**Decision:** Cut from this slice. The parse path exists; the live path can be added in a follow-up (P0.10a or similar) without any architectural change. Documented in Non-Goals.

### D5. P0.9 governance debt acknowledged, not retroactively fixed

**Why:** P0.9's tasks.md Section 0 mandated creating a `P0.8i` governance node (and the corresponding card), but neither was actually created before commit `2571003` landed. P0.10 *could* retroactively create P0.9.yaml + close out the missing P0.8i, but per user direction this slice takes the lighter path.

**Decision:** P0.10 creates only `P0.10.yaml` (this slice's card). The P0.9 debt is recorded in this design.md's Risks table (R-DEBT-1) so a future cleanup slice can pick it up.

## Non-Goals

(See `proposal.md` Non-Goals — same list.)

## Risks

| ID | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R-NET-1 | Live OpenStock runtime unreachable from CI / dev box (network, auth, key rotation) | Medium | Low (live tests are `#[ignore]`-gated, never run in CI) | Manual smoke documented in tasks 9.10/9.11 |
| R-PARSE-1 | Live `STOCK_CODES`/`TRADE_DATES`/`INDEX_KLINES` payloads diverge from fixtures (provider schema drift since fixtures captured) | Low | Medium (handler prints raw record types, so drift surfaces as `None` fields rather than crash) | The G1 status check + raw body snippet in error path makes drift easy to diagnose; the underlying parsers already validate invariants (empty records, high<low, mixed codes) |
| R-DEBT-1 | P0.9 governance debt (P0.8i / P0.9.yaml never created) — future governance audits may flag this | Medium | Low (governance is documentation-only; no runtime impact) | Recorded here for a future cleanup slice; P0.10 card is properly scoped |
| R-API-1 | `OpenStockResponse::latency_ms` is `Option<u64>` — some providers may omit it, callers must handle None | Low | Low (handlers print "(not reported)" when None) | Documented in struct doc-comment |

## Migration Path

- **P0.10 (this slice)**: live HTTP for 3 P0 read-only categories; G1/G2 fixes; env-driven config.
- **P0.10a or P0.11 (follow-up)**: ALL_STOCKS + WORKDAYS live fetchers; CliRuntime integration; retry/circuit-breaker if real-world reliability demands it; cleanup of P0.9 governance debt.
- **P0.12+ (later)**: ClickHouse write path for live-fetched data (successor to `PersistLive` for the new categories); quality_flags / route_decision_id surfacing in CLI output.
