# P0.11b 实施审核文档

> **会话日期**: 2026-07-01
> **OpenSpec 变更**: `openstock-data-consumption-p0-11`
> **提交**: `47747c5 feat(data): P0.11b import-ticks --source openstock (dry-run + apply)`
> **状态**: P0.11b 代码完成；待 live 验证（2b.10）+ spec/design 文档补遗
>
> **修订**: r2 — 已根据 `P0_11B_IMPLEMENTATION_REVIEW_AUDIT_2026-07-01.md` 意见修复 §七第 4、5 条（代码层面），补充 §二问题 4 的 legacy 语义不一致说明（P0.11c 决策项）。

---

## 一、本次会话完成的任务

P0.11 切分为三个子片（a/b/c），本会话完成了 **P0.11b** 的代码与单元测试。任务清单对齐 `openspec/changes/openstock-data-consumption-p0-11/tasks.md` §2b：

| 任务 | 状态 | 说明 |
|---|---|---|
| 2b.3 `fetch_tick_data` wrapper | ✅ | 关键修复：参数名 `symbol`，不是 `code` |
| 2b.4 `src/sources/openstock_ticks.rs` 解析器 | ✅ | 全新文件，处理嵌套信封形状 |
| 2b.5 8 个 fixture 单元测试 | ✅ | 覆盖 happy / empty / missing / 数值漂移 |
| 2b.6 `ImportTicks` 增加 `--source` | ✅ | 默认 `openstock`，保留 tdx-api legacy |
| 2b.7 `import_ticks` handler 分支 | ✅ | dry-run 默认；双闸门写入 |
| 2b.8 Live smoke 测试文件 | ✅ | `QUANTIX_OPENSTOCK_LIVE=1` 网关 |
| 2b.9 Quality gates | ✅ | fmt / clippy / 1446 tests 全绿 |
| 2b.10 Live 测试实跑 | ⏳ | 待用户确认运行（需 live 环境） |

---

## 二、解决的问题（代码层面）

### 问题 1：TICK_DATA 参数名陷阱 — `symbol` 而非 `code`

**根因**：OpenStock 的 `INDEX_KLINES` / `HISTORICAL_KLINES` 用 `code` 参数，但 `TICK_DATA` 由 `eltdx` 适配器提供，契约要求 `symbol`。在 P0.11b smoke 阶段（2026-07-01，提交 5676fba）实测确认：

```
curl ... -d '{"data_category":"TICK_DATA","params":{"code":"600000",...}}'
→ HTTP 422 "symbol is required for TICK_DATA"

curl ... -d '{"data_category":"TICK_DATA","params":{"symbol":"600000",...}}'
→ HTTP 200, 456 KB, 1800 ticks
```

**代码解决**（`src/sources/openstock_client.rs`）：

```rust
pub async fn fetch_tick_data(
    &self,
    symbol: &str,
    date: Option<&str>,
) -> Result<OpenStockResponse<TickEnvelopeRecord>> {
    let mut params = serde_json::json!({ "symbol": symbol });  // ← 关键
    if let Some(date) = date {
        params["date"] = Value::String(date.to_string());
    }
    self.fetch("TICK_DATA", params).await
}
```

**为什么这是真正的修复**：函数签名也叫 `symbol` 而非 `code`，从源头阻止后续调用方按错误习惯传参。doc-comment 明确说明这与 `fetch_index_klines`/`fetch_historical_klines` 不同，并指向 design.md D4.1。

---

### 问题 2：TICK_DATA 嵌套信封形状 — 不能复用 IndexKlineRecord

**根因**：`INDEX_KLINES` 的 `data` 是**平坦**数组：
```json
{ "data": [ {symbol, time, open, high, low, close, ...}, ... ] }
```

