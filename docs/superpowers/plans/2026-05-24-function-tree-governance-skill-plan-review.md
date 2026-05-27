# FUNCTION_TREE Governance Skill Plan — OPENDOG 视角审核意见

> Reviewed from: OPENDOG 项目（/opt/claude/opendog）
> Reviewer: Claude (GLM-5.1) + OPENDOG observation/guidance/verification 视角
> Date: 2026-05-24
> Target: [2026-05-24-function-tree-governance-skill-plan.md](./2026-05-24-function-tree-governance-skill-plan.md)

---

## 0. 关于 OPENDOG

OPENDOG 是一个多项目观察与 AI 决策支持系统，运行在 WSL2 上。它跟踪 AI 工具（Claude Code、Codex、GPT、GLM）访问了哪些文件，识别未使用/陈旧文件与活跃核心文件，并通过 daemon、CLI 和 MCP 三层入口暴露可复用的 operator/AI 操作界面。

OPENDOG 与本治理方案的关联点在于：**它提供运行时观察证据**，补充 GitNexus 的代码结构智能。

### OPENDOG 核心 AI 面向能力速查

| 能力 | MCP 工具 | CLI 等价 | 与 FT 治理的关联 |
|------|---------|---------|----------------|
| 项目级文件活跃度统计 | `get_stats` | `opendog stats --id <ID>` | `/ft:observe` 可引用活跃度证据 |
| 未使用文件检测 | `get_unused_files` | `opendog unused --id <ID>` | 拆分目标文件是否活跃的依据 |
| 时间窗口活动报告 | `get_time_window_report` | `opendog report window` | 证据收集期间的文件活动快照 |
| 快照差异对比 | `compare_snapshots` | `opendog report compare` | `/ft:implement` 前后的文件级增量验证 |
| 使用趋势（冷热） | `get_usage_trends` | `opendog report trend` | 治理目标文件的长期活跃趋势 |
| 基线快照建立 | `take_snapshot` | `opendog snapshot --id <ID>` | `/ft:init` 后建立文件基线 |
| 验证证据（test/lint/build） | `get_verification_status` | `opendog verification --id <ID>` | `/ft:closeout` 的验证证据来源 |
| AI 指导与推荐 | `get_guidance` | `opendog agent-guidance` | 跨项目优先级 + 下一步建议 |
| 数据风险审查 | `get_data_risk_candidates` | `opendog data-risk --id <ID>` | 治理目标范围内的 mock/hardcoded 风险 |
| 跨项目优先级 | `get_workspace_data_risk_overview` | `opendog workspace-data-risk` | 选择治理程序优先级 |

详细工具用法：[OPENDOG MCP Tool Reference](/opt/claude/opendog/docs/mcp-tool-reference.md)
AI 使用工作流：[OPENDOG AI Playbook](/opt/claude/opendog/docs/ai-playbook.md)

---

## 1. 总体评价

设计非常成熟。状态机四层分离（evidence → decision → authorize → implement）解决了 AI Agent 治理中的核心痛点。与 GSD 的互补定位清晰，不争夺职责。不创建平行树、不修改 FUNCTION_TREE.md 现有结构的原则正确。

以下建议按优先级排列。

---

## 2. 建议 A（高优先级）：让 OPENDOG 成为 `/ft:observe` 的运行时证据源

### 问题

当前设计中，`/ft:observe` 的证据收集完全依赖 GitNexus（代码结构智能：符号关系、调用链、影响范围）。这覆盖了"代码长什么样"的维度，但缺少"代码运行时被怎么用"的维度。

### 建议

在 `/ft:observe` 的自动收集步骤中增加 OPENDOG 作为可选证据源：

| 治理阶段 | OPENDOG 工具 | 补充的证据维度 |
|---------|-------------|-------------|
| `/ft:init` 后 | `take_snapshot` | 建立文件清单基线（大小、路径、元数据） |
| `/ft:observe` | `get_stats` + `get_unused_files` | 文件活跃度：哪些被频繁访问、哪些从未被访问 |
| `/ft:observe` | `get_time_window_report` | 证据收集期间的文件活动快照 |
| `/ft:authorize` | `get_usage_trends` | 授权目标文件的长期冷热趋势（判断拆分优先级） |
| `/ft:implement` 后 | `compare_snapshots` | 验证实际改动范围 vs 授权范围 |
| `/ft:closeout` | `get_verification_status` | test/lint/build 历史作为关闭证据 |

### 具体集成点

在 `nodes.json` 的 `source_evidence` 中增加 OPENDOG 证据路径：

```json
{
  "source_evidence": [
    "docs/reports/handlers-split-trade-analysis.md",
    "opendog://stats/quantix-rust",
    "opendog://snapshot-diff/run-12/run-15",
    "opendog://verification/quantix-rust"
  ]
}
```

