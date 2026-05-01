# quantix-rust 对 TdxQuant 集成问题清单的回复

日期：2026-04-28

对应文档：`/opt/iflow/TdxQuant/docs/TdxQuant_Integration_Questions_Quantix.md`

## 1. 回复定位

这份文档不是重复提问，而是代表 `quantix-rust` 当前立场，对 `TdxQuant` 提出的集成问题做收敛式回复。

整体前提保持不变：

- `quantix-rust` 认可对 `TdxQuant` 做选择性接入
- 高优先级能力仍然是：
  - `formula`
  - `block`
  - `subscription`
- 当前不接受：
  - `desktop trade` 进入主执行链路
  - 直接库级深耦合
  - 让 `TdxQuant` 的入口层反向主导 `quantix-rust`

## 2. 先给出结论

### 2.1 Phase 1 首选接入面

`quantix-rust` 的正式首选接入面是：

- `Bridge HTTP + JSON/JSONL`

`CLI + JSON/JSONL` 可以接受，但只作为：

- 早期 PoC
- provider 内部调试入口
- bridge 未完全成型时的过渡方案

长期正式集成面仍然应优先：

- `service-first`
- 稳定 HTTP contract
- 稳定 capability probe

### 2.2 Windows provider 所有权

`quantix-rust` 当前更偏向以下边界：

- 对 `quantix-rust` 暴露的外部 contract，应由 Windows provider 边界统一拥有
- 短期内，这个边界更适合继续由现有 `quantix_bridge` 风格的服务层承接
- `TdxQuant` 可以作为该 provider 的核心能力源

换句话说：

- 我们不反对 `TdxQuant` 未来成长为 canonical provider
- 但当前更看重“外部 contract 单点稳定”，而不是“谁直接对外暴露”

这意味着：

- `health`
- `capabilities`
- `schema_version`
- `session lifecycle`
- 错误模型

这些内容必须由一个明确边界统一定义，不应同时存在两套对外口径。

### 2.3 第一条 PoC

`quantix-rust` 推荐的第一条 PoC 是：

1. `公式选股 -> 标准股票列表 -> quantix watchlist`

如果你们希望同一条 PoC 再顺带证明通达信资产沉淀能力，可以加一个可选扩展：

2. `公式选股 -> 标准股票列表 -> TongDaXin block`

但建议主验证目标仍然是第一条。

原因：

- 它最能体现 `TdxQuant` 的差异化能力
- 对 `quantix-rust` 当前链路最友好
- 副作用小
- 容易做 contract test
- 不会过早把长期 session 生命周期问题卷进来

`subscription-watch -> JSONL -> quantix monitor` 建议作为第二阶段 PoC，而不是第一阶段。

## 3. PoC 成功标准

第一阶段 PoC 建议把成功标准锁定为下面 6 条：

1. 输出必须稳定可解析，且是面向机器的结构化结果，不依赖人类可读文本。
2. 同一份输入在固定样例或 replay 模式下应能重复得到一致输出。
3. 结果必须能直接进入 `quantix-rust watchlist`，不需要人工清洗。
4. 失败路径必须返回稳定错误结构，而不是只打印日志或自由文本。
5. PoC 不应依赖真实交易环境，也不应绑定桌面自动化能力。
6. `quantix-rust` 侧必须能为该 PoC 写出稳定 contract test。

如果这 6 条里有任何一条不满足，`quantix-rust` 会认为它更像“能力演示”，而不是“可集成能力”。

## 4. 同步 JSON contract 要求

### 4.1 建议固定的 JSON 包络

`quantix-rust` 建议同步调用至少固定以下字段：

```json
{
  "success": true,
  "code": "ok",
  "message": "optional human summary",
  "capability": "formula.screen",
  "capability_version": "v1",
  "schema_version": "2026-04-28",
  "request_id": "optional-but-recommended",
  "started_at": "2026-04-28T12:00:00Z",
  "finished_at": "2026-04-28T12:00:01Z",
  "elapsed_ms": 1042,
  "runtime": {
    "provider": "tdxquant",
    "provider_version": "x.y.z",
    "mode": "bridge"
  },
  "warnings": [],
  "data": {},
  "artifacts": []
}
```

