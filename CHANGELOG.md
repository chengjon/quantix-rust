# Changelog

All notable changes to this project are documented here.

> 状态源说明：本文记录历史变更，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 [`FUNCTION_TREE.md`](FUNCTION_TREE.md) 的状态注册表行为准。

## 2026-06-02

### Fixed
- **data export 格式验证失败关闭** (`src/cli/handlers/data_handler.rs`, `tests/data_export_cli_validation_test.rs`, `docs/CLI_COMMAND_MANUAL.html`, `tests/repo_hygiene_test.rs`, `README.md`, `FUNCTION_TREE.md`)
  - `quantix data export --format <未知格式>` 现在会在打印导出信息、读取 ClickHouse 或创建输出目录之前返回显式 `Unsupported`
  - 错误包含 `data export format 不支持` 和支持格式 `csv, parquet`，不再因为无数据而成功退出，也不再先输出导出占位信息

## 2026-06-01

### Added
- **import from-excel Excel watchlist parser** (`Cargo.toml`, `Cargo.lock`, `src/import/excel_parser.rs`, `src/import/types.rs`, `src/import/mod.rs`, `src/cli/handlers/import.rs`, `docs/CLI_COMMAND_MANUAL.html`, `tests/repo_hygiene_test.rs`, `README.md`, `FUNCTION_TREE.md`)
  - `quantix import from-excel --file <FILE> [--sheet <SHEET>]` 现在通过 `calamine` 读取 Excel workbook，并解析首个或指定 worksheet 中的股票代码/名称行
  - CLI 输出与 `from-csv` 保持一致，显示解析数量、总行数、跳过行数和逐条代码/名称/置信度；复杂 Excel schema、公式语义和持久化导入闭环仍不属于当前能力

### Fixed
- **import from-image Vision provider 失败关闭** (`src/import/image_extractor.rs`, `src/import/mod.rs`, `src/cli/handlers/import.rs`, `tests/import_image_cli_validation_test.rs`, `docs/CLI_COMMAND_MANUAL.html`, `tests/repo_hygiene_test.rs`, `README.md`, `FUNCTION_TREE.md`)
  - `quantix import from-image --model deepseek|openai` 现在会按所选 Vision provider 读取对应 API key/base URL/model 环境变量，缺少所选 provider 的 API key 时返回 `Unsupported`，错误包含 `Vision provider 尚未配置`
  - `--model openai` 不再静默复用 DeepSeek 的 base URL/model 配置；Vision API 请求错误和响应解析错误也不再被包装成 stdout 错误后成功退出
- **algo plan 参数验证失败关闭** (`src/cli/handlers/algo.rs`, `src/execution/algo/context.rs`, `tests/algo_cli_validation_test.rs`, `README.md`, `FUNCTION_TREE.md`)
  - `quantix algo plan` 现在会在输出切片预览前复用 `AlgoParams::validate`，拒绝 `side` 非 `buy`/`sell` 等无效参数，不再生成带非法方向的 JSON/table 预览
  - `AlgoParams::validate` 补充 `slice_count=0` 和 `interval_seconds=0` 校验，避免 TWAP/VWAP create/plan 进入除零或零间隔切片计划
  - `--output` 现在只接受 `table` 或 `json`，未知格式会失败关闭，不再静默回退到 table 预览
- **AI 未配置/未接线 provider 失败关闭** (`src/cli/handlers/ai.rs`, `tests/ai_cli_validation_test.rs`, `README.md`, `FUNCTION_TREE.md`, `docs/CLI_COMMAND_MANUAL.html`)
  - `quantix ai analyze` / `decide` / `ask` / `market` 现在只会选择已接线的 DeepSeek、OpenAI 或 Ollama adapter；未配置任一已接线 provider 时会返回 `Unsupported`，错误包含 `AI provider 尚未配置`，不再成功输出“未配置 LLM”提示
  - 如果环境里只配置 Gemini/Anthropic 等未接线 provider，也会返回 `Unsupported`，不再成功退出或静默回退到 Ollama；`ai config` 继续作为配置状态查看入口
- **notify check/test 缺渠道配置失败关闭** (`src/cli/handlers/notify.rs`, `tests/notify_cli_validation_test.rs`, `docs/CLI_COMMAND_MANUAL.html`, `tests/repo_hygiene_test.rs`, `README.md`, `FUNCTION_TREE.md`)
  - `quantix notify check --channel <外部渠道>` 现在会在必需环境变量缺失时返回 `Unsupported`，错误包含 `notify channel 尚未配置` 和所需变量名，不再把“环境变量未配置”作为 stdout 提示后成功退出
  - `quantix notify test --channel <外部渠道>` 现在会先校验该单一渠道配置并将发送范围收窄到该渠道；缺少必需环境变量时同样返回 `Unsupported` 且不输出测试成功占位内容
  - `quantix notify test --channel all` 继续按 `NotificationConfig::from_env()` 聚合渠道发送；`notify list` 继续作为静态渠道名称状态视图
