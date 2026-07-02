# OpenStock P0.13b-2: MinuteShare via `/data/fetch` Envelope Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `fetch_minute_share` client method consuming OpenStock `MINUTE_DATA` category via the existing `/data/fetch` envelope path, with `MinuteShare` model, CLI subcommand, live tests, and OpenSpec change.

**Architecture:** Sibling slice to P0.13b-1 (already merged at `8a1a8f2`). P0.13b-1 consumed minute OHLC candles via `/data/bars` (direct reqwest, no retry). P0.13b-2 consumes intraday time-share ticks via `/data/fetch` envelope (with retry + circuit breaker, same path as `fetch_stock_codes`). Additive only — 6 existing files modified, 7 new files created. No deletions, no signature changes to P0.13b-1 code.

**Tech Stack:** Rust (stable), `reqwest` (existing), `rust_decimal::Decimal` (existing), `serde` + `serde_json` (existing), `wiremock` (existing dev-dep), `tokio` (existing), `chrono` `NaiveDate`/`NaiveDateTime` (existing).

## Global Constraints

[From spec §5 + §6 of `docs/superpowers/specs/2026-07-02-openstock-p0-13b-2-minute-share-design.md` (R1 revision `d25410d`)]

- **INV-1A**: `fetch_minute_share` MUST call `self.fetch::<T>("MINUTE_DATA", params)` — two-arg signature `(category: &str, params: Value)`. NEVER single-JSON-body call (would compile-fail; signature is `fetch<T: DeserializeOwned>(&self, category: &str, params: Value) -> Result<OpenStockResponse<T>>` at `openstock_client.rs:180`).
- **INV-1B**: Request `params` JSON contains ONLY `{code, date}`. NEVER include `period`/`adjust` (MINUTE_DATA doesn't support those dimensions). `date` format is `"YYYY-MM-DD"`.
- **INV-2C**: Single record with missing required field → `tracing::warn!` + skip the record (do NOT fail the whole batch). Implemented via: (a) `RawMinuteRecord` business fields are `Option<...>` (serde returns None when missing, not error); (b) `parse_minute_share` returns `Option<MinuteShare>` (None when any required field is None OR timestamp parsing fails).
- **INV-3**: `price`/`amount`/`avg_price` MUST be `Decimal` (not `f64`). `volume` MUST be `i64` (integer). `RawMinuteRecord` uses `Option<Decimal>` directly (no `from_f64_retain` hop).
- **INV-4**: NEVER bypass envelope retry/circuit breaker. Do NOT add `.expect(1)` or custom reqwest call — `self.fetch::<T>()` provides retry/circuit breaker automatically.
- **Wire format**: Wiremock test mock server MUST assert request body is `{"data_category": "MINUTE_DATA", "params": {"code": "...", "date": "..."}}` — `fetch<T>()` constructs this envelope internally (see `openstock_client.rs:205-208`).
- **No P0.13b-1 modification**: This slice MUST NOT modify `MinuteBar` / `MinutePeriod` / `fetch_minute_klines` / `parse_minute_bar` / `MinuteBarRecord`. Additive only.
- **Naming**: Type is `MinuteShare` (NOT `MinuteKline`, NOT `MinuteShareRecord`). Parser is `parse_minute_share`. CLI subcommand is `fetch-minute-share`. OpenSpec change is `openstock-data-consumption-p0-13b-2`.
- **Test pattern**: Wiremock tests use `OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build")` (same helper as P0.13b-1, located at `openstock_client.rs:789`). Live tests use `#[ignore]` + `QUANTIX_OPENSTOCK_LIVE=1` env gate (same pattern as `tests/openstock_live_minute_klines.rs`).
- **File size**: All modified files stay under coding standards limits (`models.rs` < 500 lines, `openstock_client.rs` may grow but each task adds < 200 lines).

---

## File Structure

| File | Operation | Responsibility |
|------|-----------|----------------|
| `src/data/models.rs` | Modify | Add `MinuteShare` struct + 1 unit test |
| `src/sources/openstock_client.rs` | Modify | Add `RawMinuteRecord`, `fetch_minute_share`, `parse_minute_share`, `parse_time_minutes` + 3 wiremock tests + 3 unit tests |
| `src/cli/commands/data.rs` | Modify | Add `FetchMinuteShare` enum variant |
| `src/cli/handlers/openstock_handler.rs` | Modify | Add `fetch_openstock_minute_share` handler |
| `src/cli/handlers/mod.rs` | Modify | Add re-export |
| `src/cli/handlers/app_shell.rs` | Modify | Add dispatcher arm |
| `tests/openstock_live_minute_share.rs` | Create | 3 `#[ignore]` live tests (L1/L2/L3) |
| `openspec/changes/openstock-data-consumption-p0-13b-2/proposal.md` | Create | OpenSpec proposal |
| `openspec/changes/openstock-data-consumption-p0-13b-2/tasks.md` | Create | OpenSpec tasks |
| `openspec/changes/openstock-data-consumption-p0-13b-2/design.md` | Create | OpenSpec design |
| `openspec/changes/openstock-data-consumption-p0-13b-2/specs/openstock-data-consumption/spec.md` | Create | OpenSpec spec deltas |
| `.governance/programs/project-governance/cards/P0.13b-2.yaml` | Create | Governance card |

---

## Task 1: `MinuteShare` Model in `src/data/models.rs`

**Files:**
- Modify: `src/data/models.rs` (insert after `MinuteBar` struct, which ends around L160 — search `pub struct MinuteBar` to locate)
- Test: `src/data/models.rs` `#[cfg(test)] mod tests` (at bottom of file, around L264+)

**Interfaces:**
- Consumes: `chrono::NaiveDateTime`, `rust_decimal::Decimal`, `serde::{Deserialize, Serialize}` (all already imported at top of `models.rs`)
- Produces: `pub struct MinuteShare { code, timestamp, price: Option<Decimal>, volume: Option<i64>, amount: Option<Decimal>, avg_price: Option<Decimal> }` — used by Task 2 (`fetch_minute_share` return type) and Task 3 (handler output formatting)

- [ ] **Step 1: Write the failing test**

Append to `src/data/models.rs` `#[cfg(test)] mod tests` (after existing `MinuteBar`-related tests, search ` MinuteBar` in tests module to find anchor):

```rust
#[test]
fn minute_share_round_trip_serde() {
    use chrono::NaiveDate;
    let share = crate::data::models::MinuteShare {
        code: "sh600000".to_string(),
        timestamp: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap().and_hms_opt(9, 30, 0).unwrap(),
        price: Some(dec!(10.50)),
        volume: Some(123_456),
        amount: Some(dec!(1_296_288.00)),
        avg_price: Some(dec!(10.4975)),
    };
    let json = serde_json::to_string(&share).expect("serialize");
    let back: crate::data::models::MinuteShare = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.code, "sh600000");
    assert_eq!(back.volume, Some(123_456));
    assert_eq!(back.avg_price, Some(dec!(10.4975)));
}

#[test]
fn minute_share_allows_missing_optional_fields() {
    // Missing price/volume/amount/avg_price fields → all None (INV-2C foundation)
    let json = r#"{"code":"sh600000","timestamp":"2026-07-01T09:30:00"}"#;
    let share: crate::data::models::MinuteShare =
        serde_json::from_str(json).expect("deserialize with missing optionals");
    assert_eq!(share.code, "sh600000");
    assert_eq!(share.price, None);
    assert_eq!(share.volume, None);
    assert_eq!(share.amount, None);
    assert_eq!(share.avg_price, None);
}
```

Note: `dec!` macro is already defined at the top of the `tests` module (`macro_rules! dec` — verify by searching `macro_rules! dec` in `models.rs`). If absent, replace `dec!(10.50)` with `Decimal::from_str("10.50").unwrap()` and add `use std::str::FromStr;` at the top of the test function.

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test --lib --package quantix-cli models::tests::minute_share_round_trip_serde
cargo test --lib --package quantix-cli models::tests::minute_share_allows_missing_optional_fields
```

Expected: FAIL with "cannot find type `MinuteShare` in this scope" or similar.

- [ ] **Step 3: Implement `MinuteShare` struct**

Insert into `src/data/models.rs`, immediately after the closing `}` of `MinuteBar` struct (around L160). Search `pub struct MinuteBar` to find the exact location:

```rust
/// 分时点序列（P0.13b-2 新增）。
///
/// 对应 OpenStock `MINUTE_DATA` category。与 `MinuteBar` 区别：
/// - 无 OHLC（仅单一 `price`）
/// - 含 `avg_price`（均价，业务关键字段）
///
/// **Option 字段说明**：业务字段全部用 `Option` 包裹以支持 INV-2C
/// （单条记录字段缺失时 warn + skip，不中断整批）。serde 反序列化
/// 在 Option 字段缺失时返回 None 而非失败；parser 阶段检查关键字段
/// （price/volume/amount/avg_price），任一为 None 则 warn + skip。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteShare {
    pub code: String,
    pub timestamp: NaiveDateTime,
    pub price: Option<Decimal>,
    pub volume: Option<i64>,
    pub amount: Option<Decimal>,
    pub avg_price: Option<Decimal>,
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test --lib --package quantix-cli models::tests::minute_share
```

Expected: PASS (2 tests).

- [ ] **Step 5: Run clippy + fmt**

```bash
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -20
cargo fmt --all
```

Expected: 0 warnings, no fmt diff.

- [ ] **Step 6: Commit**

```bash
git add src/data/models.rs
git commit -m "$(cat <<'EOF'
feat(data): add MinuteShare struct for P0.13b-2

