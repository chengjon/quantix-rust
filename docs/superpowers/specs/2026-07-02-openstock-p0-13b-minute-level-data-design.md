# 设计：P0.13b — OpenStock 分钟级数据消费（双子切片）

> 设计日期：2026-07-02
> 状态：定稿，待实现
> 修订：2026-07-02 (R1) — 修正 CRITICAL 类型名 `MinuteKline` 冲突（`src/db/tdengine.rs:37`）→ 改名 `MinuteBar`；修正 `OpenStockClientSettings` → `OpenStockSettings`；修正 `OpenStockClient::new()` → `from_settings()`；调和 INV-2C 与 `MinuteShare` struct；修正架构图 query string → JSON body；CLI smoke 命令路径修正。详见 `2026-07-02-openstock-p0-13b-design-review.md`。
> 前序切片：P0.13a（多周期日线 K 线拉取，已合并 2026-07-02）
> 切片范围：P0.13b-1（分钟蜡烛 K 线）+ P0.13b-2（分时点序列），**两个独立 OpenSpec change**

---

## 1. 背景与动机

### 1.1 HANDOFF 报告的标注冲突

`docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md` 中存在 **同一 `MINUTE_DATA` 分类被标注为两种不同数据形态** 的错误：

| 行 | 标注 | 分类 | 状态 |
|---|---|---|---|
| 35 | "P1 分钟级 K 线" | `MINUTE_DATA` | ❌ 未接入 |
| 38 | "P2 分时图" | `MINUTE_DATA` | ❌ 未接入 |

实际查 eltdx provider 与 OpenStock scope 后确认：

- **`MINUTE_DATA` 实际是分时点序列**（intraday time-share），字段 `time_minutes/price/price_milli/volume/amount/avg_price`，**无 OHLC**。
- **真正的分钟级 K 线蜡烛**走 `/data/bars?period=1m|5m|15m|30m|60m`，分类是 `KLINES`（不是 `MINUTE_DATA`）。
- 因此**行 35 的标注是错误的**——`MINUTE_DATA` 不是分钟蜡烛。行 38 标注正确。

### 1.2 为什么拆两个子切片

分钟蜡烛（KLINES）与分时点序列（MINUTE_DATA）在以下维度完全不同：

| 维度 | 分钟蜡烛（P0.13b-1） | 分时点序列（P0.13b-2） |
|---|---|---|
| OpenStock 端点 | `/data/bars` | `/data/fetch`（envelope） |
| 数据形态 | OHLCV 蜡烛 | 单点价格序列 |
| Rust 模型 | `MinuteBar`（含 OHLC） | `MinuteShare`（含 avg_price） |
| 客户端方法 | 直 reqwest | envelope + retry + circuit breaker |
| 输入参数 | `code + period + date + adjust` | `code + date` |
| 业务用途 | 短线信号、回测 | 当日盘口走势、VWAP 辅助 |

两个子切片独立交付、独立归档、独立验证，降低单 PR 风险。

---

## 2. 关键事实（已通过代码探查确认）

### 2.1 OpenStock `/data/bars` wire 协议

源：`/opt/claude/openstock/openstock/adapters/_eltdx_timeseries.py:12-32`

```python
_PERIOD_MAP = {
    "day": "day", "daily": "day",
    "week": "week", "weekly": "week",
    "month": "month", "monthly": "month",
    "1m": "1m", "1min": "1m", "minute": "1m",
    "5m": "5m", "5min": "5m",
    "15m": "15m", "15min": "15m",
    "30m": "30m", "30min": "30m",
    "60m": "60m", "60min": "60m",
    "1h": "60m", "hour": "60m",
}
```

**主 token**：`1m|5m|15m|30m|60m`（P0.13b-1 使用这些）
**别名**：`1min|minute|5min|...|1h|hour`（P0.13b-1 **拒绝**，与 P0.13a D6 严格策略一致）

### 2.2 静默回退风险（R1）

源：`_eltdx_timeseries.py:35-36`

```python
def map_period(period):
    return _PERIOD_MAP.get(period, "day")  # ⚠️ 未知 token 静默回退到 day
```

**影响**：客户端如果发送错误 token（如 `minute1`），OpenStock 不报错，**默默返回日线数据**，看似成功实则数据错误。