- **sentiment 空 provider 输出模板失败关闭** (`src/cli/handlers/sentiment.rs`, `tests/sentiment_cli_validation_test.rs`, `README.md`, `FUNCTION_TREE.md`, `docs/CLI_COMMAND_MANUAL.html`)
  - `quantix sentiment show` / `history` / `mentions` 现在会在真实 `SentimentProvider` 接线前返回 `Unsupported`，不再成功输出中性情绪、空历史或空提及模板
- **news 未配置 provider 失败关闭** (`src/cli/handlers/news.rs`, `tests/news_cli_validation_test.rs`, `README.md`, `FUNCTION_TREE.md`, `docs/CLI_COMMAND_MANUAL.html`)
  - `quantix news search` / `code` / `trend` 现在会在没有配置 Tavily、SerpAPI 或博查 API key 时返回 `Unsupported`，不再成功输出“未配置 API”提示；`news providers` 仍用于查看 provider 配置状态
- **EastMoney 资金流向隐藏占位数据失败关闭** (`src/sources/eastmoney.rs`, `FUNCTION_TREE.md`)
  - `EastMoneySource::parse_money_flow` 不再为未映射响应返回空代码、当日日期和全 0 金额的 `MoneyFlowData`
  - 在真实资金流向字段映射接线前，该路径会返回显式 `QuantixError::Unsupported`，与 `fundamental capital-flow` CLI 的 fail-closed 边界保持一致
- **import from-excel 失败关闭命令壳** (`src/cli/commands/info.rs`, `src/cli/handlers/import.rs`, `src/cli/tests/import.rs`, `docs/CLI_COMMAND_MANUAL.html`, `tests/repo_hygiene_test.rs`, `README.md`, `FUNCTION_TREE.md`)
- 该阶段先将 `quantix import from-excel` 暴露为显式 CLI 子命令，并以 fail-closed 行为记录当时的能力边界
- CLI HTML 手册、README 与 FUNCTION_TREE 同步从“无 CLI 入口”改为“命令壳已暴露但 fail-closed”，并新增命令解析、handler 回归和手册卫生测试
- 同日后续变更已接入 watchlist 级 Excel parser；当前可用边界以本日期 `Added` 条目和 `FUNCTION_TREE.md` 为准
- **fundamental capital-flow 失败关闭命令壳** (`src/cli/commands/info.rs`, `src/cli/handlers/fundamental.rs`, `docs/CLI_COMMAND_MANUAL.html`, `tests/repo_hygiene_test.rs`, `README.md`, `FUNCTION_TREE.md`)
  - `quantix fundamental capital-flow` 现在作为显式 CLI 子命令暴露，并在真实资金流向数据源接线前返回 `QuantixError::Unsupported`
  - CLI HTML 手册、README 与 FUNCTION_TREE 同步从“无 CLI 入口”改为“命令壳已暴露但 fail-closed”，并新增回归测试和手册卫生测试
- **strategy signal list 过滤参数接线** (`src/cli/handlers/strategy_handler.rs`, `src/cli/handlers/strategy_handler/requests/signals.rs`, `docs/CLI_COMMAND_MANUAL.html`, `tests/repo_hygiene_test.rs`, `README.md`, `FUNCTION_TREE.md`)
  - `quantix strategy signal list` 现在会实际使用已解析的 `--strategy-instance`、`--strategy`、`--code` 和 `--limit`，与原有 `--approval-status` / `--signal-status` 一起过滤已落库 signal
  - CLI HTML 手册、README 与 FUNCTION_TREE 同步删除“参数尚未传入 handler”的陈旧边界说明，并新增回归测试和手册卫生测试
- **AI 命令运行时边界提示** (`src/cli/handlers/ai.rs`, `docs/CLI_COMMAND_MANUAL.html`, `tests/repo_hygiene_test.rs`, `README.md`, `FUNCTION_TREE.md`)
  - `quantix ai analyze` / `decide` / `ask` / `market` 在执行时显式打印模拟价格/指标、模拟技术面分析、问答参数或固定 prompt 边界，避免用户把 LLM 接线验证误读为实时投研或实仓决策能力
  - CLI HTML 手册、README 与 FUNCTION_TREE 同步记录该边界，并新增仓库卫生测试防止手册遗漏运行时提示说明