New model for OpenStock MINUTE_DATA category (intraday time-share
ticks). Distinct from MinuteBar: no OHLC, has avg_price. All business
fields Option-wrapped to support INV-2C skip semantics (missing field
in a single record → warn+skip, not batch failure).

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: `fetch_minute_share` + parser in `src/sources/openstock_client.rs`

**Files:**
- Modify: `src/sources/openstock_client.rs` (insert `RawMinuteRecord` after `MinuteBarRecord` struct at L744; insert `fetch_minute_share` method after `fetch_minute_klines` method; insert `parse_minute_share` + `parse_time_minutes` helpers near `parse_minute_bar`; add 3 wiremock tests + 3 unit tests in `#[cfg(test)] mod tests`)
- Test: same file, `mod tests`

**Interfaces:**
- Consumes: `OpenStockClient::fetch::<T>(category, params)` at L180; `OpenStockResponse<T>` at L789; `fast_test_cfg` test helper; `MinuteShare` from Task 1; `tracing::warn!`
- Produces: `pub async fn fetch_minute_share(&self, code: &str, date: NaiveDate) -> Result<Vec<MinuteShare>>` — used by Task 3 handler

- [ ] **Step 1: Write the failing wiremock test for happy path**

In `src/sources/openstock_client.rs`, append to `#[cfg(test)] mod tests` (after existing `fetch_minute_klines_*` wiremock tests):

