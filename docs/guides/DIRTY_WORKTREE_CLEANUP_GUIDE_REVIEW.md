# 审核意见：Dirty Worktree Cleanup Guide

> 审核对象：`docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md`（845 行）
>
> 审核日期：2026-05-27
>
> 状态源说明：本文是指南审查与规则沉淀记录，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 [`FUNCTION_TREE.md`](../../FUNCTION_TREE.md) 的状态注册表为准。

---

## 一、严重问题：编号体系错位

这是本文最突出的结构缺陷。开篇"总流程"定义了步骤 0–9 的编号体系，但后续实际章节编号与总流程存在系统性偏差：

| 总流程步骤 | 实际章节标题 | 偏差 |
|---|---|---|
| 3. Clean review base | **4.** Clean Review Worktree | +1 |
| 4. Slice extraction | **5.** Slice Extraction | +1 |
| 5. Slice validation | **6.** Product Code Rules | 编号不同，语义漂移 |
| 6. PR / merge | **8.** PR And Commit Strategy | +2 |
| 7. Root tracked realignment | **9.** Root Tracked Realignment | +2 |
| 8. Residual untracked disposition | **10.** Residual Untracked Disposition | +2 |
| 9. Final cleanup | **11.** Final Cleanup | +2 |

此外：
- 第 3 节"Explicit Approval Protocol"在总流程中无对应步骤。
- 第 7 节"Generated And Runtime Artifacts"在总流程中无对应步骤。
- 总流程中的"5. Slice validation"无独立章节，被并入"Product Code Rules"且语义缩小。

**建议**：以总流程 0–9 为权威编号，重排后续所有章节标题。Explicit Approval Protocol 和 Generated Artifacts 作为子节插入对应步骤。

---

## 二、中等问题

### 2.1 分类信息双重存在，维护成本高

第 1 节（Inventory）末尾的 Bucket 表格：

```text
| Product code | src/, tests/, benches/ | 只能切片提取，必须测试。 |
| Docs/governance | docs/, adr/ | 可独立 docs PR |
| Generated/runtime artifacts | logs/, target/ | 先归档，后路径级删除 |
| Local tool config | .mcp.json, .env | 默认不提交 |
| Backups/snapshots | timestamped backups | 先保留 |
| Unknown | 无法判断来源的文件 | 不删除，先生成 disposition |
```

第 1.5 节（Classification）的 Class 表格：

```text
| 有效代码 | src/, tests/, benches/ | 切片提取、影响分析、测试 |
| 正式文档 | 架构报告、ADR | 交叉引用检查后 docs PR |
| 临时草稿 | review draft, brainstorm | 默认保留在 recovery |
| 运行产物 | logs, coverage | 归档后路径级删除 |
| 本地配置 | .env, .mcp.json | 默认 keep local |
| 垃圾文件 | 临时缓存、空目录 | 快照覆盖后精确删除 |
```

两者高度重叠但分类粒度不同（Bucket 分 6 类，Class 分 6 类且语义不完全一致）。读者需要对照两份表格确认体系关系。

**建议**：合并为一份权威分类表，放在 1.5 Classification 下。Inventory 只保留命令模板和收集动作。

### 2.2 "原则到步骤的映射"表引用不准确

当前映射表：

```text
| 不混提交 | 5 Slice Extraction, 8 PR And Commit Strategy |
| 代码切片必须闭环 | 6 Product Code Rules |
| 先行快照再整理 | 0 Freeze, 2 Recovery Snapshot |
| 先在干净工作树提取切片再动根工作树 | 3 Clean Review Worktree, 4 Slice Extraction |
| 主干对齐后置 | 9 Root Tracked Realignment |
```

问题：
- "5 Slice Extraction"引用的是旧编号。
- "不混提交"原则与 PR And Commit Strategy 的关系比 Slice Extraction 更直接。
- 所有引用应在统一编号后刷新。

**建议**：统一编号后重建此表，确保每个引用指向正确的章节号。

### 2.3 步骤 5（Slice Validation）语义漂移为 Product Code Rules

总流程定义的 "Slice validation" 是通用验证环节，应覆盖所有切片类型。但实际章节标题"Product Code Rules"只覆盖代码切片，遗漏了文档切片、治理切片、配置切片的验证规则。

