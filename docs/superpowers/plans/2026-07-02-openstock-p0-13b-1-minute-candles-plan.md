# OpenStock P0.13b-1 (Minute Candles) Implementation Plan

> **Revision R1 (2026-07-02)**: Fixed `test_client_for` → `OpenStockClient::new(fast_test_cfg(server.uri()))` per `2026-07-02-openstock-p0-13b-1-plan-review.md` (3 wiremock tests would have failed to compile). Removed redundant `OpenStockSettings` inline import (already at file scope L4). Clarified helper location at `src/sources/openstock_client.rs:789`.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire OpenStock `/data/bars?period=1m|5m|15m|30m|60m` to a new `fetch_minute_klines` client method returning `Vec<MinuteBar>`, plus a `fetch-minute-klines` CLI subcommand.

**Architecture:** Mirrors P0.13a `fetch_klines` shape: direct reqwest to `/data/bars` (no envelope, no retry, no breaker). Key difference: parses full ISO timestamp into `NaiveDateTime` (not `NaiveDate`), so adds `NaiveDateTime` to `chrono` imports. New types `MinutePeriod` + `MinuteBar` live in `src/data/models.rs` alongside `BarPeriod` + `Kline`. Handler uses existing `OpenStockClient::from_settings()` + `OpenStockSettings` pattern.

**Tech Stack:** Rust + tokio + reqwest + serde + chrono + rust_decimal + wiremock (tests).

## Global Constraints

- **Naming** (from spec §4.1.2 D3): The new OHLCV-minute struct is named `MinuteBar` (NOT `MinuteKline`). `MinuteKline` already exists at `src/db/tdengine.rs:37` (re-exported via `src/db/mod.rs:17`) and uses `f64`/`DateTime<Utc>` — collision would cause compile ambiguity.
- **Wire tokens** (from spec D4, INV-1A): `MinutePeriod::as_str()` returns `"1m"|"5m"|"15m"|"30m"|"60m"` exactly. `FromStr` accepts only these 5 tokens (case-insensitive). **Rejects** all aliases (`1min`, `minute`, `5min`, `1h`, `hour`). This is mandatory because OpenStock `_PERIOD_MAP.get(period, "day")` silently falls back to `"day"` for unknown tokens — a too-loose `FromStr` would silently return day candles.
- **Adjust field omission** (from spec INV-1D): When `AdjustType::None`, the request body MUST omit `adjust` entirely (do not send `"adjust": ""`). Mirror P0.13a `fetch_klines` L600-602.
- **No retry** (from spec INV-1C): `/data/bars` errors propagate as `QuantixError::Other` on first failure. Do NOT route through `OpenStockClient::fetch<T>()` (which has retry + breaker).
- **`Kline` immutability**: Do NOT modify P0.13a's `Kline` struct, `BarPeriod` enum, or `fetch_klines` method. `MinuteBar` is a parallel type.
- **Error pattern**: `QuantixError::Config` for CLI argument parse failures (fail-fast before HTTP); `QuantixError::Other` for HTTP/parse failures.
- **File size limits** (per CLAUDE.md): existing files modified here are all comfortably under limits (`models.rs` ~230 lines, `openstock_client.rs` ~700+ lines but adding ~80 lines still under 800 warn, `openstock_handler.rs` ~500 lines, `data.rs` ~370 lines, `app_shell.rs` ~400 lines, `mod.rs` ~135 lines).
- **Test pattern**: TDD — write failing test → run to verify fail → implement minimal → run to verify pass → commit. Wiremock tests use `body_partial_json` + `.expect(1)` for no-retry verification (copy P0.13a T5 pattern at `openstock_client.rs:1176-1200`).
- **Live test gating**: `#[tokio::test] #[ignore = "..."]` + `if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") { return; }` early-return (copy `tests/openstock_live_klines.rs` pattern from P0.13a).

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `src/data/models.rs` | Modify | Add `MinutePeriod` enum + `MinuteBar` struct + unit tests |
| `src/sources/openstock_client.rs` | Modify | Add `fetch_minute_klines` method + 3 wiremock tests |
| `src/cli/commands/data.rs` | Modify | Add `FetchMinuteKlines` variant to `OpenStockCommands` |
| `src/cli/handlers/openstock_handler.rs` | Modify | Add `fetch_openstock_minute_klines` async fn |
| `src/cli/handlers/mod.rs` | Modify | Re-export `fetch_openstock_minute_klines` |
| `src/cli/handlers/app_shell.rs` | Modify | Add dispatcher arm |
| `tests/openstock_live_minute_klines.rs` | Create | 3 `#[ignore]` live tests |
| `openspec/changes/openstock-data-consumption-p0-13b-1/` | Create | proposal.md, tasks.md, design.md, specs/openstock-data-consumption/spec.md |
| `.governance/programs/project-governance/cards/P0.13b-1.yaml` | Create | Governance card |

---

## Task 1: `MinutePeriod` enum + `MinuteBar` struct + unit tests

**Files:**
- Modify: `src/data/models.rs:39-104` (after `BarPeriod` impl, before `AdjustType::as_openstock_param`)
- Test: same file, `#[cfg(test)] mod tests` block at L188-233

**Interfaces:**
- Produces: `pub enum MinutePeriod { Minute1, Minute5, Minute15, Minute30, Minute60 }` with `as_str()` returning `"1m"|"5m"|"15m"|"30m"|"60m"` and `impl FromStr` (strict whitelist).
- Produces: `pub struct MinuteBar { code: String, timestamp: NaiveDateTime, open: Decimal, high: Decimal, low: Decimal, close: Decimal, volume: i64, amount: Option<Decimal>, adjust_type: AdjustType }`.

