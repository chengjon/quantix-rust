# ClickHouse Market Fundamentals Debugging

本文档记录 2026-04-28 这轮 `market_fundamentals_daily` / `data import-fundamentals` 修复的排障过程、根因和可复用操作手册。

## 适用范围

- `quantix data import-fundamentals`
- `quantix market strength`
- `quantix market strength-stocks`
- ClickHouse 单机部署，尤其是 NAS 上的远端实例

## 本次环境前提

- 主要验证目标不是本地 Docker，而是远端单机 ClickHouse。
- 本轮真实烟测使用了 NAS ClickHouse HTTP 端点：`http://192.168.123.104:8123`
- 该部署是单机模式，默认不需要 `CLICKHOUSE_CLUSTER`
- 正式库名为 `quantix`

建议把凭据放在环境变量里，不要写入仓库文档或脚本默认值。

## 现场症状

### 1. 导入到一个新库时直接失败

表现：

```text
UNKNOWN_DATABASE
```

根因：

- `init_database()` 试图使用“已经绑定到目标数据库”的 ClickHouse client 执行 `CREATE DATABASE IF NOT EXISTS <db>`
- 当目标库本身还不存在时，连接上下文已经先失败了

### 2. 数据库创建成功，但建表失败

表现：

```text
CLUSTER_DOESNT_EXIST
Requested cluster 'single_cluster' not found
```

根因：

- DDL 里写了 `ON CLUSTER '{cluster}'`
- 旧逻辑会无条件替换成 `single_cluster`
- NAS 单机 ClickHouse 并没有这个 cluster 定义

### 3. 导入命令显示成功，但表内字段错位

表现：

- `snapshot_date` 变成异常日期
- `profit_source`、`updated_at` 出现二进制串位
- 回查结果明显不是原始输入

根因：

- `clickhouse-rs` 的 RowBinary 写入对 `Date` / `DateTime` 有严格编码要求
- 旧实现直接把 `chrono::NaiveDate` 和格式化后的时间字符串写入插入模型
- 对 `market_fundamentals_daily` 来说，这会导致字段编码错位

### 4. `strength` 无法输出真实 TopN

表现：

- 正式库 `quantix` 有 `sector_daily`
- 但没有 `market_fundamentals_daily`
- `market strength` / `market strength-stocks` 的总市值、净利润 TopN 缺少本地基础面支撑

## 根因到修复点映射

### 修复 1：数据库创建不再依赖目标库已存在

代码位置：

- [schema.rs](/opt/claude/quantix-rust/src/db/clickhouse/schema.rs)

处理方式：

- `init_database()` 改成通过独立 HTTP 管理查询执行 `CREATE DATABASE IF NOT EXISTS`
- 这一步不绑定目标数据库上下文

结果：

- 新库首次导入不再被 `UNKNOWN_DATABASE` 拦住

### 修复 2：单机部署默认去掉 `ON CLUSTER`

代码位置：

- [schema.rs](/opt/claude/quantix-rust/src/db/clickhouse/schema.rs)

处理方式：

- 新增 `render_cluster_ddl(sql)`
- 默认把 ` ON CLUSTER '{cluster}'` 从 DDL 中去掉
- 只有显式设置 `CLICKHOUSE_CLUSTER` 时，才渲染真实 cluster 名称

结果：

- 单机 NAS ClickHouse 可直接建表
- 真正的集群部署仍可通过环境变量启用

### 修复 3：为 RowBinary 写入拆分专用插入模型

代码位置：

- [models.rs](/opt/claude/quantix-rust/src/db/clickhouse/models.rs)
- [fundamentals.rs](/opt/claude/quantix-rust/src/db/clickhouse/fundamentals.rs)
- [etl.rs](/opt/claude/quantix-rust/src/sync/etl.rs)

处理方式：

- 保留 `MarketFundamentalSnapshotCH` 作为查询模型
- 新增 `MarketFundamentalSnapshotInsertCH` 作为写入模型
- `snapshot_date` 改为 `u16`，表示自 `1970-01-01` 起的天数
- `updated_at` 改为 `u32`，表示 Unix 秒
- 在 ETL 层显式调用：
  - `encode_clickhouse_date(...)`
  - `encode_clickhouse_datetime(...)`

结果：

- RowBinary 插入后字段不再串位
- 回查结果与原始 JSON 输入一致

### 修复 4：导入路径自动补齐数据库和表

代码位置：

- [etl.rs](/opt/claude/quantix-rust/src/sync/etl.rs)

处理方式：

- `sync_market_fundamentals()` 先执行 `self.clickhouse_client.init_database().await?`

结果：

- `quantix data import-fundamentals` 不再要求操作者手工先建库建表

### 修复 5：CLI 验收脚本支持可选 fundamentals 导入

代码位置：

- [check_market_cli_prereqs.sh](/opt/claude/quantix-rust/scripts/dev/check_market_cli_prereqs.sh)
- [run_market_cli_formal_sequence.sh](/opt/claude/quantix-rust/scripts/dev/run_market_cli_formal_sequence.sh)

处理方式：

- 新增可选环境变量 `MARKET_FUNDAMENTALS_INPUT`
- formal sequence 在存在该变量时自动执行：

```bash
quantix data import-fundamentals --input "$MARKET_FUNDAMENTALS_INPUT"
```

结果：

