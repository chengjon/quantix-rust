# 审核意见：P0.13b 分钟级数据消费设计文档

> 审核日期：2026-07-02
> 审核范围：`docs/superpowers/specs/2026-07-02-openstock-p0-13b-minute-level-data-design.md` 全文
> 审核基线：HEAD（P0.13a 已合并），对照 `src/db/tdengine.rs`、`src/sources/openstock_client.rs`、`src/core/runtime/settings.rs`、`src/cli/handlers/openstock_handler.rs`

---

## 总体评价

双子切片的设计框架清晰——分钟蜡烛走 `/data/bars`（与 P0.13a 同路径）、分时点序列走 `/data/fetch` envelope（与现有 fetch-* 同路径），8 个 Decision 每一条都有明确原因。HANDOFF 报告行 35 的 `MINUTE_DATA` mislabel 纠正是一个很有价值的发现。`_PERIOD_MAP` 静默回退到 day 的风险分析和 D4 的严格白名单防御是正确的设计决策。但有一条**阻断级**类型名称冲突和三条需要修正的 API 细节。

---

## 阻断级问题

### 1. `MinuteKline` 名称与 `src/db/tdengine.rs:37` 已有类型碰撞（**CRITICAL**）

**问题**：设计 §4.1.2 提议在 `src/data/models.rs` 新建：

```rust
pub struct MinuteKline {
    pub timestamp: NaiveDateTime,
    pub code: String,
    pub open: Decimal,
    // ...
    pub adjust_type: AdjustType,
}
```

但 `MinuteKline` **已经存在**于 `src/db/tdengine.rs:37`，且已通过 `src/db/mod.rs:17` 公开 re-export：

```rust
/// 分钟 K线数据
pub struct MinuteKline {
    pub ts: DateTime<Utc>,
    pub code: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}
```

两套 `MinuteKline` 语义不同：TDengine 版用 `DateTime<Utc>` + `f64`（数据库行映射），设计版用 `NaiveDateTime` + `Decimal` + `AdjustType`（API 消费模型）。`src/db/mod.rs` 同时 re-export 两个同名类型 → 编译歧义。

**建议**（三选一）：

| 方案 | 做法 | 优劣 |
|------|------|------|
| **A**（推荐） | 命名为 `MinuteBar`，放在 `src/data/models.rs` | 无冲突，语义映射 OpenStock `/data/bars` 的 minute-period 蜡烛。对称于 P0.13a 的 `BarPeriod` |
| **B** | 命名为 `OpenStockMinuteKline` | 无冲突但冗长 |
| **C** | 放在 `src/sources/openstock_client.rs` 内部不公开 | 局限：handler 和测试需要引用，内部类型不够用 |

推荐 **方案 A**——`MinuteBar` 与 `BarPeriod` 形成语义对：`BarPeriod` 是请求参数，`MinuteBar` 是响应数据。

---

## 需要修正

### 2. 类型名错误：`OpenStockClientSettings` → 应为 `OpenStockSettings`

**位置**：设计 L278

```rust
// 当前（错误）
pub(crate) async fn fetch_openstock_minute_klines(
    settings: &OpenStockClientSettings,  // ← 不存在
```

**实际类型**：`src/core/runtime/settings.rs:74` 中的 `OpenStockSettings`。所有现有 handler（`openstock_handler.rs:192/244/309/354`）都用 `&OpenStockSettings`。

**同时**：L291 的 `OpenStockClient::new(settings.clone())` 也不对——`new()` 接受 `OpenStockClientConfig`，不是 `OpenStockSettings`。现有代码统一使用 `OpenStockClient::from_settings(settings)?`（L193）。

**建议**：L278 改为 `&OpenStockSettings`，L291 改为 `OpenStockClient::from_settings(settings)?`。

---

### 3. `MinuteShare` 字段缺少 `Option` 包裹——与 INV-2C 不一致

**问题**：设计 §4.2.1 的 `MinuteShare`（L321-329）所有字段都是必需类型：

```rust
pub struct MinuteShare {
    pub price: Decimal,       // ← 非 Option
    pub volume: i64,
    pub amount: Decimal,
    pub avg_price: Decimal,
}
```

但 INV-2C（L397）明确写：

> 单条记录字段缺失时整条记录跳过（warn log），不中断整批

如果单个字段缺失，serde 反序列化会直接失败（`missing field`），**无法实现 skip 语义**——因为 struct level 的 `#[derive(Deserialize)]` 会把整条记录当错误冒泡。