- [ ] **Step 1: Write the failing tests**

Append to `src/data/models.rs` `#[cfg(test)] mod tests` block (after the existing `adjust_type_from_str_case_insensitive` test, before the closing `}`):

```rust
    #[test]
    fn minute_period_as_str_round_trip() {
        assert_eq!(MinutePeriod::Minute1.as_str(), "1m");
        assert_eq!(MinutePeriod::Minute5.as_str(), "5m");
        assert_eq!(MinutePeriod::Minute15.as_str(), "15m");
        assert_eq!(MinutePeriod::Minute30.as_str(), "30m");
        assert_eq!(MinutePeriod::Minute60.as_str(), "60m");
    }

    #[test]
    fn minute_period_from_str_accepts_canonical_case_insensitive() {
        assert!(matches!(MinutePeriod::from_str("1m"), Ok(MinutePeriod::Minute1)));
        assert!(matches!(MinutePeriod::from_str("5M"), Ok(MinutePeriod::Minute5)));
        assert!(matches!(MinutePeriod::from_str("15m"), Ok(MinutePeriod::Minute15)));
        assert!(matches!(MinutePeriod::from_str("30M"), Ok(MinutePeriod::Minute30)));
        assert!(matches!(MinutePeriod::from_str("60m"), Ok(MinutePeriod::Minute60)));
    }

    #[test]
    fn minute_period_from_str_rejects_aliases() {
        // D4 strict — reject 1min/minute/5min/1h/hour and any day* value
        assert!(MinutePeriod::from_str("1min").is_err());
        assert!(MinutePeriod::from_str("minute").is_err());
        assert!(MinutePeriod::from_str("5min").is_err());
        assert!(MinutePeriod::from_str("1h").is_err());
        assert!(MinutePeriod::from_str("hour").is_err());
        assert!(MinutePeriod::from_str("day").is_err());
        assert!(MinutePeriod::from_str("").is_err());
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib --package quantix-cli data::models::tests::minute_period`
Expected: FAIL with "cannot find type `MinutePeriod` in this scope" (compile error).

- [ ] **Step 3: Implement `MinutePeriod` and `MinuteBar`**

Insert in `src/data/models.rs` after the existing `impl std::str::FromStr for BarPeriod` block (around L73, before `impl AdjustType`):

```rust
/// `/data/bars` 分钟周期参数 (P0.13b-1, OpenStock API).
///
/// 与 `BarPeriod`（day/week/month）语义域不同：分钟蜡烛返回
/// `Vec<MinuteBar>`（含 `NaiveDateTime` 时间戳），日线/周线/月线
/// 返回 `Vec<Kline>`（仅 `NaiveDate`）。类型系统强制调用方区分。
///
/// Wire tokens `1m|5m|15m|30m|60m` 直接对应 OpenStock `_PERIOD_MAP`
/// 主 token。**拒绝所有别名**（`1min|minute|5min|1h|hour` 等），
/// 因为 `_PERIOD_MAP.get(period, "day")` 对未知 token 静默回退到
/// day——严格白名单 + fail-fast 是唯一安全策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinutePeriod {
    Minute1,
    Minute5,
    Minute15,
    Minute30,
    Minute60,
}

impl MinutePeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Minute1 => "1m",
            Self::Minute5 => "5m",
            Self::Minute15 => "15m",
            Self::Minute30 => "30m",
            Self::Minute60 => "60m",
        }
    }
}

impl std::str::FromStr for MinutePeriod {
    type Err = QuantixError;

    /// 仅接受 `1m|5m|15m|30m|60m`（任意大小写）。拒绝所有别名
    /// （`1min|minute|5min|1h|hour` 等）和任何非 5 个主 token 的值。
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "1m" => Ok(Self::Minute1),
            "5m" => Ok(Self::Minute5),
            "15m" => Ok(Self::Minute15),
            "30m" => Ok(Self::Minute30),
            "60m" => Ok(Self::Minute60),
            other => Err(QuantixError::Config(format!(
                "unsupported MinutePeriod `{}`: expected one of 1m|5m|15m|30m|60m",
                other
            ))),
        }
    }
}

/// 分钟级 K 线蜡烛（P0.13b-1 新增）。
///
/// **命名说明**：命名为 `MinuteBar`（不是 `MinuteKline`），因为
/// `src/db/tdengine.rs:37` 已存在公开 re-export 的 `MinuteKline`{
/// ts: DateTime<Utc>, code, open: f64, ... }——TDengine 行映射用 f64。
/// 本类型用 `Decimal` + `AdjustType`，语义不同，必须避免名称碰撞。
/// `MinuteBar` 与 P0.13a `BarPeriod` 形成请求/响应语义对。
///
/// 与 `Kline`（日线）的区别：
/// - `timestamp: NaiveDateTime`（精确到分钟）vs `date: NaiveDate`
/// - 其他字段与 `Kline` 一致
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteBar {
    pub code: String,
    pub timestamp: NaiveDateTime,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: i64,
    pub amount: Option<Decimal>,
    pub adjust_type: AdjustType,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib --package quantix-cli data::models::tests::minute_period`
Expected: PASS (3 tests).

- [ ] **Step 5: Run clippy + fmt on changed file**

Run: `cargo clippy --all-targets --workspace -- -D warnings 2>&1 | grep -E "warning|error" | head -10`
Expected: no output (clean).

Run: `cargo fmt --all -- --check`
Expected: clean (or apply `cargo fmt --all` first if dirty).

- [ ] **Step 6: Commit**

