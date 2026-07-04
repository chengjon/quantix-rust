# OpenStock P0.14 — ClickHouse 分钟级数据持久化设计

> 日期：2026-07-04（spec）/ 2026-07-05（revision: 对齐现有 KlineDataCH 规范）
> 范围：把 P0.13d 流式 API (klines + shares) 落盘到 ClickHouse `quantix` 库
> 基线：HEAD（P0.13d 已合并到 master）
> 上游：`src/sources/openstock_client.rs::fetch_minute_klines_stream` / `fetch_minute_share_stream`
> 下游：P0.15 (CLI 子命令 + scheduler)，P0.16+ (回测/聚合消费)

---

## 0. 背景与边界

P0.13d 已交付 `impl Stream<Item = Result<Vec<T>>> + 'a` 形态的流式拉取接口，但只返回给调用方内存中的 `Vec`。本切片负责把这两条流落到 ClickHouse `quantix` 库的两张新表 `minute_klines` / `minute_shares`，**不引入 CLI 子命令、不接入 scheduler、不改 P0.13 冻结面**。

**严格边界（与用户确认一致）**

- ✅ 同时落盘 klines + shares 两类分钟数据
- ❌ CLI 子命令（`Persist minute-*`）→ P0.15
- ❌ scheduler / 周期任务 / cron 触发 → P0.15
- ❌ 任何对 `src/sources/**`、`src/cli/**`、`src/scheduler/**` 的修改
- ❌ ClickHouse 之外的 sink（Parquet / DuckDB / 内存聚合）
- ❌ 任何对 `kline_data` / `minute_klines_*` 旧表的迁移

**与现有规范对齐（与 `KlineDataCH` 完全一致）**

- 类型映射：`timestamp: DateTime<Utc>`、`period: String`、`adjust: String`、OHLCV/amount 用 `Float64` —— 100% 复刻 `src/db/clickhouse/models.rs:33-47` 的 `KlineDataCH` 模式
- DDL：`ENGINE = MergeTree()`、`ON CLUSTER '{cluster}'`、`PARTITION BY (period, toYYYYMM(timestamp))`、`ORDER BY (toDate(timestamp), code, period, timestamp)` —— 完全沿用 `kline_data` 表（`schema.rs:97-118`）的写法
- 插入选项：`async_insert=1` + `wait_for_async_insert=1` —— 与 `kline.rs:204-205` 完全一致
- Decimal→f64 转换：`use rust_decimal::prelude::*;` 然后 `dec.to_f64().unwrap_or(0.0)` —— 与 `kline.rs:213-219` 完全一致
- `'{cluster}'` 占位符在运行时被替换为 `single_cluster`，由 `schema.rs:48` 等位置的 `.replace("'{cluster}'", "single_cluster")` 处理

---

## 1. 表结构

### 1.1 `quantix.minute_klines`

```sql
CREATE TABLE IF NOT EXISTS minute_klines ON CLUSTER '{cluster}'
(
    `timestamp`     DateTime,
    `code`          String,
    `period`        String,
    `adjust`        String,
    `open`          Float64,
    `high`          Float64,
    `low`           Float64,
    `close`         Float64,
    `volume`        Float64,
    `amount`        Float64,
    `date`          MATERIALIZED toDate(timestamp)
)
ENGINE = MergeTree()
PARTITION BY (period, toYYYYMM(timestamp))
ORDER BY (date, code, period, adjust, timestamp)
SETTINGS index_granularity = 8192
```

**字段映射**（`MinuteBar` → `MinuteKlineCH`，与 `KlineDataCH` 模式一致）

| MinuteBar 字段 | CH 列 | 备注 |
|---|---|---|
| `timestamp: NaiveDateTime` | `timestamp: DateTime<Utc>` | `DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)`（见 §2.1） |
| `code: String` | `code: String` | 直接 clone |
| —（不在 MinuteBar 上） | `period: String` | `period` 是 `fetch_minute_klines_stream` 的输入参数（`MinuteBar` 本身没有 `period` 字段，见 `data/models.rs:138-148`）；`bar_to_row` 接受独立 `period: MinutePeriod` 参数，通过 `period_as_str` 转 `"1m"`/`"5m"`/`"15m"`/`"30m"`/`"60m"` 字面量 |
| `adjust_type: AdjustType` | `adjust: String` | `AdjustType` 经 `adjust_as_str` 转换为 `"none"`/`"qfq"`/`"hfq"` 字面量 |
| `open/high/low/close: Decimal` | `open/high/low/close: Float64` | `dec.to_f64().unwrap_or(0.0)` |
| `volume: i64` | `volume: Float64` | `bar.volume as f64`；i64→f64 cast 在 ≤ 10^9 范围无损（A 股单 bar 远小于此） |
| `amount: Option<Decimal>` | `amount: Float64` | parser 已保证非 None（见下文 Option 处理说明），`bar.amount.unwrap_or_default().to_f64().unwrap_or(0.0)` |
| —（CH MATERIALIZED） | `date` | `MATERIALIZED toDate(timestamp)`，写入时不显式赋值，与 `kline_data.date` 一致 |

**ORDER BY 依据**

- `date` 排第一：日级范围查询直接命中分区+前缀索引（与 `kline_data` 一致）
- `code` 第二：跨日单 code 查询（最常见场景）顺序读
- `period` + `adjust` 第三：多周期/复权组合时仍能紧凑扫描；`adjust` 加入是为了让相同 period 不同 adjust 的行能在 ORDER BY 维度自然分离
- `timestamp` 末位：同一日内按时间排序

**Option 字段处理说明（INV-2D）**

