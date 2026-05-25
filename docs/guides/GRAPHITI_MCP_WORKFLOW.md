# Quantix Graphiti MCP Workflow

> 状态源说明：本文是 Graphiti MCP 工作流指南，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 [`FUNCTION_TREE.md`](../../FUNCTION_TREE.md) 的状态注册表行为准。

## Purpose

本文件定义 `quantix-rust` 项目中对 `graphiti-mcp` 的使用边界、`group_id` 规划、必查/必写节点、回填规则和质量标准。

目标不是把 Graphiti 当成“第二个任务系统”或“代码真相库”，而是把它作为：

- AI CLI 的长期语义记忆层
- 设计决策、review 结论、debug 根因、handoff 摘要、规范决议的历史检索层
- 面向“为什么这样做”“之前如何判断”的项目语义档案

本项目明确不把 Graphiti 用作：

- 当前代码实现的权威来源
- 当前任务状态库
- merge / approval / ownership 状态库
- 业务 runtime 依赖

## Core Positioning

### Graphiti Owns Semantic Memory

以下信息适合沉淀到 Graphiti：

- 设计决策、权衡、不采纳方案、非目标
- review finding、采纳/拒绝理由、残余风险
- debug 症状、根因、修复路径、验证结论
- handoff / pause checkpoint / 接手说明
- 文档规范、命名规则、术语收敛结论
- 会持续影响后续工作的 Requirement / Preference / Procedure

### Graphiti Does Not Own Source of Truth

以下信息不得以 Graphiti 作为唯一或权威来源：

- 当前代码怎么实现、调用链怎么走
- 当前任务状态、claim、审批、merge 进度
- 测试是否真的通过
- 需要强一致或强事务保证的数据

简化判断：

- 问“代码现在怎么实现” -> 查 GitNexus / 源码 / 测试
- 问“为什么之前这么定” -> 查 Graphiti
- 问“这次 review 为什么拒绝/接受” -> 查 Graphiti
- 问“当前任务到哪一步了” -> 查项目状态系统，不查 Graphiti

## How Graphiti Actually Works

把 Graphiti 用对，先要接受它的底层模型：

- 写入单位是 `episode`
- 检索对象分成 `nodes` 和 `facts`
- `facts` 带时间语义，新的事实可以使旧事实失效，而不是简单覆盖
- `group_id` 决定记忆域
- `add_memory` 是异步入队，不等于立刻可检索

这意味着项目侧工作流应遵循下面几条原则：

- 写“收敛后的结论快照”，不要写不断抖动的状态表
- 当结论变化时，追加一条新 memory，并显式说明它修正或替代了什么
- 不要把大量互不相关的内容塞进一条 memory
- 不要为每次会话随机创建新的 `group_id`
- 写入后必须走 `get_ingest_status`，不要凭感觉判断已经落图

## Tool Boundary

在 `quantix-rust` 中：

- GitNexus 负责代码结构、调用链、影响分析
- Git / 源码 / 测试负责最终实现依据
- Graphiti 负责长期语义记忆和历史结论检索

不要用 Graphiti 替代：

- `git log`
- 代码审阅
- 测试验证
- GitNexus 的 `query` / `context` / `impact`
- 当前任务状态系统

推荐问题分流：

| 问题类型 | 首选工具 |
|---|---|
| “这个模块当前怎么实现” | GitNexus / 源码 |
| “这个改动会影响哪里” | GitNexus impact/context |
| “之前为什么这么设计” | Graphiti `search_memory_facts` |
| “有哪些项目级要求/偏好/流程” | Graphiti `search_nodes` |
| “上次排障怎么定性的” | Graphiti `search_memory_facts` |
| “最近交接做到哪里” | Graphiti `get_episodes` + `search_memory_facts` |

## Configuration Boundary

### MCP Server Name

项目内统一使用 MCP server 名称：

- `graphiti-memory`

### Endpoint Policy

Graphiti MCP endpoint 属于客户端配置，不写死到仓库 runtime 或业务配置。

约定：

- 以当前 MCP 客户端配置为准
- 仓库文档可以描述“通过 MCP 配置提供”
- 不把 endpoint 当成业务系统配置项

