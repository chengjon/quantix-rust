# FUNCTION_TREE Governance Skill 规划

> Status: skill-ready
> Prepared: 2026-05-24
> Scope: 将 FUNCTION_TREE 从静态功能注册表扩展为包含治理能力的 Skill 体系
> Target plugin repo: `https://github.com/chengjon/myskills.git`
> Skill directory: `skills/function-tree/`
> Local user-level install recommendation: symlink `skills/function-tree` to `${CODEX_HOME:-$HOME/.codex}/skills/function-tree`
> Boundary: 本文档授权创建用户级 Skill 与插件清单变更；不授权 quantix-rust 应用源码变更
> Registry boundary: 本计划不是功能状态注册表；当前功能状态以 `FUNCTION_TREE.md` 的状态注册表为准

---

## 1. 背景：为什么扩展 FUNCTION_TREE

### 1.1 当前 FUNCTION_TREE 的能力边界

`FUNCTION_TREE.md` 目前承担两个职责：

| 职责 | 实现方式 | 局限 |
|------|----------|------|
| 功能状态注册表 | 表格行 `[已实现]/[部分实现]/[已设计/待实现]/[非目标]` | 静态快照，不跟踪治理过程 |
| 证据索引 | 源码路径、CLI 入口、模块目录树 | 不区分证据类型（观察/授权/实现/关闭） |

### 1.2 缺失的治理能力

参考 Steward Tree 实践指南（mystocks_spec 项目验证），以下能力已在跨 Agent 架构治理中被证明有效，但当前 FUNCTION_TREE 不具备：

| 缺失能力 | 导致的风险 | 实际案例 |
|----------|-----------|---------|
| 授权边界控制 | Agent 把证据收集当成改代码的授权 | `handlers.rs` 拆分讨论常被跳过直接开始改文件 |
| 状态机流转 | 无法区分"已观察"和"已授权实现" | miniQMT 对账能力有大量 report 但无授权记录 |
| 范围蔓延防护 | 无 forbidden scope 机制 | 因子算子开发经常漂移到评分/导出/回测 |
| 跨会话上下文恢复 | 新 Agent 必须重读全部历史 | 每次新会话花 30 分钟理解前序状态 |
| 兼容面决策跟踪 | 改了代码但没记录兼容面保留/退役决策 | `pub use` re-export 被静默删除 |
| 陈旧证据检测 | 无 git HEAD 对比机制 | 使用旧 commit 的计数/扫描结果做决策 |
| 实现传送带 | 无串行化候选处理流程 | 多个候选同时被选中导致 PR 膨胀 |

### 1.3 核心设计原则

1. **不引入第二个 TREE**。FUNCTION_TREE 是唯一的功能树名称。扩展方式是在现有结构上增加治理层，而非创建平行的 Steward Tree。

2. **Git 是唯一硬依赖真相源**。所有状态判断优先用本地 git（`git log`、`git diff`、commit hash）。`gh` CLI / GitHub API、OPENDOG MCP 是可选增强——有则补充证据和验证，无则 Skill 仍可完整运行。GitHub Issue/PR 不驱动状态机流转，仅作为证据锚定。

3. **外部工具默认可选，但 repo profile 可以提升为强制规则**。通用 Skill 中 `gh`（PR 状态验证）、OPENDOG（运行时文件活跃度）、GitNexus（代码结构智能）均为可选增强；在 `quantix-rust` profile 中，GitNexus impact / detect_changes 必须遵守 AGENTS.md，涉及 symbol edit 或 commit gate 时不可降级。

4. **确定性状态机操作必须脚本化**。`nodes.json`、`active-gates.json`、`active-gates.md`、task-card schema、scope guard、状态迁移校验由 `scripts/ft-governance.cjs` 执行；`SKILL.md` 与 `references/` 只负责触发、解释和流程导航。

---

## 2. 目标架构：三层 Skill 体系

### 2.1 整体结构

```
FUNCTION_TREE (统一概念)
├── Layer 1: 功能注册表 (现有 FUNCTION_TREE.md，保持不变)
│   ├── 状态注册表
│   ├── 证据展开
│   └── CLI 命令树
│
├── Layer 2: 治理状态机 (新增)
│   ├── .governance/programs/<program-name>/tree.md    # 人类可读治理树
│   ├── .governance/programs/<program-name>/nodes.json  # 机器索引
│   └── .governance/programs/<program-name>/cards/      # PR 任务卡
│
└── Layer 3: Skill 命令集 (新增)
    └── /ft:<command>                                    # FUNCTION_TREE Skill 入口
```

