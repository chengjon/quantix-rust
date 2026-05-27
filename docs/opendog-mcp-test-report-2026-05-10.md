# OpenDog MCP 功能测试报告

**测试日期**: 2026-05-10
**测试项目**: quantix-rust (Rust, 3769 files)
**测试目标**: 验证 OpenDog MCP 全部 API 端点的功能正确性和实用性
**测试结论**: 17 个端点全部通过，功能完整可用

> 状态源说明：本文是测试报告，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 [`FUNCTION_TREE.md`](../FUNCTION_TREE.md) 的状态注册表行为准。

---

## 一、测试结果总览

| 分类 | 端点数 | 通过 | 说明 |
|------|--------|------|------|
| 项目管理 | 3 | 3 | 注册、列表、删除 |
| 配置 | 2 | 2 | 全局配置、项目配置 |
| 快照与比较 | 3 | 3 | 两次快照 + diff 比较 |
| 统计与分析 | 4 | 4 | 文件统计、未使用文件、时间窗口、趋势 |
| 数据风险检测 | 2 | 2 | 项目级 + 工作区级风险概览 |
| 验证与监控 | 4 | 4 | 启停监控、记录/查询验证结果 |
| 智能指导 | 1 | 1 | 多层分析引擎 |
| **合计** | **17** | **17** | — |

---

## 二、各端点详细结果

### 2.1 项目管理

#### `register_project` — 注册项目

- **参数**: `id=quantix-rust`, `path=/opt/claude/quantix-rust`
- **结果**: 成功注册，自动检测项目类型，返回结构化 guidance（建议先 take_snapshot 再 start_monitor）
- **响应时间**: 快速（<1s）

#### `list_projects` — 列出项目

- **结果**: 返回 2 个项目（mystocks 已在监控中 + 新注册的 quantix-rust）
- **额外信息**: 包含每个项目的状态（monitoring/active）、创建时间、根路径

#### `delete_project` — 删除项目

- **结果**: 成功删除 quantix-rust 及其关联数据库
- **测试时序**: 在所有测试完成后执行，确认数据清理正常

### 2.2 配置

#### `get_global_config` — 全局默认配置

- **结果**: 返回 ignore_patterns（22 项：node_modules, .git, target 等）和 process_whitelist（6 项：claude, codex, node, python 等）
- **实用性**: 默认配置覆盖了常见场景，无需手动配置

#### `get_project_config` — 项目级配置

- **结果**: 显示项目继承全局配置（inherits: true），无项目级覆盖
- **关键信息**: effective 配置 = global_defaults + project_overrides 合并结果

### 2.3 快照与比较

#### `take_snapshot` (第 1 次)

- **结果**: 扫描 3769 个文件，耗时约 6 秒
- **自动忽略**: target/, .git/ 等目录被正确排除

#### `take_snapshot` (第 2 次)

- **结果**: 0 新增、0 删除，正确识别无结构性变化
- **说明**: 两次快照间隔约 100 秒，仅 settings.local.json 发生了运行时修改

#### `compare_snapshots`

- **结果**: base_run(run_id=1, 3769 files) vs head_run(run_id=2, 3769 files)
- **变更检测**: 1 个文件被修改（.claude/settings.local.json），3768 个未变
- **检测粒度**: 文件级别（size + mtime），非内容级别

### 2.4 统计与分析

#### `get_stats` — 文件使用统计

- **结果**: 文件分类统计 — source: 2505, infrastructure: 120, project: 1144
- **局限性**: 因无活动数据，所有文件 access_count=0
- **建议**: 需先启动监控并产生实际工作流活动后，统计数据才有意义

#### `get_unused_files` — 未使用文件检测

- **结果**: 3769 个文件全部标记为未使用（因无监控活动数据）
- **候选排序**: 返回优先级排序的候选列表，附带验证建议命令
- **安全护栏**: 明确提示"零访问不等于可安全删除"，需要 shell 验证

#### `get_time_window_report` — 时间窗口报告

- **结果**: 24h 窗口内 10 次访问、6 个修改文件、32 个修改事件、1 个唯一进程
- **最活跃文件**: .claude/edit_log.jsonl（10 次访问）

#### `get_usage_trends` — 使用趋势

- **结果**: 24h 内 24 个 1 小时桶，8 个跟踪文件
- **趋势检测**: 活动集中在最后一个桶（当前会话期间），之前 23 个桶全部为零
- **增量计算**: 提供 delta_access_count 用于识别上升/下降趋势

### 2.5 数据风险检测

#### `get_data_risk_candidates` (hardcoded 过滤)

- **结果**: 5 个硬编码数据候选
  - `docker-compose.prod.yml` — business 关键词 (address, email, user) + 字面量标记 ($, @)
  - `docs/operations/DEPLOYMENT.md` — business 关键词 (client, order, email, user) + 字面量标记
  - 3 个 plan 文档 — business 关键词 (account, order, amount, user) + 字面量标记
