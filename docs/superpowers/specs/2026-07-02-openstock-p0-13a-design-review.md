# 审核意见：P0.13a Multi-period K-line Fetch 设计文档

> 审核日期：2026-07-02
> 审核范围：`docs/superpowers/specs/2026-07-02-openstock-p0-13a-multi-period-klines-design.md` 全文
> 审核基线：HEAD（P0.11c 已合并），对照 `src/sources/openstock_client.rs`、`src/data/models.rs`、`src/sources/kline_aggregator.rs`、`src/cli/commands/data.rs`

---

## 总体评价

设计文档结构清晰——六个 Decision 都有三选项对比和推荐值，Architecture 层的 `/data/bars` 路径、Invariants 四条、Error Handling 矩阵、8 测试 3 阶段实施计划都写得具体。但有一条**阻断级**问题和两条需要修正的细节。

---

## 阻断级问题

### 1. `KlinePeriod` 名称冲突——与已有类型碰撞（**CRITICAL**）

**问题**：设计 §Components 1 提议在 `src/data/models.rs` 新建 `KlinePeriod` 枚举：

```rust
pub enum KlinePeriod { Day, Week, Month }
```

但 `KlinePeriod` **已经存在**于 `src/sources/kline_aggregator.rs:14`：

```rust
pub enum KlinePeriod {
    OneMinute, FiveMinutes, FifteenMinutes,
    ThirtyMinutes, OneHour, Daily,
}
```

该类型已在 `src/sources/mod.rs:29` 公开 re-export，`as_str()` 返回 `"1m"`/`"5m"`/`"1d"` 等聚合器格式字符串。新建同名类型会导致：

- **编译冲突**：`src/sources/mod.rs` 同时 re-export 两个 `KlinePeriod`，`use` 语句歧义
- **语义混乱**：聚合器用 `KlinePeriod` 表示时间窗口（1min/5min/1d），OpenStock 用同名类型表示 API period 参数（day/week/month），两套语义不可互换
- **调用方困惑**：现有聚合器消费者依赖 `KlinePeriod::Daily`，新类型有 `KlinePeriod::Day`——两个不同的 `Day`/`Daily` 变体共存

**建议**（三选一）：

| 方案 | 做法 | 优劣 |
|------|------|------|
| **A** | 新类型命名为 `BarPeriod`（或 `OpenStockPeriod`），放在 `src/data/models.rs` | 无冲突，语义清晰。推荐 |
| **B** | 扩展已有 `kline_aggregator::KlinePeriod` 加 `Week`/`Month` 变体 | 聚合器目前只处理分钟级聚合，语义域不匹配（聚合器不需要 Week/Month 窗口） |
| **C** | 新类型放在 `src/sources/openstock_client.rs` 内部，不公开 | 局限：handler 和 CLI 需要引用该类型，内部类型不够用 |

**推荐方案 A**：`BarPeriod` 直接映射 OpenStock `/data/bars` 的 `period` 参数，与聚合器的 `KlinePeriod`（时间窗口）职责分离，消费者一目了然。

---

## 需要修正

### 2. `AdjustType::as_openstock_param()` 对 `None` 返回 `""` 与现有行为不一致

**问题**：设计 L125 写 `Self::None => ""`，即发送 `"adjust": ""`。但已有的 `fetch_daily_klines`（`openstock_client.rs:492-495`）**根本不发送 `adjust` 字段**：

```rust
let mut body = serde_json::json!({
    "symbol": code,
    "period": "day",
    // ← 没有 "adjust" 键
});
```

新 `fetch_klines` 如果发送 `"adjust": ""`，OpenStock `/data/bars` 可能把空字符串当有效值处理（e.g. 尝试匹配名为 `""` 的 adjust provider），行为与不发送字段不同。

**建议**：`as_openstock_param()` 改为返回 `Option<&str>`，`None` 时跳过字段而非发送空字符串。或者在 `fetch_klines` 中条件判断：`adjust != AdjustType::None` 时才 `body["adjust"] = ...`。

---

### 3. `KlinePeriod::FromStr` 未声明大小写敏感性——与 `AdjustType::FromStr` 不一致

**问题**：设计 L134-135 明确写了 `AdjustType::FromStr` "Case-insensitive on input"。但 L104-108 的 `KlinePeriod::FromStr` 只写了 "Strict per decision D6: rejects daily/weekly/monthly aliases"，**未声明大小写行为**。

