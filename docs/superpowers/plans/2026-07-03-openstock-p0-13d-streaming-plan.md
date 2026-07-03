# OpenStock P0.13d Streaming Fetch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `fetch_minute_klines_stream` / `fetch_minute_share_stream` (`impl Stream<Item = Result<Vec<T>, QuantixError>>`) alongside existing batch APIs and a `--stream` CLI flag, so large date ranges can be processed batch-by-batch instead of accumulated in memory.

**Architecture:** Side-by-side API (D6) — existing `fetch_minute_klines` / `fetch_minute_share` are unchanged in signature and wire shape. A new private helper `fetch_minute_klines_range(start, end)` is extracted from the body of `fetch_minute_klines` and reused by both the batch and stream entry points. Stream is built with `futures::stream::iter(chunks).then(async move { ... })`. The CLI adds a `--stream` flag (default `false`) that, when set, switches the handler to a `while let Some(batch) = s.next().await` loop with per-batch progress to stderr.

**Tech Stack:** Rust + tokio + reqwest + `futures = "0.3"` (already in `Cargo.toml:38`) + `futures-util = "0.3"` (already in `Cargo.toml:39`) + wiremock (dev) + chrono.

**Spec:** `docs/superpowers/specs/2026-07-03-openstock-p0-13d-streaming-design.md` (commit `21484df`)

---

## Global Constraints

Copied verbatim from spec §3, §4.2, §4.3, §4.4, §5:

- **Side-by-side (D6)**: existing `fetch_minute_klines` / `fetch_minute_share` (Vec API) signatures and wire shapes are **completely unchanged**. P0.13a/b/c tests pass zero-modified (INV-4A).
- **No new public types**: `MinuteBar` / `MinuteShare` / `MinutePeriod` / `DateOrRange` / `AdjustType` unchanged (INV-3).
- **Stream shape (D1)**: `impl futures::Stream<Item = Result<Vec<MinuteBar>, QuantixError>> + '_` (and same for `MinuteShare`). No `Send` bound. No `Pin<Box<dyn Stream>>`.
- **Weekly chunks (D2)**: `chunk_range_weekly(start, end) -> Vec<(NaiveDate, NaiveDate)>` returns contiguous segments each ≤ 7 days (inclusive), covering `[start..=end]` with no gaps, no overlaps (INV-1B/1C). Not bound to `chrono::Weekday`.
- **Per-day share (D3)**: each calendar day in `[start..=end]` (including non-trading days) yields one `Vec<MinuteShare>`; non-trading days yield `vec![]` (INV-5B). Reuses P0.13c `fetch_minute_share_single`.
- **First-error-terminates (D4)**: on the first `Err` from a batch, the stream yields it, then returns `None` on subsequent `next()` (INV-5A).
- **Inherit retry/breaker (D5)**: klines path = direct `self.http.post()` (no retry, same as P0.13c); share path = `self.fetch::<MinuteShareEnvelope>()` (with retry+breaker, same as P0.13c).
- **Wire shape (INV-2A)**: in `fetch_minute_klines_range`, when `start == end` body has only `date`; when `start != end` body has only `start_date` + `end_date`. Same rule as P0.13c.
- **CLI defaults**: `--period` `default_value = "1m"`, `--adjust` `default_value = "none"`, `--stream` `default_value_t = false`. Existing `--date` / `--start` / `--end` semantics unchanged (INV-4B).
- **No new deps**: `futures` and `futures-util` already in workspace.
- **File sizes** (per CLAUDE.md coding standards): `openstock_client.rs` is currently ~1900 lines; spec adds ~180 → still under the 800-line module warning threshold? No — but the file is already over the limit (existing tech debt acknowledged in P0.13a/b/c); this slice follows the same pattern of additive growth rather than splitting mid-slice. `openstock_handler.rs` likewise additive.
- **Error handling**: no `.unwrap()` / `.expect()` / `panic!()` in production code (use `?` or `map_err`). No `println!` in library modules (handler-layer `println!`/`eprintln!` is fine).
- **Commit format**: `<type>(<scope>): <subject>` with `Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>` trailer.
- **Commits per task**: one commit per task, semantic intent only.

---

## File Structure

**Modified files (additive):**