而 `TICK_DATA` 的 `data` 是**嵌套**的，一个 envelope-record 包住一批 ticks：
```json
{ "data": [ {
    "meta": { symbol, trading_date, returned_count, price_base, has_more, ... },
    "ticks": [ {trade_datetime, price, volume, amount, side, order_count, status, ...}, ... ]
} ] }
```

OpenStockEnvelope 的泛型 `T` 必须能反序列化 `{meta, ticks}` 这个形状。

**代码解决**（新文件 `src/sources/openstock_ticks.rs`）：

```rust
#[derive(Debug, Deserialize)]
pub struct TickEnvelopeRecord {
    #[serde(default)]
    pub meta: Option<TickMeta>,
    #[serde(default)]
    pub ticks: Vec<TickEntry>,
}

#[derive(Debug, Deserialize)]
pub struct TickEntry {
    #[serde(default)]
    pub trade_datetime: Option<String>,
    #[serde(default)]
    pub price: Option<serde_json::Value>,        // ← 吸收漂移
    #[serde(default)]
    pub volume: Option<serde_json::Value>,
    #[serde(default)]
    pub amount: Option<serde_json::Value>,
    #[serde(default)]
    pub side: Option<String>,
}
```

`parse_tick_data` 返回 `(TickMeta, Vec<Tick>)` — 元组而非单 Vec，因为下游写入 TDengine 时还需要 `meta.trading_date` 用于报告，且 handler 的 dry-run 输出也用它。

---

### 问题 3：数值字段漂移 — 字符串 vs 数字

**根因**：在 P0.9/P0.10 已经踩过这个坑（`IndexKlineRecord` 的 `close` 字段在 baostock 是字符串 `"3040"`，在某些 provider 是数字 `3040`）。TICK_DATA 同样风险，因此 `TickEntry` 的所有数值字段都声明为 `Option<serde_json::Value>`，由 `parse_decimal` / `parse_volume` 统一吸收：

```rust
fn parse_decimal(value: Option<Value>, field: &'static str) -> Result<Decimal, TickParseError> {
    let value = value.ok_or(TickParseError::MissingField(field))?;
    let text = match value {
        Value::String(text) => text,
        Value::Number(number) => number.to_string(),
        other => return Err(TickParseError::InvalidDecimal { field, value: other.to_string() }),
    };
    Decimal::from_str(&text)...
}
```

**单元测试**：`parse_tick_data_string_numerics_ok` 专门覆盖 `"price":"10.50"` 字符串形态。

---

### 问题 4：Tick 方向映射到 TDengine status byte — ⚠️ 与 legacy tdx-api 路径语义不一致

**根因**：quantix 的 `Tick` 模型用枚举 `TradeDirection::{Buy, Sell, Neutral}`，但 TDengine 现有写入接口 `insert_ticks(&[(i64, f64, i32, f64, i32)])` 最后一个 `i32` 是从 tdx-api 继承的 `status` 字段。

**代码解决**（`src/cli/handlers/tdx_api_handler.rs`）：

```rust
let status_i = match t.direction {
    TradeDirection::Buy => 1,
    TradeDirection::Sell => -1,
    TradeDirection::Neutral => 0,
};
```

**⚠️ 与 legacy tdx-api 路径语义不一致（审核反馈）**：

同文件 legacy 分支（第 403 行附近）写的是：

```rust
let amount = price * t.volume as f64 * 100.0;
(ts_ms, price, t.volume, amount, t.status)  // ← 直接透传 tdx-api 原始字节
```

`t.status` 是 `i32`，来自 tdx-api 协议的原始字节，语义未在 quantix 代码中说明（可能是成交类型、撤单标记、买卖方向，或其他）。**两条路径写入同一 TDengine 表的同一 `direction TINYINT` 列，但语义完全不同**：

| 路径 | status 列含义 |
|---|---|
| OpenStock (P0.11b) | `TradeDirection` 映射：Buy=1, Sell=-1, Neutral=0 |
| tdx-api legacy | tdx-api 协议原始字节（语义未知） |