**防御**：`MinutePeriod::FromStr` 严格白名单（仅 5 个主 token），CLI 解析阶段 fail-fast 报 `QuantixError::Config`，绝不让错误 token 到达 wire。

### 2.3 MINUTE_DATA 实际 schema

源：`_eltdx_timeseries.py::fetch_minute_data` (L181) + `_field_schemas/_eltdx.py:64-72`

完整 8 字段：`index, time, time_minutes, price, price_milli, volume, amount, avg_price`

**字段裁剪决策**：

| 字段 | 保留 | 原因 |
|---|---|---|
| `time_minutes` | ✅ | 业务时间戳（分钟精度） |
| `price` | ✅ | 当时成交价 |
| `volume` | ✅ | 成交量 |
| `amount` | ✅ | 成交额 |
| `avg_price` | ✅ | 均价（重要业务字段） |
| `index` | ❌ | 内部序号，业务无关 |
| `time` | ❌ | ISO 字符串冗余表示，`time_minutes` 已是解析后的时间戳 |
| `price_milli` | ❌ | `price` 的毫表示（测试辅助字段），冗余 |

`MinuteShare` 最终保留 **5 个业务字段** + `code`。

---

## 3. 架构总览

### 3.1 P0.13b-1（分钟蜡烛 K 线）

```
┌─────────────────────────────────────────────────────────────────┐
│ CLI: data openstock fetch-minute-klines                          │
│   --symbol sh600000 --period 1m|5m|15m|30m|60m                   │
│   --date 2026-07-02 --adjust none|qfq|hfq                        │
└──────────────────────────┬──────────────────────────────────────┘
                           │
           ┌───────────────▼────────────────┐
           │ openstock_handler:              │
           │   fetch_openstock_minute_klines │
           │   (FromStr 严格校验 → fail-fast)│
           └───────────────┬────────────────┘
                           │
           ┌───────────────▼────────────────┐
           │ OpenStockClient::               │
           │   fetch_minute_klines(          │
           │     code, period, date, adjust) │
           │   → Vec<MinuteBar>              │
           │   (直 reqwest，无 retry/breaker) │
           └───────────────┬────────────────┘
                           │
                           ▼
                  OpenStock /data/bars (POST + JSON body)
                  { symbol, period: "1m", date, adjust? }
```

### 3.2 P0.13b-2（分时点序列）

```
┌─────────────────────────────────────────────────────────────────┐
│ CLI: data openstock fetch-minute-share                            │
│   --symbol sh600000 --date 2026-07-02                             │
└──────────────────────────┬──────────────────────────────────────┘
                           │
           ┌───────────────▼────────────────┐
           │ openstock_handler:              │
           │   fetch_openstock_minute_share  │
           └───────────────┬────────────────┘
                           │
           ┌───────────────▼────────────────┐
           │ OpenStockClient::               │
           │   fetch_minute_share(           │
           │     code, date)                 │
           │   → Vec<MinuteShare>            │
           │   (走 envelope 路径，retry+breaker)│
           └───────────────┬────────────────┘
                           │
                           ▼
                  OpenStock /data/fetch
                  {category: MINUTE_DATA, ...}
```

---

## 4. 组件设计

### 4.1 P0.13b-1 新增组件

#### 4.1.1 `MinutePeriod` 枚举（`src/data/models.rs`）

```rust
/// `/data/bars` 分钟周期参数（P0.13b-1 新增）。
///
/// 与 P0.13a 的 `BarPeriod`（day/week/month）语义域不同：
/// 分钟蜡烛返回 `Vec<MinuteBar>`（含 NaiveDateTime 时间戳），
/// 而日线/周线/月线返回 `Vec<Kline>`（仅 NaiveDate）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinutePeriod {
    Minute1,
    Minute5,
    Minute15,
    Minute30,
    Minute60,
}

impl MinutePeriod {
    /// 返回 OpenStock `/data/bars` wire token。
    /// 仅使用 _PERIOD_MAP 主 token（拒绝所有别名）。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Minute1 => "1m",
            Self::Minute5 => "5m",
            Self::Minute15 => "15m",
            Self::Minute30 => "30m",
            Self::Minute60 => "60m",
        }
    }
}

impl std::str::FromStr for MinutePeriod {
    type Err = QuantixError;

    /// 严格白名单：仅接受 `1m|5m|15m|30m|60m`（任意大小写）。
    /// **拒绝**所有别名（`1min|minute|5min|1h|hour` 等）。
    ///
    /// Why: OpenStock `_PERIOD_MAP` 对未知 token 静默回退到 day，
    /// 若接受别名会导致 wire 层歧义（如 `1h` 实际映射到 `60m`，
    /// 调用方无法区分）。严格白名单 + fail-fast 防御 R1。
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "1m" => Ok(Self::Minute1),
            "5m" => Ok(Self::Minute5),
            "15m" => Ok(Self::Minute15),
            "30m" => Ok(Self::Minute30),
            "60m" => Ok(Self::Minute60),
            other => Err(QuantixError::Config(format!(
                "unsupported MinutePeriod `{}`: expected one of 1m|5m|15m|30m|60m",
                other
            ))),
        }
    }
}
```