- **AI 配置测试运行时标题同步** (`src/cli/handlers/ai.rs`, `docs/CLI_COMMAND_MANUAL.html`, `README.md`, `tests/repo_hygiene_test.rs`, `FUNCTION_TREE.md`)
  - `quantix ai config --test` 的运行时标题从“测试 LLM 连通性”改为“检查 LLM 配置状态”，provider 行前缀同步为“检查 ...”
  - CLI HTML 手册、README 与 FUNCTION_TREE 同步说明该入口只是配置状态检查，不发起真实 API 连通性请求，并补齐单元测试与手册卫生测试
- **AI 配置测试状态避免误报可用** (`src/cli/handlers/ai.rs`, `src/cli/commands/info.rs`, `docs/CLI_COMMAND_MANUAL.html`, `tests/repo_hygiene_test.rs`)
  - `quantix ai config --test` 不再把未实际联网探测的 provider 打印成“✅ 可用”，改为明确输出“已配置（未发起真实连通性测试）”
  - CLI help 与 HTML 手册同步说明该命令是配置状态检查，不是真实 API 连通性验收，并补齐单元测试与手册卫生测试
- **fundamental dividend CLI 手册边界同步** (`docs/CLI_COMMAND_MANUAL.html`, `tests/repo_hygiene_test.rs`, `FUNCTION_TREE.md`)
  - 修正 `quantix fundamental` 总览中仍称 `dividend` 为占位输出的陈旧文案，统一写明命令壳存在但真实分红数据源未接线时会返回显式 `Unsupported`
  - 新增仓库卫生测试，防止 CLI HTML 手册重新把已 fail-closed 的分红命令写回占位成功输出状态
- **data export CLI 手册状态同步** (`docs/CLI_COMMAND_MANUAL.html`, `tests/repo_hygiene_test.rs`, `FUNCTION_TREE.md`)
  - 修正 `quantix data export` 手册中仍称 Parquet 为占位实现的陈旧文案，改为反映当前 CSV/Parquet 分支都会调用导出器写出实际文件
  - 新增仓库卫生测试，防止 CLI HTML 手册重新把已接线的 Parquet 导出写回占位状态

## 2026-05-31

### Added
- **新闻热点趋势搜索接线** (`src/cli/handlers/news.rs`, `FUNCTION_TREE.md`, `docs/CLI_COMMAND_MANUAL.html`)
  - `news trend` 复用现有 Tavily/SerpAPI/Bocha provider 聚合搜索路径，默认查询市场热点新闻，传入 `--code`/`--date` 时构造股票热点查询
  - 未配置新闻 provider 时继续输出显式配置提示；当前边界仍是热点新闻搜索，不宣称为完整趋势量化模型
- **TUI 菜单首屏接线** (`src/tui/app.rs`, `src/cli/handlers/app_shell.rs`, `src/tui/mod.rs`, `FUNCTION_TREE.md`, `docs/CLI_COMMAND_MANUAL.html`, `docs/USER_MANUAL.md`)
  - `menu --tui` 在启用 `tui` feature 的构建中进入 ratatui 首屏菜单，支持上下选择、Enter 分发、q/Esc 退出，并复用现有简单菜单 handler
  - 默认构建保留清晰的 feature-gating 提示，避免把未启用可选依赖的路径误报为完整 TUI
  - 同步关闭审计项 `AUDIT-S3-009`，并更新当前功能状态注册表与用户文档
- **GitNexus MCP 日常工作流建议** (`docs/guides/GITNEXUS_MCP_DAILY_WORKFLOW_RECOMMENDATIONS.md`, `README.md`, `FUNCTION_TREE.md`)
  - 新增 GitNexus MCP 日常使用建议，沉淀显式 repo 参数、索引新鲜度、impact/detect gate、rename/cypher 使用、HIGH/CRITICAL 风险处理和项目边界提醒
  - README 增加文档入口，并继续声明 `FUNCTION_TREE.md` 是功能状态、证据和边界的唯一注册表
  - `FUNCTION_TREE.md` 的可编辑 project-notes 区块记录本轮文档同步，不改写 generated 状态注册区

### Fixed
- **交互菜单历史回测入口接线** (`src/cli/handlers/app_shell.rs`, `FUNCTION_TREE.md`, `docs/CLI_COMMAND_MANUAL.html`)
  - `quantix menu` / `menu --tui` 选择“查看历史回测”时不再打印“功能开发中”后成功返回，而是复用现有 `backtest list` 报告列表路径
  - 补齐菜单动作映射测试，锁定该菜单项继续委托到已实现的回测报告列表命令
