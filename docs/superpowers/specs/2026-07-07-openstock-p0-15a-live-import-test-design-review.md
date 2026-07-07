# Review: docs/superpowers/specs/2026-07-07-openstock-p0-15a-live-import-test-design.md

**Date**: 2026-07-07
**Scope**: docs/superpowers/specs/2026-07-07-openstock-p0-15a-live-import-test-design.md
**File type**: md
**Doc type**: spec（path 含 `spec`，内容含测试合约 / 验收标准 / 输入输出定义）
**Perspectives**: Completeness (2x), Codebase Alignment (2x), Consistency

---

## Evidence Verification

### 文件引用

| Claimed path | Exists? | Actual location |
|------|---------|----------|
| `tests/openstock_live_import_minute.rs` | no | **目标文件**（设计意图创建） |
| `tests/openstock_live_minute_share.rs` | yes | `tests/openstock_live_minute_share.rs` (215 lines) |
| `src/cli/tests/data.rs` | yes | `src/cli/tests/data.rs`（含 `import_minute_*` 单元测试） |
| P0.15a design §11 (link) | yes | `docs/superpowers/specs/2026-07-05-openstock-p0-15a-minute-cli-persistence-design.md` (386 lines) |

### 符号引用

| Symbol | Found? | Location |
|--------|--------|----------|
| `import_openstock_minute_klines` | yes | `openstock_handler.rs:638` (`pub(crate)`) |
| `import_openstock_minute_share` | yes | `openstock_handler.rs:745` (`pub(crate)`) |
| `compute_apply` | yes | `openstock_handler.rs:42` — `fn compute_apply(apply: bool) -> bool` |
| `MINUTE_APPLY_ENV` | yes | `openstock_handler.rs:32` — `"QUANTIX_OPENSTOCK_MINUTE_APPLY"` |
| `ImportMinuteKlines` (CLI enum) | yes | `data.rs:442` — fields: code, period, adjust, start, end, apply |
| `ImportMinuteShare` (CLI enum) | yes | `data.rs:470` — fields: code, start, end, apply |
| `app_shell.rs` dispatcher arms | yes | `ImportMinuteKlines` L418, `ImportMinuteShare` L438 |
| `ClickHouseMinuteKlineSink` | yes | `minute.rs:136` — re-exported at `mod.rs:19` |
| `stream_minute_klines_to_clickhouse` | yes | `minute.rs:210` |
| `serial_test` crate | no | **未在 Cargo.toml 中找到** |

### 声明验证

| Claim | Status | Evidence |
|-------|--------|----------|
| P0.15a merged commits `92b2b1e` + `a4fc6da` | confirmed | `git log`: `92b2b1e feat(p0.15a): wire ImportMinuteKlines/Share`; `a4fc6da fix(p0.15a): use OffsetDateTime` |
| `import_openstock_minute_*` 为 `pub(crate)` | confirmed | L638, L745 — 证实无法从 `/tests/` 直接调用 |
| stdout 含 `dry_run: false, applied: true` (apply 路径) | confirmed | `openstock_handler.rs:732` |
| stdout 含 `dry_run: true, applied: false` (dry-run 路径) | confirmed | `openstock_handler.rs:699` |
| stdout 含 `OpenStock import-minute-klines (apply)` | confirmed | `openstock_handler.rs:670-671` 使用 `if will_apply { "apply" }` |
| hint 消息 `set QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to actually insert` | confirmed | `openstock_handler.rs:32` 常量值 + `openstock_handler.rs:704` format |
| `--period` / `--adjust` 被 `ImportMinuteShare` 拒绝 | confirmed | `data.rs:470-486` 只有 code/start/end/apply |
| `openstock_live_minute_share.rs` 使用 `#[ignore]` + 环境变量 early-return 模式 | confirmed | L1-7 (doc comment) + L15-26 (`settings_from_env`) |

---

## Checklist Results

### Completeness (2x)
14 items PASS。

| # | Check | Result | Notes |
|---|-------|--------|-------|
| CP1 | 所有引用符号存在 | PARTIAL | `serial_test` crate 未在 Cargo.toml 声明。其余 10/10 符号验证通过 |

### Codebase Alignment (2x)
9 items PASS。

| # | Check | Result | Notes |
|---|-------|--------|-------|
| CA1 | 引用 API 签名与代码匹配 | PASS | `compute_apply(apply: bool) -> bool` 与文档一致；`MINUTE_APPLY_ENV` 值匹配 |
| CA1 | 引用 CLI 参数与代码匹配 | PASS | `ImportMinuteKlines` 6 字段、`ImportMinuteShare` 4 字段与 `data.rs` 一致 |
| CA4 | 无名称冲突 | PASS | `openstock_live_import_minute.rs` 与现有 `openstock_live_minute_share.rs` 命名风格一致 |

### Consistency
5 items PASS。无 FAIL。

---

## Findings

### Critical Issues

无。

### Medium Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| M1 | §3 Concurrency | `serial_test` dev-dependency 未在 Cargo.toml 声明 | 实现时需手动添加；若使用 `#[serial_test::serial]` 宏需同时导入 `serial_test` crate | `grep -c serial_test Cargo.toml` → 0 | 在 §3 增加明示：`Cargo.toml [dev-dependencies] serial_test = "3"` |
| M2 | §2 T1/T2 | pre/post 使用 `ALTER TABLE ... DELETE` 做清理，未讨论 ClickHouse 权限/版本要求 | ClickHouse lightweight DELETE 需 `allow_experimental_lightweight_delete=1`（23.3+）或 `DELETE` 权限。若本地 CH 未启用，清理会失败 | T1/T2 引用 `ALTER TABLE minute_klines DELETE WHERE` | 在 §1 或 §2 增加前提声明：`DELETE` 已验证对本地 CH 26.2.4.23 可用，或改用 `TRUNCATE ... WHERE` / 手动清理 |
| M3 | §2 T3 | T3 snapshot count 使用 `date='2026-07-03'`，而 T1 post-check 使用 `timestamp >= '...' AND timestamp <= '...'` | 若 `minute_klines` 表无 `date` 列，T3 查询失败 | T1 用 `timestamp` 列过滤；T3 用 `date` 列过滤 — 两个查询列名不一致 | 统一为 `timestamp >= '...' AND timestamp <= '...'`，或确认表存在 `date` 别名列后标注 |

