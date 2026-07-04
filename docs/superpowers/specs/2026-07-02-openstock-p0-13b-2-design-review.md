# 审核意见：P0.13b-2 分时点序列设计文档

> 审核日期：2026-07-03
> 审核范围：`docs/superpowers/specs/2026-07-02-openstock-p0-13b-2-minute-share-design.md` 全文
> 审核基线：HEAD（P0.13b-1 已合并于 `8a1a8f2`），对照 `src/sources/openstock_client.rs:180-239`（`fetch<T>` 方法签名 + wire contract）

---

## 总体评价

设计文档在父设计 R1 审核基础上修正了关键问题——`MinuteShare` 字段全部改为 `Option`（支持 INV-2C skip 语义）、决策记录 D1-D6 每条都有 rejected alternatives、R2 的 time_minutes 格式歧义用 D4 双格式容错处理。与 P0.13b-1 的架构对比表（§3.2）清晰标注了两个子切片的端点/retry/字段差异。但有一处**阻断级** API 调用错误——`fetch<T>()` 方法的调用方式与实际签名不匹配。

---

## 阻断级问题

### 1. `fetch_minute_share` 中 `self.fetch::<T>()` 调用签名不匹配（**CRITICAL**）

**位置**：设计 L140-145

```rust
// 设计中的写法（错误）
let body = serde_json::json!({
    "category": "MINUTE_DATA",
    "code": code,
    "date": date.format("%Y-%m-%d").to_string(),
});
let envelope = self.fetch::<RawMinuteRecord>(body).await?;
```

**实际方法签名**（`openstock_client.rs:180-183`）：

```rust
pub async fn fetch<T: DeserializeOwned>(
    &self,
    category: &str,        // ← 参数 1：category 名
    params: Value,         // ← 参数 2：params 对象
) -> Result<OpenStockResponse<T>>
```

**`fetch` 内部构造的请求体**（L205-208）：

```json
{
    "data_category": "MINUTE_DATA",
    "params": { "code": "...", "date": "..." }
}
```

即 `fetch` 接收 `(category_name, params_object)` 两个参数，自行拼装成 `{data_category: ..., params: ...}` 的 envelope 格式。

**错误有两层**：

| 层 | 设计做的事 | 实际应该做的事 |
|----|-----------|-------------|
| 参数个数 | `self.fetch::<T>(body)` — 单个 JSON body | `self.fetch::<T>(category, params)` — 两个参数 |
| body 内容 | `{"category": "MINUTE_DATA", "code": ..., "date": ...}` | `category` 作为第一个 `&str` 参数传入；`params` 仅含 `{"code": ..., "date": ...}` |

**影响**：`fetch_minute_share` 方法编译失败——函数签名不匹配。

**正确写法**（对齐 `fetch_stock_codes` / `fetch_trade_dates` / `fetch_index_klines` 的调用模式）：

```rust
pub async fn fetch_minute_share(
    &self,
    code: &str,
    date: NaiveDate,
) -> Result<Vec<MinuteShare>> {
    let params = serde_json::json!({
        "code": code,
        "date": date.format("%Y-%m-%d").to_string(),
    });
    let resp = self.fetch::<RawMinuteRecord>("MINUTE_DATA", params).await?;
    let mut out = Vec::with_capacity(resp.records.len());
    for raw in resp.records {
        if let Some(share) = parse_minute_share(code, &raw, date) {
            out.push(share);
        } else {
            tracing::warn!(...);
        }
    }
    Ok(out)
}
```

---

## 需要修正

### 2. `RawMinuteRecord` 字段类型与 `parse_minute_share` 的 `Option` 解包可简化

**问题**：`RawMinuteRecord`（L112-118）的 `price/amount/avg_price` 定义为 `Option<f64>`，但 `parse_minute_share`（L178-181）用 `raw.price?` 解包后还需要 `Decimal::from_f64_retain(price)?` 二次转换。

**简化方案**：既然目标类型是 `Decimal`，`RawMinuteRecord` 可以直接用 `Option<Decimal>`：

```rust
struct RawMinuteRecord {
    time_minutes: String,
    price: Option<Decimal>,
    volume: Option<i64>,
    amount: Option<Decimal>,
    avg_price: Option<Decimal>,
}
```

serde + `rust_decimal` 的 `serde` feature（已在 `Cargo.toml` 启用）会自动将 JSON number 反序列化为 `Decimal`，无需 `from_f64_retain` 的中间跳。同时消除了 R3 的精度损失风险（`from_f64_retain` 对某些 float 值返回 None）。

**注意**：eLtdx 源如果输出字符串格式的数值（如 `"10.50"`），则 `Decimal` 的 serde 反序列化会失败——需要确认 OpenStock MINUTE_DATA 的数值字段是 JSON number 还是 string。如果存在漂移风险（类似 P0.9 的 `IndexKlineRecord` 教训），则 `serde_json::Value` + `parse_decimal` 模式（复刻 `openstock_index.rs:152-176`）更稳妥。

