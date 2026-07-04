# OpenStock P0.14 — ClickHouse 分钟级数据持久化设计

> 日期：2026-07-04
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

**与现有规范对齐**

- DDL 保留 `ON CLUSTER '{cluster}'` 与现有 `kline_data`、`fundamentals`、`gbbq`、`shadow_kline` 表一致；集群变量在单机环境会展开为单节点列表，无需 fallback 分支
- 时区统一使用 `DateTime64(3, 'Asia/Shanghai')`，由 `FixedOffset::east_opt(8 * 3600)` 在 Rust 侧构造 `DateTime<FixedOffset>`，不引入 `chrono-tz` 依赖
- 引擎选择 MergeTree（非 ReplacingMergeTree），与 `kline_data` 现有规范一致；去重交给上游流（按 `(date, code, period, adjust, timestamp)` 自然唯一）

---

## 1. 表结构

### 1.1 `quantix.minute_klines`

```sql
CREATE TABLE IF NOT EXISTS minute_klines ON CLUSTER '{cluster}'
(
    `timestamp`     DateTime64(3, 'Asia/Shanghai'),
    `code`          LowCardinality(String),
    `period`        Enum8('1m' = 1, '5m' = 2, '15m' = 3, '30m' = 4, '60m' = 5),
    `adjust`        Enum8('none' = 1, 'qfq' = 2, 'hfq' = 3),
    `open`          Float64,
    `high`          Float64,
    `low`           Float64,
    `close`         Float64,
    `volume`        Float64,
    `amount`        Float64,
    `ingested_at`   DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3)
)
ENGINE = MergeTree
PARTITION BY (period, toYYYYMM(timestamp))
ORDER BY (toDate(timestamp), code, period, adjust, timestamp)
SETTINGS index_granularity = 8192;
```

**字段映射**（`MinuteBar` → `MinuteKlineCH`）

| MinuteBar 字段 | CH 列 | 备注 |
|---|---|---|
| `timestamp: NaiveDateTime` | `timestamp` | 由 naive + +08:00 偏移构造为 `DateTime<FixedOffset>`（见 §2.1 `naive_to_shanghai`） |
| `code: String` | `code` | LowCardinality 收益显著（≤ 10k 个 code） |
| `period: MinutePeriod` | `period` | Enum8 转换（见 §2.5 Rust enum ↔ CH Enum8 映射） |
| `adjust_type: AdjustType` | `adjust` | Enum8 转换（同上） |
| `open/high/low/close: Decimal` | `open/high/low/close` | 见 §1.3 decimal_to_f64 |
| `volume: i64` | `volume` | `bar.volume as f64`；i64→f64 cast 在 ≤ 10^9 范围无损（A 股单 bar 远小于此） |
| `amount: Option<Decimal>` | `amount` | parser 已保证非 None（见下文 Option 处理说明），`bar.amount.unwrap()` 后 `decimal_to_f64` |
| —（CH 默认） | `ingested_at` | `DEFAULT now64(3)`，写入时不显式赋值 |

**Option 字段处理说明（INV-2B-Option）**

`MinuteBar.amount` 实际为 `Option<Decimal>`（`data/models.rs:146`），但 `parse_minute_klines`/`parse_minute_share` 已在 parser 阶段对关键字段做 `?` 解包——任一字段缺失该条记录直接被 warn + skip，不会到达 `MinuteBar`。因此 `bar_to_row` 中 `bar.amount.unwrap()`（或 `expect("parser guarantees Some")`）是安全的，运行时绝不可能 panic。`share_to_row` 对 `MinuteShare` 的 4 个 Option 字段同理。

**ORDER BY 依据**

- `toDate(timestamp)` 排第一：日级范围查询直接命中分区+前缀索引
- `code` 第二：跨日单 code 查询（最常见场景）顺序读
- `period` + `adjust` 第三：多周期/复权组合时仍能紧凑扫描
- `timestamp` 末位：同一日内按时间排序

### 1.2 `quantix.minute_shares`

```sql
CREATE TABLE IF NOT EXISTS minute_shares ON CLUSTER '{cluster}'
(
    `timestamp`     DateTime64(3, 'Asia/Shanghai'),
    `code`          LowCardinality(String),
    `price`         Float64,
    `volume`        Float64,
    `amount`        Float64,
    `avg_price`     Float64,
    `ingested_at`   DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3)
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(timestamp)
ORDER BY (toDate(timestamp), code, timestamp)
SETTINGS index_granularity = 8192;
```

**字段映射**（`MinuteShare` → `MinuteShareCH`）

| MinuteShare 字段 | CH 列 | 备注 |
|---|---|---|
| `timestamp: NaiveDateTime` | `timestamp` | 由 naive + +08:00 偏移构造为 `DateTime<FixedOffset>`（见 §2.1 `naive_to_shanghai`） |
| `code: String` | `code` | LowCardinality |
| `price: Option<Decimal>` | `price` | parser 已保证非 None，`share.price.unwrap()` 后 `decimal_to_f64` |
| `volume: Option<i64>` | `volume` | 同上，`share.volume.unwrap() as f64`；i64→f64 cast 在 ≤ 10^9 范围无损 |
| `amount: Option<Decimal>` | `amount` | 同上，unwrap 后 `decimal_to_f64` |
| `avg_price: Option<Decimal>` | `avg_price` | 同上 |
| —（CH 默认） | `ingested_at` | `DEFAULT now64(3)` |

