# 本条线完成任务总结与下一步计划

日期：2026-06-28

当前仓库状态：

- 当前分支：`master`
- 当前 HEAD：`45c0312 docs: add openstock p0.8e graphiti backfill (#312)`
- 当前 FUNCTION_TREE 程序：`project-governance`
- 当前 FUNCTION_TREE gate：`active gates: none`
- 当前治理校验：`governance validation passed`

## 一、总体结论

本条线已经从早期 qmt_live 执行语义治理，推进到 OpenStock 数据消费主线。当前所有已启动的 P0.3 到 P0.8e 相关 FUNCTION_TREE 节点均已闭合，仓库主线无开放 gate。

当前项目主线状态可以概括为：

- qmt_live 实盘就绪线已经完成架构、安全、运行时治理归档，但真实 canary 仍被外部 miniQMT/Windows Bridge 环境阻塞。
- ExecutionCapabilities 能力语义线已经完成 MVP 与只读展示接线，为 paper/mock/qmt_live 三类执行通道提供统一静态语义表达。
- OpenStock 数据消费线已经完成 OpenSpec、inventory、fixture parser、只读 fixture CLI、analysis fixture loop，以及 P0.8e shadow validation 设计门禁。
- 下一步不应继续投入 qmt_live 环境等待线，而应进入 P0.8f：OpenStock 可执行 shadow validation 第一片，继续推进 broker-independent 的真实量化数据闭环。

## 二、本条线已完成任务

### 1. Clippy `.unwrap()` 清理项目

状态：已正式闭合，不再继续清理。

完成内容：

- 按小切片、单文件、LOW-risk 优先策略清理大量生产 `.unwrap()`。
- 剩余高风险节点已纳入技术债备案，不再处理：
  - `src/strategy/test_utils.rs`：测试支撑，豁免。
  - `src/analysis/performance.rs`：CRITICAL。
  - `src/analysis/backtest.rs`：CRITICAL。
  - `src/cli/handlers/strategy_handler.rs`：HIGH，已撤回。
  - `src/cli/handlers/strategy_handler/catalog.rs`：CRITICAL。
  - 其他 HIGH 风险文件若干。
- 后续约定已经固化：本项目不再开展任何 `.unwrap()` 清理工作；存量高风险节点进入技术债管理，需单独专项评估、风险审批与定制测试方案。

FUNCTION_TREE 位置：

- 该条线已作为历史治理工作闭合，不再作为当前 active mainline 推进。
- 当前 active gate 中无 Clippy cleanup 节点。

### 2. qmt_live capability / identity hardening：P0.3

状态：已闭合。

完成内容：

- `P0.3a`：qmt_live capability identity hardening design。
- `P0.3b`：qmt_live capability snapshot seed。
- `P0.3c`：qmt_live identity reconciliation tightening。
- `P0.3d`：qmt_live error taxonomy seed。
- `P0.3e`：ExecutionCapabilities MVP。
- `P0.3f`：ExecutionCapabilities read-only observability。

主要收益：

- 初步建立 qmt_live 能力、身份、错误分类、查询/提交边界的本地语义基础。
- 引入 `ExecutionCapabilities` 静态能力描述，避免继续用运行模式字符串承载通道能力语义。
- 保持最小切片原则：不修改 `OrderStatus`、不改 bridge 协议、不改存储 schema、不做全局响应结构重写。

FUNCTION_TREE 位置：

- `.governance/programs/project-governance/tree.md`
- 节点：
  - `P0.3a` 到 `P0.3f`
- 当前状态：
  - 全部 `[x] closed`

### 3. qmt_live hardening：P0.4

状态：已闭合。

完成内容：

- `P0.4a`：qmt_live hardening design。
- `P0.4b`：qmt_live capability descriptor。
- `P0.4c`：qmt_live error taxonomy local enrichment。
- `P0.4d`：qmt_live gate runtime compatibility check。
- `P0.4e`：qmt_live diagnostics wiring。
- `P0.4f`：qmt_live identity and runtime metadata recovery。
- `P0.4g`：qmt_live reconciliation query refinement。