其中：

- `success/code/message` 负责结果语义
- `capability/capability_version/schema_version` 负责兼容治理
- `runtime` 负责运行形态说明
- `data` 放主结果
- `artifacts` 放文件、导出路径、附带产物

### 4.2 字段规范

`quantix-rust` 对格式的偏好如下：

- 时间：统一 `RFC3339`
- 股票代码：统一字符串，不要裸数字
- symbol：如无特殊原因，统一 `000001.SZ` / `600519.SH`
- 枚举：固定字面值，不用自由文本
- 布尔字段：不要用 `"yes" / "no"` 代替
- 金额与价格：若后续跨语言精度敏感，建议明确是否转字符串

### 4.3 退出码

如果使用 CLI 通路，`quantix-rust` 希望退出码也形成稳定 contract：

- `0`：成功
- 非 `0`：失败

同时失败详情仍应通过 JSON 输出，而不是只靠 shell exit code 推断。

## 5. 订阅 JSONL contract 要求

### 5.1 事件包络

如果 `TdxQuant` 提供订阅任务，`quantix-rust` 希望事件从第一版起至少具备这些字段：

```json
{
  "schema_version": "2026-04-28",
  "session_id": "sess_xxx",
  "provider_instance_id": "tdxquant-node-1",
  "subscription_id": "sub_xxx",
  "sequence": 1024,
  "event_type": "quote",
  "symbol": "000001.SZ",
  "source_ts": "2026-04-28T12:00:00.120Z",
  "event_ts": "2026-04-28T12:00:00.150Z",
  "reconnect_metadata": {
    "reconnect_count": 0
  },
  "payload": {}
}
```

其中以下字段对我们是强需求：

- `session_id`
- `subscription_id`
- `sequence`
- `event_type`
- `symbol`
- `source_ts`
- `event_ts`
- `payload`
- `schema_version`

`reconnect_metadata` 不是装饰字段。只要订阅具备重连能力，它就应进入正式 contract。

### 5.2 生命周期语义

如果订阅要成为长期稳定集成面，`quantix-rust` 认为这些语义都应存在：

- `start`
- `stop`
- `status`
- `list`
- 优雅退出
- 可观测的 session 结束原因
- 心跳或 watermark 事件
- 重连后的连续性说明

短期 CLI PoC 可以先弱化成：

- 启动一个前台订阅任务
- `Ctrl+C` 优雅退出
- 持续输出 JSONL

但如果要转为长期 provider，以上 lifecycle 项不能一直缺席。

## 6. capability 发现与 capability 分级

### 6.1 capability 发现

`quantix-rust` 明确需要 provider 提供 capability discovery。

至少需要知道：

- 当前可用 capability 列表
- 每个 capability 的版本
- 稳定性级别
- 运行前提
- 当前是否 degraded

这意味着一个 `capabilities` 响应不应只返回 `enabled=true/false`，还应能回答：

- 是不是实验能力
- 依赖是否满足
- 是否处于只读或副作用模式

### 6.2 capability 分级

`quantix-rust` 建议从一开始就做至少两套分级：

第一套是副作用分级：

- `read_only`
- `local_state_mutating`
- `live_side_effecting`

第二套是稳定性分级：

- `stable`
- `beta`
- `experimental`

这两套分级都应进入：

- 文档
- `capabilities` 输出
- 健康检查或 preflight

对于 `desktop trade` 这类能力，没有分级就不适合接入任何上层系统。

## 7. replay / contract test / 假数据模式

这部分是 `quantix-rust` 的强需求，不是附加项。

我们希望 `TdxQuant` 后续能提供：

- 固定输入样例
- 固定 JSON 输出样例
- 固定 JSONL 事件流样例
- replay 模式
- fake provider 模式
- contract test 夹具

优先服务的层次建议是：

1. provider 自测
2. bridge contract test
3. `quantix-rust` 端到端集成验证

原因很简单：

- 我们不能把每次兼容性验证都建立在真实客户端和真实桌面环境上
- 如果没有 replay / fake 模式，版本升级会非常难控

