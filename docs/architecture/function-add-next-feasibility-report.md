# `function-add-next.md` 可行性分析报告

生成日期：2026-05-03

> 状态源说明：本文是候选能力可行性分析，不作为功能状态注册表。
> 候选功能是否纳入“已设计/待实现”、证据和边界，以根目录 [`FUNCTION_TREE.md`](../../FUNCTION_TREE.md) 的状态注册表行为准。

## 1. 结论摘要

`docs/architecture/function-add-next.md` 里的 24 个候选功能，**多数并非纯绿地需求**，但 v1 报告对若干现有基础的成熟度判断偏乐观，尤其是：

- 回测引擎当前是**简单日期循环式回测**，不是事件驱动撮合框架。
- 交易日历已有 A 股时段与节假日模型，但**没有 T+1 结算抽象**。
- `load_holidays_for_year()` 仍是空桩，当前节假日能力主要依赖外部 JSON。
- 系统没有全局紧急停止 / kill switch，只存在**局部控制能力**，例如 algo 级 `pause/resume`。
- `StrategyRegistry` 不是“多策略硬编码分发”，而是**单策略 `ma_cross` 硬编码**。
- 通知渠道扩展存在明显空桩，当前只实现了部分 sender。

修正后，整体判断如下：

- **高可行**：现有架构能直接承接，主要是补规则、补状态、补 CLI 和持久化。
- **中可行**：能做，但要补服务边界、统一状态模型，或新建跨模块协调层。
- **低可行**：会把项目从当前 CLI-first 单用户系统推向平台化、多用户化或高频化。

## 2. 分析依据

本报告基于以下内容交叉得出：

- 目标文档：`docs/architecture/function-add-next.md`
- 当前能力边界：`README.md`、`FUNCTION_TREE.md`
- GitNexus 图谱与执行流查询
- 关键源码抽样：
  - `src/core/trading_calendar.rs`
  - `src/analysis/backtest.rs`
  - `src/io/validation.rs`
  - `src/anomaly/mod.rs`
  - `src/account/models.rs`
  - `src/account/router.rs`
  - `src/risk/service.rs`
  - `src/stop/service.rs`
  - `src/trade/fees.rs`
  - `src/trade/reporting.rs`
  - `src/monitoring/health.rs`
  - `src/monitoring/performance_monitor.rs`
  - `src/monitoring/notification.rs`
  - `src/monitoring/notification/service.rs`
  - `src/execution/qmt_live_adapter.rs`
  - `src/execution/qmt_task_submit_service.rs`
  - `src/execution/reconciliation.rs`
  - `src/strategy/trait_def.rs`
  - `src/strategy/registry.rs`

补充说明：

- 本次分析前已执行一次 `gitnexus analyze` 刷新本地图谱。
- 本版报告吸收了 `docs/architecture/function-add-next-feasibility-report-v2.md` 中经源码核验成立的审核意见。

## 3. 当前架构判断

### 3.1 已建立的基础

1. **执行主线已经成型**
   - 项目已围绕 `strategy request -> execution kernel -> adapter -> runtime store -> reconciliation` 建立执行闭环。
   - `paper`、`mock_live`、`qmt_live`、partial fill、`pending_cancel`、`unknown` 恢复等语义已经被认真建模。
   - `reconciliation` 不只是占位模块，而是带有报告对象、QMT live 恢复分支和 optimistic-lock 更新路径的实用引擎。

2. **风控、止损、监控、审计事件都有真实落点**
   - `risk`、`stop`、`monitoring` 都有独立服务层和持久化。
   - `stop_history`、`risk events`、`execution events`、`monitor events` 说明项目已经具备跨域审计素材，只是尚未统一成一套 operator-facing 视图。

3. **数据与研究侧有可复用组件**
   - `anomaly` 不是单一算法文件，而是完整 Isolation Forest 异常检测流水线。
   - `account router` 已实现 4 种分配策略，后续做组合/资金分配时可以直接复用 allocator 框架。

### 3.2 当前硬边界

1. **仍然是 CLI-first / 单用户 / 单机模型**
   - 没有统一 API、身份、授权、会话、租户边界。
   - 因此 RBAC、多人协作审计、平台级跟单都不自然。

2. **策略加载模型非常薄**
   - `Strategy` trait 只有 `init / on_bar / finish` 三个生命周期点。
   - `StrategyRegistry` 当前只有 `ma_cross` 一个硬编码入口，外部 Python / WASM 不能视为“补一个适配器”。

3. **回测与执行主线是分离的**
   - `src/analysis/backtest.rs` 采用按日期循环、直接撮合的轻量模型。
   - 它与 execution/risk/live request 生命周期没有统一接口，也没有共享订单状态机。

4. **时间语义还不完整**
   - 交易时段、节假日、调休工作日已有基础。
   - 但没有 T+1、结算、交割、可交易持仓可用性等语义。

## 4. 逐项可行性评估

可行性定义：

- **高**：现有架构可直接承接，主要是增量开发。
- **中**：需要补状态模型、服务边界或调用链，属于中型重构。
- **低**：需要新增平台层或显著改变产品形态。

