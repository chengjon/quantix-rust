# Handoff — P0.8f OpenStock Live Shadow Validation

日期：2026-06-29
分支：`feat/openstock-p0-8f-live-shadow-validation`（worktree：`.worktrees/openstock-p0-8f-live-shadow-validation`）
主仓库 HEAD：`45c0312` (master)
P0.8f worktree HEAD：`31d10b3 feat: add openstock p0.8f live shadow validation`

---

## 一、本条线已完成任务（不含 function-tree skill 升级）

### 1. 治理节点授权（已在 worktree commit）

**Commit**: `042359d chore(governance): add P0.8f OpenStock live shadow validation node`

新建 FUNCTION_TREE 节点 P0.8f，状态推进至 `approved-for-implementation`：

- `id`: P0.8f
- `title`: OpenStock live shadow validation
- `FT ref`: `sources/`，父节点 P0.8
- `allowed_paths`: `src/sources/openstock.rs; src/cli/commands/data.rs; src/cli/handlers/data_handler.rs; tests/openstock_live_shadow_validation_test.rs; tests/openstock_fixture_validation_cli_test.rs; docs/reports/OPENSTOCK_DATA_CONSUMPTION_P0_8F_LIVE_SHADOW_VALIDATION_2026-06-28.md; openspec/changes/openstock-data-consumption-p0-8/tasks.md; README.md; CHANGELOG.md; FUNCTION_TREE.md`
- `non_goals`: No ClickHouse writes / 不替换生产数据源路由 / 不改 qmt_live/miniQMT/ExecutionAdapter/OrderStatus / 默认 CI 无 live 网络调用 / 不提交 API Key / 不做 unwrap 清理
- `evidence.path`: 已固化线上联调关键证据（服务地址 `http://192.168.123.109:8000`、X-API-Key 鉴权 401、`/data/bars` 返回 100 KLINES rows、start/end/limit 不生效 → 必须 drift detection）

治理文件：`.governance/active-gates.json`、`.governance/active-gates.md`、`.governance/programs/project-governance/cards/P0.8f.yaml`、`nodes.json`、`tree.md`

### 2. TDD 实现（已在 worktree commit `31d10b3`）

**Commit**: `31d10b3 feat: add openstock p0.8f live shadow validation`

#### 新增/修改文件（13 个，+916 行）

| 文件 | 变化 |
|---|---|
| `src/sources/openstock.rs` | +308 行：`validate_live_shadow_payload`、`LiveShadowRequest/Report/Status/Drift` 类型、`live_shadow_error_into_quantix` |
| `src/cli/commands/data.rs` | +27 行：新增 `ValidateLive` 子命令 enum |
| `src/cli/handlers/openstock_handler.rs` | +47 行：`validate_openstock_live` CLI handler |
| `src/cli/handlers/app_shell.rs` | +10 行：match arm 分发 |
| `src/cli/handlers/mod.rs` | +1/-1 行：`use` 导入扩展 |
| `src/sources/mod.rs` | +5/-1 行：`pub use` 新增 5 个导出 |
| `tests/openstock_live_shadow_validation_test.rs` | +176 行：10 个单元测试 |
| `tests/openstock_live_validation_cli_test.rs` | +162 行：4 个 CLI 集成测试 |
| `docs/reports/OPENSTOCK_DATA_CONSUMPTION_P0_8F_LIVE_SHADOW_VALIDATION_2026-06-28.md` | +159 行：设计/验收报告 |
| `openspec/changes/openstock-data-consumption-p0-8/tasks.md` | +11 行：5f.1–5f.8 全打勾 |
| `FUNCTION_TREE.md` / `README.md` / `CHANGELOG.md` | +1/+1/+9 行 |

#### 测试覆盖（10 unit + 4 CLI = 14 个，全绿）

| 测试 | 覆盖点 |
|---|---|
| `maps_live_payload_into_dry_run_report_without_drift` | 有效 envelope → dry_run 报告 |
| `flags_drift_when_service_returns_more_records_than_limit` | limit drift 检测 |
| `flags_drift_when_returned_range_falls_outside_requested_window` | 时间窗口 drift |
| `fail_closes_on_missing_symbol_field` | 缺 symbol fail-closed |
| `fail_closes_on_unparseable_time` | 坏 time fail-closed |
| `fail_closes_on_non_daily_period_in_record` | 非 daily period fail-closed |
| `fail_closes_when_record_symbol_mismatches_request` | symbol mismatch fail-closed |
| `rejects_invalid_envelope_json` | envelope JSON 拒绝 |
| `rejects_empty_records_envelope` | 空 envelope 拒绝 |
| `report_implements_display_for_dry_run_log` | Display 实现 |
| CLI ×4 | valid / drift / fail-closed / missing-file |

