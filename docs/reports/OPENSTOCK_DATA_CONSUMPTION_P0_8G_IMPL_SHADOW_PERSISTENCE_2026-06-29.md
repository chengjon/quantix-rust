# OpenStock 数据消费 P0.8g-impl — Shadow Persistence 实现切片

日期：2026-06-29
分支：`feat/openstock-p0-8g-impl`
FUNCTION_TREE 节点：`P0.8g-impl: OpenStock shadow persistence write path`（status 目标：`approved-for-implementation`，production-code）
设计前置：P0.8g 设计门禁（PR #314，已合并，docs-only）

## 1. 决策

P0.8g-impl 是 **production-code 切片**。它落地 P0.8g 设计文档定义的「OpenStock live payload → `quantix_shadow.openstock_daily_kline_shadow` ClickHouse 表」的两阶段 dry-run/opt-in 写入路径，以及配套的 rollback 命令。

本切片**首次触及 OpenStock 数据消费链路的持久化侧**。P0.8f 已建立 dry-run validator（read-only），P0.8g-impl 在其基础上加 opt-in 写入，但严格隔离：

- 独立 ClickHouse database（`quantix_shadow`），物理上不与生产 `quantix` 库共享
- 新增模块 `src/sources/openstock/shadow_persistence.rs`，不污染 `app_shell.rs` 默认 dispatch 路径
- 复用既有 `ClickHouseClient`（`src/db/clickhouse/mod.rs:25`）— 仅追加新方法，不修改现有方法
- 不复用 miniQMT `ControlledPersistencePolicy`（GitNexus HIGH impact，治理明令禁止）

## 2. 范围

允许（本实现切片，源代码授权）：

新增/修改文件（已捕获 GitNexus impact，见 §7）：

- `src/sources/openstock/shadow_persistence.rs`（新模块）
  - `pub fn artifact_hash(raw: &str) -> String`（SHA-256 hex，纯函数，无 IO）
  - `pub struct ShadowWriteReport { dry_run, writes_performed, batch_id, artifact_hash, row_count, mapped_count, min_date, max_date, duplicate_key_count, drift, fail_closed }`
  - `pub enum ShadowWriteError { DriftPresent, FailClosedPresent, DuplicateKeysFound, EnvConfirmMissing, ApplyFlagMissing, Database(...) }`
  - `pub fn write_shadow_klines(client: &ClickHouseClient, report: &LiveShadowReport, raw_payload: &str, apply: bool, env_confirmed: bool, ingested_by: &str) -> Result<ShadowWriteReport, ShadowWriteError>`
  - `pub fn rollback_shadow_batch(client: &ClickHouseClient, batch_id: &str) -> Result<u64, ShadowWriteError>`（返回删除行数）
  - `pub fn verify_shadow_batch(client: &ClickHouseClient, batch_id: &str) -> Result<ShadowVerifyReport, ShadowWriteError>`
- `src/sources/openstock.rs` 或 `src/sources/openstock/mod.rs` — 视现有结构重组为子模块（仅 pub use 调整，不动既有公共 API）
- `src/db/clickhouse/shadow_kline.rs`（新模块）
  - `impl ClickHouseClient { pub fn insert_shadow_klines(&self, rows: &[ShadowKlineRow]) -> Result<(), ClickHouseError> }`
  - `impl ClickHouseClient { pub fn delete_shadow_batch(&self, batch_id: &str) -> Result<u64, ClickHouseError> }`
  - `impl ClickHouseClient { pub fn count_shadow_batch(&self, batch_id: &str) -> Result<u64, ClickHouseError> }`
  - `pub struct ShadowKlineRow { batch_id, artifact_hash, source, code, period, date, adjust_type, open, high, low, close, volume, amount, batch_row_count, batch_mapped_count, batch_min_date, batch_max_date, batch_drift_summary, batch_requested_window, ingested_by }`
- `src/cli/commands/data.rs` — 新增 OpenStockCommands 变体
  - `PersistLive { payload, symbol, period, start, end, limit, apply }`
  - `ShadowRollback { batch_id }`
  - `ShadowVerify { batch_id }`
