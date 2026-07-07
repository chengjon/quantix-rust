# OpenStock Live-Instance Verification Handoff

**Date:** 2026-07-07
**Author:** quantix-rust agent (Claude)
**Audience:** OpenStock runtime maintainer
**Status:** Empirical findings + open questions needing OpenStock-side decisions

---

## TL;DR

While validating P0.15a (`import-minute-klines` / `import-minute-share` CLI) against the live OpenStock instance at `http://192.168.123.104:8040`, we discovered:

1. ✅ **`import-minute-klines` works end-to-end against live** (real `sh600000` 5m bars pulled, 92ms latency)
2. ❌ **`import-minute-share` is broken against live** — `Error: Other("openstock error [unknown] ")`
3. The breakage is a parameter-name mismatch that has been latent across 4 quantix slices (P0.13b-2 / P0.13c / P0.13d / P0.14 / P0.15a) because unit tests only ever talk to a mock HTTP that shares the same wrong assumption.

**The user (chengjon) asked us to validate against the running service rather than trust documentation.** That validation surfaced the divergence. This doc lists everything we need the OpenStock maintainer to confirm or fix on the server / contract side.

---

## 1. Environment Used for Validation

| Component | Value |
|---|---|
| OpenStock URL | `http://192.168.123.104:8040` |
| Container | `openstock` (`openstock:nas`, port 8040 → 8000) |
| Auth | `X-API-Key: sk-Z8h5YC-...` (works; 401 without it, confirmed) |
| Client | `target/release/quantix` (cargo release build 2026-07-07) |
| Symbol used | `sh600000` (浦发银行, A-share, high liquidity) |
| Test dates | `2026-07-03` (single trading day), `2026-06-29..2026-07-03` (multi-day) |

---

## 2. What Works (Verified Live)

### 2.1 `/data/fetch` — `STOCK_CODES`

```bash
curl -H "X-API-Key: $KEY" -H "Content-Type: application/json" \
  -X POST http://192.168.123.104:8040/data/fetch \
  -d '{"data_category":"STOCK_CODES","params":{}}'
```
**Result:** HTTP 200, valid A-share code list (sh688xxx series etc.). ✅

### 2.2 `/data/fetch` — `MINUTE_DATA` (with `symbol` param)

```bash
curl ... -d '{"data_category":"MINUTE_DATA","params":{"symbol":"sh600000","date":"2026-07-03"}}'
```
**Result:** HTTP 200, 240 minute-share points for the trading day, real OHLCV-style data (price/volume/avg_price). ✅

### 2.3 `/data/bars` — minute klines (different endpoint!)

```bash
curl ... -X POST http://192.168.123.104:8040/data/bars \
  -d '{"symbol":"sh600000","period":"5m","adjust":"qfq","date":"2026-07-03"}'
```
**Result:** HTTP 200, real OHLCV 5m candles (`{symbol, time, open, high, low, close, volume, amount, period, adjust, adjust_mode}`). ✅

### 2.4 `quantix data openstock import-minute-klines` (full CLI)

```bash
OPENSTOCK_BASE_URL=http://192.168.123.104:8040 OPENSTOCK_API_KEY=$KEY \
  ./target/release/quantix data openstock import-minute-klines \
  --code sh600000 --period 5m --adjust qfq \
  --start 2026-07-03 --end 2026-07-03
```
**Output:**
```
OpenStock import-minute-klines (dry-run)
  code: sh600000, period: 5m, adjust: qfq
  range: 2026-07-03 .. 2026-07-03
  Streaming weekly chunks (counting only, no ClickHouse writes):
  [batch 1] would_insert: +100 (cumulative: 100)
  dry_run: true, applied: false
  would_insert_total: 100
  batches: 1, elapsed: 91.929156ms
```
✅ End-to-end works through the P0.15a handler → P0.13d stream → P0.13b range parser → real wire.

---

## 3. What's Broken (Verified Live)

### 3.1 Symptom