1. `src/sources/openstock_client.rs` — extract `fetch_minute_klines_range` private helper from existing `fetch_minute_klines` body; rewrite `fetch_minute_klines` as a thin dispatcher; add `chunk_range_weekly` free function (private to module); add `fetch_minute_klines_stream` + `fetch_minute_share_stream` public methods; add 7 unit tests (S1-S7). Add `use futures::Stream` + `use futures::stream::{StreamExt, iter as stream_iter}` at top of file (or inline at method scope — implementer's choice based on existing style).

2. `src/cli/commands/data.rs` — add `stream: bool` field (with `#[arg(long, default_value_t = false)]`) to `FetchMinuteKlines` and `FetchMinuteShare` enum variants. Other fields unchanged.

3. `src/cli/handlers/openstock_handler.rs` — add `stream: bool` parameter to `fetch_openstock_minute_klines` and `fetch_openstock_minute_share`; add streaming branch in each; update batch-path warning text to mention `--stream`.

4. `src/cli/handlers/app_shell.rs` — destructure `stream` in the two dispatcher match arms; pass through to handler.

5. `tests/openstock_live_minute_klines.rs` — append L1 live test `live_fetch_minute_klines_stream_multi_week_range`.

6. `tests/openstock_live_minute_share.rs` — append L2 live test `live_fetch_minute_share_stream_one_day_per_batch`.

**New files:**

7. `openspec/changes/openstock-data-consumption-p0-13d/proposal.md`
8. `openspec/changes/openstock-data-consumption-p0-13d/tasks.md`
9. `openspec/changes/openstock-data-consumption-p0-13d/design.md`
10. `openspec/changes/openstock-data-consumption-p0-13d/specs/openstock-data-consumption/spec.md`
11. `.governance/programs/project-governance/cards/P0.13d.yaml`

---

## Task 0: Baseline and Governance

**Files:**
- Create: `openspec/changes/openstock-data-consumption-p0-13d/proposal.md`
- Create: `openspec/changes/openstock-data-consumption-p0-13d/tasks.md`
- Create: `openspec/changes/openstock-data-consumption-p0-13d/design.md`
- Create: `openspec/changes/openstock-data-consumption-p0-13d/specs/openstock-data-consumption/spec.md`
- Create: `.governance/programs/project-governance/cards/P0.13d.yaml`

**Interfaces:**
- Produces: OpenSpec change `openstock-data-consumption-p0-13d` with 4-file structure mirroring `openstock-data-consumption-p0-13c/`. Governance card `P0.13d.yaml` scoped to `openspec/changes/openstock-data-consumption-p0-13d/*` + `src/sources/openstock_client.rs` + `src/cli/{commands/data.rs,handlers/openstock_handler.rs,handlers/app_shell.rs}` + `tests/openstock_live_minute_{klines,share}.rs`.

- [ ] **Step 1: Inspect P0.13c OpenSpec shape as template**

Run:
```bash
ls openspec/changes/openstock-data-consumption-p0-13c/
cat openspec/changes/openstock-data-consumption-p0-13c/proposal.md
```
Read the structure of all 4 P0.13c OpenSpec files. P0.13d follows the same shape: `proposal.md` (`## Why` / `## What Changes` / `## Impact` / `## Non-Goals`), `tasks.md` (numbered sections `## 0. Baseline And Governance` through `## 9. Verification`), `design.md` (D1-D6 decisions; R1-R4 risks), `specs/openstock-data-consumption/spec.md` (`## ADDED Requirements` with scenarios).

- [ ] **Step 2: Write `proposal.md`**

Content:
```markdown
# OpenStock Data Consumption P0.13d — Streaming Fetch

## Why

P0.13c's batch API accumulates an entire range into one `Vec<MinuteBar>` / `Vec<MinuteShare>`. A 6-month 1m kline range ≈ 36,000 records held in memory at once; CLI handlers further fan this out by printing every record. There is no server-side pagination primitive in OpenStock (`fetching.py:232` `execute_bars_payload` has no `limit/offset/cursor`). P0.13d adds client-side range chunking and a streaming API so callers can process large ranges batch-by-batch.

## What Changes

- New `OpenStockClient::fetch_minute_klines_stream` returning `impl Stream<Item = Result<Vec<MinuteBar>, QuantixError>>` (weekly chunks)
- New `OpenStockClient::fetch_minute_share_stream` returning `impl Stream<Item = Result<Vec<MinuteShare>, QuantixError>>` (daily batches; non-trading days yield empty Vec)
- Private helper `fetch_minute_klines_range(start, end)` extracted from existing `fetch_minute_klines` body
- Private pure function `chunk_range_weekly(start, end) -> Vec<(NaiveDate, NaiveDate)>`
- CLI `--stream` flag (default `false`) on `fetch-minute-klines` and `fetch-minute-share`; prints per-batch progress to stderr when set
- Existing batch APIs (`fetch_minute_klines` / `fetch_minute_share`) unchanged in signature, wire shape, and behavior

## Impact

- `src/sources/openstock_client.rs`: +180 lines (2 stream methods, 1 helper extract, 1 chunk fn, 7 unit tests)
- `src/cli/commands/data.rs`: +6 lines (2 × `stream: bool` field)
- `src/cli/handlers/openstock_handler.rs`: +50 lines (2 streaming branches + warning text)
- `src/cli/handlers/app_shell.rs`: +6 lines (destructure `stream` in 2 arms)
- `tests/openstock_live_minute_klines.rs`: +60 lines (1 live test)
- `tests/openstock_live_minute_share.rs`: +50 lines (1 live test)
- OpenSpec change (4 files) + governance card: +230 lines

## Non-Goals

- Server-side pagination protocol (none exists; this slice does not invent one)
- ClickHouse batch inserts from stream (separate slice)
- Backpressure / flow control (`Vec<T>` per batch is the natural unit)
- Refactor batch API to `stream.collect` (D6 rejected — causes P0.13a/b/c churn)
- Streaming for other fetchers (daily klines, index, etc.) — separate slice
- klines retry (R4 follow-up; candidate for P0.13e)
- P0.13c §13 R6 (server-side MINUTE_DATA range) — blocked on OpenStock server
```

- [ ] **Step 3: Write `tasks.md`**

Content: a markdown numbered list of sections `## 0. Baseline And Governance` through `## 9. Verification`, each section listing the work in the corresponding task in this plan. Mirror the section headings used in `openspec/changes/openstock-data-consumption-p0-13c/tasks.md`. Under each section write 2-5 bullet points; under `## 0.` include creation of `P0.13d.yaml` governance card and an `ft:new-node` placeholder (TBD if governance CLI not available — note "manual yaml placement" if so).

- [ ] **Step 4: Write `design.md`**

Content: copy the design rationale from spec §6 (D1-D6 verbatim) and §7 (R1-R4 risks verbatim, with the R1-revision note). Add a `## Alternatives Considered` section that points to spec §6 (single source of truth).

- [ ] **Step 5: Write `specs/openstock-data-consumption/spec.md`**

Content: a `## ADDED Requirements` section with requirement blocks for:
  - `REQ-STREAM-001`: streaming API for minute klines (scenarios: weekly chunking, single-day compression, error terminates stream)
  - `REQ-STREAM-002`: streaming API for minute share (scenarios: per-day batch, non-trading-day empty Vec, error terminates stream)
  - `REQ-STREAM-003`: CLI `--stream` flag (scenarios: default false, batch path unchanged when flag absent, streaming path emits per-batch progress when set)
  - `REQ-STREAM-004`: existing batch API backward compatibility (scenarios: signature unchanged, wire shape unchanged, all P0.13a/b/c tests pass zero-modified)

Each requirement has 1-3 `Scenario:` blocks in OpenSpec Gherkin-style.

- [ ] **Step 6: Write `.governance/programs/project-governance/cards/P0.13d.yaml`**

Content (mirror P0.13c.yaml structure; if no P0.13c.yaml exists, mirror the most recent P0.13 card and adjust):
```yaml
id: P0.13d
title: "OpenStock streaming fetch for minute-level data"
status: in_progress
scope:
  - openspec/changes/openstock-data-consumption-p0-13d/*
  - src/sources/openstock_client.rs
  - src/cli/commands/data.rs
  - src/cli/handlers/openstock_handler.rs
  - src/cli/handlers/app_shell.rs
  - tests/openstock_live_minute_klines.rs
  - tests/openstock_live_minute_share.rs
linked_openspec: openstock-data-consumption-p0-13d
started: "2026-07-03"
```

If the existing card format has additional fields (e.g. `milestone:`, `depends_on:`), copy and adapt them. Verify by `cat .governance/programs/project-governance/cards/P0.13c.yaml 2>/dev/null || ls .governance/programs/project-governance/cards/ | tail -5` first.

- [ ] **Step 7: Validate OpenSpec + governance**

Run:
```bash
openspec validate openstock-data-consumption-p0-13d --strict
openspec validate --all --strict
```
Expected: both exit 0 with no warnings. If `openspec` is not installed, run `npx openspec validate ...` (or note "manual review" if neither works).

- [ ] **Step 8: Commit**

```bash
git add openspec/changes/openstock-data-consumption-p0-13d/ .governance/programs/project-governance/cards/P0.13d.yaml
git commit -m "$(cat <<'EOF'
chore(openspec): scaffold openstock p0.13d governance + openspec change

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 1: Add `chunk_range_weekly` Pure Function + 4 Unit Tests (S1-S4)

**Files:**
- Modify: `src/sources/openstock_client.rs` (add `chunk_range_weekly` as a private free function inside the module, near the existing `iter_dates_inclusive` use site at line 814 or near other private helpers; pick the location closest to where `fetch_minute_klines` lives)
- Modify: `src/data/models.rs` (alternative location — see Step 1 decision below)

**Interfaces:**
- Produces: private free function `chunk_range_weekly(start: chrono::NaiveDate, end: chrono::NaiveDate) -> Vec<(chrono::NaiveDate, chrono::NaiveDate)>` covering `[start..=end]` contiguously, each segment ≤ 7 days inclusive.

- [ ] **Step 1: Decide function location**

`chunk_range_weekly` is a pure date-math helper, parallel to `iter_dates_inclusive` (currently in `src/data/models.rs:360`). The natural home is `src/data/models.rs` next to `iter_dates_inclusive`. **Place it there as a `pub(crate) fn`** so both `openstock_client.rs` and any future caller can use it; the spec calls it "private" but `pub(crate)` is the Rust idiom for "internal to this crate". Document this choice in the function's doc comment.

- [ ] **Step 2: Write failing test S1 — single-day range returns one chunk**

Append to `src/data/models.rs` test module (find the existing `#[cfg(test)] mod tests` block which already contains `iter_dates_inclusive_yields_all_days_in_order` at line 572):

```rust
#[test]
fn chunk_range_weekly_single_day_returns_one_chunk() {
    use chrono::NaiveDate;
    let d = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
    let chunks = chunk_range_weekly(d(2026, 6, 1), d(2026, 6, 1));
    assert_eq!(chunks, vec![(d(2026, 6, 1), d(2026, 6, 1))]);
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test --lib chunk_range_weekly_single_day_returns_one_chunk`
Expected: FAIL with `cannot find function chunk_range_weekly` (or similar compile error).

- [ ] **Step 4: Write failing tests S2, S3, S4 in the same test module**

```rust
#[test]
fn chunk_range_weekly_exact_7_day_returns_one_chunk() {
    use chrono::NaiveDate;
    let d = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
    // 7 days inclusive: 2026-06-01..=2026-06-07
    let chunks = chunk_range_weekly(d(2026, 6, 1), d(2026, 6, 7));
    assert_eq!(chunks, vec![(d(2026, 6, 1), d(2026, 6, 7))]);
}

#[test]
fn chunk_range_weekly_8_day_returns_two_chunks() {
    use chrono::NaiveDate;
    let d = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
    // 8 days inclusive: 2026-06-01..=2026-06-08
    // First chunk: 06-01..=06-07 (7 days); second chunk: 06-08..=06-08 (1 day)
    let chunks = chunk_range_weekly(d(2026, 6, 1), d(2026, 6, 8));
    assert_eq!(
        chunks,
        vec![
            (d(2026, 6, 1), d(2026, 6, 7)),
            (d(2026, 6, 8), d(2026, 6, 8)),
        ]
    );
}

#[test]
fn chunk_range_weekly_long_range_covers_full_window() {
    use chrono::NaiveDate;
    let d = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
    // 30 days inclusive: 2026-06-01..=2026-06-30
    let chunks = chunk_range_weekly(d(2026, 6, 1), d(2026, 6, 30));
    // Expected: 06-01..=06-07, 06-08..=06-14, 06-15..=06-21, 06-22..=06-28, 06-29..=06-30
    assert_eq!(
        chunks,
        vec![
            (d(2026, 6, 1), d(2026, 6, 7)),
            (d(2026, 6, 8), d(2026, 6, 14)),
            (d(2026, 6, 15), d(2026, 6, 21)),
            (d(2026, 6, 22), d(2026, 6, 28)),
            (d(2026, 6, 29), d(2026, 6, 30)),
        ]
    );
    // INV-1B: contiguous coverage
    assert_eq!(chunks.first().unwrap().0, d(2026, 6, 1));
    assert_eq!(chunks.last().unwrap().1, d(2026, 6, 30));
    for window in chunks.windows(2) {
        // chunks[i].1 + 1 day == chunks[i+1].0
        assert_eq!(
            window[0].1.succ_opt().unwrap(),
            window[1].0,
            "gap between {:?} and {:?}",
            window[0], window[1]
        );
    }
    // INV-1C: each chunk ≤ 7 days inclusive
    for (s, e) in &chunks {
        let n = (*e - *s).num_days() + 1;
        assert!(n <= 7, "chunk {:?}-{:?} is {} days, > 7", s, e, n);
    }
}
```

- [ ] **Step 5: Run tests to verify all four fail**

Run: `cargo test --lib chunk_range_weekly`
Expected: 4 FAILs with `cannot find function chunk_range_weekly`.

- [ ] **Step 6: Implement `chunk_range_weekly`**

Add to `src/data/models.rs` immediately after `iter_dates_inclusive` (after line 365):

```rust
/// 把 `[start..=end]` 切成连续的 7 天段（P0.13d D2）。
///
/// 返回 `Vec<(NaiveDate, NaiveDate)>`，覆盖 `[start..=end]`：
///   - 第一段从 `start` 开始
///   - 每段长度 ≤ 7 天（含端点）
///   - 段与段之间无 gap、无 overlap（`chunks[i].1 + 1 day == chunks[i+1].0`）
///   - `start == end` 时返回单元素 `vec![(start, end)]`
///
/// 不依赖 `chrono::Weekday`；纯算术切片，便于测试。
pub(crate) fn chunk_range_weekly(
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> Vec<(chrono::NaiveDate, chrono::NaiveDate)> {
    // Defensive: caller (DateOrRange::from_cli) already guarantees start <= end,
    // but the function is pure and should not panic on edge cases.
    if start > end {
        return vec![];
    }
    let mut out = Vec::new();
    let mut cursor = start;
    while cursor <= end {
        // segment end = min(cursor + 6 days, end)
        let seg_end = if (end - cursor).num_days() >= 7 {
            cursor + chrono::Duration::days(6)
        } else {
            end
        };
        out.push((cursor, seg_end));
        // next segment starts the day after seg_end; if seg_end == end loop exits
        cursor = seg_end + chrono::Duration::days(1);
    }
    out
}
```

- [ ] **Step 7: Run tests to verify all four pass**

Run: `cargo test --lib chunk_range_weekly`
Expected: 4 PASS.

- [ ] **Step 8: Run clippy on the modified file**

Run: `cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -20`
Expected: no new warnings introduced by this change.

- [ ] **Step 9: Commit**

```bash
git add src/data/models.rs
git commit -m "$(cat <<'EOF'
feat(data): add chunk_range_weekly for 7-day range subdivision

Pure date-math helper for P0.13d streaming fetch. Splits an inclusive
NaiveDate range into contiguous ≤7-day segments. 4 unit tests cover
single-day, exact-7-day, 8-day, and 30-day ranges (INV-1B/1C).

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Extract `fetch_minute_klines_range` Helper from `fetch_minute_klines`

**Files:**
- Modify: `src/sources/openstock_client.rs:693-794` (existing `fetch_minute_klines` body)

**Interfaces:**
- Produces: `OpenStockClient::fetch_minute_klines_range(&self, code: &str, period: MinutePeriod, start: NaiveDate, end: NaiveDate, adjust: AdjustType) -> Result<Vec<MinuteBar>>` as a **private** async method on `OpenStockClient`. The wire body uses the `start == end` rule (INV-2A) to choose between the `date` field and the `start_date`/`end_date` fields.
- Existing `fetch_minute_klines` (line 693) is rewritten as a thin dispatcher: parse `DateOrRange` into `(start, end)` and delegate.

- [ ] **Step 1: Verify current behavior is unchanged after extract**

Before any code change, capture the current passing tests as a baseline:
```bash
cargo test --lib openstock_client:: 2>&1 | tail -5
cargo test --test openstock_client 2>&1 | tail -5
```
Expected: all currently-passing tests still pass (these are the P0.13b-1 and P0.13c tests; we are about to refactor but not change behavior).

- [ ] **Step 2: Add `fetch_minute_klines_range` as a new private method**

Insert **immediately after** the existing `fetch_minute_klines` method (after line 794, before the `fetch_minute_share` doc comment at line 796). The body is a copy of the current `fetch_minute_klines` body (lines 700-793) with the `DateOrRange` match removed — instead, the function takes `(start, end)` directly and uses the `start == end` rule for the wire body:

```rust
/// Private helper: fetch minute klines for an inclusive `[start..=end]` sub-range.
///
/// Wire body field selection (INV-2A, preserving P0.13c):
///   - `start == end` → body has only `date` (identical to P0.13b-1 single-day wire)
///   - `start != end` → body has only `start_date` + `end_date`
///
/// This helper is shared by:
///   - `fetch_minute_klines` (batch API; dispatcher for `DateOrRange`)
///   - `fetch_minute_klines_stream` (streaming API; one call per weekly chunk)
async fn fetch_minute_klines_range(
    &self,
    code: &str,
    period: crate::data::models::MinutePeriod,
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
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
    });
    if let Some(adj) = adjust.as_openstock_param() {
        body["adjust"] = serde_json::Value::String(adj.to_string());
    }
    if start == end {
        body["date"] = serde_json::Value::String(start.format("%Y-%m-%d").to_string());
    } else {
        body["start_date"] = serde_json::Value::String(start.format("%Y-%m-%d").to_string());
        body["end_date"] = serde_json::Value::String(end.format("%Y-%m-%d").to_string());
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
        let ts = chrono::NaiveDateTime::parse_from_str(&bar.time[..19], "%Y-%m-%dT%H:%M:%S")
            .map_err(|e| {
                QuantixError::DataParse(format!("解析 minute bars 时间戳失败: {}", e))
            })?;

        out.push(crate::data::models::MinuteBar {
            code: code.to_string(),
            timestamp: ts,
            open: rust_decimal::Decimal::from_str(&format!("{}", bar.open)).unwrap_or_default(),
            high: rust_decimal::Decimal::from_str(&format!("{}", bar.high)).unwrap_or_default(),
            low: rust_decimal::Decimal::from_str(&format!("{}", bar.low)).unwrap_or_default(),
            close: rust_decimal::Decimal::from_str(&format!("{}", bar.close))
                .unwrap_or_default(),
            volume: bar.volume as i64,
            amount: Some(
                rust_decimal::Decimal::from_str(&format!("{}", bar.amount)).unwrap_or_default(),
            ),
            adjust_type: adjust,
        });
    }

    Ok(out)
}
```

- [ ] **Step 3: Rewrite `fetch_minute_klines` as a thin dispatcher**

Replace the body of `fetch_minute_klines` (lines 693-794, i.e. from `pub async fn fetch_minute_klines(` through the closing `}` at line 794) with:

```rust
pub async fn fetch_minute_klines(
    &self,
    code: &str,
    period: crate::data::models::MinutePeriod,
    date_or_range: crate::data::models::DateOrRange,
    adjust: crate::data::models::AdjustType,
) -> Result<Vec<crate::data::models::MinuteBar>> {
    use crate::data::models::DateOrRange;
    let (start, end) = match date_or_range {
        DateOrRange::Date(d) => (d, d),
        DateOrRange::Range { start, end } => (start, end),
    };
    self.fetch_minute_klines_range(code, period, start, end, adjust).await
}
```

Keep the existing doc comment on `fetch_minute_klines` (lines 680-692); optionally add one line pointing at `fetch_minute_klines_range` for the wire-shape details.

- [ ] **Step 4: Run cargo check then full openstock_client test suite**

Run:
```bash
cargo check --workspace 2>&1 | tail -10
cargo test --lib openstock_client:: 2>&1 | tail -10
cargo test --test openstock_client 2>&1 | tail -10
```
Expected: clean build; all previously-passing tests still pass. The P0.13c wiremock tests (`fetch_minute_klines_range_sends_start_date_end_date_body` at line 1634 and the Date-mode wiremock tests at lines 1494, 1558, 1599) must pass without modification — the `start == end` rule in `fetch_minute_klines_range` reproduces the same wire shape as the old Date branch in `fetch_minute_klines`.

If any test fails, **do not modify the test** — re-check the wire body construction in `fetch_minute_klines_range` against the spec §3.2 invariance. Common bug: `start != end` branch accidentally adds `date` field too.

- [ ] **Step 5: Run clippy**

Run: `cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -10`
Expected: no new warnings.

- [ ] **Step 6: Commit**

```bash
git add src/sources/openstock_client.rs
git commit -m "$(cat <<'EOF'
refactor(openstock): extract fetch_minute_klines_range helper

