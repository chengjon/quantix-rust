# OpenStock P0.13d — Streaming Fetch for Minute-Level Data

**Slice**: P0.13d
**Date**: 2026-07-03
**Spec status**: design-review-ready
**Parent**: P0.13c (multi-day range queries, shipped `d30e3df`)
**Depends on**: P0.13a/b-1/b-2/c (all completed)

---

## 1. 背景与动机

P0.13c 交付了 `fetch_minute_klines` / `fetch_minute_share` 的多日范围查询能力（`DateOrRange` enum、CLI `--start`/`--end` flag）。当前实现**一次性**返回整个范围的 `Vec<MinuteBar>` / `Vec<MinuteShare>`，handler 在 >10k 条记录时发出 warning 但不分页。

问题：**大范围（数月 / 数年）查询的内存峰值与请求大小不可控**。

- `fetch_minute_klines` 1m 周期 6 个月范围 ≈ 36,000 条 MinuteBar，全部累积在返回 Vec 中
- `fetch_minute_share` P0.13c 已逐日循环但同样累积到单 Vec
- 大范围作业在 CLI handler 内打印所有记录，进一步放大内存

OpenStock server **无服务端分页原语**（无 `limit/offset/cursor`，见 `openstock/openstock/fetching.py:232` `execute_bars_payload`）。本片通过**客户端范围切片 + 流式 API** 解决内存特征问题，不需要服务端协作。

## 2. 目标

- 提供 `fetch_minute_klines_stream` / `fetch_minute_share_stream` 返回 `impl Stream<Item = Result<Vec<T>, QuantixError>>`，使调用方可以按 batch 处理大范围数据
- **不修改**现有 `fetch_minute_klines` / `fetch_minute_share`（Vec API）签名与 wire 行为
- CLI 新增 `--stream` flag，启用后走流式路径并打印 per-batch 进度
- 保持 P0.13c 全部现有测试零修改通过

## 3. 非目标

- 服务端分页协议（OpenStock 无 `limit/offset/cursor`，本片不发明）
- ClickHouse 多日 batch 写入（独立切片；本片只输出 Vec，不写库）
- 反压（backpressure）—— `Vec<T>` per batch 已天然提供
- 重构 batch API 为 `stream::collect`（D6 拒绝）
- streaming 扩展到其他 fetcher（`fetch_klines` 日线、`fetch_index_klines` 等）—— 独立切片
- P0.13c §13 R6（服务端 MINUTE_DATA range 支持）—— 等 OpenStock server
- 为 klines 路径加 retry（R4 风险，留作 P0.13e 候选）

---

## 4. API 设计

### 4.1 新增公开方法（与现有 Vec API 并存，D6）

```rust
impl OpenStockClient {
    /// 流式拉取分钟 K 线。按"每 7 天一段"切片用户范围，每段一个
    /// /data/bars 请求，每段 yield 一个 Vec<MinuteBar>。
    ///
    /// - `Date(d)`：单段 (d, d)，一个 batch
    /// - `Range{start..end}`：从 start 起每 7 天一段；尾段可能短
    /// - 错误：首个 batch 失败即终止 stream（D4）
    /// - 不经过 retry/circuit breaker（D5，与 fetch_minute_klines 一致）
    pub fn fetch_minute_klines_stream(
        &self,
        code: &str,
        period: MinutePeriod,
        date_or_range: DateOrRange,
        adjust: AdjustType,
    ) -> impl futures::Stream<
        Item = Result<Vec<MinuteBar>, QuantixError>,
    > + '_;

    /// 流式拉取分时点序列。每个自然日 yield 一个 Vec<MinuteShare>。
    /// 非交易日 yield 空 Vec（D3，计数 = 日历天数）。
    ///
    /// - 复用 P0.13c 的 fetch_minute_share_single + iter_dates_inclusive
    /// - 每日走 self.fetch::<MinuteShareEnvelope>()（自带 retry + breaker）
    /// - 错误：首个 batch 失败即终止 stream（D4）
    pub fn fetch_minute_share_stream(
        &self,
        code: &str,
        date_or_range: DateOrRange,
    ) -> impl futures::Stream<
        Item = Result<Vec<MinuteShare>, QuantixError>,
    > + '_;
}
```