Option 处理路径与 §1.1 同理：parser 在 `parse_minute_share` 中已对 price/volume/amount/avg_price 做了 `?` 解包，到达 `MinuteShare` 的字段必为 `Some`，`unwrap` 不会 panic。

**为何 share 没有 `period` / `adjust`**

P0.13b-2 已确认 `MinuteShare` 是分笔成交，没有 period/adjust 概念。表结构反映了领域模型的这一差异。

### 1.3 decimal_to_f64 转换约束

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
> /// 任何超出 10^15 的值会被 `try_into` 拒绝并记录 warning，回退到 0.0。
> /// 这是 P0.14 的明确决策（见设计 §1.3），不在运行时降级到更高精度类型。
> ///
> /// 注意：volume 是 `i64` 而非 `Decimal`，由调用方直接 `as f64`（见 §1.1）。
> /// `decimal_to_f64` 不处理 volume。
> fn decimal_to_f64(v: Decimal) -> f64 { ... }
> ```

**回退策略**

`Decimal::try_into` 失败时（理论不可能；防御性）：
1. 返回 `0.0`
2. `tracing::warn!("decimal_to_f64 overflow: value={} — falling back to 0.0", v)`
3. 调用方继续写入（不阻塞流）

不抛错、不中断流，因为单条坏值不应阻塞整个 batch；运行时通过 `StreamStats::skipped_records` 反映。

**i64 volume 不走 `decimal_to_f64`**

`MinuteBar.volume: i64` 和 `MinuteShare.volume: Option<i64>`（unwrap 后）直接 `as f64`。A 股单 bar volume ≤ 10^9，远低于 2^53，cast 无损。

---

## 2. 写入层代码设计

### 2.1 新建 `src/db/clickhouse/minute.rs`（独立文件，单一职责）

**为什么独立文件**

- `kline.rs` 已有 400+ 行，与日线聚合逻辑耦合
- minute 路径没有聚合，纯直写，逻辑独立
- 拆分避免一个文件 > 800 行的硬阈值（见 CLAUDE.md 文件大小规范）
- 后续 P0.15 添加 CLI 子命令时，dispatcher 直接 `use minute::insert_minute_klines_batch` 即可

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
use chrono::{DateTime, FixedOffset, NaiveDate};
use clickhouse::Client;
use futures::StreamExt;
use rust_decimal::prelude::TryInto as _;
use rust_decimal::Decimal;

// ─── 时区构造 ──────────────────────────────────────────────────────────────

/// Shanghai timezone offset (+08:00). Constructed once; reused for all
/// DateTime<FixedOffset> conversions in this module.
fn shanghai_offset() -> FixedOffset {
    FixedOffset::east_opt(8 * 3600).expect("8*3600 is a valid offset")
}

/// Lift a NaiveDate to a Shanghai-midnight DateTime for partition key use.
fn naive_to_shanghai(d: NaiveDate) -> DateTime<FixedOffset> {
    d.and_hms_milli_opt(0, 0, 0, 0)
        .unwrap()
        .and_local_timezone(shanghai_offset())
        .unwrap()
}

// ─── Decimal → f64 ──────────────────────────────────────────────────────────

/// Convert Decimal to f64 for ClickHouse Float64 columns.
///
/// A 股数值范围内（|v| < 10^15）Decimal → f64 转换无损：
/// - 价格：[0.01, 9999.99]，远低于 2^53
/// - 成交量：单 bar ≤ 10^9
/// - 成交额：单 bar ≤ 10^12
///
/// 任何超出 10^15 的值会被 `try_into` 拒绝并记录 warning，回退到 0.0。
/// 这是 P0.14 的明确决策（见设计 §1.3），不在运行时降级到更高精度类型。
fn decimal_to_f64(v: Decimal) -> f64 {
    v.try_into().unwrap_or_else(|_| {
        tracing::warn!(value = %v, "decimal_to_f64 overflow; falling back to 0.0");
        0.0
    })
}

// ─── 行转换 ────────────────────────────────────────────────────────────────

fn bar_to_row(bar: &MinuteBar) -> MinuteKlineCH { ... }
fn share_to_row(share: &MinuteShare) -> MinuteShareCH { ... }

// ─── Sink trait (pub(crate), 仅用于单元测试 mock) ───────────────────────────

/// Internal sink abstraction. Used **only** by unit tests in
/// `src/db/clickhouse/tests.rs` to inject a mock without touching the
/// real ClickHouse. Not part of any public API; no upstream call sites
/// depend on this trait.
pub(crate) trait MinuteSink<T: Send + Sync>: Send + Sync {
    async fn insert_batch(&self, batch: &[T]) -> Result<usize, clickhouse::error::Error>;
}

pub(crate) struct ClickHouseMinuteKlineSink { client: Client }
pub(crate) struct ClickHouseMinuteShareSink { client: Client }

impl MinuteSink<MinuteKlineCH> for ClickHouseMinuteKlineSink {
    async fn insert_batch(&self, batch: &[MinuteKlineCH]) -> Result<usize, clickhouse::error::Error> {
        // 详见 §2.2
        ...
    }
}

// ... share sink 同理 ...

// ─── 流消费 ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StreamStats {
    pub batches: u64,
    pub input_records: u64,
    pub skipped_records: u64,
    pub inserted_records: u64,
}

/// Consume the klines stream and insert each batch into ClickHouse.
///
/// Pinning is required because `fetch_minute_klines_stream` returns
/// `impl Stream + 'a` (not Unpin). We use `futures::pin_mut!` rather
/// than requiring the caller to pin — the function is the natural owner
/// of the pinning scope.
pub async fn stream_minute_klines_to_clickhouse(
    client: &OpenStockClient,
    clickhouse: &Client,
    code: &str,
    period: MinutePeriod,
    start: NaiveDate,
    end: NaiveDate,
    adjust: AdjustType,
) -> Result<StreamStats, crate::error::QuantixError> { ... }