- **fundamental dividend 失败关闭** (`src/cli/handlers/fundamental.rs`, `FUNCTION_TREE.md`, `docs/CLI_COMMAND_MANUAL.html`)
  - `quantix fundamental dividend` 不再打印“功能开发中”后以成功状态退出；在真实分红数据源接线前改为返回 `QuantixError::Unsupported`
  - 补齐回归测试，锁定分红命令壳必须 fail-closed，避免把未实现能力误报为可用查询
- **功能状态文档残留旧状态同步** (`README.md`, `FUNCTION_TREE.md`, `docs/CLI_COMMAND_MANUAL.html`)
  - 修正 `FUNCTION_TREE.md` 中 TUI、data export、news search/code 与当前实现不一致的残留旧文案
  - README 推进顺序与 CLI HTML 手册同步反映当前 data CSV/Parquet 导出和 provider-backed news search 状态
- **流式批处理进度追踪** (`src/io/batch.rs`)
  - `BatchProcessor::stream_process` 在未知总量的流式输入下维护已观察到的 `total_batches/current_batch`，避免首批更新时除零 panic
  - 启用进度显示时使用 spinner 展示已处理批次和记录数，并补齐流式 chunk 完成回归测试
- **screener preset 输入边界硬化** (`src/screener/parser.rs`, `src/screener/evaluator.rs`, `tests/screener_parser_test.rs`, `tests/screener_evaluator_test.rs`)
  - preset 参数解析拒绝零周期/窗口、非有限阈值、重复 key 和空参数段，避免静默覆盖或接受畸形输入
  - `volume_ratio_gte` 零窗口、RSI lookback 溢出和手工构造非法 invocation 都返回显式错误，而不是除零、回绕或 panic
  - README 的 `screener` 边界说明同步补充严格参数解析约束
- **EMA benchmark 边界补齐** (`src/analysis/indicators_benches.rs`)
  - benchmark 版 EMA 在 period 大于数据长度时返回空结果，避免巨大 period 下的无效 Decimal 分母和 panic

## 2026-05-21

### Changed
- **miniQMT controlled evidence 边界继续收敛** (`FUNCTION_TREE.md`, `README.md`, `src/miniqmt_market.rs`, `src/cli/handlers/import.rs`, `tests/miniqmt_market_import_handler_test.rs`, `docs/superpowers/specs/2026-05-18-miniqmt-controlled-evidence-alignment-spec.md`)
  - `quantix import market-manifest` 继续保持 dry-run-only 主线：支持本地 artifact hash 校验、Parquet metadata `computed_row_count`、本地 reference artifact 比较、source-of-truth 汇总 JSON 比较、direct ClickHouse read-only 比较和 `quantix_regression` evidence 输出，但仍不写入 ClickHouse、不接管 miniQMT registry
  - 新增 `--comparison-source-of-truth-summary`，用于读取外部 source-of-truth 只读汇总文件，校验 dataset identity 并将 row-count/sample comparison 写入 report/evidence
  - 新增 `--comparison-clickhouse-*` opt-in 只读比较路径，用 ClickHouse HTTP `SELECT` 查询 row-count/sample，并将 `direct_clickhouse_read_only_*` comparison 写入 report/evidence
  - `FUNCTION_TREE.md` 继续作为唯一功能状态注册表，明确区分已实现、已设计/待实现与非目标边界
  - `CHANGELOG.md` 只记录历史变更，不与 `FUNCTION_TREE.md` 争夺当前状态真相

## 2026-05-14

### Changed
- **功能真相源收敛为单一注册表** (`FUNCTION_TREE.md`, `README.md`, `docs/QMT_LIVE_TRADING_SETUP.md`, `tests/repo_hygiene_test.rs`)
  - 删除根目录旧规划文件，避免与 `FUNCTION_TREE.md` 形成并行功能真相源
  - README 与 QMT 指南改为只指向 `FUNCTION_TREE.md`
  - 仓库卫生测试锁定 `FUNCTION_TREE.md` 的功能节点必须显式标状态、证据和边界

## 2026-05-03

### Changed
- **规划文档过渡收敛**（已由 2026-05-14 的单一功能注册表决策取代）
  - 删除历史开发规划与规划评审文档
  - README 当时收敛到更少入口；当前功能状态入口已进一步收敛为 `FUNCTION_TREE.md`
  - README 的建议推进顺序同步到新的交易主线稳态化优先级
  - 仓库卫生测试改为锁定旧规划文档已删除

