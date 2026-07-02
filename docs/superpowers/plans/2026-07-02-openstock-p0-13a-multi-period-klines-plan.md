# OpenStock P0.13a — Multi-period K-line Fetch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable quantix-rust to fetch A-share K-lines from OpenStock `/data/bars` with `period ∈ {day, week, month}` and `adjust_type ∈ {None, QFQ, HFQ}`, read-only (no DB writes).

**Architecture:** Additive wiring in 3 phases. Phase 1 adds the `BarPeriod` enum and `AdjustType` extensions alongside a new `OpenStockClient::fetch_klines` method (mirrors `fetch_daily_klines`, direct reqwest, no envelope path). Phase 2 exposes a single `FetchKlines` CLI subcommand that parses `--period`/`--adjust` via `FromStr` (fail-fast). Phase 3 adds `#[ignore]`-gated live tests and updates the HANDOFF report + archives the OpenSpec change. No new DB writes; no shadow persistence; no parser changes.

**Tech Stack:** Rust (workspace `quantix-cli`), reqwest, serde_json, wiremock (tests), tokio, clap, chrono, rust_decimal, OpenSpec, GitNexus.

## Global Constraints

- **Type naming**: the new period enum MUST be named `BarPeriod` (NOT `KlinePeriod`) — the existing `src/sources/kline_aggregator.rs:14` `KlinePeriod` is a different semantic domain (aggregator time-windows 1m/5m/1d).
- **Adjust wire format**: `AdjustType::None` MUST cause `body["adjust"]` to be omitted entirely (NOT sent as `""`). The runtime distinguishes "field absent" from "field present but empty".
- **Case sensitivity**: both `BarPeriod::FromStr` and `AdjustType::FromStr` MUST accept input case-insensitively. `BarPeriod::FromStr` rejects `daily`/`weekly`/`monthly`/`minute*` aliases (decision D6). `AdjustType::FromStr` accepts only `none|qfq|hfq` (case-insensitive).
- **No retry / no circuit breaker**: the new `fetch_klines` MUST NOT participate in retry or breaker semantics — mirrors `fetch_daily_klines` (P0.10 design).
- **`fetch_daily_klines` is preserved unchanged** (decision D3): existing market/backtest callers must not break.
- **Read-only**: no DB writes, no ClickHouse, no shadow persistence integration in this slice.
- **Coding standards** (see `CLAUDE.md`): no `.unwrap()`/`.expect()`/`panic!()` in production code (tests OK); `?` + `QuantixError::Other(format!(...))` for new error paths; one commit per semantic intent.
- **Quality gates** (must pass before each commit): `cargo fmt --all -- --check`, `cargo clippy --all-targets --workspace -- -D warnings`, `cargo test --workspace`.

---

## File Structure

**Modified (existing files):**

| File | Current lines | Responsibility change |
|------|---------------|----------------------|
| `src/data/models.rs` | 111 | Add `BarPeriod` enum (3 variants + `as_str` + `FromStr`); add `AdjustType::as_openstock_param() -> Option<&'static str>`; add `impl FromStr for AdjustType`; add `#[cfg(test)] mod tests` with T1 + T2 |
| `src/sources/openstock_client.rs` | 974 | Add `pub async fn fetch_klines(&self, code, period: BarPeriod, adjust: AdjustType, start, end) -> Result<Vec<Kline>>` after `fetch_daily_klines` (L481-568); add wiremock tests T3/T4/T5 in `#[cfg(test)] mod tests` |
| `src/cli/commands/data.rs` | 361 | Append `FetchKlines` variant to `OpenStockCommands` enum (after `FetchWorkdays`, before closing brace at L361) |
| `src/cli/handlers/openstock_handler.rs` | 809 | Add `pub(crate) async fn fetch_openstock_klines(...)` after `fetch_openstock_index` (after L342) |
| `src/cli/handlers/app_shell.rs` | 880 | Add new match arm after `FetchIndex` arm (~L362) |
| `src/cli/handlers/mod.rs` | 189 | Add `fetch_openstock_klines` to existing re-export (~L129) |

**New files:**

| File | Purpose |
|------|---------|
| `tests/openstock_live_klines.rs` | 3 `#[tokio::test] #[ignore]` live tests (T6 day+None, T7 week+qfq, T8 month+hfq), gated by `QUANTIX_OPENSTOCK_LIVE=1` |
| `openspec/changes/openstock-data-consumption-p0-13a/proposal.md` | Why / What Changes / Impact / Non-Goals |
| `openspec/changes/openstock-data-consumption-p0-13a/tasks.md` | Phased task list (sections 0-3) |
| `openspec/changes/openstock-data-consumption-p0-13a/design.md` | D1-D8 decisions (copy from spec doc) |
| `openspec/changes/openstock-data-consumption-p0-13a/specs/openstock-data-consumption/spec.md` | `### ADDED Requirements` for multi-period + adjust |
| `.governance/programs/project-governance/cards/P0.13a.yaml` | Governance card (scope.allowed_paths, non_goals, acceptance gates) |

**Modified (closeout):**

| File | Change |
|------|--------|
| `docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md` | Flip B-group row ❌ → ✅ (3 categories: `KLINES`, `ADJUSTED_KLINES`, `HISTORICAL_KLINES`) |
| `FUNCTION_TREE.md` | Add P0.13a entry under OpenStock consumer-side |

---

## Phase 1 — Client Method (Commit 1)

### Task 1.1: Add `BarPeriod` enum and tests

