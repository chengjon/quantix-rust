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
| `date: None, start: Some(S), end: None` | 从 S 到今天（OpenStock 决定上限） |
| `date: None, start: None, end: Some(E)` | 历史到 E（OpenStock 决定下限） |
| `date: None, start: None, end: None` | OpenStock 默认范围（与 P0.13b-1/2 当前行为一致） |
| `date: Some(_), start: Some(_), end: Some(_)` | **错误**：CLI 互斥组冲突 |

### 3.2 Wire 形状

**`fetch_minute_share`（envelope 路径，已知）**：与 `fetch_index_klines` 一致——
```json
{
  "data_category": "MINUTE_DATA",
  "params": { "code": "...", "start_date": "...", "end_date": "..." }
}
```
`start_date`/`end_date` 为 `Option`，None 时省略字段。

**`fetch_minute_klines`（直 reqwest 路径，未知）**：现有 `fetch_klines`（日 K）通过 `fetch_klines.rs::client::fetch_klines` 的 wire 契约已支持 `start_date`/`end_date`（见 `fetch_index_klines` 注释 + eltdx 适配器）。P0.13c 假设 `/data/bars` 接受同样的 `start_date`/`end_date` JSON 字段。**R1 风险**：实际 wire 字段名可能不同（如 `start`/`end`）。

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

`FetchMinuteShare` 同形（无 period/adjust）。

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

Envelope 路径的 wire body 由 `params` JSON 字典扩展：
- `Date(d)` → `params["date"] = "..."`（向后兼容 P0.13b-2 wiremock 测试）
- `Range { start, end }` → `params["start_date"] = "..."`, `params["end_date"] = "..."`

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

`from_cli` 错误示例：`"--date cannot be combined with --start/--end"`。

## 5. 不变量（Invariants）

### INV-1A — date 与 start/end 互斥
CLI `--date` 与 `(--start, --end)` 互斥。同时传入 → `Err`。`from_cli` 在 handler 层强制。

### INV-1B — Range 端点含边界
`start` 与 `end` 都是 inclusive（含）。`start > end` → `Err`。

### INV-2A — 向后兼容 wiremock 测试
现有 P0.13b-1/2 wiremock 测试不破坏。即：
- 旧测试用 `Date(d)` → wire body 仍含 `date` 字段（不变）
- 新测试用 `Range { ... }` → wire body 含 `start_date`/`end_date`，**不**含 `date`

### INV-2B — Vec 扁平
多日返回是扁平 `Vec<MinuteBar>` / `Vec<MinuteShare>`，按 timestamp 升序排列。调用方若需按日分组，自行 `group_by`。

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

### D4：wiremock-first + live-verify `/data/bars` range 字段名
**决策**：P0.13c 假设 `/data/bars` 接受 `start_date`/`end_date` JSON 字段（与 `/data/fetch` envelope 一致）。wiremock 测试先行；live 测试验证。

**原因**：避免在编码前向 OpenStock runtime 查询；wiremock 测试覆盖 happy path；live 测试发现字段名漂移时，同 PR 切换字段名（同 P0.13b-2 R3 模式）。

**替代方案（拒绝）**：先用 live 测试探明字段名。理由：违反 TDD；live 测试不能在 CI 跑。

## 7. 风险

### R1：`/data/bars` range 字段名未知
**风险**：实际 OpenStock runtime 可能用 `start`/`end`（无 `_date` 后缀）或其他命名。

**缓解**：D4 — wiremock 测试假设 `start_date`/`end_date`；live 测试验证；若漂移，单 PR 切换字段名（不影响类型签名）。

### R2：超大范围导致响应过大
**风险**：用户传 `--start=2020-01-01 --end=2026-06-30` 可能返回几百万条 minute candles，OOM 或超时。

**缓解**：本切片不引入分页 / streaming（YAGNI）。在 handler 输出加 warning："range returns N records, consider narrowing"；用户可缩小范围。**Non-goal**：分页或游标。