Pure refactor (no behavior change). Pulls the body of fetch_minute_klines
into a private helper that takes (start, end) directly, so the upcoming
streaming API can reuse it per weekly chunk without duplicating wire-shape
logic. INV-2A preserved via start == end → date, else start_date/end_date.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Add `fetch_minute_klines_stream` + `fetch_minute_share_stream`

**Files:**
- Modify: `src/sources/openstock_client.rs` (add two new public methods after `fetch_minute_share`)

**Interfaces:**
- Consumes: `chunk_range_weekly` from Task 1; `fetch_minute_klines_range` from Task 2; `fetch_minute_share_single` (existing, P0.13c, line 836); `iter_dates_inclusive` (existing, `src/data/models.rs:360`).
- Produces:
  - `OpenStockClient::fetch_minute_klines_stream(&self, code: &str, period: MinutePeriod, date_or_range: DateOrRange, adjust: AdjustType) -> impl futures::Stream<Item = Result<Vec<MinuteBar>, QuantixError>> + '_`
  - `OpenStockClient::fetch_minute_share_stream(&self, code: &str, date_or_range: DateOrRange) -> impl futures::Stream<Item = Result<Vec<MinuteShare>, QuantixError>> + '_`

- [ ] **Step 1: Add `futures` imports**

At the top of `src/sources/openstock_client.rs`, near existing `use` statements, add:

```rust
use futures::stream::{self, StreamExt};
```

If the file already has a `use futures::...` line, merge into it. Verify with `grep "^use " src/sources/openstock_client.rs | head -20`.

- [ ] **Step 2: Add `fetch_minute_klines_stream` method**

Insert after `fetch_minute_klines` (or after `fetch_minute_klines_range` from Task 2; pick whichever location keeps related methods together):

```rust
/// 流式拉取分钟 K 线（P0.13d D1/D2）。
///
/// 把 `date_or_range` 解析为 `(start, end)`，调用 `chunk_range_weekly`
/// 切成连续 ≤7 天段，每段一次 `fetch_minute_klines_range` 调用，yield
/// 一个 `Vec<MinuteBar>`。
///
/// - `Date(d)`：单段 `(d, d)`，一个 batch
/// - `Range { start, end }`：从 start 起每 7 天一段；尾段可能短
/// - 错误：首个 batch 失败即 yield `Err`，后续 `next()` 返回 `None`（D4）
/// - 不经过 retry/circuit breaker（D5；与 batch klines 路径一致）
/// - Wire shape 由 `fetch_minute_klines_range` 保证（INV-2A）
pub fn fetch_minute_klines_stream(
    &self,
    code: &str,
    period: crate::data::models::MinutePeriod,
    date_or_range: crate::data::models::DateOrRange,
    adjust: crate::data::models::AdjustType,
) -> impl futures::Stream<
    Item = Result<Vec<crate::data::models::MinuteBar>, QuantixError>,
> + '_ {
    use crate::data::models::DateOrRange;
    let (start, end) = match date_or_range {
        DateOrRange::Date(d) => (d, d),
        DateOrRange::Range { start, end } => (start, end),
    };
    let chunks = crate::data::models::chunk_range_weekly(start, end);
    stream::iter(chunks).then(move |(s, e)| async move {
        self.fetch_minute_klines_range(code, period, s, e, adjust).await
    })
}
```

Note: `code` is `&str`; the `then` closure captures it by move (the &str ref copies). The stream borrows `&self` for the lifetime of the returned `impl Stream + '_` — this satisfies INV-1D.

- [ ] **Step 3: Add `fetch_minute_share_stream` method**

Insert after `fetch_minute_share`:

```rust
/// 流式拉取分时点序列（P0.13d D1/D3）。
///
/// 每个自然日（含非交易日）yield 一个 `Vec<MinuteShare>`；非交易日 yield
/// 空 Vec（D3；batch count == 日历天数，调用方可做完整性检查）。
///
/// - 复用 P0.13c `fetch_minute_share_single`（带 retry + breaker）
/// - 错误：首个 batch 失败即 yield `Err`，后续 `next()` 返回 `None`（D4）
pub fn fetch_minute_share_stream(
    &self,
    code: &str,
    date_or_range: crate::data::models::DateOrRange,
) -> impl futures::Stream<
    Item = Result<Vec<crate::data::models::MinuteShare>, QuantixError>,
> + '_ {
    use crate::data::models::{iter_dates_inclusive, DateOrRange};
    let (start, end) = match date_or_range {
        DateOrRange::Date(d) => (d, d),
        DateOrRange::Range { start, end } => (start, end),
    };
    let days: Vec<chrono::NaiveDate> = iter_dates_inclusive(start, end).collect();
    stream::iter(days).then(move |d| async move {
        self.fetch_minute_share_single(code, d).await
    })
}
```

- [ ] **Step 4: Run cargo check**

Run: `cargo check --workspace 2>&1 | tail -15`
Expected: clean build. Common errors to watch for:
- `cannot find StreamExt in scope` → fix `use futures::stream::{self, StreamExt};`
- lifetime mismatch on `impl Stream + '_` → ensure closure captures `self` by ref (the `move` keyword plus `self.method()` does this implicitly because `self` is `&self` in the method receiver)
- `code` moved into closure but used after → `code: &str` is `Copy`, should just work

If compile fails, do **not** loosen the signature (no `Pin<Box<dyn Stream>>`, no `Send` bound). Adjust the closure / lifetime until `impl Stream + '_` works.

- [ ] **Step 5: Run all existing tests to confirm zero regressions**

Run: `cargo test --workspace 2>&1 | tail -10`
Expected: same pass count as before this task (Task 2 baseline). No new tests yet — they come in Task 4.

- [ ] **Step 6: Run clippy**

Run: `cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -10`
Expected: no new warnings.

- [ ] **Step 7: Commit**