### Runtime Boundary

本项目明确不做：

- 在业务代码里新增 `graphiti-api` / `graphiti-mcp` 依赖
- 在 CLI runtime 中把 Graphiti 当硬依赖
- 因 Graphiti 不可用而让项目逻辑失败

## Group ID Layout

按职责拆分，使用稳定 `group_id`：

- `quantix_rust_main`
  - 架构决策、设计收敛、阶段性实施结论
- `quantix_rust_review`
  - review finding、采纳/拒绝理由、残余风险
- `quantix_rust_debug`
  - 症状、根因、修复路径、验证结论
- `quantix_rust_handoff`
  - 暂停点、接手说明、下一步建议
- `quantix_rust_docs`
  - 规范决议、命名规则、术语收敛、文档维护结论

约束：

- 同类记忆落到固定 `group_id`
- 不要长期把所有内容都写进单一 group
- 不要把任务状态复制进 Graphiti 当状态库
- 同一 `group_id` 的摄入是串行的，过热写入会拖慢这一组的可检索时间

说明：

- 本项目优先按“工作流职责”分组，而不是按实体类型分组
- 如果要检索 Requirement / Preference / Procedure，优先在对应职责组里结合 `entity_types` 过滤

## Read Policy

### Session-Level Health Check

每次会话首次依赖 Graphiti，或怀疑服务异常时，先调用一次：

1. `get_status`

注意：

- `get_status` 只能证明 MCP 服务和数据库连接大致可用
- 它不等于“写入链路一定正常”
- 它也不等于“LLM / embedder / 异步队列一定健康”

### Retrieval Strategy

检索不要机械套固定顺序，而应按问题类型选择：

1. 想找“有哪些对象、规范、要求、流程、文档”
   - 用 `search_nodes`
   - 必要时加 `entity_types=["Requirement", "Preference", "Procedure", "Document"]`
2. 想找“为什么这样定、谁依赖谁、上次如何判断”
   - 用 `search_memory_facts`
3. 想看“最近几次写入了什么、最近交接到哪”
   - 用 `get_episodes`
4. 如果已经命中一个明显正确的实体或事实
   - 用 `center_node_uuid` 做二跳扩展

### Mandatory Read Points

在以下节点，开始工作前必须先查 Graphiti：

1. 新设计开始前
   - 先查 `quantix_rust_main`
   - 涉及规范或术语时补查 `quantix_rust_docs`
2. 处理 review 前
   - 先查 `quantix_rust_review`
   - 必要时补查 `quantix_rust_main`
3. 开始 debug / 排障前
   - 先查 `quantix_rust_debug`
   - 若怀疑根因与既有设计有关，再补查 `quantix_rust_main`
4. 接手已有条线前
   - 先查 `quantix_rust_handoff`
   - 再按需要查对应的 `main` / `review` / `debug`
5. 涉及项目规范、命名、流程习惯前
   - 先用 `search_nodes` 查 `Requirement` / `Preference` / `Procedure`

## Write Policy

### Mandatory Write Points

在以下节点，结论收敛后必须写入 Graphiti：

1. 设计确认后
   - 写入 `quantix_rust_main`
2. review 收敛后
   - 写入 `quantix_rust_review`
3. debug 根因确认后
   - 写入 `quantix_rust_debug`
4. 暂停 / 交接前
   - 写入 `quantix_rust_handoff`
5. 文档规范、命名规则、术语决议收敛后
   - 写入 `quantix_rust_docs`
6. 发现会持续影响后续工作的 Requirement / Preference / Procedure 时
   - 写入最相关的职责 group

### Ingest Verification Sequence

标准写入顺序：

1. `add_memory`
2. 保存返回的 `episode_uuid`
3. `get_ingest_status` 轮询到 `state == completed`
4. 必要时用 `get_episodes` 看最近 episode 是否入图
5. 再用 `search_nodes` / `search_memory_facts` 验证检索面是否符合预期

不要做的推断：