```rust
#[tokio::test]
async fn fetch_minute_share_sends_minute_data_category_and_date() {
    use wiremock::matchers::{method, path, body_partial_json};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .and(body_partial_json(serde_json::json!({
            "data_category": "MINUTE_DATA",
            "params": { "code": "sh600000", "date": "2026-07-01" }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "source": "eltdx",
            "artifact_hash": "abc123",
            "latency_ms": 42,
            "data": [
                { "time_minutes": "09:30", "price": 10.50, "volume": 12300, "amount": 129150.0, "avg_price": 10.50, "index": 0, "time": "2026-07-01T09:30:00", "price_milli": 10500 },
                { "time_minutes": "09:31", "price": 10.51, "volume": 8800, "amount": 92488.0, "avg_price": 10.505, "index": 1, "time": "2026-07-01T09:31:00", "price_milli": 10510 }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");
    let date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let shares = client.fetch_minute_share("sh600000", date).await.expect("fetch ok");
    assert_eq!(shares.len(), 2);
    assert_eq!(shares[0].code, "sh600000");
    assert_eq!(shares[0].timestamp, date.and_hms_opt(9, 30, 0).unwrap());
    assert_eq!(shares[0].price, Some(dec!(10.50)));
    assert_eq!(shares[0].volume, Some(12300));
    assert_eq!(shares[1].timestamp, date.and_hms_opt(9, 31, 0).unwrap());
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test --lib --package quantix-cli openstock_client::tests::fetch_minute_share_sends_minute_data_category_and_date
```

Expected: FAIL with "no method named `fetch_minute_share` found" or similar compile error.

- [ ] **Step 3: Add `RawMinuteRecord` struct**

Insert into `src/sources/openstock_client.rs`, immediately after the existing `MinuteBarRecord` struct (search `struct MinuteBarRecord` at L744 to find the location):

```rust
/// MINUTE_DATA 原始记录（8 字段，未裁剪）。
///
/// OpenStock envelope `records` 数组元素的反序列化目标。
/// 字段名对应 eLtdx MINUTE_DATA 输出：
///   - time_minutes: "0930" 或 "09:30" 格式
///   - price/volume/amount/avg_price: 业务字段（保留）
///   - index/time/price_milli: 冗余字段（serde default 容忍缺失）
///
/// **数值字段直接用 `Decimal`**（rust_decimal + serde 自动反序列化 JSON number）。
/// 若 live 测试发现字符串格式数值漂移，切换到 `serde_json::Value` + parse_decimal。
#[derive(Debug, serde::Deserialize)]
struct RawMinuteRecord {
    time_minutes: String,
    price: Option<rust_decimal::Decimal>,
    volume: Option<i64>,
    amount: Option<rust_decimal::Decimal>,
    avg_price: Option<rust_decimal::Decimal>,
}
```

Note: Do NOT add `#[allow(dead_code)]` — fields are read by `parse_minute_share` (added in Step 5).

- [ ] **Step 4: Add `fetch_minute_share` method**

