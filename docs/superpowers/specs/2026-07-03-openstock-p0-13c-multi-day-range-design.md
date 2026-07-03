# P0.13c 设计：OpenStock 多日范围查询（minute-level fetchers）

> 切片：P0.13b 的延续切片（P0.13b-1/2 已合并于 `f859dea`）
> 父设计：`docs/superpowers/specs/2026-07-02-openstock-p0-13b-minute-level-data-design.md`（Non-Goals 标注 "Multi-day range queries (P0.13c)"）
> 范围：为 `fetch_minute_klines` + `fetch_minute_share` 添加可选 `start_date`/`end_date` 范围参数
> 实现方式：子代理驱动开发（与 P0.13b-1/2 一致）

---

## 1. 背景

P0.13b-1 (`fetch_minute_klines`) 与 P0.13b-2 (`fetch_minute_share`) 当前只支持单日查询（必需 `date` 参数）。父设计的 Non-Goals 明确将「多日范围查询」推迟到 P0.13c。

现有项目已经支持日 K 线的范围查询：
- `fetch_klines(code, period, adjust, start: Option<&str>, end: Option<&str>)` — `/data/bars` 直 reqwest 路径
- `fetch_index_klines(code, start, end)` — `/data/fetch` envelope 路径，wire 字段 `start_date`/`end_date`
- `fetch_trade_dates(start, end)` — `/data/fetch` envelope 路径

本切片为两个 minute-level fetcher 补齐同样的范围能力。

## 2. 目标与非目标

### 目标

- `fetch_minute_klines` 支持 `start: Option<&str>` + `end: Option<&str>` 可选参数
- `fetch_minute_share` 支持 `start: Option<&str>` + `end: Option<&str>` 可选参数
- CLI `FetchMinuteKlines` + `FetchMinuteShare` 添加 `--start` / `--end` flags
- 保留 `--date` 作为单日快捷（与 `--start`/`--end` 互斥）
- 多日范围返回扁平 `Vec<MinuteBar>` / `Vec<MinuteShare>`（按时间排序）

### 非目标

- 修改 P0.13a `fetch_klines` / `BarPeriod` / `Kline`
- 修改 P0.13b-1/2 的现有类型（`MinuteBar`, `MinuteShare`, `MinutePeriod`, parsers）
- ClickHouse 写入
- 其他 category（日 K / 指数 / 板块 等）的范围扩展
- 跨期合并（不同 period 的 minute candles 不合并）
- 引入 streaming / async iterator（仍是单次 `Vec` 返回）

## 3. 架构

### 3.1 参数语义

| 调用形态 | 行为 |
|---------|------|
| `date: Some(D), start: None, end: None` | 单日查询（向后兼容，等价于 `start=D, end=D`） |
| `date: None, start: Some(S), end: Some(E)` | 范围查询（含 S、含 E） |
| `date: None, start: Some(S), end: None` | **错误**：start 必须配 end（避免 OpenStock 未定义的"开区间"语义） |
| `date: None, start: None, end: Some(E)` | **错误**：end 必须配 start（同上） |
| `date: None, start: None, end: None` | **错误**：必须至少提供 `--date` 或 `(--start, --end)` 对（R1 修订，见 §6 D5） |
| `date: Some(_), start: Some(_), end: Some(_)` | **错误**：CLI 互斥组冲突 |

**R1 修订（CRITICAL）**：原设计把 `(None, None, None)` 视为 "OpenStock 默认范围"，但 P0.13b-1/2 的 `--date` 当前是**必需参数**（`String`，非 `Option`），不存在"不传 date"的当前行为。`from_cli` 在 `(None, None, None)`、`(None, Some, None)`、`(None, None, Some)` 这三种形态下都返回 `Err`。这避免了 OpenStock runtime 对未定义语义的差异（可能返回全历史、当天、或报错）。

### 3.2 Wire 形状

**R1 修订（CRITICAL）**：经审阅意见检查 OpenStock runtime 后，发现两条路径的 server-side range 支持不对称：