```bash
git add src/sources/openstock_client.rs
git commit -m "$(cat <<'EOF'
feat(openstock): add fetch_minute_klines_stream + fetch_minute_share_stream

 impl Stream<Item = Result<Vec<T>, QuantixError>> + '_ for both paths.
 klines stream uses chunk_range_weekly (7-day segments); share stream
 iterates per calendar day. Both inherit batch-path semantics: klines no
 retry, share auto-inherits envelope retry+breaker. First Err terminates
 stream (D4). Existing batch APIs unchanged (D6, INV-4A).

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Stream Unit Tests — S5 (Equivalence), S6 (Error Terminates), S7 (Non-Trading Empty)

**Files:**
- Modify: `src/sources/openstock_client.rs` test module (append after existing wiremock tests, near line 1900+)

**Interfaces:**
- Consumes: stream methods from Task 3.
- Produces: 3 unit tests proving INV-1A, INV-5A, INV-5B.

These tests need to mock the HTTP layer without spinning up a real server. Look at how existing P0.13b-1 tests do it — they use `wiremock`. But for an in-process unit test of stream semantics we want something lighter. Two viable approaches; pick based on what existing tests already use:

**Approach A — wiremock for S5/S6/S7.** S5/S6/S7 each spin up a wiremock server like the existing tests at lines 1494+, point an `OpenStockClient` at it, and assert on stream output. This is the most realistic. Use this approach.

**Approach B — abstract the HTTP layer.** Add a trait to `OpenStockClient` so tests can inject a fake. This is too invasive for one slice (would require touching `OpenStockClient::new` + every callsite). Do **not** do this.

Pick Approach A.

- [ ] **Step 1: Write failing test S5 — klines stream collects same as a series of batch calls**

Append to `src/sources/openstock_client.rs` test module:

```rust
#[tokio::test]
async fn fetch_minute_klines_stream_collects_same_as_batch_per_chunk() {
    // INV-1A: stream yields the same records as N batch calls, one per weekly chunk.
    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
    use chrono::NaiveDate;
    use futures::StreamExt;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let base = url::Url::parse(&server.uri()).unwrap();

    // Range 14 days = 2 weekly chunks
    let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();

    // Mock: any /data/bars request returns 2 records. Count requests.
    Mock::given(method("post"))
        .and(path("/data/bars"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {"time":"2026-06-01T09:31:00+08:00","open":1.0,"high":2.0,"low":0.5,"close":1.5,"volume":100.0,"amount":150.0},
                {"time":"2026-06-01T09:32:00+08:00","open":1.5,"high":2.5,"low":1.0,"close":2.0,"volume":200.0,"amount":400.0},
            ]
        })))
        .expect(2) // 14 days / 7 = 2 chunks
        .mount(&server)
        .await;

    let client = OpenStockClient::new_for_tests(base, "test-key");
    let mut s = client.fetch_minute_klines_stream(
        "600000",
        MinutePeriod::M1,
        DateOrRange::Range { start, end },
        AdjustType::None,
    );

    let mut total = 0usize;
    while let Some(batch) = s.next().await {
        let batch = batch.expect("batch ok");
        total += batch.len();
    }
    assert_eq!(total, 4); // 2 chunks × 2 records
}
```

If `OpenStockClient::new_for_tests` does not exist, look at how existing wiremock tests construct the client (e.g. lines 1494-1530 use some constructor — re-use whatever it is). Adjust the constructor call accordingly.

- [ ] **Step 2: Run S5 and verify it fails**

Run: `cargo test --lib fetch_minute_klines_stream_collects_same_as_batch_per_chunk`
Expected: FAIL — but if the stream code is correct this should actually PASS on first run. If it does pass, that's fine — TDD discipline allows "write test, see it pass" when the underlying code is already correct. **However**, in this task we're testing code added in Task 3 which already exists. The point of this task is verification, not RED-GREEN. So a pass on first run is acceptable here. Move to Step 3.

- [ ] **Step 3: Write S6 — error on second batch terminates stream**

```rust
#[tokio::test]
async fn fetch_minute_klines_stream_terminates_on_first_batch_error() {
    // INV-5A: first Err yields, subsequent next() returns None.
    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
    use chrono::NaiveDate;
    use futures::StreamExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let base = url::Url::parse(&server.uri()).unwrap();

    // 14-day range => 2 chunks
    let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();

    // First /data/bars request (chunk 1) returns 500; expect at most 1 call total
    Mock::given(method("post"))
        .and(path("/data/bars"))
        .respond_with(ResponseTemplate::new(500).set_body_string("simulated server error"))
        .expect(1)
        .mount(&server)
        .await;

    let client = OpenStockClient::new_for_tests(base, "test-key");
    let mut s = client.fetch_minute_klines_stream(
        "600000",
        MinutePeriod::M1,
        DateOrRange::Range { start, end },
        AdjustType::None,
    );

    let first = s.next().await.expect("first item exists");
    assert!(first.is_err(), "first batch must be Err");

    // Stream must terminate after the error
    let third = s.next().await;
    assert!(third.is_none(), "stream must return None after first Err");
}
```

Note: the spec says "first batch fails" but the stream should also terminate after any single Err. Setting up the mock to fail on the **first** request and asserting 1 call total proves both INV-5A and the "no retry on klines path" property (D5).

- [ ] **Step 4: Run S6 and verify it passes**

Run: `cargo test --lib fetch_minute_klines_stream_terminates_on_first_batch_error`
Expected: PASS.

If FAIL with `expect(1)` violated (i.e. stream made more than 1 request), the stream is not terminating after the first Err. Re-check `fetch_minute_klines_stream`: `stream::iter(chunks).then(...)` will short-circuit correctly because `then` does not poll the next item until the current future resolves and the consumer asks for the next one — so this should "just work". If the test hangs, the consumer is still polling; check that the test breaks out of the loop after the Err (the `assert!(first.is_err())` does that — but if you wrote a `while let` loop, make sure to `break` on Err).

- [ ] **Step 5: Write S7 — share stream yields empty Vec for non-trading days**

```rust
#[tokio::test]
async fn fetch_minute_share_stream_yields_empty_vec_for_non_trading_days() {
    // INV-5B: server returns no records for a non-trading day; stream still yields
    // an empty Vec for that day (not skipped). batch count == calendar day count.
    use crate::data::models::DateOrRange;
    use chrono::NaiveDate;
    use futures::StreamExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let base = url::Url::parse(&server.uri()).unwrap();

    // 3-day range
    let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 3).unwrap();

    // Every MINUTE_DATA request returns empty points array
    Mock::given(method("post"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {"meta": {"trading_date": "2026-06-01"}, "points": []}
            ]
        })))
        .expect(3) // one per calendar day
        .mount(&server)
        .await;

    let client = OpenStockClient::new_for_tests(base, "test-key");
    let mut s = client.fetch_minute_share_stream("600000", DateOrRange::Range { start, end });

    let mut batch_count = 0usize;
    let mut total_records = 0usize;
    while let Some(batch) = s.next().await {
        let batch = batch.expect("batch ok");
        batch_count += 1;
        total_records += batch.len();
    }
    assert_eq!(batch_count, 3, "INV-5B: one batch per calendar day");
    assert_eq!(total_records, 0);
}
```

- [ ] **Step 6: Run S7 and verify it passes**

Run: `cargo test --lib fetch_minute_share_stream_yields_empty_vec_for_non_trading_days`
Expected: PASS.

- [ ] **Step 7: Run full workspace tests + clippy**

Run:
```bash
cargo test --workspace 2>&1 | tail -10
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -10
```
Expected: same baseline + 3 new passing tests; no clippy warnings.

- [ ] **Step 8: Commit**

```bash
git add src/sources/openstock_client.rs
git commit -m "$(cat <<'EOF'
test(openstock): add stream unit tests for INV-1A/5A/5B

- S5: klines stream collected total == sum of per-chunk batch calls
- S6: first Err terminates stream (next() returns None after Err)
- S7: share stream yields empty Vec per non-trading day (count == days)

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Wiremock Tests — W1/W2/W3 (Per-Chunk Wire Body)

**Files:**
- Modify: `src/sources/openstock_client.rs` test module

**Interfaces:**
- Consumes: stream methods from Task 3.

These tests verify the **wire shape** of each per-chunk request matches P0.13c. They complement S5/S6/S7 (which verify stream semantics). Unlike P0.13c (which had to verify the full Date/Range wire shape from scratch), here we only need to verify that the **stream layer dispatches correctly** — the underlying `fetch_minute_klines_range` and `fetch_minute_share_single` are already P0.13c-tested.

- [ ] **Step 1: Write W1 — klines stream emits per-chunk subrange body for multi-week range**