### 2.2 Layer 1 — 功能注册表（现有，不改动）

`FUNCTION_TREE.md` 继续作为：
- 功能状态单一真相源
- 证据索引
- CLI 命令树

不修改其结构。治理层通过引用 FUNCTION_TREE 节点（如 `handlers-split`、`factor-pipeline`）来关联。

### 2.3 Layer 2 — 治理状态机（新增）

目录结构：

```
.governance/
├── active-gates.json                         # 机器索引（Skill 读写此文件）
├── active-gates.md                           # 人类可读（由 .json 生成，单向同步）
└── programs/
    ├── handlers-split/
    │   ├── tree.md                          # 人类可读治理树
    │   ├── nodes.json                       # 机器可读节点索引
    │   └── cards/
    │       ├── H3.1-trade.yaml              # PR 任务卡
    │       └── H3.2-account.yaml
    ├── factor-pipeline/
    │   ├── tree.md
    │   ├── nodes.json
    │   └── cards/
    └── ...future programs...
```

#### 治理树最小字段（tree.md）

| 字段 | 用途 |
|------|------|
| Node ID | 稳定引用，如 `H1.1`、`H3.2` |
| Parent program | 所属治理程序 |
| State | 当前状态（见下方状态机） |
| Source evidence | 报告路径、JSON 路径、PR、Issue |
| Current facts | 短事实摘要（含计数和 commit hash） |
| Next gate | 唯一允许的下一步动作 |
| Forbidden scope | 当前禁止的操作列表 |
| FUNCTION_TREE ref | 关联的 FUNCTION_TREE.md 节点（可选） |

#### 机器节点最小字段（nodes.json）

字段分为**核心字段**（零外部工具即可运行）和**扩展字段**（仅当对应工具可用时才出现，不可用时整个字段省略，不留 null）。

```json
{
  "node_id": "H3.1",
  "parent_program": "handlers-split",
  "state": "authorization-prepared",
  "source_evidence": [
    "docs/reports/handlers-split-trade-analysis.md"
  ],
  "generated_artifacts": [
    ".governance/programs/handlers-split/nodes.json"
  ],
  "current_head": "a1b2c3d",
  "current_head_checked_at": "2026-05-24T10:00:00+08:00",
  "stale_if_head_mismatch": true,
  "authorized_paths": [
    "src/cli/handlers/trade_handler.rs",
    "src/cli/handlers/mod.rs"
  ],
  "forbidden_paths": [
    "src/cli/handlers/account.rs",
    "src/cli/handlers/data_handler.rs"
  ],
  "next_gate": "implement: create trade domain handler files",
  "source_edits_authorized": false,
  "implementation_target_selected": false,
  "function_tree_refs": ["cli/"]
}
```

**扩展字段**（仅当对应工具可用时由 Skill 自动追加，不可用时完全不出现）：

| 字段 | 来源工具 | 用途 | 不可用时 |
|------|---------|------|---------|
| `blocker_reason` + `unblock_target_state` | 状态机自身 | 仅 `blocked` 状态时出现 | 非此状态时省略 |
| `current_head_update_timing` | 状态机自身 | 描述性元信息，可省略 | 不影响逻辑 |
| `gitnexus_impact` | GitNexus | d=1 依赖列表 | 用 `grep`/文件扫描降级 |
| `opendog_snapshot_run_id` + `stale_if_snapshot_drift` | OPENDOG | 快照漂移检测 | 完全省略，仅靠 `current_head` 做陈旧检测 |

**降级策略**：每个扩展字段都有对应的零工具降级路径。Skill 检测到工具不可用时，直接跳过对应字段和检查步骤，不输出警告、不要求用户安装任何东西。

#### PR 任务卡最小字段（cards/<node-id>.yaml）

```yaml
task:
  id: "H3.1"
  title: "Split trade domain handlers from handlers.rs"

scope:
  allowed_paths:
    - "src/cli/handlers/trade_handler.rs"
    - "src/cli/handlers/mod.rs"
  forbidden_paths:
    - "src/cli/handlers/account.rs"
    - "src/cli/handlers/data_handler.rs"
    - "tests/**"

non_goals:
  - "Do not modify account handlers"
  - "Do not change test structure"
  - "Do not update docs/API"

acceptance:
  commit_gate:
    - "cargo check passes"
    - "cargo clippy -- -D warnings passes"
  closeout_gate:
    - "cargo build --release passes"
    - "cargo test passes"
    - "All pub use re-exports preserved in mod.rs"

governance:
  program: "handlers-split"
  node_id: "H3.1"
  approval:
    required: true
    approved_by: ""
    approved_at: ""
```