### R3：现有 wiremock 测试破坏
**风险**：扩展 `fetch_minute_klines` 签名为 `DateOrRange` enum 会破坏 P0.13b-1 现有调用。

**缓解**：INV-2A — `Date(d)` 路径 wire body 与 P0.13b-1 完全一致；P0.13b-1 现有 wiremock 测试改 `Date(...)` 形态调用（语义等价，wire 不变）。

### R4：`from_cli` 校验边界
**风险**：`start > end`、`start == end`、`only start`、`only end` 各语义。

**缓解**：D1 + 测试矩阵覆盖 4 种边界（在 §8 测试矩阵列明）。

### R5：范围与 OpenStock 实际接受的上限不一致
**风险**：OpenStock runtime 可能限制单次范围跨度（如最多 30 天）。

**缓解**：本切片不感知 runtime 限制；runtime 返回错误时 envelope path 失败冒泡；handler 错误消息清晰。

## 8. 测试矩阵

### 8.1 Unit tests (`DateOrRange`)

| ID | 描述 | 输入 | 期望 |
|----|------|------|------|
| U1 | date only | `(Some("2026-06-30"), None, None)` | `Date(2026-06-30)` |
| U2 | start+end | `(None, Some("2026-06-01"), Some("2026-06-30"))` | `Range{start:Some(...), end:Some(...)}` |
| U3 | start only | `(None, Some("2026-06-01"), None)` | `Range{start:Some(...), end:None}` |
| U4 | end only | `(None, None, Some("2026-06-30"))` | `Range{start:None, end:Some(...)}` |
| U5 | date + start conflict | `(Some("X"), Some("Y"), None)` | `Err` |
| U6 | start > end | `(None, Some("2026-06-30"), Some("2026-06-01"))` | `Err` |

### 8.2 Wiremock tests (`fetch_minute_klines` + `fetch_minute_share`)

| ID | 描述 | 验证 |
|----|------|------|
| W1 | minute_klines Range 触发 start_date/end_date body | mock 断言 body 含 `start_date`+`end_date`，**不**含 `date` |
| W2 | minute_klines Date 路径仍只发 date body | mock 断言 body 含 `date`，**不**含 `start_date`/`end_date`（向后兼容） |
| W3 | minute_share Range 同 W1（envelope params） | mock 断言 `params` 含 `start_date`+`end_date` |
| W4 | minute_share Date 同 W2（向后兼容） | mock 断言 `params` 含 `date` only |

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
| `src/data/models.rs` 或 `src/sources/openstock_client.rs` | Modify（追加 `DateOrRange` enum + `from_cli` + `populate_body`） | +80 |
| `src/sources/openstock_client.rs` | Modify（`fetch_minute_klines` + `fetch_minute_share` 签名扩展，wire body 用 populate_body） | +50 |
| `src/cli/commands/data.rs` | Modify（`FetchMinuteKlines` + `FetchMinuteShare` 加 `--start`/`--end` flags） | +20 |
| `src/cli/handlers/openstock_handler.rs` | Modify（2 handlers 增加 start/end 参数 + from_cli 校验） | +30 |
| `src/cli/handlers/app_shell.rs` | Modify（2 dispatcher arms 传递新参数） | +6 |
| P0.13b-1/2 现有 wiremock tests | Modify（改用 `Date(...)` 形态调用，wire 不变） | +0 (no line change, just call site) |
| `tests/openstock_live_minute_klines.rs` | Modify（追加 L1 多日范围 test） | +20 |
| `tests/openstock_live_minute_share.rs` | Modify（追加 L2 多日范围 test） | +20 |
| `openspec/changes/openstock-data-consumption-p0-13c/*` | Create (4 files) | +200 |
| `.governance/programs/project-governance/cards/P0.13c.yaml` | Create | +30 |

总改动：~450 行新增，~30 行修改（call site）。

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
- 其他 category 的范围扩展
