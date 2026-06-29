# P0.8g-impl 实现切片交接文档

日期：2026-06-29
分支：`feat/openstock-p0-8g-impl`（已 rebase 到 master b4566f6，干净）
Worktree：`/opt/claude/quantix-rust/.worktrees/openstock-p0-8g-impl`

## 当前状态

- ✅ PR #318 设计 gate 已合并入 master
- ✅ Worktree 分支已 rebase 到 master，HEAD = `b4566f6`
- ✅ 治理 card `P0.8g-impl.yaml`、设计报告、`approved-for-implementation` 状态已就绪
- 🟡 实现代码尚未开始 — 本 session context 不够，转交下一 session

## 关键实现决策（已在本 session 调研确认）

### 1. LiveShadowReport 不携带 klines — 需要在 shadow 模块重新解析

`src/sources/openstock.rs::validate_live_shadow_payload` 内部解析出 `mapped: Vec<Kline>`，但**返回的 `LiveShadowReport` 丢弃了 klines**（只保留 `mapped_count`、`received_date_range` 等 metadata）。

Shadow 写入需要实际的 `Kline` 数据。两条路径：

**路径 A（推荐，最小冲击）**：在 `LiveShadowReport` 上添加 `pub klines: Vec<Kline>` 字段。

- 已验证：`grep -rn "LiveShadowReport {" tests/ src/cli/` 显示**无任何代码直接构造该 struct**（都是函数返回），加字段不会破坏既有调用方
- 影响：`validate_live_shadow_payload` 在 return 时把 `mapped` clone/move 进 report；既有 `Display` impl 不变（klines 不打印）
- 测试影响：既有 P0.8f 测试断言的是 `mapped_count`、`drifts`、`status` 等字段，加 `klines` 字段不破坏断言；只需更新测试中 `assert_eq!(report.klines.len(), N)` 等可选断言

**路径 B**：新加 `pub fn extract_live_shadow_klines(raw, request) -> Result<Vec<Kline>, ...>`。问题：要么解析两次（重复工作），要么重构 `validate_live_shadow_payload` 内部（更大改动）。

**采用路径 A**：在 `LiveShadowReport` 加 `pub klines: Vec<Kline>` 字段。这算 P0.8g-impl 允许的 `src/sources/openstock.rs` 改动范围内（card allowed_paths 包含该文件）。

### 2. 模块结构

- `src/sources/openstock.rs` — 保留单文件（不转 mod 目录，避免触发更多 import 重写）。新增 `pub klines` 字段 + 把 `validate_live_shadow_payload` 的 `mapped` 移入 report
- `src/sources/openstock/shadow_persistence.rs` — **不创建子目录**，改为 **`src/sources/openstock_shadow.rs`** 新文件（card allowed_paths 包含 `src/sources/openstock.rs` + `src/sources/mod.rs`，需把 `src/sources/openstock_shadow.rs` 加进去；或在 design gate 范围内引用为「shadow_persistence.rs」并在 `src/sources/mod.rs` 加 `pub mod openstock_shadow`）。**实现时选更简单的：`src/sources/openstock_shadow.rs` + 在 mod.rs 加 `pub mod openstock_shadow`，并把 card allowed_paths 同步更新**
- `src/db/clickhouse/shadow_kline.rs` — 新文件，在 `src/db/clickhouse/mod.rs` 加 `mod shadow_kline;` + 必要的 `pub use`

### 3. ClickHouseClient 扩展（append-only）

`src/db/clickhouse/mod.rs` 的 `impl ClickHouseClient` 块在新文件 `shadow_kline.rs` 加 `impl ClickHouseClient { pub fn insert_shadow_klines(...) }` — Rust 允许同一 struct 多个 impl 块，**不需要修改既有 impl 块**。这保证 GitNexus「不修改既有方法」承诺。

### 4. CLI 子命令

`src/cli/commands/data.rs::OpenStockCommands` 追加：
```rust
PersistLive { payload: String, symbol: String, period: String, start: String, end: String, limit: Option<u64>, apply: bool }
ShadowRollback { batch_id: String }
ShadowVerify { batch_id: String }
```
Handler 在 `src/cli/handlers/openstock_handler.rs` 加 3 个 pub fn，dispatch 在 `app_shell.rs` 或既有 match arm 处补 3 个 arm。

**注意**：`src/cli/handlers/app_shell.rs` **不在** P0.8g-impl allowed_paths 里 — 需要把 dispatch arm 加在 `openstock_handler.rs` 内部的一个 pub match 函数里，或更新 card 加 `app_shell.rs` 到 allowed_paths。**推荐前者**：把 dispatch 写成 `openstock_handler::dispatch_openstock_command(cmd) -> Result<()>`，CLI 层调一次。

### 5. 关键既有 API 速查（已读源码确认）

