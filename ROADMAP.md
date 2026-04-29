# Quantix Roadmap

更新日期：2026-04-29

本文件把仓库中已经明确写出的“后续 Phase”能力、设计文档中的后续阶段、以及代码里的长期占坑，整理成一个可执行的优先级 backlog。

自 2026-04-11 起，正式 GSD 规划结构已同步建立在 `.planning/` 下；本文件继续作为项目级优先级总览与对外可读入口，`.planning/ROADMAP.md` 作为 GSD 执行入口。

## 排序原则

1. 优先补齐已经进入实现阶段的主线能力，避免长期停留在半闭环状态。
2. 优先推进最近已有连续设计和提交记录的条线，降低切换成本。
3. 将用户可见缺口与工程占坑分开管理，避免技术债挤占主线容量。

## 当前阶段判断

- 已交付能力已经推进到策略执行 Phase 29C。
- 已完成一轮项目级 MOCK 数据治理，README / USER_MANUAL / FUNCTION_MAP / 路线图系文档已与当前实现边界对齐。
- 已完成第一轮“MOCK policy -> CLI / 运行时边界”收口，相关变更已通过 squash commit `ff84845` 并入 `master`。
- 最连续、最值得继续推进的主线是：
  `strategy run (paper)` -> `strategy daemon` -> `signal` -> `execution request` -> `execution/live-ready`
- 当前最直接的后续工作，不再是补写顶层规范或重复做第一轮对齐，而是继续完成 P0.2 剩余项：`request completed` 语义、gate 可观测性与 operator 排障体验。
- README 中仍有多条能力被明确标记为“延后到后续 Phase”。

## P0：策略执行主线闭环

### 0. 已完成：MOCK policy 与执行边界事实对齐

目标：先把项目里已经存在的 mock / mock_live / qmt_live 边界写清楚并锁住，避免后续实现继续建立在含混文案之上。

已完成：

- 新增 `docs/standards/MOCK_USAGE_POLICY.md`
- 对齐 `README.md`、`docs/USER_MANUAL.md` 与多个现状/架构/路线图文档
- 将 `docs/CLI_COMMAND_MANUAL.html` 纳入文档基线
- 在 `tests/repo_hygiene_test.rs` 中加入回归保护，锁定当前边界语义

结果：

- `anomaly run --mock` 被明确归类为显式 mock 路径
- `strategy run --mode mock_live` 被明确归类为 runtime mock / 仿真执行路径
- 真实提交单路径明确为受保护的 `qmt_live`
- 泛化 `target_mode=live` 仍视为未实现，不允许静默回退到 mock

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

### 2. Execution mainline 语义加固

目标：在已有 Phase 29C 基础上继续收紧执行链路的运行边界与结果语义，而不是重复建设已交付的基础生命周期能力。

推进状态（2026-04-29）：

- 首轮“规则对代码事实”的 mock-policy 审计已经完成，并已作为单个主线提交 `ff84845` 落地。
- 已收紧 `strategy run --mode live` 路由，不再通过通用执行链静默落到 mock 语义。
- 已补齐 strategy / execution / account CLI 关于 `mock_live`、`live`、`qmt_live` 的帮助文本与兼容边界回归测试。
- P0.2 的剩余工作已收敛到 request/result 语义、daemon/operator 可观测性和 residual gate hardening。

交付项：

- 明确 `request completed` 与订单终态的区别
- 补齐 daemon/operator 侧 request 排障与可观测信息
- 收紧 `mock_live` / `live` / `qmt_live` 语义边界，避免 CLI 帮助、运行时报错和用户手册再次漂移
- 核对执行链路里是否仍存在隐式 mock 回退或半接线的 `live` 分支
- 让 operator 能直接看见“为何是 mock_live、为何不是 real live、卡在哪个 gate”

依据：

- `README.md`
- `docs/USER_MANUAL.md`
- `docs/standards/MOCK_USAGE_POLICY.md`

### 3. Real live / broker execution 收口

目标：补齐当前仍明确缺失的实盘执行边界，而不是重复规划已经交付的 execution daemon 基础能力。

交付项：

- `live` adapter
- QMT 从 preview-only 到真实执行的边界决策与实现
- 与 live adapter / broker path 对接的运行边界

依据：

- `README.md`

## P1：交易与风控能力补全

### 4. 风控规则增强

交付项：

- 实盘导入
- 波动率规则
- 行业规则
- 自动减仓

依据：

- `README.md`

## P2：用户可见能力扩展

### 5. 市场分析增强

交付项：

- 历史能力
- 详情能力
- 实时能力

依据：

- `README.md`

### 6. 监控通知能力

交付项：

- 系统通知

依据：

- `README.md`

## P3：工程占坑与基础设施完善

这些工作有价值，但不应抢占策略执行主线。

### 7. CLI / UX 占坑

交付项：

- ratatui TUI 菜单
- Parquet 导出

依据：

- `src/tui/app.rs`
- `src/cli/handlers.rs`

### 8. 数据与运行时基础能力

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

1. 继续推进 P0.2 的剩余收口，优先处理 `request completed` 与订单终态区别、daemon/operator 排障信息、以及 gate 原因可视化。
2. 在已完成的首轮 mock-policy 审计基础上，只修补仍直接影响执行边界的差口，重点是残余隐式 mock 回退、半实现 `live` 分支、以及用户可见文案/报错的再次漂移。
3. 在 P0.2 剩余项收紧完成后，再推进 P0.3，补齐 real live / broker execution 边界。
4. 主线稳定后，再处理 P1 的 risk 缺口，P2 与 P3 继续作为次级队列按需求插入。

## 非目标

以下工作当前不建议抢在 P0 之前做：

- 大范围 UI/TUI 重构
- 与主线无关的格式整理型提交
- 无明确用户需求牵引的通用基础设施扩展
