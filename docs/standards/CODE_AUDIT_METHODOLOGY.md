# quantix-rust 代码全面审核方法论

> 版本: 1.2-draft
> 更新日期: 2026-05-11
> 状态: 候选版 — 已根据审查意见修订，执行前仍需采集当前代码基线
> 适用范围: quantix-rust 项目全量代码审核
> 
> 状态源说明：本文是代码审核方法论，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 [`FUNCTION_TREE.md`](../../FUNCTION_TREE.md) 的状态注册表行为准。

---

## 目录

- [一、审核目标与原则](#一审核目标与原则)
- [二、审核覆盖范围](#二审核覆盖范围)
- [三、审核前基线采集](#三审核前基线采集)
- [四、工具矩阵](#四工具矩阵)
- [五、审核维度定义](#五审核维度定义)
  - [5.1 架构合理性评估 (权重: 30%)](#51-架构合理性评估-权重-30)
  - [5.2 CLI 菜单与功能安排评估 (权重: 20%)](#52-cli-菜单与功能安排评估-权重-20)
  - [5.3 业务流程与逻辑正确性评估 (权重: 25%)](#53-业务流程与逻辑正确性评估-权重-25)
  - [5.4 代码质量与规范合规性评估 (权重: 15%)](#54-代码质量与规范合规性评估-权重-15)
  - [5.5 测试覆盖与质量评估 (权重: 10%)](#55-测试覆盖与质量评估-权重-10)
- [六、审核工作流程](#六审核工作流程)
- [七、发现项管理与输出](#七发现项管理与输出)
- [八、风险分级与判定标准](#八风险分级与判定标准)
- [九、输出交付物](#九输出交付物)
- [十、审核执行最佳实践](#十审核执行最佳实践)
- [附录 A: 模块依赖关系拓扑](#附录-a-模块依赖关系拓扑)
- [附录 B: 审核用类型速查表](#附录-b-审核用类型速查表)
- [附录 C: 审核自检清单](#附录-c-审核自检清单)

---

## 一、审核目标与原则

### 1.1 审核目标

本次代码全面审核的目标是：

1. **评估架构合理性**：确认模块划分、依赖方向、抽象层次是否符合项目编码规范的设计意图
2. **评估功能安排**：确认 CLI 命令树设计是否一致、符合直觉、无冗余/遗漏
3. **评估业务正确性**：确认关键执行链路（策略→信号→订单→执行→风控）的逻辑闭环与语义一致性
4. **识别风险与债务**：发现架构偏移、接口不匹配、静默回退、MOCK 混淆等系统性问题
5. **提供可执行的改进建议**：按风险优先级给出具体的重构/修复建议

### 1.2 审核原则

| 原则 | 说明 |
|------|------|
| **事实优先** | 所有结论必须有代码引用、测试输出或静态分析结果支撑，不允许"看起来像"式的推测 |
| **边界验证** | MOCK 路径、真实路径、fallback 路径必须区分清楚，不得混为一谈 |
| **向后兼容** | 公共 API 变更风险优先评估；内部重构敏感度低于公共接口变更 |
| **分层审查** | 从架构→模块→文件→函数的层次逐步下钻，避免一上来就陷入细节 |
| **增量报告** | 每个发现标注严重级别（S0/S1/S2/S3/S4）和置信度（confirmed/probable/needs-repro） |
| **双向验证** | 静态代码分析 + 动态行为验证（测试、构建、CLI smoke）双向交叉确认 |

---

## 二、审核覆盖范围

### 2.1 代码库覆盖

正式审核前必须采集范围基线（采集命令见第三章）。以下为覆盖要求，具体数量以审核执行时的基线为准：

```
覆盖维度:                      覆盖要求:
─────────────────────────────────────────────────────
src/ 下全部子目录              每个模块至少覆盖: 入口文件 + 核心 trait + 主要实现
src/cli/commands/*.rs          全量命令定义文件审查
src/cli/handlers/*.rs          全量 handler 文件分发逻辑审查
tests/*.rs                     全量分类与外部依赖扫描；人工质量审查按 5.5.2 分层规则执行
config/ 下配置文件              配置项存在性、默认值合理性、与代码引用一致性
docs/ 下关键文档               README / docs/USER_MANUAL.md / 编码规范 / MOCK 规范与代码一致性
根目录 FUNCTION_TREE.md        功能树与能力边界文档与代码一致性
Cargo.toml                     依赖审查 (必要性、版本、feature flag 合理性)
```

> 最后一次观测值 (2026-05-11): src 顶层 28 子目录, cli/commands 13 文件, cli/handlers 30 文件, tests/ 72 文件。执行审核时必须重新采集。

### 2.2 全量扫描 vs 人工深审

"全面审核"指全量自动扫描 + 分层人工深审，两层不可混淆：

**全量自动扫描** (覆盖全部文件):
- 所有 `.rs` 文件进入模式扫描（unwrap/panic/println/TODO/unsafe）
- 所有文件大小统计
- 所有公共 API 签名采集
- 所有 CLI command/handler 文件进入分发完整性检查
- 所有 `tests/*.rs` 进入测试分类和外部依赖扫描

**人工深审** (分层覆盖):
- P0/P1 模块：全量深审关键路径
- P2/P3 模块：每个模块至少审查
  - 入口文件 (`mod.rs`)
  - 一个主要 service / provider / adapter
  - 一个持久化或外部依赖点（如存在）
  - 相关测试不少于 2 个；若模块测试少于 2 个则全量审查

**测试审查密度**:
- P0/P1 相关测试: 全量审查
- P2/P3 相关测试: 至少 20%，且不少于每模块 2 个
- repo hygiene / script / smoke / gate 测试: 全量审查

### 2.3 明确排除范围

以下内容不在本次审核范围内（除非与主线代码产生交叉影响）：

- `.worktrees/` 下的其他工作树
- `target/` 构建产物
- `.gitnexus/` 索引数据
- `.omc/` 会话状态
- `logs/` 运行时日志
- Python quantix 项目（仅审查 Rust 侧与其数据库/配置共享契约）

### 2.4 审核优先级矩阵

```
优先级      模块范围                       理由
───────────────────────────────────────────────────────────
P0-A      execution/         执行内核：订单生命周期、状态机、reconciliation
P0-B      strategy/          策略引擎：信号生成、daemon、运行时数据库
P0-C      risk/              风控引擎：规则评估、行业同步、回撤控制
───────────────────────────────────────────────────────────
P1-A      cli/               命令枚举 + handler 分发逻辑
P1-B      bridge/            跨系统桥接：TDX、QMT 契约
P1-C      monitor/           实时监控：告警、事件存储
P1-D      monitoring/        可观测性：health、metrics、notification
───────────────────────────────────────────────────────────
P2-A      analysis/          回测引擎、技术指标
P2-B      screener/          选股器
P2-C      sources/           数据源适配器
P2-D      stop/              止盈止损
P2-E      trade/             模拟交易
P2-F      watchlist/         自选池
───────────────────────────────────────────────────────────
P3-A      account/           多账户管理
P3-B      ai/                AI 决策模块
P3-C      anomaly/           异常检测
P3-D      fundamental/       基本面数据
P3-E      market/            市场分析
P3-F      news/              新闻搜索
P3-G      import/            导入模块
P3-H      io/                导入导出
P3-I      factor/            因子管线
P3-J      sync/              数据同步
P3-K      tasks/             任务调度
P3-L      tui/               TUI 界面
P3-M      core/              核心工具与配置
P3-N      db/                数据库客户端
P3-O      data/              数据模型
```

---

## 三、审核前基线采集

正式审核前必须先采集以下基线，并记录到审核报告 `baseline.md` 中：

### 3.1 环境基线

```bash
# Git 状态
git rev-parse HEAD
git status --porcelain

# Rust 工具链
rustc --version
cargo --version

# GitNexus 索引状态
# 读取 gitnexus://repo/quantix-rust/context 确认索引新鲜度
# 如 stale，先运行 gitnexus analyze
```

### 3.2 范围基线

```bash
# 模块清单
find src -mindepth 1 -maxdepth 1 -type d | sort

# CLI 命令定义文件
find src/cli/commands -maxdepth 1 -type f -name '*.rs' | sort

# CLI handler 文件
find src/cli/handlers -maxdepth 1 -type f -name '*.rs' | sort

# 集成测试文件
find tests -maxdepth 1 -type f -name '*.rs' | sort

# 配置文件
find config -maxdepth 1 -type f | sort

# 关键文档
echo "README.md docs/USER_MANUAL.md FUNCTION_TREE.md docs/RUST_CODING_STANDARDS.md docs/standards/MOCK_USAGE_POLICY.md" \
  | tr ' ' '\n' | while read f; do [ -f "$f" ] && echo "$f"; done
```

### 3.3 基线用途

采集结果写入 `docs/CODE_AUDIT_EVIDENCE/baseline.md`，用于：
- 审核报告中的"范围"章节引用
- 后续 reviewer 验证审核覆盖面
- 历史对比（不同时间点的审核范围变化）

---

## 四、工具矩阵

### 4.1 工具总览

按"能力 + 推荐实现"方式组织，不绑定特定工具名称：

| 能力 | 推荐实现 | 优先级 |
|------|----------|--------|
| 文本搜索 (regex/grep) | `rg` (ripgrep)，不可用时 `git grep` | P0 |
| 文件枚举 | `rg --files`, `find` (需稳定排序) | P0 |
| 文件读取 | 当前执行环境的 read/cat 工具，记录引用行号 | P0 |
| 格式检查 | `cargo fmt --check` | P0 |
| 静态 lint | `cargo clippy --all-targets --all-features` | P0 |
| 构建验证 | `cargo build --release` (完整 gate，可按环境成本延后) | P0 |
| 测试回归 | `cargo test --all-targets` | P0 |
| 知识图谱查询 | GitNexus `query` — 按概念查找执行流 | P0 |
| 符号上下文 | GitNexus `context` — 360° callers/callees + 参与执行流 | P1 |
| 影响分析 | GitNexus `impact` — 修改前必须执行 | P1 |
| 变更检测 | GitNexus `detect_changes` — 提交前必须执行 | P2 |
| 依赖审查 | `cargo audit`, `cargo tree --duplicates` | P2 |
| 历史决策/审查回顾 | Graphiti MCP — `search` 特定 group_id | P2 |
| 功能验证 | `scripts/verify_features.sh` | P2 |

### 4.2 GitNexus 使用指南

项目已由 GitNexus 索引（5410 符号, 13194 关系, 300 执行流）。使用以下资源入口：

```
# 代码库概览与索引新鲜度
READ gitnexus://repo/quantix-rust/context

# 功能聚类
READ gitnexus://repo/quantix-rust/clusters

# 全部执行流
READ gitnexus://repo/quantix-rust/processes

# 特定执行流追踪
READ gitnexus://repo/quantix-rust/process/{processName}
```

**常用查询**：

```
# 按概念查找执行流
gitnexus_query(query="execution request lifecycle")

# 符号上下文 (callers + callees + 参与的执行流)
gitnexus_context(name="ExecutionKernel")

# 上游影响分析 (谁依赖此符号)
gitnexus_impact(target="ExecutionRequest", direction="upstream")
```

### 4.3 违规模式搜索

#### 搜索模式清单

| 搜索模式 (regex) | 对应规范条款 | 严重级别 |
|-----------------|-------------|---------|
| `\.unwrap\(\)` | 禁止 unwrap() | HIGH |
| `\.expect\(` | 禁止 expect() (生产代码) | HIGH |
| `panic!\(` | 禁止 panic!() | HIGH |
| `println!\(` in lib modules | 库代码禁止 println | MEDIUM |
| `TODO[^-]` (无 issue 编号) | TODO 需带 tracking | MEDIUM |
| `let _ = ` (吞咽错误) | 禁止吞错误 | MEDIUM |
| `unsafe \{` | 禁止 unsafe | CRITICAL |

#### 排除规则

搜索时必须排除以下目录：

```
target/
.git/
.gitnexus/
.worktrees/
logs/
```

#### 统计分桶

每个搜索模式的结果按以下分桶统计并报告：

| 桶 | 范围 | 说明 |
|----|------|------|
| `production` | `src/` 下非 `#[cfg(test)]` 模块内的命中 | 需要修复的代码 |
| `unit_test` | `src/` 下 `#[cfg(test)] mod tests` 内的命中 | 测试代码，可接受但需标注 |
| `integration_test` | `tests/` 下的命中 | 测试代码，可接受但需标注 |
| `bench_example` | `benches/`、`examples/` 下的命中 (如存在) | 可接受但需标注 |
| `non_code` | `docs/`、`scripts/`、`config/` 下的命中 | 非代码文件，标注即可 |

#### 报告格式

每次模式扫描的输出必须包含：

```text
Pattern: \.unwrap\(\)
  total_matches: N
  production_matches: N
  test_matches: N
  exempted_matches: N
  actionable_matches: N
```

---

## 五、审核维度定义

### 5.1 架构合理性评估 (权重: 30%)

#### 5.1.1 模块依赖方向

验证是否遵循单向下行依赖：

```
CLI 层 ──→ Service 层 ──→ Provider/Adapter 层 ──→ Domain 层 ──→ Core 层
```

**检查项：**

- [ ] 是否存在循环依赖（如 A → B → A）
- [ ] 是否存在反向依赖（如 core 依赖 cli）
- [ ] `lib.rs` 是否仅包含 `pub mod` + `pub use` 声明（当前测量: ~50 行，合规）
- [ ] `main.rs` 是否简洁（当前: 仅日志初始化 + 命令分发）
- [ ] 各 `mod.rs` 是否仅包含子模块声明和重导出

#### 5.1.2 抽象层次一致性

**检查项：**

- [ ] Trait 是否定义在独立的 `provider.rs` / `adapter.rs` 中
- [ ] Service 是否不直接依赖具体实现，而是依赖 Trait
- [ ] 是否存在穿透抽象层的直接调用（如 CLI 直接调 DB 客户端）

#### 5.1.3 执行架构边界

基于 WSL2-Windows Bridge 架构文档 (`docs/architecture/WSL2_WINDOWS_BRIDGE_ARCHITECTURE.md`) 的明确边界：

```
quantix-rust (WSL2)                   quantix-bridge (Windows)
─────────────────────────             ────────────────────────
ExecutionKernel                       远端能力边界 (无状态)
execution_request                     非执行状态机
runtime.db                            TDX Service
risk evaluation                       QMT Preview / Task Submit
order/order_event source of truth     不拥有本地状态
```

**检查项：**

- [ ] Bridge 侧是否越界持有执行状态
- [ ] 真实提交路径是否仅通过 guarded `qmt_live`（`QmtLiveExecutionAdapter`）
- [ ] `mock_live` 是否被误写作实盘能力
- [ ] 是否存在真实路径失败后静默回退到 MOCK
- [ ] `QmtBridgePreviewAdapter` (src/execution/qmt_bridge.rs) 是否明确标注为 preview-only

#### 5.1.4 文件大小合规

| 阈值检查 | 标准 | 来源 |
|----------|------|------|
| lib.rs ≤ 150 行 | 需采集后确认 | `docs/RUST_CODING_STANDARDS.md` |
| main.rs ≤ 150 行 | 需采集后确认 | 同上 |
| 各 mod.rs ≤ 800 行 | 需逐文件核实 | 同上 |
| handlers/ 系列 ≤ 1200 行 | 需逐文件核实 | 同上 |
| 各 .rs 模块文件 ≤ 800 行 | 需逐文件核实 | 同上 |

### 5.2 CLI 菜单与功能安排评估 (权重: 20%)

#### 5.2.1 命令树审查

以下命令树从 `src/cli/commands/mod.rs` 的 `Commands` 枚举实际提取，反映代码事实：

```
quantix
├── init                    初始化配置和数据库
├── menu                    交互式菜单 (含 --tui)
├── status                  系统状态 (含 --health)
│
├── data         [子命令]   数据管理
├── strategy     [子命令]   策略管理
│   (含 strategy config/daemon/request/service/signal 子命令组)
├── task         [子命令]   任务调度 (实验性，Foundation P0)
├── analyze      [子命令]   分析工具
├── backtest     [子命令]   回测
├── performance  [子命令]   绩效
├── factor       [子命令]   因子研究
├── monitor      [子命令]   监控 (含 monitor alert/config/daemon/event/service 子命令组)
├── stop         [子命令]   止盈止损
├── watchlist    [子命令]   自选池 (含 watchlist group/tag 子命令组)
├── market       [子命令]   市场分析
├── trade        [子命令]   模拟交易
├── risk         [子命令]   风险管理 (含 risk import/lock/rebuild/rule/sync 子命令组)
├── execution    [子命令]   执行自动化 (含 execution bridge/config/daemon/qmt 子命令组)
├── anomaly      [子命令]   异常检测 (Isolation Forest)
├── algo         [子命令]   算法交易 (TWAP/VWAP)
├── account      [子命令]   账户管理 (含 account group 子命令组)
├── notify       [子命令]   通知
├── ai           [子命令]   AI 决策
├── news         [子命令]   新闻搜索
├── fundamental  [子命令]   基本面数据
├── sentiment    [子命令]   舆情分析
└── import       [子命令]   智能导入
```

**共 26 个顶层命令**（含 init/menu/status 3 个叶子命令 + 23 个子命令组）。

**检查项：**

- [ ] 命令命名是否一致（全名词，一致 ✓）
- [ ] 是否存在功能重叠（`monitor` vs `monitoring` — 前者为实时监控模块，后者为可观测性内部库模块，未暴露为命令）
- [ ] 是否有 `Commands` 枚举中声明但 handler 未分发的命令
- [ ] 子命令层次是否过深或过浅
- [ ] `--help` 输出是否清晰完整
- [ ] 与 `FUNCTION_TREE.md` 的状态标记是否一致

#### 5.2.2 功能完整性

对照 `FUNCTION_TREE.md`（项目根目录）中声明的能力，逐条验证：

- [ ] `[已实现]` 标记的功能是否真实可运行
- [ ] `[部分实现]` 标记的功能边界是否准确描述
- [ ] `[未实现]` / `[待实现]` 的功能是否有明确提示或占位
- [ ] 声明的 MOCK 限制是否在命令输出中有体现
- [ ] README 示例命令是否可执行

### 5.3 业务流程与逻辑正确性评估 (权重: 25%)

#### 5.3.1 关键类型定义（审核基准）

以下类型定义从实际代码提取，审核时以此为基准对照验证。

```rust
// src/execution/models.rs

pub enum OrderStatus {
    PendingSubmit,     // 待提交
    Submitted,         // 已提交
    Accepted,          // 已受理
    PartiallyFilled,   // 部分成交
    PendingCancel,     // 待撤销
    Filled,            // 全部成交 (终态)
    Canceled,          // 已撤销 (终态)
    Rejected,          // 已拒绝 (终态)
    Unknown,           // 状态未知 (异常)
}

pub enum ExecutionRequestStatus {
    Pending,           // 待处理
    InProgress,        // 进行中
    Completed,         // 已完成
    Failed,            // 失败
    Canceled,          // 已取消
}

pub enum StrategyRunStatus {
    Running,
    Success,
    Failed,
}

pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

// src/execution/kernel.rs:16

pub enum RiskDecision {
    Allow,
    Reject { reason: String },
}

// src/execution/kernel.rs:22

#[async_trait]
pub trait RiskEvaluator: Send + Sync {
    async fn evaluate(&self, intent: OrderIntent) -> Result<RiskDecision>;
}
```

#### 5.3.2 核心执行链路审查

**链路 1: 策略→信号→执行请求→订单→成交 (Paper/MockLive/QmtLive)**

```
strategy daemon
  → ConfiguredStrategyEvaluator::evaluate() → Signal
    → Signal → ExecutionRequestRecord (frozen snapshot)
      → ExecutionKernel::execute_request()
        → PaperExecutionAdapter / MockLiveExecutionAdapter / QmtLiveExecutionAdapter
          → Order → OrderEvent → FillDelta
            → runtime.db (持久化)
```

**检查项：**

- [ ] Signal → ExecutionRequest 的映射是否完整
- [ ] frozen snapshot 是否在 request 创建时固化
- [ ] ExecutionKernel 是否正确处理 `PendingCancel`、`Unknown`、`PartiallyFilled`
- [ ] reconciliation 是否正确对比请求快照与当前状态
- [ ] `ExecutionRequestStatus::Completed` 与 `OrderStatus` 终态 (`Filled`/`Canceled`/`Rejected`) 的语义区分是否正确

**链路 2: 风控评估**

```
OrderIntent (from signal translation)
  → RiskEvaluator::evaluate() → RiskDecision
    → Allow → 继续提交流程
    → Reject { reason } → 阻止提交
```

**检查项：**

- [ ] 风控评估是否在订单提交前执行（事前）
- [ ] `RiskDecision::Reject` 是否真正阻止了订单提交
- [ ] 是否存在事中/事后风控补充
- [ ] 行业同步数据是否被风控规则正确消费

**链路 3: QMT Live 真实提交通道**

```
QmtLiveExecutionAdapter (guarded)
  → ensure_bridge_qmt_live_mode() → bridge capability check
    → QmtTaskSubmitService::submit_order() → Windows bridge
      → /api/v1/task/execute
```

**检查项：**

- [ ] capability gate 是否在每次提交前检查（不是一次性缓存）
- [ ] bridge 不可用时是否有明确的错误信息
- [ ] 真实提交失败后是否正确回写 runtime.db
- [ ] adapter 返回的 `OrderStatus::PendingSubmit` 后续状态轮询路径是否完整

#### 5.3.3 状态机一致性

| 状态机 | 定义位置 | 终态 |
|--------|---------|------|
| OrderStatus | src/execution/models.rs | Filled, Canceled, Rejected |
| ExecutionRequestStatus | src/execution/models.rs | Completed, Failed, Canceled |
| StrategyRunStatus | src/execution/models.rs | Success, Failed |
| ApprovalStatus | src/execution/models.rs | Approved, Rejected |

**检查项：**

- [ ] 状态转换是否在代码中有明确路径
- [ ] 是否存在可达但未处理的死状态
- [ ] 状态转换是否被正确持久化到 runtime.db
- [ ] 重启后状态恢复是否正确（reconciliation 路径）
- [ ] `Unknown` 状态的恢复策略是否合理

#### 5.3.4 错误处理路径

**检查项：**

- [ ] 数据库连接失败是否有重试/降级
- [ ] 外部 API 失败是否有超时保护
- [ ] Bridge 不可达是否有明确的用户排障指引
- [ ] 错误信息是否携带足够的上下文（symbol, order_id, status）
- [ ] 是否存在 catch-all error handler 掩盖具体错误
- [ ] `AdapterError` (src/execution/adapter.rs) 的变体是否覆盖所有实际失败场景

### 5.4 代码质量与规范合规性评估 (权重: 15%)

#### 5.4.1 规范合规性

依据 `docs/RUST_CODING_STANDARDS.md` 逐一检查：

| 规范条款 | 检查方法 |
|----------|---------|
| 禁止 unwrap() | 搜索 `\.unwrap\(\)`，按第四章统计口径分桶 |
| 禁止 panic!() | 搜索 `panic!\("` |
| 库代码禁止 println! | 搜索 `println!\("` — 排除 handlers/ 目录 |
| 库代码必须用 tracing | 与上条反向验证 |
| TODO 必须带编号 | 搜索 `TODO` — 检查后续是否有 issue # |
| 禁止吞错误 | 搜索 `let _ = ` — 排除合理 guard 用法 |
| 公共 API 显式类型标注 | 逐文件审查 `pub fn` 返回值 |

#### 5.4.2 公共类型 derive 规则 (按边界分类)

不设全局硬规则。按类型边界分类要求：

| 类型类别 | 要求 | 说明 |
|----------|------|------|
| CLI 参数/输出 DTO | 至少 `Debug`；必要时 `Serialize` | 输出格式化优先 |
| 配置类型 | `Debug + Clone + Serialize + Deserialize` | 配置加载与热重载 |
| 持久化/API 边界 DTO | `Debug + Clone + Serialize + Deserialize` | 除非有明确不需要的原因 |
| 服务句柄/连接池/资源持有者 | 不强制 `Clone`/`Serialize`/`Deserialize` | 这些类型不应被克隆或序列化 |
| 领域 enum (非 DTO) | 至少 `Debug` | 其他 derive 按使用边界判定 |

#### 5.4.3 命名规范

**检查项：**

- [ ] 模块名是否使用 snake_case
- [ ] 结构体/枚举/trait 是否使用 UpperCamelCase
- [ ] 函数/方法是否使用 snake_case
- [ ] 常量是否使用 SCREAMING_SNAKE_CASE
- [ ] 测试函数命名: `test_{功能}_{场景}`

#### 5.4.4 文档注释

**检查项：**

- [ ] 公共模块是否有 `//!` 模块级文档
- [ ] 公共结构体/enum/trait 是否有 `///` 注释
- [ ] 公共函数是否有 `///` 注释
- [ ] 文档示例代码是否可编译（通过 `cargo test --doc`）

### 5.5 测试覆盖与质量评估 (权重: 10%)

#### 5.5.1 测试现状

审核开始时采集 tests/ 目录文件列表，记录实际数量与分类。

**检查项：**

- [ ] 测试文件命名是否清晰对应被测试模块
- [ ] 单元测试是否覆盖公共函数的正向路径
- [ ] 是否有关键错误分支的测试
- [ ] 是否存在依赖外部服务（网络/数据库）的测试没有 skip/ignore 标记
- [ ] 是否存在将 MOCK 测试结果伪装为真实验收的测试表述
- [ ] `cargo test --all-targets` 是否全部通过

#### 5.5.2 测试质量与抽样规则

按风险等级分层抽样：

| 等级 | 相关测试审查密度 | 说明 |
|------|----------------|------|
| P0/P1 模块 | 全量审查 | 执行/策略/风控/CLI/bridge 的关键测试 |
| P2/P3 模块 | 至少 20%，且不少于每模块 2 个 | 若模块测试少于 2 个则全量审查 |
| repo hygiene / script / smoke / gate | 全量审查 | 回归保护、脚本验证、质量门禁测试 |

**单测试质量检查项：**

- [ ] 测试断言是否实质性（不只有空断言或 `assert!(true)`）
- [ ] 是否有边界值测试
- [ ] 是否有并发/竞态测试
- [ ] fixture / fake / mock 数据是否贴近真实契约
- [ ] 是否有回归保护锁定已知 MOCK 边界（参见 `tests/repo_hygiene_test.rs`）

---

## 六、审核工作流程

### 6.1 阶段划分

```
Phase 0: 审核前基线采集           (15 min)
  ├── 记录 git commit 与 dirty 状态
  ├── 读取 GitNexus repo context，确认索引新鲜度
  ├── 如索引 stale，运行 gitnexus analyze
  ├── 创建证据目录: mkdir -p docs/CODE_AUDIT_EVIDENCE/
  ├── 采集模块/commands/handlers/tests/config/docs 范围基线
  └── 写入 docs/CODE_AUDIT_EVIDENCE/baseline.md

Phase 1: 环境准备与门禁             (30 min)
  ├── cargo fmt --check
  ├── cargo clippy --all-targets --all-features
  ├── cargo test --all-targets
  └── cargo build --release (完整 gate，可按环境成本延后到 Phase 3)

Phase 2: 全量自动扫描              (45 min)
  ├── 违规模式全量搜索 + 统计分桶 (第四章)
  ├── 文件大小统计 + 阈值检查
  ├── 公共 API 签名采集
  ├── CLI command/handler 分发完整性检查
  └── 测试分类与外部依赖扫描

Phase 2b: 架构全局扫描             (45 min)
  ├── READ gitnexus://repo/quantix-rust/clusters
  ├── 审查模块依赖拓扑
  ├── 识别违规依赖方向
  ├── 对照 FUNCTION_TREE.md 核对模块清单
  └── 生成架构健康初稿

Phase 3: P0 模块深度审查          (120 min)
  ├── execution/ (执行内核) — 3 adapter + kernel + reconciliation
  ├── strategy/ (策略引擎) — daemon + registry + evaluator
  ├── risk/ (风控引擎) — service + storage + industry
  └── 交叉验证风险集成点

Phase 4: P1 模块深度审查           (90 min)
  ├── bridge/ (跨系统桥接) — client + models + error
  ├── cli/ (命令分发) — commands/*.rs + handlers/*.rs
  ├── monitor/ (实时监控)
  └── monitoring/ (可观测性)

Phase 5: P2-P3 模块分层审查        (90 min)
  ├── 每模块入口文件 + service/provider/adapter + 持久化点
  ├── 按 5.5.2 抽样规则审查测试
  ├── FUNCTION_TREE.md 状态标记验证
  └── 全量扫描结果人工复核

Phase 6: 汇总与报告                (60 min)
  ├── 问题分级与优先级排序
  ├── 风险评估矩阵
  ├── 改进建议实施顺序
  └── 生成最终审核报告与 finding CSV
```

> 耗时标注为估算范围，实际耗时依赖环境、gate 通过率和模块深度。`cargo build --release` 成本较高，可在 Phase 1 执行或按实际环境延后至 Phase 3。

---

## 七、发现项管理与输出

### 7.1 Finding 标准 Schema

每个发现项必须包含以下字段：

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string | 稳定编号，如 `AUDIT-S1-001` |
| `severity` | enum | S0 / S1 / S2 / S3 / S4 |
| `confidence` | enum | `confirmed` / `probable` / `needs-repro` |
| `module` | string | 影响模块 |
| `file:line` | string | 精确证据位置 |
| `evidence` | string | 最小必要代码片段或命令输出摘要 |
| `rule` | string | 违反的规范条款、方法论检查项或设计边界 |
| `impact` | string | 对功能、资金安全、数据完整性、维护性的影响 |
| `reproduction` | string | 如何复现或验证 |
| `recommended_fix` | string | 建议修复方式 |
| `acceptance_criteria` | string | 修复完成的验收条件 |
| `tests_required` | string | 需要新增/更新/运行的测试 |
| `status` | enum | `open` / `accepted` / `fixed` / `deferred` / `wontfix` / `needs-repro` |

### 7.2 发现项生命周期

```
open ──→ accepted ──→ fixed
  │          │
  ├──→ deferred (有意延后，需说明原因)
  ├──→ wontfix  (不修复，需说明技术依据)
  └──→ needs-repro (证据不足，等待复现)
```

| 状态 | 含义 |
|------|------|
| `open` | 已确认，尚未处理 |
| `accepted` | 同意修复，等待排期 |
| `fixed` | 已修复并通过验收 |
| `deferred` | 有意延后，需说明原因和时间线 |
| `wontfix` | 不修复，需说明技术依据 |
| `needs-repro` | 证据不足，等待复现 |

### 7.3 阶段输出

每阶段结束后产出：

1. **发现列表** — 按 severity + confidence 二级排序
2. **证据引用** — 每个 finding 的最小必要代码片段或命令输出
3. **建议操作** — 具体可执行的修复建议

---

## 八、风险分级与判定标准

### 8.1 风险等级定义

| 等级 | 标签 | 定义 | 响应时间 |
|------|------|------|---------|
| **S0** | CRITICAL | 涉及资金安全、未经门控的真实交易、数据完整性破坏 | 立即修复 |
| **S1** | HIGH | 影响核心功能正确性或规范严重偏离 | 本次迭代 |
| **S2** | MEDIUM | 影响代码可维护性或测试可靠性 | 下一迭代 |
| **S3** | LOW | 改善性建议，不影响功能 | 后续登记 |
| **S4** | INFO | 观察项，供参考 | 按需 |

### 8.2 交易相关风险的细分判定

因本系统涉及真实交易路径，S0/S1 需额外细化：

**S0 — 资金安全与数据完整性**:

- 未门控的真实下单路径（绕过 `qmt_live` gate 可直接提交订单）
- 真实交易路径失败后静默 fallback 到 MOCK 或 paper
- 可能导致持仓/订单/成交记录丢失或错写
- unsafe 代码影响执行路径

**S1 — 功能正确性与规范红线**:

- 真实交易错误回写不完整，但不会继续提交错误订单
- 状态机处理遗漏导致核心链路不可用（如 `PendingCancel` 无处理路径）
- 生产路径 panic/unwrap 影响核心命令执行
- MOCK/真实能力在文档或 CLI 帮助文本中混淆，但不改变实际执行路径

### 8.3 判定决策树

```
发现一个问题
├── 涉及未门控真实下单/静默 MOCK fallback/数据丢失？ → S0 CRITICAL
├── 涉及真实交易回写不完整/状态机遗漏/核心路径 unwrap？ → S1 HIGH
├── 涉及 MOCK 文档混淆/非核心路径错误处理不足？ → S2 MEDIUM
├── 涉及命名/注释/非关键规范偏离？ → S3 LOW
└── 涉及架构演进建议/优化方向？ → S4 INFO
```

### 8.4 已知风险基线

审核前已知的技术债务（来自 CLAUDE.md，审核时应以实际测量值为准）：

| 问题 | 来源 | 需在审核中测量 |
|------|------|--------------|
| handlers.rs 曾达 11K+ 行 | CLAUDE.md | 当前已拆分，验证拆分后各文件大小 |
| .unwrap() 调用 | CLAUDE.md 标注 715+ | 按统计口径实际测量 |
| println! in lib modules | CLAUDE.md | 实际搜索确认 |
| 无跟踪 TODO | CLAUDE.md 标注 12+ | 实际搜索确认 |

---

## 九、输出交付物

### 9.1 主报告: `docs/CODE_AUDIT_REPORT.md`

结构：

```markdown
# quantix-rust 代码审核报告

## 1. 执行摘要
- 审核范围、基准 commit、方法论版本
- 整体健康度评分 (5 维度加权)
- Top 10 关键发现

## 2. 架构评估
- 模块依赖拓扑 (实际 vs 预期)
- 架构违规清单
- 执行边界合规状况

## 3. CLI 评估
- 命令树完整性审查
- 功能状态标记一致性 (vs FUNCTION_TREE.md)
- 命名一致性分析

## 4. 业务逻辑评估
- 核心执行链路追踪结果
- 状态机审查 (vs 5.3.1 基准)
- 错误处理分析

## 5. 代码质量评估
- 规范合规率 (按 5.4.1 逐项统计分桶)
- 文件大小合规
- 文档覆盖率

## 6. 测试评估
- 测试覆盖率估算
- 测试质量抽查结果
- MOCK 混淆检测

## 7. 发现项总览
- S0: N 项, S1: N 项, S2: N 项, S3: N 项, S4: N 项

## 8. 改进建议实施顺序
- P0 (立即修复): N 项
- P1 (本次迭代): N 项
- P2-P3 (后续登记项): N 项
```

### 9.2 辅助交付物

```
docs/CODE_AUDIT_EVIDENCE/
├── baseline.md               # 审核前范围基线
├── cargo-gates.md            # fmt/clippy/test/build 结果
├── gitnexus-queries.md       # GitNexus 查询记录
├── pattern-scan-summary.csv  # 违规模式扫描汇总 (含分桶统计)
├── sampled-files.md          # 人工深审的文件清单
└── findings.csv               # 结构化发现项清单 (按 7.1 schema)
```

### 9.3 CSV 列定义

`findings.csv` 以 7.1 中的 finding schema 为列定义，可直接导入 issue tracker。

---

## 十、审核执行最佳实践

### 10.1 效率准则

- **并行操作**：同一阶段内无依赖的读取/搜索操作批量并行
- **子代理分工**：P0 三大模块可分配独立子代理并行审查，主代理负责汇总
- **先广后深**：先跑全量自动扫描找模式问题，再对可疑模块深度读取
- **缓存利用**：读取过的模块信息记录索引，避免重复读取
- **工具选择**：文本搜索用 `rg`；结构查询用 GitNexus；历史回顾用 Graphiti

### 10.2 证据收集原则

- 每个发现必须包含：文件路径 + 行号 + 代码片段 + 规范引用
- CRITICAL 和 HIGH 级别发现必须有双重确认（静态分析 + 代码审查）
- MOCK 相关发现必须注明是 MOCK 路径、真实路径、还是混淆
- 不能复现的发现标注为 `needs-repro`，标注置信度 `probable` 或 `needs-repro`

### 10.3 报告写作原则

- 问题描述要具体：不写"代码质量差"，写"`src/monitor/service.rs:142` 使用了 `unwrap()`"
- 建议要可执行：不写"需要重构"，写"将 3 个 `unwrap()` 替换为 `map_err` + `tracing::warn!`"
- 优先级要有依据：附带判定决策树的推理路径
- 正向总结：不仅列问题，也应标注做得好的模块和模式
- **类型/模块名称必须与实际代码一致**：使用 `PaperExecutionAdapter` 而非 `PaperAdapter`

### 10.4 Graphiti 集成 (对齐 AGENTS.md)

审核开始前：

- 查询 `quantix_rust_review` 获取历史审查结论
- 若涉及设计边界、命名、架构意图，查询 `quantix_rust_main`
- 若涉及已知 bug/root cause，查询 `quantix_rust_debug`
- Graphiti 不作为当前代码事实来源；代码结构和调用关系以 GitNexus/源码为准

审核结束后：

- 将最终 review conclusions 写入 `quantix_rust_review`
- 将 S0/S1 关键发现摘要写入 `quantix_rust_review`
- 如审核过程中形成新的设计决策，另写 `quantix_rust_main`
- 每次 `add_memory` 后必须：
  1. 捕获 `episode_uuid`
  2. 轮询 `get_ingest_status` 直到 `completed`
  3. 若 ingest 失败，标注 "Graphiti backfill required"

---

## 附录 A: 模块依赖关系拓扑

### A.1 预期依赖方向（理想态）

```
                          ┌─────────┐
                          │   CLI   │ (用户交互入口)
                          └────┬────┘
                               │
              ┌────────────────┼────────────────┐
              ▼                ▼                 ▼
        ┌──────────┐   ┌───────────┐    ┌─────────────┐
        │ monitor  │   │ screener  │    │  watchlist  │
        └────┬─────┘   └─────┬─────┘    └──────┬──────┘
             │               │                  │
        ┌────┴───────────────┴──────────────────┴────┐
        │              Service 层                      │
        │  (strategy, risk, market, trade, stop, ...)│
        └──────────────────────┬──────────────────────┘
                               │
        ┌──────────────────────┼──────────────────────┐
        ▼                      ▼                       ▼
  ┌──────────┐         ┌───────────┐           ┌──────────┐
  │ sources  │         │ execution │           │   db     │
  │ (data)   │         │ (adapter) │           │(clients) │
  └────┬─────┘         └─────┬─────┘           └────┬─────┘
       │                     │                      │
       └─────────────────────┼──────────────────────┘
                             │
                    ┌────────┴────────┐
                    ▼                 ▼
              ┌──────────┐    ┌───────────┐
              │  data    │    │  core     │
              │ (models) │    │(config,   │
              └──────────┘    │ error)    │
                              └───────────┘
```

### A.2 当前模块清单

以下清单结构稳定，但**精确文件数量以审核执行时采集的基线为准**：

```
src/
├── account/      多账户管理
├── ai/           AI 决策模块
├── analysis/     回测引擎、技术指标
├── anomaly/      异常检测
├── bridge/       Windows Bridge 客户端
├── cli/          CLI 交互层
│   ├── commands/  命令定义
│   └── handlers/  命令处理
├── core/         核心配置、错误类型
├── data/         数据模型
├── db/           数据库客户端
├── execution/    执行内核 (含 adapter/kernel/reconciliation)
├── factor/       因子管线
├── fundamental/  基本面数据
├── import/       数据导入
├── io/           导入导出
├── market/       市场分析
├── monitor/      实时监控
├── monitoring/   可观测性 (health/metrics/notification)
├── news/         新闻搜索
├── risk/         风险管理
├── screener/     选股器
├── sources/      数据源适配器
├── stop/         止盈止损
├── strategy/     策略引擎 (含 daemon/registry/strategies)
├── sync/         数据同步
├── tasks/        任务调度
├── trade/        模拟交易
├── tui/          TUI 界面
└── watchlist/    自选池
```

---

## 附录 B: 审核用类型速查表

本表中所有名称均从实际代码提取，审核时以此为准，不使用缩写。

### 执行内核 (execution/)

| 文档中称呼 | 实际类型名 | 文件位置 |
|-----------|-----------|---------|
| ExecutionKernel | `ExecutionKernel` | src/execution/kernel.rs |
| RiskDecision | `RiskDecision` (enum: Allow / Reject { reason }) | src/execution/kernel.rs:16 |
| RiskEvaluator trait | `RiskEvaluator` (async fn evaluate → Result\<RiskDecision\>) | src/execution/kernel.rs:22 |
| Paper Adapter | `PaperExecutionAdapter<Store>` | src/execution/paper.rs:12 |
| MockLive Adapter | `MockLiveExecutionAdapter<C>` | src/execution/mock_live.rs:27 |
| QMT Live Adapter | `QmtLiveExecutionAdapter` | src/execution/qmt_live_adapter.rs:49 |
| QMT Bridge Preview | `QmtBridgePreviewAdapter` | src/execution/qmt_bridge.rs:8 |
| ExecutionRequest | `ExecutionRequestRecord` (model struct) | src/execution/models.rs |
| ExecutionRequestStatus | `ExecutionRequestStatus` (enum) | src/execution/models.rs |

### 订单/信号 (execution/models.rs)

| 类型 | 变体 |
|------|------|
| `OrderStatus` | PendingSubmit, Submitted, Accepted, PartiallyFilled, PendingCancel, Filled, Canceled, Rejected, Unknown |
| `OrderSide` | Buy, Sell |
| `SignalStatus` | New, Superseded, Expired |
| `ApprovalStatus` | Pending, Approved, Rejected |
| `StrategyRunStatus` | Running, Success, Failed |
| `ExecutionRequestStatus` | Pending, InProgress, Completed, Failed, Canceled |

### 策略层 (strategy/)

| 文档中称呼 | 实际类型名 | 文件位置 |
|-----------|-----------|---------|
| Strategy trait | `Strategy` trait | src/strategy/trait_def.rs |
| 策略评估器 | `ConfiguredStrategyEvaluator` | src/strategy/registry.rs:15 |
| Signal 类型 | `Signal` enum | src/strategy/trait_def.rs |

---

## 附录 C: 审核自检清单

### 前置检查

- [ ] 基线采集完成，`docs/CODE_AUDIT_EVIDENCE/baseline.md` 已写入
- [ ] GitNexus 索引新鲜（已读取 repo context 确认）
- [ ] `cargo fmt --check` 已执行
- [ ] `cargo clippy --all-targets --all-features` 基线已获取
- [ ] `cargo test --all-targets` 基线已获取
- [ ] `cargo build --release` 已执行或已记录延后原因

### 过程检查

- [ ] P0 模块（execution / strategy / risk）的每条关键执行路径已追踪
- [ ] 全量自动扫描完成（违规模式 + 文件大小 + 公共 API + 命令分发 + 测试分类）
- [ ] 所有违规模式结果已按分桶统计，`production_matches` 已标注
- [ ] 所有 unwrap/panic 调用位置已验证（区分生产代码 vs 测试代码）
- [ ] 所有桥接/bridge 模块的 MOCK 边界已确认（对照 `docs/standards/MOCK_USAGE_POLICY.md`）
- [ ] CLI 全部顶层命令审查完毕
- [ ] FUNCTION_TREE.md 状态标记与代码一致性验证完毕
- [ ] 测试审查按 5.5.2 抽样规则完成
- [ ] 所有类型引用与实际代码定义一致（对照附录 B 速查表）

### 交付检查

- [ ] 所有 CRITICAL (S0) 发现均已双重确认
- [ ] Finding CSV 按 7.1 schema 完整填充
- [ ] 每个 S0/S1 发现均有具体的修复建议和验收条件
- [ ] 报告包含正向总结（做得好的模块和模式）
- [ ] 改进建议按实际可行性排序
- [ ] `docs/CODE_AUDIT_EVIDENCE/` 目录包含全部复现材料
- [ ] Graphiti `quantix_rust_review` 已写入 review conclusions (验证 ingest completed)

---

## 变更记录

| 日期 | 版本 | 变更内容 |
|------|------|---------|
| 2026-05-11 | 1.2-draft | 根据 Codex review 全面修订：范围计数改为动态采集+基线机制；Phase 1 重排（GitNexus 前置）；工具矩阵改为"能力+实现"形式；新增违规模式统计口径与分桶规则；公共类型 derive 改为按边界分类；测试抽样改为按风险等级分层；新增 finding 标准 schema 与生命周期；新增证据目录交付物；风险决策树细化交易门控细则；Graphiti 对齐 AGENTS.md 规则；状态改为候选版 |
| 2026-05-11 | 1.1 | 根据 review 意见修正：CLI 命令树从代码重新提取、状态机定义替换为实际 enum 源码、adapter 名称修正、RiskDecision 类型核实、模块/测试数量纠正、新增附录 B 类型速查表 |
| 2026-05-11 | 1.0 | 初版起草 |
