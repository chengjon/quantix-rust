# Quantix 架构与 CLI 对齐方案

日期：2026-04-17

## 1. 目标

基于以下三份文档，对当前 `quantix-rust` 做一次务实对齐：

- `docs/豆包DS架构设计建议.md`
- `docs/CLI命令手册优化建议.md`
- `docs/缺口和对齐优化建议.md`

目标不是一次性把系统重写成多 crate 插件内核，而是在当前单 crate、已具备 CLI/回测/执行/风控雏形的基础上：

1. 吸收仍然有价值的设计原则。
2. 识别当前实现与理想架构的真实差距。
3. 规划一条“可连续交付”的演进路线。

## 2. 当前实现的客观状态

从源码现状看，当前系统已经具备：

- 内置策略 trait 与若干内置策略实现
- 基础回测引擎与绩效报告
- 模拟交易、风控、止损、监控、自选池、市场/新闻/基本面分析
- execution bridge / QMT live 相关桥接能力
- 数据源实现集合（TDX / EastMoney / WebSocket / 导入）

但它**并不是**设计文档中的这套系统：

- 不是 `quantix-common / quantix-core / quantix-data / quantix-strategy / quantix-backtest ...` 的多 crate 内核化结构
- 没有 Wasm / `abi_stable` 插件系统
- 没有通用的策略插件生命周期管理命令
- 没有统一的 `backtest run/optimize/compare`
- 没有独立的 `performance` 领域命令
- 没有面向多数据源切换的统一 CLI 管理面

因此应避免直接照搬“大一统重构方案”，否则风险会高于收益。

## 3. 哪些建议值得吸收

以下建议仍然正确，而且与现有代码方向兼容：

- **模块边界清晰化**：即使继续保留单 crate，也应逐步把策略、回测、执行、绩效、数据源的 API 边界做清楚。
- **策略标准化**：策略不应只靠硬编码字符串匹配；需要更明确的策略目录、参数规范、实例化入口。
- **回测真实度增强**：当前回测已有滑点和手续费，但还缺 A 股规则补全、参数扫描、结果对比。
- **数据源抽象统一**：当前只有局部 trait（例如 anomaly 的 `DataSource`、strategy runtime 的 `StrategyBarLoader`），缺统一上层抽象。
- **模拟 / 实盘语义分离**：现有 `trade` 与 `execution bridge qmt-*` 已经有分工，但 CLI 语义仍可更清晰。
- **CLI 命名收敛与实验能力降噪**：比起大规模 rename，更适合先加 alias、隐藏未支持项、补兼容迁移层。

## 4. 不应立即照搬的建议

以下项目暂不建议直接实施为当前阶段目标：

- 一次性拆成多 crate 微内核架构
- 立即引入 Wasm 插件 ABI
- 立即把 `trade` 全量重命名成 `paper`，再新增完整 `live`
- 一次性重写所有三级命令层级

原因：

- 当前代码已经形成一套可运行的 CLI 和 handler 结构
- execution / monitor / strategy runtime 已经有一部分状态持久化和运行时模型，贸然整体迁移会带来较大回归风险
- 用户已经围绕现有命令树建立了使用习惯和手工测试路径

## 5. 当前差距清单

### 5.1 架构层

1. **策略加载方式仍然是内置注册制**
   - 当前 `StrategyRegistry` 主要按名称构造内置 evaluator。
   - 缺插件生命周期、插件元数据、插件启停与兼容性检查。

2. **数据源抽象不统一**
   - `sources/` 下有多种实现，但缺统一 CLI 入口管理默认源、测试连接、采集入口。

3. **回测入口不足**
   - `analyze backtest` 更像“查看已有报告”，不是“发起完整回测任务”。
   - 缺参数优化、回测对比、结果持久化约定。

4. **绩效域存在实现、缺独立产品面**
   - `analysis/performance.rs` 已有丰富指标计算。
   - 但 CLI 没有把这些能力组织成 `performance` 命令族。

5. **模拟交易 / 桥接执行边界已出现，但命令语义仍偏技术化**
   - `trade` 侧偏模拟账户
   - `execution bridge qmt-*` 侧偏桥接/实盘
   - 用户角度仍不够清晰