pub async fn stream_minute_shares_to_clickhouse(
    client: &OpenStockClient,
    clickhouse: &Client,
    code: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<StreamStats, crate::error::QuantixError> { ... }
```

### 2.5 Rust enum ↔ CH Enum8 映射（period / adjust）

`clickhouse = "0.12"` crate 的 `Row` derive 要求 Rust enum ↔ CH Enum8 编号严格对应。本切片新增两个内部 enum（仅在 `models.rs` 中作为 CH 行类型的一部分），变体顺序与 §1.1 DDL 中 Enum8 编号锁定一致：

```rust
// src/db/clickhouse/models.rs

#[derive(Debug, Clone, clickhouse::Row, Serialize, Deserialize)]
pub struct MinuteKlineCH {
    pub timestamp: DateTime<FixedOffset>,
    pub code: String,
    pub period: PeriodCH,
    pub adjust: AdjustCH,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
    // ingested_at 由 CH DEFAULT now64(3) 处理，Rust 侧不写
}

/// CH Enum8 编号必须与 DDL `Enum8('1m'=1,'5m'=2,'15m'=3,'30m'=4,'60m'=5)` 一致。
/// 变体顺序即编号顺序；不可调整。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i8)]
pub enum PeriodCH {
    Minute1 = 1,
    Minute5 = 2,
    Minute15 = 3,
    Minute30 = 4,
    Minute60 = 5,
}

/// CH Enum8 编号必须与 DDL `Enum8('none'=1,'qfq'=2,'hfq'=3)` 一致。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i8)]
pub enum AdjustCH {
    None = 1,
    QFQ = 2,
    HFQ = 3,
}
```

**业务 enum ↔ CH enum 转换**（在 `minute.rs` 中，私有 helper）：

```rust
fn period_to_ch(p: MinutePeriod) -> PeriodCH {
    match p {
        MinutePeriod::Minute1 => PeriodCH::Minute1,
        MinutePeriod::Minute5 => PeriodCH::Minute5,
        MinutePeriod::Minute15 => PeriodCH::Minute15,
        MinutePeriod::Minute30 => PeriodCH::Minute30,
        MinutePeriod::Minute60 => PeriodCH::Minute60,
    }
}

fn adjust_to_ch(a: AdjustType) -> AdjustCH {
    match a {
        AdjustType::None => AdjustCH::None,
        AdjustType::QFQ => AdjustCH::QFQ,
        AdjustType::HFQ => AdjustCH::HFQ,
    }
}
```

**exhaustive match 保证**：未来若 `MinutePeriod` / `AdjustType` 新增变体，编译期会强制这两个 match 添加分支——这是 R4（Enum8 编号漂移）的编译期防线。`MinuteShareCH` 不含 enum 字段，结构更简单。

### 2.6 schema.rs：在 init_database 中注册新表创建调用

`src/db/clickhouse/schema.rs` 的 `init_database()` 函数当前依次调用 `create_kline_data_table()` 等六个建表方法。本切片新增的两个方法 **必须** 在 `init_database()` 中按相同模式追加调用，否则 INV-1A 不成立：

```rust
// src/db/clickhouse/schema.rs
pub async fn init_database(client: &Client) -> Result<(), ...> {
    create_kline_data_table(client).await?;
    create_fundamentals_table(client).await?;
    create_gbbq_table(client).await?;
    create_shadow_kline_table(client).await?;
    // ↓ P0.14 新增 ↓
    create_minute_klines_table(client).await?;
    create_minute_shares_table(client).await?;
    Ok(())
}
```

### 2.2 批量插入实现（async_insert=1）

`ClickHouseMinuteKlineSink::insert_batch` 内部：

```rust
async fn insert_batch(&self, batch: &[MinuteKlineCH]) -> Result<usize, clickhouse::error::Error> {
    if batch.is_empty() {
        return Ok(0);
    }
    let ch = self.client.clone();
    let insert = ch
        .insert("minute_klines")?
        .with_option("async_insert", "1")?
        .with_option("wait_for_async_insert", "1")?;
    // 写入所有行
    for row in batch {
        insert.write(row).await?;
    }
    insert.end().await?;
    Ok(batch.len())
}
```

**为何 `async_insert=1` + `wait_for_async_insert=1`**

- `async_insert=1`：服务端把多个客户端的小 batch 合并落地，避免 too many parts
- `wait_for_async_insert=1`：客户端等待服务端确认写入完成，错误能正确传回；不开启则只能从系统表反查
- 两者搭配是 ClickHouse 22.x+ 推荐的小批量写入模式

**为何不用 `inserter()`（缓冲式 inserter）**

- inserter 在 client 持久化场景表现好，但本切片是「一次性流消费」，每个 batch 即显式 flush
- 缓冲与流的天然边界冲突：stream 已经按 7 天 / 1 天切片，缓冲就是多余一层

### 2.3 流消费实现（pin_mut! + 首错即止）

```rust
pub async fn stream_minute_klines_to_clickhouse(
    client: &OpenStockClient,
    clickhouse: &Client,
    code: &str,
    period: MinutePeriod,
    start: NaiveDate,
    end: NaiveDate,
    adjust: AdjustType,
) -> Result<StreamStats, crate::error::QuantixError> {
    use crate::data::models::DateOrRange;

    let dor = DateOrRange::Range { start, end };
    let stream = client.fetch_minute_klines_stream(code, period, dor, adjust);
    futures::pin_mut!(stream);

    let sink = ClickHouseMinuteKlineSink { client: clickhouse.clone() };
    let mut stats = StreamStats::default();

    while let Some(batch_result) = stream.next().await {
        let batch = batch_result?;  // 首错即止 — 与 P0.13d D4 一致
        stats.batches += 1;
        stats.input_records += batch.len() as u64;

        let rows: Vec<MinuteKlineCH> = batch.iter().map(bar_to_row).collect();
        let inserted = sink.insert_batch(&rows)
            .await
            .map_err(|e| crate::error::QuantixError::Other(format!("ch insert minute_klines: {}", e)))?;
        stats.inserted_records += inserted as u64;
        stats.skipped_records += (rows.len() - inserted) as u64;
    }

    Ok(stats)
}
```

`stream_minute_shares_to_clickhouse` 结构相同，差异仅在 sink 类型 + `fetch_minute_share_stream`。

### 2.4 pub(crate) 边界

| 项 | 可见性 | 说明 |
|---|---|---|
| `decimal_to_f64` / `naive_to_shanghai` / `shanghai_offset` | 私有 | 仅模块内 |
| `bar_to_row` / `share_to_row` / `period_to_ch` / `adjust_to_ch` | 私有 | 仅模块内 |
| `MinuteSink<T>` trait | `pub(crate)` | **仅测试**注入 mock；非公共 API |
| `ClickHouseMinuteKlineSink` / `ClickHouseMinuteShareSink` | `pub(crate)` | 同上 |
| `PeriodCH` / `AdjustCH` enums | `pub` | 作为 `MinuteKlineCH` 公共字段类型，必须可访问；非测试目的 |
| `MinuteKlineCH` / `MinuteShareCH` structs | `pub` | CH 行类型，给 L1/L2 实时测试反查用 |
| `insert_minute_klines_batch` / `insert_minute_shares_batch` | `pub` | 给 P0.15 CLI dispatcher 用 |
| `stream_minute_*_to_clickhouse` | `pub` | 给 P0.15 CLI dispatcher 用 |
| `StreamStats` | `pub` | 流式结果报告 |

---

## 3. 不变量 (Invariants)

### INV-1：表存在性
**INV-1A**：`init_database()` 完成后，`quantix.minute_klines` 和 `quantix.minute_shares` 必须存在。
**INV-1B**：两张表的 `ENGINE` 必须为 `MergeTree`（与 `kline_data` 一致），不使用 `ReplacingMergeTree`。

### INV-2：类型映射
**INV-2A**：`MinuteBar::timestamp` (实际类型 `NaiveDateTime`) 必须经 `naive_to_shanghai()` 抬升到 `DateTime<FixedOffset>` (+08:00) 后写入 `DateTime64(3, 'Asia/Shanghai')`；写入和读回的 wall-clock 时刻必须相等。`MinuteShare::timestamp` 同理。
**INV-2B**：`MinuteBar::volume: i64` 与 `MinuteShare::volume: Option<i64>`（unwrap 后）通过 `as f64` 写入 `Float64` 列；读回值必须与原值在数值上相等（i64 ≤ 10^9 时 f64 精确）。
**INV-2C**：`MinuteBar::period` / `adjust_type` 通过 `period_to_ch` / `adjust_to_ch` 转换为 `PeriodCH` / `AdjustCH`，再写入 CH Enum8 列；变体顺序与 §2.5 DDL Enum8 编号严格锁定一致。
**INV-2D（Option unwrap 安全性）**：`bar_to_row` / `share_to_row` 对 `MinuteBar.amount: Option<Decimal>` 和 `MinuteShare::{price, volume, amount, avg_price}` 等 Option 字段直接 `unwrap()` 是安全的——parser 阶段已通过 `?` 操作过滤 None；运行时绝不可能 panic。任何新增的 Option 字段必须在 parser 阶段同步加入「字段缺失 → warn + skip」逻辑，才能在 row helper 中 unwrap。
**INV-2E（Enum8 编号锁定）**：`PeriodCH` / `AdjustCH` 的 `#[repr(i8)]` 编号必须与 §1.1 / §2.5 DDL 中 Enum8 字面量编号严格对应。未来若 `MinutePeriod` / `AdjustType` 业务 enum 新增变体，`period_to_ch` / `adjust_to_ch` 的 exhaustive match 会在编译期强制要求新增对应分支——这是 R4 的编译期防线。

### INV-3：流语义继承
**INV-3A**：`stream_minute_klines_to_clickhouse` 必须在第一个流错误时短路（`?`），不再继续消费后续 batch。继承自 P0.13d D4。
**INV-3B**：`stream_minute_shares_to_clickhouse` 必须在第一个 batch 失败时短路，不再继续写入。
**INV-3C**：流消费函数不得在内部 catch 错误并继续；任何错误必须传播给调用方。

### INV-4：Sink trait 不外泄
**INV-4A**：`MinuteSink<T>` trait 的可见性必须为 `pub(crate)`，**不得**出现在 `lib.rs` 的 `pub use` 中。
**INV-4B**：`ClickHouseMinuteKlineSink` / `ClickHouseMinuteShareSink` 同样 `pub(crate)`，仅测试可注入。
**INV-4C**：上游调用方（P0.15 dispatcher）只能通过 `stream_minute_*_to_clickhouse` 公共函数消费流，不能直接持有 sink。

### INV-5：DDL 集群一致性
**INV-5A**：两张表的 DDL 必须保留 `ON CLUSTER '{cluster}'`，与现有 `kline_data` / `fundamentals` 等表一致。
**INV-5B**：单机环境下 `'{cluster}'` 展开为单节点列表，DDL 必须在该模式下也能成功执行（无需 fallback 分支）。

---

## 4. 测试矩阵

### 4.1 单元测试（U1-U8，`src/db/clickhouse/tests.rs`）

| ID | 测试名 | 验证 |
|---|---|---|
| U1 | `decimal_to_f64_normal_range_is_lossless` | 1.23 / 9999.99 / 0 转换为 f64 后 `==` 精确值 |
| U2 | `decimal_to_f64_huge_value_falls_back_with_warn` | 构造超出 f64 精度的 Decimal，返回 0.0 + warn（用 `tracing::mock` 或 `assert_eq!(result, 0.0)` 验证回退） |
| U3 | `naive_to_shanghai_applies_east_8_offset` | UTC 0:00 NaiveDateTime → DateTime 带 +08:00，与 8:00 UTC 等价 |
| U4 | `bar_to_row_maps_all_minute_bar_fields` | 构造 `MinuteBar` (volume=i64, amount=Some(Decimal))，断言 `MinuteKlineCH` 字段逐一相等；覆盖 INV-2B/2D |
| U5 | `share_to_row_maps_all_minute_share_fields` | 构造 `MinuteShare`（4 个 Option 字段全 Some），断言 `MinuteShareCH` 字段逐一相等；覆盖 INV-2D |
| U6 | `stream_minute_klines_to_clickhouse_inserts_all_batches_via_mock_sink` | 注入 `MockMinuteKlineSink`，验证 batches/input_records/inserted_records 计数；stream 来自 P0.13d mock client |
| U7 | `stream_minute_shares_to_clickhouse_inserts_all_batches_via_mock_sink` | 同上，对 share 流 |
| U8 | `stream_minute_klines_to_clickhouse_short_circuits_on_first_error` | 注入第 2 batch 失败的 stream，验证函数返回 Err 且 `inserted_records` 只反映第 1 batch |
| U9 | `period_to_ch_and_adjust_to_ch_map_all_variants` | 遍历 `MinutePeriod` × `AdjustType` 全部组合，断言 Rust enum → `PeriodCH`/`AdjustCH` 映射与 DDL Enum8 编号一致；覆盖 INV-2C/2E |

**Mock 注入机制**

```rust
// src/db/clickhouse/tests.rs
struct MockMinuteKlineSink {
    inserted: Mutex<Vec<MinuteKlineCH>>,
    fail_on_batch: Option<usize>,  // Some(1) 表示第 2 batch 失败
}

impl MinuteSink<MinuteKlineCH> for MockMinuteKlineSink { ... }

#[tokio::test]
async fn stream_minute_klines_to_clickhouse_inserts_all_batches_via_mock_sink() {
    // 通过泛型参数注入 sink：流消费函数需要支持
    // `with_sink(sink: impl MinuteSink<T>)` 形式，仅在 `#[cfg(test)]` 暴露
    ...
}
```

> **注**：流消费函数的 Sink 注入需要在 §2.3 的签名中预留一个泛型参数（或专门为测试提供 `pub(crate) fn stream_minute_klines_to_clickhouse_with_sink(...)` 变体）。这是 INV-4 的合法例外：测试 hook 必须可注入，但仍然不暴露到 crate 外。

### 4.2 实时测试（L1-L2，`#[ignore]` + `QUANTIX_CLICKHOUSE_LIVE=1`）