### 4.2 内部 helper 重构（私有）

抽出现有 `fetch_minute_klines` 的核心逻辑为私有 `fetch_minute_klines_range`：

```rust
/// 私有 helper：接收已解析的 (start, end) 子范围。
/// wire body 字段名按 start == end 判断：
///   - start == end → body["date"] = start（保留 P0.13c INV-2A Date 路径 wire）
///   - start != end → body["start_date"]/body["end_date"]（P0.13c Range 路径 wire）
async fn fetch_minute_klines_range(
    &self,
    code: &str,
    period: MinutePeriod,
    start: NaiveDate,
    end: NaiveDate,
    adjust: AdjustType,
) -> Result<Vec<MinuteBar>>;

/// 私有纯函数：把 [start..=end] 切成连续的 7 天段（D2）。
/// 返回 Vec<(NaiveDate, NaiveDate)>，覆盖 [start..=end]，每段 ≤ 7 天。
fn chunk_range_weekly(
    start: NaiveDate,
    end: NaiveDate,
) -> Vec<(NaiveDate, NaiveDate)>;
```

现有 `fetch_minute_klines` 改为薄包装：

```rust
pub async fn fetch_minute_klines(
    &self, code: &str, period: MinutePeriod, dor: DateOrRange, adjust: AdjustType,
) -> Result<Vec<MinuteBar>> {
    let (start, end) = match dor {
        DateOrRange::Date(d) => (d, d),
        DateOrRange::Range { start, end } => (start, end),
    };
    self.fetch_minute_klines_range(code, period, start, end, adjust).await
}
```

Stream API 同样走 `fetch_minute_klines_range`：

```rust
pub fn fetch_minute_klines_stream(
    &self, code: &str, period: MinutePeriod, dor: DateOrRange, adjust: AdjustType,
) -> impl Stream<Item = Result<Vec<MinuteBar>, QuantixError>> + '_ {
    let (start, end) = match dor {
        DateOrRange::Date(d) => (d, d),
        DateOrRange::Range { start, end } => (start, end),
    };
    let chunks = chunk_range_weekly(start, end);
    futures::stream::iter(chunks).then(move |(s, e)| async move {
        self.fetch_minute_klines_range(code, period, s, e, adjust).await
    })
}
```

`fetch_minute_share_stream` 走 P0.13c 既有 `fetch_minute_share_single`：

```rust
pub fn fetch_minute_share_stream(
    &self, code: &str, dor: DateOrRange,
) -> impl Stream<Item = Result<Vec<MinuteShare>, QuantixError>> + '_ {
    let (start, end) = match dor {
        DateOrRange::Date(d) => (d, d),
        DateOrRange::Range { start, end } => (start, end),
    };
    futures::stream::iter(iter_dates_inclusive_vec(start, end))
        .then(move |d| async move {
            self.fetch_minute_share_single(code, d).await
        })
}
```

其中 `iter_dates_inclusive_vec` 是 P0.13c 现有 `iter_dates_inclusive` 的 `Vec<NaiveDate>` 收集版（Stream 需要 owned iterator；现有 `iter_dates_inclusive` 返回 `impl Iterator<Item = NaiveDate>`，需要先 collect）。

### 4.3 CLI（`src/cli/commands/data.rs`）

两个变体各加 `stream: bool` field（保留 P0.13c 既有 `default_value` 属性不变）：