用户输入 `Day`（首字母大写）或 `DAY`（全大写）时行为未定义。如果 `FromStr` 不做 `to_lowercase()` 预处理，`Day` → 匹配失败 → 报错。而 `AdjustType` 明确做了大小写不敏感处理。

**建议**：`KlinePeriod::FromStr` 应明确声明 "Case-insensitive, accepts only `day`/`week`/`month` (any case)，rejects `daily`/`weekly`/`monthly`/`minute*`"。两个 `FromStr` 的行为应一致。

---

## 次要意见

| 位置 | 意见 |
|------|------|
| D1 描述 "C — week/month + qfq/hfq" | `Day` 也在 scope 内（枚举包含 Day，Phase 1 测试 day+None），建议改为 "day/week/month + qfq/hfq" |
| D5 测试计数 "8 tests across 3 layers" | T1(unit) + T2(unit) + T3-T5(wiremock) + T6-T8(live) = 8，wiremock 归入 unit 层——表述可更精确为 "5 unit/wiremock + 3 live" |
| T2 位置 "src/data/models.rs #[cfg(test)] or same file" | "same file" 歧义——指 `openstock_client.rs` 还是 `data/models.rs`？建议明确 |
| L125 `"hfq"` → 但 L134 写的是 `"hff"` | L134 是笔误：`"none" \| "qfq" \| "hff"` 应为 `"hfq"` |
| `/data/bars` symbol 前缀行为 | 已存在的行为（`fetch_daily_klines` 不调用 `normalize_symbol`）：对 `sh000001`，`Kline.code` = `"sh000001"`（含前缀）。与 `/data/fetch` 的 `INDEX_KLINES` 路径（`normalize_symbol` 剥前缀 → `"000001"`）不一致。设计应文档化此差异（建议在 Invariants 加一条） |
| `f64` → `Decimal` 精度损失 | `BarRecord` 用 `f64` 反序列化后 `format!("{}", x)` → `from_str` 转 `Decimal`——这是已有路径（`fetch_daily_klines`），设计正确标注 "No new parser"，属于显式接受的 tech debt |

---

## 已验证正确的部分

| 设计声明 | 结论 | 证据 |
|----------|------|------|
| `fetch_daily_klines` 已存在，走 `/data/bars` 端点 | ✅ | `openstock_client.rs:481-568` |
| `/data/bars` 响应 shape `{data: [{time, open, high, low, close, volume, amount}]}` | ✅ | `BarRecord` struct L532-540 匹配 |
| `AdjustType` 枚举 `None`/`QFQ`/`HFQ`，无线 `FromStr` | ✅ | `models.rs:25-29`，grep 确认无 `impl FromStr for AdjustType` |
| `Kline` 结构体含 `adjust_type: AdjustType` | ✅ | `models.rs:11-21` |
| `OpenStockCommands` 已含 FetchCodes/FetchCalendar/FetchIndex 等 | ✅ | `commands/data.rs:176-361`，FetchKlines 同层新增合理 |
| D3 新增 `fetch_klines` 不修改 `fetch_daily_klines` | ✅ | `fetch_daily_klines` 硬编码 `"period": "day"` + `AdjustType::None` |
| D2 请求驱动复权、response 不 echo `adjust_type` | ✅ | 与现有架构一致 |
| 不变更 `Kline` 数据模型 | ✅ | `Kline { adjust_type }` 已覆盖 3×3 组合 |
| 无 retry/circuit-breaker（对齐 `fetch_daily_klines`） | ✅ | `/data/bars` 直接 reqwest，不经过 `fetch<T>()` 的 breaker 路径 |

---

## 总结

| 维度 | 评价 |
|------|------|
| 架构决策 | ✅ D1-D7 决策清晰、推荐默认值合理 |
| 与现有代码一致性 | ❌ `KlinePeriod` 名称与 `kline_aggregator.rs` 碰撞——阻断级 |
| API 参数兼容性 | ⚠️ `as_openstock_param()` 空字符串 vs 不发送字段的行为差异 |
| 错误处理设计 | ✅ 五类错误的路径和结果定义完整 |
| 测试覆盖 | ✅ 3 层 8 测试覆盖请求构造/解析/live 端到端 |
| 实现计划 | ✅ 3 Phase / 3 Commit 分解合理 |

**结论**：解决 `KlinePeriod` 名称冲突（推荐改名为 `BarPeriod`）、修正 `as_openstock_param()` 的 None 行为、补全 `FromStr` 大小写声明后，即可进入 Phase 1 实现。