主要收益：

- 强化 qmt_live 能力描述、错误分类、运行时兼容性检查和诊断输出。
- 补强 identity/reconciliation 查询链路，降低 qmt_live 订单状态追踪的语义歧义。
- 对 HIGH/CRITICAL GitNexus impact 区域保持隔离，不在小切片中扩大改造范围。

FUNCTION_TREE 位置：

- `.governance/programs/project-governance/tree.md`
- 节点：
  - `P0.4a` 到 `P0.4g`
- 当前状态：
  - 全部 `[x] closed`

### 4. qmt_live operational safety：P0.5

状态：已闭合并归档。

完成内容：

- `P0.5a`：qmt_live preflight doctor。
- `P0.5b`：qmt_live canary runbook and evidence artifact。
- `P0.5c`：qmt_live kill switch acceptance。
- `P0.5d`：qmt_live audit evidence closure。
- `P0.5e`：qmt_live manual intervention report。
- `P0.5f`：qmt_live release closure docs。
- `P0.5g`：qmt_live OpenSpec archive。
- 对 Graphiti ingest 异常的 slice 建立本地 backfill 报告。

主要收益：

- qmt_live canary 之前的操作安全机制完成制度化：
  - preflight doctor
  - canary runbook
  - kill switch acceptance
  - audit evidence view
  - manual intervention report
  - release closure docs
- 保持实盘安全边界：不授权实盘提交/撤单、不做 manual-intervention resolution、不修改 bridge 协议或存储 schema。

FUNCTION_TREE 位置：

- `.governance/programs/project-governance/tree.md`
- 节点：
  - `P0.5a` 到 `P0.5g`
- 当前状态：
  - 全部 `[x] closed`

### 5. qmt_live runtime readiness：P0.6

状态：已闭合，结论为 `blocked_by_environment`，后续 maintenance-only。

完成内容：

- `P0.6`：qmt_live runtime readiness OpenSpec。
- `P0.6a`：runtime environment inventory and prerequisite check。
- `P0.6b`：read-only command smoke，记录环境阻塞证据。
- `P0.6c`：redacted runtime evidence package。
- `P0.6d`：failure boundary drill。
- `P0.6e`：runtime readiness decision report。
- `QMT_LIVE_RUNTIME_READINESS_P0_6_CLOSURE_2026-06-25.md` 完成最终归档。

正式结论：

- qmt_live runtime readiness 当前为 `blocked_by_environment`。
- 阻塞原因：
  - 缺少 operator 选定的 miniQMT Windows Bridge runtime。
  - 缺少账户标签。
  - 缺少真实只读 smoke evidence。
- 在 operator 提供隔离可用 runtime 前，禁止启动 qmt_live canary。

后续约束：

- P0.6 不再主动投入开发带宽。
- 后续仅在 operator 提供可用 miniQMT Bridge 测试环境后，重跑 P0.6b 或开窄范围 runtime-smoke 节点。

FUNCTION_TREE 位置：

- `.governance/programs/project-governance/tree.md`
- 节点：
  - `P0.6`
  - `P0.6a`
  - `P0.6b`
  - `P0.6b-backfill`
  - `P0.6c`
  - `P0.6c-backfill`
  - `P0.6d`
  - `P0.6d-backfill`
  - `P0.6e`
- 当前状态：
  - 全部 `[x] closed`

### 6. ExecutionCapabilities semantics continuation：P0.7

状态：已闭合。

完成内容：

- `P0.7a`：ExecutionCapabilities mode semantics bridge。
- `P0.7b`：ExecutionCapabilities checklist mode semantics。
- `P0.7c`：ExecutionCapabilities preflight mode semantics。
- `P0.7d`：ExecutionCapabilities P0.7 documentation sync。
- 对 Graphiti ingest 异常的节点建立本地 backfill 报告。