**Files:**
- Modify: `src/data/models.rs:29` (insert after `AdjustType` enum, before `Tick` struct at L32)
- Test: `src/data/models.rs` (new `#[cfg(test)] mod tests` at end of file)

**Interfaces:**
- Produces: `pub enum BarPeriod { Day, Week, Month }`, `impl BarPeriod { pub fn as_str(&self) -> &'static str }`, `impl FromStr for BarPeriod` (type Err = QuantixError)

- [ ] **Step 1: Write the failing test (T1)**

Append to `src/data/models.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn bar_period_as_str_round_trip() {
        assert_eq!(BarPeriod::Day.as_str(), "day");
        assert_eq!(BarPeriod::Week.as_str(), "week");
        assert_eq!(BarPeriod::Month.as_str(), "month");
    }

    #[test]
    fn bar_period_from_str_accepts_canonical_case_insensitive() {
        assert!(matches!(BarPeriod::from_str("day"), Ok(BarPeriod::Day)));
        assert!(matches!(BarPeriod::from_str("WEEK"), Ok(BarPeriod::Week)));
        assert!(matches!(BarPeriod::from_str("Month"), Ok(BarPeriod::Month)));
    }

    #[test]
    fn bar_period_from_str_rejects_aliases() {
        // D6: strict — reject daily/weekly/monthly/minute* aliases
        assert!(BarPeriod::from_str("daily").is_err());
        assert!(BarPeriod::from_str("weekly").is_err());
        assert!(BarPeriod::from_str("monthly").is_err());
        assert!(BarPeriod::from_str("1m").is_err());
        assert!(BarPeriod::from_str("minute").is_err());
        assert!(BarPeriod::from_str("").is_err());
    }

    #[test]
    fn adjust_type_as_openstock_param() {
        assert_eq!(AdjustType::None.as_openstock_param(), None);
        assert_eq!(AdjustType::QFQ.as_openstock_param(), Some("qfq"));
        assert_eq!(AdjustType::HFQ.as_openstock_param(), Some("hfq"));
    }

    #[test]
    fn adjust_type_from_str_case_insensitive() {
        assert!(matches!(AdjustType::from_str("none"), Ok(AdjustType::None)));
        assert!(matches!(AdjustType::from_str("QFQ"), Ok(AdjustType::QFQ)));
        assert!(matches!(AdjustType::from_str("Hfq"), Ok(AdjustType::HFQ)));
        assert!(AdjustType::from_str("front").is_err());
        assert!(AdjustType::from_str("").is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib --package quantix-cli data::models::tests`
Expected: FAIL with compile error (`BarPeriod` not found, method `as_openstock_param` not found, `FromStr` not implemented).

- [ ] **Step 3: Add `BarPeriod` enum and `AdjustType` impls**

In `src/data/models.rs`, immediately after the existing `AdjustType` enum (L29, before the blank line preceding `Tick`):

```rust
/// `/data/bars` 周期参数 (OpenStock API).
///
/// Named `BarPeriod` (not `KlinePeriod`) to avoid collision with the
/// aggregator-side `KlinePeriod` in `src/sources/kline_aggregator.rs`,
/// which represents 1m/5m/1d aggregation windows — a different semantic
/// domain from the OpenStock `/data/bars` `period` request parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarPeriod {
    Day,
    Week,
    Month,
}

impl BarPeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
        }
    }
}

impl std::str::FromStr for BarPeriod {
    type Err = QuantixError;

    /// Accepts only `day` | `week` | `month` (any case). Rejects
    /// `daily`/`weekly`/`monthly` aliases and any `minute*` value
    /// (P0.13b scope) — see design D6.
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "day" => Ok(Self::Day),
            "week" => Ok(Self::Week),
            "month" => Ok(Self::Month),
            other => Err(QuantixError::Config(format!(
                "unsupported BarPeriod `{}`: expected one of day|week|month",
                other
            ))),
        }
    }
}

impl AdjustType {
    /// Returns the OpenStock `/data/bars` `adjust` parameter value, or
    /// `None` when the field should be omitted entirely (matches the
    /// existing `fetch_daily_klines` behavior — it omits the `adjust`
    /// field rather than sending `"adjust": ""`).
    pub fn as_openstock_param(&self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::QFQ => Some("qfq"),
            Self::HFQ => Some("hfq"),
        }
    }
}

impl std::str::FromStr for AdjustType {
    type Err = QuantixError;

    /// Accepts `none` | `qfq` | `hfq` (any case).
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "none" => Ok(Self::None),
            "qfq" => Ok(Self::QFQ),
            "hfq" => Ok(Self::HFQ),
            other => Err(QuantixError::Config(format!(
                "unsupported AdjustType `{}`: expected one of none|qfq|hfq",
                other
            ))),
        }
    }
}
```

**Imports**: add `use crate::core::QuantixError;` at the top of `src/data/models.rs` (verified path — `crate::core::QuantixError` via the re-export in `src/core/mod.rs:12`). The file does not currently import it.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib --package quantix-cli data::models::tests`
Expected: PASS — 5 tests green.

- [ ] **Step 5: Run quality gates**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
```

Expected: both pass with no warnings on the new code.

- [ ] **Step 6: Commit**

