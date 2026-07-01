# Design — OpenStock Data Consumption P0.11 (TDX-API Cleanup)

This is a **subtractive** slice (delete + reroute), unlike P0.8-P0.10 which were additive. The design weight is on (a) the consumer reroute map so we delete the right things in the right order, and (b) the ClickHouse/TDengine write path reuse so P0.11a/b do not invent new persistence code.

## D1. Why three sub-slices, not one

The 2026-06-30 handoff assumed openstock would provide P0 capabilities by some future date. The 2026-07-01 status复审 found P0 categories live-verified but P1/P2/P3 categories unverified on the quantix side. Splitting P0.11 into a/b/c reflects that readiness gradient:

- **P0.11a** (`import-klines`) — openstock side ready (INDEX_KLINES + HISTORICAL_KLINES both exist; INDEX_KLINES live-verified 2026-07-01). Can ship immediately.
- **P0.11b** (`import-ticks`) — openstock category TICK_DATA exists in `DATA_CAPABILITY_SCOPE.md` but quantix has never live-fetched it. The 2b.2 unblock gate forces a smoke test; if it fails, P0.11b splits out as a separate OpenSpec change and P0.11c proceeds with `import-ticks` still on tdx-api.
- **P0.11c** (remove `TdxApiClient`) — the actual delete. Blocked on a+b so we never end up with both bridges removed and a write path broken.

A single mega-slice would either (i) block P0.11a on the TICK_DATA smoke (slow) or (ii) risk deleting `TdxApiClient` before `import-ticks` has a working alternative (broken main). The split avoids both failure modes.

## D2. Consumer Map (gitnexus is stale; grep-grounded)

`gitnexus_impact` on `TdxApiClient` returned nominal LOW (0 impacted, "no changed symbols participate in indexed processes"). The index is stale per its own status report. **Grep-grounded reality**:

| File:line | Usage | Action |
|-----------|-------|--------|
| `src/cli/handlers/tdx_api_handler.rs` (entire file, 476 lines) | All 18 CLI subcommand handlers | P0.11c delete |
| `src/cli/handlers/data_handler.rs:348` | `DataSourceKind::TdxApi` health-check branch — `TdxApiClient::from_env()?; client.health().await?` | P0.11c reroute: remove variant, or alias to `OpenStock` and call `OpenStockClient::from_env()` with a `health()` ping (TBD: openstock has no `/health` endpoint today; simplest is remove the variant) |
| `src/tasks/collect_scheduler.rs:83,136` | `tdx_api_fallback: Arc<RwLock<Option<TdxApiClient>>>` field + `set_tdx_api_fallback` method — runtime quote collector fallback | P0.11c Option A: rewire to `OpenStockClient`; requires `fetch_realtime_quotes` wrapper + `parse_realtime_quotes` parser. Option B: delete the fallback (judge by whether `collect_scheduler` is in any active automation today) |
| `src/sources/tdx_api.rs` (entire file, 1309 lines) | The client itself, plus 33 methods | P0.11c delete |
| `src/sources/mod.rs` | `pub mod tdx_api;` re-export | P0.11c delete |

Additional `TdxApiClient` mentions in docs/guides/CHANGELOG are informational only — P0.11c task 3c.17 updates those with deprecation banners.

## D3. Write-path reuse (no new persistence code)

P0.11a/b consume the **existing** write paths; they do not invent new persistence. The handoff explicitly says storage backend selection is internal to quantix-rust (§三.1), so this slice keeps both ClickHouse and TDengine as-is.

### P0.11a ClickHouse path

- Existing write method: `src/db/clickhouse/kline.rs::ClickHouseClient::insert_kline_data_batch_with_source(klines, period, source)`. Main table is `kline_data` (not `klines` — earlier draft of this doc had the wrong name; corrected 2026-07-01 during P0.11a implementation).
- The current tdx-api `import_klines` handler calls this with `source = "THS_QFQ"`. P0.11a openstock branch calls it with `source = "OPENSTOCK"`.
- `Kline` row (`src/data/models.rs:11`) and `IndexKlineRecord` (`src/sources/openstock_index.rs:50`) have matching OHLCV shapes; `parse_index_klines` already produces `Vec<Kline>` from openstock envelopes.
- **Dry-run gate scope decision (2026-07-01)**: P0.11a introduces dry-run only on the **openstock branch** (default = dry-run, `--apply` opts in to write). The tdx-api branch is left untouched (no dry-run) to keep the slice scope tight — it will be deleted entirely in P0.11c anyway.
- **Confirmation gate**: a fresh env var `QUANTIX_OPENSTOCK_KLINE_APPLY=yes` is required when `--apply` is set. We deliberately do NOT reuse `QUANTIX_SHADOW_PERSIST_CONFIRM` from P0.8g-impl — that name is shadow-table-specific and reusing it for the main `kline_data` table would mislead operators. (The spec.md scenario text is updated to match.)
- Existing rollback: shadow `rollback_shadow_batch`. P0.11a does NOT need rollback in the main `kline_data` table — dry-run is the default; explicit `--apply` + `QUANTIX_OPENSTOCK_KLINE_APPLY=yes` is the two-step gate. Operators verify dry-run output first, then apply.

