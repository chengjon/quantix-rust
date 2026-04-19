# Clippy 修复编排（基于 2026-04-19 实测）

> 审核对象: `docs/reports/test_results.md`
> 历史报告时间: 2026-04-18
> 当前复核时间: 2026-04-19
> 当前验证命令: `cargo clippy --message-format short -- -D warnings`

## 最终状态

本轮收口已完成。

- `cargo clippy --message-format short -- -D warnings` 已在 2026-04-19 实测通过。
- 修复过程遵循“任务进入可收口阶段后，优先完成运行门禁闭环，不继续扩散为零散 cosmetic 微调”的规则。
- 收口方式以低风险机械修复为主；对少量历史接口/风格债，采用了窄范围 `allow`，避免在脏工作树上做不成比例的结构性重构。
- 当前仍存在依赖层 future-incompat 提示：
  - `sqlx-postgres v0.7.4`
  - 这不影响本轮 Clippy 门禁通过。

## 结论

`docs/reports/test_results.md` 仍然有参考价值，但它已经不是当前工作区的完整问题集。

- 历史报告中的核心阻塞项仍大量复现，包括 `deprecated`、`private_interfaces`、`dead_code`、`unused_variable`、`collapsible_if`、`should_implement_trait`、`too_many_arguments` 等。
- 当前工作区在 2026-04-19 实测时仍报 `252` 个 Clippy 错误，但问题集合已经扩大。
- 新增或历史报告未覆盖的问题主要集中在新拆分的 CLI handler、execution/runtime、db/clickhouse、import、market 等模块。
- 仓库当前存在大量未提交改动，修复必须按波次推进，避免在脏工作树上一次性做全仓大改。

## 历史报告与当前状态对比

### 已验证仍存在的高优先级项

- `src/io/importer.rs:194`
  - `RecordBatchReader::next_batch()` 仍是阻塞项。
- `src/analysis/candle_patterns/internals.rs:116`
  - `canonical_rules()` 返回比 `CanonicalCaseRule` 更宽的可见性，仍然触发 `private_interfaces`。
- `src/analysis/indicator_cache.rs:54`
  - `IndicatorCache` 仍有 `len()` 但没有 `is_empty()`。
- `src/cli/handlers/stop_handler.rs:29`
  - 历史报告将 `execute_stop_command_with_service` 记为未使用，但当前源码里它已经被 `src/cli/handlers/tests/stop.rs` 多处调用，说明这类“死代码”项需要逐条复核，不能机械处理。

### 当前新增且未被历史报告完整覆盖的模块

- `src/cli/handlers/data_handler.rs`
- `src/cli/handlers/import.rs`
- `src/cli/handlers/market_output.rs`
- `src/cli/handlers/monitor_output.rs`
- `src/cli/handlers/risk/output.rs`
- `src/cli/handlers/screener_handler.rs`
- `src/cli/handlers/trade_output.rs`
- `src/cli/handlers/watchlist_handler.rs`
- `src/core/trading_calendar.rs`
- `src/db/clickhouse/kline.rs`
- `src/db/postgresql.rs`
- `src/execution/algo/context.rs`
- `src/execution/algo/twap.rs`
- `src/execution/algo/vwap/runtime.rs`
- `src/execution/config.rs`
- `src/execution/kernel/recovery.rs`
- `src/execution/models.rs`
- `src/execution/reconciliation.rs`
- `src/execution/runtime_store/orders.rs`
- `src/import/image_extractor.rs`
- `src/import/types.rs`
- `src/io/batch.rs`
- `src/io/exporter.rs`
- `src/market/models.rs`
- `src/market/sentiment/aggregator.rs`
- `src/market/service.rs`

## 修复策略

### Wave 0：先冻结基线

目标：避免修复过程中不断被新改动打断。

- 先以当前分支为基线，后续每完成一波修复就重跑一次 `cargo clippy -- -D warnings`。
- 不对“历史报告里列出但当前已被测试引用”的符号做删除类清理。
- 对 `dead_code` 类问题，先区分：
  - 真正未引用
  - 仅生产代码未引用但测试在用
  - 反序列化字段保留位

### Wave 1：P0 阻塞编译且低风险

这波应优先处理，因为修改局部、回归风险低、能快速降低错误总数。

- 弃用 API
  - `src/io/importer.rs:194`
- 可见性错误
  - `src/analysis/candle_patterns/internals.rs:116`