| 编号 | 功能 | 当前基础 | 可行性 | 主要缺口 | 建议 |
| --- | --- | --- | --- | --- | --- |
| 1 | 数据质量校验与修复 | `src/io/validation.rs` 已有 K 线校验；`anomaly` 已是完整 Isolation Forest 流水线 | 高 | 缺自动修复、跨源一致性校验、复权因子纠偏流程 | 建议尽快做 |
| 2 | 交易日历与时间轴管理 | `src/core/trading_calendar.rs` 已有 A 股交易时段、节假日和调休工作日模型，但依赖外部 JSON | 中 | 无 T+1 / 结算抽象；`load_holidays_for_year()` 为空；缺统一时间轴 | 高优先，但不是低成本补接口 |
| 3 | L2 / 逐笔 / 订单簿 | 目前无成熟 order book 数据结构；仅有快照/行情基础 | 低 | 新数据模型、新存储、新因子、新执行假设 | 延后 |
| 4 | 回测引擎独立性 | `src/analysis/backtest.rs` 已接通 CLI，但本质是 simple date-loop backtest | 中偏低 | 无事件调度、订单簿、部分成交、请求生命周期；与 execution/risk/live 完全分离 | 可做，但别抢主线 |
| 5 | 参数调优与过拟合防护 | 已有回测入口与指标流水线 | 中 | 缺优化器、滚动验证、实验管理、稳定性报告 | 适合第二阶段 |
| 6 | 多策略组合与资金分配 | `account` 已支持多账户、4 种分配策略和订单拆分 | 中 | 缺组合层 allocator、相关性约束、策略冲突解析 | 先做轻量版 |
| 7 | 撤单与补单机制 | `cancel_order`、partial fill、`pending_cancel`、`reconciliation` 已存在 | 中偏高 | 缺“补单/改单/价格调整”策略层；缺对 broker 约束的统一抽象 | 高优先 |
| 8 | 交易成本实时预估 | `src/trade/fees.rs` 已有税费模型；回测已有 commission/slippage | 高 | 缺盘口冲击、账户维度费率、请求级实时估算 | 适合增量补强 |
| 9 | 紧急停止与熔断机制 | 已有 `risk`、`stop`、`execution daemon`、runtime store；algo 侧已有局部 `pause/resume` | 中偏高 | 无系统级 kill switch；缺跨 `strategy/execution/risk` 的全局协调层 | 安全上高优先，但需新模块 |
| 10 | 事前/事中/事后三层风控 | 已有买前检查、止损、快照同步与风险事件 | 中 | 缺明确 stage hook、提交流程中的二次否决点、事后联动动作 | 适合沿执行主线完善 |
| 11 | 净值回撤与止损体系 | `performance_monitor` 已有 drawdown；`stop`/`risk` 已有规则基础 | 高 | 缺账户级/策略级权益真值接入和自动动作联动 | 应优先 |
| 12 | 关联账户风控 | `account group` 与 router 已有基础 | 中 | 缺统一多账户风险快照与“实控关系”模型 | 后置于单账户风控 |
| 13 | 状态监控与告警增强 | `health`、`metrics`、`alert`、`notification` 已成体系 | 高 | 缺关键链路耗时、组件健康接入、执行链路指标标准化 | 适合持续增强 |
| 14 | 完整操作审计日志 | `stop_history`、`risk events`、`execution events`、`monitor events` 已存在 | 高 | 缺统一审计表、关联 ID 和跨域查询视图 | 很适合近期做 |
| 15 | 回放与复盘工具 | 已有 execution snapshot、backtest report、monitor snapshot | 中偏低 | 缺统一事件时间线、逐笔回放器、operator-facing 复盘界面 | 先做文本版 |
| 16 | 数据备份与灾难恢复 | 本地状态文件、runtime snapshot、mirror rebuild 已有局部基础 | 中 | 缺数据库备份脚本、恢复演练、跨存储一致性流程 | 可做，但偏运维工程 |
| 17 | 权限与角色管理 | 当前是单用户 CLI 工具，不存在真正会话/身份边界 | 低 | 需要账号、认证、授权、审计归属模型 | 除非产品转型，否则不建议现在做 |
| 18 | 敏感信息加密 | 已有 env 配置与部分脱敏检查脚本 | 中 | 缺 secrets provider、静态配置加密、密钥轮换 | 可以做轻量版本 |
| 19 | 外部 Python / WASM 策略 | `Strategy` trait 简洁，但 registry 只有 `ma_cross` 一个硬编码入口 | 低 | 需要重建策略加载模型、运行时边界、ABI/IPC 契约 | 只能作为独立专题做 |
| 20 | 通知渠道扩展 | 已实现 Desktop/Webhook/Log/企业微信/飞书 5 个 sender；Telegram/Discord/Slack/钉钉/Pushplus/Email 仍是空桩 | 高 | 6 个主流渠道要从零实现 sender、模板、重试/限流 | 可做，但不应占据核心排期 |
| 21 | 报表与绩效分析 | 已有 `performance`、`trade/reporting`、backtest report 存储 | 高 | 缺日/周/月聚合、暴露分析、导出格式 | 适合第二优先级 |
| 22 | 一键跟单 / 信号跟车 | 多账户与执行请求机制可复用一部分 | 低 | 缺主从关系、延迟容忍、冲突处理、合规隔离、回滚语义 | 当前阶段不建议 |
| 23 | 白名单 / 黑名单股票管理 | 已有 watchlist、risk 行业 blocklist、解析器 | 高 | 缺 stock-level rule schema 与执行前检查点 | 低成本高收益 |
| 24 | 模拟环境与实盘快速切换 | `paper` / `mock_live` / `qmt_live` / `AccountType` 已具备 | 高 | 缺标准化 promotion workflow、环境检查和回退流程 | 很适合沿现有主线做 |

