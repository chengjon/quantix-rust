# OpenStock 数据消费 P0.8g — shadow persistence opt-in 设计门禁

日期：2026-06-29
分支：`docs/openstock-p0-8g-shadow-persistence-design`
FUNCTION_TREE 节点：`P0.8g: OpenStock shadow persistence opt-in design`（status: 待授权）
前置切片：P0.8e（设计 gate）已合并；P0.8f（live shadow validation）PR #313 开启、CI 全绿、合并前

## 1. 决策

P0.8g 是**纯设计 gate**。本切片不批准、不实现任何 ClickHouse 写入路径。

产出物仅为本文档与治理节点 `P0.8g`，回答 P0.8e §"Rollback Requirements Before Any Write Path" 列出的全部 10 项设计要素：

1. shadow 表名与 database namespace
2. 完整 shadow 表 schema
3. batch identity 列
4. source artifact hash 列
5. 写入模式
6. delete-by-batch rollback 命令
7. operator runbook
8. dry-run 预览输出
9. partial write 失败行为
10. CI 证明常规运行不写 ClickHouse

本设计完全消费 P0.8f 已落地的 `LiveShadowReport` 作为输入契约，不引入新的生产 Rust 修改。

## 2. 范围

允许（本切片）：

- 编写本文档（shadow persistence 设计）
- 创建治理节点 `P0.8g` 并推进到 `approved-for-implementation`（设计 gate，等价于 P0.8e）
- 同步 OpenSpec、CHANGELOG、README、FUNCTION_TREE

禁止（本切片与下一实现切片，直至另行授权）：

- 不修改任何生产 Rust 源码
- 不写 ClickHouse
- 不替换生产数据源路由
- 不触达 qmt_live / miniQMT / ExecutionAdapter / OrderStatus
- 不做 live OpenStock 网络调用
- 不恢复 `.unwrap()` 清理
- 不修改或复用 `ControlledPersistencePolicy`（GitNexus impact HIGH，见 §9）

## 3. 输入契约（来自 P0.8f）

P0.8g 的持久化目标，是消费 P0.8f 的 dry-run report 输出。P0.8f 已定义：

```rust
pub struct LiveShadowReport {
    pub dry_run: bool,                                // 恒为 true
    pub source: &'static str,                          // "openstock_live_shadow"
    pub status: LiveShadowStatus,                      // Ok | Drift | FailClosed
    pub record_count: usize,
    pub mapped_count: usize,
    pub symbol: Option<String>,                        // 规范化的数字 code（sh600000 -> 600000）
    pub period: Option<String>,                        // "day" / "daily"
    pub received_date_range: Option<(NaiveDate, NaiveDate)>,
    pub drifts: Vec<LiveShadowDrift>,
    pub fail_closed_errors: Vec<OpenStockKlineParseError>,
}
```

设计原则：**只有 `status = Ok` 且零 drift 的 report 才允许进入 opt-in 持久化路径**。任何 fail-closed 或 drift 都必须先被操作员 triage，持久化路径不应自动覆盖服务端异常。

`LiveShadowReport` 字段映射到 §4 的 shadow schema：

| Report 字段 | Shadow schema 字段 | 用途 |
|---|---|---|
| `source` | `source` | namespace 隔离（`openstock_live_shadow`） |
| `symbol` | `code` | 规范化数字 code，对应 `Kline.code` |
| `period` | `period` | 恒为 `day`（已规范化） |
| `received_date_range` | (派生) `min_date`、`max_date` | batch 元数据 |
| `record_count` / `mapped_count` | `batch_row_count` / `batch_mapped_count` | 行数校验 |
| `drifts` | `batch_drift_summary` | 仅作 batch 元数据；drift 时禁止写 |
| `fail_closed_errors` | 不持久化 | triage-only，drift/fail-closed 时整批拒绝 |

P0.8f 的 `Kline`（CRITICAL hub）保持只读：P0.8g 仅消费已映射的 `Vec<Kline>`，不修改其定义。

## 4. Shadow Schema（回答 §1 第 1–4 项）

### 4.1 Namespace 与表名

- ClickHouse database: `quantix_shadow`（新建独立 database，隔离生产 `quantix` 写路径）
- 表名: `openstock_daily_kline_shadow`
- 命名理由：`_shadow` 后缀让运维 CLI 与监控告警可一眼区分 shadow vs 生产；database 级隔离保证「误 GRANT」不会跨界。

### 4.2 完整 schema