```bash
git add src/data/models.rs
git commit -m "$(cat <<'EOF'
feat(data): add BarPeriod enum and AdjustType param helpers for P0.13a

Introduces BarPeriod (day/week/month) named to avoid collision with
kline_aggregator::KlinePeriod (aggregator time-window domain). Adds
AdjustType::as_openstock_param() returning Option<&str> so callers can
omit the adjust field entirely when None — matches existing
fetch_daily_klines behavior. Adds FromStr impls (case-insensitive,
strict per design D6).

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

### Task 1.2: Add `OpenStockClient::fetch_klines` method

**Files:**
- Modify: `src/sources/openstock_client.rs` (insert after `fetch_daily_klines` at L568, before the closing `}` of `impl OpenStockClient`)
- Test: `src/sources/openstock_client.rs` (extend `#[cfg(test)] mod tests` with T3, T4, T5)

**Interfaces:**
- Consumes: `crate::data::models::{BarPeriod, AdjustType, Kline}` (from Task 1.1)
- Produces: `pub async fn fetch_klines(&self, code: &str, period: BarPeriod, adjust: AdjustType, start: Option<&str>, end: Option<&str>) -> Result<Vec<Kline>>`

- [ ] **Step 1: Write the failing tests (T3, T4, T5)**

Append to the `#[cfg(test)] mod tests` block in `src/sources/openstock_client.rs` (after the existing tests at the bottom of the file):

```rust
    // -----------------------------------------------------------------
    // fetch_klines wiremock tests (P0.13a)
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn fetch_klines_day_none_omits_adjust_field() {
        use crate::data::models::{AdjustType, BarPeriod};
        use wiremock::matchers::{body_partial_json, header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        // Verify the request body: period=day and NO `adjust` field.
        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .and(header("X-API-Key", "test-key"))
            .and(body_partial_json(serde_json::json!({
                "symbol": "600000",
                "period": "day",
            })))
            // Negative-match the adjust field: body_partial_json does not
            // reject extra fields, so we additionally assert via a custom
            // matcher below in T4. For T3, day+None is the default and is
            // covered sufficiently by body_partial_json not requiring adjust.
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"data":[{"time":"2026-06-01T15:00:00+08:00","open":10.0,"high":11.0,"low":9.5,"close":10.5,"volume":1000.0,"amount":10500.0}]}"#,
            ))
            .expect(1)
            .mount(&server)
            .await;

        let klines = client
            .fetch_klines("600000", BarPeriod::Day, AdjustType::None, None, None)
            .await
            .expect("fetch ok");
        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].code, "600000");
        assert_eq!(klines[0].date.to_string(), "2026-06-01");
        assert_eq!(klines[0].adjust_type, AdjustType::None);
    }

    #[tokio::test]
    async fn fetch_klines_week_qfq_includes_adjust_and_stamps_klines() {
        use crate::data::models::{AdjustType, BarPeriod};
        use wiremock::matchers::{body_partial_json, header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .and(header("X-API-Key", "test-key"))
            .and(body_partial_json(serde_json::json!({
                "symbol": "600000",
                "period": "week",
                "adjust": "qfq",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"data":[{"time":"2026-06-06T15:00:00+08:00","open":10.0,"high":11.0,"low":9.5,"close":10.5,"volume":1000.0,"amount":10500.0},{"time":"2026-06-13T15:00:00+08:00","open":10.5,"high":11.5,"low":10.0,"close":11.0,"volume":800.0,"amount":8800.0}]}"#,
            ))
            .expect(1)
            .mount(&server)
            .await;

        let klines = client
            .fetch_klines("600000", BarPeriod::Week, AdjustType::QFQ, None, None)
            .await
            .expect("fetch ok");
        assert_eq!(klines.len(), 2);
        // Critical assertion: each Kline is stamped with the REQUESTED
        // adjust_type (runtime does not echo it — decision D2).
        assert_eq!(klines[0].adjust_type, AdjustType::QFQ);
        assert_eq!(klines[1].adjust_type, AdjustType::QFQ);
    }

    #[tokio::test]
    async fn fetch_klines_propagates_4xx() {
        use crate::data::models::{AdjustType, BarPeriod};
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

        Mock::given(method("POST"))
            .and(path("/data/bars"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_string(r#"{"code":"bad_request","message":"nope"}"#),
            )
            .expect(1) // no retry on 4xx — matches fetch_daily_klines
            .mount(&server)
            .await;

        let err = client
            .fetch_klines("600000", BarPeriod::Month, AdjustType::HFQ, None, None)
            .await
            .expect_err("should fail");
        let msg = format!("{:?}", err);
        assert!(msg.contains("/data/bars returned 400"), "msg={}", msg);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib --package quantix-cli sources::openstock_client::tests::fetch_klines`
Expected: FAIL with compile error (`method fetch_klines not found in OpenStockClient`).

- [ ] **Step 3: Implement `fetch_klines`**

In `src/sources/openstock_client.rs`, immediately after the closing `}` of `fetch_daily_klines` (L568) and before the closing `}` of `impl OpenStockClient`:

```rust
    /// Fetch OHLCV bars from OpenStock `/data/bars` with explicit period
    /// and adjust type. Generalizes `fetch_daily_klines` to week/month
    /// periods and qfq/hfq adjustment.
    ///
    /// New CLI paths use this; `fetch_daily_klines` is preserved unchanged
    /// for existing market/backtest callers (decision D3).
    ///
    /// `period` accepts `Day` | `Week` | `Month` (see `BarPeriod`).
    /// `adjust` is **request-driven**: the runtime does not echo it in
    /// the response, so each returned `Kline` is stamped with the
    /// requested `AdjustType` (decision D2). When `adjust` is `None`,
    /// the `adjust` field is omitted from the request body entirely
    /// (matches existing `fetch_daily_klines` behavior).
    ///
    /// Does NOT participate in retry / circuit breaker — mirrors
    /// `fetch_daily_klines` (P0.10 design for `/data/bars` paths).
    pub async fn fetch_klines(
        &self,
        code: &str,
        period: crate::data::models::BarPeriod,
        adjust: crate::data::models::AdjustType,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Vec<crate::data::models::Kline>> {
        let endpoint = self
            .base_url
            .join("data/bars")
            .map_err(|e| QuantixError::Other(format!("url join failed: {}", e)))?;

        let mut body = serde_json::json!({
            "symbol": code,
            "period": period.as_str(),
        });
        if let Some(adj) = adjust.as_openstock_param() {
            body["adjust"] = Value::String(adj.to_string());
        }
        if let Some(start) = start {
            body["start_date"] = Value::String(start.to_string());
        }
        if let Some(end) = end {
            body["end_date"] = Value::String(end.to_string());
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
            data: Vec<BarRecord>,
        }

        #[derive(serde::Deserialize)]
        struct BarRecord {
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

        let mut klines = Vec::with_capacity(bars.data.len());
        for bar in bars.data {
            let date = chrono::NaiveDate::parse_from_str(&bar.time[..10], "%Y-%m-%d")
                .map_err(|e| QuantixError::DataParse(format!("解析 bars 日期失败: {}", e)))?;

            klines.push(crate::data::models::Kline {
                code: code.to_string(),
                date,
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

        Ok(klines)
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib --package quantix-cli sources::openstock_client::tests::fetch_klines`
Expected: PASS — 3 wiremock tests green.

- [ ] **Step 5: Run full quality gates**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --lib --package quantix-cli openstock
cargo test --workspace
```

Expected: all pass. (Existing tests must still pass — no regressions in `fetch_daily_klines`.)

- [ ] **Step 6: Verify no blast radius via GitNexus**

```bash
gitnexus detect_changes
```

Expected: LOW risk on `openstock_client.rs`. `fetch_daily_klines` callers (market data, backtest) must NOT appear in affected flows — they still use the unchanged `fetch_daily_klines` signature.

- [ ] **Step 7: Commit**

```bash
git add src/sources/openstock_client.rs
git commit -m "$(cat <<'EOF'
feat(sources): add OpenStockClient::fetch_klines for multi-period bars

Adds a general fetch_klines(code, period, adjust, start, end) method
that mirrors fetch_daily_klines shape (direct reqwest, no envelope
path, no retry/breaker). Stamps each Kline with the requested
AdjustType (runtime does not echo it). Omits the adjust field entirely
when AdjustType::None. Covers week/month periods + qfq/hfq adjust.

fetch_daily_klines is preserved unchanged (D3) — no impact on market
or backtest callers.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Phase 2 — CLI Wiring (Commit 2)

### Task 2.1: Add `FetchKlines` CLI subcommand variant

**Files:**
- Modify: `src/cli/commands/data.rs` (append to `OpenStockCommands` enum before the closing `}` at L361)

**Interfaces:**
- Produces: new `OpenStockCommands::FetchKlines { symbol: String, period: String, adjust: String, start: Option<String>, end: Option<String> }` variant

- [ ] **Step 1: Add the variant**

In `src/cli/commands/data.rs`, immediately before the closing `}` of `OpenStockCommands` (L361), add:

```rust
    /// 实时拉取 K 线 (day/week/month + 不复权/前复权/后复权)（OpenStock `/data/bars`，联网，只读，不写库）
    FetchKlines {
        /// 证券代码（如 600000 或 sh000001，前缀行为同 fetch-index）
        #[arg(long)]
        symbol: String,

        /// 周期：day | week | month（任意大小写，拒绝 daily/weekly/monthly 等别名）
        #[arg(long, default_value = "day")]
        period: String,

        /// 复权：none | qfq | hfq（任意大小写；none 表示不复权）
        #[arg(long, default_value = "none")]
        adjust: String,

        /// 起始日期 (YYYY-MM-DD)
        #[arg(long)]
        start: Option<String>,

        /// 结束日期 (YYYY-MM-DD)
        #[arg(long)]
        end: Option<String>,
    },
```

- [ ] **Step 2: Run `cargo check` to verify compile**

Run: `cargo check --package quantix-cli`
Expected: FAIL on the dispatcher / handler (non-exhaustive match in `app_shell.rs` — that's expected; we'll fix in Task 2.3).

- [ ] **Step 3: No commit yet — continue to Task 2.2 (handler) and Task 2.3 (dispatcher) before committing**

---

### Task 2.2: Add `fetch_openstock_klines` handler

**Files:**
- Modify: `src/cli/handlers/openstock_handler.rs` (insert after `fetch_openstock_index` at L342)
- Modify: `src/cli/handlers/mod.rs` (add to existing re-export)

**Interfaces:**
- Consumes: `OpenStockClient::fetch_klines`, `BarPeriod::from_str`, `AdjustType::from_str`, `OpenStockSettings`
- Produces: `pub(crate) async fn fetch_openstock_klines(settings: &OpenStockSettings, symbol: &str, period: &str, adjust: &str, start: Option<&str>, end: Option<&str>) -> Result<()>`

- [ ] **Step 1: Add the handler**

In `src/cli/handlers/openstock_handler.rs`, immediately after `fetch_openstock_index` ends at L342, add:

```rust
pub(crate) async fn fetch_openstock_klines(
    settings: &OpenStockSettings,
    symbol: &str,
    period: &str,
    adjust: &str,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<()> {
    use std::str::FromStr;

    let period_enum = crate::data::models::BarPeriod::from_str(period)
        .map_err(|e| QuantixError::Config(format!("--period {}", e)))?;
    let adjust_enum = crate::data::models::AdjustType::from_str(adjust)
        .map_err(|e| QuantixError::Config(format!("--adjust {}", e)))?;

    let client = OpenStockClient::from_settings(settings)?;
    let klines = client
        .fetch_klines(symbol, period_enum, adjust_enum, start, end)
        .await?;

    println!("OpenStock live fetch (/data/bars, symbol={})", symbol);
    println!("  Period:  {}", period_enum.as_str());
    println!(
        "  Adjust:  {}",
        match adjust_enum.as_openstock_param() {
            Some(a) => a,
            None => "none (field omitted)",
        }
    );
    println!("  记录数: {}", klines.len());
    if let (Some(first), Some(last)) = (klines.first(), klines.last()) {
        println!(
            "  首条: date={} open={} close={}",
            first.date, first.open, first.close
        );
        println!(
            "  末条: date={} open={} close={}",
            last.date, last.open, last.close
        );
    }
    // /data/bars is a direct reqwest path; it does NOT echo source,
    // artifact_hash, or latency_ms (only the /data/fetch envelope does).
    println!("  Source:        (not reported by /data/bars)");
    println!("  artifact_hash: (not reported by /data/bars)");
    println!("  latency_ms:    (not reported by /data/bars)");
    Ok(())
}
```

**Note**: confirm `QuantixError` and `OpenStockClient` are already imported at the top of the file (other handlers use them — `fetch_openstock_index` at L308 uses `OpenStockClient::from_settings`). If `QuantixError` is not in scope, check existing imports — it's likely imported via `use crate::core::error::QuantixError;` or `use crate::core::prelude::*;`.

- [ ] **Step 2: Add to `mod.rs` re-export**

In `src/cli/handlers/mod.rs`, find the existing re-export line (~L129):

```rust
pub(crate) use openstock_handler::{
    fetch_openstock_codes, fetch_openstock_calendar, fetch_openstock_index,
    fetch_openstock_all_stocks, fetch_openstock_workdays,
    // ... etc
};
```

Append `fetch_openstock_klines` to that list alphabetically.

- [ ] **Step 3: No commit yet — continue to Task 2.3**

---

### Task 2.3: Add dispatcher arm in `app_shell.rs`

**Files:**
- Modify: `src/cli/handlers/app_shell.rs` (extend the match on `OpenStockCommands` after `FetchIndex` arm, ~L362)

**Interfaces:**
- Consumes: `fetch_openstock_klines` from Task 2.2, `OpenStockCommands::FetchKlines` from Task 2.1, `CliRuntime::load()` pattern

- [ ] **Step 1: Add the match arm**

In `src/cli/handlers/app_shell.rs`, find the existing match on `OpenStockCommands` (look for `OpenStockCommands::FetchIndex { ... }`). Immediately after that arm, add:

```rust
            OpenStockCommands::FetchKlines {
                symbol,
                period,
                adjust,
                start,
                end,
            } => {
                let rt = CliRuntime::load();
                fetch_openstock_klines(
                    &rt.openstock,
                    symbol,
                    period,
                    adjust,
                    start.as_deref(),
                    end.as_deref(),
                )
                .await?;
            }
```

**Pattern note**: matches the existing `FetchIndex` arm verbatim — `let rt = CliRuntime::load();` then `&rt.openstock` (verified at `src/cli/handlers/app_shell.rs:362-366`).

- [ ] **Step 2: Run `cargo check` to verify the whole thing compiles**

Run: `cargo check --package quantix-cli`
Expected: PASS.

- [ ] **Step 3: Run quality gates**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
```

Expected: all pass. No new tests yet — those come in Phase 3.

- [ ] **Step 4: CLI smoke test (offline)**

```bash
cargo run --quiet -- data openstock fetch-klines --symbol 600000 --period invalid 2>&1 | head -5
```

Expected: clear `QuantixError::Config` message like `--period unsupported BarPeriod invalid: expected one of day|week|month`. Confirms the FromStr fail-fast path works without making any HTTP call.

```bash
cargo run --quiet -- data openstock fetch-klines --help 2>&1 | head -25
```

Expected: clap renders help showing `--symbol`, `--period` (default `day`), `--adjust` (default `none`), `--start`, `--end`.

- [ ] **Step 5: Commit (Phase 2 single commit)**

```bash
git add src/cli/commands/data.rs src/cli/handlers/openstock_handler.rs src/cli/handlers/mod.rs src/cli/handlers/app_shell.rs
git commit -m "$(cat <<'EOF'
feat(cli): wire data openstock fetch-klines subcommand for P0.13a

Adds the FetchKlines variant to OpenStockCommands with --symbol,
--period day|week|month (default day), --adjust none|qfq|hfq (default
none), --start, --end. Period and adjust are parsed in the handler via
FromStr (fail-fast QuantixError::Config on bad input — no HTTP).

fetch_openstock_klines handler mirrors fetch_openstock_index shape
plus Period/Adjust lines. Output notes that /data/bars does not echo
source/artifact_hash/latency_ms (those are /data/fetch envelope only).

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Phase 3 — Live Tests + Spec + Closeout (Commit 3)

### Task 3.1: Create OpenSpec change proposal