`MinuteBar.amount` 实际为 `Option<Decimal>`（`data/models.rs:146`），但 `parse_minute_klines` 已在 parser 阶段对关键字段做 `?` 解包——字段缺失时该条记录直接被 warn + skip，不会到达 `MinuteBar`。因此 `bar_to_row` 中 `bar.amount.unwrap_or_default()` 是安全的，运行时绝不可能产生意外 `Decimal::default()`。`share_to_row` 对 `MinuteShare` 的 4 个 Option 字段同理。

### 1.2 `quantix.minute_shares`

```sql
CREATE TABLE IF NOT EXISTS minute_shares ON CLUSTER '{cluster}'
(
    `timestamp`     DateTime,
    `code`          String,
    `price`         Float64,
    `volume`        Float64,
    `amount`        Float64,
    `avg_price`     Float64,
    `date`          MATERIALIZED toDate(timestamp)
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (date, code, timestamp)
SETTINGS index_granularity = 8192
```

**字段映射**（`MinuteShare` → `MinuteShareCH`）

| MinuteShare 字段 | CH 列 | 备注 |
|---|---|---|
| `timestamp: NaiveDateTime` | `timestamp: DateTime<Utc>` | 同 §1.1 |
| `code: String` | `code: String` | 直接 clone |
| `price: Option<Decimal>` | `price: Float64` | parser 已保证非 None，`share.price.unwrap_or_default().to_f64().unwrap_or(0.0)` |
| `volume: Option<i64>` | `volume: Float64` | 同上，`share.volume.unwrap_or_default() as f64`；i64→f64 cast 在 ≤ 10^9 范围无损 |
| `amount: Option<Decimal>` | `amount: Float64` | 同上 |
| `avg_price: Option<Decimal>` | `avg_price: Float64` | 同上 |
| —（CH MATERIALIZED） | `date` | 同 §1.1 |

Option 处理路径与 §1.1 同理：parser 在 `parse_minute_share` 中已对 price/volume/amount/avg_price 做了 `?` 解包，到达 `MinuteShare` 的字段必为 `Some`，运行时 `unwrap_or_default()` 不会产生意外零值。

**为何 share 没有 `period` / `adjust`**

P0.13b-2 已确认 `MinuteShare` 是分笔成交，没有 period/adjust 概念。表结构反映了领域模型的这一差异。

### 1.3 Decimal→f64 转换约束（A 股数值范围）

A 股数值范围下 `Decimal → f64` 精度安全：

| 字段 | 实际范围 | f64 可表示精度 |
|---|---|---|
| 价格 (Decimal) | [0.01, 9999.99]（含涨跌停） | f64 尾数 52 bit，可精确表示所有 ≤ 2^53 的整数；价格区间远小于此 |
| 成交量 (i64) | 单 bar ≤ 10^9（1 分钟内不可能更多） | `i64 as f64` 在 ≤ 2^53 ≈ 9×10^15 内无损；A 股成交量远小于此 |
| 成交额 (Decimal) | 单 bar ≤ 10^12（极端情况） | 同上；即便 10^15 仍可精确 |

> **A 股数值范围约束注释**（必须出现在 helper 注释中）：
>
> ```rust
> /// Convert Decimal to f64 for ClickHouse Float64 columns.
> ///
> /// A 股数值范围内（|v| < 10^15）Decimal → f64 转换无损：
> /// - 价格：[0.01, 9999.99]，远低于 2^53
> /// - 成交额：单 bar ≤ 10^12
> ///
> /// 与 `src/db/clickhouse/kline.rs:213-219` 完全一致：通过
> /// `rust_decimal::prelude::*` 的 `ToPrimitive::to_f64` 实现；
> /// `.unwrap_or(0.0)` 是防御性回退（理论不可能失败；A 股范围内无损）。
> /// 不写 warn 日志：与 kline.rs 静默回退模式对齐，避免正常运行时刷日志。
> ///
> /// 注意：volume 是 `i64` 而非 `Decimal`，由调用方直接 `as f64`（见 §1.1）。
> /// `decimal_to_f64` 不处理 volume。
> fn decimal_to_f64(v: Decimal) -> f64 {
>     v.to_f64().unwrap_or(0.0)
> }
> ```

**回退策略（与 kline.rs 一致，不写 warn）**

`Decimal::to_f64()` 失败时（理论不可能；防御性）：
1. 返回 `0.0`
2. 调用方继续写入（不阻塞流）

**为何不写 warn**：与 `kline.rs:213-219`、`shadow_kline.rs:50-55` 等现有写入路径完全对齐——所有现有路径都用 `.unwrap_or(0.0)` 静默回退。引入 warn 会破坏一致性，且在 A 股范围内转换绝不可能失败（属于死代码分支）。运行时通过 `StreamStats::inserted_records` vs `input_records` 的差值间接反映异常。

**i64 volume 不走 `decimal_to_f64`**

`MinuteBar.volume: i64` 和 `MinuteShare.volume: Option<i64>`（unwrap 后）直接 `as f64`。A 股单 bar volume ≤ 10^9，远低于 2^53，cast 无损。

---

## 2. 写入层代码设计

### 2.1 新建 `src/db/clickhouse/minute.rs`（独立文件，单一职责）

**为什么独立文件**

- `kline.rs` 已有 358 行，与日线查询逻辑耦合
- minute 路径是纯流式写入，逻辑独立
- 拆分避免一个文件 > 500 行的硬阈值（见 CLAUDE.md 文件大小规范）
- 后续 P0.15 添加 CLI 子命令时，dispatcher 直接 `use crate::db::clickhouse::minute::stream_minute_klines_to_clickhouse` 即可

**模块结构**

