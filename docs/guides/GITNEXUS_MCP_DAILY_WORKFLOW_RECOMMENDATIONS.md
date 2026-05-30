# GitNexus MCP 日常使用建议

状态: 审核稿
日期: 2026-05-30
适用项目: `quantix-rust`

## 背景

GitNexus MCP 升级后，本项目可以把它作为日常开发中的结构化导航和风险闸门，而不是只把它当成更强的文本搜索工具。

截至 2026-05-30 的一次检查，`quantix-rust` 的 GitNexus 索引仍然显示 `embeddings: 0`，并且索引可能落后于 HEAD。文件数、节点数、边数和 process 数会随提交快速漂移，本文不把这些数字作为稳定事实。日常使用时，应优先查看 GitNexus 工具返回的当前 staleness 和 stats。

因此，本文建议优先使用 GitNexus 做精确符号、调用图、字段访问、执行流和改动影响分析。自然语言语义检索只有在显式重新生成 embeddings 后才应被当作稳定能力使用。

本文中的工具调用示例均为 MCP pseudocode，采用 agent-facing 名称，例如:

```text
gitnexus_impact({
  repo: "quantix-rust",
  target: "<target_symbol>",
  direction: "upstream"
})
```

## 总体原则

在 `quantix-rust` 中，GitNexus 最有价值的用途是把主观判断替换成结构化证据:

- 改动前: 先确认目标符号的调用者、被调用者、执行流和影响范围。
- 改动中: 按影响范围小步修改，不扩大到无关清理。
- 改动后: 用 `detect_changes` 确认实际影响是否符合预期。
- 提交前: 把 GitNexus 结果和 cargo gates 一起作为收口证据。

一句话建议:

> 不要用 GitNexus 证明“我觉得这个改动很小”；要用它证明“调用图、字段访问、执行流和 diff hunk 都显示影响范围可控”。

## 30 秒 quick path

理解功能:

```text
gitnexus_query(...) -> gitnexus_context(...)
```

修改函数、方法、类型前:

```text
gitnexus_impact({ direction: "upstream", ... })
```

`gitnexus_impact` 是 `AGENTS.md` 规定的强制门禁；`gitnexus_context` 是推荐的补充理解步骤。

实现阶段:

```text
小步编辑 -> 针对性测试
```

工作区自查:

```text
gitnexus_detect_changes({ scope: "all", cwd: "<current_checkout_or_worktree>" })
```

提交前范围门禁:

```text
gitnexus_detect_changes({ scope: "staged", cwd: "<current_checkout_or_worktree>" })
```

重命名:

```text
gitnexus_rename({ dry_run: true, ... }) -> review -> gitnexus_rename({ dry_run: false, ... }) -> gitnexus_detect_changes(...)
```

## Mandatory gates from AGENTS.md

以下不是建议，而是本项目 `AGENTS.md` 已规定的强制要求:

- 修改任何 function、class、method 前，必须先运行 `gitnexus_impact`。
- 提交前，必须运行 `gitnexus_detect_changes`。
- 如果 impact analysis 返回 `HIGH` 或 `CRITICAL` 风险，必须先警告用户并说明 blast radius，再继续。
- rename、extract、split、refactor 等结构性变更不能用普通 find-and-replace 替代 GitNexus 工具。

本文后续章节提供的是推荐习惯和具体操作细节，不降低这些强制门禁的优先级。

## Recommended habits

推荐在强制门禁之外养成以下习惯:

- 用 `gitnexus_query` 从概念定位 execution flow。
- 用 `gitnexus_context` 查看目标符号的 callers、callees、字段访问和 process participation。
- 对 hub symbol 先用 `summaryOnly: true` 控制输出规模。
- 对字段、成员、trait 或 impl 行为，按需扩展 `relationTypes`。
- 对非 Rust 或非代码改动，补充 GitNexus 之外的验证 gate。

## 日常入口

### 1. 显式指定仓库

当前环境中有多个 indexed repo。日常使用 GitNexus MCP 时，应显式传入:

```text
repo: "quantix-rust"
```

不要依赖默认仓库选择。尤其在多个 worktree 或多个项目同时被索引时，默认选择可能不是当前项目。

### 2. 先确认索引新鲜度

如果 `list_repos` 或任何 GitNexus 工具提示索引 stale，应先刷新索引，再做正式影响分析:

```bash
gitnexus analyze
```

普通 `gitnexus analyze` 适合以下场景:

- 精确符号查询
- 文件和关键词查询
- 调用图分析
- impact 分析
- detect_changes 检查

只有当任务确实依赖自然语言、概念型或模糊代码搜索时，才建议显式运行:

```bash
gitnexus analyze --embeddings
```

如果此前索引已经包含 embeddings，普通 `gitnexus analyze` 可能删除已有 embeddings。运行前应检查 `.gitnexus/meta.json` 或当前 GitNexus stats，确认是否需要保留 embeddings 并改用 `--embeddings`。

