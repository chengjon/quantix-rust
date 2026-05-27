# Dirty Worktree Cleanup Guide

本文是一份可迁移到其他项目的脏工作树清理指南，适用于已经积累了大量未提交改动、未跟踪文件、运行产物、文档草稿、工具配置和分支漂移的仓库。

核心目标不是“尽快让 `git status` 变干净”，而是在不丢失有效工作、不误提交本地噪声、不破坏主干的前提下，把脏线拆成可审核、可恢复、可合并的最小变更。

## 目标定义

脏线清理要区分两种“干净”。

| State | Meaning | Risk |
| --- | --- | --- |
| 表面干净 | `git status` 没有输出，或者脏文件被一次性 reset/clean 掉。 | 可能丢失有效工作、删除证据、污染审计链路。 |
| 实质合规干净 | 有价值变更进入主干，无价值产物可恢复地清除，本地配置未泄漏，剩余风险有 disposition 记录。 | 可审计、可回滚、可交付。 |

本指南追求的是“实质合规干净”。因此，任何清理动作都必须满足三条刚性约束：

1. 有快照兜底：清理前能恢复 tracked diff 和 untracked files。
2. 不破坏主干：主干对齐只能在选定切片合并后执行。
3. 可迁移无业务耦合：流程只依赖 Git、文件系统、审查记录和项目自己的验证命令，不绑定具体技术栈。

## 适用场景

使用本指南处理以下情况：

| Scenario | Typical symptom | Engineering cause |
| --- | --- | --- |
| 多类型脏文件混杂 | 代码、文档、测试、日志、工具缓存和本地配置同时出现。 | 工作流没有把正式变更、运行产物和本地配置分层管理。 |
| 主干长期漂移 | `master` / `main` 落后远端，但本地还有大量未整理改动。 | 未及时 rebase/merge，且本地改动无法直接丢弃。 |
| 多代理或多工具连续写入 | 不知道哪些文件来自用户、代理、脚本、测试或生成器。 | 缺少单一执行者和操作审计记录。 |
| 多分支脏线 | 多个本地分支都有未提交改动，stash 不为空，worktree 数量多。 | 分支管理和任务隔离没有形成闭环。 |
| 需要从脏树提取价值 | 有些改动应该进主干，有些只是中间产物。 | 缺少切片提取和 path-scoped review 机制。 |
| 运行证据需要留存 | `logs/`、`var/`、`docs/reports/evidence/` 等既像产物又像审计证据。 | 没有 evidence retention 或归档策略。 |

反场景：

- 只有 1-3 个明确文件，且操作者清楚每个文件的来源。
- 只有格式化输出或缓存目录，且 `.gitignore` 已经覆盖。
- 刚开始一个功能分支，没有历史脏线和多人/多代理混写。

这些反场景不需要完整 SOP。正常使用 `git add`、`git commit`、`git restore` 或精确路径删除即可，避免用重流程消耗时间。

## 基本原则

1. 先保全，再整理。
2. 先分类，再删除。
3. 先在干净工作树提取切片，再动根工作树。
4. 每一类高风险动作都需要单独批准。
5. 不用 `git clean` 做探索性清理。
6. 不把运行产物、本地工具配置和候选文档混在同一个提交里。
7. 代码切片必须有影响分析和测试闭环。
8. 主干对齐是最后阶段，不是开场动作。

这些原则是后续 SOP 的强制约束，不是建议项。

| Principle | Operational meaning |
| --- | --- |
| 先保全，再整理 | 必须先生成 recovery snapshot、救援分支和恢复说明，再做任何 reset/delete/move。 |
| 先分类，再删除 | 删除前必须知道路径属于有效代码、正式文档、临时草稿、运行产物、本地配置还是垃圾文件。 |
| 先在干净工作树提取切片，再动根工作树 | 根工作树作为 salvage source；可合并内容在 clean review worktree 中重建。 |
| 每一类高风险动作都需要单独批准 | broad approval 不等于允许 reset、clean、rm、root realignment。 |
| 不用 `git clean` 做探索性清理 | `git clean` 只能在 disposition 和路径级审批后作为最后手段使用。 |
| 不混提交 | 每个提交只能代表一种意图，例如 docs、governance、test、code 或 artifact policy。 |
| 代码切片必须有影响分析和测试闭环 | 代码不是“文件搬运”；必须验证调用链、行为和测试结果。 |
| 主干对齐后置 | 只有选定切片已经进入主干，才能考虑 reset 根工作树到远端主干。 |