- **规则引擎**: 使用 `content.business_literal_combo` 规则（high severity）
- **误报评估**: 大部分为文档中的模板变量和说明文字，非实际硬编码风险

#### `get_workspace_data_risk_overview`

- **结果**: 跨项目聚合 — 1 个项目有 hardcoded 候选，总候选数 5 hardcoded + 32 mock
- **优先级排序**: hardcoded > mixed > mock
- **决策支持**: 提供每个项目的 attention_score（quantix-rust: 155 分，critical 级别）

### 2.6 验证与监控

#### `start_monitor` / `stop_monitor`

- **启动**: 成功，开始 /proc 扫描 + inotify 变更检测
- **停止**: 成功，状态从 monitoring 变为 stopped
- **注意**: 监控需要一定时间积累活动数据才有效

#### `record_verification_result` + `get_verification_status`

- **操作**: 记录 `cargo test` 通过（exit_code=0, summary="All 42 tests passed"）
- **门控变化**:
  - 记录前: cleanup=blocked, refactor=blocked（缺少 test/lint/build 证据）
  - 记录后: cleanup=caution（允许，建议补充 lint/build）, refactor=blocked（仍缺 build）
- **实用性**: 验证门控系统确实能根据记录的证据动态调整安全级别

### 2.7 智能指导

#### `get_guidance` — 多层分析引擎

- **层级覆盖**: 7 个分析层（constraints_boundaries, execution_strategy, multi_project_portfolio, project_toolchain, repo_status_risk, storage_maintenance, verification_evidence, workspace_observation）
- **项目工具链**: 自动检测为 Rust，推荐 cargo check / cargo clippy / cargo test
- **仓库风险**: 检测到 34 个变更文件（large_working_diff），风险等级 medium
- **策略建议**: collect_evidence_first 模式 — 先收集证据再做高风险操作
- **存储分析**: 数据库 1.8MB，无碎片整理需求

---

## 三、已测试功能的实用价值评估

### 3.1 高实用价值

| 功能 | 实用场景 | 价值说明 |
|------|----------|----------|
| **验证门控系统** | CI/CD 流水线、发布前检查 | 记录 test/lint/build 结果后，自动判断是否安全进行 cleanup/refactor。这次测试证实了门控状态确实会随证据变化而更新 |
| **快照比较** | 代码审查、变更审计 | 精确检测文件级变化（新增/删除/修改），比 git status 更关注结构性变化 |
| **数据风险检测** | 安全审计、合规检查 | 自动发现硬编码敏感数据和 mock 数据残留，减少人工审查成本 |
| **Guidance 引擎** | 开发工作流决策 | 每次调用都返回上下文相关的下一步建议，减少决策盲目性 |

### 3.2 中等实用价值

| 功能 | 实用场景 | 局限性 |
|------|----------|--------|
| **文件监控** | 长期项目健康分析 | 需要持续运行才能积累有效数据；基于采样的检测可能遗漏短暂访问 |
| **未使用文件检测** | 代码瘦身、技术债务清理 | 依赖活动数据，新项目或短时间监控会产生大量误报 |
| **使用趋势** | 模块活跃度分析 | 需要较长监控周期（7d+）才能产生有意义的趋势 |

### 3.3 辅助价值

| 功能 | 说明 |
|------|------|
| **配置管理** | 必要但无差异化价值，功能符合预期 |
| **项目 CRUD** | 基础设施功能，稳定可靠 |

---

## 四、当前测试未覆盖但已存在的功能

### 4.1 `run_verification_command` — 执行并记录验证命令

本次测试使用了 `record_verification_result`（手动记录），但未测试 `run_verification_command`（实际执行命令并自动记录）。这个端点可以直接在 MCP 内运行 `cargo test` 并捕获结果，适合自动化流水线。

**建议**: 补充测试实际命令执行和结果自动解析。

### 4.2 `compare_snapshots` 的 base_run_id / head_run_id 参数

本次测试使用了默认的"最近两次快照"比较，未指定特定的 run_id。

**建议**: 补充测试跨任意两次快照的比较，验证历史对比能力。

### 4.3 长时间监控场景

本次测试仅运行了约 2 分钟的监控。OpenDog 的核心价值（未使用文件检测、使用趋势分析）需要较长的监控周期才能体现。

**建议**: 在一个实际开发会话中启动监控，持续 1-2 小时后检查统计数据的准确性。

---

## 五、功能建议（尚未实现但有实用价值）

### 5.1 内容级快照比较

**现状**: `compare_snapshots` 仅检测文件级变化（size + mtime）。