- `add_memory` 返回成功 != 立刻可检索
- `get_status == ok` != 摄入链一定正常
- `get_episodes` 能看到 episode != 节点和事实已经完全抽取

## Memory Quality Standard

Graphiti 不是普通日志桶。写得越“像结论”，后续越好检索。

### Basic Rules

- 一条 memory 只表达一个收敛主题
- 优先写结论、理由、约束、影响，不写流水账
- 默认用 `source="text"`
- 已经有结构化对象时再考虑 `source="json"`
- 除非确实要保存对话，不要默认用 `source="message"`
- 不要把长篇原始 transcript、完整 diff、超长日志直接塞进去

### Recommended Title

建议 `name` 统一前缀：

- `design: <topic>`
- `review: <topic>`
- `debug: <topic>`
- `handoff: <topic>`
- `docs: <topic>`
- `requirement: <topic>`
- `procedure: <topic>`
- `preference: <topic>`

### Recommended Body Template

写入 Graphiti 时，优先使用结构化自然语言：

- `Topic: <主题>`
- `Context: <背景>`
- `Decision/Finding: <结论>`
- `Why: <理由或判断依据>`
- `Constraints: <关键约束>`
- `Verification: <如何验证 / 验证结果>`
- `Supersedes: <若为更新，写清替代哪条旧结论；否则写 none>`
- `Files/Commands: <相关文件或命令>`
- `Next Step: <下一步入口>`
- `Risks: <残余风险>`

这样更利于：

- `search_nodes` 抽出 Requirement / Preference / Procedure / Document 等实体
- `search_memory_facts` 抽出“主题-结论-依据-风险”的关系
- 后续 AI CLI 恢复上下文和追溯决策

### Update Rule

当旧结论被修正时：

- 不要尝试“覆盖”旧 memory
- 应追加一条新 memory
- 在正文里显式写出“之前的结论是什么、为什么现在修正”

这更符合 Graphiti 的时间语义和事实失效模型。

## Query Playbook

### Query by Entity

适用：

- 找项目里已有的要求、规范、流程、文档、关键模块

建议：

- 先 `search_nodes`
- query 写“模块名 + 主题 + 场景”
- 有明确意图时加 `entity_types`

例子：

- `query="workspace 命令使用规范"`
- `entity_types=["Procedure", "Document"]`

### Query by Fact

适用：

- 找设计理由、依赖关系、历史判断、残余风险

建议：

- 用完整自然语言提问
- 带上主题、对象、判断维度

例子：

- `query="为什么 quantix-rust 不把 Graphiti 接入 runtime"`
- `query="上次这个模块的 debug 根因和验证结论"`

### Two-Hop Expansion

当第一次 `search_memory_facts` 已经命中正确实体时：

1. 取结果中的 `source_node_uuid` 或 `target_node_uuid`
2. 再次调用 `search_memory_facts(..., center_node_uuid=...)`

适合：

- 围绕正确模块继续扩展依赖/风险
- 围绕正确文档继续扩展相关规范
- 围绕正确根因继续扩展相关修复事实

### When To Use `get_episodes`

`get_episodes` 适合：

- 看最近写入
- 看最近 handoff
- 排查“有没有写进去”

`get_episodes` 不适合替代：

- 主题检索
- 关系检索
- 规范/设计原因查询

## Scenario Playbooks

### Design Workflow

开始设计前：

1. `get_status`，仅在本会话首次使用 Graphiti 时执行
2. 搜索 `quantix_rust_main`
3. 若涉及术语、文档结构、规范，再搜索 `quantix_rust_docs`
4. 用 `search_memory_facts` 查历史理由和非目标

设计确认后，必须写入：

- 背景
- 采用方案
- 不采用方案
- 非目标
- 关键约束
- 验证思路
- 下一步入口

建议标题：

- `design: <topic>`

### Review Workflow

处理 review 前：

1. 搜索 `quantix_rust_review`
2. 必要时补查 `quantix_rust_main`
3. 检查同类 finding 是否已有历史处理结论

review 收敛后，必须写入：

- review 范围
- 采纳项
- 拒绝项及技术理由
- 残余风险
- 后续待跟进项

建议标题：

- `review: <topic>`