## 术语

| Term | Meaning |
| --- | --- |
| Dirty worktree | `git status` 中存在 tracked drift、untracked files、deleted files 或 staged changes 的工作树。 |
| Root worktree | 当前脏线所在的主工作树，通常是用户正在使用的目录。 |
| Salvage source | 在清理完成前保留不动的原始脏工作树，用于回看、对照和恢复。 |
| Recovery snapshot | 清理前生成的恢复包，包含 tracked diff、untracked archive、状态清单、分支清单和恢复说明。 |
| Clean review worktree | 从干净主干创建的临时工作树，用来提取可审核切片。 |
| Slice | 可独立审核、测试、提交和回滚的一组路径或行为变更。 |
| Path-level approval | 针对明确路径组的删除、移动或归档批准。 |
| Root realignment | 将根工作树对齐到远端主干，例如 `git reset --hard origin/master`。这是高风险操作。 |

## 总流程

```text
0. Freeze
1. Inventory and classification
2. Recovery snapshot
3. Clean review base
4. Slice extraction
5. Slice validation
6. PR / merge
7. Root tracked realignment
8. Residual untracked disposition
9. Final cleanup
```

每一步都应该可以暂停，并且暂停后仍能从文档和恢复包恢复上下文。

原则到步骤的映射：

| Principle | SOP step |
| --- | --- |
| 先保全，再整理 | 0 Freeze, 2 Recovery Snapshot |
| 先分类，再删除 | 1 Inventory and Classification, 8 Residual Untracked Disposition |
| 先在干净工作树提取切片，再动根工作树 | 3 Clean Review Worktree, 4 Slice Extraction |
| 高风险动作单独批准 | 0.1 Explicit Approval Protocol, High-Risk Operation Blacklist, 9 Final Cleanup |
| 不用 `git clean` 探索清理 | High-Risk Operation Blacklist, 8 Residual Untracked Disposition |
| 不混提交 | 4 Slice Extraction, 6 PR And Commit Strategy |
| 代码切片必须闭环 | 5 Slice Validation |
| 主干对齐后置 | 7 Root Tracked Realignment |

## 0. Freeze

清理开始前先冻结现场。

要求：

- 指定一个执行者，避免多个代理同时写同一个工作树。
- 停止自动格式化、代码生成、后台任务或会持续写日志的进程。
- 记录当前分支、远端、HEAD、worktree 列表和 dirty 状态。
- 明确哪些动作还没有批准，尤其是 reset、clean、rm、stash。

推荐先声明边界：

```text
本阶段只做清点和快照。
不执行 git reset、git clean、git restore、rm -rf、rebase、stash --include-untracked。
```

### 0.1 Explicit Approval Protocol

不要用一句“继续清理”覆盖所有高风险动作。将批准拆成阶段和路径组。

| Action | Approval needed | Notes |
| --- | --- | --- |
| Inventory | Not destructive | 只读清点不需要破坏性批准；写入恢复目录时按 snapshot 阶段批准。 |
| Recovery snapshot | Yes | 允许创建恢复目录、tar、manifest、救援分支。 |
| Clean review worktree | Yes | 允许创建新 worktree 和分支。 |
| Docs extraction | Per slice | 明确源路径、目标分支、提交边界。 |
| Code extraction | Per domain | 必须先做影响分析，再改符号。 |
| Generated artifact disposal | Per path group | 必须先证明已归档或不需要归档。 |
| Root realignment | Separate explicit approval | 最高风险；只在切片合并后做。 |
| Final untracked deletion | Per path group | 不允许 blanket delete。 |

禁止把高风险命令藏在“继续”里执行。唯一权威黑名单见 `High-Risk Operation Blacklist`；那些命令只有在恢复包已验证、路径范围明确、用户明确批准后才能执行。