```bash
git add src/data/models.rs
git commit -m "feat(data): add MinutePeriod enum and MinuteBar struct for P0.13b-1

MinutePeriod uses strict 1m|5m|15m|30m|60m whitelist FromStr to defend
against OpenStock _PERIOD_MAP silent fallback to day. MinuteBar uses
NaiveDateTime (vs Kline's NaiveDate) to preserve minute-level precision.
Named MinuteBar (not MinuteKline) to avoid collision with src/db/tdengine.rs:37.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 2: `fetch_minute_klines` client method + 3 wiremock tests

**Files:**
- Modify: `src/sources/openstock_client.rs` — add method after `fetch_klines` (around L675, before the closing `}` of `impl OpenStockClient`)
- Test: same file, in `#[cfg(test)] mod tests` block (after `fetch_klines_propagates_4xx` test around L1200)

**Interfaces:**
- Consumes: `MinutePeriod` (from Task 1), `AdjustType::as_openstock_param()`, `OpenStockClient.{base_url, http, api_key}` (existing fields).
- Produces: `pub async fn fetch_minute_klines(&self, code: &str, period: MinutePeriod, date: NaiveDate, adjust: AdjustType) -> Result<Vec<MinuteBar>>`.
- Reuses pattern: copy `fetch_klines` (L581-675) structure verbatim — only change `period.as_str()` to `period.as_str()` (same — both return `&'static str`), `date` field instead of `start_date`/`end_date`, and parse `&bar.time[..19]` as `NaiveDateTime` (vs `&bar.time[..10]` as `NaiveDate`).

**Key wire contract** (verified in spec): request body is:
```json
{
  "symbol": "sh600000",
  "period": "1m",
  "date": "2026-07-02",
  "adjust": "qfq"  // omitted entirely when adjust==None
}
```

Response shape is identical to `fetch_klines` (`{data: [{time, open, high, low, close, volume, amount}]}`), but `time` is full ISO with minute precision: `"2026-07-02T09:31:00+08:00"`.

- [ ] **Step 1: Write 3 failing wiremock tests**

Insert in `src/sources/openstock_client.rs` `#[cfg(test)] mod tests` block, after the existing `fetch_klines_propagates_4xx` test:

```rust
    // fetch_minute_klines tests (wiremock-based, P0.13b-1 Task 2)
    #[tokio::test]
    async fn fetch_minute_klines_1m_none_sends_period_1m_and_date() {
        let server = wiremock::MockServer::start().await;
        let body = serde_json::json!({
            "data": [
                {"time": "2026-07-02T09:31:00+08:00", "open": 10.0, "high": 10.5, "low": 9.9, "close": 10.2, "volume": 1000.0, "amount": 10200.0},
                {"time": "2026-07-02T09:32:00+08:00", "open": 10.2, "high": 10.4, "low": 10.1, "close": 10.3, "volume": 800.0, "amount": 8240.0},
            ]
        });
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/data/bars"))
            .and(wiremock::matchers::body_partial_json(serde_json::json!({
                "symbol": "sh600000",
                "period": "1m",
                "date": "2026-07-02"
            })))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(body))
            .expect(1)
            .mount(&server)
            .await;

        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");
        let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
        let bars = client
            .fetch_minute_klines("sh600000", MinutePeriod::Minute1, date, AdjustType::None)
            .await
            .expect("fetch_minute_klines ok");

        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].code, "sh600000");
        assert_eq!(
            bars[0].timestamp,
            chrono::NaiveDateTime::parse_from_str("2026-07-02T09:31:00", "%Y-%m-%dT%H:%M:%S").unwrap()
        );
        assert_eq!(bars[0].adjust_type, AdjustType::None);
        assert_eq!(bars[1].volume, 800);
    }

    #[tokio::test]
    async fn fetch_minute_klines_5m_qfq_sends_adjust_and_stamps_records() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/data/bars"))
            .and(wiremock::matchers::body_partial_json(serde_json::json!({
                "symbol": "sh600000",
                "period": "5m",
                "date": "2026-07-02",
                "adjust": "qfq"
            })))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"time": "2026-07-02T09:35:00+08:00", "open": 11.0, "high": 11.2, "low": 10.9, "close": 11.1, "volume": 500.0, "amount": 5550.0}
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");
        let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
        let bars = client
            .fetch_minute_klines("sh600000", MinutePeriod::Minute5, date, AdjustType::QFQ)
            .await
            .expect("fetch_minute_klines ok");

        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0].adjust_type, AdjustType::QFQ);
    }

    #[tokio::test]
    async fn fetch_minute_klines_propagates_4xx() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/data/bars"))
            .respond_with(wiremock::ResponseTemplate::new(400).set_body_string("bad period"))
            .expect(1) // no retry on 4xx — matches fetch_klines
            .mount(&server)
            .await;

        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");
        let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
        let result = client
            .fetch_minute_klines("sh600000", MinutePeriod::Minute15, date, AdjustType::None)
            .await;

        let err = result.expect_err("expected error on 400");
        let msg = format!("{:?}", err);
        assert!(
            msg.contains("/data/bars returned 400"),
            "expected '/data/bars returned 400' in error, got: {}",
            msg
        );
    }
```