- `ClickHouseClient::new(url, database, user, password)` / `from_settings` / `client()` 返回 `&clickhouse::Client`
- 现有 `insert_kline_data_batch_with_source` 在 `src/db/clickhouse/kline.rs` — 写入 `kline_data` 表，**不要触碰**
- `LiveShadowReport` 字段：`dry_run`, `source`, `status: LiveShadowStatus`, `record_count`, `mapped_count`, `symbol: Option<String>`, `period: Option<String>`, `received_date_range: Option<(NaiveDate, NaiveDate)>`, `drifts: Vec<LiveShadowDrift>`, `fail_closed_errors: Vec<OpenStockKlineParseError>` — 加完 `klines` 后用
- `Kline` 字段（CRITICAL hub，read-only）：`code, date: NaiveDate, open/high/low/close: Decimal, volume: i64, amount: Option<Decimal>, adjust_type: AdjustType`
- `AdjustType` 变体：`None=0, QFQ=1, HFQ=2` — ClickHouse LowCardinality(String) 存 `"none"/"qfq"/"hfq"`

### 6. 设计 gate 已锁定的硬约束

- `--apply` 必须配 `QUANTIX_SHADOW_PERSIST_CONFIRM=yes` 环境变量（双保险）
- dry-run gate 拒绝条件（任一命中即拒，不连 DB）：
  - `row_count == 0`
  - `mapped_count != row_count`
  - `duplicate_key_count > 0`
  - `!fail_closed_errors.is_empty()`
  - `!drifts.is_empty()`
- 8 条 default-CI 断言（无 ClickHouse 连接）+ 2 条 `#[ignore]` 集成测试（需 `QUANTIX_SHADOW_INTEGRATION=1`）
- `db/schema/quantix_shadow_init.sql` 提供给 operator 手动跑（不自动迁移）

### 7. 实现顺序（TDD，纯逻辑优先）

1. `LiveShadowReport` 加 `klines` 字段 + 调整 `validate_live_shadow_payload` 把 `mapped` 移入 — 跑既有 P0.8f 测试确认无回归
2. `src/sources/openstock_shadow.rs`：`artifact_hash(raw) -> String` 纯函数（SHA-256 hex）+ 单元测试（determinism）
3. 同文件：`ShadowKlineRow`, `ShadowWriteReport`, `ShadowWriteError`, `ShadowVerifyReport` 类型 + `build_shadow_rows_from_report(report, raw_payload, batch_id, ingested_by) -> Result<Vec<ShadowKlineRow>, ShadowWriteError>` 纯逻辑（含 dry-run gate 检查 + 重复键检测）
4. 同文件：`write_shadow_klines(client, report, raw_payload, apply, env_confirmed, ingested_by)` — dry-run gate → 双保险 → 调 client.insert_shadow_klines
5. `src/db/clickhouse/shadow_kline.rs`：`impl ClickHouseClient { insert_shadow_klines / delete_shadow_batch / count_shadow_batch }`
6. CLI 子命令 + handler dispatch
7. `db/schema/quantix_shadow_init.sql` + README runbook 段
8. 测试：8 条 default-CI + 2 条 `#[ignore]`
9. gates：`cargo fmt --check`, `cargo clippy --tests -D warnings`, `cargo test --workspace`, `openspec validate --all --strict`, GitNexus detect_changes（验证 §7 符号清单）
10. 更新 README/CHANGELOG/FUNCTION_TREE/active-gates/nodes.json/tree.md
11. PR

## 启动指令（下一 session）

```
cd /opt/claude/quantix-rust/.worktrees/openstock-p0-8g-impl
git fetch origin && git log --oneline -3   # 确认在 master b4566f6 之上
# 阅读本交接文档 + docs/reports/OPENSTOCK_DATA_CONSUMPTION_P0_8G_IMPL_SHADOW_PERSISTENCE_2026-06-29.md + .governance/programs/project-governance/cards/P0.8g-impl.yaml
# 按 §7 实现顺序推进
```

## 不确定项（需在新 session 启动时确认）

- `clickhouse::Client` crate 在 `Cargo.toml` 中确切名字和版本（P0.8g-impl 禁止加新依赖，只能用既有）
- `sha2` crate 是否已在 `Cargo.toml`（artifact_hash 需要 SHA-256；如果不在，需评估是否违反「不引入新依赖」—— `sha2` 是普遍依赖，可能在；若不在需向用户请示是否豁免）
- `chrono::DateTime<Utc>` vs `NaiveDate` 在 ShadowKlineRow 中的字段类型（ClickHouse Date vs DateTime64）
- `app_shell.rs` 是否真的需要改（dispatch 模式取决于现有 `quantix data openstock validate-live` 的 dispatch 路径；本 session 未完整确认）

## 总结

设计 gate 已稳定合并。实现路径明确，所有关键决策已记录。新 session 可按交接文档直接开干，无需重复调研。
