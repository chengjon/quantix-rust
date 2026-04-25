# quantix-rust MOCK 数据使用规范

本规范是 `quantix-rust` 项目关于 MOCK 数据、测试替身、模拟执行与相关文档表述的顶层规则源。

适用范围：

- 运行时显式 MOCK 入口，例如 `quantix anomaly run --mock`
- 运行时模拟执行路径，例如 `strategy run --mode mock_live`
- 测试中使用的 fixture、fake loader、test double、mock adapter、stub state
- README、用户手册、测试报告、阶段报告中的 MOCK 相关表述

本规范的目标不是禁止 MOCK，而是明确：

- 什么场景允许使用 MOCK
- 什么场景必须显式区分 MOCK 与真实链路
- 什么表述不能写
- 什么验证结果不能被错误升级为“真实能力已完成”

## 1. 核心原则

### 1.1 MOCK 用于解耦，不用于伪装真实能力

MOCK 可以用于开发解耦、测试稳定性、离线演练、演示环境与运行时受控模拟，但不能把尚未完成的真实链路伪装成已经完成。

### 1.2 显式 MOCK 可以接受，静默 fallback 不可接受

当用户、开发者或测试明确进入 MOCK 模式时，系统可以返回 MOCK 数据或执行模拟生命周期。

但真实路径失败后静默切到 MOCK，并继续把结果表现成真实成功，是禁止行为。

### 1.3 Mock acceptance 不等于 real-path verification

通过 MOCK 模式完成的测试、演示、命令验证，只能证明：

- 命令壳层可用
- 流程编排可运行
- 数据结构或状态机在模拟条件下成立

不能据此宣称：

- 已完成真实数据源验证
- 已完成真实券商/桥接验证
- 已完成真实交易链路验收

### 1.4 文档表述必须与当前代码边界一致

历史设计文档、阶段报告、规划文档可以保留，但不得覆盖当前事实。

当前仓库中的已知边界必须持续保持清晰：

- `mock_live` 是模拟执行路径，不是真实 broker live execution
- 真实提交路径应以当前受保护的 `qmt_live` 语义为准
- `anomaly run --mock` 是显式模拟数据入口，不代表真实市场数据链路已验证

## 2. MOCK 分类

### 2.1 运行时 MOCK

指用户或开发者在实际 CLI 运行时显式触发的模拟数据或模拟执行能力。

当前项目已存在的运行时 MOCK 主要包括：

- `quantix anomaly run --mock`
- `quantix strategy run --mode mock_live`

这类能力属于“产品可见的受控模拟路径”，必须在命令语义、输出、文档和验收结果中明确标识其模拟属性。

### 2.2 测试替身

指测试代码中使用的固定样本、假实现或替身对象，包括但不限于：

- fixture JSON / 固定 K 线样本
- fake loader
- fake resolver
- mock adapter
- stub runtime state
- 用于测试的内存 store / bridge / capability 替身

这类能力属于“测试基础设施”，其目标是稳定、可重复、可控，不需要对终端用户暴露，但必须保持命名清晰、边界清晰、不要渗透进生产业务语义。

### 2.3 文档级 MOCK 表述

指 README、`docs/USER_MANUAL.md`、阶段 spec / plan / report、测试报告、审计报告中的 MOCK 描述。

文档必须区分以下三类结论：

- MOCK 验证通过
- 部分真实链路验证通过
- 真实链路验证通过

不得混写。

## 3. 生命周期分级

本项目的 MOCK 相关能力统一按以下口径表述：

### 3.1 `verified_real`

同时满足以下条件时，才可以称为真实路径已验证：

- MOCK 模式未启用
- 请求命中了真实上游、真实桥接或真实数据链路
- 业务消费的字段来自真实返回
- 错误、空结果、加载和追踪状态没有被 MOCK 掩盖
- 输出或测试记录中明确注明为真实路径验证

### 3.2 `mock_accepted`

当流程在显式 MOCK 模式下通过时，应表述为 MOCK 验证通过。

适用示例：

- `anomaly run --mock` 在离线条件下跑通
- `mock_live` 订单生命周期按预期演进
- 测试中的 fake loader / fixture 使策略评估稳定可重现

### 3.3 `pending_real`

当真实路径仍未完成、仅具备 MOCK 能力、或真实能力仍受外部依赖/桥接能力限制时，应表述为真实路径待完成，而不是“已完成”。

## 4. 允许使用 MOCK 的场景

- 外部依赖尚未就绪，但本地流程、状态机或 CLI 交互需要继续开发
- 单元测试、集成测试需要稳定、可重复、可确定的输入
- 守护进程、执行内核、风控编排等需要在隔离环境中验证流程闭环
- 演示、培训、离线联调需要可控样本
- 需要验证 `mock_live` 的 delayed fill、partial fill、`unknown` 恢复、reconciliation 语义
- 需要验证 `anomaly` 流程在无外部市场数据时的命令与输出结构

## 5. 禁止或受限场景

