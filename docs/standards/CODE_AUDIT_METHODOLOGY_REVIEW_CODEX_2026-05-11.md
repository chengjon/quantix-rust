# CODE_AUDIT_METHODOLOGY.md 审查意见

> 审查对象: `docs/standards/CODE_AUDIT_METHODOLOGY.md`  
> 审查日期: 2026-05-11  
> 审查人: Codex  
> 结论: 方法论框架可用，但需要先修正范围计数、执行顺序、工具口径和交付 schema，之后再用于全面代码审核。
>
> 状态源说明：本文是方法论文档审查意见，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 [`FUNCTION_TREE.md`](../../FUNCTION_TREE.md) 的状态注册表行为准。

---

## 一、总体评价

`CODE_AUDIT_METHODOLOGY.md` 已经覆盖了全面代码审核所需的大部分骨架：目标、范围、工具、审核维度、执行流程、风险分级、交付物和自检清单都存在，且对本项目的关键风险点有明确意识，特别是执行链路、QMT Live 边界、MOCK/真实路径隔离、状态机一致性和 Graphiti/GitNexus 的辅助作用。

但当前版本仍有几个会影响后续实际审核质量的问题：

1. 一些“当前数量”已经与工作区事实不一致。
2. 审核流程中 GitNexus freshness/analyze 的顺序不够合理。
3. 工具命名偏 Claude Code，不完全适用于当前 Codex 执行环境。
4. “全面审核”与 P2/P3、测试抽样规则之间存在口径冲突。
5. findings 输出结构不足以支撑后续修复闭环。
6. Graphiti 写入策略需要与项目级 AGENTS.md 规则对齐。

建议先修订这些方法论问题，再进入正式代码审核。

---

## 二、需要优先修正的问题

### 1. 范围计数已漂移

位置:

- `2.1 代码库覆盖`
- `4.5.1 测试现状`
- `附录 A.2 当前模块清单`

当前文档写法:

- `src/cli/commands/` 全量 12 个命令定义文件审查
- `src/cli/handlers/` 全量 31 个 handler 文件分发逻辑审查
- `tests/` 当前 74 个文件
- 附录 A.2 中 `commands` 仍标注 12 文件，`handlers` 标注 31 文件

我在当前工作区核对结果:

```text
src/ 顶层子目录: 28
src/cli/commands/*.rs: 13
src/cli/handlers/*.rs: 30
tests/*.rs: 72
```

影响:

这些数字是审核范围基线。如果文档继续保留错误数量，后续审核报告会出现“缺文件”“多文件”一类假问题，或者遗漏真实文件。

建议:

不要在方法论正文里写死易漂移数量。建议改为:

```markdown
执行审核前必须采集当前范围基线，并写入最终报告:

- `find src -mindepth 1 -maxdepth 1 -type d | sort`
- `find src/cli/commands -maxdepth 1 -type f -name '*.rs' | sort`
- `find src/cli/handlers -maxdepth 1 -type f -name '*.rs' | sort`
- `find tests -maxdepth 1 -type f -name '*.rs' | sort`

方法论文档可保留最近一次观测值，但必须标注“执行时重新测量”。
```

---

### 2. Phase 1 顺序应调整

位置:

- `5.1 阶段划分`

当前 Phase 1:

```text
cargo build --release
cargo clippy
cargo fmt --check
cargo test
gitnexus analyze
```

问题:

后续架构扫描和执行流追踪高度依赖 GitNexus，但 `gitnexus analyze` 被放在 cargo gate 之后。若索引 stale，Phase 2 的图查询就会建立在旧结构上。

建议改为:

```text
Phase 1: 环境准备与基线采集
  ├── 读取 gitnexus://repo/quantix-rust/context，确认索引新鲜度
  ├── 如 GitNexus 提示 stale，先运行 gitnexus analyze
  ├── 采集范围基线（src/modules、commands、handlers、tests、config）
  ├── cargo fmt --check
  ├── cargo clippy --all-targets --all-features
  ├── cargo test --all-targets
  └── cargo build --release（作为发布/性能相关 gate，而非默认首个 gate）
```

补充建议:

`cargo build --release` 成本较高，且对代码审核初期价值低于 `fmt/clippy/test`。建议作为完整 gate 或发布 gate，而不是默认最先执行。