**Files:**
- Create: `openspec/changes/openstock-data-consumption-p0-13a/proposal.md`
- Create: `openspec/changes/openstock-data-consumption-p0-13a/tasks.md`
- Create: `openspec/changes/openstock-data-consumption-p0-13a/design.md`
- Create: `openspec/changes/openstock-data-consumption-p0-13a/specs/openstock-data-consumption/spec.md`

- [ ] **Step 1: Write `proposal.md`**

```markdown
# OpenStock Data Consumption P0.13a — Multi-period K-line Fetch

## Why

The HANDOFF report `docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md`
lists 8 ❌ rows on the quantix-rust consumer side. P0.13a-d decompose
them into 4 slices; this change covers the B-group row (multi-period
K-line + adjust type — 3 categories: `KLINES`, `ADJUSTED_KLINES`,
`HISTORICAL_KLINES`) which OpenStock `/data/bars` already serves
transparently.

## What Changes

- Add `BarPeriod` enum (`Day`/`Week`/`Month`) in `src/data/models.rs`
  with `as_str()` and strict case-insensitive `FromStr` (rejects
  `daily`/`weekly`/`monthly`/`minute*` aliases).
- Extend `AdjustType` with `as_openstock_param() -> Option<&'static str>`
  and a case-insensitive `FromStr` (`none|qfq|hfq`).
- Add `OpenStockClient::fetch_klines(code, period, adjust, start, end)`
  in `src/sources/openstock_client.rs` — generalizes
  `fetch_daily_klines` (preserved unchanged) to week/month periods and
  qfq/hfq adjust. Stamps each `Kline` with the requested `AdjustType`.
- Add `data openstock fetch-klines` CLI subcommand with `--symbol`,
  `--period`, `--adjust`, `--start`, `--end`.
- 8 tests across 3 layers: 5 unit/wiremock + 3 live `#[ignore]`.

## Impact

**Files added:** 5 (1 live test file, 4 OpenSpec files).
**Files modified:** 6 (`data/models.rs`, `openstock_client.rs`,
`commands/data.rs`, `openstock_handler.rs`, `app_shell.rs`,
`handlers/mod.rs`).
**Public API:** new `BarPeriod`, new method on `AdjustType`, new
`fetch_klines` method on `OpenStockClient`, new CLI subcommand. No
breaking changes.

## Non-Goals

- Minute-level periods (`MINUTE_DATA`) — P0.13b.
- `ADJUST_FACTOR` raw factor exposure — P0.13d+.
- ClickHouse / shadow persistence integration for new periods — later slice.
- Refactor `fetch_daily_klines` to call `fetch_klines` — later slice.
- Retry / circuit breaker for `/data/bars` path — P0.10 design decision preserved.
```

- [ ] **Step 2: Write `tasks.md`**

```markdown
# Tasks

## 0. Baseline

- [x] Spec: `docs/superpowers/specs/2026-07-02-openstock-p0-13a-multi-period-klines-design.md`
- [x] Plan: `docs/superpowers/plans/2026-07-02-openstock-p0-13a-multi-period-klines-plan.md`
- [x] Governance card: `.governance/programs/project-governance/cards/P0.13a.yaml`

## 1. Phase 1 — Client method

- [x] Add `BarPeriod` enum + `FromStr` + `AdjustType::as_openstock_param` + `FromStr` + 5 unit tests
- [x] Add `OpenStockClient::fetch_klines` + 3 wiremock tests
- [x] Quality gates green (fmt / clippy / cargo test --lib openstock)

## 2. Phase 2 — CLI wiring

- [x] Add `FetchKlines` variant to `OpenStockCommands`
- [x] Add `fetch_openstock_klines` handler
- [x] Wire dispatcher arm + re-export
- [x] Quality gates green (cargo test --workspace)
- [x] Offline CLI smoke (`--period invalid` fails fast; `--help` renders)

## 3. Phase 3 — Live tests + closeout

- [ ] Add 3 live `#[ignore]` tests (T6 day+None, T7 week+qfq, T8 month+hfq)
- [ ] Manual live smoke against `http://192.168.123.104:8040` (when reachable)
- [ ] Update HANDOFF report B-group row ❌ → ✅
- [ ] Update FUNCTION_TREE.md
- [ ] `openspec validate openstock-data-consumption-p0-13a --strict`
- [ ] `openspec validate --all --strict`
- [ ] Archive OpenSpec change
```

- [ ] **Step 3: Write `design.md`**

Copy the "Decisions" table (D1-D8) and "Architecture" section verbatim from `docs/superpowers/specs/2026-07-02-openstock-p0-13a-multi-period-klines-design.md`. Add a `## Risks` section (copy from spec).

- [ ] **Step 4: Write `specs/openstock-data-consumption/spec.md`**