### 5.2 CLI 层

1. `task` 中明确“不支持”的写能力仍直接暴露在默认帮助中
2. `analyze candle-pattern` 参数过载，学习成本偏高
3. `analyze screener preset-list` 命名偏机械，不够自然
4. `execution bridge qmt-*` 结构暴露了技术实现细节，缺更业务化的兼容入口
5. `account split` 与 `algo plan` 的语义边界需要进一步梳理

## 6. 建议的分阶段实施方案

### Phase A：低风险 CLI 收敛

目标：先优化可发现性和命名体验，不破坏现有脚本。

建议项：

- 给高频命令补 alias
- 隐藏明确不支持的实验性写操作
- 为过于技术化的命令增加更易懂的兼容入口
- 更新 HTML 手册与帮助输出测试

本次已落地的第一步：

- `analyze screener preset-list` 新增别名 `presets`
- `task add` 从默认帮助中隐藏
- `task start --daemon` 从默认帮助中隐藏

### Phase B：补足“架构最短板”的 CLI 面

目标：不先做插件 ABI，先补用户真正缺失的控制面。

建议新增：

- `data source list/add/set-default/test`
- `backtest run/report/compare`
- `performance report/compare`

说明：

- 这些命令可以先直接复用当前单 crate 内的服务与 store，不需要先拆多 crate。
- 优先交付“统一入口”，再逐步把内部实现抽象化。

### Phase C：策略目录化，而非立刻插件化

目标：在真正引入 Wasm 前，先把“策略实例管理”做出来。

建议新增或重构：

- `strategy create`
- `strategy update`
- `strategy delete`
- `strategy list/show`

设计原则：

- 第一阶段允许策略来源仍是内置 registry
- 但实例定义、参数模板、运行目标要进入统一配置存储
- 等实例生命周期稳定后，再演进到插件来源

### Phase D：执行命令语义整理

目标：让模拟交易和桥接执行的边界更清楚，但不做破坏式重命名。

建议：

- 保留 `trade` 作为模拟交易
- 保留 `execution` 作为执行桥接
- 逐步为 `execution bridge qmt-*` 提供更友好的兼容别名与说明文案
- 在成熟之后，再决定是否引入 `live` 作为更高层别名

### Phase E：插件化预备工作

目标：先做插件化前置条件，而不是直接上 Wasm。

前置条件：

- 抽出稳定的策略输入/输出模型
- 抽出策略元数据与参数 schema
- 为策略执行建立统一 `StrategyDescriptor` / `StrategyFactory`
- 让 registry 不再只有字符串 match

只有这些完成后，再评估：

- Wasm 插件
- `abi_stable` 动态库
- 插件签名 / 沙箱 / 版本兼容

## 7. 建议的近期实施顺序

### 第一批（现在就能做）

1. CLI 别名与帮助降噪
2. `backtest` 命令族设计草案
3. `data source` 命令族设计草案
4. 更新命令手册

### 第二批（下一阶段）

1. 实现 `backtest run`
2. 实现 `performance report/compare`
3. 实现 `strategy create/update/delete`

### 第三批（再下一阶段）

1. 实现 `data source` 管理
2. 补 `execution` 兼容别名
3. 评估 `account split` 与 `algo plan` 的合并策略

## 8. 核心原则

- **先收口 CLI，后抽象内核**
- **先统一入口，后替换内部实现**
- **先实例化管理，后插件化加载**
- **优先向后兼容，避免大规模破坏性 rename**

## 9. 结论

`豆包DS架构设计建议.md` 提供的是一个“终局目标图”；当前系统实现更接近“已经长出很多能力的单体 CLI 应用”。

因此最合理的路线不是直接重做，而是：

1. 保留当前可运行骨架
2. 吸收其中关于策略标准化、回测真实度、数据源抽象、绩效对比的有效部分
3. 通过分阶段 CLI 和内部接口演进，把系统逐步推向更稳的架构形态

这条路径现实、连续可交付，也更符合当前代码基础。