**建议**：两种方案择一：
- **方案 A**：将第 6 节改名为 "Slice Validation"，在其中按切片类型分小节给出验证规则。
- **方案 B**：新增独立的 "Slice Validation" 节，将 "Product Code Rules" 作为其子节。

### 2.4 `git stash --include-untracked` 两次出现在不同禁令列表中

| 出处 | 列表内容 |
|---|---|
| 第 3 节（Explicit Approval Protocol） | 列出 6 条禁止命令 |
| High-Risk Operation Blacklist | 表格列出 8 项高风险操作 |

两处都包含 `git stash push --include-untracked`，但表格给出了原因和替代方案，而第 3 节只给了命令名。重复维护两份不同步的列表容易产生偏差。

**建议**：第 3 节仅做"参见 High-Risk Operation Blacklist"的交叉引用，Blacklist 作为唯一权威黑名单。

---

## 三、轻微问题与改进建议

### 3.1 缺少关闭 clean review worktree 的清理步骤

文档指导用户创建 `.worktrees/dirty-cleanup-review-base`，但 Final Cleanup 中未提及何时及如何清理这个 worktree。`git worktree remove` 和关联分支的 `git branch -d` 应在最终阶段执行。

**建议**：在 Final Cleanup 的可清理对象清单中增加：

```text
- Clean review worktree（.worktrees/dirty-cleanup-review-base）及其关联分支 cleanup/dirty-worktree-review-base-YYYY-MM-DD。
```

### 3.2 `phase0-manifest.json` 未定义

第 2 节（Recovery Snapshot）的保存清单中列出了 `phase0-manifest.json`，但全文无结构说明或内容示例。执行者不知道应该在这个文件中写什么。

**建议**：给出最小 JSON schema：

```json
{
  "created_at": "YYYY-MM-DDTHH:MM:SSZ",
  "repo_path": "/absolute/path",
  "original_head": "abc123f",
  "tracked_diff_sha256": "...",
  "tracked_diff_bytes": 12345,
  "untracked_archive_sha256": "...",
  "untracked_archive_bytes": 67890,
  "inventory_errors": 0,
  "missing_required_files": []
}
```

### 3.3 `git apply --check` 验证的限制条件未说明

第 2 节说 "如果 `git apply --check` 在当前脏树里不适合运行，创建一次性 worktree 或临时 clone 验证"，但未说明什么情况下"不适合"。实际常见原因是：tracked.diff 包含新文件（`new file mode`），而当前工作树中这些文件可能被 `--intent-to-add` 标记过但内容不匹配，导致 `git apply --check` 失败。

**建议**：补充具体限制说明和替代验证命令：

```bash
# 如果在脏树中 apply --check 失败：
# 原因通常是 diff 包含 new file mode 且本地状态不一致
# 替代：在临时 clone 中验证
git clone --no-checkout . /tmp/verify-restore
cd /tmp/verify-restore
git apply --check /path/to/tracked.diff
```

### 3.4 恢复说明模板缺失

第 2 节要求 `restore-instructions.md` 是恢复包的一部分，但未给出内容模板或最小要求。

**建议**：在 Recovery Snapshot 节增加最小模板：

```markdown
# Restore Instructions

- **Created**: YYYY-MM-DD HH:MM UTC
- **Repository**: /absolute/path/to/repo
- **Original HEAD**: abc123f
- **Original branch**: feature/foo

## Restore tracked changes

    git apply var/recovery/dirty-worktree-YYYY-MM-DD/tracked.diff

## Restore untracked files

    tar -xf var/recovery/dirty-worktree-YYYY-MM-DD/untracked-files.tar -C <target-directory>

## Rescue branch

    git branch rescue/dirty-worktree-YYYY-MM-DD  # already created
    # To restore: git checkout rescue/dirty-worktree-YYYY-MM-DD
```

### 3.5 Acceptance Baselines 中一行可操作性弱

当前：

```text
| 功能可闭环 | 代码切片通过相关测试、格式化、lint 和影响分析。 |
```

"影响分析"是过程，不是可检查的基线产物。读者不知道怎样才算"通过"影响分析。

