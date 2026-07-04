# Design: openstock-data-consumption-p0-14

Full design rationale: `docs/superpowers/specs/2026-07-04-openstock-p0-14-clickhouse-minute-persistence-design.md`. This file is a quick-reference summary; the spec is authoritative.

## Context

P0.13d 流式 API 已冻结。本切片只消费该 API，落盘到 ClickHouse 两张新表。
表结构、类型映射、转换 helper、Sink trait 全部对齐现有 `kline_data` /
`KlineDataCH` / `kline.rs` 约定，零新 Convention。

## Key Decisions

### D1 — MergeTree（非 ReplacingMergeTree）

上游流按 `(date, code, period, adjust, timestamp)` 自然唯一，无需去重。
`ReplacingMergeTree` 引入 `version` 列与 merge 异步语义，对当前单 writer
场景无收益。

### D2 — DateTime + String period/adjust

100% 对齐 `KlineDataCH`。OpenStock `MinuteBar` 缺 `period` 字段，需要从
`fetch_minute_klines_stream` 输入参数 thread 到 `bar_to_row`。

**Rejected**: `DateTime64(3, 'Asia/Shanghai')` + `Enum8`（与 `kline_data`
分歧，破坏统一查询路径）。

### D3 — 独立 `minute.rs` 文件

与 `kline.rs` 关注点不同（流式 sink vs 批量查询）；CLAUDE.md 单文件 < 500 行。

### D4 — `pub(crate) trait MinuteSink`

仅测试注入；公共函数 `<S: MinuteSink<...>>` 通过 `pub(crate)` trait 约束
事实上为内部 API（INV-4D）。外部 crate 无法构造满足 trait 的类型。

### D5 — DDL 保留 `ON CLUSTER '{cluster}'`

与现有 5 张表一致；`.replace("'{cluster}'", "single_cluster")` 在运行时展开。

### D6 — `to_f64().unwrap_or(0.0)` 静默回退

与 `kline.rs:213-219` 一致；不写 warn。A 股数值范围内（|v| < 10^15）
Decimal → f64 无损，回退理论上不可达。

## Risks

| ID | Risk | Mitigation |
|---|---|---|
| **R1** | `async_insert` 需要 CH ≥ 22.x | NAS 上 ≥ 23.x（已验证） |
| **R2** | MergeTree 多 writer too-many-parts | 本切片只支持单 writer；并发在 P0.15 后才会出现 |
| **R3** | Decimal→f64 极端精度 | A 股 < 10^15 无损；exhaustive unit test U2 |
| **R4** | `MinutePeriod` / `AdjustType` 新增变体 | exhaustive match 编译期强制 |
| **R5** | NaiveDateTime→DateTime<Utc> 时区语义 | 与 `kline_data` 一致；本切片不解决全局时区问题 |
| **R6** | Sink trait 泄漏 | INV-4 编译期保证（trait + sinks 均 `pub(crate)`） |

## Alternatives Considered

ReplacingMergeTree（去重）、DateTime64+Enum8（强类型）、`Pin<Box<dyn Stream>>`
注入（mockable stream source）、把 sinks 改为 `pub`（破坏 INV-4）—— 全部
rejected，理由见 spec §6。

## Invariants

- INV-1A/1B 表存在性 + MergeTree 引擎（`init_database()` 注册 + DDL）
- INV-2A/2B/2C/2D 类型映射（timestamp/volume/period+adjust/Option）
- INV-3A/3B/3C 流语义继承（首错即止，错误上抛不吞）
- INV-4A/4B/4C/4D Sink trait 不外溢（编译期保证）
- INV-5A/5B DDL 集群一致性（`ON CLUSTER` + `single_cluster` replace）

## Migration Path

无现有数据迁移。两张新表通过 `init_database()` 自动创建。下游 P0.15
直接调用公共 API `stream_minute_{klines,shares}_to_clickhouse`。
