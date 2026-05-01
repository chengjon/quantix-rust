# TdxQuant 接入 quantix-rust 的必要性与可行性评估

日期：2026-04-28

## 1. 评估范围

本评估基于两类输入：

- `TdxQuant` 的功能地图：`/opt/iflow/TdxQuant/docs/TdxQuant_Project_Function_Map.md`
- `quantix-rust` 当前已存在的桥接、数据源、监控、执行与架构文档

重点回答四个问题：

1. `quantix-rust` 是否有必要接入 `TdxQuant`
2. 如果接入，应该接哪些能力，不该接哪些能力
3. 从当前架构看，接入是否可行，哪种方式最稳
4. 对 `TdxQuant` 本身，哪些设计再补一层，会更适合被上层系统复用

## 2. 总结结论

我的结论不是“全量接入”或“完全不接”，而是：

- `quantix-rust` 对 `TdxQuant` 有明确的选择性接入需求
- 最值得接入的是：
  - 通达信原生公式能力
  - 自定义板块治理能力
  - 持久订阅 session 之上的任务化订阅能力
- 不建议当前直接接入的是：
  - 桌面自动化交易执行主线
  - 把 `TdxQuant` 当成新的交易中心或执行状态中心
- 最可行的方式不是把 `TdxQuant` 直接嵌进 Rust 进程，而是把它放在现有 Windows bridge 边界之后，通过稳定 HTTP / JSON / JSONL 协议供 `quantix-rust` 调用

一句话判断：

`TdxQuant` 值得接，但应作为“Windows 侧通达信能力提供者”被选择性吸收，而不是作为新的上层架构中心整体并入。

## 3. 为什么有接入需要

从 `quantix-rust` 当前能力看，它已经具备：

- 多数据源行情与 K 线能力
- `BridgeTdxSource` 这类跨 `WSL2 -> Windows` 的桥接模式
- 自选、监控、K 线聚合、策略、风控、执行内核
- `QMT` bridge 和受能力门控的 `qmt_live` 路径

因此，`quantix-rust` 并不缺“再来一套基础行情/K线读取”。

真正存在补位价值的，是 `TdxQuant` 与通达信客户端深度耦合的三类能力：

### 3.1 通达信原生公式体系

这是 `TdxQuant` 当前最有差异化价值的能力之一。

`quantix-rust` 当前有自己的指标、分析与策略体系，但没有看到已经成型的“通达信公式读写、指标计算、选股计算、专家公式计算、批量选股”这一整套原生兼容层。

这意味着：

- 如果希望复用大量既有通达信公式资产，`TdxQuant` 有明显价值
- 如果希望把“公式选股结果”直接灌入 `watchlist`、`monitor`、`strategy`，也有很强的集成空间

### 3.2 自定义板块治理

`TdxQuant` 的板块能力包括：

- 读取自定义板块
- 创建 / 删除 / 重命名板块
- 清空板块
- 写入板块成分

而 `quantix-rust` 当前更多是“行业/概念/强弱板块分析”，不是“通达信客户端板块资产治理”。

如果上层工作流里需要：

- 把筛选结果落回通达信板块
- 让通达信客户端与 `quantix-rust` 自选/监控互相同步
- 让研究结果沉淀到交易员日常使用的通达信环境

那么这部分就很值得接。

### 3.3 持久订阅 session 及其任务化封装

`TdxQuant` 文档里已经明确：

- 底层持久订阅 session 已经做出来
- 但更适合日常调用的 task / daemon 入口还在继续产品化

这点和 `quantix-rust` 现有监控、watchlist、实时管线是天然可以对接的。

特别是如果 `TdxQuant` 后续能稳定提供：

- 长时运行的订阅任务
- 统一事件格式
- `JSONL` / `CSV` 输出
- 可恢复、可停止、可观测的 session 生命周期

那么它非常适合成为 `quantix-rust` 的“Windows 侧实时事件输入器”。

## 4. 哪些能力不值得现在接

### 4.1 基础行情 / K 线读取，不应作为第一优先级