主要收益：

- qmt_live promotion checklist 和 human-readable preflight report 已能展示通道模式语义。
- 已同步 `risk_notice` 与 `storage_namespace` 等运行语义。
- ExecutionCapabilities 已从底层静态声明推进到只读可观测输出。

FUNCTION_TREE 位置：

- `.governance/programs/project-governance/tree.md`
- 节点：
  - `P0.7a`
  - `P0.7a-backfill`
  - `P0.7b`
  - `P0.7b-backfill`
  - `P0.7c`
  - `P0.7d`
  - `P0.7d-backfill`
- 当前状态：
  - 全部 `[x] closed`

### 7. OpenStock data consumption：P0.8 到 P0.8e

状态：P0.8e 已闭合，当前建议进入 P0.8f。

完成内容：

- `P0.8`：OpenStock data consumption OpenSpec。
  - 将 OpenStock 确立为 qmt_live 环境阻塞后的 broker-independent 数据主线。
  - 明确不做 live OpenStock CI 请求、不写 ClickHouse、不改既有数据源路由、不触碰 qmt_live/miniQMT。
- `P0.8a`：OpenStock data consumption inventory。
  - 梳理 `Kline`、`StockQuote`、`StockInfo`、现有 `tdx_api` / `bridge_tdx` / `eastmoney` / miniQMT manifest 边界、ClickHouse 读写路径和 backtest 消费入口。
- `P0.8b`：OpenStock daily kline fixture parser。
  - 新增 fixture-owned `parse_daily_kline_json`。
  - 将 OpenStock daily-kline fixture 归一化为既有 `Vec<Kline>`。
  - 不改 `Kline` 定义，不替换数据源路由。
- `P0.8c`：OpenStock local fixture validation CLI。
  - 新增只读命令：`quantix data openstock validate-fixture --file <fixture.json>`。
  - 输出记录数、代码、日期范围、`local_fixture` 来源标记。
- `P0.8d`：OpenStock analysis fixture loop。
  - 增加 test-only fixture 到指标链路验证。
  - committed OpenStock daily fixture 经 parser 归一化为 `Vec<Kline>`，提取 close 序列后调用既有 `analysis::sma`，断言 `[None, Some(10.125)]`。
- `P0.8e`：OpenStock shadow validation design gate。
  - 固化 schema mapping、dedup key、rollback 前置条件、dry-run report gates 和 GitNexus impact targets。
  - 明确 shadow validation 与 opt-in persistence 分阶段推进。
  - 明确禁止复用 GitNexus impact 为 HIGH 的 miniQMT `ControlledPersistencePolicy`。
- `P0.8e-backfill`：OpenStock P0.8e Graphiti backfill。
  - 记录 P0.8e Graphiti closeout episode 未能完成 ingest 的本地 fallback。
  - 注意：后续补写的短 episode 也因 Graphiti rate limit 失败，因此当前仍保留 `Graphiti backfill required`。

FUNCTION_TREE 位置：

- `.governance/programs/project-governance/tree.md`
- 节点：
  - `P0.8`
  - `P0.8-backfill`
  - `P0.8a`
  - `P0.8a-backfill`
  - `P0.8b`
  - `P0.8b-backfill`
  - `P0.8c`
  - `P0.8c-backfill`
  - `P0.8c-graphiti-completion-sync`
  - `P0.8d`
  - `P0.8d-backfill`
  - `P0.8e`
  - `P0.8e-backfill`
- 当前状态：
  - 全部 `[x] closed`

## 三、当前 FUNCTION_TREE 位置

当前 FUNCTION_TREE 状态：

```text
programs: project-governance
active gates: 0
active gates: none
governance validation passed
```

当前主线最近闭合点：

```text
45c0312 docs: add openstock p0.8e graphiti backfill (#312)
db44332 docs: add openstock p0.8e shadow validation design (#310)
2cd55ef docs: add openstock p0.8d graphiti backfill (#308)
c756548 test: add openstock analysis fixture loop (#307)
```