```rust
//! src/db/clickhouse/minute.rs
//! ClickHouse write path for OpenStock minute-level data (P0.14).
//!
//! Consumes `fetch_minute_klines_stream` / `fetch_minute_share_stream`
//! (P0.13d) and writes batches to `quantix.minute_klines` / `minute_shares`.

use crate::data::models::{AdjustType, MinuteBar, MinutePeriod, MinuteShare};
use crate::db::clickhouse::models::{MinuteKlineCH, MinuteShareCH};
use crate::sources::openstock_client::OpenStockClient;
use chrono::{DateTime, NaiveDate, Utc};
use clickhouse::Client;
use futures::StreamExt;
use rust_decimal::prelude::*;

// ─── NaiveDateTime → DateTime<Utc> ──────────────────────────────────────────

/// Lift a NaiveDateTime to a UTC-tagged DateTime for ClickHouse `DateTime` columns.
///
/// 与 `src/db/clickhouse/kline.rs:210` 的模式一致：
/// `DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)`。
///
/// OpenStock 返回的 `MinuteBar.timestamp` / `MinuteShare.timestamp` 已经是
/// 北京时间的 naive 表示（无时区）；按现有约定写入为 `DateTime<Utc>`，
/// 由 ClickHouse 列声明 `DateTime`（不带时区）原样存储。读回时调用方
/// 需自行按 A 股东八区语义解读，与 `kline_data` 表一致。
fn naive_to_utc(naive: chrono::NaiveDateTime) -> DateTime<Utc> {
    DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)
}

// ─── Decimal → f64 ──────────────────────────────────────────────────────────

/// 见 §1.3 完整注释。
fn decimal_to_f64(v: rust_decimal::Decimal) -> f64 {
    v.to_f64().unwrap_or(0.0)
}

// ─── 行转换 ────────────────────────────────────────────────────────────────

fn bar_to_row(bar: &MinuteBar, period: MinutePeriod) -> MinuteKlineCH {
    MinuteKlineCH {
        timestamp: naive_to_utc(bar.timestamp),
        code: bar.code.clone(),
        // MinuteBar 本身没有 period 字段（data/models.rs:138-148）；
        // period 来自 stream 函数的输入参数，通过独立参数传入。
        period: period_as_str(&period).to_string(),
        adjust: adjust_as_str(&bar.adjust_type).to_string(),
        open: decimal_to_f64(bar.open),
        high: decimal_to_f64(bar.high),
        low: decimal_to_f64(bar.low),
        close: decimal_to_f64(bar.close),
        volume: bar.volume as f64,
        amount: bar.amount.unwrap_or_default().to_f64().unwrap_or(0.0),
    }
}

fn share_to_row(share: &MinuteShare) -> MinuteShareCH {
    MinuteShareCH {
        timestamp: naive_to_utc(share.timestamp),
        code: share.code.clone(),
        price: share.price.unwrap_or_default().to_f64().unwrap_or(0.0),
        volume: share.volume.unwrap_or_default() as f64,
        amount: share.amount.unwrap_or_default().to_f64().unwrap_or(0.0),
        avg_price: share.avg_price.unwrap_or_default().to_f64().unwrap_or(0.0),
    }
}

// period/adjust → 字面量字符串（与 OpenStock API 字面量一致）
fn period_as_str(p: &MinutePeriod) -> &'static str {
    match p {
        MinutePeriod::Minute1 => "1m",
        MinutePeriod::Minute5 => "5m",
        MinutePeriod::Minute15 => "15m",
        MinutePeriod::Minute30 => "30m",
        MinutePeriod::Minute60 => "60m",
    }
}

fn adjust_as_str(a: &AdjustType) -> &'static str {
    match a {
        AdjustType::None => "none",
        AdjustType::QFQ => "qfq",
        AdjustType::HFQ => "hfq",
    }
}

// ─── Sink trait (pub(crate), 仅用于单元测试 mock) ───────────────────────────

/// Internal sink abstraction. Used **only** by unit tests in
/// `src/db/clickhouse/tests.rs` to inject a mock without touching the
/// real ClickHouse. Not part of any public API; no upstream call sites
/// depend on this trait.
pub(crate) trait MinuteSink<T: Send + Sync>: Send + Sync {
    async fn insert_batch(&self, batch: &[T]) -> Result<usize, clickhouse::error::Error>;
}

pub(crate) struct ClickHouseMinuteKlineSink<'a> {
    client: &'a Client,
}
pub(crate) struct ClickHouseMinuteShareSink<'a> {
    client: &'a Client,
}

impl<'a> MinuteSink<MinuteKlineCH> for ClickHouseMinuteKlineSink<'a> {
    async fn insert_batch(&self, batch: &[MinuteKlineCH]) -> Result<usize, clickhouse::error::Error> {
        if batch.is_empty() {
            return Ok(0);
        }
        let mut insert = self
            .client
            .insert("minute_klines")?
            .with_option("async_insert", "1")?
            .with_option("wait_for_async_insert", "1");
        for row in batch {
            insert.write(row).await?;
        }
        insert.end().await?;
        Ok(batch.len())
    }
}

// share sink 同理 ...

// ─── 流消费 ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StreamStats {
    pub batches: u64,
    pub input_records: u64,
    pub inserted_records: u64,
}

/// Consume the klines stream and insert each batch into ClickHouse.
///
/// Pinning is required because `fetch_minute_klines_stream` returns
/// `impl Stream + 'a` (not Unpin). We use `futures::pin_mut!` rather
/// than requiring the caller to pin — the function is the natural owner
/// of the pinning scope.
pub async fn stream_minute_klines_to_clickhouse<S: MinuteSink<MinuteKlineCH>>(
    client: &OpenStockClient,
    sink: &S,
    code: &str,
    period: MinutePeriod,
    start: NaiveDate,
    end: NaiveDate,
    adjust: AdjustType,
) -> Result<StreamStats, crate::core::QuantixError> {
    use crate::data::models::DateOrRange;

    let dor = DateOrRange::Range { start, end };
    let stream = client.fetch_minute_klines_stream(code, period, dor, adjust);
    futures::pin_mut!(stream);

    let mut stats = StreamStats::default();
    while let Some(batch_result) = stream.next().await {
        let batch = batch_result?;
        stats.batches += 1;
        stats.input_records += batch.len() as u64;

        let rows: Vec<MinuteKlineCH> = batch.iter().map(|b| bar_to_row(b, period)).collect();
        let inserted = sink
            .insert_batch(&rows)
            .await
            .map_err(|e| crate::core::QuantixError::DatabaseQuery(format!("ch insert minute_klines: {}", e)))?;
        stats.inserted_records += inserted as u64;
    }

    Ok(stats)
}