**风险**：下游消费者如果按 status 列筛选，会拿到两组不可比的值，且无法通过 SQL 区分来源。**这是 P0.11c 必须解决的问题**，因为 P0.11c 会删除 tdx-api 路径，但 P0.11c 之前两个路径同时存在。

**P0.11c 决策选项**：
- A. 确认 tdx-api 的 `status` 实际语义（需要查 openstock/tdx-api 文档或代码），统一映射；或
- B. 在 TDengine schema 中拆分 `tdx_status` 和 `direction` 两列，物理隔离；或
- C. 在 P0.11c 删除 tdx-api 路径前，OpenStock 路径写入时加 `source='OPENSTOCK'` 标签列区分（但当前 schema 没有 source 列）。

**当前选择**：1/-1/0 是临时占位，**未对齐 legacy**。等待 P0.11c 决策。

---

### 问题 5：Decimal → f64 精度损失的可控降级

**根因**：`Tick.price` / `Tick.amount` 是 `rust_decimal::Decimal`（精度无损），但 TDengine 的 schema 存 `double`。原 tdx-api 路径已经在做 `t.price as f64 / 1000.0` 这样的损失转换，所以 openstock 路径等价行为是可接受的。

**代码解决**：

```rust
fn decimal_to_f64(d: rust_decimal::Decimal) -> f64 {
    use rust_decimal::prelude::ToPrimitive;
    d.to_f64().unwrap_or(0.0)
}
```

私有助于函数，只在 openstock tick 分支使用。失败时降级为 0.0（与 legacy 路径 `unwrap_or(0)` 一致）。

---

### 问题 6：dry-run / apply 双闸门，且不复用 kline 的环境变量

**根因**：P0.11a 引入了 `QUANTIX_OPENSTOCK_KLINE_APPLY=yes` 作为 ClickHouse `kline_data` 主表写入的二次确认。如果 P0.11b 复用这个变量名，操作员在执行 tick 写入时会以为同时确认了 kline 写入（或反之），存在歧义。

**代码解决**：

```rust
if std::env::var("QUANTIX_OPENSTOCK_TICK_APPLY")
    .ok()
    .as_deref()
    != Some("yes")
{
    return Err(QuantixError::Other(
        "已 --apply 但 QUANTIX_OPENSTOCK_TICK_APPLY != yes; 拒绝写入 TDengine".to_string(),
    ));
}
```

引入**新变量** `QUANTIX_OPENSTOCK_TICK_APPLY`，与 kline 那个完全独立。设计文档（design.md D3）已经把这一区分记录在案。

---

### 问题 7：CLI 兼容性 — 不破坏旧调用

**根因**：`ImportTicks` 原本是 `{ code, date }`，新增 `--source` 和 `--apply` 后，旧脚本 `quantix data tdx-api import-ticks --code 600000 --date 20260630` 的行为会**静默改变**（默认从 tdx-api 切到 openstock）。

**代码解决**：
- `--source` 默认 `openstock`（用户必须显式 `--source tdx-api` 才走 legacy）
- `--apply` 默认 `false`（即使 `QUANTIX_OPENSTOCK_TICK_APPLY=yes` 但没加 `--apply`，仍是 dry-run）
- tdx-api legacy 分支保留，加 `eprintln!("⚠️ tdx-api legacy path, scheduled for removal in P0.11c")`

**风险**：旧的 CI 脚本如果跑 `import-ticks --code X --date Y` 期望走 tdx-api，会静默切到 openstock。**这一点需要审核确认** — 是否有任何外部脚本依赖 tdx-api 默认行为？如果没有，方案可接受；如果有，需要临时把默认值改回 `tdx-api` 一个迁移期。

---

## 三、代码改动清单（commit 47747c5）