**`fetch_minute_klines`（直 `/data/bars` 路径，server-side range ✅）**：
`/data/bars` 后端 `_eltdx_timeseries.py::fetch_klines`（L92-94）已显式读 `params["start_date"]` / `params["end_date"]` 并转发到 eltdx `get_kline`。Wire body：
```json
{
  "code": "...", "period": "1m", "adjust": "none",
  "start_date": "2026-06-01", "end_date": "2026-06-30"
}
```
Server 直接返回多日扁平 records。**R1 风险保留**：实际字段名漂移（如 `start`/`end`）仍由 live test 验证。

**`fetch_minute_share`（`/data/fetch MINUTE_DATA` envelope 路径，server-side range ❌）**：
OpenStock `_eltdx_timeseries.py::fetch_minute_data`（L181-208）**只接受单个 `date` 参数**，无 `start_date`/`end_date` 处理逻辑。每条 response item 是单日 series `{"meta": {...trading_date...}, "points": [...]}`，每条 point 只有 `time_minutes`（如 `"0930"`）+ `time_label`（如 `"09:31"`），**无日期字段**。日期需要从 `meta.trading_date` 取。

**结论**：`fetch_minute_share` 的 `Range` 模式必须在 client 侧**逐日循环**——把 `Range { start, end }` 展开为 N 个 `Date(di)` 调用，每次发 `params: {code, date: di}` wire body（与 P0.13b-2 单日 wire 完全一致），合并 N 个 response 的 `records` 到扁平 `Vec<MinuteShare>`。`fetch_minute_share` 的 `Date(d)` 模式 wire body 与 P0.13b-2 完全相同（向后兼容）。

| Fetcher | Range 实现路径 | 单次请求? |
|---------|---------------|----------|
| `fetch_minute_klines` | server-side（一次请求带 start_date/end_date） | ✅ 是 |
| `fetch_minute_share` | client-side 循环（N 次单日请求） | ❌ 否（N 次） |

### 3.3 CLI 形状

```rust
FetchMinuteKlines {
    #[arg(long)] symbol: String,
    #[arg(long, default_value = "1m")] period: String,
    // 互斥组：date OR (start/end)
    #[arg(long)] date: Option<String>,
    #[arg(long)] start: Option<String>,
    #[arg(long)] end: Option<String>,
    #[arg(long, default_value = "none")] adjust: String,
}
```

使用 clap `group` 互斥约束，或在 handler 层做 if-let 校验（推荐后者——更简单，错误消息可定制）。

**R1 修订**：当前 `FetchMinuteKlines.date` 是 `String`（必需）。改为 `Option<String>` 是向后兼容的 SUPERSET——旧 CLI `--date 2026-07-02` 仍工作（clap 填入 `Some("2026-07-02")`）。`from_cli(None, None, None)` 返回 `Err`（见 §3.1 R1 修订）。

`FetchMinuteShare` 同形（无 period/adjust；其当前 `date: String` 也改为 `Option<String>`）。

## 4. 组件设计

### 4.1 `fetch_minute_klines` 签名扩展

```rust
pub async fn fetch_minute_klines(
    &self,
    code: &str,
    period: MinutePeriod,
    date_or_range: DateOrRange,  // 新 enum
    adjust: AdjustType,
) -> Result<Vec<MinuteBar>>
```

**新 enum `DateOrRange`**（`src/data/models.rs` 或 `src/sources/openstock_client.rs`）：

```rust
pub enum DateOrRange {
    Date(NaiveDate),
    Range { start: Option<NaiveDate>, end: Option<NaiveDate> },
}
```

构造 helper：
```rust
impl DateOrRange {
    /// 解析 CLI 互斥输入。
    pub fn from_cli(
        date: Option<&str>,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Self> { ... }
}
```

向 wire body 转换：
```rust
impl DateOrRange {
    fn populate_body(&self, body: &mut serde_json::Value) {
        match self {
            DateOrRange::Date(d) => {
                body["date"] = Value::String(d.format("%Y-%m-%d").to_string());
            }
            DateOrRange::Range { start, end } => {
                if let Some(s) = start {
                    body["start_date"] = Value::String(s.format("%Y-%m-%d").to_string());
                }
                if let Some(e) = end {
                    body["end_date"] = Value::String(e.format("%Y-%m-%d").to_string());
                }
            }
        }
    }
}
```

**向后兼容**：`fetch_minute_klines` 现有调用方（CLI handler、tests）继续工作——CLI handler 用 `DateOrRange::Date` / `from_cli`，现有 wiremock 测试不破坏。