## 1. Inventory

目标是把脏线从“看起来很多”转化成可处理的类别。

建议收集：

- 当前分支和 HEAD。
- 本地分支列表。
- 远端主干位置。
- `git status --porcelain=v1`。
- tracked diff。
- untracked 文件列表、大小和哈希。
- worktree 列表。
- stash 列表。

命令模板：

```bash
git status --porcelain=v1 > status-porcelain.txt
git diff --binary > tracked.diff
git branch --list > branch-list.txt
git stash list > stash-list.txt
git worktree list --porcelain > worktree-list.txt
git rev-parse HEAD > local-head.txt
git ls-remote origin refs/heads/master > remote-master.txt
```

如果项目主干叫 `main`，把 `master` 替换成 `main`。

Inventory 只负责收集事实；分类结论以 `1.1 Classification` 的标准分类表为准，避免维护两套分类模型。

### 1.1 Classification

清理前必须把脏文件标准化归类。不要只看路径名，要结合内容、来源、是否可重建、是否有审计价值来判断。

| Class | Content range | Main question | Default action | Can merge to trunk? |
| --- | --- | --- | --- | --- |
| 有效代码 | `src/`, `tests/`, `benches/`, migrations, scripts | 是否改变产品行为或验证能力？ | 切片提取、影响分析、测试。 | 可以，但必须独立提交。 |
| 正式文档 | 架构报告、ADR、runbook、spec、policy | 是否是团队后续要引用的权威材料？ | 交叉引用检查后 docs PR。 | 可以。 |
| 临时草稿 | review draft、brainstorm、旧计划、未定稿 spec | 是否已被正式记录替代？ | 默认保留在 recovery，必要时单独转正。 | 不直接合并。 |
| 运行产物 | logs、coverage、test timing、raw evidence、generated reports | 是否可重建或已归档？ | 归档后路径级删除。 | 通常不合并。 |
| 本地配置 | `.env`, `.mcp.json`, IDE settings, local credentials | 是否只对当前机器有效？ | 默认 keep local 或加入 ignore。 | 通常不合并。 |
| 垃圾文件 | 临时缓存、空目录、重复副本、失败生成物 | 是否有恢复价值？ | 快照覆盖后精确删除。 | 不合并。 |

判定顺序：

1. 先查是否包含密钥、token、主机名、个人路径或本地端口。
2. 再查是否可由命令重新生成。
3. 再查是否已被正式文档、spec、PR 或 release note 替代。
4. 最后决定 commit、fix then commit、keep local、preserve then delete、defer。

分类输出应写成表格，而不是停留在口头判断。推荐字段：

```text
Path | Class | Evidence | Risk | Recommended disposition | Approval needed
```

### 1.2 Multi-Branch Dirty Lines

如果多个本地分支或 worktree 都有脏状态，不要把它们合并成一个 cleanup 任务。先建立分支级清单：

```text
Branch/worktree | HEAD | Dirty count | Stash count | Owner | Disposition
```

处理规则：

1. 每个分支单独做 inventory 和 recovery snapshot。
2. 只选择一个根工作树作为当前执行对象。
3. 其他脏分支标记为 `defer`、`separate cleanup` 或 `owner review`。
4. 不用全局 `git stash --include-untracked` 试图一次性收纳所有分支状态。
5. 合并或删除 worktree 前，必须确认对应分支已经 merged、discarded 或 archived。

## 2. Recovery Snapshot

在任何破坏性动作前，必须能回答一个问题：如果清理错了，如何恢复？

建议创建一个恢复目录：

```bash
mkdir -p var/recovery/dirty-worktree-YYYY-MM-DD
```

最少保存：

```text
status-porcelain.txt
tracked.diff
untracked-files.txt
untracked-sha256.txt
untracked-sizes.txt
untracked-files.tar
branch-list.txt
stash-list.txt
worktree-list.txt
local-head-show.txt
restore-instructions.md
phase0-manifest.json
```

同时创建救援分支：

```bash
git branch rescue/dirty-worktree-YYYY-MM-DD
```

恢复包应记录：