**Note for implementer**: The `fast_test_cfg` helper is defined at `src/sources/openstock_client.rs:789` in the existing `#[cfg(test)] mod tests` block. The standard pattern (used at L811, L841, L867, L1093, L1134, L1182) is `OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build")`. Imports (`MinutePeriod`, `AdjustType`, `OpenStockClient`, `wiremock`) are already in scope in the test module.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib --package quantix-cli sources::openstock_client::tests::fetch_minute_klines`
Expected: FAIL with "no method named `fetch_minute_klines` found" (compile error).

- [ ] **Step 3: Implement `fetch_minute_klines` method**

Insert in `src/sources/openstock_client.rs` after the existing `fetch_klines` method's closing `}` (around L675, before the `impl` block closing `}`):

```rust
    /// Fetches minute-level OHLCV candles from `/data/bars` with `period`
    /// in `1m|5m|15m|30m|60m`. Mirrors `fetch_klines` shape (direct reqwest,
    /// no envelope, no retry, no circuit breaker). Returns `Vec<MinuteBar>`
    /// with `NaiveDateTime` timestamps (minute precision preserved from
    /// the wire ISO string).
    ///
    /// The returned `MinuteBar` records are stamped with the requested
    /// `adjust_type` (the runtime does not echo it back — decision D2
    /// request-driven, matching `fetch_klines`).
    ///
    /// `AdjustType::None` causes the `adjust` field to be omitted entirely
    /// from the request body (matches `fetch_klines` wire shape).
    ///
    /// `date` is sent as `"date": "YYYY-MM-DD"` (single-day scope per spec §8
    /// P0.13b-1; multi-day range query is a P0.13c concern).
    pub async fn fetch_minute_klines(
        &self,
        code: &str,
        period: crate::data::models::MinutePeriod,
        date: chrono::NaiveDate,
        adjust: crate::data::models::AdjustType,
    ) -> Result<Vec<crate::data::models::MinuteBar>> {
        use std::str::FromStr;

        let endpoint = self
            .base_url
            .join("data/bars")
            .map_err(|e| QuantixError::Other(format!("url join failed: {}", e)))?;

        let mut body = serde_json::json!({
            "symbol": code,
            "period": period.as_str(),
            "date": date.format("%Y-%m-%d").to_string(),
        });
        if let Some(adj) = adjust.as_openstock_param() {
            body["adjust"] = serde_json::Value::String(adj.to_string());
        }

        let resp = self
            .http
            .post(endpoint)
            .header("X-API-Key", self.api_key.clone())
            .json(&body)
            .send()
            .await
            .map_err(|e| QuantixError::Network(format!("/data/bars request failed: {}", e)))?;

        let status = resp.status();
        let raw = resp
            .text()
            .await
            .map_err(|e| QuantixError::Network(format!("/data/bars body read failed: {}", e)))?;

        if !status.is_success() {
            return Err(QuantixError::Other(format!(
                "/data/bars returned {}: {}",
                status,
                raw.chars().take(200).collect::<String>()
            )));
        }

        #[derive(serde::Deserialize)]
        struct BarsResponse {
            data: Vec<MinuteBarRecord>,
        }

        #[derive(serde::Deserialize)]
        struct MinuteBarRecord {
            time: String,
            open: f64,
            high: f64,
            low: f64,
            close: f64,
            volume: f64,
            amount: f64,
        }

        let bars: BarsResponse = serde_json::from_str(&raw)
            .map_err(|e| QuantixError::Other(format!("/data/bars parse failed: {}", e)))?;

        let mut out = Vec::with_capacity(bars.data.len());
        for bar in bars.data {
            // Wire time format: "2026-07-02T09:31:00+08:00" → take first 19 chars
            // "2026-07-02T09:31:00" → parse as NaiveDateTime (no timezone).
            let ts = chrono::NaiveDateTime::parse_from_str(&bar.time[..19], "%Y-%m-%dT%H:%M:%S")
                .map_err(|e| {
                    QuantixError::DataParse(format!("解析 minute bars 时间戳失败: {}", e))
                })?;

            out.push(crate::data::models::MinuteBar {
                code: code.to_string(),
                timestamp: ts,
                open: rust_decimal::Decimal::from_str(&format!("{}", bar.open))
                    .unwrap_or_default(),
                high: rust_decimal::Decimal::from_str(&format!("{}", bar.high))
                    .unwrap_or_default(),
                low: rust_decimal::Decimal::from_str(&format!("{}", bar.low))
                    .unwrap_or_default(),
                close: rust_decimal::Decimal::from_str(&format!("{}", bar.close))
                    .unwrap_or_default(),
                volume: bar.volume as i64,
                amount: Some(
                    rust_decimal::Decimal::from_str(&format!("{}", bar.amount))
                        .unwrap_or_default(),
                ),
                adjust_type: adjust,
            });
        }

        Ok(out)
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib --package quantix-cli sources::openstock_client::tests::fetch_minute_klines`
Expected: PASS (3 tests).

- [ ] **Step 5: Run clippy + fmt**

Run: `cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -5`
Expected: clean.

Run: `cargo fmt --all -- --check`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add src/sources/openstock_client.rs
git commit -m "feat(sources): add fetch_minute_klines client method for P0.13b-1

Direct /data/bars reqwest (no envelope/retry/breaker, matching fetch_klines).
Parses ISO timestamp to NaiveDateTime (vs fetch_klines' NaiveDate) to preserve
minute precision. Stamps each MinuteBar with requested adjust_type.
3 wiremock tests: 1m+none, 5m+qfq, 15m+4xx (no-retry via .expect(1)).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 3: CLI `FetchMinuteKlines` subcommand + handler + dispatcher

**Files:**
- Modify: `src/cli/commands/data.rs:176-385` (add variant to `OpenStockCommands` enum, after `FetchKlines`)
- Modify: `src/cli/handlers/openstock_handler.rs` (add `fetch_openstock_minute_klines` async fn)
- Modify: `src/cli/handlers/mod.rs:130` (re-export)
- Modify: `src/cli/handlers/app_shell.rs:367-385` (add dispatcher arm)

**Interfaces:**
- Consumes: `MinutePeriod::from_str`, `AdjustType::from_str`, `NaiveDate::parse_from_str`, `OpenStockSettings`, `OpenStockClient::from_settings`, `fetch_minute_klines` (from Task 2).
- Produces: CLI subcommand `data openstock fetch-minute-klines --symbol <s> --period <p> --date <d> --adjust <a>`.

- [ ] **Step 1: Add `FetchMinuteKlines` variant to `OpenStockCommands` enum**

Insert in `src/cli/commands/data.rs` after the existing `FetchKlines { ... }` variant (around L363-385):

```rust
    /// 拉取分钟级 K 线蜡烛 (P0.13b-1, OpenStock /data/bars period=1m|5m|15m|30m|60m)
    FetchMinuteKlines {
        #[arg(long)] symbol: String,
        #[arg(long, default_value = "1m")] period: String,
        #[arg(long)] date: String,
        #[arg(long, default_value = "none")] adjust: String,
    },
```

- [ ] **Step 2: Add `fetch_openstock_minute_klines` handler**

Insert in `src/cli/handlers/openstock_handler.rs` (after the existing `fetch_openstock_klines` function):

```rust
pub(crate) async fn fetch_openstock_minute_klines(
    settings: &OpenStockSettings,
    symbol: String,
    period: String,
    date: String,
    adjust: String,
) -> Result<()> {
    use std::str::FromStr;

    use crate::data::models::{AdjustType, MinutePeriod};
    use crate::sources::openstock_client::OpenStockClient;

    let period_enum = MinutePeriod::from_str(&period)
        .map_err(|e| QuantixError::Config(format!("--period: {}", e)))?;
    let adjust_enum = AdjustType::from_str(&adjust)
        .map_err(|e| QuantixError::Config(format!("--adjust: {}", e)))?;
    let date_parsed = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| QuantixError::Config(format!("--date: {}", e)))?;

    let client = OpenStockClient::from_settings(settings)?;
    let bars = client
        .fetch_minute_klines(&symbol, period_enum, date_parsed, adjust_enum)
        .await?;

    println!(
        "OpenStock live fetch (/data/bars, symbol={}, minute={})",
        symbol,
        period_enum.as_str()
    );
    println!("  Date:   {}", date);
    println!(
        "  Adjust: {}",
        adjust_enum
            .as_openstock_param()
            .unwrap_or("none (field omitted)")
    );
    println!("  记录数: {}", bars.len());
    if !bars.is_empty() {
        println!("  First:  {:?}", bars.first());
        println!("  Last:   {:?}", bars.last());
    }
    println!("  Source:        (not reported by /data/bars)");
    println!("  artifact_hash: (not reported by /data/bars)");
    println!("  latency_ms:    (not reported by /data/bars)");
    Ok(())
}
```

**Note for implementer**: `OpenStockSettings` is already imported at `openstock_handler.rs:4` (verified). The inline `use` block intentionally omits it (only `MinutePeriod`, `AdjustType`, `OpenStockClient` need inline imports for this function). Run `cargo check` after Step 2 to confirm no missing/extra imports.

- [ ] **Step 3: Re-export handler in `src/cli/handlers/mod.rs`**

Modify line 130 from:
```rust
    fetch_openstock_index, fetch_openstock_klines, fetch_openstock_workdays,