#### 4.1.2 `MinuteBar` 结构体（`src/data/models.rs`）

```rust
/// 分钟级 K 线蜡烛（P0.13b-1 新增）。
///
/// **命名说明**：命名为 `MinuteBar`（不是 `MinuteKline`），因为
/// `src/db/tdengine.rs:37` 已存在公开 re-export 的 `MinuteKline`{
/// ts: DateTime<Utc>, code, open: f64, ... }——TDengine 行映射用 f64。
/// 本类型用 `Decimal` + `AdjustType`，语义不同，必须避免名称碰撞。
/// `MinuteBar` 与 P0.13a `BarPeriod` 形成请求/响应语义对。
///
/// 与 `Kline`（日线）的区别：
/// - `timestamp: NaiveDateTime`（精确到分钟）vs `date: NaiveDate`
/// - 其他字段与 `Kline` 一致
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteBar {
    pub code: String,
    pub timestamp: NaiveDateTime,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: i64,
    pub amount: Option<Decimal>,
    pub adjust_type: AdjustType,
}
```

#### 4.1.3 `fetch_minute_klines` 方法（`src/sources/openstock_client.rs`）

```rust
pub async fn fetch_minute_klines(
    &self,
    code: &str,
    period: MinutePeriod,
    date: NaiveDate,
    adjust: AdjustType,
) -> Result<Vec<MinuteBar>> {
    // 复用 /data/bars 端点（POST + JSON body），body:
    //   { symbol, period: "1m"|"5m"|..., date: "YYYY-MM-DD", adjust?: "qfq"|"hfq" }
    // 与 fetch_klines 同路径：直 reqwest，无 envelope，无 retry/breaker。
    //
    // 时间戳解析：OpenStock 返回 `time` 字段（ISO 字符串或 epoch 秒），
    // 转为 NaiveDateTime。若字段缺失或格式不符 → QuantixError::Other。
    todo!()
}
```

#### 4.1.4 CLI 子命令（`src/cli/commands/data.rs`）

```rust
FetchMinuteKlines {
    #[arg(long)] symbol: String,
    #[arg(long, default_value = "1m")] period: String,
    #[arg(long)] date: String,           // YYYY-MM-DD
    #[arg(long, default_value = "none")] adjust: String,
}
```

#### 4.1.5 Handler（`src/cli/handlers/openstock_handler.rs`）

```rust
pub(crate) async fn fetch_openstock_minute_klines(
    settings: &OpenStockSettings,
    symbol: String,
    period: String,
    date: String,
    adjust: String,
) -> Result<()> {
    let period_enum = MinutePeriod::from_str(&period)
        .map_err(|e| QuantixError::Config(format!("--period: {}", e)))?;
    let adjust_enum = AdjustType::from_str(&adjust)
        .map_err(|e| QuantixError::Config(format!("--adjust: {}", e)))?;
    let date_parsed = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| QuantixError::Config(format!("--date: {}", e)))?;

    let client = OpenStockClient::from_settings(settings)?;
    let klines = client.fetch_minute_klines(&symbol, period_enum, date_parsed, adjust_enum).await?;

    // 输出格式（与 P0.13a fetch-klines 同形状）：
    println!("OpenStock live fetch (/data/bars, symbol={}, minute={})", symbol, period_enum.as_str());
    println!("  Date:   {}", date);
    println!("  Adjust: {}", adjust_enum.as_openstock_param().unwrap_or("none (field omitted)"));
    println!("  记录数: {}", klines.len());
    if !klines.is_empty() {
        println!("  First:  {:?}", klines.first());
        println!("  Last:   {:?}", klines.last());
    }
    println!("  Source:        (not reported by /data/bars)");
    println!("  artifact_hash: (not reported by /data/bars)");
    println!("  latency_ms:    (not reported by /data/bars)");
    Ok(())
}
```