**建议**：要么把 `price/volume/amount/avg_price` 改为 `Option<...>`（允许 serde 成功解析，parse 阶段再 `ok_or` + `warn!` + skip），要么 INV-2C 明确写为「serde 解析失败 → 整批报错，不 skip」。当前 struct 定义与 INV-2C 冲突。

---

### 4. 架构图小误：`?period=1m` 应改为 JSON body

**位置**：设计 L129-130

```
OpenStock /data/bars
?period=1m (或 5m/15m/30m/60m)
```

`/data/bars` 是 POST + JSON body（不是 GET + query string），与 P0.13a 的 `fetch_klines` 一致。应改为：

```
OpenStock /data/bars
{ symbol, period: "1m", ... }
```

---

## 次要意见

| 位置 | 意见 |
|------|------|
| L563 CLI smoke 命令 | 写的是 `cargo run -q -- openstock fetch-minute-klines`，但正确路径应为 `cargo run -q -- data openstock fetch-minute-klines`（`FetchMinuteKlines` 挂在 `OpenStockCommands` 下，`OpenStockCommands` 挂在 `DataCommands::OpenStock` 下） |
| L297 `as_openstock_param()` | 返回 `Option<&str>`，`.unwrap_or("none (field omitted)")` 仅用于显示目的——语义正确，但 `Some("qfq")` 之外还有 `Some("hfq")` 两个值，显示时应直接 `adjust_val` 而非硬编码 |
| D3 决策描述 | "MinuteKline 独立结构体（不复用 Kline）" 正确——NaN 精度要求不同，但应补充说明与 TDengine `MinuteKline` 的命名冲突已通过改名解决 |
| D5 决策描述 | "MinuteKline" 残留——应随类型改名同步更新 |
| §7.1 T1 测试 | "`MinuteKline` 序列化往返" → 应随类型改名同步更新 |

---

## 已验证正确的部分

| 设计声明 | 结论 | 证据 |
|----------|------|------|
| `_PERIOD_MAP` 静默回退到 day | ✅ | `_eltdx_timeseries.py:72` — `return _PERIOD_MAP.get(period, "day")` |
| D4 严格白名单拒绝别名 | ✅ | 防御 `map_period` 默认值陷阱，设计合理 |
| P0.13b-1 复用 `/data/bars`，直 reqwest | ✅ | 与 P0.13a `fetch_klines` 同路径 |
| P0.13b-2 走 `/data/fetch` envelope + retry | ✅ | 与 `fetch_stock_codes` / `fetch_index_klines` 同路径 |
| D3 不扩展 `BarPeriod`（独立 `MinutePeriod`） | ✅ | 返回类型不同（`Vec<MinuteBar>` vs `Vec<Kline>`），类型系统强制区分 |
| `MINUTE_DATA` 8 字段 → 5 字段裁剪 | ✅ | 丢弃 `index/time/price_milli`，收口明确 |
| Non-Goals 清晰 | ✅ | 不写 DB、不重构 envelope、不碰 P0.13a |
| 风险矩阵 R1-R5 | ✅ | 每条有缓解措施，R2（time 字段格式未知）用 wiremock 先行探路 |
| INV-1D（None 时省略 adjust 字段） | ✅ | 与 P0.13a v2 设计对齐 |

---

## 总结

| 维度 | 评价 |
|------|------|
| 双子切片分解 | ✅ 分钟蜡烛 vs 分时点的端点/模型/客户端路径完全正交，分开合理 |
| 与现有代码一致性 | ❌ `MinuteKline` 与 `tdengine.rs` 碰撞——阻断级 |
| API 细节准确性 | ⚠️ `OpenStockClientSettings` 不存在、`new()` 签名错误 |
| 不变量可测试性 | ⚠️ INV-2C（skip 缺失字段）与 `MinuteShare` struct 定义冲突 |
| 风险防御 | ✅ R1 静默回退 + D4 白名单是最强防御 |
| 决策文档化 | ✅ D1-D8 每条有原因，`_PERIOD_MAP` 源码引用精确到行 |

**结论**：解决 `MinuteKline` 名称冲突（推荐改名为 `MinuteBar`）、修正 `OpenStockClientSettings` → `OpenStockSettings`、调和 INV-2C 与 struct 定义后，即可进入 P0.13b-1 实现。