### 4.2 `fetch_minute_share` 签名扩展

```rust
pub async fn fetch_minute_share(
    &self,
    code: &str,
    date_or_range: DateOrRange,
) -> Result<Vec<MinuteShare>>
```

**R1 修订（CRITICAL）**：`fetch_minute_share` 的 `Range` 模式**不能**通过单次 envelope 请求实现——OpenStock `MINUTE_DATA` server-side 不接受 `start_date`/`end_date`，且 record 中无日期字段（见 §3.2 R1 修订证据）。改为 **client-side 逐日循环**：

```rust
match date_or_range {
    DateOrRange::Date(d) => {
        // 与 P0.13b-2 完全一致：params = {code, date: d}，单次请求
        self.fetch_minute_share_single(code, d).await
    }
    DateOrRange::Range { start, end } => {
        // start 与 end 都是 Some（from_cli 已强制）
        let mut all = Vec::new();
        for d in iter_dates_inclusive(start.unwrap(), end.unwrap()) {
            let day_records = self.fetch_minute_share_single(code, d).await?;
            all.extend(day_records);
        }
        Ok(all)
    }
}

// P0.13b-2 现有 fetch_minute_share 主体重构为内部 helper：
async fn fetch_minute_share_single(&self, code: &str, date: NaiveDate) -> Result<Vec<MinuteShare>> { ... }
```

`iter_dates_inclusive(start, end)` 生成 `start..=end` 的日历日迭代器（包含非交易日；server 返回空 records 时该日 `Vec` 为空，loop 自然跳过——**不依赖 client 侧交易日历**）。**Non-goal**：跨交易日过滤（OpenStock 已对每个 `date` 返回该日数据或空数组）。

**注意**：`fetch_minute_share` 的 `Date` 模式 wire body 与 P0.13b-2 完全一致（`params = {code, date}`，无 `start_date`/`end_date`）——P0.13b-2 现有 wiremock 测试不破坏。

### 4.3 CLI handler 互斥校验

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
    let dor = DateOrRange::from_cli(date.as_deref(), start.as_deref(), end.as_deref())?;
    // ... 其余与现有 handler 一致
}
```

`from_cli` 错误示例（**R1 修订**：错误消息包含参数名 + 用法提示）：
- `"--date cannot be combined with --start/--end; use either --date for single day or --start/--end for range"`
- `"--start and --end must be provided together (semi-open ranges are not supported)"`
- `"at least one of --date or (--start, --end) is required"`

## 5. 不变量（Invariants）

### INV-1A — date 与 start/end 互斥
CLI `--date` 与 `(--start, --end)` 互斥。同时传入 → `Err`。`from_cli` 在 handler 层强制。

### INV-1B — Range 端点含边界 + start/end 必须成对
`start` 与 `end` 都是 inclusive（含）。`start > end` → `Err`。`start` 与 `end` **必须同时提供**（半开区间不支持，见 §3.1 R1 修订）；只提供一侧 → `Err`。

### INV-2A — 向后兼容 wiremock 测试
现有 P0.13b-1/2 wiremock 测试不破坏。即：
- 旧测试用 `Date(d)` → wire body 仍含 `date` 字段（不变）
- `fetch_minute_klines` 新测试用 `Range { ... }` → wire body 含 `start_date`/`end_date`，**不**含 `date`
- `fetch_minute_share` 新测试用 `Range { ... }` → 触发 N 次单日请求，每次 wire body 含 `date`（**不**含 `start_date`/`end_date`，因为 server 不支持）

### INV-2B — Vec 扁平
多日返回是扁平 `Vec<MinuteBar>` / `Vec<MinuteShare>`，按 timestamp 升序排列。调用方若需按日分组，自行 `group_by`。

**R1 修订**：对 `fetch_minute_share`，扁平化由 client-side 循环自然产生——`iter_dates_inclusive(start, end)` 按日升序迭代，每条记录的 timestamp 由 `meta.trading_date`（来自 server 响应）+ `time_minutes` 组合得到，跨日记录因此有正确日期。**INV-2C**：`fetch_minute_share` 的 `Range` 模式必须保证从 `meta.trading_date` 取日期（**不**依赖 client 侧已知的 `start`/`end` 参数），因为非交易日请求 server 可能返回相邻交易日的 series（`meta.trading_date ≠ requested_date`）。

### INV-3 — 不修改 P0.13b-1/2 类型
`MinuteBar`/`MinuteShare`/`MinutePeriod` struct 定义不变；parsers 不变。仅扩展 fetcher 签名。

## 6. 决策记录

### D1：`DateOrRange` enum 而非 `Option` 多参数
**决策**：用 `enum DateOrRange { Date, Range }` 单参数表达互斥，而非 `(date: Option, start: Option, end: Option)` 三参数。

**原因**：
- 编译时强制互斥（不可能同时设置 date 和 start）
- `from_cli` 是唯一构造路径，handler 单点校验
- 易于扩展（未来加 `Year(u32)` 等形态不破坏签名）

**替代方案（拒绝）**：三 `Option` 参数。理由：调用方需手动检查互斥，违反 DRY。

### D2：CLI 用 handler 层校验（不用 clap group）
**决策**：CLI 不用 `#[group(skip)]` / `#[group(multiple = false)]`，而是在 `DateOrRange::from_cli` 中校验。