---

### 3. 工具命名需要去环境绑定

位置:

- `3.1 工具总览`
- `3.3 违规模式搜索清单`
- `8.1 效率准则`

当前文档使用:

- `grep_files`
- `read_file`
- `GitNexus context`
- `GitNexus impact`

问题:

这些名称偏 Claude Code 语境，且与当前 Codex/AGENTS.md 约束不完全一致。本项目明确要求文本/文件搜索优先用 `rg` 或 `rg --files`。GitNexus MCP 的实际工具名在当前环境是 `mcp__gitnexus__query/context/impact/detect_changes/rename/cypher`。

建议:

将工具矩阵改为“能力 + 推荐实现”的形式:

| 能力 | 推荐实现 | 说明 |
|------|----------|------|
| 文本搜索 | `rg`, `git grep` | 优先 `rg`，不可用再 fallback |
| 文件枚举 | `rg --files`, `find` | 需要稳定排序 |
| 文件读取 | 当前执行环境的 read/cat/sed 工具 | 记录引用行号 |
| 结构查询 | GitNexus `query/context/cypher` | 用于执行流、依赖、符号关系 |
| 影响分析 | GitNexus `impact` | 修改代码符号前必须执行 |
| 变更影响检查 | GitNexus `detect_changes` | 提交前必须执行 |
| 历史决策/审查记忆 | Graphiti MCP | 读写后验证 ingest |

---

### 4. “全面审核”与抽样规则不一致

位置:

- `2.1 代码库覆盖`
- `5.1 Phase 5`
- `4.5 测试覆盖与质量评估`

当前文档同时表达:

- 对项目进行全量代码审核
- P2/P3 模块只做入口文件快速审查
- 测试质量抽查至少 5 个集成测试文件

问题:

如果目标是“全面代码审核”，至少应保证全量自动扫描覆盖所有模块和测试文件。人工深审可以抽样，但抽样标准必须可复现。

建议拆成两层:

```markdown
全量覆盖:

- 所有 Rust 文件进入模式扫描、文件大小统计、公开 API/unwrap/panic/println/TODO/unsafe 检查。
- 所有 CLI command/handler 文件进入分发完整性检查。
- 所有 tests/*.rs 进入测试分类和外部依赖扫描。

人工深审:

- P0/P1 模块全量深审关键路径。
- P2/P3 每个模块至少审查:
  - 入口文件
  - 一个主要 service/provider/adapter
  - 一个持久化或外部依赖点（如存在）
  - 相关测试不少于 2 个；若模块测试少于 2 个则全量审查
```

测试抽样建议:

将“至少 5 个集成测试文件”改为“每个风险等级至少覆盖固定比例”:

```text
P0/P1 相关测试: 全量审查
P2/P3 相关测试: 至少 20%，且不少于每模块 2 个
repo hygiene / script / smoke 测试: 全量审查
```

---

### 5. 违规模式搜索需要定义统计口径

位置:

- `3.3 违规模式搜索清单`
- `4.4.1 规范合规性`
- `6.3 已知风险基线`

问题:

文档列了 regex，但没有定义:

- 排除哪些目录
- 如何区分生产代码和测试代码
- 如何处理误报
- 如何处理合理豁免
- 最终报告如何统计

建议增加固定统计口径:

```markdown
违规模式统计必须分桶:

- `src/` 生产代码
- `src/` 中 `#[cfg(test)]` 测试代码
- `tests/` 集成测试
- `examples/` / `benches/` 如存在
- docs/scripts/config 中的非生产命中

每个模式输出:

- total_matches
- production_matches
- test_matches
- exempted_matches
- actionable_matches
```

建议补充排除范围:

```text
排除 target/, .git/, .gitnexus/, .worktrees/, logs/, 构建产物和生成物。
```

---

### 6. 公共类型 derive 要求过宽

位置:

- `4.4.1 规范合规性`

当前规则:

```text
公共类型必须 derive(Debug, Clone, Serialize, Deserialize)
```

问题:

这不应作为全局硬规则。并非所有公共类型都应该 `Clone` 或 `Deserialize`，例如服务句柄、资源所有者、包含连接池或 trait object 的类型。强行执行会制造错误重构建议。

建议改为:

```markdown
公共类型 derive 规则按边界分类:

- CLI 参数/输出 DTO: 需要 Debug，必要时 Serialize
- 配置类型: 需要 Debug + Clone + Serialize + Deserialize
- 持久化/API 边界 DTO: 需要 Debug + Clone + Serialize + Deserialize，除非有明确原因
- 服务/资源所有者/连接句柄: 不强制 Clone/Serialize/Deserialize
- 领域 enum: 至少 Debug；是否 Clone/Serialize/Deserialize 按使用边界判定
```

---

### 7. findings 输出 schema 不足

位置:

- `5.2 每阶段输出`
- `7.2 辅助文档`
- `8.2 证据收集原则`

当前只要求:

- 发现清单
- 代码引用
- 建议操作

问题:

这不足以支持后续修复、验收和追踪。尤其是全面审核通常会产出大量问题，没有固定 schema 会导致 CSV、报告和 issue tracker 之间难以对齐。

建议增加标准 finding schema:

```markdown
每个 finding 必须包含:

- id: 稳定编号，如 AUDIT-S1-001
- severity: S0/S1/S2/S3/S4
- confidence: confirmed/probable/needs-repro
- module: 影响模块
- file:line: 精确证据位置
- evidence: 最小必要代码片段或命令输出摘要
- rule: 违反的规范、方法论检查项或设计边界
- impact: 对功能、资金安全、数据完整性、维护性的影响
- reproduction: 如何复现或验证
- recommended_fix: 建议修复方式
- acceptance_criteria: 修复完成的验收条件
- tests_required: 需要新增/更新/运行的测试
- owner_or_followup: 后续归属或跟踪项
- status: open/accepted/fixed/deferred/wontfix
```

CSV 也应以这个 schema 为列定义。

---

### 8. 风险分级决策树需要补充“真实交易门控”细则

位置:

- `6.1 风险等级定义`
- `6.2 判定决策树`

当前决策树将“涉及资金/持仓/真实交易”直接归为 S0。

问题:

这个方向是正确的，但还需要区分:

- 真实交易路径可达但未门控
- MOCK 被误表述为真实能力
- 真实路径失败后静默 fallback 到 MOCK
- 真实交易失败但错误回写不完整
- 仅文档/帮助文本混淆真实能力

建议:

```markdown
S0:

- 未门控真实下单
- 真实交易路径失败后静默 fallback 到 MOCK 或 paper
- 可能导致持仓/订单/成交记录丢失或错写

S1:

- 真实交易错误回写不完整，但不会继续提交错误订单
- 状态机处理遗漏导致核心链路不可用
- 生产路径 panic/unwrap 影响核心命令

S2:

- MOCK/真实能力在文档或 CLI 输出中混淆，但不改变实际执行路径
- 非核心路径错误上下文不足
```

---

### 9. Graphiti 集成需要与项目规则对齐

位置:

- `8.4 Graphiti 集成`
- `附录 C 交付检查`

当前写法:

```text
审核结束后，将 CRITICAL 和 HIGH 发现写入 Graphiti
```

问题:

项目 AGENTS.md 要求更严格:

- review handling 需要读 `quantix_rust_review`
- 设计意图可能相关时读 `quantix_rust_main`
- review conclusions 需要写 `quantix_rust_review`
- 每次写入必须验证 ingest completed

建议改为:

```markdown
审核开始前:

- 查询 `quantix_rust_review` 获取历史审查结论。
- 若涉及设计边界、命名、架构意图，查询 `quantix_rust_main`。
- 若涉及 bug/root cause，查询 `quantix_rust_debug`。

审核过程中:

- 不把 Graphiti 当作当前代码事实来源。
- 代码结构和调用关系以 GitNexus/源码为准。

审核结束后:

- 将最终 review conclusions 写入 `quantix_rust_review`。
- 将 S0/S1 关键发现摘要写入 `quantix_rust_review`。
- 如形成设计决策，另写 `quantix_rust_main`。
- 每次 `add_memory` 后必须记录 `episode_uuid`，并轮询 `get_ingest_status` 直到 `completed`。
```

---

### 10. 文档状态不宜宣称“全部技术错误已修正”

位置:

- 文档头部状态字段

当前写法:

```text
状态: 已修订 — 根据 review 意见修正全部技术错误
```

问题:

这类绝对表述容易过时，也会削弱后续审核者的验证意识。当前仍存在范围计数和执行口径问题，因此不宜写“全部技术错误”。

建议改为:

```text
状态: 候选版 — 已根据上一轮 review 修订，执行前仍需采集当前代码基线
```

---

## 三、建议补充的章节

### 1. 审核前基线采集

建议新增到 `五、审核工作流程` 前:

```markdown
## 审核前基线采集