```rust
#[tokio::test]
async fn fetch_minute_klines_stream_emits_per_chunk_subrange_body() {
    // INV-2A: each chunk request body uses start_date/end_date of that chunk
    // (no `date` field) for a multi-week Range input.
    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
    use chrono::NaiveDate;
    use futures::StreamExt;
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let base = url::Url::parse(&server.uri()).unwrap();

    // 14-day range => chunk 1: 06-01..=06-07, chunk 2: 06-08..=06-14
    let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();

    // Chunk 1 body assertion
    Mock::given(method("post"))
        .and(path("/data/bars"))
        .and(body_partial_json(serde_json::json!({
            "start_date": "2026-06-01",
            "end_date": "2026-06-07",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
        .expect(1)
        .mount(&server)
        .await;

    // Chunk 2 body assertion
    Mock::given(method("post"))
        .and(path("/data/bars"))
        .and(body_partial_json(serde_json::json!({
            "start_date": "2026-06-08",
            "end_date": "2026-06-14",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
        .expect(1)
        .mount(&server)
        .await;

    let client = OpenStockClient::new_for_tests(base, "test-key");
    let mut s = client.fetch_minute_klines_stream(
        "600000",
        MinutePeriod::M1,
        DateOrRange::Range { start, end },
        AdjustType::None,
    );
    while let Some(b) = s.next().await {
        b.expect("ok");
    }
}
```

- [ ] **Step 2: Run W1 — should PASS on first run**

Run: `cargo test --lib fetch_minute_klines_stream_emits_per_chunk_subrange_body`
Expected: PASS. If FAIL, the chunk boundaries in `chunk_range_weekly` don't match expected (recheck Task 1) or the wire body in `fetch_minute_klines_range` is wrong (recheck Task 2).

- [ ] **Step 3: Write W2 — share stream emits one request per calendar day**

```rust
#[tokio::test]
async fn fetch_minute_share_stream_emits_one_request_per_calendar_day() {
    // INV-2B: each calendar day emits one /data/fetch MINUTE_DATA request.
    use crate::data::models::DateOrRange;
    use chrono::NaiveDate;
    use futures::StreamExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let base = url::Url::parse(&server.uri()).unwrap();

    // 5-day range
    let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 5).unwrap();

    Mock::given(method("post"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"meta": {"trading_date": "2026-06-01"}, "points": []}]
        })))
        .expect(5)
        .mount(&server)
        .await;

    let client = OpenStockClient::new_for_tests(base, "test-key");
    let mut s = client.fetch_minute_share_stream("600000", DateOrRange::Range { start, end });
    while let Some(b) = s.next().await {
        b.expect("ok");
    }
}
```

- [ ] **Step 4: Run W2 — should PASS on first run**

Run: `cargo test --lib fetch_minute_share_stream_emits_one_request_per_calendar_day`
Expected: PASS.

- [ ] **Step 5: Write W3 — klines stream Date-mode emits single batch with `date` field**

```rust
#[tokio::test]
async fn fetch_minute_klines_stream_date_mode_emits_single_batch_with_date_field() {
    // INV-2A Date path: Date(d) → 1 chunk (d,d) → body has `date` only.
    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
    use chrono::NaiveDate;
    use futures::StreamExt;
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let base = url::Url::parse(&server.uri()).unwrap();
    let d = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();

    Mock::given(method("post"))
        .and(path("/data/bars"))
        .and(body_partial_json(serde_json::json!({"date": "2026-06-01"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
        .expect(1)
        .mount(&server)
        .await;

    let client = OpenStockClient::new_for_tests(base, "test-key");
    let mut s = client.fetch_minute_klines_stream(
        "600000",
        MinutePeriod::M1,
        DateOrRange::Date(d),
        AdjustType::None,
    );
    let mut batches = 0;
    while let Some(b) = s.next().await {
        b.expect("ok");
        batches += 1;
    }
    assert_eq!(batches, 1, "Date(d) must produce exactly 1 batch");
}
```

- [ ] **Step 6: Run W3 — should PASS**

Run: `cargo test --lib fetch_minute_klines_stream_date_mode_emits_single_batch_with_date_field`
Expected: PASS.

- [ ] **Step 7: Run all tests + clippy**

Run:
```bash
cargo test --workspace 2>&1 | tail -10
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -5
```
Expected: +3 new tests passing (W1/W2/W3) on top of Task 4 totals.

- [ ] **Step 8: Commit**

```bash
git add src/sources/openstock_client.rs
git commit -m "$(cat <<'EOF'
test(openstock): add stream wiremock tests W1-W3 for per-chunk wire body

- W1: klines stream Range mode emits start_date/end_date per chunk
- W2: share stream emits one /data/fetch per calendar day
- W3: klines stream Date mode emits single batch with date field

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: CLI `--stream` Flag in `data.rs`

**Files:**
- Modify: `src/cli/commands/data.rs:386-407` (`FetchMinuteKlines`)
- Modify: `src/cli/commands/data.rs:413-428` (`FetchMinuteShare`)

**Interfaces:**
- Produces: two new `stream: bool` fields with `#[arg(long, default_value_t = false)]`.

- [ ] **Step 1: Add `stream` field to `FetchMinuteKlines`**

In `src/cli/commands/data.rs`, locate the `FetchMinuteKlines` enum variant (begins at line 386). Append a new field after the existing `adjust: String,` field (after line 406):

```rust
        /// Stream batches (P0.13d). Emits per-batch progress to stderr.
        /// Default false; when absent, batch API behavior is unchanged.
        #[arg(long, default_value_t = false)]
        stream: bool,
```

The variant should now look like:
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

        /// Stream batches (P0.13d). Emits per-batch progress to stderr.
        /// Default false; when absent, batch API behavior is unchanged.
        #[arg(long, default_value_t = false)]
        stream: bool,
    },
```

- [ ] **Step 2: Add `stream` field to `FetchMinuteShare`**

Same change for the `FetchMinuteShare` variant (begins at line 413). Append after the `end: Option<String>,` field:

```rust
        /// Stream batches (P0.13d). Emits per-batch progress to stderr.
        /// Default false; when absent, batch API behavior is unchanged.
        #[arg(long, default_value_t = false)]
        stream: bool,
```

- [ ] **Step 3: cargo check**

Run: `cargo check --workspace 2>&1 | tail -20`
Expected: compile errors in `src/cli/handlers/app_shell.rs` (the match arms no longer destructure all fields). **This is expected** — Task 7 fixes the handlers. Do not commit yet; the codebase must remain compileable per task, so combine this with Task 7's handler changes into a single commit, OR temporarily use `..` in the match arm. The cleanest path is to do Task 7's app_shell.rs change **now** as part of this commit.

- [ ] **Step 4: Update app_shell.rs match arms (Task 7 first half — keep commit atomic)**

In `src/cli/handlers/app_shell.rs` at lines 385-413, destructure `stream` in both arms:

```rust
            OpenStockCommands::FetchMinuteKlines {
                symbol,
                period,
                date,
                start,
                end,
                adjust,
                stream,
            } => {
                let rt = CliRuntime::load();
                fetch_openstock_minute_klines(
                    &rt.openstock,
                    symbol,
                    period,
                    date,
                    start,
                    end,
                    adjust,
                    stream,
                )
                .await?;
            }
            OpenStockCommands::FetchMinuteShare {
                symbol,
                date,
                start,
                end,
                stream,
            } => {
                let rt = CliRuntime::load();
                fetch_openstock_minute_share(&rt.openstock, symbol, date, start, end, stream)
                    .await?;
            }