Insert into `src/sources/openstock_client.rs`, immediately after the closing `}` of `fetch_minute_klines` method (search `pub async fn fetch_minute_klines` and find the method's closing brace). Insert as a new method on the same `impl OpenStockClient` block:

```rust
/// 消费 MINUTE_DATA category（分时点序列 / 分时图 ticks）。
///
/// 走 `/data/fetch` envelope 路径，复用 retry + circuit breaker
/// （与 `fetch_stock_codes` / `fetch_trade_dates` 同路径）。
///
/// **调用签名**（对齐 `fetch_stock_codes`）：`fetch<T>()` 接收
/// `(category: &str, params: Value)` 双参数，内部拼装为
/// `{data_category, params}` envelope。
///
/// **category 无 period/adjust 维度** — params 仅 `{code, date}`。
///
/// 解析：response.records 是 8 字段的数组，parse_minute_share 裁剪到
/// 5 业务字段。单条记录关键字段缺失 → warn + skip（INV-2C）。
pub async fn fetch_minute_share(
    &self,
    code: &str,
    date: NaiveDate,
) -> Result<Vec<crate::data::models::MinuteShare>> {
    let params = serde_json::json!({
        "code": code,
        "date": date.format("%Y-%m-%d").to_string(),
    });
    let resp = self.fetch::<RawMinuteRecord>("MINUTE_DATA", params).await?;
    let records = resp.records;
    let mut out = Vec::with_capacity(records.len());
    for raw in records {
        if let Some(share) = parse_minute_share(code, &raw, date) {
            out.push(share);
        } else {
            tracing::warn!(
                code = code,
                date = %date,
                time_minutes = %raw.time_minutes,
                "MINUTE_DATA record missing required field or invalid time, skipping"
            );
        }
    }
    Ok(out)
}
```

- [ ] **Step 5: Add `parse_minute_share` and `parse_time_minutes` helpers**

Insert as free `fn`s inside the same file, immediately before the `#[cfg(test)] mod tests` line (search `#[cfg(test)]` near end of file). Place after the existing `parse_minute_bar` helper (search `fn parse_minute_bar`):

```rust
/// 解析 MINUTE_DATA 单条记录为 `MinuteShare`。
///
/// 丢弃字段：`index`（内部序号）、`time`（ISO 冗余）、`price_milli`（毫表示）。
/// 保留字段：`time_minutes, price, volume, amount, avg_price`。
///
/// 返回 `Option<MinuteShare>`：当 4 个关键字段（price/volume/amount/avg_price）
/// 任一为 None，或 `time_minutes` 解析失败时返回 None，调用方 warn + skip（INV-2C）。
fn parse_minute_share(
    code: &str,
    raw: &RawMinuteRecord,
    date: NaiveDate,
) -> Option<crate::data::models::MinuteShare> {
    let price = raw.price?;
    let volume = raw.volume?;
    let amount = raw.amount?;
    let avg_price = raw.avg_price?;
    let (hh, mm) = parse_time_minutes(&raw.time_minutes)?;
    let timestamp = date.and_hms_opt(hh, mm, 0)?;
    Some(crate::data::models::MinuteShare {
        code: code.to_string(),
        timestamp,
        price: Some(price),
        volume: Some(volume),
        amount: Some(amount),
        avg_price: Some(avg_price),
    })
}

/// 解析 `time_minutes` 字段为 (HH, MM)。
///
/// 接受两种格式（D4 双格式容错，防御 R2 格式歧义）：
///   - "0930"     → (9, 30)
///   - "09:30"    → (9, 30)
///
/// 长度不匹配或字符非数字 → None（触发 INV-2C skip）。
fn parse_time_minutes(s: &str) -> Option<(u32, u32)> {
    let cleaned: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.len() != 4 {
        return None;
    }
    let hh: u32 = cleaned[..2].parse().ok()?;
    let mm: u32 = cleaned[2..].parse().ok()?;
    if hh >= 24 || mm >= 60 {
        return None;
    }
    Some((hh, mm))
}
```

- [ ] **Step 6: Run wiremock test to verify it passes**

```bash
cargo test --lib --package quantix-cli openstock_client::tests::fetch_minute_share_sends_minute_data_category_and_date
```

Expected: PASS.

- [ ] **Step 7: Write failing wiremock test for INV-2C skip semantics**

Append to `mod tests`:

```rust
#[tokio::test]
async fn fetch_minute_share_skips_records_with_missing_required_field() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "source": "eltdx",
            "data": [
                // record 0: complete
                { "time_minutes": "09:30", "price": 10.50, "volume": 100, "amount": 1050.0, "avg_price": 10.50 },
                // record 1: missing avg_price → skip
                { "time_minutes": "09:31", "price": 10.51, "volume": 200, "amount": 2102.0 },
                // record 2: missing volume → skip
                { "time_minutes": "09:32", "price": 10.52, "amount": 526.0, "avg_price": 10.52 },
                // record 3: invalid time_minutes → skip
                { "time_minutes": "99:99", "price": 10.53, "volume": 300, "amount": 3159.0, "avg_price": 10.53 },
                // record 4: complete
                { "time_minutes": "1130", "price": 10.54, "volume": 400, "amount": 4216.0, "avg_price": 10.54 }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");
    let date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let shares = client.fetch_minute_share("sh600000", date).await.expect("fetch ok");
    // 2 of 5 records pass: indices 0 and 4
    assert_eq!(shares.len(), 2, "expected 2 valid records, got {:?}", shares);
    assert_eq!(shares[0].timestamp, date.and_hms_opt(9, 30, 0).unwrap());
    assert_eq!(shares[1].timestamp, date.and_hms_opt(11, 30, 0).unwrap());
}
```

- [ ] **Step 8: Run to verify fail-then-pass**

First run to confirm it compiles (test should pass on first try since parser already implemented in Step 5):

```bash
cargo test --lib --package quantix-cli openstock_client::tests::fetch_minute_share_skips_records_with_missing_required_field
```

Expected: PASS. If fail, inspect parser logic.

- [ ] **Step 9: Write failing wiremock test for 4xx propagation**

Append to `mod tests`:

```rust
#[tokio::test]
async fn fetch_minute_share_propagates_4xx() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": { "code": "NOT_FOUND", "message": "unknown code" }
        })))
        // envelope retry policy: 4xx → fail-fast, expect exactly 1 call
        .expect(1)
        .mount(&server)
        .await;

    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");
    let date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let err = client.fetch_minute_share("invalid_code", date).await.expect_err("must error");
    let msg = format!("{err}");
    assert!(msg.contains("404") || msg.contains("NOT_FOUND") || msg.contains("unknown"),
        "expected error to mention status/error, got: {msg}");
}
```

- [ ] **Step 10: Run to verify pass**

```bash
cargo test --lib --package quantix-cli openstock_client::tests::fetch_minute_share_propagates_4xx
```

Expected: PASS.

- [ ] **Step 11: Add unit tests for `parse_time_minutes`**

Append to `mod tests`:

```rust
#[test]
fn parse_time_minutes_accepts_compact_format() {
    assert_eq!(parse_time_minutes("0930"), Some((9, 30)));
    assert_eq!(parse_time_minutes("1130"), Some((11, 30)));
    assert_eq!(parse_time_minutes("1500"), Some((15, 0)));
}

#[test]
fn parse_time_minutes_accepts_colon_format() {
    assert_eq!(parse_time_minutes("09:30"), Some((9, 30)));
    assert_eq!(parse_time_minutes("11:30"), Some((11, 30)));
}

#[test]
fn parse_time_minutes_rejects_invalid() {
    assert_eq!(parse_time_minutes("99:99"), None);   // out of range
    assert_eq!(parse_time_minutes("25:00"), None);   // hour > 23
    assert_eq!(parse_time_minutes("12:60"), None);   // minute > 59
    assert_eq!(parse_time_minutes("abc"), None);     // non-numeric
    assert_eq!(parse_time_minutes("123"), None);     // too short
    assert_eq!(parse_time_minutes("12345"), None);   // too long
}
```

- [ ] **Step 12: Run all openstock_client tests**

```bash
cargo test --lib --package quantix-cli openstock_client::tests
```

Expected: ALL PASS (existing P0.13a/P0.13b-1 tests + new tests).

- [ ] **Step 13: Run clippy + fmt**

```bash
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -20
cargo fmt --all
```

Expected: 0 warnings.

- [ ] **Step 14: Commit**

```bash
git add src/sources/openstock_client.rs
git commit -m "$(cat <<'EOF'
feat(sources): add fetch_minute_share client method for P0.13b-2

Consumes MINUTE_DATA category via /data/fetch envelope path (with
retry + circuit breaker). Uses two-arg fetch<T>("MINUTE_DATA", params)
signature consistent with fetch_stock_codes. RawMinuteRecord uses
Option<Decimal> directly (no from_f64_retain hop). parse_minute_share
returns Option, enabling INV-2C skip semantics (missing field → warn+skip,
not batch failure). parse_time_minutes accepts both "0930" and "09:30"
formats to mitigate unknown wire schema (R2).

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: CLI wiring (`FetchMinuteShare` subcommand + handler + dispatcher)

**Files:**
- Modify: `src/cli/commands/data.rs` (add `FetchMinuteShare` variant after `FetchMinuteKlines` at L386)
- Modify: `src/cli/handlers/openstock_handler.rs` (add `fetch_openstock_minute_share` after `fetch_openstock_minute_klines` at L411)
- Modify: `src/cli/handlers/mod.rs` (add re-export at L130)
- Modify: `src/cli/handlers/app_shell.rs` (add dispatcher arm after `FetchMinuteKlines` arm at L385)

**Interfaces:**
- Consumes: `OpenStockClient::from_settings`, `OpenStockSettings` (existing), `NaiveDate::parse_from_str`, `MinuteShare` from Task 1, `fetch_minute_share` from Task 2
- Produces: `pub(crate) async fn fetch_openstock_minute_share(settings: &OpenStockSettings, symbol: String, date_str: String) -> Result<()>` — invoked by CLI dispatcher

- [ ] **Step 1: Add `FetchMinuteShare` CLI enum variant**

In `src/cli/commands/data.rs`, search for `FetchMinuteKlines {` (around L386). Insert immediately after the closing `}` of the `FetchMinuteKlines` variant block:

```rust
    /// Fetch OpenStock MINUTE_DATA category (intraday time-share ticks).
    ///
    /// Returns per-minute price + avg_price for the given code/date.
    /// Distinct from fetch-minute-klines: no OHLC, no period/adjust.
    FetchMinuteShare {
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        date: String,
    },
```

- [ ] **Step 2: Add handler `fetch_openstock_minute_share`**

In `src/cli/handlers/openstock_handler.rs`, search for `pub(crate) async fn fetch_openstock_minute_klines` (L411) to find the location. Insert immediately after the closing `}` of that handler:

```rust
pub(crate) async fn fetch_openstock_minute_share(
    settings: &crate::core::runtime::OpenStockSettings,
    symbol: String,
    date_str: String,
) -> crate::core::Result<()> {
    let client = crate::sources::openstock_client::OpenStockClient::from_settings(settings)?;
    let date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .map_err(|e| crate::core::QuantixError::Other(format!("invalid date '{}': {}", date_str, e)))?;
    let started = std::time::Instant::now();
    let shares = client.fetch_minute_share(&symbol, date).await?;
    let latency_ms = started.elapsed().as_millis();

    let base_url = settings.base_url.as_deref().unwrap_or("(not set)");
    println!("OpenStock MINUTE_DATA (time-share ticks)");
    println!("  Code:     {}", symbol);
    println!("  Date:     {}", date);
    println!("  Endpoint: {}/data/fetch", base_url);
    println!("  Records:  {}", shares.len());
    if let (Some(first), Some(last)) = (shares.first(), shares.last()) {
        println!("  First:    {:?}", first);
        println!("  Last:     {:?}", last);
    }
    println!("  latency_ms: {}", latency_ms);
    Ok(())
}
```

- [ ] **Step 3: Add re-export in `mod.rs`**

In `src/cli/handlers/mod.rs`, search `fetch_openstock_minute_klines` (L130). Append to the existing re-export line:

Change:
```rust
    fetch_openstock_index, fetch_openstock_klines, fetch_openstock_minute_klines,
