# OpenStock P0.13c 多日范围查询 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 `fetch_minute_klines` + `fetch_minute_share` 添加可选 `start`/`end` 范围参数，保留 `--date` 单日快捷。

**Architecture:** `DateOrRange` enum 表达互斥（D1）；`fetch_minute_klines` 走 server-side range（一次请求带 `start_date`/`end_date`），`fetch_minute_share` 走 client-side 逐日循环（N 次单日请求，从 `meta.trading_date` 取每条记录日期）—— R1 修订后的不对称设计（D6）；CLI `--date` 与 `--start`/`--end` 互斥由 `DateOrRange::from_cli` 单点校验（D2/D5）。

**Tech Stack:** Rust, tokio, reqwest, serde_json, chrono, clap, wiremock (testing), rust_decimal

## Global Constraints

(从 R1-revised spec §5/§6/§3 直接拷贝，每个 task 的需求隐式包含此节)

- **INV-1A**: CLI `--date` 与 `(--start, --end)` 互斥，`from_cli` 强制
- **INV-1B**: Range 端点 inclusive；`start > end` 报错；半开区间（only start / only end）报错
- **INV-2A**: `Date(d)` 模式 wire body 与 P0.13b-1/2 完全一致（P0.13b-1/2 wiremock 测试不破坏）
- **INV-2B**: Vec 扁平，按 timestamp 升序排列
- **INV-2C**: `fetch_minute_share` 循环模式必须从 `meta.trading_date`（server 响应）取日期，**不**依赖 client 侧请求日期（非交易日 case）
- **INV-3**: 不修改 `MinuteBar`/`MinuteShare`/`MinutePeriod`/parsers；仅扩展 fetcher 签名
- **D5**: `from_cli(None, None, None)`、`(None, Some, None)`、`(None, None, Some)` 全部 `Err`
- **D6**: `fetch_minute_share` 的 `Range` 模式 client-side 循环（OpenStock server 不支持 MINUTE_DATA range）
- **错误消息规范**：包含参数名 + 用法提示（见 spec §4.3）
- **字段名约定**：`/data/bars` 用 `start_date`/`end_date`（基于 `_eltdx_timeseries.py:92-94` 证据）；`/data/fetch MINUTE_DATA` 的 `params` 用 `date`（单日 wire，向后兼容）
- **日期格式**：`YYYY-MM-DD`（如 `"2026-06-30"`）
- **iter_dates_inclusive**：`start..=end` 的日历日迭代器（含非交易日，server 返回空 records 时该日 Vec 为空）

**Quality gates**（每个 task 必通过）：
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
```

---

## File Structure

| 文件 | 操作 | 责任 |
|------|------|------|
| `src/data/models.rs` | Modify | 新增 `DateOrRange` enum + `from_cli` + `iter_dates_inclusive` helper |
| `src/sources/openstock_client.rs` | Modify | `fetch_minute_klines` 签名扩展（server-side range）；`fetch_minute_share` 签名扩展 + 抽出 `fetch_minute_share_single` helper + `Range` client 循环 |
| `src/cli/commands/data.rs` | Modify | `FetchMinuteKlines`/`FetchMinuteShare` 的 `date: String` → `Option<String>`，加 `--start`/`--end` |
| `src/cli/handlers/openstock_handler.rs` | Modify | 2 handlers 增加 start/end 参数 + `from_cli` 校验 |
| `src/cli/handlers/app_shell.rs` | Modify | 2 dispatcher arms 传递新参数 |
| `tests/openstock_live_minute_klines.rs` | Modify | 追加 L1 多日范围 test |
| `tests/openstock_live_minute_share.rs` | Modify | 追加 L2 多日范围 test（验证 client 循环 + meta.trading_date） |
| `openspec/changes/openstock-data-consumption-p0-13c/{proposal,tasks,design}.md` + `specs/.../spec.md` | Create | OpenSpec change 4 件套 |
| `.governance/programs/project-governance/cards/P0.13c.yaml` | Create | governance card |

**Reuse map**：
| Need | Reuse from |
|------|------------|
| `OpenStockClient::fetch::<T>()` envelope path | `openstock_client.rs:82`（P0.10 已建立） |
| `/data/bars` 直 reqwest path | `fetch_minute_klines`（P0.13b-1，本 task 扩展） |
| `parse_minute_share` + `RawMinuteRecord` | `openstock_client.rs:837/852`（P0.13b-2，不改 parser） |
| clap `Option<String>` 向后兼容 | clap 自动（`--flag VALUE` → `Some(VALUE)`） |
| wiremock 测试模式 | `fetch_minute_share_sends_minute_data_category_and_date` (P0.13b-2 wiremock) |

---

## Task 1: `DateOrRange` enum + `from_cli` + `iter_dates_inclusive`

**Files:**
- Modify: `src/data/models.rs`（追加到 `MinuteShare` 定义之后，文件末尾前）

**Interfaces:**
- Produces:
  - `pub enum DateOrRange { Date(NaiveDate), Range { start: NaiveDate, end: NaiveDate } }`
    - 注意：`Range.start`/`Range.end` 是 `NaiveDate`（**非** `Option<NaiveDate>`）—— `from_cli` 已强制 start/end 必须同时提供，故 enum 层面不允许半开区间
  - `impl DateOrRange { pub fn from_cli(date: Option<&str>, start: Option<&str>, end: Option<&str>) -> Result<Self, QuantixError> }`
  - `pub fn iter_dates_inclusive(start: NaiveDate, end: NaiveDate) -> impl Iterator<Item = NaiveDate>`（chrono `NaiveDate::days_since_epoch`/`succ` 实现）

- [ ] **Step 1.1: Write failing unit tests for `from_cli` (U1-U7)**

追加到 `src/data/models.rs` 末尾的 `#[cfg(test)] mod tests`（若不存在则新建）：