```sql
CREATE TABLE quantix_shadow.openstock_daily_kline_shadow
(
    -- 持久化身份
    batch_id           String         COMMENT 'ULID 或 UUIDv7，每次写入唯一',
    artifact_hash      String         COMMENT 'SHA-256(raw_payload_bytes)',
    source             LowCardinality(String)  COMMENT 'openstock_live_shadow',

    -- 业务键（对应 Kline 语义）
    code               String         COMMENT '规范化数字 code, e.g. 600000',
    period             LowCardinality(String)  COMMENT 'day',
    date               Date,
    adjust_type        LowCardinality(String)  COMMENT 'none | qfq | hfq',

    -- 行情
    open               Decimal(38, 8),
    high               Decimal(38, 8),
    low                Decimal(38, 8),
    close              Decimal(38, 8),
    volume             Int64,
    amount             Nullable(Decimal(38, 8)),

    -- batch 元数据（与 LiveShadowReport 对齐）
    batch_row_count    UInt32,
    batch_mapped_count UInt32,
    batch_min_date     Date,
    batch_max_date     Date,
    batch_drift_summary String        COMMENT 'JSON: [] 或 drift 列表',
    batch_requested_window String     COMMENT 'JSON: {start, end, limit}',

    -- 操作追踪
    ingested_at        DateTime64(3)  DEFAULT now64(3),
    ingested_by        String         COMMENT 'operator 标识 / hostname'
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(date)
ORDER BY (source, code, period, date, adjust_type, batch_id)
SETTINGS index_granularity = 8192;
```

设计要点：

- `ORDER BY` 前缀 `(source, code, period, date, adjust_type)` 等于去重键（见 §5），保证同一逻辑行的多版本写共存，便于对比；`batch_id` 作为尾部，让按批删除走稀疏索引。
- `artifact_hash` 不进 ORDER BY（避免膨胀），但 §5 的去重键包含它，因为不同批次可能基于同一份 captured payload。
- Nullable 仅用于 `amount`（P0.8f 允许缺省）。所有其他字段是 NOT NULL，因 P0.8f 的 fail-closed 已强制拒绝缺字段记录。
- `decimal(38,8)` 与现有 `rust_decimal::Decimal` 精度对齐，避免 silent truncation。
- `batch_*` 列冗余存每个 batch 的元数据，让单行查询即可重建 report，便于 audit。

### 4.3 与 `quantix` 生产库的关系

- 完全隔离：`quantix_shadow` 不被任何生产 SELECT/INSERT 路径引用
- 升级路径：P0.8h 或更晚的切片才会评估"是否把 shadow 表内容 promote 到生产"，本切片不承诺该路径
- 回滚路径：drop database `quantix_shadow` 等于完全撤销 shadow persistence，零生产影响

## 5. Deduplication 与 Batch Identity（回答 §1 第 3、4 项）

### 5.1 行级去重键

```
source + period + code + date + adjust_type
```

复用 P0.8e §"Deduplication Contract" 的同一键。`batch_id` 不进键 — 同一逻辑行可被多批次重写（如操作员用同一份 payload 重跑），shadow 表保留全部版本以便对比。

### 5.2 Batch identity

```
batch_id   = ULID (时间序，26 字符)
artifact_hash = SHA-256(raw_payload_bytes) 十六进制小写
```

两者组合即 P0.8e §"Rollback" 的最小回滚身份 `batch_id + source + artifact_hash`。

- `batch_id` 由 CLI 在写入前生成（不依赖 DB），便于失败重试用同一 ID
- `artifact_hash` 由 P0.8f 已具备的 raw payload bytes 计算（P0.8f 当前未计算哈希，实现切片需补一个 `pub fn artifact_hash(raw: &str) -> String` helper，纯函数，无网络/IO）

### 5.3 重复键检测

实现切片必须：在写入前对同一 `batch_id` 内部按 §5.1 键排序，若发现重复 → **fail closed，不写**，报告 `duplicate_key_count > 0`。

P0.8f 的 `fail_closed_errors` 已经定义了 fail-closed 语义，shadow persistence 复用同一机制而非引入新的错误类型。

## 6. 写入与回滚（回答 §1 第 5、6、9 项）

### 6.1 写入模式：两阶段

**阶段 1 — dry-run preview（默认）**

```
quantix data openstock persist-live \
  --payload <file|-> \
  --symbol 600000 --period day --start 2026-06-22 --end 2026-06-23 \
  --dry-run
```

输出（stdout，机器可解析）：

```json
{
  "dry_run": true,
  "writes_performed": false,
  "target": "quantix_shadow.openstock_daily_kline_shadow",
  "batch_id": "01J0QX...",
  "artifact_hash": "a3f5...",
  "row_count": 2,
  "mapped_count": 2,
  "min_date": "2026-06-22",
  "max_date": "2026-06-23",
  "duplicate_key_count": 0,
  "drift": [],
  "fail_closed": []
}
```

**阶段 2 — opt-in 实写**

