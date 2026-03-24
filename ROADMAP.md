# Quantix Roadmap

更新日期：2026-03-22

本文件把仓库中已经明确写出的“后续 Phase”能力、设计文档中的后续阶段、以及代码里的长期占坑，整理成一个可执行的优先级 backlog。

当前仓库没有独立的 `.planning/` 规划结构；在引入正式规划系统前，本文件作为项目级路线图与 backlog 的统一入口。

## 排序原则

1. 优先补齐已经进入实现阶段的主线能力，避免长期停留在半闭环状态。
2. 优先推进最近已有连续设计和提交记录的条线，降低切换成本。
3. 将用户可见缺口与工程占坑分开管理，避免技术债挤占主线容量。

## 当前阶段判断

- 已交付能力已经推进到策略执行 Phase 29B。
- 最连续、最值得继续推进的主线是：
  `strategy run (paper)` -> `strategy daemon` -> `signal` -> `execution request` -> `execution/live-ready`
- README 中仍有多条能力被明确标记为“延后到后续 Phase”。

## P0：策略执行主线闭环

### 1. Phase 29C：Live-ready execution hardening

目标：把现有 paper 执行骨架推进到“接近 live 约束、但仍可控”的阶段。

交付项：

- `mock_live` adapter
- delayed / partial fills
- `Unknown` 注入与恢复
- open-order 扫描与 reconciliation
- 网络故障模拟
- account / order reconciliation scaffolding

依据：

- `docs/superpowers/specs/2026-03-17-phase29a-strategy-paper-execution-kernel-design.md`

### 2. Execution request 生命周期闭环

目标：结束 Phase 29B 只会创建 `pending` request 的半闭环状态。

交付项：

- `execution_request` 从 `pending` 走向 `completed` / `failed` / `canceled`
- 将 signal 审批后的 request 执行结果写回 runtime store
- 补齐 request 查询与排障信息，便于后续 daemon/operator 使用

依据：

- `docs/superpowers/specs/2026-03-18-phase29b-strategy-signal-daemon-design.md`

### 3. Execution automation 收口

目标：补齐 README 已经明确延后的自动化能力。

交付项：

- 自动审批策略
- execution daemon
- 与 live adapter 对接的运行边界

依据：

- `README.md`

## P1：交易与风控能力补全

### 4. Stop 命令补全

交付项：

- `stop status`
- `stop history`
- `stop update`
- 百分比止损 / 止盈参数

依据：

- `README.md`

### 5. 风控规则增强

交付项：

- 实盘导入
- 波动率规则
- 行业规则
- 自动减仓

依据：

- `README.md`

## P2：用户可见能力扩展

### 6. 市场分析增强

交付项：

- 历史能力
- 详情能力
- 实时能力

依据：

- `README.md`

### 7. 监控通知能力

交付项：

- 系统通知

依据：

- `README.md`

## P3：工程占坑与基础设施完善

这些工作有价值，但不应抢占策略执行主线。

### 8. CLI / UX 占坑

交付项：

- ratatui TUI 菜单
- Parquet 导出

依据：

- `src/tui/app.rs`
- `src/cli/handlers.rs`

### 9. 数据与运行时基础能力

交付项：

- 交易日历节假日数据加载
- batch 流式进度显示
- monitoring health / metrics 导出

依据：

- `src/core/trading_calendar.rs`
- `src/io/batch.rs`
- `src/monitoring/mod.rs`

## 执行建议

下一阶段建议按下面顺序推进：

1. 先完成 P0.1 和 P0.2，保证 execution 主线从“可生成 request”升级为“可追踪执行结果”。
2. 再推进 P0.3，补齐自动审批和 execution daemon。
3. 主线稳定后，再处理 P1 的 stop / risk 缺口。
4. P2 与 P3 作为次级队列，按用户需求和资源情况插入。

## 非目标

以下工作当前不建议抢在 P0 之前做：

- 大范围 UI/TUI 重构
- 与主线无关的格式整理型提交
- 无明确用户需求牵引的通用基础设施扩展