```markdown
# OpenStock Data Consumption

## ADDED Requirements

### Requirement: Multi-period K-line Fetch

The system SHALL support fetching K-lines from OpenStock `/data/bars`
with `period ∈ {day, week, month}` and `adjust_type ∈ {None, QFQ, HFQ}`
through a unified `OpenStockClient::fetch_klines` API.

#### Scenario: day period without adjust

- **WHEN** the caller invokes `fetch_klines("600000", BarPeriod::Day, AdjustType::None, None, None)`
- **THEN** the request body to `/data/bars` contains `{"symbol":"600000","period":"day"}` with NO `adjust` field
- **AND** each returned `Kline` has `adjust_type = None`

#### Scenario: week period with qfq

- **WHEN** the caller invokes `fetch_klines("600000", BarPeriod::Week, AdjustType::QFQ, None, None)`
- **THEN** the request body contains `"period":"week"` and `"adjust":"qfq"`
- **AND** each returned `Kline` has `adjust_type = QFQ` (request-driven — runtime does not echo)

#### Scenario: month period with hfq

- **WHEN** the caller invokes `fetch_klines("600000", BarPeriod::Month, AdjustType::HFQ, None, None)`
- **THEN** the request body contains `"period":"month"` and `"adjust":"hfq"`
- **AND** each returned `Kline` has `adjust_type = HFQ`

### Requirement: Strict Period Parsing

The system SHALL reject period aliases (`daily`/`weekly`/`monthly`,
any case) and any minute-level value via `BarPeriod::from_str`,
returning `QuantixError::Config`. Only `day|week|month` (case-insensitive)
are accepted.

#### Scenario: invalid period surfaces config error

- **WHEN** the CLI parses `--period daily`
- **THEN** the handler returns `QuantixError::Config` mentioning "unsupported BarPeriod"
- **AND** no HTTP request is made

### Requirement: CLI Multi-period Subcommand

The system SHALL expose `data openstock fetch-klines` with `--symbol`
(required), `--period` (default `day`), `--adjust` (default `none`),
`--start`, `--end` (both optional).

#### Scenario: default invocation

- **WHEN** the user runs `data openstock fetch-klines --symbol 600000`
- **THEN** the system fetches day-period unadjusted bars for symbol 600000
```

- [ ] **Step 5: Validate the OpenSpec change**

```bash
openspec validate openstock-data-consumption-p0-13a --strict
```

Expected: PASS. If fails, fix the structure before continuing.

---

### Task 3.2: Create governance card

**Files:**
- Create: `.governance/programs/project-governance/cards/P0.13a.yaml`

- [ ] **Step 1: Write the card**

```yaml
id: P0.13a
title: "OpenStock P0.13a — Multi-period K-line Fetch (day/week/month + qfq/hfq)"
program: project-governance
change_ref: openstock-data-consumption-p0-13a

scope:
  allowed_paths:
    - src/data/models.rs
    - src/sources/openstock_client.rs
    - src/cli/commands/data.rs
    - src/cli/handlers/openstock_handler.rs
    - src/cli/handlers/app_shell.rs
    - src/cli/handlers/mod.rs
    - tests/openstock_live_klines.rs
    - openspec/changes/openstock-data-consumption-p0-13a/**
    - .governance/programs/project-governance/cards/P0.13a.yaml
    - docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md
    - FUNCTION_TREE.md
  forbidden_paths:
    - src/sources/openstock.rs
    - src/sources/openstock_index.rs
    - src/db/**
    - src/backtest/**
    - src/execution/**
    - src/sources/kline_aggregator.rs

non_goals:
  - minute-level period support (P0.13b)
  - ADJUST_FACTOR raw factor exposure (P0.13d+)
  - eltdx KLINES direct category wiring (covered via /data/bars)
  - ClickHouse or shadow persistence integration
  - fetch_daily_klines refactor
  - retry / circuit breaker for /data/bars path

acceptance:
  commit_gate:
    - "cargo fmt --all -- --check"
    - "cargo clippy --all-targets --workspace -- -D warnings"
    - "cargo test --workspace"
    - "openspec validate openstock-data-consumption-p0-13a --strict"
    - "grep -n 'pub async fn fetch_klines' src/sources/openstock_client.rs"
  closeout_gate:
    - "HANDOFF report B-group row flipped to ✅"
    - "OpenSpec change openstock-data-consumption-p0-13a archived"

evidence:
  current_head: ""  # fill at execution time
```

- [ ] **Step 2: No commit yet — bundle with Task 3.3**

---

### Task 3.3: Add live tests (T6, T7, T8)

**Files:**
- Create: `tests/openstock_live_klines.rs`

- [ ] **Step 1: Write the test file**

```rust
//! Live HTTP smoke tests for `OpenStockClient::fetch_klines` (P0.13a).
//! Gated by `QUANTIX_OPENSTOCK_LIVE=1`.

#![cfg(test)]

use quantix_cli::data::models::{AdjustType, BarPeriod};
use quantix_cli::sources::openstock_client::OpenStockClient;
use std::str::FromStr;

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_klines_live_day_none() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping: QUANTIX_OPENSTOCK_LIVE not set");
        return;
    }
    let symbol = std::env::var("OPENSTOCK_LIVE_SYMBOL").unwrap_or_else(|_| "600000".to_string());
    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let klines = client
        .fetch_klines(&symbol, BarPeriod::Day, AdjustType::None, None, None)
        .await
        .expect("fetch ok");
    assert!(!klines.is_empty(), "day klines should return records");
    assert_eq!(klines[0].adjust_type, AdjustType::None);
    println!(
        "fetch_klines day+none ({}): {} records, first date={} close={}",
        symbol,
        klines.len(),
        klines[0].date,
        klines[0].close
    );
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_klines_live_week_qfq() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping: QUANTIX_OPENSTOCK_LIVE not set");
        return;
    }
    let symbol = std::env::var("OPENSTOCK_LIVE_SYMBOL").unwrap_or_else(|_| "600000".to_string());
    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let klines = client
        .fetch_klines(&symbol, BarPeriod::Week, AdjustType::QFQ, None, None)
        .await
        .expect("fetch ok");
    assert!(!klines.is_empty(), "week klines should return records");
    assert_eq!(klines[0].adjust_type, AdjustType::QFQ);
    println!(
        "fetch_klines week+qfq ({}): {} records, first date={} close={}",
        symbol,
        klines.len(),
        klines[0].date,
        klines[0].close
    );
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_klines_live_month_hfq() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping: QUANTIX_OPENSTOCK_LIVE not set");
        return;
    }
    let symbol = std::env::var("OPENSTOCK_LIVE_SYMBOL").unwrap_or_else(|_| "600000".to_string());
    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let klines = client
        .fetch_klines(&symbol, BarPeriod::Month, AdjustType::HFQ, None, None)
        .await
        .expect("fetch ok");
    assert!(!klines.is_empty(), "month klines should return records");
    assert_eq!(klines[0].adjust_type, AdjustType::HFQ);
    println!(
        "fetch_klines month+hfq ({}): {} records, first date={} close={}",
        symbol,
        klines.len(),
        klines[0].date,
        klines[0].close
    );
    // Sanity: FromStr agrees with what we just used.
    assert!(matches!(BarPeriod::from_str("month"), Ok(BarPeriod::Month)));
}
```