```
quantix data openstock persist-live ... --apply
```

- 必须显式 `--apply`；缺省即 dry-run，与 P0.8f 的 dry-run-only 行为对齐
- 必须设置环境变量 `QUANTIX_SHADOW_PERSIST_CONFIRM=yes`（双保险，防止命令行误触）
- 写入使用 ClickHouse `INSERT INTO quantix_shadow.openstock_daily_kline_shadow VALUES (...)`

### 6.2 失败行为（partial write）

ClickHouse INSERT 在单批内是原子的（除非 split-by-max-rows 触发，本设计批大小 100 行，远低于该阈值）。但仍需处理：

| 失败模式 | 行为 |
|---|---|
| DB 连接失败 | 不写，CLI 退出 ≠ 0，输出 `writes_performed=false, error=...` |
| INSERT 部分失败（罕见） | ClickHouse 不返回 partial 状态；以异常退出，记录 `batch_id` 供操作员用 §6.3 验证 |
| 网络中断 | 同上，由操作员用 §6.3 的 row_count 校验决定是否回滚 |

### 6.3 回滚命令

```
quantix data openstock shadow-rollback --batch-id 01J0QX...
```

执行 `ALTER TABLE ... DELETE WHERE batch_id = ?`（ClickHouse mutation），删除该 batch 全部行。

- 回滚不依赖 `artifact_hash`（CLI 已记录 `batch_id`，单标识足够）
- 回滚是幂等的：第二次调用删除 0 行，不报错
- 回滚操作记录到 `quantix_shadow.shadow_audit_log`（见 §7）

### 6.4 双保险：dry-run gate

实写前，CLI 内部必须先跑一次完整 dry-run preview 并校验：

- `row_count > 0`
- `mapped_count == row_count`
- `duplicate_key_count == 0`
- `fail_closed == []`
- `drift == []`（任何 drift 都拒绝写）

任一不满足，CLI 立即退出 ≠ 0，不发起任何 DB 连接。

## 7. Operator Runbook（回答 §1 第 7 项）

### 7.1 前置条件

- 操作员已通过外部手段捕获 OpenStock `/data/bars` 响应到本地文件（payload file）
- ClickHouse `quantix_shadow` database 已存在（init 脚本见 §10）
- 环境变量 `QUANTIX_SHADOW_PERSIST_CONFIRM=yes` 已设置

### 7.2 正常流程

```bash
# 1. dry-run preview
quantix data openstock persist-live \
  --payload captured_600000_2026-06-22.json \
  --symbol 600000 --period day --start 2026-06-22 --end 2026-06-23 \
  --dry-run

# 2. 校验输出（操作员人工核对 row_count、date range、drift 为空）

# 3. opt-in 实写
QUANTIX_SHADOW_PERSIST_CONFIRM=yes quantix data openstock persist-live \
  --payload captured_600000_2026-06-22.json \
  --symbol 600000 --period day --start 2026-06-22 --end 2026-06-23 \
  --apply

# 4. 验证
quantix data openstock shadow-verify --batch-id <上一步返回的 batch_id>
```

### 7.3 异常流程

- 漂移/drift：不写。操作员 triage 服务端问题（已知 `/data/bars` 不裁剪 start/end/limit），重新捕获或调整请求窗口
- fail-closed：不写。操作员根据 `fail_closed_errors` 修复 payload（通常是捕获损坏或字段缺失）
- partial write 怀疑：用 `shadow-verify --batch-id` 检查 ClickHouse 实际行数是否等于 `batch_row_count`，不等则 §6.3 回滚

### 7.4 审计

`quantix_shadow.shadow_audit_log` 表（schema 见 §10）记录每次 dry-run / apply / rollback 操作。操作员可按时间或 batch_id 查询。

## 8. CI 证明（回答 §1 第 10 项）

### 8.1 默认 CI 行为

- `cargo test` 默认套件**不**连接 ClickHouse，**不**触发 shadow persistence 代码路径
- 所有 shadow persistence 单元测试用 in-memory mock，集成测试用 `#[ignore]` 标记并要求 `QUANTIX_SHADOW_INTEGRATION=1` 环境变量

### 8.2 CI 必备断言

实现切片必须包含以下测试，且在默认 CI 中运行：

| 测试 | 断言 |
|---|---|
| `persist_live_dry_run_does_not_connect_to_clickhouse` | 调用 `--dry-run` 时 `ClickHouseClient::insert` 调用计数 = 0 |
| `persist_live_requires_apply_flag` | 缺省 `--apply` 时即使环境变量已设，也不写 |
| `persist_live_requires_env_confirm` | `--apply` 但缺环境变量时，不写，退出 ≠ 0 |
| `persist_live_rejects_drift` | dry-run gate 在 drift 非空时拒绝，不发起 DB 连接 |
| `persist_live_rejects_fail_closed` | 同上，fail-closed 非空时拒绝 |
| `persist_live_rejects_duplicates` | 同上，duplicate_key_count > 0 时拒绝 |

