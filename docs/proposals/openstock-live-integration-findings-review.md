# 审核意见：openstock-live-integration-findings.md

> 审核日期：2026-07-01
> 审核范围：`docs/proposals/openstock-live-integration-findings.md` 全文
> 审核基线：HEAD `4557768`，对照源码 `src/sources/openstock_client.rs`、`src/sources/openstock_codes.rs`、`src/sources/openstock_index.rs`、`src/sources/openstock_calendar.rs`、`src/cli/handlers/openstock_handler.rs`、`tests/openstock_live_index.rs`

---

## 总体评价

文档结构清晰，五类发现（Q1–Q5）和五条 OpenStock 侧建议（O1–O5）都附带了具体证据、代码位置和可操作方案。作为联调记录，质量较高。以下三条需修正。

---

## 必须修正

### 1. Q2：`tradeStatus` 的 serde alias 有事实性错误

**位置**：提案第 93 行

```rust
#[serde(default, alias = "trade_status")] pub listing_date: Option<String>,
```

**问题**：runtime 返回的字段名是 `tradeStatus`（camelCase），不是 `trade_status`（snake_case）。serde 的 `alias` 做的是**精确字符串匹配**——`alias = "trade_status"` 永远匹配不到 `tradeStatus`。

**证据**：提案自身 §二.O2 的 curl 示例返回 `{"code":"sh.000001","tradeStatus":"1","code_name":"上证综合指数"}`——字段名是 `tradeStatus`（驼峰），不是 `trade_status`（下划线）。

**修正方案**（二选一）：

- **方案 A**（推荐）：在 struct 上加 `#[serde(rename_all = "camelCase")]`，字段用 Rust 惯例 `trade_status`，serde 自动做 `tradeStatus` ↔ `trade_status` 转换。
- **方案 B**：直接写 `alias = "tradeStatus"`（精确匹配），但需注意 `code_name` 是 snake_case，不能统一 rename_all。

---

### 2. Q2：`tradeStatus` 语义上不等于 `listing_date`

**问题**：runtime 的 `tradeStatus: "1"` 更可能是「交易状态」（1=正常交易，0=停牌/退市），不是「上市日期」。把 `tradeStatus` alias 到 `listing_date` 是语义错误——即使 serde 解析通过，后续 `parse_listing_date("1")` 会把 "1" 当日期解析，必然报 `InvalidCode`。

**建议**：按提案自身提到的「或独立字段」方案走——

```rust
#[serde(rename_all = "camelCase")]
pub struct StockListRecord {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default, alias = "code_name")]
    pub name: Option<String>,
    #[serde(default)]
    pub market: Option<String>,
    #[serde(default)]
    pub listing_date: Option<String>,       // runtime 当前不返，保持 Option
    #[serde(default)]
    pub trade_status: Option<String>,       // 新增独立字段
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
```

`listing_date` 不加 alias，等 runtime 补齐字段后自动生效。

---

### 3. Q1：live test 未覆盖日期过滤

**问题**：`tests/openstock_live_index.rs` 中 live test 调用：

```rust
client.fetch_index_klines(&symbol, None, None)
```

即不带任何日期参数。即使 Q1 的参数名修好了，现有 live smoke test 也不会验证日期过滤是否生效。

**补充发现**：`validate_openstock_index` handler（`openstock_handler.rs:158-190`）的 `_start`/`_end` 参数带下划线前缀（未使用），注释写「kept for symmetry with validate-live」。当前验证路径也不检验日期范围，与 Q1 的日期参数被忽略现象一致。

**建议**：在提案 §三 推进顺序中增加一条——修完参数名后，给 live test 增加 `OPENSTOCK_LIVE_START` / `OPENSTOCK_LIVE_END` 环境变量驱动的变体，验证返回的 K 线日期落在请求区间内。

---

## 已验证准确的部分

| 发现 | 结论 | 证据 |
|------|------|------|
| Q1 参数名不一致 | ✅ 诊断正确 | `fetch_trade_dates`（同源 baostock）用 `start_date`/`end_date`（`openstock_client.rs:340-345`），`fetch_index_klines` 用 `start`/`end`（`openstock_client.rs:357-361`），注释已在 329-333 行注明 baostock contract 是 `start_date`/`end_date` |
| Q3 STOCK_CODES extra 透传 | ✅ 正确 | `StockCodeRecord.extra` 已经是 `HashMap` catch-all，`symbol`/`market` 会被自动捕获 |
| Q4 字符串数值解析 | ✅ 已闭环 | `IndexKlineRecord` 字段是 `Option<serde_json::Value>`，`parse_decimal`/`parse_volume` 的 `Value::String` 分支存在（`openstock_index.rs:152-176`） |
| Q5 WORKDAYS false | ✅ 标记合理 | 需 runtime 侧确认 2026-07-01 是否交易日 |

---

## 对推进顺序的补充

当前 §三 第 1 步「用 curl 对照三种 payload」是对的，但建议在此之前先读 `fetch_trade_dates` 已有的 doc comment（`openstock_client.rs:329-333`）：

> Runtime contract (`baostock._fetch_trade_dates`): accepts `start_date` / `end_date`

INDEX_KLINES 同样是 baostock provider，极大概率参数名一致。对照实验可以简化为只测 `start_date`/`end_date` 单种变体，省掉 `start`/`end` 和不带日期两轮。

---

## 总结

| 维度 | 评价 |
|------|------|
| 证据充分性 | ✅ 每条发现都有 curl 复现命令或代码行号引用 |
| 技术准确性 | ⚠️ Q2 的 serde alias 写法有事实错误，语义映射也需调整 |
| 可操作性 | ✅ 每条都有明确的修改方案和代码位置 |
| 测试覆盖意识 | ⚠️ 未注意到 live test 不验证日期过滤，建议补充 |

**结论**：Q2 的两处问题修正后，本文档即可作为实施依据。