**原因**：clap group 错误消息生硬、不可定制；from_cli 错误消息明确指向冲突参数。

### D3：保留 `--date` 作为单日快捷
**决策**：保留 `--date`。`DateOrRange::from_cli(Some("2026-06-30"), None, None)` 返回 `Date(2026-06-30)`。

**原因**：向后兼容 P0.13b-1/2 调用方；单日查询是最常见用例（不要强迫用户写 `--start X --end X`）。

### D4：wiremock-first + live-verify `/data/bars` range 字段名（仅 minute_klines）
**决策**：`fetch_minute_klines`（直 `/data/bars` 路径）假设 server 接受 `start_date`/`end_date` JSON 字段（基于 `_eltdx_timeseries.py:92-94` 证据）。wiremock 测试先行；live 测试验证字段名漂移。

**原因**：避免在编码前向 OpenStock runtime 查询；wiremock 测试覆盖 happy path；live 测试发现字段名漂移时，同 PR 切换字段名（同 P0.13b-2 R3 模式）。

**R1 修订**：`fetch_minute_share` 不适用此决策——server 已确认不支持 range（见 §3.2 R1 修订）。`fetch_minute_share` 的 wire body 在 `Date` 和 `Range` 模式下都是单日 `params: {code, date}`，仅请求次数不同（1 vs N）。

**替代方案（拒绝）**：先用 live 测试探明字段名。理由：违反 TDD；live 测试不能在 CI 跑。

### D5：`(None, None, None)` 与半开区间都报错（R1 修订）
**决策**：`from_cli(None, None, None)`、`from_cli(None, Some, None)`、`from_cli(None, None, Some)` 全部返回 `Err`。

**原因**：
- P0.13b-1/2 当前 `--date` 是必需 `String`，没有"不传 date"的现行行为可继承。
- OpenStock runtime 对 `params: {code}`（无日期）的行为未定义——可能返回全历史、当天、或报错。
- 半开区间（`start` only 或 `end` only）需要 client 侧猜默认值（今天？历史最早？），语义模糊。

**替代方案（拒绝）**：允许 `(None, None, None)` 走 "默认范围"。理由：runtime 行为不可预测；YAGNI（用户总能显式给 `--start`/`--end`）。

### D6：`fetch_minute_share` 用 client-side 循环而非 server-side range（R1 修订）
**决策**：`fetch_minute_share` 的 `Range` 模式在 client 侧逐日循环，**不**发 `start_date`/`end_date` 给 server。

**原因**：
- OpenStock `_eltdx_timeseries.py::fetch_minute_data` 只读 `params["date"]`，无 `start_date`/`end_date` 处理。
- MINUTE_DATA 的 response record 无日期字段，server 即使返回多日 records 也无法区分日期。
- client-side 循环每次请求都拿到单日 series + `meta.trading_date`，日期解析可靠。

**替代方案（拒绝）**：等 OpenStock server 增加 MINUTE_DATA range 支持。理由：跨项目协调；本切片可在 client 侧独立完成；后续 server 支持时可单 PR 切换。

## 7. 风险