// `stream_minute_shares_to_clickhouse<S: MinuteSink<MinuteShareCH>>` 结构相同
```

**注意**：流消费函数接收 `sink: &S` 而非 `clickhouse: &Client`——这样单元测试可以注入 `MockMinuteKlineSink`；生产调用方（P0.15 dispatcher）传入 `ClickHouseMinuteKlineSink { client: &ch_client.client }` 即可。公共 API 仍然只暴露 `stream_minute_*_to_clickhouse`，sink 类型本身 `pub(crate)`。

### 2.2 DDL：在 init_database 中注册新表创建调用

`src/db/clickhouse/schema.rs` 的 `init_database()` 函数当前依次调用 6 个 `create_*_table()` 方法（`schema.rs:22-27`）。本切片新增的两个方法 **必须** 在 `init_database()` 中按相同模式追加调用：

```rust
// src/db/clickhouse/schema.rs
pub async fn init_database(&self) -> Result<()> {
    // ... 现有代码 ...
    self.create_stock_info_table().await?;
    self.create_stock_quotes_table().await?;
    self.create_kline_data_table().await?;
    self.create_limit_up_events_table().await?;
    self.create_gbbq_events_table().await?;
    self.create_market_tables().await?;
    // ↓ P0.14 新增 ↓
    self.create_minute_klines_table().await?;
    self.create_minute_shares_table().await?;

    info!("所有 ClickHouse 表创建成功");
    Ok(())
}

async fn create_minute_klines_table(&self) -> Result<()> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS minute_klines ON CLUSTER '{cluster}' (
            timestamp DateTime,
            code String,
            period String,
            adjust String,
            open Float64,
            high Float64,
            low Float64,
            close Float64,
            volume Float64,
            amount Float64,
            date MATERIALIZED toDate(timestamp)
        )
        ENGINE = MergeTree()
        PARTITION BY (period, toYYYYMM(timestamp))
        ORDER BY (date, code, period, adjust, timestamp)
        SETTINGS index_granularity = 8192
    "#;
    self.client
        .query(sql.replace("'{cluster}'", "single_cluster").as_str())
        .execute()
        .await
        .map_err(|e| QuantixError::DatabaseConnection(format!("创建 minute_klines 表失败: {}", e)))?;
    info!("minute_klines 表创建成功");
    Ok(())
}