## 2026-05-02

### Changed
- **功能树文档统一为单一 canonical 文件** (`FUNCTION_TREE.md`, `README.md`, `tests/repo_hygiene_test.rs`)
  - 根目录 `FUNCTION_TREE.md` 升级为当前主线唯一功能树与能力边界文档
  - 历史功能清单文档停止单独维护并从主线移除
  - 活跃入口与卫生校验统一切换到 `FUNCTION_TREE.md`
  - 该统一已通过 PR #62 并入 `master`，形成当前主线基线 `origin/master@562fe84`

### Fixed
- **`qmt_live` cancel 路由收口并并入本地主线** (`src/execution/qmt_task_submit_service.rs`, `src/execution/qmt_live_adapter.rs`, `tests/qmt_task_contract_test.rs`, `tests/qmt_live_adapter_test.rs`)
  - `QmtTaskSubmitService` 新增 `resolve_external_order_id_for_cancel(task_id)`，复用现有 task-result 查询路径解析 broker order identity
  - `QmtLiveExecutionAdapter::cancel_order` 先执行 `ensure_bridge_qmt_live_mode`，再将 `task_id -> external_order_id` 解析后调用兼容 `qmt_cancel_order`
  - 补齐 pending、无 `external_order_id`、preview-only gate、compat cancel failure 等聚焦回归覆盖

## 2026-05-01

### Added
- **miniQMT task-contract 提交服务与聚焦回归覆盖** (`src/execution/qmt_task_submit_service.rs`, `tests/qmt_task_contract_test.rs`, `tests/qmt_live_adapter_test.rs`)
  - 新增 `QmtTaskSubmitService`，统一封装 `/api/v1/task/execute` 回执、`/api/v1/task/result/{task_id}` 查询/轮询与 identity 校验
  - 新增 `qmt_live_adapter_test` / `qmt_task_contract_test`，锁定 `PendingSubmit` receipt、pending `result: null`、ack/reject 映射与轮询超时语义

### Changed
- **Bridge runtime / client / qmt_live 语义对齐 miniQMT v1 合同** (`src/core/runtime.rs`, `src/bridge/client.rs`, `src/bridge/models.rs`, `src/execution/qmt_live_adapter.rs`)
  - `BridgeRuntimeSettings` 增加 bearer token、contract version、poll interval、poll timeout 等运行时配置加载
  - `BridgeHttpClient` 与 bridge models 扩展 `/api/v1/task/execute`、`/api/v1/task/result` 合同
  - `QmtLiveExecutionAdapter` 从旧的 broker-submit 语义切到 task receipt/result 语义：
    - `submit_order` 返回 `PendingSubmit` task receipt
    - `query_order` 映射 pending / acknowledgement / reject / execution
    - `cancel_order` 继续保留兼容取消端点

### Fixed
- **手动 `qmt_live` gate 分类诊断补齐** (`src/execution/request_diagnostics.rs`, `src/cli/handlers/execution_handler.rs`, `src/cli/handlers/tests/strategy_execution.rs`)
  - 手动 `qmt_live` 路径在 capability / mode 阻塞时输出结构化 `execution_diagnostics`
  - 统一补齐 gate category 与 detail payload，避免 request detail/list 丢失真实阻塞原因

## 2026-04-30

### Added
- **execution request 结构化执行诊断主线化** (`src/execution/request_diagnostics.rs`, `src/execution/daemon.rs`, `src/cli/handlers/execution_handler.rs`, `tests/execution_daemon_test.rs`, `tests/repo_hygiene_test.rs`)
  - 执行 daemon 成功/失败路径统一写出 `execution_diagnostics`
  - request detail/list CLI 展示结构化诊断负载，便于回看 gate、bridge 和 runtime 失败原因

## 2026-04-29

### Fixed
- **策略运行 live 路由收口** (`src/cli/handlers/strategy_handler/catalog.rs`, `src/cli/commands/strategy.rs`)
  - `strategy run --mode live` 不再走通用执行链路，并明确引导到 `qmt_live request + execution bridge`
  - 泛化 `live` 不再静默落回 mock 语义，真实提交仍保持受保护的 `qmt_live` 单路径边界

### Added
- **CLI mock policy 边界回归测试** (`src/cli/tests/strategy.rs`, `src/cli/tests/execution.rs`, `src/cli/tests/account.rs`, `src/cli/tests/mod.rs`)
  - 新增 strategy / execution / account 三处 CLI 帮助文本与解析边界测试
  - 锁定 `mock_live` / `live` / `qmt_live` 的当前兼容、拒绝与提示文案行为