正式审核前必须记录:

- 当前 git commit
- 工作区是否 dirty
- GitNexus index 状态
- Rust toolchain 版本
- cargo metadata 是否成功
- src 顶层模块列表
- CLI commands/handlers 文件列表
- tests 文件列表
- config 文件列表
- docs 关键文档列表
```

### 2. 发现项生命周期

建议新增:

```markdown
Finding 状态:

- open: 已确认，尚未处理
- accepted: 同意修复，等待排期
- fixed: 已修复并通过验收
- deferred: 有意延后，需说明原因
- wontfix: 不修复，需说明技术依据
- needs-repro: 证据不足，等待复现
```

### 3. 审核报告复现包

建议最终交付物增加:

```text
docs/CODE_AUDIT_EVIDENCE/
├── baseline.md
├── commands.txt
├── cargo-gates.md
├── gitnexus-queries.md
├── pattern-scan-summary.csv
└── sampled-files.md
```

这可以让后续 reviewer 复核每个结论如何得出。

---

## 四、建议的修订优先级

### P0: 使用前必须修

1. 修正或动态化 commands/handlers/tests 数量。
2. 调整 Phase 1，把 GitNexus freshness 检查前置。
3. 将工具命名改为环境无关，明确 `rg`/GitNexus/Graphiti 的角色。
4. 增加 finding schema。

### P1: 正式审核前应修

1. 明确 P2/P3 与测试的抽样规则。
2. 为违规模式搜索增加统计口径和排除规则。
3. 修正公共类型 derive 的硬规则。
4. 对风险分级增加真实交易门控细则。

### P2: 可在审核执行中迭代

1. 增加审核证据目录。
2. 增加 finding 生命周期定义。
3. 把阶段耗时改为估算范围，并注明依赖环境与 gate 耗时。

---

## 五、建议替换片段

### 文档状态

```markdown
> 版本: 1.2-draft
> 更新日期: 2026-05-11
> 状态: 候选版 — 已根据审查意见修订，执行前仍需采集当前代码基线
> 适用范围: quantix-rust 项目全量代码审核
```

### Phase 1

```markdown
Phase 1: 环境准备与基线采集
  ├── 记录 git commit 与 dirty 状态
  ├── 读取 GitNexus repo context，确认索引新鲜度
  ├── 如索引 stale，运行 gitnexus analyze
  ├── 采集模块/commands/handlers/tests/config/docs 范围基线
  ├── cargo fmt --check
  ├── cargo clippy --all-targets --all-features
  ├── cargo test --all-targets
  └── cargo build --release（完整 gate，可按环境成本延后）
```

### Finding Schema

```markdown
| 字段 | 说明 |
|------|------|
| id | 稳定编号，如 AUDIT-S1-001 |
| severity | S0/S1/S2/S3/S4 |
| confidence | confirmed/probable/needs-repro |
| module | 影响模块 |
| file:line | 精确证据位置 |
| evidence | 最小必要证据 |
| rule | 对应规范或设计边界 |
| impact | 影响说明 |
| reproduction | 复现或验证方式 |
| recommended_fix | 建议修复 |
| acceptance_criteria | 验收条件 |
| tests_required | 需要的测试或 gate |
| status | open/accepted/fixed/deferred/wontfix/needs-repro |
```

---

## 六、最终建议

建议不要直接用当前 `CODE_AUDIT_METHODOLOGY.md` 启动全面审核。先发布一个 `1.2-draft` 修订版，至少解决 P0 项，再开始正式审计。

建议下一步:

1. 按本审查意见修改 `CODE_AUDIT_METHODOLOGY.md`。
2. 用动态命令重新采集当前范围基线。
3. 用修订后的方法论做一次 P0 模块试审。
4. 根据试审结果再冻结方法论为 `1.2`。