**建议**：

```text
| 功能可闭环 | 代码切片通过相关测试、格式化、lint；影响分析报告已生成，且无 HIGH/CRITICAL 未解决项。 |
```

### 3.6 "Recommended Document Set" 的 fallback 路径缺少目录前缀

文档推荐产出 `openspec/changes/<cleanup-change>/`，对非 OpenSpec 项目给出 fallback：

```text
cleanup-plan.md
cleanup-tasks.md
cleanup-policy.md
closure-summary.md
recovery/restore-instructions.md
```

这些路径缺少目录前缀，会散落在仓库根目录。

**建议**：

```text
docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_YYYY-MM-DD.md
docs/reports/DIRTY_WORKTREE_CLEANUP_TASKS_YYYY-MM-DD.md
docs/reports/DIRTY_WORKTREE_CLEANUP_POLICY.md
docs/reports/DIRTY_WORKTREE_CLEANUP_CLOSURE_SUMMARY_YYYY-MM-DD.md
var/recovery/dirty-worktree-YYYY-MM-DD/restore-instructions.md
```

### 3.7 缺少多分支脏线的处理提示

当前指南假设脏线集中在当前分支。如果用户同时有几个本地分支都有未推送改动（stash + 多个 dirty branches），清理策略会更复杂。

**建议**：在适用场景表格中增加一行：

```text
| 多分支脏线 | 多个本地分支都有未提交改动，stash 不为空。 | 未纳入工作流的分支管理。 |
```

处理方法可简化为 Common Failure Modes 中的一条：

```text
### 8. 忽略多分支脏线

只在当前分支做清理，但其他分支也积累了未提交改动，后续切分支时出现冲突或意外覆盖。

替代：

    先 git worktree list 和 git branch -vv 列出所有分支状态，
    为每个 dirty branch 单独做 inventory，再按依赖顺序逐一清理。
```

### 3.8 命令附录中 `cargo test` 与通用性声明的轻微矛盾

第 825 行：

```bash
cargo test
```

文档目标声明"不绑定具体技术栈"，但命令附录中使用了 Rust 特定命令。

**建议**：

```bash
# Run tests for this project (e.g., cargo test, pytest, npm test, go test ./...)
```

### 3.9 缺少 `--porcelain=v1` 选择说明

文档始终使用 `git status --porcelain=v1`，但 Git 2.11+ 支持 v2 格式（面向解析器的优化格式）。读者可能疑惑为何不选 v2。

**建议**：在命令附录首行加注释：

```bash
# --porcelain=v1 用于最大兼容性：v1 输出稳定，跨 Git 版本一致。
# v2 格式（--porcelain=v2）适合增量解析但兼容性窗口较窄。
```

---

## 四、可直接采纳的逐行修正

| 行号（约） | 当前内容 | 建议修改 |
|---|---|---|
| 99 | `10 Residual Untracked Disposition` | 统一为正确步骤编号 |
| 106 | `6 Product Code Rules` | 统一为正确章节编号 |
| 107 | `9 Root Tracked Realignment` | 统一为正确章节编号 |
| 304 | `Inventory \| Yes` | 此处说 Inventory 需要批准，但 Inventory 是只读操作，建议改为 `Not destructive` 或与只读原则一致 |
| 514 | 对齐后验证命令块 | 建议追加 `git diff --stat origin/master` 作为额外肉眼确认 |
| 825 | `cargo test` | 改为 `# Run project tests` 并注释多语言适配 |
| — | 第 3 节 Explicit Approval Protocol | 删除重复的禁止命令列表，统一引用 Blacklist |

---

## 五、总体评价

### 优点

- **原则明确**：8 条刚性约束覆盖了保全→分类→提取→审批→对齐的全链路，且不依赖具体工具栈。
- **流程设计合理**：冻结→清点→快照→干净工作树→切片→合并→对齐→残余处置的流水线逻辑清晰。
- **务实**：反场景和适用场景表格过滤掉不需要本指南的简单情况，防止过度使用。
- **Blacklist 表格**是全文最有价值的部分之一，每条高风险操作都给了原因和安全替代。

### 主要改进方向