// create_minute_shares_table 同理（详见 §1.2 DDL）
```

### 2.3 pub(crate) 边界

| 项 | 可见性 | 说明 |
|---|---|---|
| `decimal_to_f64` / `naive_to_utc` / `period_as_str` / `adjust_as_str` | 私有 | 仅模块内 |
| `bar_to_row` / `share_to_row` | 私有 | 仅模块内 |
| `MinuteSink<T>` trait | `pub(crate)` | **仅测试**注入 mock；非公共 API |
| `ClickHouseMinuteKlineSink` / `ClickHouseMinuteShareSink` | `pub(crate)` | 同上 |
| `MinuteKlineCH` / `MinuteShareCH` structs | `pub`（在 `models.rs`） | CH 行类型，给 L1/L2 实时测试反查用 |
| `stream_minute_*_to_clickhouse<S: MinuteSink<...>>` | `pub` | 给 P0.15 CLI dispatcher 用 |
| `StreamStats` | `pub` | 流式结果报告 |

---

## 3. 不变量 (Invariants)

### INV-1：表存在性
**INV-1A**：`init_database()` 完成后，`quantix.minute_klines` 和 `quantix.minute_shares` 必须存在（§2.2 在 `init_database()` 中追加调用是必需条件）。
**INV-1B**：两张表的 `ENGINE` 必须为 `MergeTree()`（与 `kline_data` 一致），不使用 `ReplacingMergeTree`。

### INV-2：类型映射（与 KlineDataCH 一致）
**INV-2A**：`MinuteBar::timestamp` (`NaiveDateTime`) 必须经 `naive_to_utc()` 抬升为 `DateTime<Utc>` 后写入 `DateTime` 列；写入和读回的 wall-clock 时刻必须相等。`MinuteShare::timestamp` 同理。
**INV-2B**：`MinuteBar::volume: i64` 与 `MinuteShare::volume: Option<i64>`（unwrap 后）通过 `as f64` 写入 `Float64` 列；读回值必须与原值在数值上相等（i64 ≤ 10^9 时 f64 精确）。
**INV-2C**：`MinuteBar::period` / `adjust_type` 通过 `period_as_str` / `adjust_as_str` 转换为字符串字面量 `"1m"`/`"5m"`/`"15m"`/`"30m"`/`"60m"` 和 `"none"`/`"qfq"`/`"hfq"`，写入 CH `String` 列；字面量集合与 OpenStock API 字面量一致。
**INV-2D（Option unwrap 安全性）**：`bar_to_row` / `share_to_row` 对 `MinuteBar.amount: Option<Decimal>` 和 `MinuteShare::{price, volume, amount, avg_price}` 等 Option 字段使用 `.unwrap_or_default()` 是安全的——parser 阶段已通过 `?` 操作过滤 None；运行时绝不可能产生意外 `Decimal::default()` 或 `0`。任何新增的 Option 字段必须在 parser 阶段同步加入「字段缺失 → warn + skip」逻辑，才能在 row helper 中 unwrap。

### INV-3：流语义继承
**INV-3A**：`stream_minute_klines_to_clickhouse` 必须在第一个流错误时短路（`?`），不再继续消费后续 batch。继承自 P0.13d D4。
**INV-3B**：`stream_minute_shares_to_clickhouse` 必须在第一个 batch 失败时短路，不再继续写入。
**INV-3C**：流消费函数不得在内部 catch 错误并继续；任何错误必须传播给调用方。

### INV-4：Sink trait 不外泄
**INV-4A**：`MinuteSink<T>` trait 的可见性必须为 `pub(crate)`，**不得**出现在 `lib.rs` 的 `pub use` 中。
**INV-4B**：`ClickHouseMinuteKlineSink` / `ClickHouseMinuteShareSink` 同样 `pub(crate)`，仅测试可注入。
**INV-4C**：上游调用方（P0.15 dispatcher）只能通过 `stream_minute_*_to_clickhouse` 公共函数消费流，不能直接持有 sink。
**INV-4D**：`stream_minute_*_to_clickhouse<S: MinuteSink<...>>` 是公共函数，但类型参数 `S` 受 `pub(crate) trait MinuteSink` 约束——外部 crate 无法构造满足 trait 的具体类型，从而无法调用这两个函数。这是有意为之的「内部 API」标记；P0.15 dispatcher 在本 crate 内，可正常构造 `ClickHouseMinuteKlineSink` 传入。

### INV-5：DDL 集群一致性
**INV-5A**：两张表的 DDL 必须保留 `ON CLUSTER '{cluster}'`，与现有 `kline_data` / `fundamentals` 等表一致。
**INV-5B**：DDL 文本通过 `.replace("'{cluster}'", "single_cluster")` 处理，与现有所有 `create_*_table()` 方法相同；单机环境 `single_cluster` 是合法的集群变量值。

---

## 4. 测试矩阵

### 4.1 单元测试（U1-U8，`src/db/clickhouse/tests.rs`）

| ID | 测试名 | 验证 |
|---|---|---|
| U1 | `decimal_to_f64_normal_range_is_lossless` | 1.23 / 9999.99 / 0 转换为 f64 后 `==` 精确值 |
| U2 | `decimal_to_f64_extreme_value_falls_back_to_zero` | 构造超出 f64 精度的 Decimal，断言返回 0.0（与 kline.rs 静默回退一致） |
| U3 | `naive_to_utc_preserves_wall_clock` | NaiveDateTime "2026-07-04T09:30:00" → DateTime<Utc>，反查 wall-clock 仍是 09:30:00 |
| U4 | `bar_to_row_maps_all_minute_bar_fields` | 构造 `MinuteBar` (volume=i64, amount=Some(Decimal))，断言 `MinuteKlineCH` 字段逐一相等；覆盖 INV-2B/2C/2D |
| U5 | `share_to_row_maps_all_minute_share_fields` | 构造 `MinuteShare`（4 个 Option 字段全 Some），断言 `MinuteShareCH` 字段逐一相等；覆盖 INV-2D |
| U6 | `stream_minute_klines_to_clickhouse_inserts_all_batches_via_mock_sink` | 注入 `MockMinuteKlineSink`，验证 batches/input_records/inserted_records 计数；stream 来自 P0.13d mock client |
| U7 | `stream_minute_shares_to_clickhouse_inserts_all_batches_via_mock_sink` | 同上，对 share 流 |
| U8 | `stream_minute_klines_to_clickhouse_short_circuits_on_first_error` | 注入第 2 batch 失败的 stream，验证函数返回 Err 且 `inserted_records` 只反映第 1 batch |

### 4.2 实时测试（L1-L2，`#[ignore]` + `QUANTIX_CLICKHOUSE_LIVE=1`）

**L1：`tests/clickhouse_live_minute_klines.rs`**

```rust
#[tokio::test]
#[ignore = "live ClickHouse + OpenStock; set QUANTIX_CLICKHOUSE_LIVE=1 to run"]
async fn live_stream_minute_klines_to_clickhouse_round_trip() {
    if std::env::var("QUANTIX_CLICKHOUSE_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let os_client = OpenStockClient::new(OpenStockClientConfig {
        base_url: std::env::var("OPENSTOCK_BASE_URL").unwrap_or_default(),
        api_key: std::env::var("OPENSTOCK_API_KEY").unwrap_or_default(),
        timeout_secs: 30,
    }).expect("os client");
    let ch_client = build_clickhouse_client_from_env().expect("CLICKHOUSE_*");

    let start = NaiveDate::from_ymd_opt(2026, 6, 23).unwrap();
    let end   = NaiveDate::from_ymd_opt(2026, 6, 24).unwrap();

    let sink = ClickHouseMinuteKlineSink { client: &ch_client.client };
    let stats = stream_minute_klines_to_clickhouse(
        &os_client, &sink, "sh600000", MinutePeriod::Minute1, start, end, AdjustType::None,
    ).await.expect("stream ok");

    assert!(stats.batches >= 1);
    assert!(stats.inserted_records > 0);

    // 反查验证
    let rows: Vec<MinuteKlineCH> = ch_client.client
        .query("SELECT * FROM minute_klines WHERE code = ? AND timestamp >= ? AND timestamp <= ? ORDER BY timestamp")
        .bind("sh600000")
        .bind(start.and_hms_opt(0,0,0).unwrap())
        .bind(end.and_hms_opt(23,59,59).unwrap())
        .fetch_all().await.expect("query ok");

    assert!(!rows.is_empty());
    assert_eq!(rows.len() as u64, stats.inserted_records);
}
```