**L1：`tests/clickhouse_live_minute_klines.rs`**

```rust
#[tokio::test]
#[ignore = "live ClickHouse + OpenStock; set QUANTIX_CLICKHOUSE_LIVE=1 to run"]
async fn live_stream_minute_klines_to_clickhouse_round_trip() {
    if std::env::var("QUANTIX_CLICKHOUSE_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    // 1. 读 OpenStock + CH 配置
    let os_client = OpenStockClient::from_env().expect("OPENSTOCK_*");
    let ch_client = build_clickhouse_client_from_env().expect("CLICKHOUSE_*");

    // 2. 选小范围（sh600000, 1m, none, 2026-06-23..2026-06-24，2 个交易日）
    let start = NaiveDate::from_ymd_opt(2026, 6, 23).unwrap();
    let end   = NaiveDate::from_ymd_opt(2026, 6, 24).unwrap();

    // 3. 调用 stream_minute_klines_to_clickhouse
    let stats = stream_minute_klines_to_clickhouse(&os_client, &ch_client, "sh600000",
        MinutePeriod::Minute1, start, end, AdjustType::None)
        .await.expect("stream ok");

    // 4. 断言 stats.batches >= 1，stats.inserted_records > 0
    assert!(stats.batches >= 1);
    assert!(stats.inserted_records > 0);

    // 5. 反查 ClickHouse，验证 timestamp 范围在 [start, end] 内
    let rows: Vec<MinuteKlineCH> = ch_client
        .query("SELECT * FROM minute_klines WHERE code = ? AND timestamp >= ? AND timestamp <= ? ORDER BY timestamp")
        .bind("sh600000")
        .bind(start.and_hms_opt(0,0,0).unwrap())
        .bind(end.and_hms_opt(23,59,59).unwrap())
        .fetch_all().await.expect("query ok");

    assert!(!rows.is_empty());
    assert_eq!(rows.len() as u64, stats.inserted_records);

    // 6. 清理（避免污染表）— 可选；或保留作为 smoke 证据
}
```