### P0.11b TDengine path

- Existing write: `src/db/tdengine.rs::TdengineClient` (used by current `import-ticks` tdx-api path).
- P0.11b reroutes the source from tdx-api to openstock `TICK_DATA` but does not modify `TdengineClient`. Same dry-run / `--apply` gate pattern.

## D4. Field-shape risk for `HISTORICAL_KLINES` and `TICK_DATA`

`INDEX_KLINES` is live-verified: codes are prefixed (`sh.000001`), numerics are strings, `time` is the date column. `HISTORICAL_KLINES` and `TICK_DATA` are likely different:

- `HISTORICAL_KLINES` may have different field names (`date` vs `time`, `close` vs `close_price`).
- `TICK_DATA` is a fundamentally different shape (per-tick record, not OHLCV bar).

P0.11a/b mirrors the P0.9 pattern: fixture-driven parser tests first, then live wiring. The 2026-07-01联调 discovered via `ALL_STOCKS` that runtime shapes can drift significantly from `DATA_CAPABILITY_SCOPE.md` (the `tradeStatus` vs `listing_date` discussion). P0.11b task 2b.2 explicitly smoke-tests before committing.

If a shape mismatch surfaces, fix it in the parser (mirrors how P0.9/P0.10 `IndexKlineRecord` accepts `Option<serde_json::Value>` for numerics and `parse_decimal`/`parse_volume` handle string vs number). **Do not** change OpenStockClient wrapper signatures for shape drift — that belongs in the parser layer.

### D4.1 TICK_DATA live-verified shape (2026-07-01 smoke)

Smoke command (task 2b.1, executed 2026-07-01):

```
curl -X POST http://192.168.123.104:8040/data/fetch \
  -H "X-API-Key: ..." -H "Content-Type: application/json" \
  -d '{"data_category":"TICK_DATA","params":{"symbol":"600000","date":"20260630"}}'
```

**Critical**: parameter is `symbol`, NOT `code`. Sending `code` returns HTTP 422 `"symbol is required for TICK_DATA"`. The fetch_index_klines / fetch_historical_klines wrappers use `code`; the new fetch_tick_data wrapper MUST use `symbol` to match the eltdx adapter contract.

Response envelope (HTTP 200, 456 KB, 1800 ticks):
- `data` is NOT a flat array. It is `[{meta: {...}, ticks: [...]}]` — array of 1 envelope-record, each containing a meta object and a ticks array.
- `meta` fields: `symbol` (prefixed, e.g. `"sh600000"`), `start`, `count`, `returned_count`, `trading_date`, `price_base`, `has_more`, `requested_date`.
- `ticks[]` fields per entry: `index`, `absolute_index`, `time` (`"HH:MM"`), `time_minutes`, `trade_datetime` (ISO `YYYY-MM-DDTHH:MM:SS`), `price` (float), `price_milli` (int), `volume` (int), `amount` (float), `order_count`, `status` (0/1), `side` (`"buy"`/`"sell"`), `price_delta_raw`, `price_acc_raw`.
- Other envelope fields: `source: "eltdx"`, `data_category: "TICK_DATA"`, `gateway`, `endpoint_name`, `route_decision_id`, `request_id`, `exchange_time`, `received_at`, `staleness_ms`, `cache_state`, `circuit_state`, `quality_flags`, `latency_ms`.

Parser design implications for `src/sources/openstock_ticks.rs`:
- Record type is **not** reusable from IndexKlineRecord. New file with:
  - `TickEnvelopeRecord { meta: TickMeta, ticks: Vec<TickEntry> }`
  - `TickMeta { symbol, trading_date, returned_count, price_base, has_more, ... }`
  - `TickEntry { trade_datetime, price, volume, amount, side, order_count, status, ... }`