**建议**: 增加内容级 diff 能力，类似 `git diff --stat` 的输出，显示每个变更文件的增删行数。这能让用户在不离开 MCP 的情况下快速判断变更规模。

**实用场景**: 代码审查前的快速预检、发布前的变更范围确认。

### 5.2 验证结果的过期策略

**现状**: 验证结果记录后不会自动过期，需要新的记录覆盖旧结果。

**建议**: 增加验证结果的 TTL（Time To Live）配置。例如，test 结果 24h 后自动标记为 stale，build 结果 7d 后标记为 stale。这样可以自动提醒团队定期运行验证。

**实用场景**: 长期运行的分支，避免使用过时的验证结果做决策。

### 5.3 自定义验证门控规则

**现状**: 门控规则是硬编码的（cleanup 需要 test，refactor 需要 test + build）。

**建议**: 允许项目级自定义门控规则。例如，某些项目可能还要求 security scan 或 coverage threshold 作为门控条件。

**实用场景**: 安全敏感项目、合规要求严格的项目。

### 5.4 数据风险检测的上下文感知

**现状**: `get_data_risk_candidates` 基于关键词和字面量模式匹配，产生较多误报（如文档中的模板变量被标记为硬编码风险）。

**建议**:
1. 增加文件类型权重 — `.md` 文档中的 business 关键词应降低优先级
2. 增加上下文分析 — 区分 YAML 模板变量（`${VAR}`）和实际硬编码值
3. 支持用户标记误报（白名单机制），后续扫描不再报告

**实用场景**: 减少审计噪音，聚焦真正的安全风险。

### 5.5 文件依赖图分析

**现状**: OpenDog 跟踪文件访问和修改，但不理解文件间的依赖关系。

**建议**: 结合文件访问模式构建隐式依赖图。例如，如果每次编辑 `A.rs` 后紧接着编辑 `B.rs`，可以推断它们可能相关。这能帮助评估变更的影响范围。

**实用场景**: 变更影响评估、代码审查优先级排序。

### 5.6 项目健康评分

**现状**: Guidance 返回分散的多层分析，但没有一个聚合的"健康分数"。

**建议**: 基于验证状态、数据风险、代码活跃度、未使用文件比例等维度，计算一个 0-100 的项目健康评分。可以在 `list_projects` 时直接展示，帮助快速定位最需要关注的项目。

**实用场景**: 多项目管理、团队周报、技术债务跟踪。

### 5.7 增量快照

**现状**: 每次 `take_snapshot` 都执行完整文件系统扫描（3769 个文件）。

**建议**: 基于上一次快照和 inotify 事件，仅扫描有变化的文件。对于大型项目（10K+ 文件），可以显著减少扫描时间。

**实用场景**: 大型代码库的频繁快照、CI 环境中的快速检查。

### 5.8 与 Git 集成的变更关联

**现状**: `compare_snapshots` 和 `detect_changes` 是独立的，不与 git commit/branch 关联。

**建议**: 允许快照关联 git ref（commit hash 或 branch name），这样可以在 PR 审查时快速回答"这个 PR 改了哪些文件，其中有多少是活跃使用的"。

**实用场景**: PR 审查、发布审查、分支对比。

### 5.9 通知和告警

**现状**: OpenDog 是被动查询模式，不会主动推送信息。

**建议**: 增加可配置的告警规则：
- 新增硬编码数据候选时告警
- 验证结果从 pass 变为 fail 时告警
- 未使用文件超过阈值时告警

**实用场景**: 安全审计、CI/CD 集成、团队协作。

### 5.10 MCP 资源（Resources）支持

**现状**: OpenDog 仅提供工具（Tools），未暴露 MCP 资源（Resources）。

**建议**: 将关键状态暴露为 MCP 资源 URI，例如：
- `opendog://projects` — 项目列表
- `opendog://project/{id}/health` — 项目健康摘要
- `opendog://project/{id}/verification` — 验证状态

这样 Claude 可以通过 `ReadMcpResource` 而非调用工具来获取只读状态，减少 token 消耗。

---

## 六、总结

OpenDog MCP 的核心功能完整且稳定，17 个端点全部通过测试。其最大价值在于：

1. **验证门控** — 为 cleanup/refactor 操作提供基于证据的安全判断
2. **数据风险检测** — 自动发现硬编码和 mock 数据残留
3. **智能指导** — 每次调用都返回上下文相关的策略建议

主要改进方向集中在减少误报（数据风险检测的上下文感知）、提高效率（增量快照）、和增强集成（git 关联、MCP 资源）。

OpenDog 适合在以下场景中持续使用：
- 作为 Claude Code 会话的常驻上下文层，提供项目状态感知
- 在 CI/CD 流水线中作为质量门控
- 在安全审计中作为自动化的数据风险扫描器