在 `/ft:observe` 命令描述中增加一步：

> 1. 调用 GitNexus 工具收集代码智能证据（现有）
> 2. **调用 OPENDOG MCP 工具收集运行时观察证据（新增）**
>    - `get_stats` → 文件活跃度
>    - `get_time_window_report` → 近期活动形状
>    - `get_unused_files` → 未使用文件列表
> 3. 合并两种证据写入 tree.md 证据账本

### 为什么重要

以 `handlers-split` 为例：GitNexus 告诉你 `trade_handler` 有哪些调用者和被调用者（结构视角），OPENDOG 告诉你这个文件在过去 7 天是否被活跃编辑/访问（运行时视角）。两者结合才能做出好的拆分优先级决策。

---

## 3. 建议 B（高优先级）：增加 `active-gates.json` 机器索引

### 问题

`active-gates.md` 是 Markdown 表格，但 `/ft:gate` 需要解析它来恢复上下文。Markdown 表格解析脆弱且容易出错（列对齐、特殊字符、合并单元格等）。

你的设计在 `nodes.json`（机器索引）+ `tree.md`（人类可读）上已经用了双轨模式，但 `active-gates.md` 缺少机器可读的对应物。

### 建议

增加 `.governance/active-gates.json`：

```
.governance/
├── active-gates.json          # 机器索引（Skill 读写此文件）
├── active-gates.md            # 人类可读（由 .json 生成，单向同步）
└── programs/
    └── ...
```

`active-gates.json` 结构：

```json
{
  "updated_at": "2026-05-24T10:00:00+08:00",
  "active_nodes": [
    {
      "gate": "H3.1",
      "program": "handlers-split",
      "state": "authorization-prepared",
      "current_blocker": "Awaiting human review",
      "next_allowed": "implement: create trade domain handler files",
      "forbidden": ["source edits until approved"],
      "ft_ref": "cli/",
      "current_facts": "trade_handler has 47 callers, 3 d=1 WILL BREAK dependencies"
    }
  ]
}
```

所有 Skill 命令读写 JSON，Markdown 仅作人类查看用途。

---

## 4. 建议 C（高优先级）：增加 `blocked` 状态

### 问题

10 个状态覆盖了正常流转，但没有处理异常路径：

- 证据收集发现阻塞项（如上游依赖未就绪）
- 授权被驳回（人工审核不同意 allowed_paths）
- 实现中途发现需要回退（PR 审查发现设计问题）

当前设计中这些情况只能"停留在上一个状态"，语义模糊——无法区分"正在正常收集证据"和"收集证据时发现了阻塞项"。

### 建议

增加 `blocked` 状态：

```
evidence-prepared ──────► blocked (发现阻塞项)
     ▲                      │
     └──────────────────────┘ (阻塞解除，补充证据)

authorization-prepared ──► blocked (授权被驳回)
     ▲                      │
     └──────────────────────┘ (修订授权包)

approved-for-implementation ► blocked (实现中发现超出预期)
     ▲                      │
     └──────────────────────┘ (回退到 revise authorization)
```

`blocked` 状态的约束：

| 字段 | 要求 |
|------|------|
| `blocker_reason` | 必填，记录阻塞原因 |
| `next_gate` | 固定为 "resolve blocker" |
| `source_edits_authorized` | 必须为 `false` |
| `unblock_target_state` | 阻塞解除后应回到的状态 |

nodes.json 中：

```json
{
  "state": "blocked",
  "blocker_reason": "upstream dependency `data_loader` not yet refactored, see F1.2",
  "unblock_target_state": "evidence-prepared"
}
```

---

## 5. 建议 D（中优先级）：陈旧证据检测增加快照漂移维度

### 问题

`stale_if_head_mismatch` 用 git commit hash 做陈旧检测是正确的硬依赖。但在两次 commit 之间，AI agent 可能多次编辑文件但不 commit——此时 commit hash 不变但代码已大幅偏离。

### 建议

在 `stale_if_head_mismatch` 之外增加可选的 `stale_if_snapshot_drift` 检查：

```json
{
  "current_head": "a1b2c3d",
  "stale_if_head_mismatch": true,
  "evidence_snapshot_run_id": 12,
  "stale_if_snapshot_drift": true
}
```

检查逻辑（在 `/ft:authorize` 时执行）：

1. 调用 OPENDOG `compare_snapshots`，base_run_id = evidence_snapshot_run_id
2. 如果 diff 中有文件与 authorized_paths 有交集 → 触发陈旧警告
3. 不阻止流转，但强制更新 evidence 并重新走 observe

这利用了 OPENDOG 的 snapshot diff 能力，不依赖 git commit，覆盖了"未 commit 的漂移"场景。

---

## 6. 建议 E（中优先级）：`/ft:gate` 上下文恢复应分两级

### 问题