#### 活跃门控汇总（active-gates.md）

```markdown
# Active Gates

> 仅列出未关闭的治理节点。关闭后从此表移除。

| Gate | Program | Current blocker | Next allowed | Forbidden | FT ref |
|------|---------|----------------|--------------|-----------|--------|
| H1.1 | handlers-split | Evidence collection | Run GitNexus impact on `handlers` | Source edits | `cli/` |
| F2.3 | factor-pipeline | Awaiting review | Score CSV export | New operators | `factor/` |
```

#### 活跃门控机器索引（active-gates.json）

所有 Skill 命令读写 `active-gates.json`，`active-gates.md` 仅作人类查看用途。

```json
{
  "updated_at": "2026-05-24T10:00:00+08:00",
  "active_nodes": [
    {
      "gate": "H1.1",
      "program": "handlers-split",
      "state": "evidence-prepared",
      "current_blocker": "Evidence collection",
      "next_allowed": "Run GitNexus impact on handlers",
      "forbidden": ["source edits"],
      "ft_ref": "cli/",
      "current_facts": "handlers.rs 11K+ lines, 47 symbols, CRITICAL tech debt"
    }
  ]
}
```

### 2.4 Layer 3 — Skill 命令集

```
skills/function-tree/
├── SKILL.md                    # 主入口：触发规则 + 命令总览
├── references/
│   ├── STATE_MACHINE.md        # 状态、证据、授权、关闭规则
│   └── QUANTIX_PROFILE.md      # quantix-rust 强制 GitNexus / Graphiti 规则
├── scripts/
│   └── ft-governance.cjs       # 确定性状态机 / schema / guard CLI
├── commands/
│   ├── ft-init.md              # /ft:init — 初始化治理程序
│   ├── ft-observe.md           # /ft:observe — 记录观察/证据
│   ├── ft-authorize.md         # /ft:authorize — 创建授权包
│   ├── ft-implement.md         # /ft:implement — 绑定 PR 实现
│   ├── ft-closeout.md          # /ft:closeout — 关闭节点
│   ├── ft-gate.md              # /ft:gate — 查看活跃门控
│   └── ft-status.md            # /ft:status — 治理程序状态总览
├── templates/
│   ├── program-tree.md         # 治理树模板
│   ├── node.json               # 节点 JSON 模板
│   └── task-card.yaml          # PR 任务卡模板
└── guards/
    └── ft-scope-check.sh       # thin wrapper: 调用 scripts/ft-governance.cjs scope-check
```

#### 命令详细设计

##### `/ft:init <program-name> --ref <function-tree-node>`

初始化一个新的治理程序。

**动作**：
1. 创建 `.governance/programs/<program-name>/` 目录
2. 生成 `tree.md`（状态图例 + 空树 + 空证据账本）
3. 生成 `nodes.json`（空数组）
4. 更新 `.governance/active-gates.md`（如果首次创建则初始化）
5. 如果指定 `--ref`，在 tree.md 中关联 FUNCTION_TREE.md 节点

**输入**：
- `--ref`：关联 FUNCTION_TREE.md 中的功能节点（如 `cli/`、`factor/`）
- `--description`：程序目标简述

**输出**：
- 治理程序目录和初始文件
- 提示下一步：`/ft:observe` 收集基线证据

##### `/ft:observe <node-id> --evidence <path-or-description>`

记录观察和证据。状态 → `evidence-prepared`。

**动作**：
1. **基线证据**（零外部工具即可完成，始终执行）：
   - 文件级扫描：行数、模块依赖、目录结构（`find`/`wc`/`grep`）
   - git 证据：最近改动 commit、blame 热点文件、`git diff --stat`
   - 用户提供的 `--evidence` 路径内容
2. **GitNexus 增强**（可用时自动追加，不可用时跳过）：
   - `gitnexus_impact` — 影响范围
   - `gitnexus_query` — 执行流程
   - `gitnexus_context` — 符号全貌
3. **OPENDOG 增强**（可用时自动追加，不可用时跳过）：
   - `get_stats` + `get_unused_files` — 文件活跃度
   - `get_time_window_report` — 近期活动形状
4. 合并所有可用证据写入 `tree.md` 证据账本
5. 更新 `nodes.json` 对应节点（写入 `current_head`，扩展字段仅在有对应工具时追加）
6. 更新 `active-gates.json` + `active-gates.md`

**强制约束**：
- 状态只能是 `evidence-prepared`
- **禁止**：不授权任何代码改动
- **禁止**：不选择实现目标