**建议**：在 D2 中明确数值字段的 serde 策略。如果确认为 JSON number → 直接用 `Decimal`；如果存在 string 风险 → 用 `serde_json::Value` + 已有 `parse_decimal` helper。

---

### 3. Handler 中 `date` 变量名 shadow 不清晰

**位置**：设计 L213-222

```rust
pub(crate) async fn fetch_openstock_minute_share(
    settings: &OpenStockSettings,
    symbol: String,
    date: String,                    // ← 参数：用户输入的原始字符串
) -> Result<()> {
    let client = OpenStockClient::from_settings(settings)?;
    let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")  // ← shadow 为 NaiveDate
        .map_err(|e| QuantixError::Other(...))?;
    ...
    println!("  Date:     {}", date);  // ← 打印 NaiveDate，行为正确但变量名混淆
```

虽然 `NaiveDate` 实现了 `Display` 且格式与输入一致（`"2026-06-30"`），但 `date` 同时承担了 "用户输入的原始字符串" 和 "解析后的 NaiveDate" 两种语义。建议分开命名：

```rust
let date_input = date;  // 或直接用 date_str
let date = NaiveDate::parse_from_str(&date_input, "%Y-%m-%d")?;
```

---

## 次要意见

| 位置 | 意见 |
|------|------|
| L140 `"category"` 字段名 | 实际 wire 字段名是 `"data_category"`（见 `fetch` 方法 L206），不是 `"category"`。PARAMS 内部的字段直接扁平到 params JSON 中，不需要额外包裹 |
| L52 `OpenStockEnvelope<RawMinuteRecord>` | 实际返回类型是 `OpenStockResponse<RawMinuteRecord>`（`fetch` 返回 `OpenStockResponse`，不是 `OpenStockEnvelope`）。但 `resp.records` 访问方式相同 |
| L228 `settings.base_url` | `OpenStockSettings.base_url` 是 `Option<String>`，直接 fmt 会打印 `Some("http://...")` 或 `None`。应改为 `settings.base_url.as_deref().unwrap_or("(not set)")` |

---

## 已验证正确的部分

| 设计声明 | 结论 | 证据 |
|----------|------|------|
| `MinuteShare` 字段全部为 `Option` | ✅ | 支持 INV-2C skip 语义（修正了父设计 R1 审核中的 struct 定义冲突） |
| D3 parser 返回 `Option` 而非 `Result` | ✅ | INV-2C 的正确实现："不中断整批" |
| D4 timestamp 双格式容错 | ✅ | "0930" 和 "09:30" 都接受，防御 R2 格式歧义 |
| D1 内联 category 字符串（不抽常量） | ✅ | 与 `fetch_stock_codes("STOCK_CODES", ...)` 风格一致 |
| D5 内联 parser（不抽独立模块） | ✅ | 与 P0.13b-1 `fetch_minute_klines` 的 inline `MinuteBarRecord` 模式一致 |
| D6 Decimal 精度 | ✅ | 与 `Kline`/`MinuteBar` 统一 |
| §3.2 与 P0.13b-1 的对比表 | ✅ | 端点（`/data/bars` vs `/data/fetch`）、retry（无 vs 有）、字段（OHLC vs price+avg_price）对比正确 |
| INV-1A envelope 路径 | ✅ | 复用 `fetch::<T>()` 即自动获得 retry + circuit breaker |
| INV-2C skip 语义 | ✅ | struct 用 `Option` + parser 返回 `Option` |
| R4 envelope 失败与 INV-2C 的维度区分 | ✅ | 正确解释了整批失败 vs 单条 skip 的语义区别 |
| 测试矩阵 7+3 测试 | ✅ | 4 unit + 3 wiremock + 3 live，覆盖合理 |

---

## 总结

| 维度 | 评价 |
|------|------|
| 上一轮审核修正 | ✅ `MinuteShare` 的 `Option` 字段、INV-2C 语义、D3 决策全部修正 |
| API 调用正确性 | ❌ `self.fetch::<T>(body)` 双参数签名不匹配——编译失败 |
| `RawMinuteRecord` 数值策略 | ⚠️ `Option<f64>` + `from_f64_retain` 绕路——直接用 `Decimal` 或 `Value` 更简洁安全 |
| 决策完整性 | ✅ D1-D6 每条有 rejected alternatives |
| 风险覆盖 | ✅ R1-R5 覆盖 schema 未知/格式歧义/精度/envelope/并发 |
| 实施可行性 | ✅ 文件清单 12 项，~600 行新增，0 删除 |

**结论**：修正 `fetch::<T>()` 的调用签名（对齐 `fetch_stock_codes` 的 `self.fetch("MINUTE_DATA", params)` 双参数模式）、评估 `RawMinuteRecord` 直接用 `Decimal` 的可行性后，即可编写实施计划。