### 8.3 静态保证

- shadow persistence 代码隔离在 `src/sources/openstock/shadow_persistence.rs`（或子模块），不进 `src/cli/handlers/app_shell.rs` 默认 dispatch 路径
- `Cargo.toml` 不引入新 ClickHouse 客户端依赖；复用现有 `clickhouse.rs` crate
- `.governance/active-gates.md` 显示 `P0.8g` 为 `approved-for-implementation`（仅授权设计 gate 本身；实现切片需独立 card）

## 9. GitNexus Impact Summary（设计 gate 评估）

P0.8g 设计 gate 不修改任何代码。本节列出**实现切片**的目标符号，留作该切片启动时的 fresh impact check 基准：

| 候选目标 | 当前风险（来自 P0.8e） | 预期用途 | P0.8g 决策 |
|---|---|---|---|
| `src/sync/etl.rs::DataSync.write_klines_to_clickhouse` | LOW | 借鉴写入模式 | 仅参考，不在 P0.8g 实现 |
| `src/cli/handlers/import.rs::validate_clickhouse_table_identifier` | LOW | 借鉴标识符校验 | 仅参考 |
| `src/cli/handlers/import.rs::validate_clickhouse_column_identifier` | LOW | 同上 | 仅参考 |
| `src/sources/openstock.rs::validate_live_shadow_payload` | n/a (P0.8f 新增) | 输入源 | 只读消费 |
| `src/sources/openstock.rs::LiveShadowReport` | n/a (P0.8f 新增) | 输入契约 | 只读消费 |
| `src/miniqmt_market.rs::ControlledPersistencePolicy.parse` | HIGH | — | **禁止复用或修改**；OpenStock shadow 不得耦合 miniQMT 策略 |

实现切片启动时必须重跑 `gitnexus_impact` on 全部新增/修改符号，并在该切片的 evidence baseline 中记录 LOW/MEDIUM 边界。HIGH 风险项不允许进入实现。

## 10. Shadow Database 初始化脚本（参考，不在本切片执行）

```sql
CREATE DATABASE IF NOT EXISTS quantix_shadow;

-- 主表（见 §4.2）
CREATE TABLE IF NOT EXISTS quantix_shadow.openstock_daily_kline_shadow (...);

-- 审计表
CREATE TABLE IF NOT EXISTS quantix_shadow.shadow_audit_log
(
    event_at   DateTime64(3) DEFAULT now64(3),
    event      LowCardinality(String),  -- 'dry_run' | 'apply' | 'rollback' | 'verify'
    batch_id   String,
    operator   String,
    row_count  UInt32,
    detail     String                   -- JSON
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(event_at)
ORDER BY (event_at, batch_id);
```

此脚本属实现切片产物，本设计 gate 仅记录 schema，不创建任何 DB 对象。

## 11. Non-Goals（继承 P0.8e/f，并细化）

- 不修改任何生产 Rust 源码（本切片）
- 不实现 ClickHouse 写入（本切片）
- 不替换生产数据源路由
- 不触达 qmt_live / miniQMT / ExecutionAdapter / OrderStatus
- 不修改或复用 `ControlledPersistencePolicy`（GitNexus HIGH）
- 不修改 `Kline` 定义（CRITICAL hub，只读）
- 不做 live OpenStock 网络调用
- 不恢复 `.unwrap()` 清理
- 不承诺 shadow → 生产 promotion 路径（留待 P0.8h 或更晚）

## 12. 验收标准（本设计 gate）

P0.8g 设计 gate 完成条件：

- OpenSpec 任务 `5g.1`–`5g.x` 标记完成（见 tasks.md）
- README / CHANGELOG / FUNCTION_TREE 引用本设计 gate
- GitNexus `detect_changes` 确认仅 docs/governance 范围（无源码符号被触及）
- PR CI 通过
- 治理节点 `P0.8g` 状态推进到 `approved-for-implementation`（设计 gate 等价于 P0.8e 的最终态）
- Graphiti ingest：本报告作为本地 backfill 凭证；Graphiti 可用时尝试 episode 写入

## 13. 下一步

- **P0.8g-impl（实现切片）**：本设计 gate 合并后启动；必须先做 fresh GitNexus impact、TDD RED→GREEN、复用 P0.8f 的 `validate_live_shadow_payload`，不允许改变 P0.8f 的 dry-run 默认语义
- **P0.8h**：OpenStock → analysis/backtest 的更宽链路验证，优先 fixture/artifact 驱动