当前检查中 `quantix-rust` 的 embeddings 数量为 0。这不代表 `gitnexus_query` 没价值；它仍然有 BM25、关键词匹配和 process 分组价值。但使用时应提供更具体的符号、模块、业务词，并用 `gitnexus_context` 验证候选结果。

## 推荐工作流

### 1. 理解功能: `gitnexus_query -> gitnexus_context`

当问题是“这个功能在哪里实现”或“某条业务路径怎么跑”时，先用 `gitnexus_query` 找执行流:

```text
gitnexus_query({
  repo: "quantix-rust",
  query: "market command data provider flow"
})
```

拿到候选入口后，再对具体符号使用 `gitnexus_context`:

```text
gitnexus_context({
  repo: "quantix-rust",
  name: "run_market_command"
})
```

`gitnexus_query` 适合找概念和流程入口，`gitnexus_context` 适合确认一个具体符号的 callers、callees、字段访问和 process participation。

### 2. 修改符号前: mandatory impact gate

修改函数、类型、trait、impl method 或共享配置前，`gitnexus_impact` 是强制门禁:

```text
gitnexus_impact({
  repo: "quantix-rust",
  target: "<target_symbol>",
  direction: "upstream"
})
```

建议在 impact 前先看完整上下文:

```text
gitnexus_context({
  repo: "quantix-rust",
  name: "<target_symbol>"
})
```

`gitnexus_context` 有助于理解，但不能替代 mandatory impact gate。

如果结果是 `HIGH` 或 `CRITICAL`，应先停下来说明:

- 风险等级
- 直接调用者
- 受影响 modules
- 受影响 processes
- 是否需要缩小改动、拆分任务或补充测试

### 3. Hub symbol 先用 summary

对共享入口、provider、runtime context、strategy service、risk path、backtest task resolver 这类 hub symbol，不建议直接拉完整 impact。优先使用:

```text
gitnexus_impact({
  repo: "quantix-rust",
  target: "<target_symbol>",
  direction: "upstream",
  summaryOnly: true,
  limit: 50
})
```

先看风险等级、直接调用者数量、受影响模块和 processes，再决定是否分页查看细节。

### 4. 字段、成员和 trait 行为要扩展 relationTypes

默认 impact 更偏向调用、导入、继承和实现关系。以下改动需要额外关注字段和成员关系:

- 修改结构体字段
- 修改 runtime context 或 provider context
- 修改 trait method 签名或默认行为
- 修改 impl method 的外部契约
- 修改 strategy、risk、backtest、market data 等共享状态对象

建议按需增加:

```text
gitnexus_impact({
  repo: "quantix-rust",
  target: "<target_symbol>",
  direction: "upstream",
  summaryOnly: true,
  relationTypes: [
    "CALLS",
    "IMPORTS",
    "IMPLEMENTS",
    "ACCESSES",
    "HAS_METHOD",
    "HAS_PROPERTY",
    "METHOD_OVERRIDES",
    "METHOD_IMPLEMENTS"
  ]
})
```

噪声控制规则:

- 扩展 `relationTypes` 时默认先加 `summaryOnly: true`。
- 先看 d=1 直接依赖，再决定是否展开 d=2/d=3。
- 对 hub symbol 使用 `limit` 和 `offset` 分页钻取。
- 不要机械全开；当改动可能通过字段读写、成员方法、trait 实现或 override 传播时，再扩大关系类型。

### 5. 修改后: `gitnexus_detect_changes`

`scope: "all"` 适合开发过程中的工作区 sanity check，但不是最终提交门禁。它会混入用户未提交改动、多任务并行改动或其他无关脏工作区内容。

开发中自查:

```text
gitnexus_detect_changes({
  repo: "quantix-rust",
  scope: "all",
  cwd: "/opt/claude/quantix-rust"
})
```

提交前范围门禁应优先使用 staged:

```text
gitnexus_detect_changes({
  repo: "quantix-rust",
  scope: "staged",
  cwd: "/opt/claude/quantix-rust"
})
```

如果在 linked worktree 中开发，`cwd` 或 `worktree` 必须指向当前 worktree，例如:

```text
gitnexus_detect_changes({
  repo: "quantix-rust",
  scope: "staged",
  worktree: "/opt/claude/quantix-rust/.worktrees/<worktree-name>"
})
```

排查回归或审查当前分支相对主线的影响时:

```text
gitnexus_detect_changes({
  repo: "quantix-rust",
  scope: "compare",
  base_ref: "main",
  cwd: "/opt/claude/quantix-rust"
})
```

### 6. 重命名必须走 `gitnexus_rename`

符号重命名不要使用普通 find-and-replace。先预览:

```text
gitnexus_rename({
  repo: "quantix-rust",
  symbol_name: "<old_name>",
  new_name: "<new_name>",
  dry_run: true
})
```

审查结果时区分:

- `graph`: 通常可信度高。
- `text_search`: 需要人工复核上下文。

确认后再执行非 dry-run，并用 `gitnexus_detect_changes` 收口。

### 7. 复杂结构问题用 `gitnexus_cypher`

当 `gitnexus_query`、`gitnexus_context`、`gitnexus_impact` 无法直接回答结构问题时，再使用 `gitnexus_cypher`。适合的问题包括:

- 谁写入了某个字段？
- 哪些类型实现了某个 trait？
- 某类 process 的入口有哪些？
- 某个 module 是否出现跨边界调用？
- 哪些测试直接覆盖某个 handler 或 provider？

`gitnexus_cypher` 应作为高级工具使用，不作为日常第一步。

## 异常路径

### 1. impact 找不到符号

先不要猜符号名。按以下顺序收敛:

```text
gitnexus_query({ repo: "quantix-rust", query: "<concept_or_error>" })
gitnexus_context({ repo: "quantix-rust", name: "<candidate_name>" })
gitnexus_impact({ repo: "quantix-rust", target_uid: "<resolved_uid>", direction: "upstream" })
```

如果 `context` 返回多个候选，使用 `file_path`、`kind` 或 `uid` 消歧。

### 2. 工具提示 index stale

先刷新索引:

```bash
gitnexus analyze
```

如果需要保留或生成 embeddings，改用:

```bash
gitnexus analyze --embeddings
```

刷新后重新运行原 GitNexus 查询，不要复用 stale 结果作为正式门禁证据。

### 3. symbol ambiguous

多候选符号时，不要让工具默认选择。使用:

```text
gitnexus_context({
  repo: "quantix-rust",
  name: "<symbol_name>",
  file_path: "src/path/to/file.rs"
})
```

或在后续工具中直接使用上一步返回的 `uid` / `target_uid`。

### 4. HIGH 或 CRITICAL

`HIGH` 或 `CRITICAL` 不是自动禁止改动，但必须先收口风险说明:

- 哪些 d=1 直接依赖会受影响？
- 哪些 execution flows 会受影响？
- 是否可以缩小改动到局部 adapter、wrapper 或 provider？
- 是否需要先补测试或拆成设计授权任务？

在用户确认前，不应把高风险改动当作普通小修继续推进。

## 工具使用建议

| 场景 | 首选工具 | 目的 |
| --- | --- | --- |
| 找功能入口 | `gitnexus_query` | 从概念找到 execution flow |
| 看单个符号 | `gitnexus_context` | 查看 callers, callees, fields, processes |
| 修改前评估 | `gitnexus_impact` | 评估 blast radius；本项目中是强制门禁 |
| 修改后自查 | `gitnexus_detect_changes({ scope: "all" })` | 工作区 sanity check，可能混入无关 dirty changes |
| 提交前范围门禁 | `gitnexus_detect_changes({ scope: "staged" })` | 本项目提交前强制 gate |
| 符号重命名 | `gitnexus_rename` | 图感知重命名 |
| 自定义结构查询 | `gitnexus_cypher` | 回答复杂图问题 |
| API 路由项目 | `api_impact`, `route_map`, `shape_check` | 本项目中优先级较低 |

## 本项目的边界提醒

GitNexus 对 `quantix-rust` 最可靠的覆盖范围是 Rust symbol graph。以下类型的改动不能只依赖 GitNexus impact 结论:

| 改动类型 | GitNexus 边界 | 替代或补充 gate |
| --- | --- | --- |
| `.github/workflows/*.yml` | Rust symbol graph 覆盖有限 | `actionlint`、CI dry-run、现有 workflow structure tests |
| Docker workflow | 可能只表现为文件或外部流程 | Docker build/smoke、相关 CI job、路径和上下文检查 |
| shell scripts | 不进入 Rust 调用图 | `bash -n`、shellcheck、任务相关 smoke |
| Markdown 文档 | 不影响 Rust symbol graph | 链接/路径检查、文档结构检查、相关规范审查 |
| 配置文件 | 可能间接影响 Rust 运行时 | `cargo check`、相关集成测试、配置加载 smoke |
| 外部运行时行为 | MCP 图无法证明环境正确 | fake provider/replay mode、contract-test fixtures、手动或脚本化验收 |

如果 GitNexus 对这些改动报告低风险或无 affected processes，只能说明 Rust symbol graph 层面没有发现影响，不能替代对应 gate。

## 审核关注点

审核本文时，建议重点确认:

- 是否要把部分建议提升为 `AGENTS.md` 的强制规则。
- 是否要规定何时运行 `gitnexus analyze --embeddings`。
- 是否要为 linked worktree 开发补充固定的 `cwd` 或 `worktree` 使用约定。
- 是否要为 hub symbol 定义默认的 `summaryOnly` 和分页阈值。
- 是否要为 CI、shell、Docker、docs 类改动明确 GitNexus 之外的验证清单。