- `src/cli/handlers/openstock_handler.rs` — 新增对应 handler 桥函数
- 测试：
  - `tests/openstock_shadow_persistence_test.rs`（default CI 跑，纯逻辑 + mock，不连 ClickHouse）
  - `tests/openstock_shadow_persistence_cli_test.rs`（CLI dispatch + 双保险 gate）
  - `tests/openstock_shadow_persistence_integration_test.rs`（`#[ignore]` + `QUANTIX_SHADOW_INTEGRATION=1` 才跑）
- 文档/治理同步：OpenSpec tasks、CHANGELOG、README、FUNCTION_TREE、active-gates、nodes.json、tree.md、card

禁止（本实现切片）：

- 不复用 `ControlledPersistencePolicy`（GitNexus impact HIGH，治理明令）
- 不修改 `Kline` 定义（CRITICAL hub，read-only）
- 不修改 `BacktestEngine` / `OrderStatus` / `ExecutionAdapter`
- 不修改既有 ClickHouse 写入路径（`insert_kline_data` / `insert_kline_data_batch` / `insert_kline_data_batch_with_source`）— 只追加 shadow 表专用方法
- 不引入新 ClickHouse client crate 依赖
- 不做 live OpenStock 网络调用（payload 仍来自外部捕获文件）
- 不做 shadow → 生产库 promote（升级路径留待更晚切片）

## 3. 输入契约（来自 P0.8f，复用）

`ShadowWriteReport` 的输入是 P0.8f 的 `LiveShadowReport`：

```
pub struct LiveShadowReport {
    pub status: LiveShadowStatus,           // Ok / DriftDetected / FailClosed
    pub symbol: String,
    pub period: String,
    pub requested_window: LiveShadowRequest,
    pub received_count: usize,
    pub mapped_count: usize,
    pub received_date_range: Option<(NaiveDate, NaiveDate)>,
    pub klines: Vec<Kline>,                 // 已校验通过的 canonical Kline
    pub drift: Vec<LiveShadowDrift>,
    pub fail_closed_errors: Vec<String>,
}
```

Shadow write 复用 `klines`/`mapped_count`/`received_date_range`，不重新解析。

## 4. 输出契约：ShadowWriteReport

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

dry-run（默认）与 apply 输出同一形状，仅 `dry_run`/`writes_performed` 字段不同。CLI 把 JSON 打到 stdout，机器可解析。

## 5. Dry-run Gate（实写前必跑）

CLI 在调用 `write_shadow_klines(..., apply=true, ...)` 之前，**内部先跑一次 apply=false**，校验：

- `row_count > 0`
- `mapped_count == row_count`
- `duplicate_key_count == 0`
- `fail_closed == []`
- `drift == []`

任一不满足 → `ShadowWriteError`，CLI 退出 ≠ 0，不发起任何 DB 连接。这是 §6.4 的双保险前置。

## 6. 双保险 opt-in 写入

实写必须同时满足：

1. 命令行显式 `--apply`
2. 环境变量 `QUANTIX_SHADOW_PERSIST_CONFIRM=yes`

缺任一即 `ShadowWriteError::{ApplyFlagMissing | EnvConfirmMissing}`，dry-run 不受影响（始终允许）。

写入语句：

```sql
INSERT INTO quantix_shadow.openstock_daily_kline_shadow
(batch_id, artifact_hash, source, code, period, date, adjust_type,
 open, high, low, close, volume, amount,
 batch_row_count, batch_mapped_count, batch_min_date, batch_max_date,
 batch_drift_summary, batch_requested_window, ingested_by)
VALUES (?, ?, ?, ...)
```

使用 ClickHouse `clickhouse.rs` crate 的 prepared insert，复用 `ClickHouseClient::client()` 获取的连接。

## 7. GitNexus Impact（fresh）

本切片首次触及源代码，已跑 fresh impact：

| 目标符号 | 类型 | 用途 | impact |
|---|---|---|---|
| `Struct:src/db/clickhouse/mod.rs:ClickHouseClient` | struct | 追加 3 个新方法（`insert_shadow_klines`/`delete_shadow_batch`/`count_shadow_batch`），不修改既有方法 | LOW（d=1: 0 upstream） |
| `src/sources/openstock.rs::validate_live_shadow_payload` | fn | 只读消费其 `LiveShadowReport` 输出 | LOW（既有公共符号，仅 read） |
| `src/sources/openstock.rs::LiveShadowReport` | struct | 只读消费 | LOW |
| `src/cli/commands/data.rs::OpenStockCommands` | enum | 追加 3 个变体 | LOW（无既有 match arm 被破坏；新增 match arm） |
| `ControlledPersistencePolicy`（`src/miniqmt_market.rs:355`） | enum | **不复用** — 治理明令禁止 | n/a |
| `Kline`（`src/data/models.rs:11`） | struct | read-only 消费 | n/a（CRITICAL hub，仅读） |