**L2：`tests/clickhouse_live_minute_shares.rs`** — 结构同 L1，对 share 流。

### 4.3 测试覆盖对应不变量

| INV | 覆盖 |
|---|---|
| INV-1 | schema.rs 测试（沿用现有 `init_database` 测试模式，验证 §2.6 新调用链）+ L1/L2 反查 |
| INV-2A | L1（timestamp 反查）、U3（时区构造） |
| INV-2B | U4（volume=i64 as f64）、L2（volume 反查） |
| INV-2C | U9（period/adjust enum 映射表） |
| INV-2D | U4 + U5（Option 字段 unwrap 后字段值匹配） |
| INV-2E | U9 + 编译期 exhaustive match 保证 |
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
| `src/db/clickhouse/models.rs` | 新增 `MinuteKlineCH` / `MinuteShareCH` 结构体 + `PeriodCH` / `AdjustCH` enum（含 `#[repr(i8)]` 编号锁定，见 §2.5） | +90 |
| `src/db/clickhouse/schema.rs` | 新增 `create_minute_klines_table()` / `create_minute_shares_table()` 方法；**在 `init_database()` 中追加调用**（见 §2.6，INV-1A 必需） | +70 |
| `src/db/clickhouse/mod.rs` | `pub mod minute;` + `pub use minute::{StreamStats, stream_minute_*_to_clickhouse};` | +5 |
| `src/db/clickhouse/tests.rs` | U1-U9 单元测试（U9 为 enum 映射表测试，覆盖 INV-2C/2E） | +275 |