**输出**：
- 证据包写入 tree.md 证据账本
- 节点状态更新为 `evidence-prepared`
- next gate 提示：`/ft:authorize` 或 `/ft:observe`（补充证据）

##### `/ft:authorize <node-id> --allow <paths> --forbid <paths>`

创建授权包。状态 → `authorization-prepared` → `approved-for-implementation`。

**动作**：
1. 验证当前节点已有 `evidence-prepared` 状态
2. 验证 `allowed_paths` 内的文件存在
3. 调用 GitNexus 验证 allowed_paths 的上游影响
4. 列出所有 d=1（WILL BREAK）依赖为必须更新项
5. 生成 `cards/<node-id>.yaml` 任务卡
6. 更新 tree.md、nodes.json、active-gates.md

**强制约束**：
- 必须从 `evidence-prepared` 状态转移
- 必须显式列出 `allowed_paths`
- `forbidden_paths` 推荐但不强制；`non_goals` 必须至少一项
- OPENDOG 可用时，`get_usage_trends` 补充授权目标的长期冷热趋势
- 陈旧检测：`current_head` 不匹配时阻止；OPENDOG 可用时，`compare_snapshots` 检测未 commit 的漂移

**输出**：
- 任务卡 YAML 文件
- 节点状态更新为 `authorization-prepared`
- next gate 提示：人工审核后 `/ft:implement`

##### `/ft:implement <node-id> --pr <number>`

绑定 PR 实现。状态 → `implementation-merged`。

**动作**：
1. 验证节点处于 `approved-for-implementation` 状态
2. 用 `git diff` 获取变更范围（硬依赖，本地可靠）
3. 如果指定了 `--pr` 且 `gh` 可用，补充拉取 PR diff 做交叉验证（可选增强）
4. 检查 diff 中修改的文件是否全部在 `allowed_paths` 内
5. 如果超出范围，**警告并阻止**（除非用户显式确认扩展授权）
6. 更新 tree.md、nodes.json
7. 从 `active-gates.md` 临时移除（等 closeout）

**强制约束**：
- 必须从 `approved-for-implementation` 状态转移
- diff 超出 allowed_paths 时必须人工确认
- 无 `gh` 认证时仍可完整运行，回退到 `git diff`

**输出**：
- 节点状态更新为 `implementation-merged`
- next gate 提示：`/ft:closeout`

##### `/ft:closeout <node-id>`

关闭节点。状态 → `closeout-prepared` → `archived`。

**动作**：
1. 验证：
   - 合并确认：优先用 `git log --merges` 检查本地 git（硬依赖）；`gh` 可用时补充 PR 状态验证（可选增强）
   - 测试通过？ (`cargo test`, `cargo clippy`)
   - 兼容面保留？ (re-export 完整性)
2. 生成关闭报告回答以下问题：
   - 哪个 PR 合并了？哪个 commit？
   - 哪些行为变了？
   - 哪些兼容面保留了？
   - 哪些测试/smoke 通过了？
   - 下一个 gate 是什么？
   - 什么仍然未授权？
3. 如果有 FUNCTION_TREE ref，检查是否需要更新 FUNCTION_TREE.md
4. 从 `active-gates.md` 移除
5. 状态 → `closeout-prepared` → `archived`

**输出**：
- 关闭报告写入 tree.md
- FUNCTION_TREE.md 状态更新（如果需要）
- active-gates.md 更新

##### `/ft:gate [--verbose]`

查看活跃门控汇总。新 Agent 上下文恢复入口。

**动作**：
1. 读取 `.governance/active-gates.json`
2. 快速模式（默认）：门控表 + 每节点一句话 facts summary
3. 详细模式（`--verbose`）：完整 source_evidence 路径、forbidden_paths、上次状态变更时间

**输出**：
- 快速模式：30 秒内可理解全局治理状态
- 详细模式：完整上下文恢复（约 2 分钟阅读）

##### `/ft:status [program-name]`

治理程序状态总览。

**动作**：
1. 如果指定程序名，显示该程序的完整 tree.md
2. 如果不指定，显示所有程序的概要
3. 显示状态机位置、已完成/进行中/待处理的节点数

---

## 3. 状态机设计

### 3.1 节点状态流转

