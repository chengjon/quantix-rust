# 审核意见：Dirty Worktree Cleanup Guide

> 审核对象：`docs/guides/DIRTY_WORKTREE_CLEANUP_GUIDE.md`（845 行）
>
> 审核日期：2026-05-27

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