```
To:
```rust
    fetch_openstock_index, fetch_openstock_klines, fetch_openstock_minute_klines,
    fetch_openstock_minute_share,
```

(Or add as a new line in the same `pub use crate::cli::handlers::openstock_handler::{...}` block.)

- [ ] **Step 4: Add dispatcher arm in `app_shell.rs`**

In `src/cli/handlers/app_shell.rs`, search `OpenStockCommands::FetchMinuteKlines {` (L385). Insert immediately after the closing `}` of that match arm (around L390, before next `OpenStockCommands::*` arm):

```rust
            OpenStockCommands::FetchMinuteShare { symbol, date } => {
                fetch_openstock_minute_share(&rt.openstock, symbol, date).await?;
            }
```

- [ ] **Step 5: Verify CLI compiles + clippy**

```bash
cargo build --bin quantix 2>&1 | tail -10
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -10
cargo fmt --all
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 6: Verify CLI smoke (help)**

```bash
cargo run -q -- data openstock fetch-minute-share --help
```

Expected: prints help with `--symbol`, `--date` flags.

- [ ] **Step 7: Commit**

```bash
git add src/cli/commands/data.rs src/cli/handlers/openstock_handler.rs src/cli/handlers/mod.rs src/cli/handlers/app_shell.rs
git commit -m "$(cat <<'EOF'
feat(cli): add fetch-minute-share CLI subcommand for P0.13b-2

Wires fetch_minute_share to the CLI under 'data openstock fetch-minute-share'.
Handler mirrors fetch-minute-klines output shape but omits Period/Adjust
rows (MINUTE_DATA doesn't support those dimensions). Uses date_str param
name to avoid shadow with parsed NaiveDate. base_url from settings is
Option<String>, unwrapped with "(not set)" sentinel.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Live tests + OpenSpec change + governance card

**Files:**
- Create: `tests/openstock_live_minute_share.rs`
- Create: `openspec/changes/openstock-data-consumption-p0-13b-2/proposal.md`
- Create: `openspec/changes/openstock-data-consumption-p0-13b-2/tasks.md`
- Create: `openspec/changes/openstock-data-consumption-p0-13b-2/design.md`
- Create: `openspec/changes/openstock-data-consumption-p0-13b-2/specs/openstock-data-consumption/spec.md`
- Create: `.governance/programs/project-governance/cards/P0.13b-2.yaml`

**Interfaces:**
- Consumes: `OpenStockClient::from_env`/`from_settings`, `fetch_minute_share` from Task 2, `MinuteShare` from Task 1
- Produces: live integration test file, OpenSpec change package, governance card

- [ ] **Step 1: Create live tests file**

Create `tests/openstock_live_minute_share.rs`:

```rust
//! Live integration tests for OpenStock MINUTE_DATA category (P0.13b-2).
//!
//! Skipped by default. Run with:
//!   QUANTIX_OPENSTOCK_LIVE=1 \
//!   OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
//!   OPENSTOCK_API_KEY=<key> \
//!   cargo test --test openstock_live_minute_share -- --ignored

