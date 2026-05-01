# TdxQuant Next Steps 边界对齐说明

日期：2026-04-28

参考文档：`/opt/iflow/TdxQuant/docs/TdxQuant_Next_Steps.md`

## 1. 目的

这份说明用于在 `quantix-rust` 侧固化一个前提：

- `TdxQuant_Next_Steps.md` 已经可以视为 `TdxQuant` 当前的 canonical capability boundary
- 后续如果 `quantix-rust` 要接入 `TdxQuant`，默认应按这份边界理解其对外能力

它不是对 `TdxQuant` 的重新评审，而是一次边界确认。

## 2. 当前可视为已对齐的核心边界

根据该文档，`quantix-rust` 后续应默认接受以下前提：

### 2.1 TdxQuant 的角色定位

`TdxQuant` 当前应被视为：

- `Windows side TongDaXin capability provider`

而不是：

- `quantix-rust` 的主数据服务替代品
- `quantix-rust` 的主任务编排中心
- 当前优先接入的标准化交易中心

### 2.2 优先稳定的正式能力线

未来接入应优先围绕：

- `formula`
- `subscription`
- `block`

而不是优先围绕：

- `catalog`
- `report`
- `desktop trade`

### 2.3 正式集成面的默认预期

`quantix-rust` 后续应继续按如下预期准备：

- 长期正式集成面优先是 `HTTP + JSON / JSONL`
- `CLI + JSON` 只作为早期 PoC 或调试路径
- 上层不应把 `task / report / catalog` 视为稳定 provider contract

### 2.4 provider contract 先于横向扩功能

`TdxQuant` 后续最重要的建设方向已明确是：

- 固定同步 JSON contract
- 固定订阅 JSONL contract
- 建立 capability discovery / health probe / doctor / preflight
- 提供 capability grading
- 提供 replay / fake / contract test 夹具

这意味着 `quantix-rust` 后续联调时，默认应先关注：

- schema
- capability naming
- versioning
- lifecycle semantics
- testability

而不是先追求更多入口数量。

## 3. 对 quantix-rust 的直接含义

### 3.1 未来接入顺序不变

`quantix-rust` 后续如果实际开始接入，仍应遵守此前已形成的顺序：

1. `formula.screen` 类 contract
2. `subscription-watch` 类 JSONL contract
3. `block` 写入治理能力

### 3.2 执行与交易边界不变

即使 `TdxQuant` 后续继续发展交易线，`quantix-rust` 当前仍应保持以下边界：

- 执行内核归 `quantix-rust`
- 风控归 `quantix-rust`
- `runtime.db` 和 order / order_event 的状态所有权归 `quantix-rust`
- `desktop trade` 继续作为高风险独立 capability 看待

因此，当前不应把 `TdxQuant` 交易线纳入 `quantix-rust` 主执行链路假设。

### 3.3 capability probe 与 contract test 需要提前准备

既然 `TdxQuant` 已明确把 capability discovery、contract 和 replay 作为 next steps，`quantix-rust` 后续应提前按此准备：

- capability probe 消费逻辑
- 同步 JSON contract test
- 订阅 JSONL 消费与验证逻辑
- watchlist 导入型 PoC 路径

## 4. 当前默认采纳的接入前提

除非后续 `TdxQuant` 自身路线再次变化，否则 `quantix-rust` 当前默认采纳以下前提：

- `TdxQuant_Next_Steps.md` 是上游能力边界的主参考文档
- `formula`、`subscription`、`block` 是优先接入面
- 正式 provider contract 将优先走 `HTTP + JSON / JSONL`
- capability discovery、grading、replay、contract test 是正式接入前提的一部分
- `desktop trade` 不进入当前主执行链路

## 5. 结论

`quantix-rust` 现在可以把 `TdxQuant_Next_Steps.md` 视为一个有效的未来对齐基线。

后续若开始实际接入，不应再从“是否认可 Windows provider / 是否需要公式能力”这类问题重新讨论起，而应直接进入：

- contract 对齐
- capability probe 对齐
- PoC 落地
- replay / test fixture 对齐

一句话总结：

从今天起，`quantix-rust` 可以把 `TdxQuant` 理解为一个正在主动收敛为 **Windows 侧、协议稳定、可发现、可测试、可分级** 的 TongDaXin provider，而后续接入准备也应按这条边界展开。