### 5.1 禁止把 `mock_live` 写成实盘能力

以下表述禁止出现：

- “`mock_live` 已支持真实实盘交易”
- “`mock_live` 等价于 live 交易”
- “`mock_live` 验证通过，因此券商真实链路已完成”

### 5.2 禁止真实路径失败后静默回落到 MOCK

如果真实数据源、桥接、下单或查询失败：

- 可以失败
- 可以显式提示切换到 MOCK
- 可以在单独的 MOCK 模式下重试

但不能在同一条用户路径上静默返回 MOCK 数据并表现成真实成功。

### 5.3 禁止在生产业务模块散落大块匿名假数据

生产业务代码中不应散落难以追踪的大块内联假数据，尤其是：

- 没有明确命名的临时样本
- 与真实契约不一致的便利字段
- 无法判断是否仅测试使用的 fallback 对象

优先做法：

- 集中到 fixture / helper / 专用 mock 模块
- 保持字段形状尽量贴近真实契约
- 在命名上明确 `mock` / `fake` / `fixture` / `stub`

### 5.4 禁止把测试替身结果上升为真实验收结论

测试报告、阶段报告、手工验证记录中，如果依赖：

- fixture
- fake loader
- mock adapter
- `--mock`
- `mock_live`

则结论必须显式标明为 MOCK 或模拟执行验证，不能直接写成“真实能力完成”。

## 6. 运行时 MOCK 规则

### 6.1 `anomaly run --mock`

`anomaly run --mock` 的定位是：

- 提供离线、可控、可重复的异常检测流程输入
- 支持命令验证、输出验证、参数验证与演示

它不证明：

- 东方财富等真实行情源已验证
- 实时或批量市场数据链路稳定
- 真实行情质量满足生产要求

### 6.2 `strategy run --mode mock_live`

`mock_live` 的定位是：

- 验证非终态订单生命周期模拟
- 验证 delayed fill、partial fill、`pending_cancel`、`unknown` 等状态语义
- 验证 reconciliation / recovery / fill-delta accounting 的工程闭环

`mock_live` 不代表：

- 真实 broker live execution
- 默认真实下单入口
- 通用 `live` 模式已经可用

对于 `mock_live` 相关文档和输出，应持续保持以下语义：

- 订单状态可以是非终态
- request 可能已经 `completed`，但 order status 仍是非终态
- 一个逻辑上的 mock-live 订单可能写出多笔 `TradeRecord`

## 7. 测试替身规则

### 7.1 命名必须一眼可识别

测试代码中的替身、样本和辅助类型应优先使用以下命名：

- `fixture`
- `fake`
- `mock`
- `stub`
- `sample`

避免使用模糊命名让人误以为是真实实现。

### 7.2 优先集中管理

能复用的测试样本、假加载器、模拟状态，应优先放在：

- 测试 helper
- fixture 文件
- 专用测试模块

而不是在多个测试或生产模块里重复内联复制。

### 7.3 数据形状应尽量贴近真实契约

测试替身可以简化，但不应长期偏离真实结构到足以误导实现。

尤其禁止：

- 增加真实链路中不存在、却被业务代码默认为存在的关键字段
- 使用明显不合法的数据形状掩盖真实边界问题

### 7.4 测试替身不得渗透为生产默认路径

测试 helper、fake loader、mock adapter 不应成为生产默认分支，也不应被文档描述为真实默认行为。

## 8. 文档与验收表述规则

推荐表述：

- “MOCK 验证通过，真实链路验证仍待完成。”
- “`mock_live` 生命周期模拟通过，不代表真实券商执行已完成。”
- “离线模拟数据路径通过，真实市场数据源未在本次验收范围内。”
- “测试使用 fixture / fake loader，结论仅覆盖策略逻辑与流程编排。”

禁止表述：

- “已通过”但不说明是 MOCK 还是真实链路
- “实盘已完成”但证据仅为 `mock_live`
- “真实数据已验证”但实际使用了 `--mock`
- “live 可用”但实际只验证了 preview、paper 或 mock 路径

## 9. 文档维护要求

- 本文档是项目级 MOCK 顶层规范
- README 和 USER_MANUAL 中涉及 MOCK 的用户可见说明，应与本文档一致
- 历史 spec / plan / report 可保留，但不得覆盖当前规则
- 当新增运行时 MOCK 入口或新的测试替身体系时，应同步更新本文档

## 10. 自检清单

在新增或修改 MOCK 相关能力时，至少确认：

- [ ] 这是显式 MOCK，而不是静默 fallback
- [ ] 文档明确区分了 MOCK 与真实链路
- [ ] 验收结论没有把 MOCK 成果误报为真实能力
- [ ] 测试替身命名清晰，边界清晰
- [ ] fixture / fake / mock 数据尽量贴近真实契约
- [ ] `mock_live` 与真实实盘语义没有混淆
- [ ] README / USER_MANUAL 的相关表述仍与当前代码一致