### R1：`/data/bars` range 字段名未知（仅 minute_klines）
**风险**：实际 OpenStock runtime 可能用 `start`/`end`（无 `_date` 后缀）或其他命名。

**缓解**：D4 — wiremock 测试假设 `start_date`/`end_date`；live 测试验证；若漂移，单 PR 切换字段名（不影响类型签名）。**R1 修订**：仅 `fetch_minute_klines` 适用；`fetch_minute_share` 走 client-side 循环（D6），无字段名风险。

### R2：超大范围导致响应过大
**风险**：用户传 `--start=2020-01-01 --end=2026-06-30` 可能返回几百万条 minute candles，OOM 或超时。

**缓解**：本切片不引入分页 / streaming（YAGNI）。在 handler 输出加 warning："range returns N records, consider narrowing"；用户可缩小范围。**Non-goal**：分页或游标。**R1 修订对 fetch_minute_share**：N 次单日请求而非一次大请求——避免单次 OOM，但 N 上限仍受 R5 制约。

### R3：现有 wiremock 测试破坏
**风险**：扩展 `fetch_minute_klines` 签名为 `DateOrRange` enum 会破坏 P0.13b-1 现有调用。

**缓解**：INV-2A — `Date(d)` 路径 wire body 与 P0.13b-1 完全一致；P0.13b-1 现有 wiremock 测试改 `Date(...)` 形态调用（语义等价，wire 不变）。`fetch_minute_share` 同理。

### R4：`from_cli` 校验边界
**风险**：`start > end`、`start == end`、`only start`、`only end`、`(None, None, None)` 各语义。

**缓解**：D5 + 测试矩阵覆盖 6 种边界（U1-U7 在 §8 测试矩阵列明，含 R1 新增的 U7 全 None case）。

### R5：范围与 OpenStock 实际接受的上限不一致
**风险**：OpenStock runtime 可能限制单次范围跨度（如最多 30 天）。

**缓解**：本切片不感知 runtime 限制；runtime 返回错误时 envelope path 失败冒泡；handler 错误消息清晰。**R1 修订对 fetch_minute_share**：循环模式下 N 由 client 决定，但 N 次请求的累计延迟可能很高（如 30 日 × ~200ms/req = 6s）；handler 输出含 `latency_ms` 提示。

### R6：MINUTE_DATA server-side range 支持未来可能引入（R1 新增）
**风险**：未来 OpenStock 可能为 MINUTE_DATA 增加 `start_date`/`end_date` 支持，使 client-side 循环变成不必要的 N 次请求。

**缓解**：D6 决策可单 PR 切换——`fetch_minute_share` 的 `Range` 分支从循环改为单次 envelope 请求时，签名不变（`DateOrRange` enum 不变），只改 impl。在 §13 后续事项记录"当 server 支持时，切换 fetch_minute_share Range impl 为单次请求"。

## 8. 测试矩阵

### 8.1 Unit tests (`DateOrRange`)

| ID | 描述 | 输入 | 期望 |
|----|------|------|------|
| U1 | date only | `(Some("2026-06-30"), None, None)` | `Date(2026-06-30)` |
| U2 | start+end | `(None, Some("2026-06-01"), Some("2026-06-30"))` | `Range{start:Some(...), end:Some(...)}` |
| U3 | start only（R1 修订） | `(None, Some("2026-06-01"), None)` | `Err`（半开区间不支持） |
| U4 | end only（R1 修订） | `(None, None, Some("2026-06-30"))` | `Err`（同上） |
| U5 | date + start conflict | `(Some("X"), Some("Y"), None)` | `Err` |
| U6 | start > end | `(None, Some("2026-06-30"), Some("2026-06-01"))` | `Err` |
| U7 | all None（R1 新增） | `(None, None, None)` | `Err`（D5） |

### 8.2 Wiremock tests (`fetch_minute_klines` + `fetch_minute_share`)