1. **修复编号错位**——最影响可读性的事，应最先处理。
2. **合并冗余表格**——Inventory bucket 和 Classification class 合并后减少对照成本。
3. **补充缺失的操作细节**——restore-instructions 模板、clean worktree 删除步骤、phase0-manifest 定义。
4. **区分验证步骤和规则约束**——Slice Validation 应成为独立环节，Product Code Rules 作为其子集。
5. **消除技术栈耦合**——命令附录中 Rust 特定命令改为通用占位。

整体质量较高，上述修正是打磨性质的。

---

## 六、本次执行后的补充规则沉淀（2026-06-03）

> 背景：本轮在清理脏线后继续完成 PR #190、#191、#192 三个小切片，并多次执行 post-merge `gitnexus analyze`、worktree/branch 清理和 Graphiti closeout。以下经验应回填到正式指南 `docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md`，作为 Final Cleanup、Residual Disposition 和后续开发切片的补充规则。

### 6.1 明确区分“脏线清理完成”和“后续开发切片”

脏线清理结束的判定不应只是“当前没有未提交业务文件”，还应包含：

```text
- 根 worktree 已回到目标主线分支；
- `git status --short --branch` 干净；
- 本轮保留项已有明确 disposition；
- 后续开发任务已转入隔离 worktree 或独立分支；
- post-merge 工具刷新和记忆写入已经完成或明确记录 backfill。
```

本次 #190–#192 的经验说明：脏线清理完成后继续开发时，不应在刚清干净的 root worktree 上累积新改动。每个后续任务都应按独立小切片执行：

```text
new worktree -> TDD red/green -> docs/hygiene guard -> local gates -> PR -> CI -> merge -> post-merge cleanup
```

这条规则能防止“清理脏线”工作重新变成新的混合脏线来源。

### 6.2 `.mcp.json` 与 `var/` 需要单独 disposition，不能混入普通残余处置

本轮形成的稳定约束是：

```text
.mcp.json
  默认视为本地 MCP/agent 配置。
  若用户确认加入 ignore，则只提交 ignore 规则，不提交文件内容。

var/
  默认视为本地运行时、恢复、缓存或证据残留目录。
  除非用户明确要求处理具体路径，否则保持不动。
```

建议正式指南在 Residual Untracked Disposition 中增加“special local residuals”小节，明确：

- `.mcp.json`、`.env`、本地凭据类文件优先 `ignore/keep local`，不要进入功能 PR。
- `var/` 这类运行时目录必须先做 owner/path disposition；没有明确 owner 或清理授权时保持不动。
- 最终汇报必须显式说明这些保留项是否被触碰。本轮收口时使用了 `touches_mcp_json=false`、`touches_var=false` 这类检查，效果清晰。

### 6.3 GitNexus analyze 噪声应作为已知工具刷新，不得用宽泛 reset 处理

本轮每次 merge 后运行 `gitnexus analyze` 都可能改写以下工具说明文件：

```text
AGENTS.md
CLAUDE.md
.claude/skills/gitnexus/gitnexus-cli/SKILL.md
.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md
.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md
```

这些变化通常是 GitNexus instruction block / skill 文档刷新噪声，不属于业务改动。正式指南应增加 post-merge 检查模板：

```text
1. `git status --short --branch`
2. 若只出现 GitNexus instruction/skill 噪声，先查看 diff 摘要确认来源。
3. 只恢复已确认的噪声文件：
   `git restore AGENTS.md CLAUDE.md .claude/skills/gitnexus/...`
4. 不使用 `git reset --hard` 或宽泛 checkout。
5. 再次确认 `git status --short --branch` 干净。
```

关键点是“先确认 diff，再精确恢复”。即使这些文件多次出现，也不能因为熟悉而跳过确认。

### 6.4 post-merge cleanup 应包含工具状态和记忆状态，不只是 Git 分支清理

本轮较可靠的 post-merge checklist 是：

```text
- PR merged / merge commit captured
- root worktree `git pull --ff-only`
- feature worktree removed
- local feature branch deleted
- remote feature branch deleted or confirmed deleted
- `gitnexus analyze` completed
- GitNexus metadata points to current HEAD
- GitNexus-generated instruction noise restored if present
- `.mcp.json` / `var/` / external paths confirmed untouched
- Graphiti closeout memory written
- Graphiti ingest status verified as completed
- final `git status --short --branch` clean
```