```
observed                    # 初次观察/发现问题
  │
  ▼
evidence-prepared           # 证据已收集，事实已记录
  │                         # ⛔ 禁止：不授权代码改动
  │ ◄──── blocked           # 发现阻塞项（blocker_reason 必填，source_edits_authorized=false）
  ▼
decision-prepared           # 方向已选择（可选，简单程序可跳过）
  │                         # ⛔ 禁止：不实现
  │ ◄──── blocked           # 决策被驳回
  ▼
authorization-prepared      # 授权包已准备
  │                         # 包含：allowed_paths, forbidden_paths, task-card
  │ ◄──── blocked           # 授权被驳回（unblock_target_state 指明回退目标）
  ▼
approved-for-implementation # 授权已批准
  │                         # ✅ 可以：按授权范围改代码
  │ ◄──── blocked           # 实现中发现超出预期
  ▼
implementation-ready        # 实现已准备好，证据绑定 commit/branch；PR URL 可选
  │
  ▼
implementation-landed       # 实现已落地到目标分支；merge commit / commit range 为硬证据
  │
  ▼
closeout-prepared           # 关闭报告已生成
  │                         # 回答：合并了什么？兼容面？测试？下一个 gate？
  ▼
archived                    # 已归档
```

`blocked` 状态约束：`blocker_reason` 必填，`source_edits_authorized` 必须为 `false`，`unblock_target_state` 指明阻塞解除后回退的目标状态。

### 3.2 强制规则

| 规则 | 说明 |
|------|------|
| 不可跳过 evidence 直接到 implementation | 必须先收集事实 |
| 不可在 closeout 中选择下一个实现目标 | 除非 closeout 显式包含候选选择包 |
| 不可删除兼容面而无独立决策 | re-export 保留需要单独的授权 |
| 不可从陈旧证据解锁实现 | evidence 的 current_head 必须与 HEAD 匹配 |
| GitHub 不可用不得阻塞状态机 | PR/Issue 只能作为可选证据锚；Git commit / branch / diff 是硬证据 |
| quantix-rust symbol edit 不可绕过 GitNexus | impact / detect_changes 在 quantix-rust profile 中是强制门禁 |
| 证据包不等于授权 | 记录事实 ≠ 允许改代码 |

### 3.3 证据类型分类

| 类型 | 能做什么 | 不能做什么 |
|------|---------|-----------|
| 证据包 | 记录当前事实、计数、消费者、阻塞项 | 授权代码改动 |
| 决策包 | 选择方向或分类归属 | 实现该方向 |
| 授权包 | 定义未来的写入范围和必须通过的 gate | 在同一包中修改源码 |
| 实现 PR | 改动确切的授权文件 | 扩大范围或删除兼容面 |
| 关闭包 | 记录合并结果和下一个 gate | 启动下一个实现线 |

---

## 4. 以 quantix-rust 为例的落地方案

### 4.1 第一个治理程序：`handlers-split`

**目标**：将 `src/cli/handlers.rs`（11K+ 行，CRITICAL 级技术债）拆分为 `src/cli/handlers/*.rs` 模块目录。

**治理树结构**：

```text
handlers-split (治理程序)
│
├── H1. 基线证据
│   ├── H1.1 当前模块清单 + 行数 + 依赖
│   │   State: evidence-prepared
│   │   Evidence: gitnexus_impact(handlers), gitnexus_cypher(调用关系)
│   │   Next: collect handler cross-call matrix
│   │   Forbidden: source edits, file creation, test changes
│   │   FT ref: cli/
│   │
│   ├── H1.2 handler 间调用关系矩阵
│   │   State: evidence-prepared
│   │   Evidence: gitnexus_cypher query results
│   │   Next: classify handler domains
│   │   Forbidden: source edits
│   │
│   └── H1.3 外部消费者矩阵
│       State: evidence-prepared
│       Evidence: gitnexus_impact(upstream) for each handler
│       Next: decision — split strategy
│       Forbidden: source edits
│
├── H2. 拆分决策
│   ├── H2.1 拆分方案选择：按 domain 拆分
│   │   State: decision-prepared
│   │   Decision: 按 trade/account/data/... 域拆分
│   │   Forbidden: source edits, delete any file
│   │
│   └── H2.2 兼容面决策：pub use re-export 保留期
│       State: decision-prepared
│       Decision: mod.rs 必须保留所有 pub use re-export
│       Forbidden: delete re-exports, change test imports
│
├── H3. 实现（每批一个 domain）
│   ├── H3.1 trade domain 拆分
│   │   Allowed: src/cli/handlers/trade_handler.rs, src/cli/handlers/mod.rs
│   │   Forbidden: 其他 domain, tests/, docs/
│   │
│   ├── H3.2 account domain 拆分
│   │   Allowed: src/cli/handlers/account.rs, src/cli/handlers/mod.rs
│   │   Forbidden: 其他 domain
│   │
│   ├── H3.3 data domain 拆分
│   │   ...
│   │
│   └── H3.N 其余 domain（每批独立授权）
│
└── H4. 关闭
    ├── H4.1 整体验证
    │   Checks: cargo build, cargo clippy, cargo test, re-export 完整性
    │
    └── H4.2 FUNCTION_TREE.md 更新
        FT ref: cli/ → 更新 handlers 证据路径
```