宣称 30 秒恢复上下文，但当前只输出一个表格。新 Agent 知道"H3.1 当前阻塞是 evidence collection"还不够，还需要知道**之前已经收集了什么、结论是什么**。

### 建议

`/ft:gate` 输出分两级（与 OPENDOG guidance 的 `detail=summary` vs `detail=decision` 模式一致）：

**快速模式**（默认 `/ft:gate`）：

```
Active Gates (2 nodes)
─────────────────────────────────────────────────────
| Gate | Program        | State        | Next                |
|------|----------------|--------------|---------------------|
| H3.1 | handlers-split | auth-prepared| Implement trade     |
| F2.3 | factor-pipeline| evidence     | Collect coverage    |

Facts summary:
  H3.1: 47 callers, 3 WILL BREAK, trade domain isolated
  F2.3: 12 operators, 4 missing tests, pipeline incomplete
```

**详细模式**（`/ft:gate --verbose`）：

- 包含完整 `source_evidence` 路径列表
- 每个 `next_gate` 的完整描述
- `forbidden_paths` 完整列表
- 上次状态变更时间和触发者

---

## 7. 建议 F（中优先级）：Git Hook 守卫应分两层检查

### 问题

Phase 3 的 `ft-scope-check.sh` 守卫脚本计划做简单路径匹配。但路径匹配粒度粗——无法检测"改了 authorized_paths 内的文件，但改动了该文件的未授权导出项"。

### 建议

守卫分两层：

| 层级 | 方法 | 精度 | 依赖 |
|------|------|------|------|
| L1 | 路径匹配（`ft-scope-check.sh`） | 文件级 | 无 |
| L2 | GitNexus `detect_changes` | 符号级 | GitNexus 可用 |

L1 始终运行。L2 仅在 GitNexus 可用时运行，提供更精确的符号级影响分析。两层都通过才放行，否则警告并附上影响范围详情。

---

## 8. 建议 G（低优先级）：与 OPENDOG guidance 的潜在集成

### 问题

Section 5.2 正确指出 FT 治理层不与 GSD 竞争，但未提到 OPENDOG 的 guidance 系统。

### 建议

这是一个未来扩展点，不需要现在实现。当 OPENDOG 检测到 `.governance/` 目录存在且包含活跃门控时，可以在 `get_guidance` 的 payload 中增加提示：

```json
{
  "guidance": {
    "layers": {
      "governance": {
        "active_programs": 2,
        "blocked_nodes": ["H3.1"],
        "next_gate": "/ft:authorize H3.1"
      }
    }
  }
}
```

当前阶段可以在 Skill 层面的 `/ft:gate` 中整合 OPENDOG guidance 的结果（反向：FT 调用 OPENDOG），未来可以双向打通。

---

## 9. 小问题（低优先级）

### 9.1 cards/*.yaml 的 acceptance.checks 分 tier

`cargo build --release` 很慢。建议分 tier：

- **提交门控**（快速）：`cargo check` + `cargo clippy`
- **closeout 门控**（完整）：`cargo build --release` + `cargo test`

### 9.2 forbidden_paths "至少一个禁止项" 过于严格

有些简单重构确实没有需要禁止的路径。建议改为：

- `non_goals` 必须至少一项（保证思考过范围边界）
- `forbidden_paths` 推荐但不强制

### 9.3 nodes.json 的 current_head 更新时机未说明

建议明确：在 `/ft:observe` 时写入，在 `/ft:authorize` 时校验，在 `/ft:implement` 时更新为 merge commit。

---

## 10. 审核建议汇总

| 编号 | 建议 | 优先级 | 改动范围 |
|------|------|--------|---------|
| A | OPENDOG 作为 `/ft:observe` 运行时证据源 | 高 | `/ft:observe` 命令描述 + nodes.json schema |
| B | 增加 `active-gates.json` 机器索引 | 高 | 目录结构 + Skill 命令读写逻辑 |
| C | 增加 `blocked` 状态 | 高 | 状态机设计 + nodes.json schema |
| D | 陈旧证据增加快照漂移检测 | 中 | nodes.json schema + `/ft:authorize` 逻辑 |
| E | `/ft:gate` 分快速/详细两级 | 中 | `/ft:gate` 命令描述 |
| F | Git Hook 守卫分两层检查 | 中 | Phase 3 守卫脚本设计 |
| G | OPENDOG guidance 集成（未来） | 低 | OPENDOG 扩展点，当前不实现 |
| 9.1 | acceptance.checks 分 tier | 低 | 任务卡模板 |
| 9.2 | forbidden_paths 放宽为推荐 | 低 | `/ft:authorize` 约束规则 |
| 9.3 | current_head 更新时机明确 | 低 | nodes.json schema 说明 |

**核心设计无需大改**。A/B/C 三项建议如果采纳，可以在 Phase 1 一并落地，不增加额外 Phase。