### Changed
- **文档与卫生契约同步到主线收口事实** (`docs/CLI_COMMAND_MANUAL.html`, `tests/repo_hygiene_test.rs`, 历史规划文件)
  - 同步 `strategy run` 与相关 execution wording 到当前 mock policy 边界
  - 将这轮执行边界收口作为后续 P0.2 / P0.3 的新主线基线

## 2026-04-26

### Added
- **MOCK 数据顶层规范与事实对齐** (`docs/standards/MOCK_USAGE_POLICY.md`, `README.md`, `docs/USER_MANUAL.md`)
  - 新增项目级 `MOCK_USAGE_POLICY`，明确 `anomaly run --mock` 与 `strategy run --mode mock_live` 都属于显式 mock/runtime mock 路径
  - 明确 `mock_live` 是仿真执行与联调加固路径，不等于真实券商实盘
  - 明确真实提交单路径是受保护的 `qmt_live`
  - 明确泛化 `target_mode=live` 仍未实现，且真实路径不得静默回退到 mock
- **文档事实收敛**（历史功能清单文档, `docs/GAP_ANALYSIS.md`, 历史规划文档, `docs/QMT_LIVE_TRADING_SETUP.md`）
  - 同步更新当前能力说明、架构边界、规划判断与操作手册，消除 `mock_live` / `live` / `qmt_live` 表述漂移
  - 为部分历史设计/计划文档补充“历史结论不代表当前实现状态”的上下文说明

### Changed
- **CLI 手册纳入文档基线** (`docs/CLI_COMMAND_MANUAL.html`)
  - 将生成的 CLI 手册纳入仓库基线，避免 clean checkout 下文档卫生测试缺文件

### Added
- **仓库卫生回归保护** (`tests/repo_hygiene_test.rs`)
  - 新增针对 mock policy、README/USER_MANUAL 边界表述、当前能力摘要与相关说明文档的回归校验
  - 锁定 `qmt-preview`、`mock_live`、`qmt_live` 的当前语义，后续文档漂移将直接触发测试失败

## 2026-03-28

### Fixed
- **测试编译错误修复**
  - 修复 Rust 2024 edition `FromStr` prelude 移除导致的编译错误 (`src/ai/types.rs`)
  - 补充 `OrderQueryResponse` 新增的 `rejection_reason` 字段 (`tests/execution_kernel_test.rs`)
  - 补充 `MockLiveFaultInjection` 新增字段 (`delay_seconds`, `rejection_reason`, `timeout_seconds`)
  - 修复 `test_fuzzy` 断言错误 (ExactName 为正确匹配方式)

### Added
- **基本面分析模块连线** (`src/fundamental/eastmoney.rs`, `src/cli/handlers.rs`)
  - `EastMoneyFundamentalProvider` 实现 `FundamentalProvider` trait
  - CLI handlers 连线 valuation/earnings/institution/dragon-tiger 数据展示
- **舆情分析模块连线** (`src/market/sentiment/aggregator.rs`, `src/cli/handlers.rs`)
  - `SentimentAggregator` 新增 `get_mentions()` 和 `get_history()` 方法
  - CLI handlers 连线 sentiment show/history/mentions 格式化输出
- **智能导入模块** (`src/import/code_resolver.rs`)
  - `CodeResolver` 股票代码/名称/拼音解析
- **EastMoney API 响应解析** (`src/fundamental/`)
  - `ValuationFetcher`: 解析 push2 API 估值数据 (PE/PB/市值/ROE/EPS)，含单位转换
  - `EarningsFetcher`: 解析财报数据 (营收/净利润/毛利率)，含单位转换
  - `InstitutionFetcher`: 解析机构持仓 API，含机构类型映射和变动方向判断
  - `DragonTigerFetcher`: 解析龙虎榜 API，含 PascalCase 反序列化和日期解析
  - 新增 14 个单元测试

### Fixed
- **deprecated chrono API**: `from_hms` → `from_hms_opt`, `from_timestamp_opt` → `DateTime::from_timestamp`
- **unused imports**: 通过 `cargo fix` 自动修复 60+ 处未使用导入 (41 文件)

## 2026-03-27 (续3)

### Added

- **P0.2 执行请求生命周期增强** (`src/cli/mod.rs`, `src/cli/handlers.rs`)
  - `strategy request show --request-id <ID>` - 查看单个请求详情
  - `strategy request list --stats` - 显示请求统计摘要
  - `strategy request list --status <STATUS>` - 按状态过滤
  - `strategy request list --target-mode <MODE>` - 按执行模式过滤
  - `strategy request list --target-account <ACCOUNT>` - 按目标账户过滤
  - `--verbose` 标志用于详细输出