### 5.2 新建（5 处）

| 文件 | 用途 | 预估行数 |
|---|---|---|
| `src/db/clickhouse/minute.rs` | 转换 helper + enum 映射 + Sink trait + 批量插入 + 流消费 | +230 |
| `tests/clickhouse_live_minute_klines.rs` | L1 实时测试 | +80 |
| `tests/clickhouse_live_minute_shares.rs` | L2 实时测试 | +80 |
| `openspec/changes/openstock-data-consumption-p0-14/{proposal,tasks,design}.md` + `specs/openstock-data-consumption/spec.md` | OpenSpec change（与 P0.13 系列同形） | +300 |
| `.governance/programs/project-governance/cards/P0.14.yaml` | 治理卡片，范围严格限定到 db/clickhouse 子树 | +40 |

**总预估：~1170 行新增 / 0 行删除**

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
    - docs/superpowers/plans/2026-07-04-openstock-p0-14-*.md
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
| R3 | `Decimal → f64` 极端值精度损失 | A 股数值范围内（< 10^15）无损；helper 注释明确范围；overflow 回退 0.0 + warn（U2 覆盖） |
| R4 | `MinutePeriod` / `AdjustType` 新增变体时 Enum8 编号漂移 | 在 `bar_to_row` 的转换处添加编译期 exhaustive match；变体顺序在 DDL 注释中固化 |
| R5 | `DateTime<FixedOffset>` 与 ClickHouse `DateTime64(3, 'Asia/Shanghai')` 时区不一致 | INV-2A + L1 反查；`FixedOffset::east_opt(8*3600)` 与 ClickHouse 时区名 'Asia/Shanghai' 在 wall-clock 上等价（无 DST，中国不实行夏令时） |
| R6 | `MinuteSink` trait 泄漏到公共 API | INV-4 编译期保证；`pub(crate)` 在 lib.rs `pub use` 中不出现；`cargo doc --document-private-items` 检查 |