### 4.2 第二个治理程序候选：`factor-pipeline`（示意）

```text
factor-pipeline (治理程序)
│
├── F1. 证据
│   ├── F1.1 当前算子清单 + 测试覆盖
│   ├── F1.2 数据加载器边界
│   └── F1.3 CLI 入口完整性
│
├── F2. 实现
│   ├── F2.1 新算子（cs_rank 等）添加
│   ├── F2.2 评分路径完善
│   └── F2.3 导出格式扩展
│
└── F3. 关闭
    └── F3.1 FUNCTION_TREE.md factor/ 节点更新
```

### 4.3 与 GitNexus 的联动

| Skill 命令 | GitNexus 工具 | 用途 |
|-----------|--------------|------|
| `/ft:observe` | `gitnexus_impact` | 收集改动影响范围作为证据 |
| `/ft:observe` | `gitnexus_cypher` | 查询调用关系矩阵 |
| `/ft:observe` | `gitnexus_context` | 获取符号全貌（调用者/被调用者） |
| `/ft:observe` | `gitnexus_query` | 按概念搜索执行流 |
| `/ft:authorize` | `gitnexus_impact` | 验证 authorized_paths 的上游影响 |
| `/ft:implement` | `gitnexus_detect_changes` | 验证 PR diff 不超出授权范围 |
| `/ft:closeout` | `gitnexus_detect_changes` | 最终确认改动范围 |

### 4.3b 与 OPENDOG 的联动（可选增强，无 OPENDOG 时完全跳过）

| Skill 命令 | OPENDOG 工具 | 补充的证据维度 | 无 OPENDOG 降级 |
|-----------|-------------|-------------|----------------|
| `/ft:init` 后 | `take_snapshot` | 建立文件清单基线 | 用 `git ls-files` 建立 |
| `/ft:observe` | `get_stats` + `get_unused_files` | 文件活跃度 | 用 `git log --since` 估算 |
| `/ft:observe` | `get_time_window_report` | 近期活动快照 | 用 `git diff --stat HEAD~N` 替代 |
| `/ft:authorize` | `get_usage_trends` | 冷热趋势 | 不补充此维度，仅靠结构证据 |
| `/ft:implement` 后 | `compare_snapshots` | 实际改动 vs 授权 | 用 `git diff --name-only` 替代 |
| `/ft:closeout` | `get_verification_status` | test/lint/build 历史 | 直接运行 `cargo test/clippy` |

### 4.4 与 FUNCTION_TREE.md 的联动

| 时机 | 动作 |
|------|------|
| `/ft:init --ref <node>` | 在治理树中标记关联的 FT 节点 |
| `/ft:observe` | 如果发现 FT 节点状态与实际不符，标记为需更新 |
| `/ft:closeout` | 检查 FT 节点是否需要更新状态/证据/边界 |

### 4.5 Git Hook 守卫集成（Phase 3）

在 `.claude/settings.json` 的 hooks 中添加：

```json
{
  "hooks": {
    "PostToolUse": [{
      "matcher": "Bash|Edit|Write",
      "command": "bash .governance/guards/ft-scope-check.sh",
      "description": "Check file edits against active FUNCTION_TREE governance authorization"
    }]
  }
}
```

守卫逻辑（两层检查）：
- `.governance/guards/ft-scope-check.sh` 是项目内 thin wrapper，实际调用 Skill 的 `scripts/ft-governance.cjs scope-check`。
- **L1 路径匹配**（始终运行）：检查编辑文件是否在 `authorized_paths` 内
- **L2 GitNexus detect_changes**（通用 Skill 可选；quantix-rust profile 强制）：符号级影响分析，检测 authorized_paths 内文件的未授权导出项变更
- 超出范围时输出警告（hooks 不能阻止，只能提醒）

---

## 5. 与现有工具的关系

### 5.1 FUNCTION_TREE vs GSD vs GitNexus

```
                    规划力          治理力          代码智能
                    (Planning)     (Governance)   (Code Intelligence)
 ─────────────────────────────────────────────────────────────────
 GSD              ████████████    ██░░░░░░░░░░   ░░░░░░░░░░░░
 FUNCTION_TREE    ████░░░░░░░░    █████████████  ░░░░░░░░░░░░   ← 本规划扩展
 GitNexus         ░░░░░░░░░░░░    ░░░░░░░░░░░░   █████████████
```