#### 门禁复核（由本会话独立重跑验证）

| 门禁 | 结果 |
|---|---|
| `cargo fmt --check` | ✅ EXIT=0 |
| `cargo clippy --all-targets -- -D warnings` | ✅ EXIT=0 |
| `cargo test`（全量） | ✅ 1358 passed |
| P0.8f focused tests | ✅ 14/14 passed |
| `git diff --check master...HEAD` | ✅ EXIT=0（无 whitespace/error） |
| GitNexus `detect_changes`（compare master） | ⚠️ HIGH（见下） |

#### GitNexus detect_changes 风险评估

- **报 HIGH**：9 个 process 受影响
- **实际风险**：**无回归风险**——9 个 process 全部经过 `run_data_command`（CLI 总入口）
- 改动性质：`app_shell.rs` 只新增一个 `OpenStockCommands::ValidateLive` match arm（纯加法），既有 case 全部未动
- 既有命令（query_kline_data / import_market_fundamentals / list_data_sources / add_data_source / set_default_data_source / test_data_source / export_data / run_tdx_api_command）行为零变更
- 9 process 是结构化"触及"，非逻辑改动

### 3. allowed_paths 合规性核对（本会话发现）

| 文件 | 在 allowed_paths? | 性质 | 处置 |
|---|---|---|---|
| `src/cli/handlers/app_shell.rs` | ❌ | match arm 分发，必要接线 | 需扩 card 或备 evidence |
| `src/cli/handlers/openstock_handler.rs` | ❌（card 写的是 `data_handler.rs`，疑似笔误） | 实现函数 | 需修 card 笔误 |
| `src/cli/handlers/mod.rs` | ❌ | `use` 导入（mod.rs 规则允许） | 需扩 card |
| `src/sources/mod.rs` | ❌ | `pub use`（mod.rs 规则允许） | 需扩 card |
| `tests/openstock_live_validation_cli_test.rs` | ❌ | CLI 集成测试 | 需扩 card |

**结论**：实现未越界，但 card `allowed_paths` 写得过窄，需要在 closeout 时**修订 card**（扩 allowed_paths 至 5 个额外 plumbing 文件）或记录 evidence 备案。

---

## 二、⚠️ 未提交工作（另一个会话迭代的真实 envelope 契约扩展）

worktree `31d10b3` 之后，另一个会话基于真实生产 `/data/bars` envelope 又改了 2 个文件，**尚未 commit**：

```
 M src/sources/openstock.rs                       (+53/-15)
 M tests/openstock_live_shadow_validation_test.rs (+78)
```

### 改动要点

基于真实 NAS 服务 envelope（`http://192.168.123.104:8040`）观察到的契约差异：

| 字段 | 假设（已提交版本） | 真实（未提交迭代） |
|---|---|---|
| envelope records 字段名 | `records` | `data`（保留 `records` alias） |
| symbol 格式 | `600000` | `sh600000` / `sz000001`（需 normalize） |
| period 值 | `daily` | `day`（需 `is_daily_period` 容忍） |
| 时间戳 | `2026-06-22` | `2026-06-22T15:00:00+08:00`（ISO 8601） |
| envelope.adjust_type | 解析 | 硬编码 `AdjustType::None`（生产不返回） |

### 新增测试（未提交）

```rust
// 真实 envelope 形状（2026-06-28 NAS openstock 服务实测）
// Top-level: {"data": [...], "source": ..., "data_category": ...}
// Record:    symbol=`sh600000`, time=`2026-01-23T15:00:00+08:00`, period=`day`

fn accepts_real_envelope_with_data_field_and_normalizes_symbol_prefix()
// ...（其他真实形状相关测试）
```

### 下一步必须先做