**L2：`tests/clickhouse_live_minute_shares.rs`** — 结构同 L1，对 share 流。

### 4.3 测试覆盖对应不变量

| INV | 覆盖 |
|---|---|
| INV-1 | schema.rs 测试（沿用现有 `init_database` 测试模式）+ L1/L2 反查 |
| INV-2A | L1（timestamp 反查）、U3（naive_to_utc 转换） |
| INV-2B | U4（volume=i64 as f64）、L2（volume 反查） |
| INV-2C | U4（period/adjust 字面量转换） |
| INV-2D | U4 + U5（Option 字段 unwrap_or_default 后字段值匹配） |
| INV-3A | U8 |
| INV-3B | U8（share 对应版本） |
| INV-3C | U6/U7（不 catch）+ U8（首错即止） |
| INV-4 | 编译期保证（`pub(crate)` 不可在 crate 外引用） |
| INV-5 | L1/L2（DDL 在真实集群执行） |

---

## 5. 文件改动清单

### 5.1 修改（4 处）

| 文件 | 改动 | 预估行数 |
|---|---|---|
| `src/db/clickhouse/models.rs` | 新增 `MinuteKlineCH` / `MinuteShareCH` 结构体 + `clickhouse::Row` derive（与 `KlineDataCH` 模式一致） | +50 |
| `src/db/clickhouse/schema.rs` | 新增 `create_minute_klines_table()` / `create_minute_shares_table()` 方法；**在 `init_database()` 中追加调用**（见 §2.2，INV-1A 必需） | +70 |
| `src/db/clickhouse/mod.rs` | `pub mod minute;` + `pub use minute::{StreamStats, stream_minute_*_to_clickhouse};` + 把 `MinuteKlineCH` / `MinuteShareCH` 加入 `pub use self::models::{...}` 列表 | +10 |
| `src/db/clickhouse/tests.rs` | U1-U8 单元测试 | +250 |

### 5.2 新建（5 处）

| 文件 | 用途 | 预估行数 |
|---|---|---|
| `src/db/clickhouse/minute.rs` | 转换 helper + Sink trait + 批量插入 + 流消费 | +200 |
| `tests/clickhouse_live_minute_klines.rs` | L1 实时测试 | +80 |
| `tests/clickhouse_live_minute_shares.rs` | L2 实时测试 | +80 |
| `openspec/changes/openstock-data-consumption-p0-14/{proposal,tasks,design}.md` + `specs/openstock-data-consumption/spec.md` | OpenSpec change（与 P0.13 系列同形） | +300 |
| `.governance/programs/project-governance/cards/P0.14.yaml` | 治理卡片，范围严格限定到 db/clickhouse 子树 | +40 |

**总预估：~1080 行新增 / 0 行删除**

### 5.3 不动（forbidden_paths）

| 路径 | 原因 |
|---|---|
| `src/sources/**` | P0.13 系列冻结面；本切片只消费 stream API |
| `src/db/clickhouse/{kline,fundamentals,gbbq,shadow_kline}.rs` | 现有表写入路径不受影响 |
| `src/cli/**` | CLI 子命令推迟到 P0.15 |
| `src/scheduler/**` | scheduler 推迟到 P0.15 |

### 5.4 治理卡片 scope

`.governance/programs/project-governance/cards/P0.14.yaml`:

```yaml
id: P0.14
title: "ClickHouse minute-level data persistence (klines + shares)"
state: in_progress
scope:
  allowed_paths:
    - src/db/clickhouse/{mod,models,schema,minute,tests}.rs
    - tests/clickhouse_live_minute_{klines,shares}.rs
    - openspec/changes/openstock-data-consumption-p0-14/**
    - docs/superpowers/specs/2026-07-04-openstock-p0-14-clickhouse-minute-persistence-design.md
    - docs/superpowers/plans/2026-07-05-openstock-p0-14-*.md
    - .governance/programs/project-governance/cards/P0.14.yaml
  forbidden_paths:
    - src/sources/**
    - src/cli/**
    - src/scheduler/**
    - src/db/clickhouse/{kline,fundamentals,gbbq,shadow_kline}.rs
linked_openspec: openstock-data-consumption-p0-14
acceptance_gates:
  - cargo fmt --all -- --check
  - cargo clippy --all-targets --workspace -- -D warnings
  - cargo test --workspace
  - openspec validate openstock-data-consumption-p0-14 --strict
  - openspec validate --all --strict
non_goals:
  - "CLI subcommands for minute-* persistence (P0.15)"
  - "scheduler / cron triggers (P0.15)"
  - "ReplacingMergeTree / deduplication (MergeTree + upstream uniqueness)"
  - "Parquet / DuckDB / alternative sinks"
  - "Migration of legacy minute_klines_* tables"
  - "Enum8 column types (would diverge from kline_data convention)"
  - "DateTime64(3, 'Asia/Shanghai') column type (would diverge from kline_data convention)"
```

---

## 6. 验收门禁

```bash
# 1. 格式
cargo fmt --all -- --check

# 2. Lint
cargo clippy --all-targets --workspace -- -D warnings

# 3. 单元 + 集成测试（U1-U8 必须全过；L1/L2 默认 skip）
cargo test --workspace

# 4. OpenSpec 校验
openspec validate openstock-data-consumption-p0-14 --strict
openspec validate --all --strict

# 5. GitNexus 改动分析（期望 LOW，仅 ClickHouse 写入路径）
gitnexus detect_changes
gitnexus detect_changes --scope compare --base_ref master

# 6. 实时冒烟（手动，仅在 NAS 环境可达时运行）
QUANTIX_CLICKHOUSE_LIVE=1 \
OPENSTOCK_BASE_URL=http://192.168.123.104:8040 \
OPENSTOCK_API_KEY=<key> \
CLICKHOUSE_URL=http://192.168.123.104:8123 \
CLICKHOUSE_USER=default \
CLICKHOUSE_PASSWORD=c790414J \
cargo test --test clickhouse_live_minute_klines --test clickhouse_live_minute_shares -- --ignored
```