- **AI 决策模块** (`src/ai/`) [Phase 2]
  - `LLMAdapter` - OpenAI 协议统一适配器
  - 多模型支持：OpenAI、DeepSeek、Gemini、Anthropic、Ollama
  - `DecisionEngine` - 决策仪表盘生成
  - `PromptTemplate` - Tera 模板引擎集成
  - `ConversationManager` - 多轮对话上下文管理
  - `SkillRegistry` - 策略技能包管理
- **新闻搜索模块** (`src/news/`) [Phase 3]
  - `NewsProvider` trait - 新闻提供者接口
  - 多源支持：Tavily、SerpAPI、博查搜索、Brave、SearXNG
  - `NewsAggregator` - 多源 fallback 聚合
  - `NewsCache` - 本地缓存存储
  - `NewsArticle`、`NewsSearchRequest`、`NewsSearchResult` 数据模型
- **错误处理增强** (`src/core/error.rs`)
  - 新增 `QuantixError::Network` 变体用于网络错误
- **依赖更新** (`Cargo.toml`)
  - 添加 `url = "2.5"` 用于 URL 解析
  - 添加 `futures = "0.3"` 用于异步流处理

### Changed

- 更新 `README.md` 添加 P0.2、AI 模块、News 模块说明
- 更新历史功能清单文档，增加新 CLI 命令和模块功能树

## 2026-03-27 (续2)

### Added

- **多账户管理系统** (`src/account/*`)
  - `AccountConfig` - 账户配置模型 (Paper/Live/MockLive)
  - `AccountGroup` - 账户组配置，支持资金分配策略
  - `AllocationStrategy` - Equal/Proportional/Weighted/PrimaryFirst
  - `AccountRegistry` - 账户注册表，管理账户和组的 CRUD
  - `AccountRouter` - 智能订单路由，按策略拆分订单
  - `JsonAccountRegistryStore` - JSON 持久化存储
- **账户管理 CLI 命令**
  - `quantix account register/list/show/update/remove/default`
  - `quantix account group create/list/show/remove/add-account/remove-account/set-strategy`
  - `quantix account summary` - 资金聚合视图
  - `quantix account split` - 订单拆分预览
- **算法交易执行器** (`src/execution/algo/*`)
  - TWAP (时间加权平均价格) 执行器
  - VWAP (成交量加权平均价格) 执行器
  - 算法执行上下文和状态管理
- **执行模型增强** (`src/execution/models.rs`)
  - `FillDetails` 扩展：增量成交追踪 (last_fill_price, last_fill_quantity, total_fills)
  - Broker 元数据 (commission, fees, venue, broker_fill_id)
  - `OrderStatus` Serialize/Deserialize 派生
- **风控增强**
  - 行业集中度检查 (`check_industry_limit`) 占位实现
  - 自动减仓触发检测 (`check_auto_reduce_trigger`)
  - 新增事件类型：`IndustryLimitTriggered`, `AutoReduceTriggered`, `AutoReduceExecuted`
- **系统通知增强** (`src/monitoring/notification.rs`)
  - `FeishuSender` - 飞书 Webhook 通知
  - `WechatWorkSender` - 企业微信通知
  - 修复飞书发送器的借用错误
- **Graphiti MCP 集成**
  - MCP 配置：`graphiti-memory` 服务
  - 端点：`http://192.168.123.104:8011/mcp`
  - Group IDs: `quantix_rust_main`, `_review`, `_debug`, `_handoff`, `_docs`
  - 设计记忆写入：多账户管理系统架构决策

### Changed

- 更新 `README.md` 添加多账户管理和 Graphiti MCP 说明
- 更新历史功能清单文档，增加账户管理模块和 CLI 命令树

## 2026-03-27 (续)

### Added

- **风控模块增强** (`src/risk/*`)
  - 新增 `IndustryLimit` 规则类型 - 行业集中度限制
  - 新增 `AutoReduce` 规则类型 - 自动减仓
  - `check_industry_limit` 函数（占位实现，待集成行业分类数据）
  - `check_auto_reduce_trigger` 函数 - 自动减仓触发检查
  - `AutoReduceDecision` 类型 - 自动减仓决策结果
  - 新增事件类型：`IndustryLimitTriggered`, `AutoReduceTriggered`, `AutoReduceExecuted`