| 文件 | 类型 | 行数 | 作用 |
|---|---|---|---|
| `src/sources/openstock_ticks.rs` | 新建 | 297 | TickEnvelopeRecord / TickMeta / TickEntry + parse_tick_data + 8 单元测试 |
| `src/sources/openstock_client.rs` | 修改 | +38 | 新增 `fetch_tick_data(symbol, date)` 便利方法 |
| `src/sources/mod.rs` | 修改 | +5 | 注册新模块 + 公开导出 |
| `src/cli/commands/data.rs` | 修改 | +9 | `ImportTicks` 增加 `--source` + `--apply` 字段 |
| `src/cli/handlers/tdx_api_handler.rs` | 修改 | +126/-4 | `import_ticks` 分支重构 + `decimal_to_f64` 私有助手 |
| `tests/openstock_tick_data_live_test.rs` | 新建 | 63 | `#[ignore]` live smoke |
| `openspec/changes/.../tasks.md` | 修改 | +7/-7 | 2b.3-2b.9 标记完成 |

**总计**：7 files changed, 591 insertions(+), 8 deletions(-)

---

## 四、单元测试覆盖（8 个，全部通过）

`src/sources/openstock_ticks.rs::tests` 模块：

| 测试名 | 覆盖的场景 |
|---|---|
| `parse_tick_data_happy` | 标准 envelope-record，2 条 tick（buy + sell），验证 meta 解析 + prefix-strip + Decimal 精度 |
| `parse_tick_data_empty_errors` | `data: []` → `EmptyRecords` 错误 |
| `parse_tick_data_missing_meta_errors` | envelope-record 缺 `meta` → `MissingMeta` |
| `parse_tick_data_missing_trade_datetime_errors` | tick 缺 `trade_datetime` → `MissingField("ticks[].trade_datetime")` |
| `parse_tick_data_string_numerics_ok` | `"price":"10.50"` 字符串数值漂移吸收（P0.9/P0.10 教训） |
| `parse_tick_data_invalid_decimal_errors` | `"price":"not-a-number"` → `InvalidDecimal` |
| `parse_tick_data_unknown_side_defaults_neutral` | `"side":"unknown"` → `TradeDirection::Neutral` |
| `parse_tick_data_mixed_symbol_errors` | 同一 envelope 里两条 record 的 `meta.symbol` 不一致 → `MixedSymbol` |

**质量门**：
- `cargo fmt --check` ✅
- `cargo clippy --workspace --all-targets -- -D warnings` ✅
- `cargo test --workspace` → **1446 passed / 0 failed / 16 ignored**

---

## 五、未完成的事项（需要审核确认/下一步）

### 5.1 待 live 验证（2b.10）

```bash
QUANTIX_OPENSTOCK_LIVE=1 \
OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
OPENSTOCK_API_KEY=sk-ICdVlZ72X59SAIm2TfX3ZzM9qLan5bk5 \
cargo test --test openstock_tick_data_live_test -- --ignored
```

需要确认：fetch_tick_data 在真实 runtime 下能拉到 1800 ticks 左右（与 2026-07-01 手工 smoke 一致），且 parser 不会因为真实响应里的额外字段（`price_milli`、`order_count`、`status`、`price_delta_raw` 等）炸掉。

**风险**：`TickEntry` 用了 `#[serde(default)]` 但没有 `deny_unknown_fields`，所以多出来的字段会被静默丢弃 — 这通常是好事（向前兼容），但如果下游需要这些字段（例如 `order_count` 用于撮合计数），需要在后续切片扩展 `TickEntry`。

### 5.2 待 spec.md 补遗

`openspec/changes/openstock-data-consumption-p0-11/specs/openstock-data-consumption/spec.md` 目前只有 P0.11a 的场景（dry-run kline、apply kline）。P0.11b 需要补：

- `Scenario: TICK_DATA dry-run` — fetch + parse，不写 TDengine
- `Scenario: TICK_DATA apply without env var refused` — `--apply` 但 `QUANTIX_OPENSTOCK_TICK_APPLY != yes` 返回错误
- `Scenario: TICK_DATA apply writes TDengine` — 完整路径
- `Scenario: TICK_DATA parameter name must be symbol` — 文档化 422 trap