当前可视为位于：

```text
project-governance
└── sources/
    └── P0.8 OpenStock data consumption
        ├── P0.8  OpenSpec                       closed
        ├── P0.8a inventory                      closed
        ├── P0.8b fixture parser                 closed
        ├── P0.8c local fixture validation CLI    closed
        ├── P0.8d analysis fixture loop           closed
        ├── P0.8e shadow validation design gate   closed
        └── P0.8f executable shadow validation    proposed next
```

qmt_live 相关位置：

```text
project-governance
└── qmt_live / execution
    ├── P0.3 capability / identity hardening      closed
    ├── P0.4 qmt_live hardening                   closed
    ├── P0.5 operational safety                   closed / archived
    ├── P0.6 runtime readiness                    closed / blocked_by_environment / maintenance-only
    └── P0.7 ExecutionCapabilities semantics      closed
```

## 四、当前风险与边界

### qmt_live 风险边界

- 当前 qmt_live 不能进入 canary。
- 原因不是本地代码门禁失败，而是外部 runtime 证据不足。
- 必须等待 operator 提供隔离可用的 miniQMT Windows Bridge 测试环境、账户标签和只读 smoke evidence。
- 在此之前不应继续扩展 qmt_live runtime readiness 线。

### OpenStock 风险边界

- 现阶段 OpenStock 已完成 fixture/local validation/analysis loop，但尚未完成可执行 shadow validation。
- P0.8e 只完成设计 gate，没有写生产代码。
- P0.8f 必须继续保持只读、可回滚、无生产路由替换。

### Graphiti 风险边界

- 多个节点出现 Graphiti ingest processing、jsondecodeerror 或 rate_limit。
- 当前策略是：
  - Graphiti 可用时继续尝试写入。
  - ingest 未达到 `completed` 时，不宣称 Graphiti 闭合。
  - 用仓库内 Markdown backfill 报告保存等价结论。
- 当前 P0.8e 仍保留：`Graphiti backfill required`。

## 五、下一步任务计划

### P0.8f：OpenStock executable shadow validation 第一片

优先级：最高。

目标：

- 将 P0.8e 设计门禁推进为第一条可执行 shadow validation。
- 继续只读验证 OpenStock fixture/source 输出与项目 canonical `Kline` / 分析链路的兼容性。
- 产出 dry-run shadow validation report，不写入生产 ClickHouse。

建议边界：

- 从 `master` 新建独立分支。
- 新建 FUNCTION_TREE 节点：`P0.8f: OpenStock executable shadow validation`。
- 先跑 Graphiti read；若 Graphiti 仍限流或失败，则记录本地 fallback。
- 先跑 GitNexus impact。
- 采用 TDD：
  - 先写 P0.8f shadow validation contract test。
  - RED 后再实现最小只读逻辑。
- 只允许读取 committed fixture 或显式本地输入。
- 禁止 live OpenStock CI 请求。
- 禁止写 ClickHouse。
- 禁止替换生产数据源路由。
- 禁止触碰 qmt_live、miniQMT、ExecutionAdapter、OrderStatus。
- 禁止恢复 `.unwrap()` 清理。

建议验收：

- 新增或扩展只读 shadow validation 输出：
  - 输入记录数。
  - symbol/code。
  - 日期范围。
  - canonical `Kline` 映射成功数。
  - dedup key 预览。
  - schema mismatch / invalid row 的 fail-closed 报告。
  - `dry_run: true` 明确标记。
- 测试覆盖：
  - 有效 fixture。
  - 重复记录。
  - 缺字段。
  - 日期格式错误。
  - 价格字段不合法。
  - 非 daily period。
  - 混合 code fail-closed。