| ID | 描述 | 验证 |
|----|------|------|
| W1 | minute_klines Range 触发 start_date/end_date body | mock 断言 body 含 `start_date`+`end_date`，**不**含 `date` |
| W2 | minute_klines Date 路径仍只发 date body | mock 断言 body 含 `date`，**不**含 `start_date`/`end_date`（向后兼容） |
| W3 | minute_share Range 触发 N 次单日请求（R1 修订） | mock 设置 `Matcher` 接受任意 `params.date`；断言被调用 N 次（= 范围天数）；每次 body 含 `date`，**不**含 `start_date`/`end_date` |
| W4 | minute_share Date 同 W2（向后兼容） | mock 断言 `params` 含 `date` only，被调用 1 次 |
| W5 | minute_share Range 跳过非交易日（R1 新增） | mock 对非交易日 date 返回空 records；client 循环不 panic；最终 Vec 仅含交易日记录 |

### 8.3 Live tests (`#[ignore]`)

| ID | 描述 | 数据 |
|----|------|------|
| L1 | sh600000 范围 5 天 1m candles | 断言非空，首/末 timestamp 跨多日 |
| L2 | sh600000 范围 5 天 minute_share | 同 L1 |
| L3 | sh600000 `start > end` 错误 | 断言 Err（来自 from_cli，不触发 HTTP） |

## 9. OpenSpec 变更

新增 `openspec/changes/openstock-data-consumption-p0-13c/`：
- proposal / tasks / design / spec deltas（4 件套）

`spec.md` deltas 标 `## MODIFIED Requirements`（修改 fetch_minute_klines + fetch_minute_share 的 scenario，增加 range 输入），不破坏 P0.13b-1/2 已合并的 `## ADDED Requirements`。

## 10. Governance Card

`.governance/programs/project-governance/cards/P0.13c.yaml`：
- allowed_paths: 涉及的所有文件（含 `DateOrRange` 所在文件）
- forbidden_paths: P0.13a/P0.13b-1/P0.13b-2 的关键 symbol
- non_goals: ClickHouse、跨期合并、其他 category、分页

## 11. 文件清单

| 文件 | 操作 | 行数预估 |
|------|------|---------|
| `src/data/models.rs` | Modify（追加 `DateOrRange` enum + `from_cli` + `iter_dates_inclusive` helper；**R1 修订**：固定到 models.rs，与 `MinutePeriod` 同级） | +110 |
| `src/sources/openstock_client.rs` | Modify（`fetch_minute_klines` 签名 + wire body 用 `start_date`/`end_date`；`fetch_minute_share` 签名 + 抽出 `fetch_minute_share_single` helper + `Range` 分支 client 循环） | +90 |
| `src/cli/commands/data.rs` | Modify（`FetchMinuteKlines` + `FetchMinuteShare` 的 `date: String` 改 `Option<String>`，加 `--start`/`--end` flags） | +20 |
| `src/cli/handlers/openstock_handler.rs` | Modify（2 handlers 增加 start/end 参数 + from_cli 校验 + 错误消息含参数名） | +40 |
| `src/cli/handlers/app_shell.rs` | Modify（2 dispatcher arms 传递新参数） | +6 |
| P0.13b-1/2 现有 wiremock tests | Modify（改用 `Date(...)` 形态调用，wire 不变） | +0 (no line change, just call site) |
| `tests/openstock_live_minute_klines.rs` | Modify（追加 L1 多日范围 test，验证 server-side range wire） | +25 |
| `tests/openstock_live_minute_share.rs` | Modify（追加 L2 多日范围 test，验证 client-side 循环 + `meta.trading_date` 解析） | +35 |
| `openspec/changes/openstock-data-consumption-p0-13c/*` | Create (4 files) | +220 |
| `.governance/programs/project-governance/cards/P0.13c.yaml` | Create | +30 |

**R1 修订**：总改动上调到 ~540 行新增（client 循环 + `iter_dates_inclusive` helper + 新增 wiremock W5 test），~35 行修改（call site）。

## 12. 验证

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --lib --package quantix-cli openstock
cargo test --workspace
openspec validate openstock-data-consumption-p0-13c --strict
openspec validate --all --strict
gitnexus detect_changes    # expect LOW
```

## 13. 后续事项（超出 P0.13c 范围）

- 分页 / streaming（处理超大范围）
- ClickHouse 批量写入多日数据
- **R1 新增**：当 OpenStock server 为 MINUTE_DATA 增加 `start_date`/`end_date` 支持时，将 `fetch_minute_share` 的 `Range` 分支从 client-side 循环切换为单次 envelope 请求（D6 替代路径；签名不变，仅 impl 改）
- 其他 category 的范围扩展