- **订单对账模块** (`src/execution/reconciliation.rs`)
  - `OpenOrderScanner` 扫描未完成订单
  - `ReconciliationService` 对账服务，比较本地状态与 broker 状态
  - Unknown 状态订单自动恢复（基于 mock_live 填充计划）
  - 超时订单自动标记失败
- **监控基础设施** (`src/monitoring/health.rs`, `src/monitoring/metrics.rs`)
  - `HealthRegistry` - 健康检查注册表
  - `ComponentHealth` - 组件健康状态
  - `SystemHealth` - 系统整体健康报告
  - `MetricsCollector` - 指标收集器（Counter/Gauge/Histogram）
  - `MetricsExporter` - 指标导出器（Prometheus/JSON 格式）
- **交易日历节假日加载** (`src/core/trading_calendar.rs`, `config/holidays.json`)
  - 从 JSON 配置文件加载 A 股节假日数据
  - 支持调休工作日（周末补班）判断
  - 2024-2026 年节假日数据预置
- **系统通知模块** (`src/monitoring/notification.rs`)
  - `NotificationService` - 多渠道通知服务
  - `DesktopSender` - 桌面通知（Linux notify-send / Windows toast）
  - `WebhookSender` - HTTP POST Webhook 通知
  - `LogSender` - 日志文件通知
  - `QuietHours` - 静默时段配置
  - `NotificationChannel` 枚举 - 支持 Desktop/Webhook/Log/Email

## 2026-03-27

### Added

- **股票异常检测模块** (`src/anomaly/*`)
  - Isolation Forest 算法实现，用于检测 A 股市场异常股票
  - 特征提取：成交量回报 (volume returns)、对数回报 (log returns)、EOM 指标
  - 线性回归统计（斜率、R²、p值）
  - A股特有过滤器：ST股票、涨跌停、停牌、新股
- **东方财富数据源** (`src/anomaly/eastmoney_source.rs`)
  - `EastMoneyAnomalySource` 实现 `DataSource` trait
  - 真实 A 股列表获取（沪深主板、创业板、科创板、北交所）
  - K线数据获取（支持 1/5/15/30/60 分钟、日线）
  - 复权类型支持（前复权 qfq、后复权 hfq、不复权）
- **订单对账模块** (`src/execution/reconciliation.rs`)
  - `OpenOrderScanner` 扫描未完成订单
  - `ReconciliationService` 对账服务，比较本地状态与 broker 状态
  - Unknown 状态订单自动恢复（基于 mock_live 填充计划）
  - 超时订单自动标记失败
- **CLI 命令**
  - `quantix anomaly run` - 使用东方财富 API 检测异常股票
  - `quantix anomaly run --mock` - 使用模拟数据测试
  - `quantix anomaly run --top 20 --output json` - 自定义输出

### Changed

- 更新 `README.md` 添加 Phase 30 异常检测模块说明
- 更新当前完成状态，记录异常检测模块集成

### Algorithm

- 移植自 Surpriver 项目 (Python)
- 理论基础：异常股票未来价格波动是正常股票的 2x+
- 使用 Isolation Forest 无监督学习，不预测方向

## 2026-03-26

### Added

- Added the historical function-list document to record the then-current completed functional design and system-level function tree.
- Added Windows Bridge v1 integration on the Rust side:
  - `src/bridge/*` HTTP client, models, and error layer
  - `src/sources/bridge_tdx.rs` for `TDX bridge source`
  - `src/execution/qmt_bridge.rs` for `QMT preview-only` request previewing
  - `quantix execution bridge status`
  - `quantix execution bridge qmt-preview --request-id <ID>`
- Added bridge-focused test coverage:
  - `tests/bridge_client_test.rs`
  - `tests/bridge_tdx_source_test.rs`
  - `tests/watchlist_bridge_lookup_test.rs`
  - `tests/qmt_bridge_preview_test.rs`

### Changed

- Merged the strategy/execution prerequisite branch chain required by the bridge work into local `master`.
- Updated `README.md` and `docs/USER_MANUAL.md` to reflect the current completed tasks, execution boundaries, and Windows Bridge v1 operator workflow.
- Updated architecture and implementation-plan docs to use the canonical Windows-side path:
  - `/mnt/d/mystocks/quantix/quantix_bridge`

### Completed Design State

- `quantix-rust` continues to own:
  - `execution_request`
  - frozen execution snapshots
  - `ExecutionKernel`
  - `runtime.db`
  - paper/mock-live execution state
- Windows Bridge v1 currently completes:
  - `TDX bridge source`
  - `QMT preview-only`
- Real live broker execution remains deferred.