```rust
FetchMinuteKlines {
    #[arg(long)]
    symbol: String,

    #[arg(long, default_value = "1m")]      // P0.13c 既有
    period: String,

    #[arg(long, default_value = "none")]    // P0.13c 既有
    adjust: String,

    #[arg(long)]
    date: Option<String>,

    #[arg(long)]
    start: Option<String>,

    #[arg(long)]
    end: Option<String>,

    #[arg(long, default_value_t = false)]   // 新增
    stream: bool,
}
// FetchMinuteShare 同样加 stream: bool（保留 P0.13c 既有 default_value 属性）
```

向后兼容：默认 false；现有 `--date` / `--start` / `--end` 调用形态行为不变。

### 4.4 Handler（`src/cli/handlers/openstock_handler.rs`）

`fetch_openstock_minute_klines` / `fetch_openstock_minute_share` 各加 `stream: bool` 参数。stream 分支示例：

```rust
if stream {
    let mut s = client.fetch_minute_klines_stream(symbol, period, dor, adjust);
    let mut total = 0usize;
    let mut batches = 0usize;
    let started = std::time::Instant::now();
    while let Some(result) = s.next().await {
        let batch = result?;
        batches += 1;
        total += batch.len();
        eprintln!(
            "[batch {}] +{} bars (cumulative: {}, elapsed: {:?})",
            batches, batch.len(), total, started.elapsed()
        );
        for bar in &batch {
            println!("{}", format_minute_bar(bar));
        }
    }
    eprintln!(
        "Done. Total: {} bars across {} batches, {:?} total",
        total, batches, started.elapsed()
    );
    return Ok(());
}
```

Batch 路径 warning 文案改为 `"consider narrowing or use --stream"`。

---

## 5. 不变量

| ID | 描述 |
|---|---|
| **INV-1A** | Stream API 与 batch API 行为等价：`(stream.collect::<Vec<_>>() concat) == batch API result`（同一范围、symbol、period、adjust） |
| **INV-1B** | `chunk_range_weekly(start, end)` 无缝覆盖 `[start..=end]`：`chunks[0].0 == start && chunks.last().1 == end && chunks[i].1 + 1 day == chunks[i+1].0` |
| **INV-1C** | `chunk_range_weekly` 每段长度 ≤ 7 天（含端点）：`(end - start).num_days() + 1 ≤ 7` |
| **INV-1D** | Stream 类型签名：`impl Stream<Item = Result<Vec<MinuteBar>, QuantixError>>` 与 `impl Stream<Item = Result<Vec<MinuteShare>, QuantixError>>`，借用 `&self`（无 `'static` 约束） |
| **INV-2A** | `fetch_minute_klines_range` wire body 与 P0.13c 完全一致：`start == end` → `{date}`；`start != end` → `{start_date, end_date}` |
| **INV-2B** | `fetch_minute_share_stream` 每日 batch wire body 与 P0.13c `fetch_minute_share_single` 完全一致 |
| **INV-3** | 现有 `MinuteBar` / `MinuteShare` / `MinutePeriod` / `DateOrRange` / `AdjustType` 类型签名与字段**完全不变** |
| **INV-4A** | 现有 `fetch_minute_klines` / `fetch_minute_share`（Vec API）签名与行为**完全不变**，P0.13a/b/c 全部现有测试零修改通过 |
| **INV-4B** | CLI `--date` / `--start` / `--end` flag 行为**完全不变**（仅新增 `--stream`，默认 false） |
| **INV-5A** | Stream 错误语义：首个 batch 失败即终止（`yield Err → return None on next()`） |
| **INV-5B** | 非交易日 share batch yield **空 Vec**（而非 skip） |

---

## 6. 决策记录

### D1 — Stream 类型：`impl Stream<Item = Result<Vec<T>>>`

**选择**：返回 `impl futures::Stream<Item = Result<Vec<...>, QuantixError>>`，不用 `Pin<Box<dyn Stream>>`、channel、callback。