### 4.2 P0.13b-2 新增组件

#### 4.2.1 `MinuteShare` 结构体（`src/data/models.rs`）

```rust
/// 分时点序列（P0.13b-2 新增）。
///
/// 对应 OpenStock `MINUTE_DATA` 分类。与 `MinuteBar` 区别：
/// - 无 OHLC（仅单一 `price`）
/// - 含 `avg_price`（均价，业务关键字段）
///
/// **Option 字段说明**：业务字段全部用 `Option` 包裹以支持 INV-2C
/// （单条记录字段缺失时 warn + skip，不中断整批）。serde 反序列化
/// 在 Option 字段缺失时返回 None 而非失败；parser 阶段检查关键字段
/// （price/volume/amount/avg_price），任一为 None 则 warn + skip。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteShare {
    pub code: String,
    pub timestamp: NaiveDateTime,
    pub price: Option<Decimal>,
    pub volume: Option<i64>,
    pub amount: Option<Decimal>,
    pub avg_price: Option<Decimal>,
}
```

#### 4.2.2 `fetch_minute_share` 方法（`src/sources/openstock_client.rs`）

```rust
pub async fn fetch_minute_share(
    &self,
    code: &str,
    date: NaiveDate,
) -> Result<Vec<MinuteShare>> {
    // 走 /data/fetch envelope 路径（与 fetch_stock_codes 等同路径）：
    //   POST /data/fetch { category: "MINUTE_DATA", code, date }
    // 复用现有 envelope + retry + circuit breaker。
    //
    // 解析：response.records 是 8 字段的数组，parse_minute_share 裁剪到 5 业务字段。
    todo!()
}
```

#### 4.2.3 `parse_minute_share` 解析器（`src/sources/openstock_parsers.rs` 或同 client 文件）

```rust
/// 解析 MINUTE_DATA 单条记录（8 字段）为 `MinuteShare`（5 业务字段）。
///
/// 丢弃字段：`index`（内部序号）、`time`（ISO 冗余）、`price_milli`（毫表示）。
/// 保留字段：`time_minutes, price, volume, amount, avg_price`。
///
/// 返回 `Option<MinuteShare>` 而非 `Result`：当关键字段（price/volume/
/// amount/avg_price）任一为 None 时返回 None，调用方 warn + skip，
/// 实现 INV-2C "不中断整批" 语义。
pub(crate) fn parse_minute_share(
    code: &str,
    raw: &RawMinuteRecord,
) -> Option<MinuteShare> {
    todo!()
}
```

#### 4.2.4 CLI 子命令 + Handler

```rust
FetchMinuteShare {
    #[arg(long)] symbol: String,
    #[arg(long)] date: String,
}

pub(crate) async fn fetch_openstock_minute_share(
    settings: &OpenStockClientSettings,
    symbol: String,
    date: String,
) -> Result<()> {
    // 输出形状与 fetch-minute-klines 类似（无 Period/Adjust 行）
    todo!()
}
```

---

## 5. 不变量（Invariants）

### 5.1 P0.13b-1 不变量

| ID | 描述 |
|---|---|
| INV-1A | `MinutePeriod` 仅接受 5 个主 token，所有别名在 FromStr 阶段拒绝 |
| INV-1B | `fetch_minute_klines` 返回的 `MinuteBar` 必须含 `NaiveDateTime`（非 `NaiveDate`） |
| INV-1C | `/data/bars` 4xx/5xx 错误必须以 `QuantixError::Other` 传播，**不重试**（与 fetch_klines 对齐） |
| INV-1D | `AdjustType::None` 时请求体**省略** `adjust` 字段（不发送 `"adjust": ""`） |

### 5.2 P0.13b-2 不变量