**门禁通过条件**：1-5 必须全过；6 在 NAS 可达时手动运行，必须返回 OK 且 `inserted_records > 0`。

---

## 7. 风险登记

| ID | 风险 | 缓解 |
|---|---|---|
| R1 | `async_insert=1` + `wait_for_async_insert=1` 在 ClickHouse < 22.x 不支持 | NAS 上 ClickHouse 版本 ≥ 23.x（验证步骤加入 L1 之前的环境检查） |
| R2 | MergeTree 在多 writer 并发下出现 too many parts | P0.14 只支持单 writer 调用（CLI/scheduler 在 P0.15 后才会触发并发）；async_insert=1 内部合并 |
| R3 | `Decimal → f64` 极端值精度损失 | A 股数值范围内（< 10^15）无损；与 `kline.rs:213-219` 静默回退一致（`.unwrap_or(0.0)`） |
| R4 | 新增 `MinutePeriod` / `AdjustType` 变体时字面量映射漏掉 | `period_as_str` / `adjust_as_str` 用 exhaustive match，新增变体编译期强制处理 |
| R5 | `NaiveDateTime → DateTime<Utc>` 时区语义混淆（OpenStock 返回的是北京时间 naive） | 与 `kline_data` 表的 `DateTime<Utc>` 约定一致（同样存储北京时间 naive）；调用方按 A 股东八区语义解读。本切片不解决时区一致性全局问题，仅保持与现有 `kline_data` 不变 |
| R6 | `MinuteSink<T>` trait 泄漏到公共 API | INV-4 编译期保证；`pub(crate)` 在 lib.rs `pub use` 中不出现；`cargo doc` 检查（不含 `--document-private-items` 时 trait 不出现） |

---

## 8. 非目标 (Non-Goals)

- ❌ **CLI 子命令**（`persist minute-klines`、`persist minute-shares`）→ P0.15
- ❌ **scheduler / cron 触发器**（每日定时拉取并落盘）→ P0.15
- ❌ **ReplacingMergeTree / 显式去重**：MergeTree + 上游 `(date, code, period, adjust, timestamp)` 自然唯一已足够
- ❌ **Parquet / DuckDB / 其他 sink**：本切片只 ClickHouse
- ❌ **遗留 `minute_klines_*` 表迁移**：旧表存在与否不影响本切片
- ❌ **流控 / 背压**：`Vec<T>` per batch 是天然单位
- ❌ **批量回填（multi-month backfill）工具**：本切片只提供 API；批量回填由 P0.15 CLI 子命令承担
- ❌ **数据质量监控（dq checks、anomaly detection）**：交给消费侧（P0.16+）做
- ❌ **跨 provider 对账**（tdx-api vs OpenStock）：完全 out of scope
- ❌ **修改 P0.13 系列公共 API**：stream 接口已冻结，本切片只消费
- ❌ **Enum8 列类型 / `DateTime64(3, 'Asia/Shanghai')` 时区列**：会与现有 `kline_data` 表的 `String` period + `DateTime` 约定分歧；本切片严格对齐 `kline_data`
- ❌ **统一时区语义重构**（让所有表都用 `DateTime64(3, 'Asia/Shanghai')`）：是单独的重构切片，不在 P0.14 范围

---

## 9. 决策记录 (Decisions)

### D1：表引擎 = MergeTree（非 ReplacingMergeTree）
**选择**：`MergeTree()`（与 `kline_data` 一致，带括号）
**理由**：与 `kline_data` 现有规范一致；上游流按 `(date, code, period, adjust, timestamp)` 自然唯一，不需要 CH 层去重。
**Rejected**：`ReplacingMergeTree`（`stock_info`/`gbbq_events` 用，但 minute 数据不需要版本化）、`AggregatingMergeTree`。

### D2：DateTime（无时区）+ String period/adjust，完全对齐 KlineDataCH
**选择**：`timestamp DateTime`、`period String`、`adjust String`；Rust 侧 `DateTime<Utc>` 中转。
**理由**：100% 复刻 `KlineDataCH`（`models.rs:33-47`）和 `kline_data` 表（`schema.rs:97-118`）的类型约定；零新 Convention；与 `kline.rs:210` 的 `DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)` 写入路径完全一致；`kline.rs:213-219` 的 `to_f64().unwrap_or(0.0)` 转换模式可直接复用。
**Rejected**：`DateTime64(3, 'Asia/Shanghai')` + Rust `FixedOffset`（会与 `kline_data` 分歧，引入新 Convention）；`chrono-tz` 依赖（本切片不需要）；Enum8 列类型（`clickhouse-rs` Row derive 对 Enum8 支持需要额外 Rust enum 定义，与 `kline_data` 的 String 模式分歧）。

### D3：独立新建 `minute.rs`，不与 `kline.rs` 合并
**选择**：`src/db/clickhouse/minute.rs` 独立文件。
**理由**：`kline.rs` 已有 358 行日线查询逻辑，minute 路径是纯流式写入，关注点不同；CLAUDE.md 文件大小规范要求单文件 < 500 行。
**Rejected**：把 minute 写入合并到 `kline.rs`（违反单一职责）。