## 5. 建议优先级重排

`function-add-next.md` 原始优先级方向大致合理，但结合源码现状，建议按以下三层重新组织。

### 5.1 第一层：直接服务当前交易主线

这些功能最能提升“实盘前可控性”和“执行主线闭环质量”：

1. 数据质量校验与修复
2. 交易日历 / T+1 / 时间轴统一
3. 撤单 / 补单 / 对账收口
4. 净值回撤控制
5. 完整审计日志
6. 模拟环境到 `mock_live` / `qmt_live` 的切换流程
7. 关键链路监控与告警增强
8. 最小可用的系统级紧急停止能力

说明：

- 我接受“通知渠道补齐不应占据第一层核心排期”这一审核意见。
- 我不完全接受“紧急停止应降到第二层”。基础薄弱是事实，但在 live promotion 前，它仍是安全必需项，只是要按“新建全局协调层”来估算，而不是按增量修补来估算。

### 5.2 第二层：能力增强，但不应抢主线资源

1. 回测引擎重构
2. 参数调优与过拟合防护
3. 多策略组合与资金分配
4. 关联账户风控
5. 交易成本实时预估
6. 报表与绩效分析
7. 股票白名单 / 黑名单
8. 数据备份与恢复
9. 敏感信息加密
10. 通知渠道补齐

### 5.3 第三层：应作为独立产品方向

1. L2 / 逐笔 / 订单簿
2. 可视化回放 / 复盘界面
3. RBAC / 角色权限
4. 外部 Python / WASM 策略平台
5. 一键跟单 / 信号跟车

这些功能会改变项目边界：

- 从 CLI 工具走向平台
- 从单用户走向多用户
- 从中低频 A 股执行走向更高频或更复杂 broker 语义

## 6. 原始 24 项之外，建议补充的 3 个强关联能力

审核意见提出的这三项，经源码核验后我认为成立；如需纳入后续设计项，应登记到 `FUNCTION_TREE.md` 的状态注册表：

1. **策略生命周期管理**
   - 当前 `Strategy` trait 只有 `init / on_bar / finish`。
   - 如果后续要做多策略运行、紧急停止、暂停恢复，这个生命周期边界太薄。

2. **配置热重载**
   - 风控规则、止损参数、通知配置在 daemon 场景下都适合支持热更新。
   - 否则 operator 每次改参数都要重启服务，运维成本高。

3. **优雅停机**
   - 对 daemon / live-ready 场景非常重要。
   - 最少要覆盖：停止接收新信号、等待在途订单、持久化快照、结构化退出。

## 7. 推荐实施路线

### Phase A：2-4 周，围绕“交易主线稳态化”

- 数据校验接入采集/同步入口
- 统一交易日历 / T+1 / 时间轴语义
- 统一审计日志视图
- 账户级/策略级回撤风控
- `paper -> mock_live -> qmt_live` promotion checklist
- 最小可用的全局 kill switch
- 关键链路告警，不要求一次补齐全部 sender

### Phase B：4-8 周，围绕“策略与风控能力增强”

- 撤单与补单策略化
- 多策略资金路由初版
- 股票白名单 / 黑名单
- 报表聚合与绩效导出
- 备份与恢复脚本 + 演练文档
- 通知 sender 扩展
- 配置热重载与优雅停机

### Phase C：专题化推进，不与主线混做

- 策略生命周期重构
- 外部策略平台（先外部进程，再考虑 WASM）
- 回放 / 复盘时间线
- 多账户统一风险域
- 更高级的 live broker 流程自动化
- RBAC / 跟单 / 高频数据能力

## 8. 最终判断

如果问题是“这些功能能不能加进这个项目”，答案是：

- **能加，而且其中相当一部分已经有可复用基础。**

如果问题是“是否应该按文档顺序一股脑加入”，答案是：

- **不建议。**

更合理的做法是把候选功能拆成两类：

1. **沿现有执行主线继续闭环的功能**
   - 这类应该优先推进。
2. **会把项目推向平台化 / 多用户化 / 高频化的功能**
   - 这类必须单独立题、单独设计、单独排期。

对当前仓库最合适的策略不是“功能越多越好”，而是：

- 先把 `execution + risk + monitor + audit + promotion workflow` 打磨到可重复操作、可恢复、可追责；
- 再扩展研究型和平台型能力。