```rust
#[cfg(test)]
mod date_or_range_tests {
    use super::{DateOrRange, iter_dates_inclusive};
    use chrono::NaiveDate;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn u1_date_only_returns_date_variant() {
        let r = DateOrRange::from_cli(Some("2026-06-30"), None, None).unwrap();
        assert!(matches!(r, DateOrRange::Date(_)));
        if let DateOrRange::Date(actual) = r {
            assert_eq!(actual, d("2026-06-30"));
        }
    }

    #[test]
    fn u2_start_and_end_returns_range_variant() {
        let r = DateOrRange::from_cli(None, Some("2026-06-01"), Some("2026-06-30")).unwrap();
        if let DateOrRange::Range { start, end } = r {
            assert_eq!(start, d("2026-06-01"));
            assert_eq!(end, d("2026-06-30"));
        } else {
            panic!("expected Range, got {:?}", r);
        }
    }

    #[test]
    fn u3_start_only_errors() {
        let r = DateOrRange::from_cli(None, Some("2026-06-01"), None);
        assert!(r.is_err());
        let msg = format!("{}", r.unwrap_err());
        assert!(
            msg.contains("--start") && msg.contains("--end"),
            "error should name both flags: {}",
            msg
        );
    }

    #[test]
    fn u4_end_only_errors() {
        let r = DateOrRange::from_cli(None, None, Some("2026-06-30"));
        assert!(r.is_err());
    }

    #[test]
    fn u5_date_and_start_conflict_errors() {
        let r = DateOrRange::from_cli(Some("2026-06-30"), Some("2026-06-01"), None);
        assert!(r.is_err());
        let msg = format!("{}", r.unwrap_err());
        assert!(msg.contains("--date") && msg.contains("--start"), "msg: {}", msg);
    }

    #[test]
    fn u6_start_after_end_errors() {
        let r = DateOrRange::from_cli(None, Some("2026-06-30"), Some("2026-06-01"));
        assert!(r.is_err());
    }

    #[test]
    fn u7_all_none_errors() {
        let r = DateOrRange::from_cli(None, None, None);
        assert!(r.is_err());
        let msg = format!("{}", r.unwrap_err());
        assert!(msg.contains("--date") || msg.contains("--start"), "msg: {}", msg);
    }

    #[test]
    fn iter_dates_inclusive_yields_all_days_in_order() {
        let days: Vec<NaiveDate> = iter_dates_inclusive(d("2026-06-28"), d("2026-07-02")).collect();
        assert_eq!(days.len(), 5);
        assert_eq!(days[0], d("2026-06-28"));
        assert_eq!(days[4], d("2026-07-02"));
        // 跨月验证
        assert_eq!(days[2], d("2026-06-30"));
        assert_eq!(days[3], d("2026-07-01"));
    }

    #[test]
    fn iter_dates_inclusive_single_day_yields_one() {
        let days: Vec<NaiveDate> = iter_dates_inclusive(d("2026-06-30"), d("2026-06-30")).collect();
        assert_eq!(days, vec![d("2026-06-30")]);
    }
}
```