#![cfg(test)]

use quantix_cli::core::runtime::OpenStockSettings;
use quantix_cli::sources::openstock_client::OpenStockClient;

fn settings_from_env() -> Option<OpenStockSettings> {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        return None;
    }
    let base_url = std::env::var("OPENSTOCK_BASE_URL").ok()?;
    let api_key = std::env::var("OPENSTOCK_API_KEY").ok()?;
    Some(OpenStockSettings {
        base_url: Some(base_url),
        api_key: Some(api_key),
        timeout_secs: 30,
    })
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_minute_share_live_sh600000_recent_trading_day() {
    let Some(settings) = settings_from_env() else {
        return;
    };
    let client = OpenStockClient::from_settings(&settings).expect("client build");
    // Use a recent past date; adjust if market was closed (weekend/holiday)
    let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
    let shares = client.fetch_minute_share("sh600000", date).await.expect("fetch ok");
    assert!(!shares.is_empty(), "expected non-empty time-share ticks");
    // Trading hours for SH: 09:30-11:30, 13:00-15:00 → first tick around 09:30
    let first = &shares[0];
    assert!(first.timestamp.hour() >= 9, "first tick hour too early: {:?}", first);
    // Sanity: avg_price should be positive
    assert!(first.avg_price.map(|p| p > rust_decimal::Decimal::ZERO).unwrap_or(false),
        "expected positive avg_price, got: {:?}", first);
    println!("L1 sh600000 {} -> {} ticks, first={:?}", date, shares.len(), first);
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_minute_share_live_weekend_returns_empty_or_error() {
    let Some(settings) = settings_from_env() else {
        return;
    };
    let client = OpenStockClient::from_settings(&settings).expect("client build");
    // 2026-06-27 is a Saturday → market closed
    let saturday = chrono::NaiveDate::from_ymd_opt(2026, 6, 27).unwrap();
    let result = client.fetch_minute_share("sh600000", saturday).await;
    // Either empty Vec (graceful) or Err (envelope error) — both acceptable
    match result {
        Ok(shares) => {
            assert!(shares.is_empty(),
                "expected empty ticks on weekend, got {} records: {:?}", shares.len(), &shares[..shares.len().min(3)]);
            println!("L2 weekend → empty Vec (graceful)");
        }
        Err(e) => {
            println!("L2 weekend → envelope error (acceptable): {e}");
        }
    }
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_minute_share_live_unknown_code_propagates_error() {
    let Some(settings) = settings_from_env() else {
        return;
    };
    let client = OpenStockClient::from_settings(&settings).expect("client build");
    let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
    let result = client.fetch_minute_share("invalid_code_xyz", date).await;
    assert!(result.is_err(), "expected error for unknown code, got: {:?}", result);
    println!("L3 unknown code → error: {:?}", result.unwrap_err());
}
```

Note: Verify the import paths (`quantix_cli::core::runtime::OpenStockSettings`, `quantix_cli::sources::openstock_client::OpenStockClient`) by grepping an existing live test like `tests/openstock_live_minute_klines.rs`. Adjust crate name / pub visibility if different.

- [ ] **Step 2: Verify integration test compiles (skipped without env)**

```bash
cargo test --test openstock_live_minute_share 2>&1 | tail -5
```

Expected: 3 tests, all ignored (no output / "ignored").

- [ ] **Step 3: Create OpenSpec proposal**

```bash
mkdir -p openspec/changes/openstock-data-consumption-p0-13b-2/specs/openstock-data-consumption
```

Write `openspec/changes/openstock-data-consumption-p0-13b-2/proposal.md`:

```markdown
# Proposal: openstock-data-consumption-p0-13b-2

## Why

P0.13b-1 added minute OHLC candles via `/data/bars` (KLINES minute periods).
P0.13b-2 completes the minute-level story by adding intraday time-share ticks
via the `MINUTE_DATA` category. Time-share ticks (price + avg_price per
minute, no OHLC) are required to render intraday 分时图 charts in MyStocks
frontend.

The two paths are architecturally orthogonal: P0.13b-1 uses direct reqwest
(no retry); P0.13b-2 uses the `/data/fetch` envelope path (with retry +
circuit breaker, same path as fetch_stock_codes). Each fits in its own
slice for risk isolation.

## What Changes

- Add `MinuteShare` struct in `src/data/models.rs` (Option-wrapped fields
  for INV-2C skip semantics)
- Add `fetch_minute_share(code, date)` client method in
  `src/sources/openstock_client.rs` — calls `self.fetch::<T>("MINUTE_DATA", params)`
- Add `parse_minute_share` + `parse_time_minutes` inline helpers
- Add `FetchMinuteShare { --symbol, --date }` CLI subcommand
- Add `fetch_openstock_minute_share` handler
- Add 3 `#[ignore]` live tests (L1/L2/L3)
- Add governance card `P0.13b-2.yaml`

## Impact

| Area | Change |
|------|--------|
| `src/data/models.rs` | +60 lines (struct + tests) |
| `src/sources/openstock_client.rs` | +180 lines (method + helpers + tests) |
| `src/cli/commands/data.rs` | +6 lines (enum variant) |
| `src/cli/handlers/openstock_handler.rs` | +30 lines (handler) |
| `src/cli/handlers/mod.rs` | +1 line (re-export) |
| `src/cli/handlers/app_shell.rs` | +3 lines (dispatcher arm) |
| `tests/openstock_live_minute_share.rs` | +60 lines (new file) |

Total: ~340 lines added, 0 deleted. No P0.13b-1 code modified.

## Non-Goals

- Multi-day range queries (P0.13c)
- ClickHouse writes / shadow persistence integration
- Other categories (REALTIME_QUOTES, depth, etc.)
- Refactoring envelope retry/circuit breaker
- Migrating existing parsers to a dedicated module (cross-slice refactor)
```

- [ ] **Step 4: Create OpenSpec tasks**

Write `openspec/changes/openstock-data-consumption-p0-13b-2/tasks.md`:

```markdown
# Tasks: openstock-data-consumption-p0-13b-2

## 0. Baseline and Governance

- [ ] Confirm HEAD is at the post-P0.13b-1 merge commit
- [ ] Create `.governance/programs/project-governance/cards/P0.13b-2.yaml`
      with allowed_paths covering all files touched in this slice and
      forbidden_paths excluding P0.13b-1 symbols

## 1. MinuteShare Model (Task 1)

- [ ] Add `MinuteShare` struct to `src/data/models.rs` with Option-wrapped
      business fields
- [ ] Add unit test `minute_share_round_trip_serde`
- [ ] Add unit test `minute_share_allows_missing_optional_fields`

## 2. Client Method (Task 2)

- [ ] Add `RawMinuteRecord` struct with `Option<Decimal>` business fields
- [ ] Add `fetch_minute_share` method calling
      `self.fetch::<RawMinuteRecord>("MINUTE_DATA", params)`
- [ ] Add `parse_minute_share` returning `Option<MinuteShare>`
- [ ] Add `parse_time_minutes` accepting "0930" and "09:30" formats
- [ ] Add wiremock test `fetch_minute_share_sends_minute_data_category_and_date`
- [ ] Add wiremock test `fetch_minute_share_skips_records_with_missing_required_field`
- [ ] Add wiremock test `fetch_minute_share_propagates_4xx`
- [ ] Add unit tests for `parse_time_minutes`

## 3. CLI Wiring (Task 3)

- [ ] Add `FetchMinuteShare` variant to `OpenStockCommands` enum
- [ ] Add `fetch_openstock_minute_share` handler
- [ ] Re-export in `src/cli/handlers/mod.rs`
- [ ] Add dispatcher arm in `app_shell.rs`

## 4. Live Tests (Task 4)

- [ ] Create `tests/openstock_live_minute_share.rs` with L1/L2/L3 tests
- [ ] All tests `#[ignore]` + `QUANTIX_OPENSTOCK_LIVE=1` env gate

## 5. OpenSpec Change

- [ ] proposal.md, tasks.md, design.md, spec deltas

## 6. Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --workspace -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `openspec validate openstock-data-consumption-p0-13b-2 --strict`
- [ ] `openspec validate --all --strict`
- [ ] `gitnexus detect_changes` — expect LOW risk
```

- [ ] **Step 5: Create OpenSpec design**

Write `openspec/changes/openstock-data-consumption-p0-13b-2/design.md`:

```markdown
# Design: openstock-data-consumption-p0-13b-2

Full design rationale: `docs/superpowers/specs/2026-07-02-openstock-p0-13b-2-minute-share-design.md`
(R1 revision `d25410d`).

## Key Decisions

- **D1**: Inline category string `"MINUTE_DATA"` (matches fetch_stock_codes style)
- **D2**: `RawMinuteRecord` uses `Option<Decimal>` directly (no from_f64_retain hop)
- **D3**: `parse_minute_share` returns `Option` (not `Result`) to support INV-2C skip
- **D4**: `parse_time_minutes` accepts both "0930" and "09:30" formats (R2 mitigation)
- **D5**: Parser inline in `openstock_client.rs` (consistent with P0.13b-1)
- **D6**: All price/amount/avg_price fields use `Decimal` (project-wide consistency)

## Risks

- **R1**: MINUTE_DATA actual schema unverified → `#[serde(default)]` + wiremock-first
- **R2**: `time_minutes` format ambiguity → D4 dual-format acceptance
- **R3**: String-vs-number serde drift → fall back to `serde_json::Value` + `parse_decimal`
- **R4**: Envelope-level failure (5xx, retry exhausted) → whole batch fails (different
       dimension from INV-2C single-record skip)
- **R5**: Concurrent modification with P0.13b-1 → none (P0.13b-1 merged)

## Invariants

- **INV-1A**: Must use `self.fetch::<T>()` envelope path (not direct reqwest)
- **INV-1B**: Request params must be `{code, date}` only (no period/adjust)
- **INV-2C**: Single record missing field → warn+skip (not batch fail)
- **INV-3**: `price`/`amount`/`avg_price` are `Decimal`; `volume` is `i64`
- **INV-4**: Never bypass envelope retry/circuit breaker
```

- [ ] **Step 6: Create OpenSpec spec deltas**

Write `openspec/changes/openstock-data-consumption-p0-13b-2/specs/openstock-data-consumption/spec.md`:

```markdown
## ADDED Requirements

### Requirement: Consume MINUTE_DATA Category

The system SHALL provide a `fetch_minute_share(code, date)` client method
that consumes the OpenStock `MINUTE_DATA` category via the `/data/fetch`
envelope path with retry and circuit breaker.

#### Scenario: Successful fetch with complete records

- WHEN `fetch_minute_share` is called with a valid code and trading day
- THEN the system issues `POST /data/fetch` with body
  `{data_category: "MINUTE_DATA", params: {code, date}}`
- AND returns `Vec<MinuteShare>` containing all complete records

#### Scenario: Records with missing fields are skipped

- WHEN the envelope contains records where one or more required fields
  (price, volume, amount, avg_price) are missing
- THEN the system emits a `tracing::warn!` for each skipped record
- AND returns `Vec<MinuteShare>` containing only the complete records
- AND does NOT fail the whole operation

#### Scenario: 4xx HTTP response

- WHEN the OpenStock runtime returns HTTP 4xx
- THEN the system fails fast (no retry) and propagates the error

### Requirement: MinuteShare Model

The system SHALL provide a `MinuteShare` struct with fields:
`code: String`, `timestamp: NaiveDateTime`, `price: Option<Decimal>`,
`volume: Option<i64>`, `amount: Option<Decimal>`,
`avg_price: Option<Decimal>`.

#### Scenario: Serialization round-trip

- WHEN a `MinuteShare` is serialized and deserialized
- THEN all fields are preserved

#### Scenario: Missing optional fields deserialize as None

- WHEN a JSON record omits one or more optional fields
- THEN serde deserialization succeeds with the missing fields as `None`

### Requirement: CLI Subcommand fetch-minute-share

The system SHALL provide a `data openstock fetch-minute-share` CLI
subcommand accepting `--symbol` and `--date` (YYYY-MM-DD) arguments.

#### Scenario: CLI smoke

- WHEN invoked as `data openstock fetch-minute-share --symbol sh600000 --date 2026-06-30`
- THEN the system fetches and prints the time-share ticks summary
```

- [ ] **Step 7: Create governance card**

Write `.governance/programs/project-governance/cards/P0.13b-2.yaml`:

```yaml
id: P0.13b-2
title: "OpenStock MINUTE_DATA time-share ticks (分时点序列)"
state: in_progress
scope:
  allowed_paths:
    - src/data/models.rs
    - src/sources/openstock_client.rs
    - src/cli/commands/data.rs
    - src/cli/handlers/openstock_handler.rs
    - src/cli/handlers/mod.rs
    - src/cli/handlers/app_shell.rs
    - tests/openstock_live_minute_share.rs
    - openspec/changes/openstock-data-consumption-p0-13b-2/**
    - docs/superpowers/specs/2026-07-02-openstock-p0-13b-2-minute-share-design.md
  forbidden_paths:
    - src/db/**
    - src/backtest/**
    - src/execution/**
    - src/sources/openstock.rs
    - src/sources/openstock_shadow.rs
    - src/sources/kline_aggregator.rs
    - src/sources/openstock_client.rs::fetch_klines          # P0.13a
    - src/sources/openstock_client.rs::fetch_minute_klines   # P0.13b-1
    - src/data/models.rs::Kline                              # P0.13a
    - src/data/models.rs::BarPeriod                          # P0.13a
    - src/data/models.rs::MinutePeriod                       # P0.13b-1
    - src/data/models.rs::MinuteBar                          # P0.13b-1
acceptance_gates:
  - cargo fmt --all -- --check
  - cargo clippy --all-targets --workspace -- -D warnings
  - cargo test --workspace
  - openspec validate openstock-data-consumption-p0-13b-2 --strict
  - openspec validate --all --strict
non_goals:
  - "Multi-day range queries (P0.13c)"
  - "ClickHouse writes / shadow persistence"
  - "Migrate existing parsers to dedicated module"
  - "Modify P0.13b-1 symbols (MinuteBar, MinutePeriod, fetch_minute_klines)"
```

- [ ] **Step 8: Run full verification**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -5
cargo test --workspace 2>&1 | tail -20
openspec validate openstock-data-consumption-p0-13b-2 --strict
openspec validate --all --strict
gitnexus detect_changes
git diff --check
```

Expected: all PASS, detect_changes LOW risk.

- [ ] **Step 9: Commit**

```bash
git add tests/openstock_live_minute_share.rs \
        openspec/changes/openstock-data-consumption-p0-13b-2 \
        .governance/programs/project-governance/cards/P0.13b-2.yaml
git commit -m "$(cat <<'EOF'
docs(p0-13b-2): add live tests, OpenSpec change, governance card

Live tests L1/L2/L3 cover sh600000 happy path, weekend edge case, and
unknown code error propagation. OpenSpec change package mirrors P0.13b-1
shape. Governance card forbids touching P0.13a/P0.13b-1 symbols.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## End of Plan

After all 4 tasks complete, the controller dispatches a whole-branch code review using `superpowers:requesting-code-review`. Post-merge follow-ups (out of plan scope):
- `git push origin master`
- `gitnexus analyze`
- `openspec archive openstock-data-consumption-p0-13b-2`
- Flip governance card `P0.13b-2.yaml` state → `completed`
- Begin P0.13c brainstorm (multi-day range queries)