### Debug Workflow

开始 debug 前：

1. 搜索 `quantix_rust_debug`
2. 搜索相同症状、模块、错误模式
3. 若怀疑与既有设计选择有关，再补查 `quantix_rust_main`

根因确认后，必须写入：

- 症状
- 复现条件
- 根因
- 修复思路
- 验证结论
- 未解决问题

建议标题：

- `debug: <topic>`

### Handoff Workflow

接手前：

1. 搜索 `quantix_rust_handoff`
2. 查看最近 checkpoint
3. 再按主题补查 `main` / `review` / `debug`

暂停或交接前，必须写入：

- 当前做到哪里
- 已完成项
- 未完成项
- 当前风险
- 下一步建议
- 相关文件 / 命令 / 验证状态

建议标题：

- `handoff: <topic>`

### Docs Workflow

当工作主要涉及规范、手册、命名、术语收敛时：

1. 搜索 `quantix_rust_docs`
2. 必要时补查 `quantix_rust_main`
3. 结论确认后写回 `quantix_rust_docs`

建议标题：

- `docs: <topic>`

## Fallback Rule

如果 Graphiti MCP 当前不可用、未接入、`get_status` 失败、或某次 `get_ingest_status` 最终进入 `failed`：

1. 仍视为必须执行“记忆沉淀”动作
2. 在本地文档、设计文档、review 文档、debug 记录或 handoff 文档中留下同等摘要
3. 显式写一行：

```text
Graphiti backfill required
```

4. 待 Graphiti 恢复后，按原 `group_id` 补写 memory

不允许：

- 直接跳过
- 只口头说“以后补”
- 不留下等价摘要

## Dangerous Tools Boundary

虽然 Graphiti MCP 还暴露了维护工具，但项目日常开发流程默认不使用：

- `delete_episode`
- `delete_entity_edge`
- `clear_graph`

这些工具只适用于：

- 错误数据回滚
- 测试数据清理
- 管理员级维护

它们不应该出现在普通设计、review、debug、handoff 流程中。

## Minimal MCP Reference

最常用工具：

- `get_status`
- `add_memory`
- `get_ingest_status`
- `search_nodes`
- `search_memory_facts`
- `get_episodes`

关键参数约定：

- 写入用 `group_id`
- 检索用 `group_ids`
- `group_ids` 是列表，不是单个字符串
- `get_episodes` 的数量参数是 `max_episodes`

推荐使用习惯：

- 想找实体 / 规范 / 流程 -> `search_nodes`
- 想找关系 / 理由 / 历史结论 -> `search_memory_facts`
- 想看最近写入 / 最近 handoff -> `get_episodes`
- 写入后不要默认立刻可搜到 -> `get_ingest_status`

## Anti-Patterns

本项目明确避免以下用法：

- 把 Graphiti 当审批系统
- 把 Graphiti 当任务状态库
- 把 Graphiti 接进业务 runtime
- 只写“已完成”“已修复”这种无上下文摘要
- 把长篇原始对话或日志整段塞进一条 memory
- 为每次会话创建随机 `group_id`
- 看到 `add_memory` 返回成功就假定检索可用
- 用 Graphiti 替代 GitNexus、测试、源码审阅

## Adoption Sequence

本项目推荐按以下顺序落地：

1. 先把本规范写入项目文档
2. 再把“必查 Graphiti / 必写摘要 / backfill 规则”写入 `AGENTS.md`
3. 在 MCP 客户端接入 `graphiti-memory`
4. 之后在设计 / review / debug / handoff / docs 流程中按本规范执行

## One-Screen Summary

如果只记最小规则，记这几条：

1. Graphiti 是“长期语义记忆”，不是“代码真相”也不是“任务状态”
2. 搜索前先想清楚你是在找 entity、fact 还是 recent episode
3. 设计、review、debug、handoff、docs 收敛后必须写入
4. 写入要写“结论 + 理由 + 约束 + 验证 + 风险”，不要只写结果
5. `add_memory` 后必须轮询 `get_ingest_status`
6. 结论变化时追加新 memory，并显式说明它修正了什么