1. **决定是否保留这部分未提交工作**——它是真实生产契约的必要扩展，建议保留
2. 跑门禁复核：`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
3. 若全绿 → 独立 commit（建议 message：`feat(openstock): align live shadow validator with real envelope shape`）
4. 若有失败 → 修复后重跑

---

## 三、P0.8f 治理 closeout 状态（仍未推进）

### 还差什么

| 项 | 状态 |
|---|---|
| FUNCTION_TREE 节点状态 | ❌ 仍是 `[ ] approved-for-implementation`，未推进到 `[x] closed` |
| `active-gates.md` | ❌ 仍是 approved-for-implementation |
| `ft doc` 刷新 | ❌ 未跑 |
| PR 创建 | ❌ 未做 |

### 推进步骤（function-tree skill 已升级到 `a04fe29`，可用）

```bash
cd /opt/claude/quantix-rust/.worktrees/openstock-p0-8f-live-shadow-validation

# 1. 先处理未提交工作（见上节）
# 2. 跑门禁
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

# 3. 推进治理状态（ft-governance.cjs 现已可用）
~/.claude/skills/function-tree/scripts/ft-governance.cjs transition \
  project-governance P0.8f --to closed

# 4. 刷新 FUNCTION_TREE.md
~/.claude/skills/function-tree/scripts/ft-governance.cjs doc

# 5. 创建 PR
gh pr create --base master \
  --title "feat: add openstock p0.8f live shadow validation" \
  --body "..."
```

---

## 四、下一步工作计划（按优先级）

### P0（立即）

1. **处理 P0.8f 未提交的真实 envelope 迭代**（见第二节）
2. **完成 P0.8f 治理 closeout**（见第三节）
3. **修订 P0.8f card 的 `allowed_paths`**：补 5 个 plumbing 文件
   - `src/cli/handlers/app_shell.rs`
   - `src/cli/handlers/openstock_handler.rs`（替换笔误的 `data_handler.rs`）
   - `src/cli/handlers/mod.rs`
   - `src/sources/mod.rs`
   - `tests/openstock_live_validation_cli_test.rs`
4. **创建 P0.8f PR** 并跑 CI

### P1（P0.8f 合并后）

5. **P0.8g: OpenStock shadow persistence opt-in 设计**
   - 启动条件：P0.8f dry-run report 稳定 + GitNexus impact 确认 LOW/MEDIUM
   - 设计门禁先行：明确 shadow namespace / table / manifest / rollback / row count 校验
   - 仍禁止替换生产数据源路由

### P2（P0.8g 通过后）

6. **P0.8h: OpenStock 到 analysis/backtest 的更宽链路验证**
   - 更真实数据样本验证 parser / canonical Kline / indicators / backtest path / dry-run report
   - 优先 fixture/artifact 驱动，避免 CI 依赖外网

### 维护态（不主动推进）

- **qmt_live runtime readiness**：等 operator 提供 miniQMT Windows Bridge runtime + 账户标签 + 只读 smoke evidence
- **Graphiti ingest**：多个节点 ingest processing / rate_limit；策略是仓库内 Markdown backfill 保存等价结论，待 Graphiti 稳定后回补

---

## 五、当前明确禁止的事项（延续 P0.8f card non_goals）

- ❌ 不写生产 ClickHouse
- ❌ 不替换生产数据源路由
- ❌ 不改 qmt_live / miniQMT / ExecutionAdapter / OrderStatus
- ❌ 默认 CI 不做 live OpenStock 网络调用
- ❌ 不提交 API Key 到仓库
- ❌ 不复用 GitNexus impact=HIGH 的 miniQMT `ControlledPersistencePolicy`
- ❌ 不继续 `.unwrap()` cleanup（已闭合为技术债备案）
- ❌ 不启动 qmt_live canary

---

## 六、关键参考文件

- 本 handoff：`docs/reports/HANDOFF_P0_8F_SHADOW_VALIDATION_2026-06-29.md`
- P0.8f 设计/验收：`docs/reports/OPENSTOCK_DATA_CONSUMPTION_P0_8F_LIVE_SHADOW_VALIDATION_2026-06-28.md`
- P0.8e shadow design gate：`docs/reports/OPENSTOCK_DATA_CONSUMPTION_P0_8E_SHADOW_VALIDATION_DESIGN_2026-06-28.md`
- OpenSpec tasks：`openspec/changes/openstock-data-consumption-p0-8/tasks.md`
- FUNCTION_TREE：`FUNCTION_TREE.md` + `.governance/programs/project-governance/tree.md`
- 上一份总结（master，未提交）：`docs/reports/THREAD_COMPLETION_SUMMARY_AND_NEXT_PLAN_2026-06-28.md`