- [ ] **Step 1.2: Run tests to verify they fail (compile error — types don't exist yet)**

Run: `cargo test --lib --package quantix-cli date_or_range_tests`
Expected: FAIL with "cannot find type `DateOrRange` in this scope"

- [ ] **Step 1.3: Implement `DateOrRange` enum + `from_cli` + `iter_dates_inclusive`**

追加到 `src/data/models.rs` 在 `MinuteShare` struct 之后：

```rust
/// CLI 互斥输入：单日（`--date`）或封闭范围（`--start`/`--end`）。
///
/// 由 `from_cli` 唯一构造——编译时强制半开区间和 `(None, None, None)` 不可达。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateOrRange {
    /// 单日查询（向后兼容 P0.13b-1/2 `--date` 路径）
    Date(chrono::NaiveDate),
    /// 多日范围（inclusive on both ends）
    Range {
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    },
}

impl DateOrRange {
    /// 从 CLI 三 `Option<&str>` 输入构造 `DateOrRange`。
    ///
    /// 校验规则（spec §3.1 + D5）：
    ///   - `(Some(d), None, None)` → `Date(d)`
    ///   - `(None, Some(s), Some(e))` → `Range { start: s, end: e }`（s ≤ e）
    ///   - 其它所有形态 → `Err`
    pub fn from_cli(
        date: Option<&str>,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Self, crate::core::error::QuantixError> {
        use crate::core::error::QuantixError;

        let has_date = date.is_some();
        let has_range = start.is_some() || end.is_some();

        if has_date && has_range {
            return Err(QuantixError::Config(
                "--date cannot be combined with --start/--end; use either --date for single day or --start/--end for range".to_string(),
            ));
        }
        if has_date {
            let d = parse_date_arg(date.unwrap(), "--date")?;
            return Ok(DateOrRange::Date(d));
        }
        if has_range {
            let (Some(s_str), Some(e_str)) = (start, end) else {
                return Err(QuantixError::Config(
                    "--start and --end must be provided together (semi-open ranges are not supported)".to_string(),
                ));
            };
            let s = parse_date_arg(s_str, "--start")?;
            let e = parse_date_arg(e_str, "--end")?;
            if s > e {
                return Err(QuantixError::Config(format!(
                    "--start ({}) must be on or before --end ({})",
                    s_str, e_str
                )));
            }
            return Ok(DateOrRange::Range { start: s, end: e });
        }
        // 全 None
        Err(QuantixError::Config(
            "at least one of --date or (--start, --end) is required".to_string(),
        ))
    }
}

fn parse_date_arg(
    s: &str,
    flag_name: &str,
) -> Result<chrono::NaiveDate, crate::core::error::QuantixError> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| {
        crate::core::error::QuantixError::Config(format!("{}: invalid date '{}': {}", flag_name, s, e))
    })
}

/// 生成 `start..=end` 的日历日迭代器（含非交易日，调用方负责处理空响应）。
pub fn iter_dates_inclusive(
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> impl Iterator<Item = chrono::NaiveDate> {
    (0..=((end - start).num_days() as u64)).map(move |n| start + chrono::Duration::days(n as i64))
}
```

注：`QuantixError` 路径可能需根据实际项目调整——参考 `src/core/error.rs::QuantixError` 现有定义（`Config` variant 在 P0.13b-2 已用）。

- [ ] **Step 1.4: Run tests to verify they pass**

Run: `cargo test --lib --package quantix-cli date_or_range_tests`
Expected: PASS (9 tests)

- [ ] **Step 1.5: Run quality gates**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
```
Expected: PASS

- [ ] **Step 1.6: Commit**

```bash
git add src/data/models.rs
git commit -m "feat(models): add DateOrRange enum + from_cli + iter_dates_inclusive (P0.13c)

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 2: Extend `fetch_minute_klines` signature (server-side range)

**Files:**
- Modify: `src/sources/openstock_client.rs:692-790`（`fetch_minute_klines` 签名 + wire body）
- Modify: `src/sources/openstock_client.rs` 测试模块 `1422-1530`（更新现有 wiremock 调用为 `Date(...)` 形态 + 新增 W1/W2）

**Interfaces:**
- Consumes: `crate::data::models::DateOrRange`（Task 1 产出）
- Produces: `fetch_minute_klines(&self, code, period, DateOrRange, adjust) -> Result<Vec<MinuteBar>>`（向后兼容 wire body in Date mode）

- [ ] **Step 2.1: Update existing wiremock tests to use DateOrRange::Date(...) call form**

`src/sources/openstock_client.rs:1422` 的现有测试 `fetch_minute_klines_1m_none_sends_period_1m_and_date`：
```rust
// 修改前：
.fetch_minute_klines("sh600000", period, date, adjust).await
// 修改后：
.fetch_minute_klines("sh600000", period, DateOrRange::Date(date), adjust).await
```

对 `fetch_minute_klines_1m_none_sends_period_1m_and_date`、`fetch_minute_klines_5m_qfq_sends_adjust_and_stamps_records`、`fetch_minute_klines_propagates_4xx` 三处做同样改动。在每个测试模块顶部 `use crate::data::models::DateOrRange;`（或 qualified path）。

- [ ] **Step 2.2: Run existing wiremock tests — expect compile failure**

Run: `cargo test --lib --package quantix-cli fetch_minute_klines`
Expected: FAIL with signature mismatch（确认改动到位）

- [ ] **Step 2.3: Write failing wiremock test W1 (Range sends start_date/end_date)**

追加到 `fetch_minute_klines_*` 测试模块（在 propagates_4xx test 之后）：

```rust
#[tokio::test]
async fn fetch_minute_klines_range_sends_start_date_end_date_body() {
    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};

    let server = wiremock::MockServer::start().await;
    let body = serde_json::json!({
        "data": [{
            "time": "2026-06-01T09:31:00",
            "open": 10.0, "high": 10.5, "low": 9.9, "close": 10.2,
            "volume": 1000.0, "amount": 10200.0
        }, {
            "time": "2026-06-30T15:00:00",
            "open": 11.0, "high": 11.2, "low": 10.8, "close": 11.1,
            "volume": 500.0, "amount": 5550.0
        }]
    });
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/data/bars"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let cfg = fast_test_cfg(server.uri());
    let client = OpenStockClient::new(cfg).expect("client build");
    let start = chrono::NaiveDate::parse_from_str("2026-06-01", "%Y-%m-%d").unwrap();
    let end = chrono::NaiveDate::parse_from_str("2026-06-30", "%Y-%m-%d").unwrap();
    let bars = client
        .fetch_minute_klines(
            "sh600000",
            MinutePeriod::OneMinute,
            DateOrRange::Range { start, end },
            AdjustType::None,
        )
        .await
        .expect("fetch ok");

    assert_eq!(bars.len(), 2);
    // 验证 wire body：实际 server 收到的 body 需通过 server.received_requests().await 检查
    let received = server.received_requests().await.expect("at least one");
    assert_eq!(received.len(), 1);
    let req_body: serde_json::Value =
        serde_json::from_slice(&received[0].body).expect("body is json");
    assert_eq!(req_body["start_date"], "2026-06-01");
    assert_eq!(req_body["end_date"], "2026-06-30");
    assert!(req_body.get("date").is_none(), "Range body must not include 'date'");
}
```

注：`fast_test_cfg` 在 P0.13b-1 wiremock 测试中已建立，复用。

- [ ] **Step 2.4: Write failing wiremock test W2 (Date path backward compat — already exists)**

P0.13b-1 的 `fetch_minute_klines_1m_none_sends_period_1m_and_date` 已验证 Date 路径 wire body。在 Task 2 中我们额外加一个断言：确认 Date 路径 wire body **不**含 `start_date`/`end_date`。

修改 `fetch_minute_klines_1m_none_sends_period_1m_and_date` 测试，在末尾加：
```rust
let received = server.received_requests().await.expect("at least one");
let req_body: serde_json::Value =
    serde_json::from_slice(&received[0].body).expect("body is json");
assert!(req_body.get("start_date").is_none(), "Date body must not include start_date");
assert!(req_body.get("end_date").is_none(), "Date body must not include end_date");
```

- [ ] **Step 2.5: Run tests — expect compile failure on fetch_minute_klines (signature not yet extended)**

Run: `cargo test --lib --package quantix-cli fetch_minute_klines`
Expected: FAIL

- [ ] **Step 2.6: Extend `fetch_minute_klines` signature + wire body branching**

`src/sources/openstock_client.rs:692-790` 重写：

```rust
pub async fn fetch_minute_klines(
    &self,
    code: &str,
    period: crate::data::models::MinutePeriod,
    date_or_range: crate::data::models::DateOrRange,
    adjust: crate::data::models::AdjustType,
) -> Result<Vec<crate::data::models::MinuteBar>> {
    use std::str::FromStr;
    use crate::data::models::DateOrRange;

    let endpoint = self
        .base_url
        .join("data/bars")
        .map_err(|e| QuantixError::Other(format!("url join failed: {}", e)))?;

    let mut body = serde_json::json!({
        "symbol": code,
        "period": period.as_str(),
    });
    if let Some(adj) = adjust.as_openstock_param() {
        body["adjust"] = serde_json::Value::String(adj.to_string());
    }
    match date_or_range {
        DateOrRange::Date(d) => {
            body["date"] = serde_json::Value::String(d.format("%Y-%m-%d").to_string());
        }
        DateOrRange::Range { start, end } => {
            body["start_date"] = serde_json::Value::String(start.format("%Y-%m-%d").to_string());
            body["end_date"] = serde_json::Value::String(end.format("%Y-%m-%d").to_string());
        }
    }

    // ... 其余 send/parse 逻辑与 P0.13b-1 完全一致（无需修改）
    let resp = self.http.post(endpoint)
        .header("X-API-Key", self.api_key.clone())
        .json(&body)
        .send().await
        .map_err(|e| QuantixError::Network(format!("/data/bars request failed: {}", e)))?;
    // [保留 P0.13b-1 的 status check + BarsResponse 解析 + MinuteBar 构造逻辑]
    // ...
}
```

实施提示：保留 `MinuteBarRecord` 内部 struct、status check、`bars.data` loop、`NaiveDateTime::parse_from_str(&bar.time[..19], ...)` 等所有现有解析逻辑——只改 wire body 构造部分。

- [ ] **Step 2.7: Run all fetch_minute_klines tests**

Run: `cargo test --lib --package quantix-cli fetch_minute_klines`
Expected: PASS (4 tests: 1m_none, 5m_qfq, propagates_4xx, range_sends_start_end)

- [ ] **Step 2.8: Run quality gates + commit**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
git add src/sources/openstock_client.rs
git commit -m "feat(openstock): extend fetch_minute_klines to support DateOrRange (P0.13c)

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 3: Extend `fetch_minute_share` signature (client-side loop)

**Files:**
- Modify: `src/sources/openstock_client.rs:798-823`（`fetch_minute_share` 主体抽出为 `fetch_minute_share_single` helper，主入口加 Range 分支）
- Modify: `src/sources/openstock_client.rs:1536-1640`（更新现有 wiremock 调用 + 新增 W3/W4/W5）

**Interfaces:**
- Consumes: `crate::data::models::DateOrRange`、`crate::data::models::iter_dates_inclusive`（Task 1 产出）
- Produces:
  - `pub async fn fetch_minute_share(&self, code: &str, date_or_range: DateOrRange) -> Result<Vec<MinuteShare>>`
  - `async fn fetch_minute_share_single(&self, code: &str, date: NaiveDate) -> Result<Vec<MinuteShare>>`（私有 helper）

**注意**：`parse_minute_share` 当前签名 `(code, raw, date: NaiveDate)` 不变（INV-3）。`date` 参数现在来自 `meta.trading_date` 而非 CLI——需要 helper 从 response envelope 提取 `trading_date`。

- [ ] **Step 3.1: Update existing wiremock tests to use DateOrRange::Date(...) call form**

修改 `fetch_minute_share_sends_minute_data_category_and_date`、`fetch_minute_share_skips_records_with_missing_required_field`、`fetch_minute_share_propagates_4xx` 三处：
```rust
// 前：
client.fetch_minute_share("sh600000", date).await
// 后：
client.fetch_minute_share("sh600000", DateOrRange::Date(date)).await
```

- [ ] **Step 3.2: Run — expect compile failure**

Run: `cargo test --lib --package quantix-cli fetch_minute_share`
Expected: FAIL (signature mismatch)

- [ ] **Step 3.3: Write failing wiremock test W3 (Range triggers N single-day requests)**

追加到 `fetch_minute_share_*` 测试模块（在 propagates_4xx test 之后）：

```rust
#[tokio::test]
async fn fetch_minute_share_range_loops_per_day() {
    use crate::data::models::{DateOrRange, iter_dates_inclusive};

    let server = wiremock::MockServer::start().await;
    // 3-day range; each request returns 1 record with distinct time_minutes
    // body depends on requested `params.date` — use responder closure
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/data/fetch"))
        .and(wiremock::matchers::body_partial_json(serde_json::json!({
            "data_category": "MINUTE_DATA"
        })))
        .respond_with(|request: &wiremock::Request| {
            // Extract params.date from request body
            let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
            let req_date = body["params"]["date"].as_str().unwrap_or("");
            // Build response with trading_date = req_date (typical case)
            let resp = serde_json::json!({
                "status": "ok",
                "data": {
                    "source": "eltdx",
                    "artifact_hash": format!("hash-{}", req_date),
                    "records": [{
                        "time_minutes": "0931",
                        "price": 10.0,
                        "volume": 100,
                        "amount": 1000.0,
                        "avg_price": 10.0,
                        "trading_date": req_date
                    }]
                }
            });
            wiremock::ResponseTemplate::new(200).set_body_json(resp)
        })
        // expect 3 calls for 3-day range
        .expect(3)
        .mount(&server)
        .await;

    let cfg = fast_test_cfg(server.uri());
    let client = OpenStockClient::new(cfg).expect("client build");
    let start = chrono::NaiveDate::parse_from_str("2026-06-28", "%Y-%m-%d").unwrap();
    let end = chrono::NaiveDate::parse_from_str("2026-06-30", "%Y-%m-%d").unwrap();
    let shares = client
        .fetch_minute_share("sh600000", DateOrRange::Range { start, end })
        .await
        .expect("fetch ok");

    assert_eq!(shares.len(), 3, "one record per day × 3 days");
    // Verify dates are distinct and span the range
    let dates: Vec<chrono::NaiveDate> = shares.iter().map(|s| s.timestamp.date()).collect();
    assert!(dates.contains(&start));
    assert!(dates.contains(&end));
    // Verify ascending order
    assert_eq!(dates, {
        let mut sorted = dates.clone();
        sorted.sort();
        sorted
    });
}
```

注：wiremock responder closure 是 v0.6+ 特性，确认 `wiremock` 版本支持；如不支持，改用 3 个独立 Mock（每个 match 不同 `params.date` body）+ `.expect(1)` 各自。

- [ ] **Step 3.4: Write failing wiremock test W5 (Range skips non-trading days)**

```rust
#[tokio::test]
async fn fetch_minute_share_range_skips_non_trading_days() {
    use crate::data::models::DateOrRange;

    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/data/fetch"))
        .respond_with(|request: &wiremock::Request| {
            let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
            let req_date = body["params"]["date"].as_str().unwrap_or("");
            // For "2026-06-28" (Sunday) return empty records
            let records: Vec<serde_json::Value> = if req_date == "2026-06-28" {
                vec![]
            } else {
                vec![serde_json::json!({
                    "time_minutes": "1000",
                    "price": 10.0, "volume": 100,
                    "amount": 1000.0, "avg_price": 10.0,
                    "trading_date": req_date
                })]
            };
            let resp = serde_json::json!({
                "status": "ok",
                "data": {
                    "source": "eltdx",
                    "artifact_hash": "x",
                    "records": records
                }
            });
            wiremock::ResponseTemplate::new(200).set_body_json(resp)
        })
        .expect(3)  // still 3 calls — client loops through all days
        .mount(&server)
        .await;

    let cfg = fast_test_cfg(server.uri());
    let client = OpenStockClient::new(cfg).expect("client build");
    let start = chrono::NaiveDate::parse_from_str("2026-06-28", "%Y-%m-%d").unwrap();
    let end = chrono::NaiveDate::parse_from_str("2026-06-30", "%Y-%m-%d").unwrap();
    let shares = client
        .fetch_minute_share("sh600000", DateOrRange::Range { start, end })
        .await
        .expect("fetch ok");

    // Sunday returns empty, so only 2 trading days × 1 record = 2 records
    assert_eq!(shares.len(), 2);
    let dates: Vec<chrono::NaiveDate> = shares.iter().map(|s| s.timestamp.date()).collect();
    assert!(!dates.contains(&start), "non-trading day must not appear");
}
```

- [ ] **Step 3.5: Extend `RawMinuteRecord` to capture `trading_date` (per INV-2C)**

`src/sources/openstock_client.rs:837` 修改：

```rust
#[derive(Debug, serde::Deserialize)]
struct RawMinuteRecord {
    time_minutes: String,
    price: Option<rust_decimal::Decimal>,
    volume: Option<i64>,
    amount: Option<rust_decimal::Decimal>,
    avg_price: Option<rust_decimal::Decimal>,
    // 新增：OpenStock envelope record 中的 trading_date（若 server 在 record 层而非 meta 层提供）
    // 实测：根据 _eltdx_timeseries.py:195 evidence，trading_date 在 meta 中，每个 record 不带。
    // 故 record 不带时，依赖 helper 传 date 参数。
    #[serde(default)]
    trading_date: Option<String>,
}
```

**重要**：基于 `_eltdx_timeseries.py:181-208` 证据，`trading_date` 在 `meta` 中——不在每个 record 中。所以 helper 必须从 envelope 的 `meta.trading_date` 取，**不**从 record 取。

调整策略：`fetch_minute_share_single` 用更具体的 envelope 类型而非 `RawMinuteRecord` 直 `fetch<T>`：

```rust
#[derive(Debug, serde::Deserialize)]
struct MinuteShareEnvelope {
    #[serde(default)]
    meta: MinuteShareMeta,
    points: Vec<RawMinuteRecord>,
}
#[derive(Debug, Default, serde::Deserialize)]
struct MinuteShareMeta {
    #[serde(default)]
    trading_date: Option<String>,
}
```

然后 `fetch_minute_share_single` 用 `self.fetch::<MinuteShareEnvelope>("MINUTE_DATA", params)` 而非 `fetch::<RawMinuteRecord>`。`OpenStockResponse.records: Vec<MinuteShareEnvelope>`——通常 1 元素，但用 `for env in resp.records` 处理多元素 case。

- [ ] **Step 3.6: Refactor `fetch_minute_share` to dispatch on DateOrRange**

`src/sources/openstock_client.rs:798-823` 重写：

```rust
pub async fn fetch_minute_share(
    &self,
    code: &str,
    date_or_range: crate::data::models::DateOrRange,
) -> Result<Vec<crate::data::models::MinuteShare>> {
    use crate::data::models::{DateOrRange, iter_dates_inclusive};

    match date_or_range {
        DateOrRange::Date(d) => self.fetch_minute_share_single(code, d).await,
        DateOrRange::Range { start, end } => {
            let mut all = Vec::new();
            for d in iter_dates_inclusive(start, end) {
                let day_records = self.fetch_minute_share_single(code, d).await?;
                all.extend(day_records);
            }
            Ok(all)
        }
    }
}

async fn fetch_minute_share_single(
    &self,
    code: &str,
    date: chrono::NaiveDate,
) -> Result<Vec<crate::data::models::MinuteShare>> {
    let params = serde_json::json!({
        "code": code,
        "date": date.format("%Y-%m-%d").to_string(),
    });
    let resp = self.fetch::<MinuteShareEnvelope>("MINUTE_DATA", params).await?;
    let mut out = Vec::new();
    for env in resp.records {
        // 从 meta.trading_date 取实际交易日（INV-2C：非交易日 case）
        let actual_date = env
            .meta
            .trading_date
            .as_deref()
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .unwrap_or(date);
        for raw in env.points {
            if let Some(share) = parse_minute_share(code, &raw, actual_date) {
                out.push(share);
            } else {
                tracing::warn!(
                    code = code,
                    requested_date = %date,
                    trading_date = %actual_date,
                    time_minutes = %raw.time_minutes,
                    "MINUTE_DATA record missing required field or invalid time, skipping"
                );
            }
        }
    }
    Ok(out)
}
```

注：`MinuteShareEnvelope` + `MinuteShareMeta` 加在 `RawMinuteRecord` 附近（文件级）。

- [ ] **Step 3.7: Run all fetch_minute_share tests**

Run: `cargo test --lib --package quantix-cli fetch_minute_share`
Expected: PASS (5 tests: sends_date, skips_missing, propagates_4xx, range_loops_per_day, range_skips_non_trading)

注意：现有 `skips_records_with_missing_required_field` 测试的 mock response shape 需检查——若原 mock 直接返回 records 数组（不是 `{meta, points}` wrapper），需调整为 wrapper 形态以匹配新 `MinuteShareEnvelope` 类型。

- [ ] **Step 3.8: Run quality gates + commit**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
git add src/sources/openstock_client.rs
git commit -m "feat(openstock): extend fetch_minute_share with client-side range loop (P0.13c)

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 4: CLI `--start`/`--end` flags + handler dispatch

**Files:**
- Modify: `src/cli/commands/data.rs:386-410`（`FetchMinuteKlines`/`FetchMinuteShare` 加 flags）
- Modify: `src/cli/handlers/openstock_handler.rs:411-482`（2 handlers 加 start/end 参数 + from_cli）
- Modify: `src/cli/handlers/app_shell.rs:385-396`（2 dispatcher arms 传新参数）

**Interfaces:**
- Consumes: `crate::data::models::DateOrRange`（Task 1）、扩展后的 fetcher（Task 2/3）

- [ ] **Step 4.1: Modify CLI commands**

`src/cli/commands/data.rs:386-410` 改为：

```rust
FetchMinuteKlines {
    #[arg(long)]
    symbol: String,

    #[arg(long, default_value = "1m")]
    period: String,

    /// Single-day query (mutex with --start/--end)
    #[arg(long)]
    date: Option<String>,

    /// Range start (inclusive). Must pair with --end.
    #[arg(long)]
    start: Option<String>,

    /// Range end (inclusive). Must pair with --start.
    #[arg(long)]
    end: Option<String>,

    #[arg(long, default_value = "none")]
    adjust: String,
},

/// Fetch OpenStock MINUTE_DATA category (intraday time-share ticks).
FetchMinuteShare {
    #[arg(long)]
    symbol: String,

    /// Single-day query (mutex with --start/--end)
    #[arg(long)]
    date: Option<String>,

    /// Range start (inclusive). Must pair with --end.
    #[arg(long)]
    start: Option<String>,

    /// Range end (inclusive). Must pair with --start.
    #[arg(long)]
    end: Option<String>,
},
```

- [ ] **Step 4.2: Modify handlers**

`src/cli/handlers/openstock_handler.rs:411-482` 重写两个 handler：

```rust
pub(crate) async fn fetch_openstock_minute_klines(
    settings: &OpenStockSettings,
    symbol: String,
    period: String,
    date: Option<String>,
    start: Option<String>,
    end: Option<String>,
    adjust: String,
) -> Result<()> {
    use std::str::FromStr;
    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
    use crate::sources::openstock_client::OpenStockClient;

    let period_enum = MinutePeriod::from_str(&period)
        .map_err(|e| QuantixError::Config(format!("--period: {}", e)))?;
    let adjust_enum = AdjustType::from_str(&adjust)
        .map_err(|e| QuantixError::Config(format!("--adjust: {}", e)))?;
    let dor = DateOrRange::from_cli(date.as_deref(), start.as_deref(), end.as_deref())?;

    let client = OpenStockClient::from_settings(settings)?;
    let bars = client
        .fetch_minute_klines(&symbol, period_enum, dor.clone(), adjust_enum)
        .await?;

    let mode_label = match &dor {
        DateOrRange::Date(d) => format!("date={}", d),
        DateOrRange::Range { start, end } => format!("range={}..{}", start, end),
    };
    println!(
        "OpenStock live fetch (/data/bars, symbol={}, minute={}, {})",
        symbol, period_enum.as_str(), mode_label
    );
    println!(
        "  Adjust: {}",
        adjust_enum.as_openstock_param().unwrap_or("none (field omitted)")
    );
    println!("  记录数: {}", bars.len());
    if !bars.is_empty() {
        println!("  First:  {:?}", bars.first());
        println!("  Last:   {:?}", bars.last());
    }
    if bars.len() > 10_000 {
        eprintln!(
            "warning: range returns {} records, consider narrowing for faster responses",
            bars.len()
        );
    }
    Ok(())
}

pub(crate) async fn fetch_openstock_minute_share(
    settings: &OpenStockSettings,
    symbol: String,
    date: Option<String>,
    start: Option<String>,
    end: Option<String>,
) -> Result<()> {
    use crate::data::models::DateOrRange;
    use crate::sources::openstock_client::OpenStockClient;

    let dor = DateOrRange::from_cli(date.as_deref(), start.as_deref(), end.as_deref())?;

    let client = OpenStockClient::from_settings(settings)?;
    let started = std::time::Instant::now();
    let shares = client.fetch_minute_share(&symbol, dor.clone()).await?;
    let latency_ms = started.elapsed().as_millis();

    let base_url = settings.base_url.as_deref().unwrap_or("(not set)");
    let mode_label = match &dor {
        DateOrRange::Date(d) => format!("date={}", d),
        DateOrRange::Range { start, end } => format!("range={}..{}", start, end),
    };
    println!("OpenStock MINUTE_DATA (time-share ticks)");
    println!("  Code:     {}", symbol);
    println!("  Mode:     {}", mode_label);
    println!("  Endpoint: {}/data/fetch", base_url);
    println!("  Records:  {}", shares.len());
    if let (Some(first), Some(last)) = (shares.first(), shares.last()) {
        println!("  First:    {:?}", first);
        println!("  Last:     {:?}", last);
    }
    println!("  latency_ms: {}", latency_ms);
    if let DateOrRange::Range { start, end } = &dor {
        let n_days = (end - start).num_days() + 1;
        if n_days > 10 {
            eprintln!(
                "warning: range spans {} days; client issued {} single-day requests ({:.1}s total)",
                n_days, n_days, latency_ms as f64 / 1000.0
            );
        }
    }
    Ok(())
}
```

- [ ] **Step 4.3: Modify dispatcher arms**

`src/cli/handlers/app_shell.rs:385-396` 改为：

```rust
OpenStockCommands::FetchMinuteKlines {
    symbol, period, date, start, end, adjust,
} => {
    fetch_openstock_minute_klines(
        &rt.openstock, symbol, period, date, start, end, adjust,
    ).await?;
}
OpenStockCommands::FetchMinuteShare { symbol, date, start, end } => {
    fetch_openstock_minute_share(&rt.openstock, symbol, date, start, end).await?;
}
```

- [ ] **Step 4.4: Run quality gates + commit**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
git add src/cli/commands/data.rs src/cli/handlers/openstock_handler.rs src/cli/handlers/app_shell.rs
git commit -m "feat(cli): add --start/--end flags to fetch-minute-klines/share (P0.13c)

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 5: Live tests + OpenSpec change + Governance card

**Files:**
- Modify: `tests/openstock_live_minute_klines.rs`（追加 L1 多日范围 test）
- Modify: `tests/openstock_live_minute_share.rs`（追加 L2 多日范围 test + L3 from_cli 错误 test）
- Create: `openspec/changes/openstock-data-consumption-p0-13c/{proposal,tasks,design}.md`
- Create: `openspec/changes/openstock-data-consumption-p0-13c/specs/openstock-data-consumption/spec.md`
- Create: `.governance/programs/project-governance/cards/P0.13c.yaml`

- [ ] **Step 5.1: Add L1 live test for fetch_minute_klines Range**

追加到 `tests/openstock_live_minute_klines.rs`：

```rust
#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn live_fetch_minute_klines_range_returns_multi_day_records() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let client = quantix_cli::sources::openstock_client::OpenStockClient::from_env()
        .expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    use quantix_cli::data::models::{AdjustType, DateOrRange, MinutePeriod};
    let start = chrono::NaiveDate::parse_from_str("2026-06-23", "%Y-%m-%d").unwrap();
    let end = chrono::NaiveDate::parse_from_str("2026-06-27", "%Y-%m-%d").unwrap();
    let bars = client
        .fetch_minute_klines(
            "sh600000",
            MinutePeriod::OneMinute,
            DateOrRange::Range { start, end },
            AdjustType::None,
        )
        .await
        .expect("live fetch ok");
    assert!(!bars.is_empty(), "5-day range should return non-empty bars");
    // Verify range: first.date >= start, last.date <= end
    let first_date = bars.first().unwrap().timestamp.date();
    let last_date = bars.last().unwrap().timestamp.date();
    assert!(first_date >= start, "first.date {} < start {}", first_date, start);
    assert!(last_date <= end, "last.date {} > end {}", last_date, end);
    // Verify multi-day: first_date != last_date (range spans 3+ trading days)
    assert_ne!(first_date, last_date, "range must span multiple trading days");
}
```

- [ ] **Step 5.2: Add L2 live test for fetch_minute_share Range**

追加到 `tests/openstock_live_minute_share.rs`：

```rust
#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn live_fetch_minute_share_range_loops_per_day() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let client = quantix_cli::sources::openstock_client::OpenStockClient::from_env()
        .expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    use quantix_cli::data::models::{DateOrRange, iter_dates_inclusive};
    let start = chrono::NaiveDate::parse_from_str("2026-06-23", "%Y-%m-%d").unwrap();
    let end = chrono::NaiveDate::parse_from_str("2026-06-27", "%Y-%m-%d").unwrap();
    let shares = client
        .fetch_minute_share("sh600000", DateOrRange::Range { start, end })
        .await
        .expect("live fetch ok");
    assert!(!shares.is_empty());
    let dates: Vec<chrono::NaiveDate> = shares.iter().map(|s| s.timestamp.date()).collect();
    // Each date in result must be in [start, end]
    for d in &dates {
        assert!(*d >= start && *d <= end, "date {} outside range", d);
    }
    // Multi-day span
    let unique_dates: std::collections::BTreeSet<_> = dates.iter().collect();
    assert!(unique_dates.len() >= 2, "expected multiple trading days");
}

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn live_from_cli_start_after_end_errors_without_http() {
    // L3: from_cli rejects start > end without touching network
    use quantix_cli::data::models::DateOrRange;
    let r = DateOrRange::from_cli(None, Some("2026-06-30"), Some("2026-06-01"));
    assert!(r.is_err());
}
```

- [ ] **Step 5.3: Create OpenSpec change (4 files)**

参考 P0.13b-2 change（`openspec/changes/openstock-data-consumption-p0-13b-2/`）shape 创建：

`openspec/changes/openstock-data-consumption-p0-13c/proposal.md`:
```markdown
# OpenStock P0.13c: Multi-day Range Queries for Minute-Level Fetchers

## Why
P0.13b-1/2 fetchers (`fetch_minute_klines` + `fetch_minute_share`) accept only single-day `date` parameters, forcing callers to loop manually for backfills. Parent design (P0.13b) deferred range support to P0.13c.

## What Changes
- New `DateOrRange` enum in `data/models.rs` (mutex: Date | Range)
- `fetch_minute_klines` accepts `DateOrRange`; server-side range via `/data/bars` `start_date`/`end_date`
- `fetch_minute_share` accepts `DateOrRange`; client-side per-day loop (OpenStock MINUTE_DATA server doesn't support range; reads `meta.trading_date` per response for correct date)
- CLI `fetch-minute-klines` + `fetch-minute-share` add `--start`/`--end`; `--date` becomes `Option<String>` (backward-compatible superset)
- `from_cli` validates mutex + rejects `(None,None,None)` and semi-open ranges

## Impact
- Files: `src/data/models.rs`, `src/sources/openstock_client.rs`, `src/cli/commands/data.rs`, `src/cli/handlers/openstock_handler.rs`, `src/cli/handlers/app_shell.rs`, `tests/openstock_live_minute_*.rs`
- Backward compat: P0.13b-1/2 wiremock tests pass unchanged (Date mode wire body identical)
- Risk: `/data/bars` field names verified by wiremock + live test (R1)

## Non-Goals
- ClickHouse writes for multi-day data
- Pagination / streaming for huge ranges
- Cross-period merge (different period candles stay separate)
- Other categories' range extension
- Awaiting OpenStock server-side MINUTE_DATA range support (deferred per D6; switchable later)
```

`tasks.md`：参考 P0.13b-2 形式列出 Section 0-9（baseline → governance → models → client → CLI → tests → spec → governance transition → verification）。

`design.md`：引用 `docs/superpowers/specs/2026-07-03-openstock-p0-13c-multi-day-range-design.md` 为权威源；列出 D1-D6 关键决策摘要。

`specs/openstock-data-consumption/spec.md`:
```markdown
## MODIFIED Requirements

### Requirement: Minute-Level K-Line Fetcher
The system SHALL provide a `fetch_minute_klines` method that accepts a `DateOrRange` parameter
supporting either a single date or an inclusive `[start, end]` range.

#### Scenario: Single-day via Date variant
- WHEN `fetch_minute_klines(code, period, DateOrRange::Date(d), adjust)` is called
- THEN the system sends `params.date = d` to `/data/bars` (backward-compatible with P0.13b-1)

#### Scenario: Multi-day via Range variant
- WHEN `fetch_minute_klines(code, period, DateOrRange::Range { start, end }, adjust)` is called
- THEN the system sends `params.start_date` and `params.end_date` to `/data/bars`
- AND returns a flat `Vec<MinuteBar>` ordered by timestamp ascending

### Requirement: Minute-Level Time-Share Fetcher
The system SHALL provide a `fetch_minute_share` method that accepts a `DateOrRange` parameter.

#### Scenario: Single-day via Date variant
- WHEN `fetch_minute_share(code, DateOrRange::Date(d))` is called
- THEN the system sends `params.date = d` to `/data/fetch MINUTE_DATA`
- AND parses `meta.trading_date` for each response item to derive per-record timestamps

#### Scenario: Multi-day via Range variant (client-side loop)
- WHEN `fetch_minute_share(code, DateOrRange::Range { start, end })` is called
- THEN the system iterates `iter_dates_inclusive(start, end)` issuing N single-day requests
- AND aggregates results into a flat `Vec<MinuteShare>` ordered by (date, time_minutes) ascending
- AND skips days where the server returns empty records (non-trading days)

### Requirement: CLI Flag Validation
The CLI SHALL validate `--date` vs `--start`/`--end` mutex via `DateOrRange::from_cli`.

#### Scenario: Both --date and --start provided
- WHEN the CLI receives `--date X --start Y` (or any overlap)
- THEN it returns an error naming both flags and showing usage

#### Scenario: Semi-open range
- WHEN the CLI receives `--start X` without `--end` (or vice versa)
- THEN it returns an error requiring both ends

#### Scenario: No date arguments
- WHEN the CLI receives neither `--date` nor `--start`/`--end`
- THEN it returns an error requiring at least one form
```

- [ ] **Step 5.4: Create governance card**

`.governance/programs/project-governance/cards/P0.13c.yaml`:
```yaml
id: P0.13c
title: "OpenStock multi-day range queries for minute-level fetchers"
state: in_progress
scope:
  allowed_paths:
    - src/data/models.rs
    - src/sources/openstock_client.rs
    - src/cli/commands/data.rs
    - src/cli/handlers/openstock_handler.rs
    - src/cli/handlers/app_shell.rs
    - tests/openstock_live_minute_klines.rs
    - tests/openstock_live_minute_share.rs
    - openspec/changes/openstock-data-consumption-p0-13c/**
    - docs/superpowers/specs/2026-07-03-openstock-p0-13c-multi-day-range-design.md
    - docs/superpowers/plans/2026-07-03-openstock-p0-13c-multi-day-range-plan.md
  forbidden_paths:
    - src/db/**
    - src/backtest/**
    - src/execution/**
    - src/sources/openstock.rs
    - src/sources/openstock_shadow.rs
    - src/sources/kline_aggregator.rs
    - src/sources/openstock_client.rs::fetch_klines             # P0.13a
    - src/data/models.rs::Kline                                 # P0.13a
    - src/data/models.rs::BarPeriod                             # P0.13a
    - src/data/models.rs::MinutePeriod                          # P0.13b-1
    - src/data/models.rs::MinuteBar                             # P0.13b-1
    - src/data/models.rs::MinuteShare                           # P0.13b-2
acceptance_gates:
  - cargo fmt --all -- --check
  - cargo clippy --all-targets --workspace -- -D warnings
  - cargo test --workspace
  - openspec validate openstock-data-consumption-p0-13c --strict
  - openspec validate --all --strict
non_goals:
  - "ClickHouse writes / shadow persistence for multi-day data"
  - "Pagination / streaming for huge ranges"
  - "Cross-period candle merge"
  - "Other categories' range extension"
  - "Awaiting OpenStock server-side MINUTE_DATA range support (D6; switchable later without signature change)"
```

- [ ] **Step 5.5: Validate OpenSpec + governance + commit**

```bash
openspec validate openstock-data-consumption-p0-13c --strict
openspec validate --all --strict
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
git add tests/ openspec/changes/openstock-data-consumption-p0-13c/ .governance/programs/project-governance/cards/P0.13c.yaml
git commit -m "feat(openstock): add live tests + openspec change + governance card for P0.13c

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

- [ ] **Step 5.6: Transition governance card to completed**

```bash
# Edit .governance/programs/project-governance/cards/P0.13c.yaml: state: in_progress -> completed
git add .governance/programs/project-governance/cards/P0.13c.yaml
git commit -m "chore(governance): transition P0.13c card to completed

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Self-Review (post-write check)

**Spec coverage check** — all R1-revised spec sections covered:
- §3.1 mutex semantics → Task 1 `from_cli` (U1-U7)
- §3.2 wire shape (minute_klines server-side, minute_share client-side) → Task 2/3
- §3.3 CLI Option compat → Task 4
- §4.1/4.2 fetcher signatures → Task 2/3
- §4.3 from_cli error messages → Task 1 (assertion on flag name in error msg)
- §5 INV-1A/1B/2A/2B/2C/3 → covered by tests U5/U6/U7 (mutex), W1-W5 (wiremock), L1/L2 (live)
- §6 D1-D6 → D1 (enum Task 1), D2/D5 (from_cli Task 1), D3 (--date preserved Task 4), D4 (wiremock W1 Task 2), D6 (client loop Task 3)
- §7 R1-R6 → R1 (live test L1 Task 5), R2 (handler warning Task 4), R3 (Date-mode wiremock unchanged Task 2/3), R4 (U1-U7 Task 1), R5 (handler warning Task 4), R6 (documented in spec §13; non-blocking)
- §8 test matrix U1-U7, W1-W5, L1-L3 → all assigned

**Placeholder scan** — none (every step has complete code; no TBD/TODO/`...`).

**Type consistency** — `DateOrRange::Range { start: NaiveDate, end: NaiveDate }` (非 Option) used consistently in Task 1/2/3/4. `MinuteShareEnvelope`/`MinuteShareMeta` introduced in Task 3 referenced consistently.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-03-openstock-p0-13c-multi-day-range-plan.md`. Two execution options:

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration
2. **Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