原因不是它没价值，而是 `quantix-rust` 已经有：

- `TdxSource`
- `BridgeTdxSource`
- 东方财富 / WebSocket / 本地文件等多条数据路径

所以单纯“再多一个行情来源”不是最强痛点。

只有在以下情况下，它才值得优先接：

- `TdxQuant` 的行情质量或稳定性显著优于现有桥
- 它能带来现有 bridge 没有的通达信字段
- 它能统一行情、公式、板块、订阅，减少 Windows 侧重复实现

### 4.2 桌面自动化交易主线，不建议直接并入

这是我最明确的保留意见。

`quantix-rust` 当前架构已经把执行生命周期、风控、`runtime.db`、order / order_event 的状态所有权留在本地内核。

而 `TdxQuant` 的桌面自动化交易主线更像：

- 通达信客户端辅助执行层
- 券商桌面自动化适配层
- 面向人机协同的交易辅助层

这两者并不天然冲突，但边界很容易被写坏。

如果现在把 `TdxTradeManager` 这类能力直接接入 `quantix-rust` 主执行路径，会立刻引入几个问题：

- 执行状态究竟由谁说了算
- 失败重试和订单查询语义如何统一
- GUI 自动化失败如何映射到现有订单状态机
- 风控前置、风控回写、审计落库谁负责

因此当前更合理的策略是：

- 可以把这条线视为未来独立研究方向
- 但不应作为本轮接入重点
- 更不应让它绕过 `ExecutionKernel` 和现有 broker adapter 边界

## 5. 能力映射与接入优先级

| TdxQuant 能力 | quantix-rust 当前状态 | 是否有接入需要 | 优先级 | 建议方式 |
|---|---|---:|---:|---|
| `market/meta` 行情与静态资料 | 已有多数据源与 bridge | 有，但不强 | 中 | 作为现有 Windows bridge 的可替换 provider |
| `formula` 公式读写/指标/选股 | 当前无明显等价通达信原生层 | 很强 | 高 | 新增桥接协议或 CLI JSON 协议 |
| `block` 自定义板块治理 | 当前更偏分析，不偏客户端板块资产 | 很强 | 高 | 新增板块同步与落板块任务 |
| `runtime subscription session` | 当前有监控/WS/采集，但不是通达信 session 产品化路径 | 较强 | 高 | 订阅任务 -> JSONL 事件流 -> monitor/watchlist |
| `report/catalog/task` | `quantix-rust` 也有自己的 CLI/handler/task 体系 | 有，但应选择性复用 | 中 | 只复用稳定任务，不复用治理层 |
| `financial/transaction` | 本项目已有其他财经数据路径 | 有限 | 中低 | 仅在字段独特或口径必要时考虑 |
| `desktop trade` | 当前已存在 QMT bridge + 执行内核 | 不建议当前接 | 低 | 暂不进入主执行链路 |

## 6. 从当前架构看，最可行的接入方式

### 6.1 推荐方案：把 TdxQuant 放在现有 Windows bridge 边界之后

这是我最推荐的方案。

原因很直接：

- `quantix-rust` 已经接受“Windows 侧承接专有能力，Rust 侧保持内核所有权”的架构
- `BridgeHttpClient` 和 `BridgeTdxSource` 已经证明这种边界在当前仓库里是成立的
- Rust 侧当前真正依赖的是稳定协议，不依赖 Windows 侧内部实现细节

因此，最好的落点不是：

- Rust 直接 import Python
- Rust 直接和 `TdxQuant` 内部对象耦合

而是：

- Windows 侧 bridge 把 `TdxQuant` 当作能力提供者
- 对 `quantix-rust` 暴露稳定 API
- Rust 侧只消费标准化结果

这能保持几个重要优点：

- 语言边界清晰
- Windows 依赖不泄漏到 Rust 主运行时
- 未来可以替换 provider，而不必改 Rust 上层
- 更适合把订阅、公式、板块做成独立 contract test

### 6.2 备选方案：通过 CLI + JSON / JSONL 协议对接