**理由**：
- 编译期单态化，零堆分配（除 Vec 本身）
- 调用方惯用 `while let Some(r) = s.next().await`，与 ecosystem 标准一致
- `Vec<T>` per batch 比单条 `T` 减少 `next()` 调用次数

**Rejected**：callback（不能 await）、Paginator struct（样板代码）、tokio mpsc（错误传播复杂）、`Pin<Box<dyn Stream>>`（运行时开销）。

### D2 — Klines 切片：固定 7 天一段

**选择**：`chunk_range_weekly` 按 `start + 7 days` 切，不绑定自然周。

**理由**：
- 不依赖 `chrono::Weekday`，跨日历无歧义
- 切片均匀，每段 ≤ 7 天
- 纯函数易测

**Rejected**：自然周（依赖 Weekday）、月度（长度不均）、日度（太碎）、自适应（复杂且无必要——1m 周度 ≈ 1.2k bars 已远低于阈值）。

### D3 — Share 切片：一日一 batch，非交易日 yield 空 Vec

**选择**：每个 `NaiveDate` yield 一个 `Vec<MinuteShare>`；非交易日 yield `vec![]`。

**理由**：
- 完美复用 P0.13c `fetch_minute_share_single`
- batch count == 日历天数 → 调用方可做完整性检查

**Rejected**：跳过非交易日（丢失 day-level 信号）、typed enum `Trading/NonTrading`（增加类型复杂度）、N 天累积（多余 knob）。

### D4 — 错误语义：首个错误终止 stream

**选择**：任一 batch 失败 → yield `Err(...)` → 后续 `next()` 返回 `None`。

**理由**：
- 与 batch API 语义一致
- 调用方已处理的 prior batches 是已完成的副作用（与 batch API 同样特性）
- 简单可预测

**Rejected**：错误不终止（易吞错）、末尾聚合（非标准 stream shape）、stream 内置 retry（与 D5 正交）。

### D5 — 重试/熔断：继承现有路径行为

**选择**：stream 各 batch 走各自现有底层路径，不引入新 retry/breaker 包装。

**理由**：
- klines 路径现状无 retry（`fetch_minute_klines` 直接 `self.http.post()`）—— stream 保持现状
- share 路径已有 retry+breaker（`fetch_minute_share_single` → `fetch::<T>()`）—— stream 自动继承
- 一致性：stream 与 batch 行为对齐（INV-1A 前提）
- 避免引入"stream 比 batch 更稳健"的不对称

**Rejected**：stream klines batch 加 retry（范围蔓延）、stream share 移除 retry（退步）。

### D6 — 双 API 并存（side-by-side）

**选择**：`fetch_minute_klines`（Vec）与 `fetch_minute_klines_stream`（Stream）并存，内部各自实现。

**理由**：
- 零 churn：P0.13a/b/c 全部 wiremock / live / governance 不变
- INV-4A 直接保证
- 等价性由 INV-1A（S5 测试）+ L1 live 验证

**Rejected**：`fetch_minute_klines = stream.collect()`（DRY 但 batch 内存特征不变、引入 churn 风险、违反用户决策）、deprecate batch API（大规模 churn，所有 P0.13b/c 调用方需迁移）。

---

## 7. 风险登记

| ID | 风险 | 缓解 |
|---|---|---|
| **R1** | stream 类型签名引入新公共 trait bound，下游 compile error | 公开方法返回 `impl Stream + '_`（不写 `Send`）；调用方 `use futures::StreamExt`；CI `cargo build --workspace` 兜底 |
| **R2** | chunk_range_weekly 边界 bug（off-by-one、漏覆盖） | S4 端到端覆盖测试 |
| **R3** | CLI `--stream` flag 在 batch 路径下被忽略，行为歧义 | 设计保证：flag 默认 false；flag=true 时 handler 走完全独立的 stream 分支 |
| **R4** | klines stream 无 retry，transient 5xx 导致大范围作业失败 | D5 已记录；作为已知不对称；可作 follow-up P0.13e |

