# OpenStock Data Consumption P0.14 — ClickHouse 分钟级数据持久化

## Why

P0.13d 交付了流式 API 但只返回内存 `Vec`。下游（回测、聚合、可视化）需要分钟数据持久化到 ClickHouse 才能跨会话查询。本切片把 klines + shares 两条流落到 `quantix.minute_klines` / `minute_shares`，为 P0.15 CLI 子命令和 scheduler 触发器提供干净的公共 API。

## What Changes

- **新增**（4 处文件修改 + 0 处删除）：
  - `src/db/clickhouse/models.rs`：`MinuteKlineCH` / `MinuteShareCH` 行类型（与 `KlineDataCH` 类型约定一致）
  - `src/db/clickhouse/schema.rs`：`create_minute_klines_table` / `create_minute_shares_table` 方法 + 在 `init_database()` 中追加调用
  - `src/db/clickhouse/mod.rs`：注册 `minute` 模块 + `pub use` 公共 API
  - `src/db/clickhouse/tests.rs`：U1–U8 单元测试 + L1/L2 实时测试
- **新建**：`src/db/clickhouse/minute.rs`（转换 helper + Sink trait + 流消费）
- **DDL**：两张 `MergeTree()` 表，`DateTime` + `String period/adjust`，完全对齐 `kline_data`

## Impact

**公共 API**：新增 `stream_minute_klines_to_clickhouse` / `stream_minute_shares_to_clickhouse` / `StreamStats` / `MinuteKlineCH` / `MinuteShareCH`，无现有 API 变更。

**下游 enable**：P0.15 CLI 子命令（`persist minute-klines`、`persist minute-shares`）和 scheduler 周期触发可直接调用这两个函数。

**冻结面**：P0.13d stream API 不动；`src/sources/**` / `src/cli/**` / `src/scheduler/**` 不修改。

## Non-Goals

- CLI 子命令、scheduler / cron 触发器（P0.15）
- ReplacingMergeTree / 显式去重（MergeTree + 上游自然唯一）
- Parquet / DuckDB / 其他 sink
- 遗留 `minute_klines_*` 表迁移
- Enum8 列类型 / `DateTime64(3, 'Asia/Shanghai')`（与 `kline_data` 约定分歧）
- 流控 / 背压 / 数据质量监控