### D4：Sink trait `pub(crate)`，仅用于单元测试 mock
**选择**：`pub(crate) trait MinuteSink<T>` + `pub(crate) struct ClickHouseMinute*Sink`，trait 作为流消费函数的泛型约束。
**理由**：trait 抽象的价值仅在于 mock 注入；公共 API `stream_minute_*_to_clickhouse<S: MinuteSink<...>>` 通过泛型接收 sink，但 `S` 受 `pub(crate) trait` 约束，外部 crate 无法构造具体类型，事实上是「内部 API」。
**Rejected**：完全私有 trait（测试无法注入）；`pub` trait（暴露不必要的抽象）。

### D5：DDL 保留 `ON CLUSTER '{cluster}'` + `.replace("'{cluster}'", "single_cluster")`
**选择**：DDL 文本保留 `ON CLUSTER '{cluster}'`，运行时通过 `.replace("'{cluster}'", "single_cluster")` 处理，与现有 5 张表完全一致。
**理由**：集群变量在单机环境展开为 `single_cluster`，DDL 仍可执行；保留这一行让 DDL 模板在所有环境下都能直接复用。
**Rejected**：根据环境变量移除 `ON CLUSTER`（增加分支，与现有规范分歧）。

### D6：Decimal → f64 用 `to_f64().unwrap_or(0.0)`，与 kline.rs 一致（不写 warn）
**选择**：`use rust_decimal::prelude::*;` 然后 `dec.to_f64().unwrap_or(0.0)`。
**理由**：与 `kline.rs:213-219`、`shadow_kline.rs:50-55` 等现有写入路径 100% 一致；A 股数值范围内（< 10^15）转换不可能失败，`.unwrap_or(0.0)` 是死代码分支防御；写 warn 会与现有路径不一致，且正常运行时刷日志。
**Rejected**：`try_into` + warn（与现有路径不一致）；返回 `Result` 传播错误（违反「单坏值不阻塞 batch」原则）。

---

## 10. 与 P0.13 系列的衔接

| 来源 | P0.14 衔接点 |
|---|---|
| P0.13a `MinutePeriod` enum | 在 `period_as_str` 转换处使用 |
| P0.13b-1 `MinuteBar` struct | `bar_to_row` 输入 |
| P0.13b-2 `MinuteShare` struct | `share_to_row` 输入 |
| P0.13c `DateOrRange::Range` | 流消费函数内部构造并传给 stream API |
| P0.13d `fetch_minute_*_stream` | 流消费函数的 stream 源 |
| P0.13d D4 首错即止 | INV-3A/3B/3C 直接继承 |
| P0.13d INV-4A 公共 API 不变 | 本切片不修改 `src/sources/**`，零影响 |

**P0.14 → P0.15 衔接**：

- P0.15 CLI dispatcher 直接调用 `stream_minute_klines_to_clickhouse` / `stream_minute_shares_to_clickhouse`，传入 `ClickHouseMinuteKlineSink { client: &ch_client.client }`
- P0.15 scheduler 周期触发同样调用这两个函数
- 公共 API 在 P0.14 冻结，P0.15 不再调整

---

## 11. 实施顺序（给 writing-plans 的提示）

建议任务拆分（每个任务一个独立可测试交付）：

1. **T1：DDL + 模型**：`schema.rs` 新增两个 `create_*_table()` 方法并在 `init_database()` 中追加调用（§2.2）；`models.rs` 新增 `MinuteKlineCH` / `MinuteShareCH`；schema 单元测试（验证 INV-1A/1B/2A）
2. **T2：转换 helper + 流消费**：`minute.rs` 新建；`decimal_to_f64` / `naive_to_utc` / `period_as_str` / `adjust_as_str` / `bar_to_row` / `share_to_row` + Sink trait + `stream_minute_*_to_clickhouse`；U1-U8 单元测试（含 mock 注入）
3. **T3：实时测试**：L1/L2 文件，`#[ignore]` + 环境变量门控
4. **T4：OpenSpec + 治理**：`openspec/changes/.../` + `P0.14.yaml` 卡片

每个任务都遵循 TDD（先写测试 → 实现 → 验证 → 提交）。

---

## 12. 总结

P0.14 是一个边界清晰、风险可控的「管道」切片：

- **上游**：复用 P0.13d 已冻结的流式 API，零修改
- **本切片**：实现「流 → batch → ClickHouse」的最短路径，独立 `minute.rs`，与 `kline_data` / `KlineDataCH` 完全同构的 MergeTree 表
- **下游**：为 P0.15 CLI/scheduler 提供干净的公共 API (`stream_minute_*_to_clickhouse`)

通过 6 项决策（MergeTree / DateTime+String 对齐 / 独立文件 / pub(crate) sink / DDL 集群语法 / unwrap_or 静默回退）锁定设计，通过 9 项不变量 + 8 个单元测试 + 2 个实时测试覆盖关键路径，通过严格的 forbidden_paths 保证 P0.13 冻结面和 P0.15 延后边界。

**与上一版 spec 的差异（2026-07-04 → 2026-07-05 修订）**：

- DDL：`DateTime64(3, 'Asia/Shanghai')` → `DateTime`；移除 Enum8 列；保留 `MATERIALIZED toDate(timestamp)` 与 `kline_data` 一致
- 类型映射：`DateTime<FixedOffset>` → `DateTime<Utc>`；`PeriodCH`/`AdjustCH` enum → `String` 字面量
- 转换 helper：`naive_to_shanghai` → `naive_to_utc`；新增 `period_as_str` / `adjust_as_str`
- D6：`try_into` + warn → `to_f64().unwrap_or(0.0)` 静默回退（与 kline.rs 一致）
- 总行数：~1170 → ~1080（移除 Enum8 相关代码）
- 新增 INV-2D 锁定 Option unwrap_or_default 安全性
- 修订 INV-4D 显式说明 `pub(crate) trait` 作为泛型约束的外部不可构造语义