```
to:
```rust
    fetch_openstock_index, fetch_openstock_klines, fetch_openstock_minute_klines,
    fetch_openstock_workdays,
```

- [ ] **Step 4: Add dispatcher arm in `src/cli/handlers/app_shell.rs`**

After the existing `OpenStockCommands::FetchKlines { ... } => { ... }` arm (around L367-385), add:

```rust
            OpenStockCommands::FetchMinuteKlines {
                symbol,
                period,
                date,
                adjust,
            } => {
                fetch_openstock_minute_klines(
                    &rt.openstock,
                    symbol,
                    period,
                    date,
                    adjust,
                )
                .await?;
            }
```

- [ ] **Step 5: Run `cargo check` to verify compile**

Run: `cargo check --workspace 2>&1 | tail -20`
Expected: clean (no errors, no warnings).

- [ ] **Step 6: Run CLI help to verify subcommand registered**

Run: `cargo run -q -- data openstock --help 2>&1 | tail -30`
Expected: output contains `fetch-minute-klines`.

- [ ] **Step 7: Run full test suite (regression)**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: all tests pass (including P0.13a's — no regression).

- [ ] **Step 8: Run clippy + fmt**

Run: `cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -5`
Expected: clean.

Run: `cargo fmt --all -- --check`
Expected: clean.

- [ ] **Step 9: Commit**

```bash
git add src/cli/commands/data.rs src/cli/handlers/openstock_handler.rs src/cli/handlers/mod.rs src/cli/handlers/app_shell.rs
git commit -m "feat(cli): add fetch-minute-klines CLI subcommand for P0.13b-1

Wires MinutePeriod + AdjustType via strict FromStr (fail-fast on bad input).
Handler output mirrors fetch-klines shape. Period/Adjust enum parsing happens
in the handler (not the client) to surface user-friendly --period: errors.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 4: Live tests + OpenSpec change + governance card + archive