## 8. 性能与批量要求

### 8.1 公式能力

第一阶段我们更关注：

- 稳定性
- 可解析性
- 契约固定

而不是极限性能。

但仍建议 `TdxQuant` 在设计时考虑：

- 单次批量公式筛选的吞吐
- 结果分页或分块能力
- 超时与错误边界

### 8.2 板块能力

如果未来做 `block` 联动，建议默认考虑：

- 一次写入数百到数千成分股
- 幂等写入
- 覆盖写与增量写的区分

### 8.3 订阅能力

订阅场景下，`quantix-rust` 更在意：

- 不要静默丢事件
- 出现重连时要可观测
- 能看到 session 级统计

建议 provider 至少能暴露：

- 已发送事件数
- 丢弃事件数
- 当前订阅数
- 重连次数
- 最后活动时间

## 9. 责任边界

`quantix-rust` 希望这条边界明确锁死：

### 9.1 TdxQuant 负责

- TongDaXin 特有能力
- `formula`
- `block`
- `subscription`
- Windows 侧能力实现细节

### 9.2 quantix-rust 负责

- 执行内核
- 风控
- `runtime.db`
- order / order_event 的状态所有权
- `watchlist / monitor / strategy` 的上层业务语义

### 9.3 当前明确不接受

- `desktop trade` 进入 `quantix-rust` 主执行链路
- `catalog / report` 反向主导 `quantix-rust` 入口结构
- 直接库级嵌入 / FFI / PyO3 深耦合

## 10. 对 TdxQuant 问卷中 8 个关键项的直接回复

### 10.1 第一阶段更偏哪种接入面

答：明确偏向 `Bridge HTTP + JSON/JSONL`。

### 10.2 Windows provider 所有权更希望放在谁那里

答：短期更希望外部 contract 由现有 Windows bridge 边界统一承接，`TdxQuant` 可以作为内部 provider；长期不排斥 `TdxQuant` 成为 canonical provider，但前提是 contract 已稳定。

### 10.3 第一条 PoC 选哪条

答：`公式选股 -> 标准股票列表 -> quantix watchlist`。

### 10.4 PoC 成功标准是什么

答：结构化输出稳定、可重复运行、可脱离真实交易环境复验、可进入 `watchlist` 下游、可写 contract test。

### 10.5 同步 JSON contract 要求

答：需要固定 envelope、时间格式、symbol 格式、枚举字面值、错误结构与退出码语义。

### 10.6 订阅 JSONL contract 是否需要 `sequence / source_ts / reconnect_metadata`

答：需要，而且这是正式 contract 的必要字段，不是可选增强。

### 10.7 是否需要 capability 分级与 capability 发现机制

答：需要，而且建议从第一版就做。

### 10.8 是否需要 replay / contract test / 假数据模式

答：需要，这是 Rust 侧长期可维护集成的基础条件。

## 11. 我们建议 TdxQuant 下一步给出的东西

如果你们希望尽快进入实际联调，`quantix-rust` 最希望下一步看到的是下面三类产物之一：

### 11.1 最优先

一份 `formula.screen` 的稳定 JSON schema 草案。

### 11.2 第二优先

一份 `subscription-watch` 的 JSONL 事件 schema 草案，至少包含：

- `session_id`
- `subscription_id`
- `sequence`
- `symbol`
- `source_ts`
- `event_ts`
- `payload`

### 11.3 第三优先

一份 capability discovery 响应样例。

只要这三样里先落地一到两样，双方就能很快开始写实际 contract test，而不是继续停留在抽象讨论层。

## 12. 最终立场

`quantix-rust` 对 `TdxQuant` 的正式态度可以概括为：

- 愿意接
- 但只接差异化能力
- 愿意先做窄 PoC
- 明确偏好桥接协议而非深耦合
- 把可测试、可回放、可发现、可分级看得和“功能本身”一样重要

如果 `TdxQuant` 能围绕这些要求稳定输出 machine contract，它非常有机会成为 `quantix-rust` 在 Windows 侧的高价值 TongDaXin capability provider。