- `strength` 的真实 TopN 可以通过本地 JSON 快速补齐
- 验收链路不再只依赖不稳定的远端单股基本面抓取

## 推荐排障顺序

### 第一步：确认正式库是否缺表

```bash
curl -sS -u "$CLICKHOUSE_USER:$CLICKHOUSE_PASSWORD" \
  "http://$CLICKHOUSE_HOST:8123/?database=quantix&query=SHOW%20TABLES%20LIKE%20'market_fundamentals_daily'%20FORMAT%20TabSeparatedRaw"
```

如果没有输出，说明正式库里还没有这张表。

### 第二步：确认板块数据是否已经存在

```bash
curl -sS -u "$CLICKHOUSE_USER:$CLICKHOUSE_PASSWORD" \
  "http://$CLICKHOUSE_HOST:8123/?database=quantix&query=SELECT%20max(trade_date)%20FROM%20sector_daily%20WHERE%20sector_type%3D'industry'%20FORMAT%20TabSeparatedRaw"
```

如果这里有日期，说明强弱板块基础排名已经有了，缺的是 fundamentals 本地表。

### 第三步：先用临时库做导入烟测

```bash
CLICKHOUSE_URL=http://192.168.123.104:8123 \
CLICKHOUSE_DB=quantix_smoke_fundamentals_20260428 \
CLICKHOUSE_USER=default \
CLICKHOUSE_PASSWORD='***' \
cargo run --bin quantix -- \
  data import-fundamentals --input /abs/path/market_fundamentals.json
```

### 第四步：回查导入结果是否真实落盘

```bash
curl -sS -u "$CLICKHOUSE_USER:$CLICKHOUSE_PASSWORD" \
  "http://$CLICKHOUSE_HOST:8123/?database=quantix_smoke_fundamentals_20260428&query=SELECT%20code%2C%20toString(snapshot_date)%2C%20market_cap%2C%20latest_report_profit%2C%20profit_source%2C%20pe_dynamic%2C%20toString(updated_at)%20FROM%20market_fundamentals_daily%20FORMAT%20JSONEachRow"
```

预期：

- `snapshot_date` 与 JSON 输入一致
- `market_cap` / `latest_report_profit` 数值正确
- `profit_source` 不乱码

### 第五步：再导入正式库

```bash
CLICKHOUSE_URL=http://192.168.123.104:8123 \
CLICKHOUSE_DB=quantix \
CLICKHOUSE_USER=default \
CLICKHOUSE_PASSWORD='***' \
cargo run --bin quantix -- \
  data import-fundamentals --input /abs/path/market_fundamentals.json
```

### 第六步：运行真实业务命令

```bash
quantix market strength --date 2026-03-14 --strong-top 3 --weak-top 3 --stock-top 10

quantix market strength-stocks --date 2026-03-14 --strong-top 3 --metric market-cap --top 10

quantix market strength-stocks --date 2026-03-14 --strong-top 3 --sector 银行 --metric profit --top 10
```

## 本次真实 NAS 验证结论

真实烟测已经验证通过，目标库为：

- `quantix_smoke_fundamentals_20260428_rowfix`

验证结果：

- 能自动创建数据库
- 能自动创建 `market_fundamentals_daily`
- 能正确写入 1 条 `MarketFundamentalSyncRecord`
- 回查结果正确：

```json
{"code":"000021","snapshot_date":"2026-03-14","market_cap":1200.5,"latest_report_profit":18.6,"profit_source":"smoke","pe_dynamic":22.4}
```

## 自动化覆盖

本轮相关测试包括：

- `execute_admin_sql_surfaces_http_errors_without_target_database_context`
- `render_cluster_ddl_omits_on_cluster_without_env`
- `render_cluster_ddl_uses_cluster_env_when_present`
- `test_encode_clickhouse_date_uses_days_since_unix_epoch`
- `test_encode_clickhouse_datetime_uses_unix_seconds`
- `run_market_cli_formal_sequence_script_test`
- `check_market_cli_prereqs_script_test`

## 常见误区

### 误区 1：单机 ClickHouse 也应该硬编码 `single_cluster`

错误。

单机部署默认不需要 `ON CLUSTER`。只有真实集群部署才应该设置 `CLICKHOUSE_CLUSTER`。

### 误区 2：导入成功日志就等于字段写对了

错误。

RowBinary 类型错位时，插入命令可能仍然返回成功，但表内字段已损坏。必须做实际回查。

### 误区 3：`sector_daily` 有数据就代表 `strength` 的 TopN 一定可用

错误。

`strength` 的板块强弱可以依赖 `sector_daily`，但强势板块内个股按总市值 / 净利润排序仍依赖 `market_fundamentals_daily`。

### 误区 4：为了调试再拉一个本地 ClickHouse Docker

本条线不推荐。

如果已有稳定的 NAS 单机 ClickHouse，优先直接对远端临时库做烟测，避免环境漂移。

## 后续建议

- 把正式 `quantix` 库的 fundamentals 导入纳入日常数据准备流程
- `market strength` 正式验收前，先检查 `sector_daily` 最新日期
- `market strength` 正式验收前，先检查 `market_fundamentals_daily` 是否存在
- `market strength` 正式验收前，先检查 `market_fundamentals_daily` 记录数是否大于 0
- 如果后续接入真正的 ClickHouse 集群，再通过 `CLICKHOUSE_CLUSTER` 开启集群 DDL，不要回退到硬编码 cluster 名