**Files:**
- Create: `tests/openstock_live_minute_klines.rs`
- Create: `openspec/changes/openstock-data-consumption-p0-13b-1/proposal.md`
- Create: `openspec/changes/openstock-data-consumption-p0-13b-1/tasks.md`
- Create: `openspec/changes/openstock-data-consumption-p0-13b-1/design.md`
- Create: `openspec/changes/openstock-data-consumption-p0-13b-1/specs/openstock-data-consumption/spec.md`
- Create: `.governance/programs/project-governance/cards/P0.13b-1.yaml`

- [ ] **Step 1: Write 3 live `#[ignore]` tests**

Create `tests/openstock_live_minute_klines.rs`:

```rust
//! Live OpenStock /data/bars minute-candle integration tests (P0.13b-1).
//!
//! These tests hit the real OpenStock runtime and are `#[ignore]`-gated by
//! default. To run them locally:
//!
//! ```sh
//! QUANTIX_OPENSTOCK_LIVE=1 \
//! OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
//! OPENSTOCK_API_KEY=<key> \
//! cargo test --test openstock_live_minute_klines -- --ignored
//! ```

use std::str::FromStr;

use quantix_cli::core::runtime::OpenStockSettings;
use quantix_cli::data::models::{AdjustType, MinutePeriod};
use quantix_cli::sources::openstock_client::OpenStockClient;

fn settings_from_env() -> Option<OpenStockSettings> {
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
async fn fetch_minute_klines_live_1m_none() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let settings = settings_from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let client = OpenStockClient::from_settings(&settings).expect("client ok");
    let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
    let bars = client
        .fetch_minute_klines("sh600000", MinutePeriod::Minute1, date, AdjustType::None)
        .await
        .expect("fetch ok");
    assert!(!bars.is_empty(), "expected non-empty 1m bars");
    println!("1m+none bars: {}", bars.len());
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_minute_klines_live_5m_qfq() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let settings = settings_from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let client = OpenStockClient::from_settings(&settings).expect("client ok");
    let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
    let bars = client
        .fetch_minute_klines("sh600000", MinutePeriod::Minute5, date, AdjustType::QFQ)
        .await
        .expect("fetch ok");
    assert!(!bars.is_empty(), "expected non-empty 5m qfq bars");
    assert_eq!(bars[0].adjust_type, AdjustType::QFQ);
    println!("5m+qfq bars: {}", bars.len());
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_minute_klines_live_60m_hfq() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let settings = settings_from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let client = OpenStockClient::from_settings(&settings).expect("client ok");
    let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
    let bars = client
        .fetch_minute_klines("sh600000", MinutePeriod::Minute60, date, AdjustType::HFQ)
        .await
        .expect("fetch ok");
    assert!(!bars.is_empty(), "expected non-empty 60m hfq bars");
    assert_eq!(bars[0].adjust_type, AdjustType::HFQ);
    println!("60m+hfq bars: {}", bars.len());
}
```

- [ ] **Step 2: Verify the live tests are discovered and ignored**

Run: `cargo test --test openstock_live_minute_klines 2>&1 | tail -10`
Expected: 3 tests shown as ignored (not run).

- [ ] **Step 3: Create OpenSpec change proposal**

Create `openspec/changes/openstock-data-consumption-p0-13b-1/proposal.md`:

```markdown
# OpenStock Data Consumption P0.13b-1

## Why

P0.13a delivered day/week/month K-line fetching via `/data/bars`. The
HANDOFF report row 35 (corrected in this slice) tags minute-level K-line
candles as the next priority for short-timeframe signal / backtest
workloads. OpenStock's `_PERIOD_MAP` accepts `1m|5m|15m|30m|60m` on the
same `/data/bars` endpoint, so this slice is purely additive client
wiring — no server-side changes.

## What Changes

- Add `MinutePeriod` enum (strict 1m|5m|15m|30m|60m FromStr — rejects
  all aliases like `1min`/`minute`/`1h` to defend against OpenStock
  `_PERIOD_MAP` silent day-fallback for unknown tokens).
- Add `MinuteBar` struct (NaiveDateTime timestamp — distinct from P0.13a
  `Kline`'s NaiveDate; named `MinuteBar` not `MinuteKline` to avoid
  collision with `src/db/tdengine.rs:37` existing `MinuteKline` f64 type).
- Add `OpenStockClient::fetch_minute_klines(code, period, date, adjust)`
  returning `Vec<MinuteBar>` via direct `/data/bars` reqwest (no envelope,
  no retry, no breaker — matching `fetch_klines`).
- Add CLI `data openstock fetch-minute-klines` subcommand.

## Impact

- New files: `tests/openstock_live_minute_klines.rs`.
- Modified files: `src/data/models.rs`, `src/sources/openstock_client.rs`,
  `src/cli/commands/data.rs`, `src/cli/handlers/openstock_handler.rs`,
  `src/cli/handlers/mod.rs`, `src/cli/handlers/app_shell.rs`.
- No DB writes, no persistence — read-only consumption.
- No regression to P0.13a's `BarPeriod`/`Kline`/`fetch_klines`.

## Non-Goals

- Time-share point series via `/data/fetch MINUTE_DATA` (deferred to P0.13b-2).
- Multi-day range queries (single `date` param only; range is P0.13c).
- ClickHouse writes / shadow persistence (read-only).
- Retry / circuit breaker on `/data/bars` (matches `fetch_klines` P0.13a decision).
```

- [ ] **Step 4: Create OpenSpec tasks.md**

Create `openspec/changes/openstock-data-consumption-p0-13b-1/tasks.md`:

```markdown
# OpenStock Data Consumption P0.13b-1 — Tasks

## 1. Baseline And Governance
- [ ] Create `.governance/programs/project-governance/cards/P0.13b-1.yaml`
- [ ] Confirm clean working tree (P0.13a merged, R1 spec revisions committed)