| ID | 描述 |
|---|---|
| INV-2A | `MinuteShare` 仅含 5 个业务字段，丢弃 `index/time/price_milli` |
| INV-2B | `/data/fetch MINUTE_DATA` 复用现有 envelope 路径（含 retry + circuit breaker） |
| INV-2C | 单条记录字段缺失（`MinuteShare` 业务字段为 None）时 warn + skip，**不中断整批**——`MinuteShare` 业务字段全部 `Option` 包裹使 serde 不在字段缺失时硬失败，parser 显式判定后跳过 |
| INV-2D | `timestamp` 来自 `time_minutes`（分钟精度），不用 `time` 字段（避免双源） |

---

## 6. 错误处理矩阵

### 6.1 P0.13b-1

| 错误源 | 触发条件 | 传播路径 | 用户可见信息 |
|---|---|---|---|
| CLI 参数解析 | `--period` 不是 5 token 之一 | `FromStr → QuantixError::Config` | "unsupported MinutePeriod ..." |
| CLI 参数解析 | `--date` 不是 YYYY-MM-DD | `parse_from_str → QuantixError::Config` | "--date: ..." |
| HTTP 4xx/5xx | `/data/bars` 返回非 2xx | `resp.status()` 检查 → `QuantixError::Other` | "/data/bars returned {status}" |
| 字段缺失 | 响应缺 `time`/`open` 等必需字段 | `serde` 错误 → `QuantixError::Other` | "missing field ..." |
| 时间戳解析失败 | `time` 字段格式无法解析 | `parse → QuantixError::Other` | "invalid timestamp ..." |

### 6.2 P0.13b-2

| 错误源 | 触发条件 | 传播路径 | 用户可见信息 |
|---|---|---|---|
| HTTP 4xx/5xx | `/data/fetch` envelope error | 现有 `fetch<T>` 路径（含 retry） | "OpenStock /data/fetch error ..." |
| envelope 失败 | `success: false` + error msg | 现有 envelope 错误路径 | envelope.error.message |
| 单字段缺失 | 单条记录字段缺失 | `parse_minute_share` warn + skip | (warn log only) |
| 全部记录缺失 | 所有记录都缺字段 | 返回空 Vec | "0 records parsed" |

---

## 7. 测试覆盖（每子切片 8 测试，3 层）

### 7.1 P0.13b-1 测试矩阵

| ID | 层 | 文件 | 内容 |
|---|---|---|---|
| T1 | unit | `src/data/models.rs` | `MinutePeriod` as_str 往返 + FromStr 严格（拒绝别名 `1min/minute/1h`） |
| T2 | unit | `src/data/models.rs` | `MinuteBar` 序列化往返（timestamp 字段） |
| T3 | wiremock | `src/sources/openstock_client.rs` | `fetch_minute_klines` period=1m + adjust=none + 200 OK → Vec 非空 |
| T4 | wiremock | 同 | period=5m + adjust=qfq → body 含 `"adjust":"qfq"`，返回带 `adjust_type: QFQ` |
| T5 | wiremock | 同 | period=15m + 4xx → `QuantixError::Other`，`.expect(1)` 锁定无重试 |
| T6 | live | `tests/openstock_live_minute_klines.rs` | `#[ignore]` + `QUANTIX_OPENSTOCK_LIVE=1`，sh600000 + 1m + date=最近交易日 |
| T7 | live | 同 | sh600000 + 5m + qfq |
| T8 | live | 同 | sh600000 + 60m + hfq |

### 7.2 P0.13b-2 测试矩阵

| ID | 层 | 文件 | 内容 |
|---|---|---|---|
| T1 | unit | parser 文件 | `parse_minute_share` 8 字段 → 5 字段（验证裁剪） |
| T2 | unit | 同 | 单字段缺失（如 `avg_price` null）→ warn + skip，其他记录正常返回 |
| T3 | wiremock | `src/sources/openstock_client.rs` | `fetch_minute_share` 200 OK → Vec 非空，timestamp 来自 `time_minutes` |
| T4 | wiremock | 同 | envelope 5xx → 触发 retry（mock 期望 2 次调用），最终失败 `QuantixError::Other` |
| T5 | wiremock | 同 | envelope `success: false` → 错误信息正确传播 |
| T6 | live | `tests/openstock_live_minute_share.rs` | `#[ignore]` + 环境变量，sh600000 + 最近交易日 |
| T7 | live | 同 | sh000001（指数）+ 最近交易日 |
| T8 | live | 同 | 跨市场 sh300688（创业板）+ 最近交易日 |

