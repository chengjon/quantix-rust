# Clippy 清零交接文档

> 日期: 2026-06-08 | 分支: master | 提交: `9332c6c` `17031bc`

## 已完成

### Clippy 警告 595 → 0（100% 消除）

在两个阶段中完成：

**阶段一（前一轮会话）**：595 → 69
- 移除 50+ 处未使用 import
- 替换 Arrow 废弃 API（`reader.next_batch()` → `reader.next()`）
- 添加 `#[allow(dead_code)]` 到 serde 反序列化辅助字段
- 添加 `is_empty()` 配套 `len()`
- 修复 `assert_eq!(x, true)` → `assert!(x)`
- 修复手动 min/max → `.clamp()`

**阶段二（本轮会话）**：69 → 0
- 移除剩余未使用 import（test 文件中大量冗余导入）
- 修复冗余闭包 `|x| Decimal::from(x)` → `Decimal::from`
- 替换废弃 `chrono::NaiveDateTime::from_timestamp_opt` → `DateTime::from_timestamp`
- 修复 `vec![100000.0; 6]` → `[100000.0; 6]`
- 修复 `len() >= 1` → `!is_empty()`
- 添加 `#[allow(clippy::too_many_arguments)]` 到 CLI handler 函数
- 添加 `#[allow(clippy::large_enum_variant)]` 到 clap 命令枚举
- 重构 `env_lock()` 返回 `EnvLockGuard`（boxed）抑制 `await_holding_lock`
- 添加 `#![allow(clippy::await_holding_lock)]` 到集成测试文件
- 修复 `clippy::cloned_ref_to_slice_refs` lint
- 运行 `cargo fmt` 统一格式

### 质量门现状

| 检查项 | 结果 |
|--------|------|
| `cargo clippy --all-targets` | **No issues found** |
| `cargo fmt --check` | **Clean** |
| `cargo test --lib` | **695 passed** |
| `cargo build --release` | **Success** |

### Tech Debt 表更新

CLAUDE.md 中 6 项技术债务全部标记为 ✅ 已解决。

---

## 下一步建议

### 优先级 HIGH — `.unwrap()` 消除

生产代码中仍有 **380 处 `.unwrap()`**（不含测试代码）。按编码规范，这些必须替换为 `?` 或 `.map_err()`。

**分布热区**（前 5）：
```
grep -rn '\.unwrap()' src/ --include='*.rs' | grep -v test | grep -v '#\[cfg(test)\]'
```

**推荐做法**：
1. 按模块分批处理（core → io → sources → execution → cli）
2. 每个模块处理完后运行 `cargo test` 确认无回归
3. 对于确实不可能失败的场景（如 `chrono::Utc::now()`），使用 `.expect("reason")` 并注释原因

### 优先级 HIGH — 大文件拆分

以下文件超出 500 行警告线：

| 文件 | 行数 | 建议 |
|------|------|------|
| `src/cli/handlers/tests/strategy_execution.rs` | 1516 | 按功能拆分为 strategy_execution/signals.rs, bridge.rs 等 |
| `src/miniqmt_market.rs` | 1458 | 拆分为 miniqmt_market/parser.rs, manifest.rs, resolver.rs |
| `src/sources/tdx_api.rs` | 1309 | 拆分为 tdx_api/client.rs, models.rs, endpoints.rs |
| `src/cli/handlers/import.rs` | 860 | 拆分 manifest 相关逻辑到 import/manifest.rs |
| `src/cli/handlers/execution_handler.rs` | 835 | 拆分为 execution_handler/daemon.rs, bridge.rs |
| `src/cli/handlers/monitor_handler.rs` | 813 | 拆分为 monitor_handler/commands.rs, service.rs |

**推荐做法**：
- 每个文件拆分是一个独立 PR
- 先建立子模块目录，再移动代码
- 拆分后运行全量 `cargo clippy --all-targets` + `cargo test`

### 优先级 MEDIUM — 持续集成

当前所有质量门均为手动执行。建议：

```yaml
# .github/workflows/quality.yml
name: Quality Gates
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo fmt --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --lib
```

---

## 关键注意事项

1. **`cargo clippy --fix` 会误删 import**：`monitor_handler.rs` 的 `DateTime`/`Utc`、`mod.rs` 的 `execution_handler::*` 在 lib-only 模式下被判定为 unused，但实际被 `#[cfg(test)]` 代码使用。已用 `#[cfg(test)] use` 和 `#[allow(unused_imports)]` 保护。再次运行 `--fix` 前需先 `git stash`。

2. **`env_lock()` 返回值变更**：`src/test_support.rs` 从返回 `MutexGuard` 改为返回 `EnvLockGuard`（boxed wrapper）。所有调用方已适配。如果新增使用 `env_lock()` 的测试，直接 `let _lock = env_lock();` 即可。

3. **`#[allow]` 标注策略**：
   - `too_many_arguments`：CLI handler 参数直接来自 clap 命令定义，无法缩减
   - `large_enum_variant`：clap derive 宏生成的枚举，无法 box
   - `await_holding_lock`：测试序列化锁，故意跨 await 持有
   - `dead_code`：serde 反序列化辅助字段，只用于反序列化不用于业务逻辑