## 2. Data Models (`src/data/models.rs`)
- [ ] Add `MinutePeriod` enum (Minute1/5/15/30/60) with `as_str()` + strict `FromStr`
- [ ] Add `MinuteBar` struct with `NaiveDateTime` timestamp
- [ ] Add 3 unit tests (as_str round trip, canonical accept, alias reject)

## 3. Client Method (`src/sources/openstock_client.rs`)
- [ ] Add `fetch_minute_klines(code, period, date, adjust) -> Vec<MinuteBar>`
- [ ] 3 wiremock tests (1m+none, 5m+qfq, 15m+4xx-no-retry)

## 4. CLI Wiring
- [ ] Add `FetchMinuteKlines` variant to `OpenStockCommands` (`src/cli/commands/data.rs`)
- [ ] Add `fetch_openstock_minute_klines` handler (`src/cli/handlers/openstock_handler.rs`)
- [ ] Re-export in `src/cli/handlers/mod.rs`
- [ ] Add dispatcher arm (`src/cli/handlers/app_shell.rs`)

## 5. Live Tests
- [ ] Create `tests/openstock_live_minute_klines.rs` with 3 `#[ignore]` tests

## 6. Quality Gates
- [ ] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --workspace`
- [ ] `openspec validate openstock-data-consumption-p0-13b-1 --strict`

## 7. HANDOFF Report Correction
- [ ] Update `docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md`
      row 35: correct `MINUTE_DATA` mislabel (clarify minute candles go via KLINES/`/data/bars`,
      not MINUTE_DATA). Mark B-group minute-candles row as ✅ P0.13b-1.

## 8. Archive
- [ ] `openspec archive openstock-data-consumption-p0-13b-1` (after merge)
- [ ] Governance: mark P0.13b-1 card state as `completed`
```

- [ ] **Step 5: Create OpenSpec design.md (reference to spec)**

Create `openspec/changes/openstock-data-consumption-p0-13b-1/design.md`:

```markdown
# OpenStock P0.13b-1 Design

**Canonical spec:** `docs/superpowers/specs/2026-07-02-openstock-p0-13b-minute-level-data-design.md` (R1 revisions applied per `docs/superpowers/specs/2026-07-02-openstock-p0-13b-design-review.md`).

This OpenSpec change implements only the P0.13b-1 sub-slice (minute candles
via `/data/bars`). P0.13b-2 (time-share via `/data/fetch MINUTE_DATA`) is a
separate OpenSpec change to be opened after P0.13b-1 archives.

## Key Decisions (subset of spec D1-D8)

- **D3**: New `MinutePeriod` enum (not extending P0.13a `BarPeriod`);
  new `MinuteBar` struct (not reusing `Kline`, not colliding with
  `src/db/tdengine.rs:37` `MinuteKline`).
- **D4**: Strict `FromStr` whitelist — only `1m|5m|15m|30m|60m`.
  Defends against OpenStock `_PERIOD_MAP.get(period, "day")` silent
  day-fallback for unknown tokens.
- **D5**: `MinuteBar.timestamp: NaiveDateTime` (vs `Kline.date: NaiveDate`)
  to preserve minute-level precision.
- **Reuse**: Same `/data/bars` endpoint as P0.13a `fetch_klines`; same
  direct-reqwest pattern (no envelope/retry/breaker).

## Risks

- **R1** (silent day-fallback) — mitigated by D4 strict whitelist.
- **R2** (time field format) — verified via wiremock tests using ISO format
  `"2026-07-02T09:31:00+08:00"`; live tests confirm real wire shape.
```

- [ ] **Step 6: Create OpenSpec spec.md (ADDED Requirements)**

Create `openspec/changes/openstock-data-consumption-p0-13b-1/specs/openstock-data-consumption/spec.md`:

```markdown
# OpenStock Data Consumption Spec Delta — P0.13b-1

## ADDED Requirements

### Requirement: Minute-level K-line candles via /data/bars

The system SHALL provide a `fetch_minute_klines(code, period, date, adjust)`
method on `OpenStockClient` that fetches OHLCV candles at minute granularity
(1m|5m|15m|30m|60m) via POST to `/data/bars` with JSON body
`{symbol, period, date, adjust?}`.

#### Scenario: Strict period whitelist

- **WHEN** `MinutePeriod::from_str("1min")` is called
- **THEN** the result SHALL be `Err(QuantixError::Config)` with a message
  listing `1m|5m|15m|30m|60m` as the only accepted tokens

#### Scenario: Adjust field omission on None

- **WHEN** `fetch_minute_klines(code, period, date, AdjustType::None)` is called
- **THEN** the request body SHALL NOT contain the `adjust` key

#### Scenario: 4xx propagation without retry

- **WHEN** `/data/bars` returns HTTP 400
- **THEN** the method SHALL return `Err(QuantixError::Other)` containing
  "/data/bars returned 400" on the first attempt, without retrying

#### Scenario: Minute-precision timestamp preserved

- **WHEN** the wire response contains `"time": "2026-07-02T09:31:00+08:00"`
- **THEN** the returned `MinuteBar.timestamp` SHALL equal the parsed
  `NaiveDateTime` for `2026-07-02T09:31:00`

### Requirement: fetch-minute-klines CLI subcommand

The system SHALL provide a `data openstock fetch-minute-klines` subcommand
accepting `--symbol`, `--period` (default `1m`), `--date` (required,
`YYYY-MM-DD`), and `--adjust` (default `none`).

#### Scenario: Bad --period surfaces as Config error

- **WHEN** the user runs `data openstock fetch-minute-klines --symbol sh600000 --period 1min --date 2026-07-02`
- **THEN** the CLI SHALL exit with a `QuantixError::Config` whose message
  contains "--period:" and "expected one of 1m|5m|15m|30m|60m"
```