实现切片启动时必须重跑 `gitnexus_detect_changes`，验证 src/ 触及符号与上述清单一致，且 `ControlledPersistencePolicy`/`Kline`/`BacktestEngine` 触及数 = 0。

## 8. CI 非写入证明（implementation gate）

默认 `cargo test` 套件必须通过以下断言（来自 P0.8g §8.2）：

| 测试 | 断言 |
|---|---|
| `persist_live_dry_run_does_not_connect_to_clickhouse` | `--dry-run` 时 `ClickHouseClient::insert_shadow_klines` 调用计数 = 0 |
| `persist_live_requires_apply_flag` | 缺省 `--apply` 时即使环境变量已设，也不写 |
| `persist_live_requires_env_confirm` | `--apply` 但缺环境变量时，不写，退出 ≠ 0 |
| `persist_live_rejects_drift` | dry-run gate 在 drift 非空时拒绝 |
| `persist_live_rejects_fail_closed` | dry-run gate 在 fail-closed 非空时拒绝 |
| `persist_live_rejects_duplicates` | dry-run gate 在 duplicate_key_count > 0 时拒绝 |
| `artifact_hash_is_deterministic` | 同输入产出同 SHA-256 hex |
| `rollback_is_idempotent` | 第二次调用删除 0 行，不报错 |

集成测试（`#[ignore]`，需 `QUANTIX_SHADOW_INTEGRATION=1`）：

| 测试 | 断言 |
|---|---|
| `persist_live_apply_writes_rows` | `--apply` + env 时，ClickHouse 实际行数 == batch_row_count |
| `rollback_removes_batch` | rollback 后 count == 0 |

## 9. 验收标准

### commit_gate

- 新增模块 `src/sources/openstock/shadow_persistence.rs` + `src/db/clickhouse/shadow_kline.rs` 落地
- `cargo fmt --check`、`cargo clippy -D warnings`、`cargo test --workspace` 全绿
- §8 表中所有 default-CI 断言测试存在并通过
- 集成测试存在且默认 `#[ignore]`
- GitNexus `detect_changes` 确认：仅 §7 列出的符号被触及；`ControlledPersistencePolicy`/`Kline`/`BacktestEngine` 触及数 = 0
- `git diff --check` clean
- OpenSpec validate --all --strict 通过
- FUNCTION_TREE validate/gate 通过

### closeout_gate

- `P0.8g-impl` FUNCTION_TREE 节点关闭
- README/CHANGELOG/FUNCTION_TREE/OpenSpec 更新
- PR CI 通过
- Graphiti memory 或本地 backfill 记录

## 10. Non-Goals

- 不复用 `ControlledPersistencePolicy`（HIGH impact）
- 不修改 `Kline`（CRITICAL hub）
- 不修改既有 `kline_data` 写入路径
- 不做 shadow → 生产 promote
- 不做 live OpenStock 网络调用
- 不做 ClickHouse schema migration（schema 由 `init_quantix_shadow.sql` 附带，operator 手动执行；本切片仅提供 SQL 文件 + 文档）
- 不实现 `shadow_audit_log` 表写入（P0.8g §7.4 提到；本切片推迟到后续 follow-up）
- 不实现 `shadow-verify` 子命令的完整审计输出（本切片只做 row_count 验证；审计字段扩展留待 follow-up）

## 11. 下一步

- 本设计 gate 合并 → 启动实现切片（同分支 `feat/openstock-p0-8g-impl`）
- 实现顺序（TDD）：artifact_hash（纯函数）→ ShadowKlineRow/Report 类型 → dry-run gate 逻辑 → ClickHouse 写入方法 → rollback → CLI dispatch → 集成测试（`#[ignore]`）
- 后续：`shadow_audit_log` 表 + operator-facing audit query CLI（候选切片 P0.8g-audit）