---

## 8. 非目标 (Non-Goals)

- ❌ **CLI 子命令**（`persist minute-klines`、`persist minute-shares`）→ P0.15
- ❌ **scheduler / cron 触发器**（每日定时拉取并落盘）→ P0.15
- ❌ **ReplacingMergeTree / 显式去重**：MergeTree + 上游 `(date, code, period, adjust, timestamp)` 自然唯一已足够；如未来出现重复，再评估 ReplacingMergeTree
- ❌ **Parquet / DuckDB / 其他 sink**：本切片只 ClickHouse
- ❌ **遗留 `minute_klines_*` 表迁移**：旧表存在与否不影响本切片；P0.14 只新增 `minute_klines` / `minute_shares` 两张表
- ❌ **流控 / 背压**：`Vec<T>` per batch 是天然单位；ClickHouse 写入速度远高于 OpenStock 拉取速度，无背压问题
- ❌ **批量回填（multi-month backfill）工具**：本切片只提供 API；批量回填由 P0.15 CLI 子命令承担
- ❌ **数据质量监控（dq checks、anomaly detection）**：交给消费侧（P0.16+）做
- ❌ **跨 provider 对账**（tdx-api vs OpenStock）：完全 out of scope
- ❌ **修改 P0.13 系列公共 API**：stream 接口已冻结，本切片只消费

---

## 9. 决策记录 (Decisions)

### D1：表引擎 = MergeTree（非 ReplacingMergeTree）
**选择**：MergeTree
**理由**：与 `kline_data` 现有规范一致；上游流按 `(date, code, period, adjust, timestamp)` 自然唯一，不需要 CH 层去重；ReplacingMergeTree 的 merge 是异步的，查询时仍需 `FINAL` 或 `argMax`，复杂度收益不匹配。
**Rejected**：ReplacingMergeTree、AggregatingMergeTree。

### D2：DateTime64(3, 'Asia/Shanghai') + Rust FixedOffset
**选择**：ClickHouse 列声明 'Asia/Shanghai' 时区；Rust 侧用 `DateTime<FixedOffset>` (+08:00) 传入。
**理由**：列原生绑定 A 股东八区时间，读写无歧义；Rust 侧不引入 `chrono-tz` 依赖；中国不实行夏令时，'Asia/Shanghai' 与 +08:00 在历史数据上始终等价。
**Rejected**：UTC 存储 + 查询时 `toTimezone`（查询方负担重）；`chrono-tz` crate（额外 30+ KB 编译产物，本切片不需要 DST 处理）。

### D3：独立新建 `minute.rs`，不与 `kline.rs` 合并
**选择**：`src/db/clickhouse/minute.rs` 独立文件。
**理由**：`kline.rs` 已有日线聚合逻辑（400+ 行），minute 路径是纯直写，关注点不同；CLAUDE.md 文件大小规范要求单文件 < 500 行；拆分后两个文件各自单一职责。
**Rejected**：把 minute 写入合并到 `kline.rs`（违反单一职责）；把 minute 写入分散到 `models.rs` + `schema.rs`（破坏内聚）。

### D4：Sink trait `pub(crate)`，仅用于单元测试 mock
**选择**：`pub(crate) trait MinuteSink<T>` + `pub(crate) struct ClickHouseMinute*Sink`，trait 仅在测试中通过 `with_sink` 形式注入。
**理由**：trait 抽象的价值仅在于 mock 注入；公共 API 只暴露 `stream_minute_*_to_clickhouse` 函数；上游调用方（P0.15 CLI dispatcher）不需要知道 sink 概念。
**Rejected**：完全私有 trait（测试无法注入，需通过 HTTP 层 wiremock，慢且脆）；`pub` trait（暴露不必要的抽象，未来变更成本高）。