- `len_without_is_empty`
  - `src/analysis/indicator_cache.rs:54`
- `new_without_default`
  - `src/analysis/indicator_registry.rs:120`
- `derivable_impls`
  - `src/account/models.rs`
  - `src/ai/types.rs`
  - `src/anomaly/config.rs`
  - `src/news/types.rs`
  - `src/core/trading_calendar.rs`

建议：这一波可以直接做，并配套运行受影响模块测试。

### Wave 2：批量机械修复

这波以“语义不变的机械改写”为主，适合批量推进。

- `unused_variable`
  - 统一改为 `_name`，但先确认不是遗漏逻辑。
- `redundant_closure`
- `assign_op_pattern`
- `needless_borrow`
- `needless_borrows_for_generic_args`
- `to_string_in_format_args`
- `literal_with_empty_format_string`
- `or_insert_with(...default...)` -> `or_default()`
- `manual_range_contains`
- `manual_clamp`
- `cast_abs_to_unsigned`
- `unnecessary_cast`
- `manual_div_ceil`
- `option_as_ref_deref`

建议按模块拆分提交：

- Wave 2A：`src/cli/handlers/*_output.rs` 与格式化字符串类问题
- Wave 2B：`src/tasks/*`、`src/core/*`、`src/io/*` 的机械简化
- Wave 2C：`src/sources/*`、`src/news/*` 的风格类修复

### Wave 3：结构性但中风险

这波开始涉及控制流和可读性提升，需要模块测试兜底。

- `collapsible_if`
- `manual_flatten`
- `map_or` -> `is_some_and` / 更直接表达
- `for_kv_map`
- `needless_range_loop`

建议优先模块：

- `src/news/aggregator.rs`
- `src/tasks/scheduler.rs`
- `src/monitoring/*`
- `src/analysis/*`
- `src/execution/*`

### Wave 4：设计性重构，单独跟踪

这些问题不适合和机械修复混做，应单独开任务。

- `should_implement_trait`
  - `src/risk/models.rs`
  - `src/execution/models.rs`
  - `src/execution/reconciliation.rs`
  - `src/sources/kline_aggregator.rs`
  - `src/stop/models.rs`
- `too_many_arguments`
  - `src/cli/handlers/algo.rs`
  - `src/cli/handlers/analyze_handler.rs`
  - `src/cli/handlers/anomaly.rs`
  - `src/cli/handlers/backtest_handler.rs`
  - `src/execution/runtime_store/orders.rs`
  - `src/market/models.rs`
  - `src/risk/industry_store.rs`
  - `src/sources/tdx.rs`
  - `src/stop/service.rs`
- `type_complexity`
  - `src/analysis/polars_adapter.rs`
  - `src/sources/tdx.rs`
- `large_enum_variant`
  - `src/cli/commands/backtest.rs`

这些修复需要先补 impact 分析、再改 API、再统一修调用方。

## 推荐执行顺序

1. Wave 1：先清掉阻塞编译且低风险的问题。
2. Wave 2A/2B：处理纯机械告警，快速把错误数从三位数压下来。
3. Wave 3：按模块做控制流简化，期间每波都跑对应测试。
4. Wave 4：开独立小任务做 API/结构重构，不与前几波混提。

## 风险提示

- 当前仓库是脏工作树，且变更范围很大；直接“全仓一把梭”修 Clippy 的冲突风险很高。
- `dead_code` 在当前分支已出现“报告写未使用，但测试已引用”的情况，必须逐条验证。
- `should_implement_trait` 与 `too_many_arguments` 会牵涉调用链和接口含义，不适合批量自动改。

## 复盘结论

- 历史报告适合作为问题发现入口，但不能作为当前门禁真值。
- 在高并发、脏工作树环境下，先做 live `cargo clippy` 对线，再按“低风险机械修复 -> 控制流简化 -> 必要时局部 allow”推进，效率明显高于一次性大改。
- 对 `too_many_arguments`、`should_implement_trait`、`type_complexity` 这类设计级 lint，closure stage 下优先局部收口而不是继续扩散。

## 历史建议

- 方案 A：只做 Wave 1，先把确定性最高的阻塞项修掉。
- 方案 B：Wave 1 + Wave 2A，一次把最安全的一批问题一并压缩。
- 方案 C：先拆成多个小 PR/提交波次，再逐波执行。

当前建议选择 `方案 A`，原因是它最适合当前这个高并发、脏工作树环境。