> **R1 修订**：删除原 R2（`futures` crate 不在 workspace）—— 已验证 `Cargo.toml:38-39` 已声明 `futures = "0.3"` 与 `futures-util = "0.3"`，无需新增依赖。R3-R5 顺次重编号。

---

## 8. 测试矩阵

| ID | 类型 | 名称 | 覆盖 INV |
|---|---|---|---|
| **S1** | unit | `chunk_range_weekly_single_day_returns_one_chunk` | INV-1B/1C |
| **S2** | unit | `chunk_range_weekly_exact_7_day_returns_one_chunk` | INV-1B/1C |
| **S3** | unit | `chunk_range_weekly_8_day_returns_two_chunks` | INV-1B/1C |
| **S4** | unit | `chunk_range_weekly_long_range_covers_full_window` | INV-1B/1C（端点衔接 + 总覆盖） |
| **S5** | unit | `fetch_minute_klines_stream_collects_same_as_batch` | INV-1A（注入 mock helper，断言两路径输出相同） |
| **S6** | unit | `stream_terminates_on_first_batch_error` | INV-5A（注入第 2 batch 失败；断言 stream yield [Ok, Err] 后 None） |
| **S7** | unit | `share_stream_yields_empty_vec_for_non_trading_days` | INV-5B（mock 每日返回空 records；count == 日历天数） |
| **W1** | wiremock | `fetch_minute_klines_stream_emits_weekly_subrange_body` | INV-2A（多周范围，wiremock 收 N 次请求；每次 body 含对应 sub_start/sub_end，无 `date` 字段） |
| **W2** | wiremock | `fetch_minute_share_stream_emits_one_request_per_calendar_day` | INV-2B（范围 7 天 → 7 次请求） |
| **W3** | wiremock | `fetch_minute_klines_stream_date_mode_emits_single_batch` | INV-2A（`Date(d)` → 1 次请求，body 含 `date`） |
| **L1** | live `#[ignore]` | `live_fetch_minute_klines_stream_multi_week_range` | INV-1A（流式 vs batch 等价） |
| **L2** | live `#[ignore]` | `live_fetch_minute_share_stream_one_day_per_batch` | INV-5B（每日一 batch） |

**总计：7 单元 + 3 wiremock + 2 live = 12 测试**。

测试设计原则：
- 不重新测 wire shape（P0.13c 已覆盖）—— W1/W2/W3 只验证调用次数 + 每次 body 正确
- S5 等价测试是核心：通过 trait 注入 mock 底层 helper，让 batch 与 stream 走相同 fake 数据
- S6 用 `then` 注入失败；构造 fake stream helper，第 2 batch 返回 Err

---

## 9. 文件改动清单

| 文件 | 操作 | 估算 |
|---|---|---|
| `src/sources/openstock_client.rs` | 新增 stream API + 私有 helper + 7 单元测试 | +180 |
| `src/cli/commands/data.rs` | 2 个变体各加 `stream: bool` field | +6 |
| `src/cli/handlers/openstock_handler.rs` | 2 个 handler 各加 stream 参数 + 流式分支 + warning 文案 | +50 |
| `src/cli/handlers/app_shell.rs` | 2 个 dispatcher arm 解构 stream 字段 | +6 |
| `tests/openstock_live_minute_klines.rs` | L1 live 测试 | +60 |
| `tests/openstock_live_minute_share.rs` | L2 live 测试 | +50 |
| `openspec/changes/openstock-data-consumption-p0-13d/{proposal,tasks,design}.md` + `specs/openstock-data-consumption/spec.md` | OpenSpec 4 文件 | +200 |
| `.governance/programs/project-governance/cards/P0.13d.yaml` | 新建 governance card | +30 |

> R1 修订：删除原 `Cargo.toml` 行——`futures = "0.3"` 已在 workspace（`Cargo.toml:38`）。