### D5：DDL 保留 `ON CLUSTER '{cluster}'`
**选择**：DDL 文本保留 `ON CLUSTER '{cluster}'`，与现有 `kline_data` / `fundamentals` / `gbbq` / `shadow_kline` 完全一致。
**理由**：集群变量在单机环境展开为单节点列表，DDL 仍可执行；保留这一行让 DDL 模板在所有环境下都能直接复用，无需 fallback 分支。
**Rejected**：根据环境变量移除 `ON CLUSTER`（增加分支，与现有规范分歧）。

### D6：Decimal → f64 用 `try_into` + warn 回退，不阻塞流
**选择**：`Decimal::try_into` 失败时返回 0.0 + `tracing::warn!`，batch 继续写入。
**理由**：A 股数值范围内不可能失败（已通过 INV-2B 锁定范围）；防御性回退保证单条坏值不阻塞整 batch；运行时通过 `StreamStats::skipped_records` 反映。
**Rejected**：失败时返回 Err 并终止流（违反「单坏值不阻塞 batch」原则）；不写 warn（运行时静默损失数据无法排查）。

### D7：流消费函数签名为 `async fn` 而非返回 `impl Future`
**选择**：`pub async fn stream_minute_klines_to_clickhouse(...) -> Result<StreamStats, QuantixError>`
**理由**：与 P0.13d 流 API 的消费模式（`while let Some(...) = stream.next().await`）天然契合；`async fn` 比手写 `Pin<Box<dyn Future>>` 更符合 Rust 2024 习惯；错误类型已统一为 `QuantixError::Other`。
**Rejected**：返回 `impl Stream<Item = Result<StreamStats>>`（过度抽象，调用方仍只能 await）；同步函数返回 Future（暴露细节）。

---

## 10. 与 P0.13 系列的衔接

| 来源 | P0.14 衔接点 |
|---|---|
| P0.13a `MinutePeriod` enum | 在 D1 Enum8 列定义、`bar_to_row` 转换处使用 |
| P0.13b-1 `MinuteBar` struct | `bar_to_row` 输入 |
| P0.13b-2 `MinuteShare` struct | `share_to_row` 输入 |
| P0.13c `DateOrRange::Range` | 流消费函数内部构造并传给 stream API |
| P0.13d `fetch_minute_*_stream` | 流消费函数的 stream 源 |
| P0.13d D4 首错即止 | INV-3A/3B/3C 直接继承 |
| P0.13d INV-4A 公共 API 不变 | 本切片不修改 `src/sources/**`，零影响 |

**P0.14 → P0.15 衔接**：

- P0.15 CLI dispatcher 直接调用 `stream_minute_klines_to_clickhouse` / `stream_minute_shares_to_clickhouse`
- P0.15 scheduler 周期触发同样调用这两个函数
- 公共 API 在 P0.14 冻结，P0.15 不再调整

---

## 11. 实施顺序（给 writing-plans 的提示）

建议任务拆分（每个任务一个独立可测试交付）：

1. **T1：DDL + 模型 + enum**：`schema.rs` 新增两个 `create_*_table()` 方法并在 `init_database()` 中追加调用（§2.6）；`models.rs` 新增 `MinuteKlineCH` / `MinuteShareCH` + `PeriodCH` / `AdjustCH`（含 `#[repr(i8)]` 编号锁定，§2.5）；schema 单元测试（验证 INV-1A/2E）
2. **T2：转换 helper + enum 映射**：`minute.rs` 新建；`decimal_to_f64` / `naive_to_shanghai` / `period_to_ch` / `adjust_to_ch` / `bar_to_row` / `share_to_row`；U1-U5 + U9 单元测试（U9 覆盖 INV-2C/2E enum 映射）
3. **T3：Sink trait + 批量插入**：`MinuteSink<T>` trait + 两个 ClickHouse sink 实现；可注入 sink 的 `insert_minute_*_batch` 函数
4. **T4：流消费**：`stream_minute_*_to_clickhouse`；U6-U8 单元测试（含 mock 注入）
5. **T5：实时测试**：L1/L2 文件，`#[ignore]` + 环境变量门控
6. **T6：OpenSpec + 治理**：`openspec/changes/.../` + `P0.14.yaml` 卡片

每个任务都遵循 TDD（先写测试 → 实现 → 验证 → 提交）。

---

## 12. 总结

P0.14 是一个边界清晰、风险可控的「管道」切片：

- **上游**：复用 P0.13d 已冻结的流式 API，零修改
- **本切片**：实现「流 → batch → ClickHouse」的最短路径，独立 `minute.rs`，独立的 MergeTree 表
- **下游**：为 P0.15 CLI/scheduler 提供干净的公共 API (`stream_minute_*_to_clickhouse`)

通过 5 项决策（MergeTree / 时区 / 独立文件 / pub(crate) sink / DDL 一致性）锁定设计，通过 10 项不变量 + 8 个单元测试 + 2 个实时测试覆盖关键路径，通过严格的 forbidden_paths 保证 P0.13 冻结面和 P0.15 延后边界。