- 门禁：
  - `cargo fmt --check`
  - P0.8f focused tests
  - OpenStock 相关测试
  - `cargo clippy -- -D warnings`
  - `cargo test`
  - `git diff --check`
  - FUNCTION_TREE gate/validate
  - GitNexus detect_changes

### P0.8g：OpenStock shadow persistence opt-in 设计或实现

优先级：P0.8f 通过后再启动。

启动条件：

- P0.8f 可执行 shadow validation 已合并。
- dry-run report 能稳定描述将写入的数据、dedup key 和冲突行为。
- GitNexus impact 确认为 LOW/MEDIUM 且边界清晰。

建议方向：

- 先做 opt-in persistence 设计门禁，必要时再进入实现。
- 明确 shadow namespace / table / manifest，不污染生产数据。
- 明确 rollback 机制。
- 明确写入前后 row count 校验。
- 明确不替换现有生产数据源路由。

### P0.8h：OpenStock 到 analysis/backtest 的更宽链路验证

优先级：P0.8g 之后。

目标：

- 从更真实的 OpenStock 数据样本验证：
  - parser
  - canonical `Kline`
  - analysis indicators
  - backtest input path
  - dry-run report
- 仍优先保持本地 fixture 或可审计 artifact 驱动，避免 CI 依赖外部网络。

### qmt_live runtime readiness 重启条件

优先级：低，维护态。

只有满足以下条件才重新启动：

- operator 提供隔离可用 miniQMT Windows Bridge runtime。
- operator 提供账户标签。
- 可以执行只读 smoke：
  - qmt status
  - qmt preview
  - qmt query
- 可以保存脱敏 evidence package。

重启方式：

- 不重做 P0.6 全套规划。
- 优先复用 P0.6 框架，重跑 P0.6b。
- 若范围更窄，可新建 `P0.7 runtime-smoke` 或后续等价节点。

## 六、建议立即执行顺序

1. 新建 `P0.8f` FUNCTION_TREE 节点。
2. Graphiti read，若失败则记录 fallback。
3. GitNexus impact：
   - OpenStock parser/validator 相关符号。
   - CLI/report 输出相关符号。
   - 任何可能接近 ClickHouse 写路径的符号。
4. 编写 P0.8f contract test，先跑 RED。
5. 实现最小 read-only shadow validation。
6. 跑 focused tests。
7. 跑全门禁。
8. 更新 README / CHANGELOG / FUNCTION_TREE / OpenSpec tasks。
9. GitNexus detect_changes。
10. PR、CI、merge、Graphiti memory 或本地 backfill。

## 七、当前不建议做的事

- 不继续 `.unwrap()` cleanup。
- 不继续 qmt_live runtime readiness 等环境。
- 不启动 qmt_live canary。
- 不做 live broker submit/cancel。
- 不修改 `ExecutionAdapter` / `OrderStatus`。
- 不替换生产数据源路由。
- 不写生产 ClickHouse。
- 不做 OpenStock live network CI。
- 不把 miniQMT `ControlledPersistencePolicy` 复用到 OpenStock，因为此前 GitNexus impact 已标记为 HIGH。

## 八、参考文件

- `FUNCTION_TREE.md`
- `.governance/programs/project-governance/tree.md`
- `CHANGELOG.md`
- `README.md`
- `openspec/changes/openstock-data-consumption-p0-8/tasks.md`
- `docs/reports/OPENSTOCK_DATA_CONSUMPTION_P0_8_OPENSPEC_2026-06-26.md`
- `docs/reports/OPENSTOCK_DATA_CONSUMPTION_P0_8A_INVENTORY_2026-06-26.md`
- `docs/reports/OPENSTOCK_DATA_CONSUMPTION_P0_8E_SHADOW_VALIDATION_DESIGN_2026-06-28.md`
- `docs/reports/OPENSTOCK_DATA_CONSUMPTION_P0_8E_GRAPHITI_BACKFILL_2026-06-28.md`
- `docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6E_2026-06-25.md`
- `docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6_CLOSURE_2026-06-25.md`