| 层面 | GSD | FUNCTION_TREE (扩展后) | GitNexus |
|------|-----|----------------------|----------|
| 规划 | legacy planning docs / phase notes / implementation plans | 不参与规划 | 不参与规划 |
| 授权 | PLAN.md 隐含授权 | 显式 allowed/forbidden paths | 提供影响数据 |
| 实现 | execute-phase | 绑定 PR，验证范围 | detect_changes |
| 验证 | verify-work | closeout 检查 | impact analysis |
| 注册 | STATE.md | FUNCTION_TREE.md | 知识图谱 |

### 5.2 不与 GSD 竞争

FUNCTION_TREE 治理层不替代 GSD 的规划/执行流程，而是补充 GSD 不提供的：

| GSD 不提供的 | FUNCTION_TREE 治理层提供 |
|-------------|------------------------|
| 证据/授权/实现的四层分离 | 每层有独立状态和禁止操作 |
| 显式 forbidden scope | 每个节点列出禁止操作 |
| 陈旧证据检测 | current_head 匹配检查 |
| 兼容面决策跟踪 | 独立的保留/退役决策 |
| 跨 Agent 上下文恢复 | `/ft:gate` 30 秒恢复 |

### 5.3 可选协作模式

GSD 和 FUNCTION_TREE 治理层可以串联使用：

```
/gsd:plan-phase 5           → GSD 创建执行计划
/ft:init handlers-split     → FT 为该计划创建治理程序
/ft:observe H1.1            → FT 收集证据（可调 GitNexus）
/ft:authorize H3.1          → FT 授权具体实现范围
/gsd:execute-phase 5        → GSD 执行（FT 监控范围）
/ft:closeout H3.1           → FT 关闭节点
```

---

## 6. 实施计划

### Phase 1: 核心 Skill + 模板（1-2 天）

| 步骤 | 产物 | 依赖 |
|------|------|------|
| 1.1 创建 Skill 目录结构 | `skills/function-tree/` in `chengjon/myskills` | 无 |
| 1.2 编写 SKILL.md 主入口 | YAML frontmatter + trigger-only description + 命令总览 | 1.1 |
| 1.3 编写 `/ft:init` 命令 | `commands/ft-init.md` | 1.2 |
| 1.4 编写 `/ft:observe` 命令 | `commands/ft-observe.md` | 1.2 |
| 1.5 编写 `/ft:authorize` 命令 | `commands/ft-authorize.md` | 1.2 |
| 1.6 编写 `/ft:implement` 命令 | `commands/ft-implement.md` | 1.2 |
| 1.7 编写 `/ft:closeout` 命令 | `commands/ft-closeout.md` | 1.2 |
| 1.8 编写 `/ft:gate` 命令 | `commands/ft-gate.md` | 1.2 |
| 1.9 编写 `/ft:status` 命令 | `commands/ft-status.md` | 1.2 |
| 1.10 创建确定性 CLI | `scripts/ft-governance.cjs` | 1.2 |
| 1.11 创建模板文件 | `templates/*.md, *.json, *.yaml` | 1.10 |
| 1.12 创建 profile/reference | `references/STATE_MACHINE.md`, `references/QUANTIX_PROFILE.md` | 1.10 |
| 1.13 以 `handlers-split` 为首个治理程序验证 | `.governance/programs/handlers-split/` | 1.3-1.12 |
| 1.14 Skill 验证 | quick_validate + realistic forward-test prompts | 1.13 |

### Phase 2: GitNexus 集成（半天）

| 步骤 | 产物 | 依赖 |
|------|------|------|
| 2.1 `/ft:observe` 自动调 GitNexus | evidence 自动收集 | Phase 1 |
| 2.2 `/ft:authorize` 自动验证上游影响 | d=1 依赖列表 | Phase 1 |
| 2.3 `/ft:implement` 调 `detect_changes` | 范围检查 | Phase 1 |

### Phase 3: Git Hook 守卫（半天）

| 步骤 | 产物 | 依赖 |
|------|------|------|
| 3.1 编写 `ft-scope-check.sh` | 守卫脚本 | Phase 1 |
| 3.2 集成到 `.claude/settings.json` hooks | 配置变更 | 3.1 |
| 3.3 以 `handlers-split` 测试守卫 | 验证 | 3.2 |

### Phase 4: 独立 CLI（可选，后续）