```

- [ ] **Step 5: Update handler signatures (Task 7 second half)**

In `src/cli/handlers/openstock_handler.rs`:

For `fetch_openstock_minute_klines` (line 411): add `stream: bool,` parameter between `adjust: String,` and the closing `)`. Add the streaming branch as the first thing inside the function body (before the batch path):

```rust
pub(crate) async fn fetch_openstock_minute_klines(
    settings: &OpenStockSettings,
    symbol: String,
    period: String,
    date: Option<String>,
    start: Option<String>,
    end: Option<String>,
    adjust: String,
    stream: bool,
) -> Result<()> {
    use std::str::FromStr;

    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
    use crate::sources::openstock_client::OpenStockClient;
    use futures::StreamExt;

    let period_enum = MinutePeriod::from_str(&period)
        .map_err(|e| QuantixError::Config(format!("--period: {}", e)))?;
    let adjust_enum = AdjustType::from_str(&adjust)
        .map_err(|e| QuantixError::Config(format!("--adjust: {}", e)))?;
    let dor = DateOrRange::from_cli(date.as_deref(), start.as_deref(), end.as_deref())?;

    let client = OpenStockClient::from_settings(settings)?;

    if stream {
        let mode_label = match &dor {
            DateOrRange::Date(d) => format!("date={}", d),
            DateOrRange::Range { start, end } => format!("range={}..{}", start, end),
        };
        println!(
            "OpenStock stream fetch (/data/bars, symbol={}, minute={}, {})",
            symbol,
            period_enum.as_str(),
            mode_label
        );
        println!(
            "  Adjust: {}",
            adjust_enum
                .as_openstock_param()
                .unwrap_or("none (field omitted)")
        );
        eprintln!("  Streaming weekly chunks:");
        let mut s = client.fetch_minute_klines_stream(&symbol, period_enum, dor.clone(), adjust_enum);
        let mut total = 0usize;
        let mut batches = 0usize;
        let started = std::time::Instant::now();
        while let Some(result) = s.next().await {
            let batch = result?;
            batches += 1;
            total += batch.len();
            eprintln!(
                "  [batch {}] +{} bars (cumulative: {}, elapsed: {:?})",
                batches,
                batch.len(),
                total,
                started.elapsed()
            );
            for bar in &batch {
                println!("{}", format_minute_bar(bar));
            }
        }
        eprintln!(
            "  Done. Total: {} bars across {} batches, {:?} total",
            total,
            batches,
            started.elapsed()
        );
        return Ok(());
    }

    // Batch path (existing behavior, unchanged when --stream absent)
    let bars = client
        .fetch_minute_klines(&symbol, period_enum, dor.clone(), adjust_enum)
        .await?;

    let mode_label = match &dor {
        DateOrRange::Date(d) => format!("date={}", d),
        DateOrRange::Range { start, end } => format!("range={}..{}", start, end),
    };
    println!(
        "OpenStock live fetch (/data/bars, symbol={}, minute={}, {})",
        symbol,
        period_enum.as_str(),
        mode_label
    );
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
    if bars.len() > 10_000 {
        eprintln!(
            "warning: range returns {} records, consider narrowing or use --stream",
            bars.len()
        );
    }
    Ok(())
}
```

Note: `format_minute_bar` may not exist as a helper. Either (a) inline the `println!("{:?}", bar);` (drop the formatted helper) or (b) check if existing code uses `print!("{:?}", bar)` and follow suit. **Prefer (a)** — replace `for bar in &batch { println!("{}", format_minute_bar(bar)); }` with `for bar in &batch { println!("{:?}", bar); }` unless an existing `format_minute_bar` is found via grep.

For `fetch_openstock_minute_share` (line 469): add `stream: bool,` parameter and add a streaming branch:

```rust
pub(crate) async fn fetch_openstock_minute_share(
    settings: &OpenStockSettings,
    symbol: String,
    date: Option<String>,
    start: Option<String>,
    end: Option<String>,
    stream: bool,
) -> Result<()> {
    use crate::data::models::DateOrRange;
    use crate::sources::openstock_client::OpenStockClient;
    use futures::StreamExt;

    let dor = DateOrRange::from_cli(date.as_deref(), start.as_deref(), end.as_deref())?;
    let client = OpenStockClient::from_settings(settings)?;

    if stream {
        let mode_label = match &dor {
            DateOrRange::Date(d) => format!("date={}", d),
            DateOrRange::Range { start, end } => format!("range={}..{}", start, end),
        };
        println!("OpenStock MINUTE_DATA stream (time-share ticks)");
        println!("  Code:     {}", symbol);
        println!("  Mode:     {}", mode_label);
        eprintln!("  Streaming one batch per calendar day:");
        let mut s = client.fetch_minute_share_stream(&symbol, dor.clone());
        let mut total = 0usize;
        let mut batches = 0usize;
        let mut empty_batches = 0usize;
        let started = std::time::Instant::now();
        while let Some(result) = s.next().await {
            let batch = result?;
            batches += 1;
            if batch.is_empty() {
                empty_batches += 1;
            }
            total += batch.len();
            eprintln!(
                "  [batch {}] +{} records (cumulative: {}, empty: {}, elapsed: {:?})",
                batches,
                batch.len(),
                total,
                empty_batches,
                started.elapsed()
            );
            for share in &batch {
                println!("{:?}", share);
            }
        }
        eprintln!(
            "  Done. Total: {} records across {} batches ({} empty), {:?} total",
            total,
            batches,
            empty_batches,
            started.elapsed()
        );
        return Ok(());
    }

    // Batch path (existing behavior)
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
        let n_days = (*end - *start).num_days() + 1;
        if n_days > 10 {
            eprintln!(
                "warning: range spans {} days; consider using --stream for live progress",
                n_days
            );
        }
    }
    Ok(())
}
```

- [ ] **Step 6: Verify imports compile**

Run: `cargo check --workspace 2>&1 | tail -15`
Expected: clean. If missing `futures::StreamExt` in handler, add `use futures::StreamExt;` at the top of `src/cli/handlers/openstock_handler.rs` (top-level import) — re-using the workspace dep already in `Cargo.toml:38`.

- [ ] **Step 7: Run full tests + clippy**

Run:
```bash
cargo test --workspace 2>&1 | tail -10
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -10
```
Expected: all tests still pass (no behavior change unless `--stream` is used; CLI tests don't set `--stream` so batch path runs). No new clippy warnings.

- [ ] **Step 8: Smoke test the CLI manually**

Run:
```bash
cargo run -q -- data openstock fetch-minute-klines --symbol 600000 --date 2026-06-01 2>&1 | tail -10
cargo run -q -- data openstock fetch-minute-klines --symbol 600000 --date 2026-06-01 --stream 2>&1 | tail -10
```
Expected: first command runs the batch path (will fail on connection if OPENSTOCK_BASE_URL not set, but **parse** must succeed — i.e. no clap error). Second command parses `--stream` and runs the stream path (same connection outcome). The goal is to verify clap accepts the new flag and dispatches correctly; actual HTTP success requires a live server.

- [ ] **Step 9: Commit**

```bash
git add src/cli/commands/data.rs src/cli/handlers/openstock_handler.rs src/cli/handlers/app_shell.rs
git commit -m "$(cat <<'EOF'
feat(cli): add --stream flag to fetch-minute-klines + fetch-minute-share

New bool flag (default false). When set, handlers switch to a streaming
loop that prints per-batch progress to stderr and records to stdout.
Existing batch path is unchanged when flag absent (INV-4B). Batch-path
warning text now mentions --stream as a mitigation.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Live Integration Tests L1 + L2

**Files:**
- Modify: `tests/openstock_live_minute_klines.rs` (append L1)
- Modify: `tests/openstock_live_minute_share.rs` (append L2)

**Interfaces:**
- Consumes: stream methods from Task 3; `settings_from_env()` helper already in each test file.

- [ ] **Step 1: Append L1 to `tests/openstock_live_minute_klines.rs`**

```rust
#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn live_fetch_minute_klines_stream_multi_week_range() {
    // L1: stream API and batch API are equivalent for a multi-week range.
    use futures::StreamExt;

    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let settings = settings_from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let client = OpenStockClient::from_settings(&settings).expect("client from settings");

    // 14-day range → 2 weekly chunks
    use chrono::NaiveDate;
    let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
    let dor = DateOrRange::Range { start, end };

    let period = MinutePeriod::M1;
    let adjust = AdjustType::None;

    // Batch call
    let batch_result = client
        .fetch_minute_klines("600000", period, dor.clone(), adjust)
        .await
        .expect("batch fetch ok");

    // Stream call: collect
    let mut stream_result = Vec::new();
    let mut s = client.fetch_minute_klines_stream("600000", period, dor, adjust);
    let mut batch_count = 0;
    while let Some(batch) = s.next().await {
        let batch = batch.expect("stream batch ok");
        batch_count += 1;
        stream_result.extend(batch);
    }
    assert!(batch_count >= 2, "14-day range should produce >= 2 chunks");

    // INV-1A: same length and same first/last timestamp
    assert_eq!(
        batch_result.len(),
        stream_result.len(),
        "batch and stream must return same record count"
    );
    if !batch_result.is_empty() {
        assert_eq!(
            batch_result.first().unwrap().timestamp,
            stream_result.first().unwrap().timestamp,
            "first timestamp must match"
        );
        assert_eq!(
            batch_result.last().unwrap().timestamp,
            stream_result.last().unwrap().timestamp,
            "last timestamp must match"
        );
    }
}
```