- `parse_tick_data(envelope) -> Result<Vec<TickEntry>>` — flattens the single envelope-record's ticks. Returns the meta separately or via a `(meta, ticks)` tuple if downstream needs `trading_date` / `price_base`.
- Quantix `Tick` (`src/data/models.rs:33`) fields: `code, timestamp, price, volume, amount, direction`. Mapping:
  - `code` ← strip prefix from `meta.symbol` (e.g. "sh600000" → "600000")
  - `timestamp` ← parse `trade_datetime` (ISO, second precision)
  - `price` ← `Decimal::try_from(price)` (float → Decimal; or use `price_milli`/1000 for exact)
  - `volume` ← i64 from `volume`
  - `amount` ← `Decimal::try_from(amount)`
  - `direction` ← `match side { "buy" => Buy, "sell" => Sell, _ => Neutral }`
- Status field (0/1) is dropped — semantics unknown, not in the `Tick` model. Document this in parser comment.
- TDengine write path: existing `src/db/tdengine.rs` client unchanged per design D3.

## D5. Naming: `--source` flag vs new top-level subcommand

Two options for surfacing the data-source switch:

- **Option N1 (chosen)**: `import-klines --source <openstock|tdx-api>` flag on the existing `tdx-api` subcommand path. Pro: zero CLI surface change; users who already run `quantix data tdx-api import-klines` see only the new default. Con: the command name still says `tdx-api`, which is misleading once openstock is the default.
- **Option N2 (rejected)**: New top-level `quantix data openstock import-klines`. Pro: clean name. Con: duplicates the surface; forces user retraining; transition window creates two commands doing the same thing.

N1 wins on transition friction. The misleading name is temporary — P0.11c deletes the `tdx-api` parent entirely and migrates `import-klines` / `import-ticks` up one level to `quantix data import-klines` (no parent) or `quantix data openstock import-klines`. **Decision for P0.11c task 3c.9**: when removing the `TdxApi` parent, decide whether to promote `ImportKlines` / `ImportTicks` to top-level `DataCommands` variants or relocate under `OpenStockCommands`. Default: promote to top-level, since by P0.11c openstock is the only source and the indirection is meaningless.

## D6. Governance debt — not carried forward

P0.10's design.md Risks table acknowledged that P0.8i / P0.9.yaml governance cards were never created. P0.11 does **not** retroactively fix that debt (per user direction in P0.10). P0.11.yaml is created for this slice; past debts remain.

## D7. FUNCTION_TREE.md update choreography

FUNCTION_TREE.md is the project's status registry. P0.11c task 3c.16 touches 5 lines:

- **L95** (`quantix data` row): drop "tdx-api 子命令已实现 18 个子命令" claim.
- **L212** (tree node): remove `tdx-api REST 数据源 (tdx_api)`.
- **L658** (`tdx-api fallback`): remove the "交易日历备选数据源" line.
- **L781** (subcommand list): drop `tdx-api` line.
- **L1126** (`tdx-api bridge` sources row): mark deprecated/removed.

All updates happen in P0.11c, not earlier — sub-slices a/b do not change FUNCTION_TREE because the tdx-api surface still exists during their execution.

## Risks Summary

| ID | Risk | Mitigation |
|----|------|-----------|
| R1 | `HISTORICAL_KLINES` shape differs from `INDEX_KLINES` | Fixture-driven parser tests before live wiring; parser absorbs shape drift via `Option<Value>` + `parse_decimal` pattern |
| R2 | `TICK_DATA` not live-functional in openstock | Explicit 2b.2 unblock gate; if fails, P0.11b splits out, P0.11c proceeds with `import-ticks` still on tdx-api |
| R3 | Hidden `TdxApiClient` consumer not caught by grep | 3c.12 grep audit + 3c.13 build clean + `gitnexus detect_changes` post-check |
| R4 | `collect_scheduler` fallback rewire breaks an unrelated execution flow | Only scheduler edit is at 3c.5; full scheduler test suite runs before merge |
| R5 | openstock `REALTIME_QUOTES` shape unknown when P0.11c Option A starts | Same as R1: fixture-driven parser tests first |
| R6 | Removing `tdx-api` Docker service breaks a running `quantix` instance still pointing at it | 3c.15 leaves `docker-compose.yml` `tdx-api` block commented, not deleted; operators can re-enable during transition |