建议正式指南把这组检查放到 Final Cleanup，而不是散在 PR / merge 和 Recovery sections 之间。原因是：PR 合并成功并不代表工作结束；索引刷新、工具噪声恢复、语义记忆写入和 root worktree 干净确认都属于同一个收口动作。

### 6.5 外部路径必须先确认 repo identity，不能直接合入当前项目

本轮出现过误把 `/opt/claude/GitNexus/openspec/changes/2026-06-02-p0-p1-p2-review/review.md` 当作本项目 quantix 任务输入的情况，随后用户明确撤回该合并指令。

正式指南应增加一条外部路径规则：

```text
如果用户给出的文件路径不在当前 repository root 下：
  1. 先确认该路径所属 repo；
  2. 判断它是外部参考、跨项目输入，还是当前项目应纳入的 artifact；
  3. 未确认前不得 copy/move/merge 到当前 repo；
  4. 如果用户撤回或纠正，应在后续 summary 中明确“已忽略该外部路径”。
```

这能避免跨项目 review、OpenSpec change 或临时 evidence 被误归档到错误仓库。

### 6.6 长运行工具句柄丢失时，用状态交叉验证，不要假设结果

本轮 `gitnexus analyze` 曾在长运行过程中跨上下文恢复，原 session handle 已失效。可靠做法不是声明成功，而是重新验证：

```text
- `ps -ef` 确认没有仍在运行的 analyze 进程；
- `git status --short --branch` 查看是否留下文件变更；
- `.gitnexus/meta.json` 或 GitNexus tool 状态确认 indexed commit 是否等于 HEAD；
- 若留下工具噪声，按 6.3 精确恢复；
- 再做最终 clean status。
```

正式指南中应把这类“resume after long-running tool”放入 Common Failure Modes。否则清理任务跨 compact/resume 后，容易误把未完成工具步骤当成完成状态。

### 6.7 CI 外部状态不要伪装成代码问题

本轮 CI 曾受仓库 visibility / billing / public 状态影响，用户调整仓库状态后才继续推进。指南应补充：

```text
CI blocked by external account/repository setting != code failure.
```

处理原则：

- 先记录 CI 的真实阻塞原因和时间点。
- 不为外部 CI 状态随意改代码、改测试或扩大 diff。
- 外部状态解除后，重新触发或等待 CI，再继续 merge。
- closeout summary 中区分“本地 gate passed”和“remote CI passed/blocked”。

### 6.8 Graphiti / 记忆写入也是收口证据，但不是状态权威

本轮 closeout memory 的做法是：PR 合并、GitNexus analyze、worktree/branch 清理、root clean 都完成后，再写 Graphiti，并轮询 ingest 到 `completed`。

正式指南可以增加：

```text
Graphiti memory is closure evidence, not task status authority.
```

建议规则：

- 只有在事实收敛后写入：merge commit、CI 结果、清理结果、保留项 disposition。
- 写入后必须记录 episode UUID 并验证 ingest status。
- 如果 Graphiti 不可用，写本地 backfill 摘要并标注 `Graphiti backfill required`。
- 不用 Graphiti 代替 `git status`、CI 状态或 GitNexus detect/analyze 结果。

### 6.9 建议追加到正式指南的最小补丁清单

正式 `DIRTY_WORKTREE_CLEANUP_GUIDE.md` 建议最少增加这些内容：

1. 在 Residual Untracked Disposition 中加入 `.mcp.json` / `var/` special-case policy。
2. 在 Final Cleanup 中加入 post-merge GitNexus analyze / instruction-noise restore / clean status checklist。
3. 在 Common Failure Modes 中加入“long-running tool session lost after resume”的交叉验证办法。
4. 在 Scope / Applicability 中加入“external path must confirm repo identity”规则。
5. 在 CI / PR 流程中加入“external CI blocker is not code failure”的处理原则。
6. 在 Closure Summary 模板中加入 Graphiti closeout / backfill 字段。