- 创建时间。
- 仓库路径。
- 原始 HEAD。
- tracked diff 字节数和 SHA-256。
- untracked archive 字节数和 SHA-256。
- 清点错误数量。
- 缺失的必要文件。
- 恢复命令示例。

`phase0-manifest.json` 的最小 schema：

```json
{
  "created_at": "YYYY-MM-DDTHH:MM:SSZ",
  "repo_root": "/path/to/repo",
  "head": "<git-sha>",
  "branch": "master",
  "remote_base": "origin/master",
  "tracked_diff": {
    "path": "tracked.diff",
    "bytes": 0,
    "sha256": "<sha256>"
  },
  "untracked_archive": {
    "path": "untracked-files.tar",
    "bytes": 0,
    "sha256": "<sha256>",
    "file_count": 0
  },
  "inventory": {
    "status_entries": 0,
    "inventory_errors": 0,
    "required_missing": []
  },
  "rescue_branch": "rescue/dirty-worktree-YYYY-MM-DD"
}
```

`restore-instructions.md` 的最小模板：

```text
# Restore Instructions

## Source

- Repository: `/path/to/repo`
- Original HEAD: `<git-sha>`
- Rescue branch: `rescue/dirty-worktree-YYYY-MM-DD`

## Restore tracked changes

git switch rescue/dirty-worktree-YYYY-MM-DD
git apply var/recovery/dirty-worktree-YYYY-MM-DD/tracked.diff

## Restore untracked files

tar -xf var/recovery/dirty-worktree-YYYY-MM-DD/untracked-files.tar -C .

## Verify

git status --porcelain=v1
```

快照不是心理安慰，必须验证：

```bash
tar -tf var/recovery/dirty-worktree-YYYY-MM-DD/untracked-files.tar >/dev/null
git apply --check var/recovery/dirty-worktree-YYYY-MM-DD/tracked.diff
```

`git apply --check` 的限制：

- 在原始脏树中验证可能失败，因为工作树已有同名 untracked 文件、deleted file 状态或 new file mode 冲突。
- 失败不一定表示快照不可恢复；可能只是验证环境不是干净基线。
- 最可靠做法是在临时 clone 或一次性 worktree 中验证。

替代验证模板：

```bash
git clone <repo-url> /tmp/dirty-restore-check
cd /tmp/dirty-restore-check
git switch rescue/dirty-worktree-YYYY-MM-DD
git apply --check /path/to/repo/var/recovery/dirty-worktree-YYYY-MM-DD/tracked.diff
tar -tf /path/to/repo/var/recovery/dirty-worktree-YYYY-MM-DD/untracked-files.tar >/dev/null
```

### Optional Remote Snapshot Branch

团队协作或高价值清理建议额外建立远端快照分支。远端快照不是替代本地恢复包，而是防止本机损坏、误删或代理会话丢失。

先做敏感信息检查。不要把 `.env`、token、私钥、本地数据库连接串或个人凭据推到远端。

推荐安全形态：

```text
本地完整恢复包 + 远端 rescue 分支指针 + 可选的加密/脱敏 archive
```

基础远端救援分支：

```bash
git branch rescue/dirty-worktree-YYYY-MM-DD
git push origin rescue/dirty-worktree-YYYY-MM-DD
```

如团队要求“远端也保存恢复包”，使用单独 snapshot 分支，并且只在私有可信远端、敏感信息检查通过后执行：

```bash
git switch --orphan snapshot/dirty-worktree-YYYY-MM-DD
git rm -rf .
mkdir -p recovery
cp -a var/recovery/dirty-worktree-YYYY-MM-DD recovery/
git add recovery
git commit -m "snapshot: preserve dirty worktree recovery package"
git push origin snapshot/dirty-worktree-YYYY-MM-DD
git switch -
```

如果恢复包可能包含敏感信息，改用加密归档：

```bash
tar -cf dirty-worktree-YYYY-MM-DD.tar var/recovery/dirty-worktree-YYYY-MM-DD
gpg -c dirty-worktree-YYYY-MM-DD.tar
```

然后只推送 `.gpg` 文件或把它存到团队认可的安全存储中。

不要直接执行：