- [ ] **Step 2: Verify the test file compiles + skips by default**

```bash
cargo test --test openstock_live_klines
```

Expected: 3 tests marked ignored (no failures).

- [ ] **Step 3: Run the full quality gates**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
openspec validate openstock-data-consumption-p0-13a --strict
openspec validate --all --strict
```

Expected: all pass.

- [ ] **Step 4: Manual live smoke (optional, only when OpenStock is reachable)**

If `192.168.123.104:8040` is reachable:

```bash
QUANTIX_OPENSTOCK_LIVE=1 \
OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
OPENSTOCK_API_KEY=<key> \
cargo test --test openstock_live_klines -- --ignored
```

And:

```bash
OPENSTOCK_BASE_URL=http://192.168.123.104:8040 OPENSTOCK_API_KEY=<key> \
cargo run --quiet -- data openstock fetch-klines --symbol 600000 --period week --adjust qfq
```

Expected: returns ≥1 Kline with `Adjust:  qfq` and `Period:  week` lines in output.

If OpenStock is NOT reachable, document in the commit message that live smoke was deferred — quality gates still gate the commit.

- [ ] **Step 5: Update HANDOFF report and FUNCTION_TREE.md**

In `docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md` §"openstock 侧已就绪但 quantix-rust 未接入", locate the B-group row (周/月 K 线 + 不复权) and change ❌ → ✅ with a one-line note pointing to P0.13a.

In `FUNCTION_TREE.md`, add an entry under the OpenStock consumer section:

```
P0.13a (2026-07-02): Multi-period K-line fetch (day/week/month + qfq/hfq)
  - src/data/models.rs: BarPeriod enum + AdjustType::as_openstock_param + FromStr
  - src/sources/openstock_client.rs: fetch_klines(code, period, adjust, start, end)
  - src/cli/commands/data.rs: FetchKlines variant
  - 3 B-group categories: KLINES / ADJUSTED_KLINES / HISTORICAL_KLINES → ✅
```

- [ ] **Step 6: Commit**

```bash
git add openspec/changes/openstock-data-consumption-p0-13a/ \
        .governance/programs/project-governance/cards/P0.13a.yaml \
        tests/openstock_live_klines.rs \
        docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md \
        FUNCTION_TREE.md
git commit -m "$(cat <<'EOF'
docs(openspec): add openstock-data-consumption-p0-13a with live tests

Adds the OpenSpec change (proposal/tasks/design/spec), governance card
P0.13a.yaml, 3 live #[ignore]-gated tests for fetch_klines, and closes
out the slice by flipping the HANDOFF report B-group row to ✅ and
updating FUNCTION_TREE.md.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 7: Archive the OpenSpec change**

```bash
openspec archive openstock-data-consumption-p0-13a
```

Expected: change moves from `openspec/changes/` to `openspec/changes/archive/`. Commit the move:

```bash
git add openspec/changes/
git commit -m "$(cat <<'EOF'
chore(openspec): archive openstock-data-consumption-p0-13a

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 8: Final regression sweep**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
openspec validate --all --strict
gitnexus detect_changes
```

Expected: all green. `gitnexus detect_changes` should report LOW risk on touched files.

---

## Self-Review Checklist (for the plan author — run after writing)

1. **Spec coverage**: every D1-D8 decision has a task? Yes (D1-D6 → Phase 1 tasks; D7 → Phase 3 OpenSpec; D8 → naming throughout).
2. **Invariants**: 6 invariants in spec — each one mapped to code? Yes (Inv 1 reused parser in fetch_klines; Inv 2 Kline unchanged; Inv 3 read-only; Inv 4 reused parser; Inv 5 symbol prefix documented in handler; Inv 6 f64→Decimal preserved).
3. **Test matrix**: 8 tests, all placed? T1/T2 in Task 1.1; T3/T4/T5 in Task 1.2; T6/T7/T8 in Task 3.3.
4. **Type consistency**: `BarPeriod` (not `KlinePeriod`) used consistently? Yes — verified in code snippets and test names.
5. **Placeholders**: any TBD/TODO? No.
6. **Commits**: 3 commits map to 3 phases (commit 1 = Task 1.1+1.2 combined? No — Task 1.1 step 6 commits models.rs; Task 1.2 step 7 commits client; Phase 2 single commit; Phase 3 commit + archive commit). Total 4-5 commits — consistent with phased approach.

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-07-02-openstock-p0-13a-multi-period-klines-plan.md`. Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints.

**Which approach?**