If `OpenStockClient::from_settings` is not the right constructor name, look at how existing live tests in the same file construct the client and reuse that exact pattern.

- [ ] **Step 2: Append L2 to `tests/openstock_live_minute_share.rs`**

```rust
#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn live_fetch_minute_share_stream_one_day_per_batch() {
    // L2: share stream yields one batch per calendar day; non-trading days
    // produce empty Vec.
    use futures::StreamExt;

    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let settings = settings_from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let client = OpenStockClient::from_settings(&settings).expect("client from settings");

    use chrono::NaiveDate;
    let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 7).unwrap();
    let dor = DateOrRange::Range { start, end };

    let mut s = client.fetch_minute_share_stream("600000", dor);
    let mut batches = 0usize;
    let mut total = 0usize;
    while let Some(batch) = s.next().await {
        let batch = batch.expect("stream batch ok");
        batches += 1;
        total += batch.len();
    }
    // INV-5B: exactly one batch per calendar day (7)
    assert_eq!(batches, 7, "share stream must yield one batch per calendar day");
    eprintln!("live share stream: {} batches, {} total records", batches, total);
}
```

- [ ] **Step 3: Verify both tests are `#[ignore]`-gated and skip by default**

Run:
```bash
cargo test --test openstock_live_minute_klines 2>&1 | tail -5
cargo test --test openstock_live_minute_share 2>&1 | tail -5
```
Expected: both pass with 0 run / N ignored (the new tests are ignored). The CI `cargo test --workspace` will not invoke them.

- [ ] **Step 4: Run all workspace tests**

Run: `cargo test --workspace 2>&1 | tail -5`
Expected: same pass count + 2 more ignored. Total: 1480 passed (1478 + 2 new in this task's earlier steps minus this task's contribution... actually +12 from prior tasks + 2 ignored = net 0 new passing, 2 new ignored).

- [ ] **Step 5: Run clippy**

Run: `cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -5`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add tests/openstock_live_minute_klines.rs tests/openstock_live_minute_share.rs
git commit -m "$(cat <<'EOF'
test(openstock): add live stream tests L1/L2 (ignored by default)

- L1: klines stream vs batch equivalence on a 14-day live range
- L2: share stream yields exactly one batch per calendar day over 7 days

Both tests are #[ignore]-gated and skip unless QUANTIX_OPENSTOCK_LIVE=1.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Final Validation, Governance Update, OpenSpec Validate

**Files:**
- Modify: `.governance/programs/project-governance/cards/P0.13d.yaml` (status: in_progress → complete)

**Interfaces:** none new.

- [ ] **Step 1: Run full quality gate**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
```
Expected: all three pass. Test count: baseline + 4 (Task 1) + 3 (Task 4) + 3 (Task 5) = +10 passing; +2 ignored (Task 7). Total = 1488 passed / 29 ignored (or whatever the baseline was + the deltas).

- [ ] **Step 2: Run OpenSpec validation**

```bash
openspec validate openstock-data-consumption-p0-13d --strict
openspec validate --all --strict
```
Expected: both exit 0.

- [ ] **Step 3: Run GitNexus change detection**

```bash
gitnexus detect_changes
```
Expected: LOW risk; touched symbols are `OpenStockClient` (new methods only), `chunk_range_weekly` (new), `fetch_openstock_minute_klines` / `fetch_openstock_minute_share` (new parameter). No CRITICAL hub symbols modified.

If HIGH/CRITICAL, investigate what got hit unexpectedly — likely the `fetch_minute_klines` signature is unchanged but its body was rewritten (Task 2); GitNexus should still report it as LOW because no caller's contract changed. If GitNexus flags existing callers as affected, they are false positives (the body change is invisible to callers) — note them in the final review.

- [ ] **Step 4: Update governance card status**

Edit `.governance/programs/project-governance/cards/P0.13d.yaml`: change `status: in_progress` to `status: complete`. Add `completed: "2026-07-03"` if the format supports it.

- [ ] **Step 5: Run openspec transition if the CLI is available**

If `ft:new-node` / `ft:transition` CLI tools exist (check `which ft` or look in `.governance/` scripts):
```bash
ft:transition P0.13d --status complete
```
If not, the manual yaml edit in Step 4 is sufficient.

- [ ] **Step 6: Final commit**

```bash
git add .governance/programs/project-governance/cards/P0.13d.yaml
git commit -m "$(cat <<'EOF'
chore(governance): mark P0.13d complete

All 12 tests passing (10 unit + 2 live ignored). OpenSpec validated.
GitNexus LOW risk. Existing batch API unchanged (INV-4A verified by
zero-modified P0.13a/b/c tests).

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 7: Final smoke test (manual, optional)**

If the OpenStock runtime at `192.168.123.104:8040` is reachable:
```bash
QUANTIX_OPENSTOCK_LIVE=1 \
OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
OPENSTOCK_API_KEY=<key> \
cargo test --test openstock_live_minute_klines --test openstock_live_minute_share -- --ignored
```
Expected: L1 + L2 pass.

```bash
OPENSTOCK_BASE_URL=http://192.168.123.104:8040 OPENSTOCK_API_KEY=<key> \
cargo run -q -- data openstock fetch-minute-klines \
  --symbol 600000 --period 1m --start 2026-06-01 --end 2026-06-30 --stream
```
Expected: 5 batches, each ~1.2k bars, total ~5k bars.

- [ ] **Step 8: Hand off to final review**

The slice is now ready for the final whole-branch code review. The implementer (or controller, depending on which SDD mode) runs:
```bash
git log --oneline 21484df..HEAD
```
and dispatches the final reviewer with the branch diff covering all 8 task commits.

---

## Self-Review Notes

**Spec coverage check:**
- §4.1 stream API signatures → Task 3 ✓
- §4.2 helper extraction + chunk_range_weekly → Tasks 1, 2 ✓
- §4.3 CLI flag → Task 6 ✓
- §4.4 handler streaming branch → Task 6 ✓
- §5 INV-1A → S5 (Task 4) + L1 (Task 7) ✓
- §5 INV-1B/1C → S1-S4 (Task 1) ✓
- §5 INV-1D → compile-time check via signature in Task 3 ✓
- §5 INV-2A → W1, W3 (Task 5) + inherited from P0.13c wiremock tests ✓
- §5 INV-2B → W2 (Task 5) ✓
- §5 INV-3 → no public type changes anywhere ✓
- §5 INV-4A → batch API unchanged, full test suite regression in every task ✓
- §5 INV-4B → Task 6 default_value_t=false ✓
- §5 INV-5A → S6 (Task 4) ✓
- §5 INV-5B → S7 (Task 4) + L2 (Task 7) ✓
- §6 D1-D6 → captured in design.md (Task 0) ✓
- §7 R1-R4 → captured in design.md (Task 0) ✓
- §8 test matrix S1-S7, W1-W3, L1-L2 → Tasks 1, 4, 5, 7 ✓
- §10 quality gates → Task 8 ✓
- §11 verification phases → Task 8 ✓

**Placeholder scan:** none. All code blocks contain complete Rust. Where a helper name is uncertain (`OpenStockClient::new_for_tests`, `OpenStockClient::from_settings`), the step instructs the implementer to grep existing tests for the right constructor.

**Type consistency:**
- `chunk_range_weekly(start: NaiveDate, end: NaiveDate) -> Vec<(NaiveDate, NaiveDate)>` — Task 1 produces, Tasks 3+5 consume ✓
- `fetch_minute_klines_range(&self, code: &str, period: MinutePeriod, start: NaiveDate, end: NaiveDate, adjust: AdjustType) -> Result<Vec<MinuteBar>>` — Task 2 produces, Task 3 consumes ✓
- `fetch_minute_klines_stream(&self, ..., date_or_range: DateOrRange, ...) -> impl Stream<Item = Result<Vec<MinuteBar>, QuantixError>> + '_` — Task 3 produces, Tasks 4+5+6+7 consume ✓
- `fetch_minute_share_stream(&self, ..., date_or_range: DateOrRange) -> impl Stream<Item = Result<Vec<MinuteShare>, QuantixError>> + '_` — Task 3 produces, Tasks 4+5+6+7 consume ✓

---

**End of plan.**