如果 `TdxQuant` 当前最稳定的是 CLI，而不是长驻服务，那么第二可行方案是：

- Rust 或 shell 层调用 `tdxquant ...`
- 强制要求机器可读输出
- 批处理走 `JSON`
- 长时订阅走 `JSONL`

这个方案的优点是：

- 接得快
- 对 `TdxQuant` 入侵小
- 适合先验证公式、板块、报表、任务类能力

缺点也很明显：

- 进程管理更复杂
- 长时 session 的可观测性比服务方式弱
- 错误重试、健康检查、能力发现不如 HTTP 服务自然

因此我认为：

- 它适合做第一阶段 PoC
- 但不适合作为长期唯一集成方式

### 6.3 不推荐方案：直接库级嵌入 / FFI / PyO3 深耦合

这条路理论上可做，实践上我不建议现在做。

原因：

- `TdxQuant` 仍在开发中，内部结构和边界未完全稳定
- Windows 专有依赖、桌面自动化依赖、Python 运行时管理会显著放大集成复杂度
- `quantix-rust` 当前已经有很明确的“bridge is boundary”架构方向，没有必要反向破坏这条边界

除非未来出现强需求，例如：

- 必须毫秒级同进程调用某个公式内核
- 该能力无法通过服务/CLI 暴露

否则不建议走这条路。

## 7. 我对 TdxQuant 的核心改进建议

如果你的目标是把 `TdxQuant` 做成“可被 Python / Rust 上层量化系统复用的本地工具层”，那我建议你优先补的是“可集成性”，而不是继续横向加功能。

### 7.1 把“能力层”和“入口层”彻底分开

当前文档里同时出现了：

- `TdxApiManager`
- `TdxTaskManager`
- `report`
- `catalog`
- `TdxTradeManager`

这说明你已经意识到有多条能力线，但我建议再进一步：

- 核心能力层：行情、公式、板块、订阅、客户端联动、桌面交易
- 产品入口层：task / report / catalog

上层项目最好依赖“核心能力 contract”，而不是依赖入口层实现细节。

否则后面最容易出现的问题就是：

- 上层本来只想调用一个稳定能力
- 结果被 `catalog`、`preset`、`bundle`、输出格式、目录布局绑死

### 7.2 先定义稳定机器协议，再谈跨项目复用

这是最关键的一条建议。

如果你真想让 Rust 项目稳定接入 `TdxQuant`，至少要先把下面这些东西固定下来：

- 错误码
- JSON 响应结构
- `JSONL` 事件结构
- 任务退出码
- 字段命名规范
- 时间格式
- 符号格式
- 复权 / 周期 / 市场枚举口径

尤其是订阅任务，建议尽快固定：

- `session_id`
- `subscription_id`
- `event_type`
- `symbol`
- `event_ts`
- `source_ts`
- `payload`
- `sequence`

否则一旦上层开始接入，后续每改一次输出，都会造成兼容性成本。

### 7.3 增加能力发现机制

建议 `TdxQuant` 明确提供一个能力清单接口或命令，例如：

- 当前启用了哪些能力
- 当前运行模式是什么
- 哪些接口是稳定的
- 哪些接口仍是实验性的
- 当前是否支持 `formula` / `block` / `subscription` / `desktop_trade`

这样上层系统可以做：

- 启动前 capability probe
- 运行时 degrade
- 明确报错而不是调用后才发现不可用

### 7.4 把“订阅底层能力”尽快产品化成长期稳定契约

你文档里已经识别出这个方向，我认为判断是对的。

这块如果做成，价值会非常高。

建议补齐的不是更多“功能点”，而是：

- `start / stop / status / list`
- 优雅退出
- session 恢复语义
- 事件落盘策略
- 背压与缓冲策略
- 丢事件后的告警语义
- 运行统计

如果这些先补齐，`TdxQuant` 会非常适合做监控系统的 Windows 侧实时入口。

### 7.5 对桌面自动化交易能力做更强的边界隔离