```bash
git add -A
git commit -m "backup dirty worktree"
git push
```

这种做法会把本地配置、密钥、运行产物和草稿全部混成一个无法审计的远端提交。

## 3. Clean Review Worktree

不要直接在脏根工作树里整理提交。用干净工作树承接要进入主干的切片。

模板：

```bash
git fetch origin master
git worktree add .worktrees/dirty-cleanup-review-base origin/master -b cleanup/dirty-worktree-review-base-YYYY-MM-DD
```

如果主干是 `main`：

```bash
git fetch origin main
git worktree add .worktrees/dirty-cleanup-review-base origin/main -b cleanup/dirty-worktree-review-base-YYYY-MM-DD
```

干净工作树的用途：

- 接收从脏根工作树复制出来的候选切片。
- 保持 path-scoped diff 清晰。
- 运行测试和验证。
- 生成 PR。

根工作树继续作为 salvage source，直到最终 root realignment。

## 4. Slice Extraction

切片要按“可独立审核”划分，而不是按“文件刚好在一起”划分。

推荐顺序：

1. 文档和架构报告。
2. 治理配置和流程文档。
3. 测试-only 或低风险测试补充。
4. 低风险代码修复。
5. 高风险代码域。
6. 运行产物和证据 retention。

每个切片都要有：

- 包含路径列表。
- 排除路径列表。
- 来源哈希或复制证明。
- 预期提交信息。
- 验证命令。
- 回滚方式。

切片示例：

```text
Slice: docs architecture audit package
Include:
- docs/reports/ARCHITECTURE_AUDIT_YYYY-MM-DD.md
- docs/reports/ARCHITECTURE_AUDIT_REVIEW_YYYY-MM-DD.md
- docs/superpowers/specs/YYYY-MM-DD-architecture-audit-design.md

Exclude:
- logs/
- var/
- raw evidence JSON
- local MCP config

Validation:
- markdown reference scan
- git diff --check
- optional docs link check
```

## 5. Slice Validation

切片提取后必须验证。验证强度按风险分层：文档切片做结构和引用检查；治理切片做配置/schema 检查；代码切片做影响分析、格式化、lint 和测试；运行产物切片做归档和 disposition 检查。

### 5.1 Product Code Rules

代码切片比文档切片风险更高。

执行前：

- 找到受影响符号、调用方和执行流。
- 做 blast radius 分析。
- 如果风险高，先告知并缩小切片。
- 不要把无关格式化和行为修改混在一起。

提交前：

- 运行相关测试。
- 运行格式化检查。
- 运行 lint 或静态检查。
- 运行变更检测，确认影响范围符合预期。

代码切片提交应回答：

```text
改了什么行为？
为什么属于本切片？
直接调用方是否更新？
哪些测试覆盖了它？
哪些风险被延后？
```

### 5.2 Generated And Runtime Artifact Validation

运行产物是最容易误删、也最容易误提交的部分。

常见路径：

```text
logs/
var/
tmp/
target/
coverage/
test_timing.csv
docs/reports/evidence/
.governance/backups/
```

处理策略：

1. 先归档到 recovery snapshot。
2. 分类为 local-only、generated evidence、policy artifact、unknown。
3. 只对明确路径组申请删除批准。
4. 删除时用精确路径，不用 `git clean -fd`。
5. 如果产物代表正式证据，转成可审查报告或 retention policy 后再提交。

精确删除示例：

```bash
rm -rf logs test_timing.csv var/reports docs/reports/evidence
```

执行这类命令前必须明确：

- 哪些路径会被删除。
- 是否已在恢复包里。
- 是否保留 recovery 目录。
- 是否会影响当前工具运行。

## 6. PR And Commit Strategy

优先用多个窄 PR，而不是一个巨大 PR。

推荐提交形态：

```text
docs: add architecture audit reports
chore: add governance cleanup policy
test: preserve validated cleanup slices
docs: add issue tracker cleanup disposition
```

每个 PR 应说明：

- 包含哪些路径。
- 明确排除哪些路径。
- 验证命令。
- 是否还有未处理的 residual untracked items。

不要在 PR 中混入：