```bash
OPENSTOCK_BASE_URL=... OPENSTOCK_API_KEY=... \
  ./target/release/quantix data openstock import-minute-share \
  --code sh600000 --start 2026-07-03 --end 2026-07-03
# Output:
#   OpenStock import-minute-share (dry-run)
#   ...
#   Streaming weekly chunks (counting only, no ClickHouse writes):
#   Error: Other("openstock error [unknown] ")
```

### 3.2 Root Cause (client side)

`src/sources/openstock_client.rs::fetch_minute_share_single` (around L931) currently builds its body as:

```json
{"data_category":"MINUTE_DATA","params":{"code":"sh600000","date":"2026-07-03"}}
```

But the live server **rejects `code` and requires `symbol`** for `MINUTE_DATA`:

```bash
curl ... -d '{"data_category":"MINUTE_DATA","params":{"code":"sh600000","date":"2026-07-03"}}'
# HTTP 422
# {"detail":{"code":"invalid_request","message":"symbol is required for MINUTE_DATA",
#   "request_id":"fetch-request","details":{"category":"MINUTE_DATA","provider":"eltdx"}}}
```

Switching `code` → `symbol` returns 200 with full data. **The parameter-name contract is `symbol`, not `code`.** This affects every consumer of `MINUTE_DATA` in quantix (P0.13b-2 onward).

### 3.3 Why unit tests didn't catch it

The unit test `fetch_minute_share_sends_minute_data_category_and_date` (`openstock_client.rs` ~L1785) asserts:

```rust
.and(body_partial_json(json!({
    "data_category": "MINUTE_DATA",
    "params": { "code": "sh600000", "date": "2026-07-01" }
})))
```

The mock is set up to **accept whatever the code produces**, so the test passes regardless of whether the wire contract matches the live server. The test was echoing the same wrong assumption that the implementation had. Mock-server tests cannot validate external contract conformance — only live calls can.

### 3.4 Secondary symptom: opaque error

The CLI surfaced the 422 as `Error: Other("openstock error [unknown] ")`. The `[unknown]` is our `code` slot coming up empty because our error-envelope parser (`OpenStockErrorEnvelope`-style) doesn't read into `detail.code` — the live error shape is:

```json
{"detail":{"code":"invalid_request","message":"...","request_id":"...","details":{...}}}
```

Our code currently expects something flatter, so when a real 422 arrives, we lose both the `code` and the actionable `message`. **Even after fixing the param name, please also fix the error parser so future divergences are visible.**

---

## 4. Open Questions for the OpenStock Maintainer

These are decisions / confirmations only the OpenStock-side owner can make. Numbered for easy reply.

### Q1. Parameter name for `MINUTE_DATA` — is `symbol` the canonical name?

We will change our client to send `symbol` instead of `code` for `MINUTE_DATA`. Please confirm:

- (a) `symbol` is the long-term canonical param name (not about to be reverted to `code`)
- (b) The same name (`symbol`) is also expected for other `data_category` values that take a per-symbol lookup (e.g. `KLINES`, `INDEX_KLINES`, etc.) — or whether each category has its own convention.

