# Changelog

All notable changes to this project are documented here.

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
- **文档与卫生契约同步到主线收口事实** (`docs/CLI_COMMAND_MANUAL.html`, `tests/repo_hygiene_test.rs`, `ROADMAP.md`)
  - 同步 `strategy run` 与相关 execution wording 到当前 mock policy 边界
  - 将这轮执行边界收口作为后续 P0.2 / P0.3 的新主线基线

## 2026-04-26

### Added
- **MOCK 数据顶层规范与事实对齐** (`docs/standards/MOCK_USAGE_POLICY.md`, `README.md`, `docs/USER_MANUAL.md`)
  - 新增项目级 `MOCK_USAGE_POLICY`，明确 `anomaly run --mock` 与 `strategy run --mode mock_live` 都属于显式 mock/runtime mock 路径
  - 明确 `mock_live` 是仿真执行与联调加固路径，不等于真实券商实盘
  - 明确真实提交单路径是受保护的 `qmt_live`
  - 明确泛化 `target_mode=live` 仍未实现，且真实路径不得静默回退到 mock
- **文档事实收敛** (`docs/FUNCTION_MAP.md`, `docs/GAP_ANALYSIS.md`, `docs/DEVELOPMENT_ROADMAP.md`, `docs/ROADMAP_REVIEW.md`, `docs/QMT_LIVE_TRADING_SETUP.md`)
  - 同步更新当前能力说明、架构边界、路线图判断与操作手册，消除 `mock_live` / `live` / `qmt_live` 表述漂移
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
- 更新 `docs/FUNCTION_MAP.md` 增加新 CLI 命令和模块功能树

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
- 更新 `docs/FUNCTION_MAP.md` 增加账户管理模块和 CLI 命令树

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

- Added `docs/FUNCTION_MAP.md` to record the current completed functional design and system-level function tree.
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