- 本地 MCP 配置。
- 日志。
- 缓存。
- 原始运行证据。
- 未审核历史草稿。
- 与当前切片无关的格式化 churn。

## 7. Root Tracked Realignment

只有在选定切片已经合并到远端主干后，才考虑对齐根工作树。

推荐顺序：

```bash
git fetch origin master
git status --porcelain=v1
git rev-list --left-right --count HEAD...origin/master
git reset --hard origin/master
```

注意：

- `git reset --hard origin/master` 只处理 tracked files。
- 它不会删除 untracked files。
- 不要同时运行 `git clean -fd`。
- root realignment 需要单独明确批准。

对齐后验证：

```bash
git rev-parse --short HEAD
git rev-parse --short origin/master
git rev-list --left-right --count HEAD...origin/master
git status --porcelain=v1
git diff --stat origin/master
```

期望结果：

```text
HEAD == origin/master
ahead/behind == 0 0
tracked drift == none
remaining status == only untracked residuals
```

## 8. Residual Untracked Disposition

root realignment 后通常还会剩下一批未跟踪项。不要急着删。

为剩余项生成 disposition 表：

| Path | Evidence | Recommendation |
| --- | --- | --- |
| `.mcp.json` | local MCP host config | keep local or delete after confirming obsolete |
| `docs/...draft.md` | superseded by tracked spec/archive | preserve via recovery, delete after approval |
| `var/recovery/...` | recovery snapshot | keep until all disposition decisions are complete |
| `docs/...historical.md` | may have documentation value | defer to separate docs PR |

每个 residual item 的结论只能是以下之一：

| Disposition | Meaning |
| --- | --- |
| Commit | 有明确仓库价值，引用已验证，可作为窄 PR。 |
| Fix then commit | 有价值但当前内容有错误、过期引用或目标不明。 |
| Keep local | 本地配置或工作资料，不进入仓库。 |
| Preserve then delete | 已被恢复包覆盖，可按路径级批准删除。 |
| Defer | 价值不明，需要 owner 决策或单独任务。 |

## 9. Final Cleanup

最终清理只在这些条件满足后进行：

- 主干已经包含所有选定切片。
- recovery snapshot 已验证。
- residual disposition 已审阅。
- 每个删除路径都有明确批准。
- 没有正在运行的工具依赖待删除路径。

最终可清理对象：

- 已合并的本地临时分支。
- 已合并的远端 cleanup 分支。
- 已合并且不再需要的 clean review worktree。
- 已明确无用的 untracked local config。
- 已被恢复包覆盖并批准删除的历史草稿。
- 已迁移到外部存储或不再需要的 recovery 包。

clean review worktree 的清理模板：

```bash
git worktree list --porcelain
git worktree remove .worktrees/dirty-cleanup-review-base
git branch -d cleanup/dirty-worktree-review-base-YYYY-MM-DD
```

远端 cleanup 分支只有在 PR 已合并、没有后续审计需要时再删除。

谨慎处理：

- `var/recovery/`
- `.worktrees/`
- `.mcp.json`
- `.env`
- 手写历史文档
- 原始证据文件

## High-Risk Operation Blacklist

这些操作不是永远禁止，但不能在没有快照、分类、审批和回滚方案时执行。

| Risky operation | Why risky | Safe alternative |
| --- | --- | --- |
| `git reset --hard` at start | 丢失 tracked diff 的现场上下文。 | 先保存 `tracked.diff`，在 clean worktree 提取切片，最后再 root realignment。 |
| `git clean -fd` | 删除所有 untracked，包括文档草稿、恢复包、本地配置和证据。 | 生成 untracked disposition，按路径级 `rm -rf <approved-path>`。 |
| `git pull` before inventory | 合并远端变化后会污染原始脏线证据。 | 先 `git fetch`，记录 ahead/behind，再决定 merge/rebase。 |
| `git add -A` | 把运行产物、本地配置、草稿和正式变更混到一起。 | 按 slice 使用显式路径 `git add path1 path2`。 |
| `git stash --include-untracked` | stash 成为黑箱，后续难审计，且容易丢失恢复路径。 | 建 recovery snapshot 和 rescue branch。 |
| `rm -rf logs var tmp` | 可能删除审计证据和恢复包。 | 先归档，再路径级审批，再精确删除非 recovery 子目录。 |
| `git checkout -- .` | 无差别丢弃 tracked 修改。 | 针对已确认无价值路径使用 `git restore -- <path>`。 |
| force push cleanup branch | 可能覆盖他人的审查基线。 | 新建分支或追加提交，必要时在 PR 中解释。 |