The runtime currently signals inconsistency: e.g. our `INDEX_KLINES` requests already use `code` (and that's documented as "verified 2026-07-01 against live"), but `MINUTE_DATA` rejects `code`. Is that intentional per-category asymmetry?

### Q2. Why is minute kline served from `/data/bars`, not `/data/fetch`?

`MINUTE_KLINES` is **not a valid `data_category`** under `/data/fetch`:

```bash
curl ... -d '{"data_category":"MINUTE_KLINES","params":{...}}'
# HTTP 422
# {"detail":{"code":"internal_error","message":"Unsupported data_category: MINUTE_KLINES", ...}}
```

Minute klines come from a completely different endpoint, `/data/bars`, with a different body shape (no `data_category`/`params` envelope — flat `{symbol, period, adjust, date | start_date+end_date}`) and a different response shape (`{data: [...]}` array of bar records directly, no `meta` block per record).

Please confirm:

- (a) This split is intentional: minute-share = `/data/fetch` + `MINUTE_DATA`; minute-kline = `/data/bars`
- (b) `/data/bars` is stable (we built P0.13b/P0.13c/P0.13d against it)
- (c) Whether `/data/bars` is planned to be unified under `/data/fetch` as a `MINUTE_KLINES` category at some point (so we know whether to plan a migration)

### Q3. `date` vs `start_date`/`end_date` semantics on `/data/bars`

We observed that calling `/data/bars` with `{"symbol":"sh600000","period":"5m","adjust":"qfq","date":"2026-07-03"}` returns bars **spanning 2026-07-02 14:45 .. 2026-07-03 15:00** — i.e. ~1 day's worth of trailing data through the requested `date`, not just bars *on* 2026-07-03.

Is `date` interpreted as "through end of this date" (trailing) by the runtime? Our quantix `DateOrRange::Date(d)` handler treats it as "single day = `(d, d)` range" and expects `date` to mean "only that day". If the runtime's semantics differ, please document:

- What `date` means on `/data/bars` (single day vs cutoff vs N-day lookback)
- Whether single-day requests should use `start_date == end_date == date` to be unambiguous

### Q4. `TRADE_DATES` returns `is_trading_day` as string `"0"`/`"1"`, not boolean

```bash
curl ... -d '{"data_category":"TRADE_DATES","params":{"year":2026}}'
# {"data":[{"calendar_date":"2015-01-05","is_trading_day":"1"}, ...]}
```

We currently handle this in our parser (P0.10), but it's an unusual shape. Is this intentional? If you ever normalize to actual booleans, please flag — we'll add a compatibility shim.

### Q5. Error envelope shape

The live 4xx error shape is:

```json
{"detail":{"code":"invalid_request","message":"...","request_id":"...","details":{...}}}
```

The `code` values we've seen include `invalid_request`, `internal_error`. What is the **canonical set of error codes** we should map to user-facing behavior? Right now we treat all of them as a generic `Other`, which means a misconfigured request looks identical to a server bug. A short enumeration (e.g. `invalid_request | unauthorized | not_found | rate_limited | internal_error | provider_unavailable`) would let us do proper retry / fail-fast branching on the client.

### Q6. Auth header naming and rotation

`X-API-Key: sk-Z8h5YC-HyMdjb9qtFWqYU8WIYziKf7n8` works. Two asks:

- (a) Confirm `X-API-Key` is the canonical header (not `Authorization: Bearer ...`) so we don't need to support both
- (b) If this key is a long-lived shared key, please set up a rotation policy reminder. If it's instance-scoped, please tell us how to discover it dynamically (so we don't bake it into env files)

### Q7. Latency / batch size expectations

For `MINUTE_DATA`, the live response for one trading day is ~30 KB (240 points). For `import-minute-share` over a year (≈250 trading days), that's ~7.5 MB across 250 stream batches. Our weekly chunking (P0.13d) was sized assuming this. Are there server-side rate-limits, max-response-size limits, or recommended chunk sizes we should respect? Right now we issue one request per calendar day for share, and one request per ≤7-day range for klines.

---

## 5. What's Not Yet Validated (Need OpenStock + ClickHouse)

We did NOT validate the **apply path** (`--apply` + `QUANTIX_OPENSTOCK_MINUTE_APPLY=yes` → ClickHouse write) because:

1. The minute-share dry-run broke (Q1 above) — apply can't work if dry-run can't even count.
2. We don't have a confirmed ClickHouse instance wired into the same machine that runs the CLI.

Once Q1 is fixed and you give us the ClickHouse connection (or confirm we should reuse the openstock-side ClickHouse if any), we will:

- Run `import-minute-klines --apply` for a small known range
- `SELECT count(), min(time), max(time) FROM minute_klines WHERE code='sh600000' AND ...`
- Run `import-minute-share --apply` for the same range
- Verify against `minute_shares`
- Spot-check 5 rows for byte-correct field mapping (price/volume/adjust/period)