将 Skill 逻辑提取为独立 `ft` CLI 工具，可在 CI/CD 和非 Claude Code 环境中使用。此阶段不在本次规划范围内。

---

## 7. 效率提升预估

| 场景 | 当前（无治理层） | 扩展后（有治理层） | 提升 |
|------|----------------|-------------------|------|
| 新 Agent 恢复上下文 | 读 30 分钟历史对话 | `/ft:gate` 30 秒 | ~60x |
| 意外范围蔓延 | 代码审查或运行时发现 | `/ft:authorize` 显式范围 + 守卫 | 预防性 |
| handlers.rs 拆分 | 一次全拆 → 大 PR → 难审 | 每批一个 domain → 小 PR | PR 体积 -80% |
| 兼容面误删 | 编译时或运行时才发现 | closeout 检查 re-export 完整性 | 预防性 |
| 陈旧证据误导 | Agent 用旧扫描结果改代码 | `stale_if_head_mismatch` 阻止 | 预防性 |
| FUNCTION_TREE 更新滞后 | 改了代码忘了更新 FT | closeout 自动检查 FT 节点 | 及时性 |

---

## 8. 审核清单

请审核以下决策点：

- [ ] **命名**：统一使用 FUNCTION_TREE（不用 Steward Tree），Skill 命令前缀 `/ft:`
- [ ] **目录**：治理层放 `.governance/`（不在 `.planning/` 内，避免与 GSD 冲突）
- [ ] **状态机**：11 个状态（含 `blocked`）和强制规则是否合理
- [ ] **外部工具策略**：Git 硬依赖；通用 Skill 中 `gh`/OPENDOG/GitNexus 可选增强；quantix-rust profile 中 GitNexus 对 symbol edit / commit gate 强制
- [ ] **双轨文件**：active-gates.json（机器读写）+ active-gates.md（人类查看），与 nodes.json/tree.md 模式一致
- [ ] **任务卡 acceptance 分 tier**：commit_gate（cargo check + clippy）vs closeout_gate（build --release + test）
- [ ] **forbidden_paths 放宽**：推荐但不强制；non_goals 必须至少一项
- [ ] **/ft:gate 两级**：默认快速（30 秒），--verbose 完整上下文
- [ ] **守卫两层**：L1 路径匹配（始终）+ L2 GitNexus detect_changes（通用可选，quantix-rust 强制）
- [ ] **与 FUNCTION_TREE.md 的关系**：不改动 FT.md 现有结构，仅通过 closeout 更新状态
- [ ] **与 GSD 的关系**：不替代 GSD，补充 GSD 不提供的治理能力
- [ ] **首个治理程序**：以 `handlers-split` 作为验证目标
- [ ] **Phase 划分**：Phase 1 (Skill + OPENDOG/GitNexus 可选集成) → Phase 2 (GitNexus 深化) → Phase 3 (Hook 两层守卫)

---

## 附录 A: Steward Tree 实践 → FUNCTION_TREE 治理层的概念映射

| Steward Tree 概念 | FUNCTION_TREE 治理层对应 |
|-------------------|------------------------|
| Steward Tree | 治理程序 (`program`) |
| Node | 治理节点 (`node`) |
| Lane | 不使用（扁平化节点 + parent_program） |
| Active Gate Register | `.governance/active-gates.md` |
| Task Card | `.governance/programs/<name>/cards/<node-id>.yaml` |
| Machine Index | `.governance/programs/<name>/nodes.json` |
| Human Tree | `.governance/programs/<name>/tree.md` |
| Closeout Report | `tree.md` 内的关闭记录 |
| Evidence Package | `/ft:observe` 的输出 |
| Authorization Package | `/ft:authorize` 的输出 |
| Forbidden Scope | 每个节点的 `forbidden_paths` + `forbidden` 列表 |

## 附录 B: 反模式对照

| 反模式 | FUNCTION_TREE 治理层如何防止 |
|--------|---------------------------|
| 证据包包含可选代码修复 | `/ft:observe` 强制 state=evidence-prepared，禁止 source_edits_authorized |
| 候选扫描变成待办池 | `/ft:authorize` 要求每个候选独立分类和授权 |
| 合并后跳过 closeout | `/ft:implement` → `/ft:closeout` 是强制状态流 |
| 兼容面被静默删除 | closeout 检查 re-export 完整性 |
| Issue label 被当成完整状态 | 状态机独立于 GitHub labels |
| 生成的 artifact 缺少 commit 元数据 | nodes.json 强制 `current_head` + `stale_if_head_mismatch` |