### Low Issues

| # | Section | Issue | Evidence | Recommendation |
|---|---------|-------|----------|----------------|
| L1 | §1 Gates | 增加了 `QUANTIX_CLICKHOUSE_LIVE=1` gate，但参考模式 `openstock_live_minute_share.rs` 仅用 `QUANTIX_OPENSTOCK_LIVE=1` | `openstock_live_minute_share.rs:4-7` 只列出 `QUANTIX_OPENSTOCK_LIVE=1` | 解释新增 gate 的理由（ClickHouse 写入是高危操作，需要独立的确认开关） |
| L2 | §2 T1 | post-cleanup `DELETE` 在断言之后执行 | 若断言 panic，post cleanup 不运行，残留数据影响下次运行 | 在 T1/T2 增加 `pre:` 清理说明（"每次运行前手动清理，或首次运行后已清"）；或将 post cleanup 改为 pre-only（每次测试前清理而非后清理）同时接受幂等 |
| L3 | §4 | 未显式引用 `openstock_live_minute_share.rs` 文件路径 | 仅口头称其为模式，读者需自行定位 | 在 §1 Surface 增加 "模式参考: `tests/openstock_live_minute_share.rs`" |

---

## Strengths

- **设计决策有据**：§0 清晰追溯 P0.15a 设计文档的 acceptance gap，说明为什么这个测试文件有必要性。"no backing file" 的诊断可验证（`openstock_live_import_minute.rs` 当前确实不存在）。
- **为什么用 subprocess 的论证充分**：`pub(crate)` 可见性约束 + 端到端测试价值（arg parse → dispatcher → handler → sink），避免了为测试提升 API 可见性的反模式。
- **T3（dry-run 不写入）设计精妙**：不仅验证 `applied: false`，还验证 hint 消息出现（证明 `--apply` flag 被解析但 env gate 拒绝了），反向检查 count 不变（证明真的没写入）。三层断言覆盖了最常见的 "flag 没传进去" 和 "静默 dry-run 但误以为 apply" 两种 bug。
- **日期选择有出处**：`2026-07-03` 明确说明来自 2026-07-07 手动验证，防止后续读者质疑 "为什么是这个日期"。
- **Non-goals 清晰**：明确排除 `--adjust` 变体、并发压力、负向参数验证——每一项都给出去向（在哪里已被覆盖）。
- **与现有测试对齐**：`#[ignore]` + env-var early-return 模式完全模仿 `openstock_live_minute_share.rs`，风格一致。
- **输出断言与 handler 代码逐字匹配**：`applied: true`、`dry_run: true, applied: false`、hint 消息 — 全部通过交叉验证确认。

---

## Recommendations

1. **在 §3 明示 `serial_test` dev-dependency 添加方式**（M1）：`Cargo.toml` 中 `[dev-dependencies] serial_test = "3"`。
2. **在 §1 或 §2 声明 ClickHouse DELETE 权限前提**（M2）：确认对本地 CH 26.2.4.23 可用。
3. **统一 T3 snapshot count 列名**（M3）：将 `date='2026-07-03'` 改为 `timestamp >= '...' AND timestamp <= '...'` 形式，与 T1 post-check 一致。
4. **补充 `QUANTIX_CLICKHOUSE_LIVE=1` gate 理由**（L1）：解释为何需要独立于 `QUANTIX_OPENSTOCK_LIVE` 的确认开关。
5. **将 post-cleanup 改为 pre-only**（L2）：测试前清理而非测试后清理，使得幂等且不依赖测试成功执行。
6. **在 §1 显式引用参考模式文件路径**（L3）。

---

## Scoring

| Dimension | Score (1-5) | Evidence |
|-----------|-------------|----------|
| Technical Accuracy | 5 | 所有输出断言、CLI 参数、handler 签名与代码逐字匹配。0 矛盾发现 |
| Completeness (2x) | 4 | 覆盖 apply/dry-run/serial 三大场景。扣分：CLI 参数 `--code` 用 `short` (`-c`) 但文档 CLI 命令用 `--code`（长格式），未讨论与 `period`/`adjust` 长格式参数的交互——但这不是 test 文档职责 |
| Codebase Alignment (2x) | 5 | 子进程调用、env gate 常量、输出格式、early-return 模式均与现有代码一致 |
| Actionability | 5 | 文件名、测试用例结构、断言、CLI 调用、环境变量清单全部具体可执行 |
| Terminology Consistency | 5 | `import-minute-klines`/`import-minute-share`、`--apply`、`QUANTIX_OPENSTOCK_MINUTE_APPLY` 术语与代码库一致 |
| **Overall** | **4.7** | 加权：(5+4×2+5×2+5+5)/7 = 4.71 |

---

## Verdict

APPROVE_WITH_NOTES

设计质量高——测试合约定义精确，所有输出断言与 handler 代码逐字验证通过，subprocess 决策论证充分，T3 三层断言设计精巧。三个 Medium 问题（`serial_test` 未声明、DELETE 权限前提、T3 列名不一致）不阻塞设计通过，但应在实施前修正以避免返工。0 个 Critical 问题。