安全执行高危动作前必须能回答：

```text
我会影响哪些路径？
这些路径是否已被快照覆盖？
是否有敏感信息风险？
是否有 owner 的路径级批准？
失败后如何恢复？
执行后用什么命令验收？
```

## Verification Checklist

清理结束前逐项确认：

```text
[ ] 已创建 recovery snapshot。
[ ] 已验证 tracked.diff 可应用或可解释。
[ ] 已验证 untracked archive 可读取。
[ ] 已创建 clean review worktree。
[ ] 每个提交都是 path-scoped。
[ ] 每个代码切片都有影响分析。
[ ] 每个代码切片都有测试或验证记录。
[ ] 没有把 local config 提交进仓库。
[ ] 没有把 generated/runtime artifacts 误提交。
[ ] root realignment 已单独批准。
[ ] root HEAD 与 origin/main 或 origin/master 对齐。
[ ] 剩余 untracked items 已生成 disposition。
[ ] 删除动作都是路径级批准。
[ ] 最终状态和剩余风险已写入 handoff 或 cleanup report。
```

## Acceptance Baselines

一次脏线清理只有满足以下基线，才算完成。

| Baseline | Acceptance evidence |
| --- | --- |
| 工作树实质干净 | tracked drift 为零；剩余 untracked 都有 disposition。 |
| 成果不丢失 | 有价值代码、测试、文档已通过 PR/commit 进入主干，或明确 defer。 |
| 变更可追溯 | 每个提交 path-scoped，PR 描述包含 include/exclude 和验证命令。 |
| 功能可闭环 | 代码切片通过相关测试、格式化、lint；影响分析报告已生成，且无 HIGH/CRITICAL 未解决项。 |
| 分支基线正常 | root `HEAD` 与 `origin/main` 或 `origin/master` 对齐，ahead/behind 为 `0 0`。 |
| 备份完备 | recovery snapshot 可读取，tracked diff 可验证，restore instructions 存在。 |
| 本地配置未泄漏 | `.env`、token、本地 MCP/IDE 配置未进入正式提交。 |
| 删除可审计 | 每个删除路径都有审批记录和恢复来源。 |

## Common Failure Modes

### 1. 一开始就 `git reset --hard`

这会丢掉 tracked changes 的现场上下文。如果没有 recovery snapshot，很难判断哪些改动有价值。

替代：

```text
先保存 tracked.diff 和 untracked archive，再在 clean worktree 提取切片。
```

### 2. 用 `git clean -fd` 快速清空 untracked

这会删除本地配置、证据、恢复包、未提交文档和工具状态。

替代：

```text
生成 untracked disposition，按路径级批准精确删除。
```

### 3. 把所有东西塞进一个 cleanup PR

这会让审查者无法判断哪些是正式改动、哪些只是清理副产物。

替代：

```text
docs/governance/code/tests/generated artifacts 分开 PR 或分开提交。
```

### 4. 把根工作树当成施工区

在脏根工作树上继续编辑会让来源更混乱。

替代：

```text
根工作树作为 salvage source，干净 worktree 作为 review base。
```

### 5. 清理报告和正式记录并存但没有权威规则

如果草稿、review、OpenSpec archive、PR 描述都存在，后续代理不知道该信谁。

替代：

```text
明确 authoritative source。例如：OpenSpec archive/spec 是正式记录，草稿只保留在 recovery snapshot。
```

### 6. 先 pull 再整理

这会让远端变化、自动合并结果和本地脏线混在一起，后续无法判断问题来源。

替代：

```text
先 fetch 和 inventory，记录分叉关系，再决定是否在 clean worktree 上重放切片。
```

### 7. 把本地配置当作项目配置提交