- [ ] **Step 7: Create governance card**

Create `.governance/programs/project-governance/cards/P0.13b-1.yaml`:

```yaml
id: P0.13b-1
title: "OpenStock minute-level K-line candles (KLINES minute periods)"
state: in_progress
scope:
  allowed_paths:
    - src/data/models.rs
    - src/sources/openstock_client.rs
    - src/cli/commands/data.rs
    - src/cli/handlers/openstock_handler.rs
    - src/cli/handlers/mod.rs
    - src/cli/handlers/app_shell.rs
    - tests/openstock_live_minute_klines.rs
    - openspec/changes/openstock-data-consumption-p0-13b-1/**
    - docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md
  forbidden_paths:
    - src/db/**
    - src/backtest/**
    - src/execution/**
    - src/sources/openstock.rs           # shadow persistence (legacy path)
    - src/sources/openstock_shadow.rs   # shadow persistence (legacy path)
    - src/sources/kline_aggregator.rs   # different semantic domain (P0.13a D6)
    - src/sources/openstock_client.rs::fetch_klines   # P0.13a — do not modify
    - src/data/models.rs::Kline         # P0.13a — do not modify
    - src/data/models.rs::BarPeriod     # P0.13a — do not modify
acceptance_gates:
  - cargo fmt --all -- --check
  - cargo clippy --all-targets --workspace -- -D warnings
  - cargo test --workspace
  - openspec validate openstock-data-consumption-p0-13b-1 --strict
  - openspec validate --all --strict
non_goals:
  - "Time-share MINUTE_DATA via /data/fetch (P0.13b-2)"
  - "Multi-day range queries (P0.13c)"
  - "ClickHouse writes / shadow persistence"
```

- [ ] **Step 8: Validate OpenSpec**

Run: `openspec validate openstock-data-consumption-p0-13b-1 --strict`
Expected: PASS.

Run: `openspec validate --all --strict`
Expected: PASS.

- [ ] **Step 9: Run all quality gates**

Run: `cargo fmt --all -- --check && cargo clippy --all-targets --workspace -- -D warnings && cargo test --workspace 2>&1 | tail -10`
Expected: all pass.

- [ ] **Step 10: Commit**

```bash
git add tests/openstock_live_minute_klines.rs openspec/changes/openstock-data-consumption-p0-13b-1/ .governance/programs/project-governance/cards/P0.13b-1.yaml
git commit -m "docs(p0-13b-1): add live tests, OpenSpec change, governance card

3 #[ignore] live tests gated by QUANTIX_OPENSTOCK_LIVE=1.
OpenSpec change: proposal/tasks/design/spec with strict period whitelist
+ 4xx-no-retry + adjust omission invariants as scenarios.
Governance card: scope forbids touching P0.13a Kline/BarPeriod/fetch_klines.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Verification

```bash
# Quality gates (run before each commit)
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --lib --package quantix-cli data::models::tests::minute_period
cargo test --lib --package quantix-cli sources::openstock_client::tests::fetch_minute_klines
cargo test --workspace                                      # regression incl. new live tests (skipped)

# Manual live smoke (only when OpenStock runtime is reachable)
QUANTIX_OPENSTOCK_LIVE=1 \
OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
OPENSTOCK_API_KEY=<key> \
cargo test --test openstock_live_minute_klines -- --ignored

# CLI smoke (when live)
OPENSTOCK_BASE_URL=http://192.168.123.104:8040 OPENSTOCK_API_KEY=<key> \
cargo run -q -- data openstock fetch-minute-klines --symbol sh600000 --period 1m --date 2026-07-02

# Spec + governance
openspec validate openstock-data-consumption-p0-13b-1 --strict
openspec validate --all --strict
gitnexus detect_changes                                     # expect LOW on client + handlers
git diff --check
```

## Critical Files

- `/opt/claude/quantix-rust/src/data/models.rs:39-104` — `MinutePeriod` + `MinuteBar` insertion point
- `/opt/claude/quantix-rust/src/sources/openstock_client.rs:581-675` — `fetch_klines` (pattern reference); insertion point L675
- `/opt/claude/quantix-rust/src/sources/openstock_client.rs:1083-1200` — P0.13a wiremock tests (pattern reference)
- `/opt/claude/quantix-rust/src/cli/commands/data.rs:363-385` — `FetchKlines` (pattern reference); insert `FetchMinuteKlines` after
- `/opt/claude/quantix-rust/src/cli/handlers/app_shell.rs:367-385` — `FetchKlines` dispatcher arm (pattern reference)
- `/opt/claude/quantix-rust/src/cli/handlers/mod.rs:130` — re-export line
- `/opt/claude/quantix-rust/src/core/runtime/settings.rs:74` — `OpenStockSettings` (real type, not `OpenStockClientSettings`)
- `/opt/claude/quantix-rust/tests/openstock_live_klines.rs` — `#[ignore]` + env gate pattern reference
- `/opt/claude/quantix-rust/docs/superpowers/specs/2026-07-02-openstock-p0-13b-minute-level-data-design.md` — canonical spec
- `/opt/claude/quantix-rust/docs/superpowers/specs/2026-07-02-openstock-p0-13b-design-review.md` — R1 review revisions
- `/opt/claude/openstock/openstock/adapters/_eltdx_timeseries.py:12-32` — `_PERIOD_MAP` (wire contract authority)
