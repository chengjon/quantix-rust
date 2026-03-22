# PHASE29C Completion Report

## 概要

本报告收口 `Phase 29C mock_live execution foundation` 当前已完成的 slice。

本轮交付的目标不是完整的 execution daemon 或 real live adapter，而是先把当前策略执行路径从“立即成交”升级为“支持非终态订单生命周期、可恢复、可追踪”的执行基础。

本轮完成后，项目已经具备：

- `strategy run --mode mock_live` CLI 入口
- mock-live adapter
- 非终态订单状态模型
- runtime SQLite 持久化的 mock-live 私有状态
- optimistic locking 驱动的 recovery loop
- 针对 mock-live 的独立集成测试

## 本轮完成范围

### 1. 公共执行模型

已完成：

- `OrderStatus::PendingCancel`
- `OrderRecord.remaining_quantity`
- `OrderRecord.last_transition_at`
- `OrderRecord.version`
- `RecoverySummary.unchanged`
- `RecoverySummary.failed`
- `RecoverySummary.skipped`
- `MockLiveOrderState`
- `MockLiveFillStep`
- `MockLiveFaultInjection`

意义：

- 订单生命周期现在可以表达取消中、恢复耗尽、增量成交等后续 live-ready 语义
- 后续 store / adapter / kernel 不再需要猜字段结构

### 2. Runtime Store 与 Schema

已完成：

- `orders` 表扩展：
  - `remaining_quantity`
  - `last_transition_at`
  - `version`
- 新增 `mock_live_orders`
- schema 启动时兼容旧库的列补齐
- 旧订单数据的基础 backfill：
  - `remaining_quantity = requested_quantity - filled_quantity`
  - `last_transition_at = updated_at`
- typed state helper：
  - `insert_mock_live_order_state`
  - `get_mock_live_order_state`
  - `update_mock_live_order_state`
  - `list_recoverable_mock_live_orders`
  - `try_update_order_with_version`

意义：

- 公共 `orders` 现在足够自描述
- mock-live 私有状态被隔离在 `mock_live_orders`
- recovery 能用乐观锁安全推进状态

### 3. Mock Live Adapter

已完成：

- `ExecutionAdapter::adapter_name()`
- `PaperExecutionAdapter` identity
- `MockLiveExecutionAdapter`
- `SystemMockLiveClock`
- `MockLiveExecutionAdapter::with_state_template(...)`
- 支持：
  - 默认 `Accepted`
  - `PartiallyFilled -> Filled`
  - `PendingCancel -> Canceled`
  - `unknown_once`
  - `unknown_always`

意义：

- `paper` 和 `mock_live` 已经是两个独立 adapter
- `mock_live` 不再借用 `paper` 的立即成交语义

### 4. Execution Kernel

已完成：

- `execute_once()` 记录真实 adapter identity，而不是硬编码 `paper`
- `execute_once()` 接受 non-final submit 结果
- partial fill 时会触发 `sync_after_fill()`
- `recover_pending_orders()` 已从空实现升级为真实 recovery loop
- recovery 支持：
  - 扫描 recoverable mock-live 订单
  - query adapter
  - optimistic-lock 更新
  - version 冲突后重试一次
  - `recovery_exhausted` 事件
  - 公共状态保持 `Unknown`

意义：

- 订单已经可以从一次性 submit 过渡到多步生命周期
- recovery 路径不再是占位接口

### 5. CLI 与文档

已完成：

- parser 接受 `--mode mock_live`
- `execute_strategy_run_with_components(...)` 支持 `mock_live`
- README 与 USER_MANUAL 已记录：
  - `mock_live` 命令
  - 非终态状态边界
  - `paper` / `mock_live` / `live` 边界
- `repo_hygiene_test` 已同步更新

意义：

- 用户可见表述已经和当前实现一致
- 仓库 hygiene 现在会持续约束这条文档边界

### 6. 测试覆盖

本轮新增或增强的测试层：

- `tests/execution_runtime_store_test.rs`
- `tests/mock_live_adapter_test.rs`
- `tests/execution_kernel_test.rs`
- `src/cli/tests/strategy.rs`
- `tests/strategy_mock_live_run_test.rs`
- `tests/repo_hygiene_test.rs`

## 提交链

本轮核心提交如下：

1. `4c00842` `feat: extend phase29c execution models`
2. `5e6d00a` `feat: add phase29c mock live runtime store primitives`
3. `278bf9e` `feat: add phase29c mock live execution adapter`
4. `846f3c7` `feat: support phase29c non-final execution kernel states`
5. `dc30761` `feat: add phase29c pending order recovery`
6. `e250031` `feat: wire phase29c mock live strategy run mode`
7. `d709d95` `docs: document phase29c mock live execution boundary`
8. `6a96e7c` `test: cover phase29c strategy mock live run`

## 验证结果

本轮 fresh 验证通过的关键命令：

```bash
cargo test --test execution_runtime_store_test -- --nocapture
cargo test --test mock_live_adapter_test -- --nocapture
cargo test --test execution_kernel_test -- --nocapture
cargo test --test strategy_mock_live_run_test -- --nocapture
cargo test --lib cli::tests::strategy:: -- --nocapture
cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture
cargo test --test repo_hygiene_test -- --nocapture
```

结果：

- `execution_runtime_store_test`: `18 passed`
- `mock_live_adapter_test`: `4 passed`
- `execution_kernel_test`: `17 passed`
- `strategy_mock_live_run_test`: `3 passed`
- `cli::tests::strategy`: `5 passed`
- `cli::handlers::tests::test_strategy_`: `4 passed`
- `repo_hygiene_test`: `16 passed`

## 当前残余风险

### 1. `sync_after_fill()` 仍然偏粗

当前 kernel 仍使用：

- `filled_quantity > 0` 就触发 `sync_after_fill()`

而不是最终设计里更严格的：

- 基于 `old_filled_quantity -> new_filled_quantity` 的 `apply_fill_delta(...)`

这意味着：

- 目前测试语义是成立的
- 但后续若把 recovery 真正接到账户增量变更，仍建议补上显式 fill-delta helper

### 2. 还没有 execution-request consumer

当前 slice 还没有把：

- `execution_request`
- daemon 自动消费
- auto-approval

接进来。

所以当前能力仍然是：

- direct `strategy run --mode mock_live`

而不是完整自动执行链路。

### 3. Graphiti 写入存在上游限流

本轮开发过程里，部分 `quantix_rust_handoff` 写入成功，部分在 ingest 阶段被上游 `429` 限流打断。

当前处理方式符合项目规则：

- 已保留本地等价摘要
- 已在计划文档中标记 `Graphiti backfill required`

这不影响代码交付，但会影响长期记忆完整性。

## 当前结论

`Phase 29C` 这一轮 slice 已经达到“可用的 mock-live 执行基础”：

- 能从 CLI 进入
- 能返回非终态订单
- 能把私有状态写进 runtime
- 能做 recovery
- 能通过 dedicated integration test 验证核心路径

它还不是完整的 live execution system，但已经把架构边界、状态模型、持久化和 recovery 这几个最关键的基础层搭好了。

## 建议下一步

优先建议两条路线中的一条：

1. 补 `apply_fill_delta` 和更严格的账户增量更新语义
2. 进入下一阶段，把 execution-request consumer / daemon automation 接进当前 mock-live 基础