`.mcp.json`、`.env`、IDE workspace、个人路径和本机端口通常只对当前机器有效。

替代：

```text
提交 example/template 或 docs，真实本地配置留在本机或安全存储。
```

## Long-Term Prevention

脏线清理不能只解决当下，还要降低复发概率。

### Gitignore Governance

维护项目级 `.gitignore`，覆盖稳定且可重建的产物：

```text
target/
coverage/
tmp/
logs/
.env
*.local.*
```

不要把“可能是正式证据”的路径盲目 ignore。比如 `docs/reports/evidence/` 可能是运行产物，也可能是审计证据，应先建立 retention policy。

### Local Config Templates

本地配置应使用模板：

```text
.env.example
.mcp.example.json
config/local.example.toml
```

真实配置保持 untracked，或者进入安全密钥管理系统。

### Commit Hygiene

团队约定：

- 每天结束前清理或提交本日有效切片。
- PR 不混入 generated artifacts。
- 提交信息表达意图，而不是表达“cleanup everything”。
- 大型清理必须先写 plan，再执行。

### Branch Sync Hygiene

长期分支应定期同步主干：

```bash
git fetch origin
git rev-list --left-right --count HEAD...origin/master
```

发现长期漂移时，不要直接 pull 到脏树。先保存快照，再在 clean worktree 里重放有效切片。

### Tool Output Boundaries

工具、测试、代理和生成器应明确输出目录：

```text
var/reports/
logs/
tmp/
target/
docs/reports/evidence/
```

输出目录必须有 retention policy：哪些可以删，哪些要转成正式报告，哪些必须归档。

## Recommended Document Set

复杂脏线清理建议产出这些文档：

```text
docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_YYYY-MM-DD.md
docs/reports/DIRTY_WORKTREE_CLEANUP_PLAN_YYYY-MM-DD-review.md
openspec/changes/<cleanup-change>/
openspec/specs/worktree-cleanup/spec.md
docs/reports/UNTRACKED_REMAINDER_DISPOSITION_REVIEW_YYYY-MM-DD.md
var/recovery/dirty-worktree-YYYY-MM-DD/restore-instructions.md
```

如果项目不用 OpenSpec，也应保留等价结构：

```text
docs/reports/dirty-worktree-cleanup-plan.md
docs/reports/dirty-worktree-cleanup-tasks.md
docs/reports/dirty-worktree-cleanup-policy.md
docs/reports/dirty-worktree-cleanup-closure-summary.md
var/recovery/dirty-worktree-YYYY-MM-DD/restore-instructions.md
```

## Minimal Command Appendix

以下命令是模板，执行前应替换主干名、路径和日期。`--porcelain=v1` 用于最大兼容性：v1 输出稳定，跨 Git 版本一致；v2 更适合增量解析，但兼容性窗口较窄。

```bash
# Inventory
git status --porcelain=v1
git diff --binary
git branch --list
git worktree list --porcelain

# Recovery
mkdir -p var/recovery/dirty-worktree-YYYY-MM-DD
git diff --binary > var/recovery/dirty-worktree-YYYY-MM-DD/tracked.diff
git status --porcelain=v1 > var/recovery/dirty-worktree-YYYY-MM-DD/status-porcelain.txt
git branch rescue/dirty-worktree-YYYY-MM-DD

# Clean review worktree
git fetch origin master
git worktree add .worktrees/dirty-cleanup-review-base origin/master -b cleanup/dirty-worktree-review-base-YYYY-MM-DD

# Validate slice
git diff --check
# Run project formatting/lint checks, for example:
# cargo fmt --check
# npm run lint
# ruff check .
# Run project tests, for example:
# cargo test
# pytest
# npm test
# go test ./...

# Root tracked realignment after merge and approval
git fetch origin master
git reset --hard origin/master

# Residual check
git diff --stat origin/master
git status --porcelain=v1
```

## Operating Rule

脏线清理的成功标准不是“没有文件了”，而是：

```text
有价值的变更进入主干；
无价值的产物被可恢复地清除；
本地配置没有泄漏；
主干重新可验证；
剩余风险被明确记录。
```