### 5.3 待 design.md D4.1 标注完成

D4.1 当前以"Parser design implications"（设计意图）的语气写的，但 parser 已经实现。需要把小节标题改为"TICK_DATA shape (live-verified, parser-shipped)"并在末尾加一句指向 `src/sources/openstock_ticks.rs`。

### 5.4 待 design.md 补一条决策

`D3. P0.11b TDengine path` 只写了一句话（"same dry-run / --apply gate pattern"）。实际实现引入了：
- 新环境变量 `QUANTIX_OPENSTOCK_TICK_APPLY`（与 kline 的隔离决策）
- `TradeDirection → i32` 映射（Buy=1/Sell=-1/Neutral=0）

这两条应该补到 D3，否则未来维护者需要逆向工程 handler 才能理解。

---

## 六、P0.11c 启动条件

P0.11c（删除 `TdxApiClient` + 18 个 CLI 子命令）**仍然被阻塞**，直到：

1. ✅ P0.11a 合并（commit d5e9b75，已完成）
2. ⏳ P0.11b live 测试实跑通过（2b.10，待执行）
3. ⏳ P0.11b spec.md / design.md 补遗完成

**P0.11c 的工作量预估**（按 tasks.md §3c）：
- 代码删除：~1800 行（tdx_api.rs 1309 + tdx_api_handler.rs 476）
- 改动文件：collect_scheduler.rs（fallback 重接）、data_handler.rs:348（DataSourceKind::TdxApi 分支）、data.rs（删除 TdxApi 枚举）、app_shell.rs（dispatcher）
- 文档更新：FUNCTION_TREE.md 5 处行号、README、CHANGELOG、TDX_API_BRIDGE_GUIDE.md
- docker-compose.yml：注释掉 tdx-api 服务

P0.11c 是**纯减法**切片，应该比 a/b 更直接，但 grep 审计（3c.12）必须彻底 — 任何遗漏的 `TdxApiClient` 引用都会编译失败。

---

## 七、审核要点（请重点检查）

1. **status 字段映射**（Buy=1/Sell=-1/Neutral=0）—— **与 legacy tdx-api 路径的 `t.status` 原始字节语义不一致**。两条路径写同一 TDengine 表的同一列，值不可比。需要在 P0.11c 决策统一方案（见第二节"问题 4"末尾的 A/B/C 选项）。当前 P0.11b 选择临时占位，未对齐 legacy。
2. **`--source` 默认切到 `openstock`** 是否会破坏任何 CI / 运维脚本？见第二节"问题 7"。
3. **新环境变量名 `QUANTIX_OPENSTOCK_TICK_APPLY`** 是否合理？还是应该统一为 `QUANTIX_OPENSTOCK_APPLY` 一个变量管所有 openstock 写入？
4. ~~`TickEntry` 丢弃 `price_milli` / `order_count` / `status` 等字段~~ — **已修复（审核反馈）**：`TickEntry` 和 `TickMeta` 加了 `#[serde(flatten)] extra: HashMap<String, Value>`，对齐项目 parser 约定（`StockCodeRecord` 等）。
5. ~~`decimal_to_f64` 失败降级 0.0~~ — **已修复（审核反馈）**：改为 `decimal_to_f64(d, field) -> Result<f64, QuantixError>`，失败时返回错误让调用方决策。tick 循环用 `collect::<Result<Vec<_>>>()?`，任何 tick 转换失败会中止整批写入（不再静默污染数据）。
6. 是否同意把 `parse_tick_data` 的 `(TickMeta, Vec<Tick>)` 元组返回改为命名的 `TickDataBatch { meta, ticks }` 结构体？元组当前没有语义标签，未来扩展（例如加 `quality_flags`）会破坏调用点。