---

## 8. 实施计划（3 Phase × 2 子切片）

### 8.1 P0.13b-1 实施顺序

| Phase | 任务 | Commit |
|---|---|---|
| 1.1 | `MinutePeriod` 枚举 + `MinuteBar` 结构体 + unit tests (T1, T2) | commit 1 |
| 1.2 | `fetch_minute_klines` 方法 + wiremock tests (T3-T5) | commit 2 |
| 2 | CLI `FetchMinuteKlines` + handler + dispatcher + mod.rs | commit 3 |
| 3 | Live tests (T6-T8) + OpenSpec change + governance card + archive | commit 4 |

### 8.2 P0.13b-1 完成后的中间步骤

1. Push to origin/master
2. `gitnexus analyze` 刷新索引
3. Archive P0.13b-1 OpenSpec change
4. 更新 HANDOFF 报告：**纠正行 35 的 mislabel**（`MINUTE_DATA` → 实际描述应为分时图，分钟蜡烛由 `KLINES` 提供）
5. 启动 P0.13b-2

### 8.3 P0.13b-2 实施顺序

| Phase | 任务 | Commit |
|---|---|---|
| 1 | `MinuteShare` 结构体 + `parse_minute_share` + unit tests (T1, T2) | commit 1 |
| 2 | `fetch_minute_share` 方法（envelope 路径）+ wiremock tests (T3-T5) | commit 2 |
| 3 | CLI `FetchMinuteShare` + handler + dispatcher + mod.rs | commit 3 |
| 4 | Live tests (T6-T8) + OpenSpec change + governance card + archive | commit 4 |

---

## 9. 决策记录（D1-D8）

### D1：拆两个独立子切片
**决策**：P0.13b 拆为 P0.13b-1（分钟蜡烛）+ P0.13b-2（分时点序列），各自独立 OpenSpec change。
**原因**：数据形态、端点、模型、客户端方法完全不同，合并会引入跨域复杂度。

### D2：复用 `/data/bars` 端点（不新建端点）
**决策**：P0.13b-1 复用 P0.13a 的 `/data/bars`，仅 `period` 参数从 day/week/month 扩展到 1m/5m/15m/30m/60m。
**原因**：OpenStock 已支持，无需服务端改动。

### D3：新建 `MinutePeriod` 枚举（不扩展 `BarPeriod`），新建 `MinuteBar` 结构体（不复用 `Kline`，不与 `src/db/tdengine.rs:37` `MinuteKline` 碰撞）
**决策**：
- P0.13b-1 新建 `MinutePeriod`，不动 P0.13a 的 `BarPeriod`。
- 蜡烛响应类型命名为 `MinuteBar`（**不是** `MinuteKline`）——`src/db/tdengine.rs:37` 已有公开 re-export 的 `MinuteKline`（f64 + DateTime\<Utc\>，TDengine 行映射），命名碰撞会导致编译歧义。
**原因**：
- 返回类型不同（`Vec<MinuteBar>` vs `Vec<Kline>`），类型系统强制区分；保持 P0.13a 稳定。
- `MinuteBar` 与 `BarPeriod` 形成请求/响应语义对；避免 TDengine `MinuteKline` 的 f64 vs API 层 Decimal 类型歧义。

### D4：wire token 使用主 token（拒绝别名）
**决策**：`MinutePeriod::as_str()` 返回 `1m|5m|15m|30m|60m`；`FromStr` 拒绝所有别名。
**原因**：OpenStock `_PERIOD_MAP` 对未知 token 静默回退到 day（R1），严格白名单 + fail-fast 是唯一安全策略。

### D5：`MinuteBar` 独立结构体（不复用 `Kline`）
**决策**：新建 `MinuteBar { timestamp: NaiveDateTime, ... }`，不复用 `Kline { date: NaiveDate, ... }`。
**原因**：时间精度是业务必需，把 `Kline.date` 改成 `NaiveDateTime` 会破坏所有现有消费者。命名见 D3。