---

## 6. Suggested OpenStock-Side Actions (in priority order)

1. **Reply to Q1** (confirm `symbol` is canonical for `MINUTE_DATA` and ideally for all per-symbol categories) — this unblocks our one-line client fix and the broader rename decision.
2. **Reply to Q2/Q3** (confirm endpoint split + `date` semantics) — this affects whether our range parser is correct.
3. **Document or fix the error envelope** (Q5) — even just a stable enumeration of `code` values would be enough.
4. Optional: clarify `is_trading_day` type (Q4), header canonicality (Q6), rate limits (Q7).

---

## 7. Suggested Quantix-Side Actions (already planned, listed for visibility)

These are ours to execute, no OpenStock input needed — but listed so you can see the full picture:

1. Fix `fetch_minute_share_single` to send `symbol` (1 line).
2. Fix the locked-in unit test to assert `symbol` (the mock should require the real wire shape, not just echo whatever the implementation does).
3. Improve error-envelope parser so live 4xx surfaces a meaningful `code` instead of `[unknown]`.
4. Re-run live dry-run for `import-minute-share` and confirm 240 points / day.
5. Live-validate the apply path against ClickHouse once ClickHouse endpoint is settled.
6. Add an `#[ignore]` live-integration test in `tests/openstock_live_import_minute.rs` that actually hits `192.168.123.104:8040` and asserts a known symbol returns ≥1 row — guarded by an env var so CI doesn't run it.

---

## 8. Reproduction Recipe (for OpenStock-side)

If you want to reproduce our findings on your end:

```bash
KEY="sk-Z8h5YC-HyMdjb9qtFWqYU8WIYziKf7n8"
URL="http://192.168.123.104:8040"

# 1. Confirm auth works
curl -sS -o /dev/null -w "%{http_code}\n" "$URL/health"
# → 401 (expected)

# 2. Confirm STOCK_CODES works
curl -sS -o /tmp/a.json -w "%{http_code}\n" -H "X-API-Key: $KEY" \
  -H "Content-Type: application/json" -X POST "$URL/data/fetch" \
  -d '{"data_category":"STOCK_CODES","params":{}}'
# → 200

# 3. Reproduce the breakage (current quantix behavior)
curl -sS -o /tmp/b.json -w "%{http_code}\n" -H "X-API-Key: $KEY" \
  -H "Content-Type: application/json" -X POST "$URL/data/fetch" \
  -d '{"data_category":"MINUTE_DATA","params":{"code":"sh600000","date":"2026-07-03"}}'
# → 422, body: {"detail":{"code":"invalid_request","message":"symbol is required for MINUTE_DATA",...}}

# 4. Confirm the fix shape
curl -sS -o /tmp/c.json -w "%{http_code}\n" -H "X-API-Key: $KEY" \
  -H "Content-Type: application/json" -X POST "$URL/data/fetch" \
  -d '{"data_category":"MINUTE_DATA","params":{"symbol":"sh600000","date":"2026-07-03"}}'
# → 200, body has 240 points

# 5. Confirm /data/bars works for klines
curl -sS -o /tmp/d.json -w "%{http_code}\n" -H "X-API-Key: $KEY" \
  -H "Content-Type: application/json" -X POST "$URL/data/bars" \
  -d '{"symbol":"sh600000","period":"5m","adjust":"qfq","date":"2026-07-03"}'
# → 200
```

---

## 9. Contact / Next Steps

This document will be committed to `quantix-rust` at `docs/reports/OPENSTOCK_HANDOFF_2026-07-07.md`. Replies can be:

- Inline comments on the file (chengjon/quantix-rust)
- A reply handoff doc in OpenStock's own docs area
- Direct fix-on-server + "this is now resolved, please retest" message

Once Q1–Q3 are answered we can finish P0.15a (apply-path validation + Task 6 live tests) within a single session.
