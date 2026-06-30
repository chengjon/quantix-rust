# OpenStock Data Consumption P0.9 — Design

## Context

OpenStock runtime exposes a uniform `POST /data/fetch` endpoint with `data_category` routing — confirmed by two authoritative docs (`docs/CONNECTION_GUIDE.md`, `docs/DATA_CAPABILITY_SCOPE.md`). The 5 P0 categories are live and healthy:

| Category | Use case | Provider |
|---|---|---|
| `STOCK_CODES` / `ALL_STOCKS` | 全市场代码列表 | eltdx |
| `TRADE_DATES` / `WORKDAYS` | 交易日历 | eltdx |
| `INDEX_KLINES` | 指数 K 线 | baostock (slower, ~5s p50) |

P0.8 covered daily klines only. This slice adds the next three P0 capabilities plus a generic client skeleton — all fixture-driven, no live network in CI.

## Decisions

### D1. Why option (b) for `OpenStockClient` (struct + reqwest now)

- **Alternative considered**: option (a) — define a `trait OpenStockBackend` with a `Mock` impl now and a `Reqwest` impl in a later slice.
- **Rejected because**: deferring reqwest impl means deferring all deserialization, which is the actual contract surface. When live wiring lands, we'd be forced into a second rewrite pass to learn whether the envelope shape parses at all.
- **Chosen**: option (b) — implement `OpenStockClient` as a struct backed by reqwest today. Tests stay fixture-only by exercising the shared deserialization paths (`OpenStockResponse::from_envelope`, parser fns) directly. No live HTTP in tests.

### D2. Why `OpenStockEnvelope<T>` vs `OpenStockResponse<T>` split (issue 4 resolution)

The two types serve different roles in the parse pipeline:

- `OpenStockEnvelope<T>` is a **raw serde target** — 1:1 with the JSON body. Every metadata field is `Option`/`#[serde(default)]` because OpenStock providers may omit any of them. Crate-internal use only.
- `OpenStockResponse<T>` is the **public post-parse view** — flattened to `(records, source, artifact_hash, received_at)`. The `artifact_hash` is computed client-side from the raw body via `openstock_shadow::artifact_hash` (per `CONNECTION_GUIDE.md §migration`: "OpenStock does not push artifact_hash; the consumer computes it").

Parsers (`parse_stock_codes` etc.) consume `OpenStockEnvelope<T>` and return `Vec<T>` — they do NOT depend on `OpenStockResponse<T>` or `OpenStockClient`. This keeps fixture-only tests HTTP-free.

### D3. Why re-export `artifact_hash` instead of forking it (issue 1 resolution)

`openstock_shadow::artifact_hash` at `src/sources/openstock_shadow.rs:27` is the canonical SHA-256 of raw payload bytes — already used by the daily-kline shadow persistence write path. Forking it into `openstock_envelope.rs` would:

- Violate the additive-only rule (no editing `openstock_shadow.rs`).
- Risk hash drift if one copy ever changes.

Instead, `openstock_envelope.rs` adds `pub use crate::sources::openstock_shadow::artifact_hash;` — in-scope re-export, single source of truth. `OpenStockResponse::from_envelope` calls it directly. `mod.rs` re-exports it as `openstock_artifact_hash` (disambiguated name) so external callers don't conflict with the canonical path.

### D4. Why flat `tests/` layout (issue 2 resolution)

Existing test convention is flat: `tests/account_cli_validation_test.rs`, `tests/bridge_client_test.rs`, `tests/cli_integration.rs`, `tests/factor_test.rs`, etc. There is no `tests/sources/` today. Introducing one for this slice would deviate without precedent. New test files are flat:

- `tests/openstock_codes.rs`
- `tests/openstock_calendar.rs`
- `tests/openstock_index.rs`
- `tests/openstock_client.rs`

Fixtures stay at `tests/fixtures/openstock/*.json` — matches existing `tests/fixtures/` location.

### D5. Why fixture/parser parallel paths (issue 3 resolution)

The existing `parse_daily_kline_json` (`openstock.rs:63-97`) is the legacy local-fixture parser used by `quantix data openstock validate-fixture`. The existing `validate_live_shadow_payload` (`openstock.rs:349-448`) is the live-envelope path for daily klines. Both stay untouched.

The new envelope parsers (`parse_stock_codes`, `parse_trade_dates`, `parse_workdays`, `parse_index_klines`) are the **only** parse path for the 5 P0 categories — they consume `OpenStockEnvelope<T>` directly. There is no legacy fixture parser for these categories to displace.

This is parallel, not superseding: daily kline keeps its fixture parser for legacy reasons; the 4 new categories use the envelope parser exclusively. Verified contract: `/data/fetch` always returns `data` as a JSON array for the 5 in-scope categories (per `CONNECTION_GUIDE.md` envelope spec).

### D6. Why per-category CLI subcommands

The existing `OpenStockCommands` enum uses per-concern variants (`ValidateFixture`, `ValidateLive`, `PersistLive`, `ShadowRollback`, `ShadowVerify`). Adding 3 more (`ValidateCodes`, `ValidateCalendar`, `ValidateIndex`) is consistent with that granularity. A single `Validate { --kind: ... }` discriminator would diverge from the established pattern.

(Issue 5 in plan review flagged this as design-polish; we keep the per-category pattern for consistency. If a future slice consolidates CLI surface, it can do so uniformly across all OpenStock commands.)

### D7. Why `pub(crate)` visibility widen is safe (additive)

`normalize_symbol` (`openstock.rs:517`) and `parse_live_time` (`openstock.rs:531`) are currently private. Widening to `pub(crate)` makes them callable from `openstock_index.rs` (new sibling module in the same crate). This is purely additive: no signature change, no behavior change, no external API change (still not `pub`). The alternative — duplicating the helpers — would risk drift on the symbol-prefix and date-parsing contract.

## Non-Goals

- No live HTTP calls in CI or default CLI behavior — fixtures only.
- No ClickHouse or shadow persistence writes — covered by P0.8g-impl, not extended here.
- No replacement of `parse_daily_kline_json` (legacy fixture parser stays).
- No trait introduction — `OpenStockClient` is a struct.
- No refactor of `openstock_shadow.rs` — only re-exports its `artifact_hash`.

## Risks

| Risk | Mitigation |
|---|---|
| OpenStock envelope shape diverges from docs | Verified against `CONNECTION_GUIDE.md` and `DATA_CAPABILITY_SCOPE.md` (authoritative). Fixtures encode the documented shape. |
| `INDEX_KLINES` from baostock has higher latency (~5s p50) | This slice is offline-only; latency doesn't affect fixture validation. Live calls are deferred. |
| `pub(crate)` widen leaks internal helpers | Acceptable: same crate, signature unchanged, no external API impact. |
| Future live-wiring slice needs more metadata than `OpenStockResponse<T>` carries | `OpenStockEnvelope<T>` remains accessible as the raw serde target; `OpenStockResponse::from_envelope` is a view, not a replacement. |

## Migration Path

This slice establishes the type scaffolding. The next slice (P0.10) is expected to wire `OpenStockClient` to live HTTP for the 5 P0 categories — purely additive: no parser rewrite, no envelope shape change, only the network calls go live behind existing fixture-tested methods.