### D6：`MinuteShare` 裁剪到 5 业务字段
**决策**：丢弃 `index/time/price_milli`，保留 `time_minutes/price/volume/amount/avg_price`。
**原因**：被丢弃字段是冗余/内部字段；保留它们会让数据模型臃肿，且 `price_milli` 等是测试辅助字段不应进生产 schema。

### D7：P0.13b-2 复用现有 envelope 路径
**决策**：`fetch_minute_share` 走 `/data/fetch` envelope（含 retry + circuit breaker），不直 reqwest。
**原因**：与 P0.13a 已有的 `fetch_stock_codes/fetch_trade_dates/fetch_index_klines` 保持一致，复用成熟基础设施。

### D8：先做 P0.13b-1 再做 P0.13b-2
**决策**：P0.13b-1 完成归档后再启动 P0.13b-2。
**原因**：分钟蜡烛逻辑更接近 P0.13a（同端点），风险更低；先完成它能减少 P0.13b-2 的不确定性。

---

## 10. 非目标（Non-Goals）

- 不修改 P0.13a 已合并的 `BarPeriod` / `fetch_klines` / `Kline` 结构体
- 不新增 ClickHouse 写入路径（只读消费）
- 不实现分钟级数据的回测/策略（仅数据消费层）
- 不重构 OpenStock envelope 路径（复用现有）
- 不实现 websocket 实时推送（仍为 HTTP 拉取）
- 不纠正 HANDOFF 报告行 35 的 mislabel（留待 P0.13b-1 完成时作为归档步骤一并修正）

---

## 11. 风险与缓解

| ID | 风险 | 缓解 |
|---|---|---|
| R1 | OpenStock `_PERIOD_MAP` 静默回退 | D4：严格 FromStr + fail-fast |
| R2 | 分钟蜡烛响应 `time` 字段格式未知（ISO 字符串 vs epoch） | P0.13b-1 Phase 1.2 先用 wiremock 模拟两种格式，确定实际格式后选定解析器；live 测试验证 |
| R3 | `MINUTE_DATA` 响应字段在不同 provider 间差异 | P0.13b-2 解析器对缺失字段 warn + skip，不硬失败 |
| R4 | 分钟级数据量大（一日 240 条 1m 蜡烛 vs 1 条日线） | 本切片只读不写，性能影响仅在 HTTP 响应大小；不进入性能优化范围 |
| R5 | OpenStock `/data/bars` 对 `date` 参数的语义（单日 vs 范围）未知 | P0.13b-1 Phase 1.2 wiremock 验证单日；若服务端支持范围查询，留待 P0.13c |

---

## 12. 验证清单

### 12.1 P0.13b-1 验证

```bash
# Quality gates
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --lib --package quantix-cli data::models::tests
cargo test --lib --package quantix-cli sources::openstock_client::tests::fetch_minute
cargo test --workspace                              # regression

# Manual live smoke
QUANTIX_OPENSTOCK_LIVE=1 \
OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
OPENSTOCK_API_KEY=<key> \
cargo test --test openstock_live_minute_klines -- --ignored

# CLI smoke (注意：FetchMinuteKlines 挂在 DataCommands::OpenStock → OpenStockCommands 下)
cargo run -q -- data openstock fetch-minute-klines --symbol sh600000 --period 1m --date 2026-07-02

# Spec + governance
openspec validate openstock-data-consumption-p0-13b-1 --strict
openspec validate --all --strict
gitnexus detect_changes
```

### 12.2 P0.13b-2 验证

（同形状，替换 `p0-13b-1` → `p0-13b-2`，`fetch-minute-klines` → `fetch-minute-share`）

---

## 13. 引用

- P0.13a 设计：`docs/superpowers/specs/2026-07-02-openstock-p0-13a-multi-period-klines-design.md`
- P0.13a 评审：`docs/superpowers/specs/2026-07-02-openstock-p0-13a-design-review.md`
- HANDOFF 报告：`docs/reports/HANDOFF_TDX_API_TO_OPENSTOCK_DATA_CAPABILITY_GAPS_2026-06-30.md`（行 35, 38）
- OpenStock `_PERIOD_MAP`：`/opt/claude/openstock/openstock/adapters/_eltdx_timeseries.py:12-32`
- OpenStock `MINUTE_DATA` schema：`/opt/claude/openstock/openstock/adapters/_field_schemas/_eltdx.py:64-72`