**总估算：约 +578 行新增 / +30 行修改**（与 P0.13c 的 +540/+35 体量相当）。

---

## 10. 验收门禁

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace                    # 含 12 个新测试
openspec validate openstock-data-consumption-p0-13d --strict
openspec validate --all --strict
gitnexus detect_changes                   # 期望 LOW（additive，未触碰 CRITICAL hub）
git diff --check
```

## 11. 验证流程

**阶段 1 — 离线（CI 默认）**：
```bash
cargo test --workspace
# 期望：1478 passed (1466 现有 + 12 新) / 29 ignored (27 + 2 新)
```

**阶段 2 — live 网络（手动 QUANTIX_OPENSTOCK_LIVE=1）**：
```bash
QUANTIX_OPENSTOCK_LIVE=1 \
OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
OPENSTOCK_API_KEY=<key> \
cargo test --test openstock_live_minute_klines --test openstock_live_minute_share -- --ignored
```

**阶段 3 — CLI smoke**：
```bash
cargo run -q -- data openstock fetch-minute-klines \
  --symbol 600000 --period 1m --start 2026-06-01 --end 2026-06-30 --stream
# 期望：[batch 1/5] +1180 bars ... Done. Total: ~5000 bars across 5 batches
```

**阶段 4 — GitNexus / Governance**：
```bash
gitnexus detect_changes
# 期望 LOW；touched symbols: OpenStockClient (新方法), fetch_openstock_minute_* (新参数)
openspec validate --all --strict
```

---

## 12. 决策对照表（vs P0.13c）

| 维度 | P0.13c | P0.13d |
|---|---|---|
| 切片目标 | 多日范围查询支持 | 流式 / 分页获取 |
| 新增公共类型 | `DateOrRange` enum | 无（复用现有） |
| 新增公共方法 | `fetch_minute_klines(Range)` + `fetch_minute_share(Range)` | `fetch_minute_klines_stream` + `fetch_minute_share_stream` |
| CLI 新 flag | `--start` / `--end` | `--stream` |
| 测试矩阵 | 7 unit + 5 wiremock + 3 live = 15 | 7 unit + 3 wiremock + 2 live = 12 |
| 关键 INV | INV-2C（trading_date 来自 meta） | INV-1A（stream vs batch 等价） |
| 行数 | +540 新 / +35 改 | +580 新 / +30 改 |

---

## 13. 后续事项（超出 P0.13d 范围）

- **P0.13e 候选**（R4 follow-up）：为 klines batch API 加 retry（stream 路径自动继承）；如生产环境出现 transient 5xx 失败率高，作为下一切片
- 服务端分页协议（若 OpenStock server 引入 `limit/offset`）
- ClickHouse 多日 batch 写入（stream → batch insert）
- 反压 / 流控
- streaming 扩展到其他 fetcher（日线、index 等）
- P0.13c §13 R6：当 OpenStock server 为 MINUTE_DATA 增加 range 支持时，share stream 可改为单次请求 + 内部分批

---

## 14. References

- P0.13c 设计：`docs/superpowers/specs/2026-07-03-openstock-p0-13c-multi-day-range-design.md`
- P0.13c plan：`docs/superpowers/plans/2026-07-03-openstock-p0-13c-multi-day-range-plan.md`
- OpenStock server `/data/bars` 实现：`openstock/openstock/fetching.py:232`
- OpenStock server KLINES adapter：`openstock/openstock/adapters/_eltdx_timeseries.py:39`
- 现有 client retry/breaker：`src/sources/openstock_client.rs:180`（`fetch<T>`）
- 现有 `fetch_minute_klines`：`src/sources/openstock_client.rs:693`
- 现有 `fetch_minute_share_single`：P0.13c 引入
- 现有 `iter_dates_inclusive`：`src/data/models.rs`
- futures::Stream 文档：https://docs.rs/futures/latest/futures/stream/trait.Stream.html