建议把桌面自动化交易能力视为单独的高风险 capability，而不是和普通查询/公式/板块能力混在一起。

至少应该在文档和接口层明确区分：

- 只读能力
- 会改本地客户端状态的能力
- 会触发实盘副作用的能力

如果后续你想给上层系统接入，这种 capability 分级会非常重要。

### 7.6 为上层系统设计“结果可复验”的测试夹具

建议尽早给 `TdxQuant` 增加：

- 固定样例输入
- 固定 JSON 输出样例
- 契约测试
- 假数据 / 回放数据模式

这样外部项目在对接时才能：

- 不依赖真实客户端做每次联调
- 用 contract test 验证升级是否兼容

## 8. 对 quantix-rust 的实际接入建议

### 8.1 第一阶段：不要急着全接，先做一个窄 PoC

我建议第一阶段只验证下面三条中的一条或两条：

- 公式计算/选股
- 自定义板块写入
- 订阅任务输出事件流

不要一上来就同时接：

- 行情
- 公式
- 板块
- 任务
- 桌面交易

那样会把“架构验证”和“能力扩展”混在一起。

### 8.2 最推荐的第一个 PoC：公式选股 -> watchlist / block

这是当前性价比最高的切入点。

原因：

- 差异化强
- 副作用可控
- 结果容易验证
- 与 `quantix-rust` 的 watchlist、monitor、strategy 都有自然连接点

一条很自然的闭环是：

1. `TdxQuant` 执行公式选股
2. 输出标准化股票列表
3. `quantix-rust` 导入为 watchlist
4. 可选地同步写回通达信自定义板块
5. 后续监控、分析、策略都基于这份结果继续工作

### 8.3 第二阶段：订阅任务 -> JSONL -> monitor

如果第一阶段稳定，再推进订阅能力整合。

推荐形态：

- `TdxQuant` 长时运行订阅任务
- 持续输出标准化 `JSONL`
- `quantix-rust` 只负责消费事件并进入监控/分析链路

这样能避免：

- Rust 直接接管 Windows session 生命周期
- 跨语言共享长连接状态
- 将高波动实时逻辑塞进现有执行内核

### 8.4 第三阶段：再考虑是否需要板块与报表层深度复用

这一阶段不是不能做，而是不应过早做。

原因是：

- `quantix-rust` 自己已经有 CLI / handler / task 体系
- `TdxQuant` 也有 task / report / catalog 入口

如果太早复用入口层，会把两个系统的治理层耦合在一起。

所以更好的顺序是：

- 先复用能力
- 再决定是否复用入口

## 9. 最终建议

### 9.1 我建议接

但只接下面这些方向：

- 公式能力
- 自定义板块能力
- 任务化订阅能力

### 9.2 我不建议现在接

- 桌面自动化交易主线进入 `quantix-rust` 主执行路径
- 直接库级嵌入 Python / FFI 深耦合
- 让 `TdxQuant` 的 `catalog/report` 反向主导 `quantix-rust` 的入口结构

### 9.3 我建议你的 TdxQuant 先补齐

- 稳定 JSON / JSONL contract
- 统一错误模型
- 统一输出结构
- 能力发现机制
- 订阅任务生命周期语义
- capability 分级

## 10. 最后的判断

如果把这件事看成“把 `TdxQuant` 全量并入 `quantix-rust`”，我不赞成。

如果把它看成“让 `TdxQuant` 成为 `quantix-rust` 的 Windows 侧通达信能力 provider，并优先贡献公式、板块、订阅三类差异化能力”，我认为：

- 有必要
- 可行
- 而且很值得做

我建议的总体路线是：

1. 保持 `quantix-rust` 的执行内核、风控、状态所有权不变
2. 让 `TdxQuant` 在 Windows 侧输出稳定 contract
3. 先做窄能力 PoC
4. 通过 bridge 或 CLI 协议接入
5. 等 contract 稳定后，再扩大接入面

这条路线兼顾了当前项目架构现实、TdxQuant 仍在开发的事实，以及后续多项目复用的长期目标。
