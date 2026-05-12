# FUNCTION_TREE

> 本文件是当前主线唯一的功能树与能力边界文档。
>
> 它与根目录 `ROADMAP.md` 一起，构成当前仅保留的两份活跃规划/能力边界入口。
>
> 历史上的 `docs/FUNCTION_MAP.md` 已并入本文件，不再作为单独文档维护。
>
> 当前对应主线基线：`origin/master@562fe84`（PR #62）。

> 状态标记说明：
> `[已实现]` = 当前主线已落地并可用
> `[部分实现]` = 已有模块/命令入口，但能力未完全闭环
> `[未实现]` = 文档已列出，但当前主线尚未落地
> `[待实现]` = 明确保留为后续实现项
> `[非目标]` = 当前边界明确不做

本文档以目录树形式展示 Quantix 量化交易系统的功能层次结构。

```
quantix-rust/
├── 📦 核心层 (core/) [部分实现]
│   ├── 配置管理 (config) [已实现]
│   │   ├── QuantixConfig - 全局配置结构 [已实现]
│   │   └── 运行时配置加载 [已实现]
│   ├── 错误处理 (error) [已实现]
│   │   ├── QuantixError - 统一错误类型 [已实现]
│   │   └── Result<T> - 结果类型别名 [已实现]
│   ├── 运行时环境 (runtime) [已实现]
│   │   ├── CliRuntime - CLI 路径/服务/桥接总运行时 [已实现]
│   │   ├── BridgeRuntimeSettings - Bridge 运行时配置 [已实现]
│   │   │   ├── base_url / api_key [已实现]
│   │   │   ├── bearer_token / contract_version [已实现]
│   │   │   └── qmt poll interval / timeout [已实现]
│   │   └── RuntimeContext - 异步运行时上下文 [已实现]
│   ├── 交易日历 (trading_calendar) [部分实现]
│   │   ├── TradingCalendar - A股交易日历 [部分实现]
│   │   ├── 交易时段判断 (Morning/Afternoon/Auction/Closed) [已实现]
│   │   ├── 节假日加载 (JSON配置) [已实现]
│   │   ├── 调休工作日支持 [已实现]
│   │   └── 按年份动态补充节假日 [待实现]
│   └── 交易时间工具 (trading_time) [已实现]
│       └── 交易时段计算 [已实现]
│
├── 📊 数据层 (data/) [已实现]
│   ├── 数据获取 (fetcher) [已实现]
│   │   └── Fetcher trait - 行情数据获取接口 [已实现]
│   ├── 数据模型 (models) [已实现]
│   │   └── OHLCV、K线等数据结构 [已实现]
│   └── 数据存储 (storage) [已实现]
│       └── Storage trait - K线存取接口 [已实现]
│
├── 🔌 数据源 (sources/) [部分实现]
│   ├── AKShare 数据源 (akshare) [部分实现]
│   │   ├── Python AKShare 健康检查 [已实现]
│   │   └── StockInfo / Kline 拉取 [未实现]
│   ├── 通达信数据源 (tdx) [部分实现]
│   │   ├── TDX TCP 实时行情批量采集 [已实现]
│   │   └── StockInfo / Kline 拉取 [未实现]
│   ├── 通达信文件读取 (tdx_file) [已实现]
│   │   └── day/lc1/lc5 文件解析 [已实现]
│   ├── Bridge TDX 数据源 (bridge_tdx) [已实现]
│   │   └── 通过 Windows Bridge 获取 TDX 数据 [已实现]
│   ├── 东方财富数据源 (eastmoney) [已实现]
│   │   └── 东财 API 行情获取 [已实现]
│   ├── WebSocket 行情 (websocket) [已实现]
│   │   └── 实时行情推送 [已实现]
│   ├── 行情采集器 [已实现]
│   │   ├── QuoteCollector - 通用行情采集 [已实现]
│   │   ├── AuctionCollector - 竞价数据采集 [已实现]
│   │   └── KlineAggregator - K线聚合 [已实现]
│   └── 缠论数据适配 [待实现]
│
├── 📈 策略层 (strategy/) [部分实现]
│   ├── 策略定义 (trait_def) [已实现]
│   │   ├── Strategy trait - 策略接口 [已实现]
│   │   └── Signal 类型定义 [已实现]
│   ├── 内置策略 (strategies/) [已实现]
│   │   ├── 突破策略 (breakout) [已实现]
│   │   ├── 网格策略 (grid) [已实现]
│   │   ├── 均线交叉 (ma_cross) [已实现]
│   │   ├── 均值回归 (mean_reversion) [已实现]
│   │   └── 动量策略 (momentum) [已实现]
│   ├── 策略注册 (registry) [已实现]
│   │   └── 策略注册表管理 [已实现]
│   ├── 策略运行时 (runtime) [已实现]
│   │   ├── 策略实例管理 [已实现]
│   │   └── 信号生成与存储 [已实现]
│   ├── 策略守护进程 (daemon) [部分实现]
│   │   ├── 定时策略执行（当前要求恰好一个 enabled 股票） [部分实现]
│   │   ├── `bootstrap_policy=latest_only` [已实现]
│   │   ├── 自动审批信号联动 execution config [已实现]
│   │   └── 多股票并行守护 / 更宽松配置约束 [未实现]
│   └── 策略服务 (systemd) [已实现]
│       ├── 服务安装/卸载 [已实现]
│       ├── 启动/停止 [已实现]
│       └── 开机自启 [已实现]
│
├── ⚡ 执行层 (execution/) [部分实现]
│   ├── 执行核心 (kernel) [已实现]
│   │   └── ExecutionKernel - 执行决策核心 [已实现]
│   ├── 执行请求 (models) [已实现]
│   │   ├── ExecutionRequest - 执行请求结构 [已实现]
│   │   └── FrozenExecutionSnapshot - 冻结快照 [已实现]
│   ├── Paper 交易 (paper) [部分实现]
│   │   ├── buy / sell 即时撮合 [已实现]
│   │   └── query_order / cancel_order [未实现]
│   ├── Mock Live 模式 (mock_live) [已实现]
│   │   └── 伪实盘模式（手动确认） [已实现]
│   ├── 运行时存储 (runtime_store) [已实现]
│   │   └── SQLite 运行状态持久化 [已实现]
│   ├── 请求诊断 (request_diagnostics) [已实现]
│   │   ├── execution_diagnostics 结构化负载 [已实现]
│   │   └── `qmt_live` gate / bridge / runtime 失败分类 [已实现]
│   ├── 订单对账 (reconciliation) [已实现]
│   │   ├── OpenOrderScanner - 未完成订单扫描 [已实现]
│   │   ├── ReconciliationService - 对账服务 [已实现]
│   │   ├── Unknown 状态自动恢复 [已实现]
│   │   └── 超时订单自动标记失败 [已实现]
│   ├── QMT Bridge (qmt_bridge) [已实现]
│   │   └── QMT 预览请求 [已实现]
│   ├── QMT Live 门控 (qmt_live_gate) [已实现]
│   │   └── 能力/模式校验，只允许 guarded `qmt_live` [已实现]
│   ├── QMT 任务提交服务 (qmt_task_submit_service) [已实现]
│   │   ├── `/api/v1/task/execute` receipt 提交 [已实现]
│   │   ├── `/api/v1/task/result/{task_id}` 查询/轮询 [已实现]
│   │   ├── `client_order_id` / `local_submission_id` identity 校验 [已实现]
│   │   └── `task_id -> external_order_id` 撤单前 broker identity 解析 [已实现]
│   ├── QMT Live 适配器 (qmt_live_adapter) [已实现]
│   │   ├── submit_order -> `PendingSubmit` task receipt [已实现]
│   │   ├── query_order -> pending / accepted / rejected / filled 映射 [已实现]
│   │   └── cancel_order -> `qmt_live` gate + `task_id -> external_order_id` + 兼容取消端点 [已实现]
│   ├── 算法交易执行器 (algo) [部分实现]
│   │   ├── AlgoParams / AlgoContext / AlgoState [已实现]
│   │   ├── TWAP 执行器 [已实现]
│   │   ├── VWAP 执行器 [已实现]
│   │   └── POV / Iceberg [待实现]
│   ├── 执行适配器 (adapter) [已实现]
│   │   └── 多 broker 适配接口 [已实现]
│   ├── 执行守护进程 (daemon) [已实现]
│   │   ├── 执行服务后台运行 [已实现]
│   │   └── 成功/失败路径都写出结构化执行诊断 [已实现]
│   └── 执行配置 (config) [已实现]
│       └── 执行参数配置 [已实现]
│
├── 👥 账户管理 (account/) [已实现]
│   ├── 账户模型 (models) [已实现]
│   │   ├── AccountConfig - 账户配置 [已实现]
│   │   ├── AccountType - 账户类型 (Paper/Live/MockLive) [已实现]
│   │   ├── AccountGroup - 账户组 [已实现]
│   │   ├── AllocationStrategy - 分配策略 [已实现]
│   │   │   ├── Equal - 平均分配 [已实现]
│   │   │   ├── Proportional - 按资金比例 [已实现]
│   │   │   ├── Weighted - 自定义权重 [已实现]
│   │   │   └── PrimaryFirst - 主账户优先 [已实现]
│   │   ├── OrderSplitRequest - 订单拆分请求 [已实现]
│   │   ├── SplitTarget - 拆分目标 (Single/Group) [已实现]
│   │   └── OrderSplitResult - 拆分结果 [已实现]
│   ├── 账户注册表 (registry) [已实现]
│   │   ├── AccountRegistry - 账户注册表 [已实现]
│   │   ├── 账户 CRUD 操作 [已实现]
│   │   ├── 账户组 CRUD 操作 [已实现]
│   │   └── 默认账户管理 [已实现]
│   ├── 智能路由 (router) [已实现]
│   │   ├── AccountRouter - 账户路由器 [已实现]
│   │   ├── 订单拆分逻辑 [已实现]
│   │   └── 多账户分配 [已实现]
│   └── 账户存储 (storage) [已实现]
│       ├── JsonAccountRegistryStore - JSON存储 [已实现]
│       └── ~/.quantix/accounts/registry.json [已实现]
│
├── 🛡️ 风控层 (risk/) [已实现]
│   ├── 风控模型 (models) [已实现]
│   │   ├── RiskRule - 风控规则 [已实现]
│   │   ├── RiskRuleType - 规则类型 [已实现]
│   │   │   ├── PositionLimit - 持仓限制 [已实现]
│   │   │   ├── DailyLossLimit - 日内亏损限制 [已实现]
│   │   │   ├── VolatilityLimit - 波动率限制 [已实现]
│   │   │   ├── IndustryLimit - 行业集中度限制 [已实现]
│   │   │   └── AutoReduce - 自动减仓 [已实现]
│   │   ├── RiskState - 风控状态 [已实现]
│   │   └── RiskAccountSnapshot - 账户快照 [已实现]
│   ├── 风控服务 (service) [已实现]
│   │   ├── RiskService - 风控服务核心 [已实现]
│   │   ├── 买入前风控检查 [已实现]
│   │   ├── 行业集中度检查 [已实现]
│   │   └── 自动减仓触发检测 [已实现]
│   ├── 风控存储 (storage) [已实现]
│   │   └── SQLite 风控数据持久化 [已实现]
│   ├── 实盘流水导入 (importer) [已实现]
│   │   └── 标准化流水导入 [已实现]
│   ├── 导入存储 (import_store) [已实现]
│   │   └── 导入数据缓存 [已实现]
│   ├── 账户重建 (rebuild) [已实现]
│   │   └── 实盘镜像账户重建 [已实现]
│   └── 波动率计算 (volatility) [已实现]
│       └── 历史波动率计算 [已实现]
│
├── 📉 监控层 (monitoring/) [部分实现]
│   ├── 告警系统 (alert) [已实现]
│   │   ├── AlertManager - 告警管理器 [已实现]
│   │   ├── AlertThreshold - 阈值配置 [已实现]
│   │   ├── AlertLevel (Info/Warning/Error/Critical) [已实现]
│   │   └── AlertType (Signal/Position/Performance/Risk/System) [已实现]
│   ├── 健康检查 (health) [已实现]
│   │   ├── HealthRegistry - 健康检查注册表 [已实现]
│   │   ├── ComponentHealth - 组件健康状态 [已实现]
│   │   └── SystemHealth - 系统整体健康报告 [已实现]
│   ├── 指标收集 (metrics) [已实现]
│   │   ├── MetricsCollector - 指标收集器 [已实现]
│   │   ├── Counter/Gauge/Histogram 类型 [已实现]
│   │   └── MetricsExporter - 指标导出 (Prometheus/JSON) [已实现]
│   ├── 通知系统 (notification) [部分实现]
│   │   ├── NotificationService - 通知服务 [已实现]
│   │   ├── DesktopSender - 桌面通知 (Linux/Windows) [已实现]
│   │   ├── WebhookSender - HTTP POST 通知 [已实现]
│   │   ├── LogSender - 日志文件通知 [已实现]
│   │   ├── WechatWorkSender - 企业微信通知 [已实现]
│   │   ├── FeishuSender - 飞书通知 [已实现]
│   │   ├── Telegram / Discord / Slack / Dingtalk / Pushplus [待实现]
│   │   ├── Email 渠道 [待实现]
│   │   └── QuietHours - 静默时段配置 [已实现]
│   ├── 信号监控 (signal_monitor) [已实现]
│   │   └── 策略信号实时追踪 [已实现]
│   ├── 持仓监控 (position_monitor) [已实现]
│   │   ├── PositionMonitor - 持仓状态监控 [已实现]
│   │   └── PositionSnapshot - 持仓快照 [已实现]
│   └── 性能监控 (performance_monitor) [已实现]
│       ├── PerformanceMonitor - 实时性能监控 [已实现]
│       └── RealtimeMetrics - 实时指标计算 [已实现]
│
├── 📋 自选池 (watchlist/) [已实现]
│   ├── 自选模型 (models) [已实现]
│   │   ├── WatchlistEntry - 自选条目 [已实现]
│   │   └── WatchlistHistoryEvent - 历史事件 [已实现]
│   ├── 行情解析 (resolver) [已实现]
│   │   ├── WatchlistResolver - 行情数据解析 [已实现]
│   │   ├── TdxWatchlistQuoteLookup - TDX行情查询 [已实现]
│   │   ├── BridgeTdxWatchlistQuoteLookup - Bridge行情查询 [已实现]
│   │   └── PostgresWatchlistNameLookup - 名称查询 [已实现]
│   ├── 自选服务 (service) [已实现]
│   │   └── WatchlistService - 自选池管理 [已实现]
│   └── 自选存储 (storage) [已实现]
│       └── WatchlistStorage - 持久化存储 [已实现]
│
├── 🛑 止盈止损 (stop/) [已实现]
│   ├── 止损模型 (models) [已实现]
│   │   ├── StopRule - 止损规则 [已实现]
│   │   ├── StopTriggerKind - 触发类型 [已实现]
│   │   │   ├── Fixed - 固定价格 [已实现]
│   │   │   ├── Percentage - 百分比 [已实现]
│   │   │   └── Trailing - 跟踪止损 [已实现]
│   │   └── StopHistoryEvent - 历史事件 [已实现]
│   ├── 止损服务 (service) [已实现]
│   │   ├── StopService - 止损服务 [已实现]
│   │   └── 实时止损评估 [已实现]
│   └── 止损存储 (storage) [已实现]
│       └── SqliteStopRuleStore - SQLite存储 [已实现]
│
├── 💰 模拟交易 (trade/) [已实现]
│   ├── 交易模型 (models) [已实现]
│   │   ├── PaperTradeAccount - 模拟账户 [已实现]
│   │   ├── TradeRecord - 交易记录 [已实现]
│   │   ├── TradePosition - 持仓 [已实现]
│   │   └── TradeSide - 买卖方向 [已实现]
│   ├── 费用计算 (fees) [已实现]
│   │   ├── FeeConfig - 费用配置 [已实现]
│   │   ├── FeeBreakdown - 费用明细 [已实现]
│   │   └── calculate_fee_breakdown - 费用计算 [已实现]
│   ├── 交易服务 (service) [已实现]
│   │   ├── TradeService - 交易服务 [已实现]
│   │   └── PaperTradeStore - 模拟账户存储 [已实现]
│   ├── 报告服务 (reporting) [已实现]
│   │   └── TradeReportingService - 交易报告 [已实现]
│   └── 交易存储 (storage) [已实现]
│       └── JsonPaperTradeStore - JSON存储 [已实现]
│
├── 🔍 市场分析 (market/) [部分实现]
│   ├── 市场模型 (models) [已实现]
│   │   ├── MarketOverview - 市场概览 [已实现]
│   │   ├── BoardRankRow - 板块排名 [已实现]
│   │   ├── LeaderRow - 龙头股 [已实现]
│   │   ├── MarketSentimentSnapshot - 市场情绪 [已实现]
│   │   └── NorthFlowSnapshot - 北向资金 [已实现]
│   └── 市场服务 (service) [已实现]
│       ├── MarketService - 市场服务 [已实现]
│       ├── MarketDataReader - 数据读取 [已实现]
│       ├── 行业板块分析 [已实现]
│       ├── 概念板块分析 [已实现]
│       ├── 龙头股识别 [已实现]
│       └── 北向资金分析 [已实现]
│   ├── 强度分析 (strength) [已实现]
│   │   ├── MarketAnalysisFoundation - 市场基础画像 [已实现]
│   │   ├── MarketStrengthReport - 强弱板块报告 [已实现]
│   │   └── foundation / strength-stocks 相关分析 [已实现]
│   └── 舆情聚合 (sentiment) [部分实现]
│       ├── SentimentProvider trait [已实现]
│       ├── SentimentAggregator - 无provider时空结果聚合 [部分实现]
│       ├── SentimentData / SocialMention / SentimentScore / SentimentHistoryPoint [已实现]
│       ├── 默认provider接入 [未实现]
│       └── 趋势计算 [待实现]
│
├── 🎯 选股器 (screener/) [已实现]
│   ├── 选股模型 (models) [已实现]
│   │   ├── ScreenRow - 筛选结果 [已实现]
│   │   ├── ScreenRunOptions - 运行选项 [已实现]
│   │   ├── PresetInvocation - 预设调用 [已实现]
│   │   └── ScreenUniverse - 股票池 [已实现]
│   ├── 条件解析 (parser) [已实现]
│   │   └── parse_preset_invocation - 条件解析 [已实现]
│   ├── 条件评估 (evaluator) [已实现]
│   │   ├── evaluate_preset - 条件评估 [已实现]
│   │   └── required_lookback - 回溯计算 [已实现]
│   └── 选股服务 (service) [已实现]
│       ├── ScreenerService - 选股服务 [已实现]
│       └── DailyKlineLoader - 日线加载 [已实现]
│
├── 📐 技术分析 (analysis/) [已实现]
│   ├── 技术指标 (indicators) [已实现]
│   │   ├── MA/EMA/SMA 均线 [已实现]
│   │   ├── MACD [已实现]
│   │   ├── RSI [已实现]
│   │   ├── BOLL 布林带 [已实现]
│   │   ├── KDJ [已实现]
│   │   └── 更多指标... [已实现]
│   ├── K线形态 (candle_patterns) [已实现]
│   │   └── K线形态识别 [已实现]
│   ├── 回测引擎 (backtest) [已实现]
│   │   ├── BacktestEngine - 回测引擎 [已实现]
│   │   └── BacktestResult - 回测结果 [已实现]
│   ├── 竞价分析 (auction) [已实现]
│   │   ├── AuctionAnalyzer - 竞价分析器 [已实现]
│   │   ├── SectorStats - 板块统计 [已实现]
│   │   └── StrengthLevel - 强度等级 [已实现]
│   ├── 性能计算 (performance) [已实现]
│   │   ├── PerformanceCalculator - 性能计算 [已实现]
│   │   └── PerformanceReport - 性能报告 [已实现]
│   ├── 投资组合 (portfolio) [已实现]
│   │   ├── Portfolio - 投资组合 [已实现]
│   │   ├── Position - 持仓 [已实现]
│   │   └── Order - 订单 [已实现]
│   └── Polars 适配 (polars_adapter) [已实现]
│       ├── PolarsCalculator - Polars计算 [已实现]
│       └── 批量K线数据处理 [已实现]
│
├── 🧮 因子研究 (factor/) [部分实现]
│   ├── FactorDataset - Polars long-form 因子面板 [已实现]
│   ├── FactorDataLoader trait - 异步数据加载边界 [已实现]
│   ├── CsvFactorDataLoader - 本地CSV首切片加载器 [已实现]
│   ├── 因子算子 (operators) [部分实现]
│   │   ├── cs_rank - 按date横截面rank [已实现]
│   │   ├── ts_delay - 按symbol时间序列延迟 [已实现]
│   │   └── ts_delta - 按symbol时间序列差分 [已实现]
│   ├── 因子目录 (catalog) [部分实现]
│   │   ├── rank_close [已实现]
│   │   ├── delay_close_1 [已实现]
│   │   └── delta_close_1 [已实现]
│   ├── 因子检查 (check) [部分实现]
│   │   ├── required columns / dtype / uniqueness [已实现]
│   │   └── basic no-lookahead structure check [已实现]
│   ├── 因子导出 (export) [部分实现]
│   │   ├── CSV字符串导出 [已实现]
│   │   ├── JSON摘要导出 [已实现]
│   │   └── Parquet文件导出 [已实现]
│   ├── CLI (quantix factor) [部分实现]
│   │   ├── factor list [已实现]
│   │   ├── factor compute --input CSV [已实现]
│   │   └── factor evaluate --input CSV --format table/json/csv [已实现]
│   ├── Alpha101 [部分实现]
│   │   └── alpha101_002 / 003 / 005 / 006 / 012 [已实现]
│   ├── Alpha191 [部分实现]
│   │   ├── alpha191_101 / 102 / 103 [已实现]
│   │   ├── alpha191_104 / 105 / 106 / 107 / 108 / 109 / 110 [已实现]
│   │   └── alpha191_111 / 112 / 113 / 114 / 115 / 116 / 117 / 118 / 119 / 120 [已实现]
│   ├── IC/IR / correlation / neutralization [部分实现]
│   │   ├── IC/IR evaluation [已实现]
│   │   ├── factor value correlation [已实现]
│   │   └── neutralization - 横截面OLS残差中性化 [已实现]
│   └── layered factor backtest - 等权分层收益 / long-short [已实现]
│
├── 🎲 异常检测 (anomaly/) [已实现]
│   ├── Isolation Forest (forest) [已实现]
│   │   └── IsolationForest - 隔离森林算法 [已实现]
│   ├── 统计函数 (statistics) [已实现]
│   │   └── 平均路径长度计算 [已实现]
│   ├── 特征提取 (features) [已实现]
│   │   ├── FeatureExtractor - 特征提取器 [已实现]
│   │   ├── volume returns - 成交量回报 [已实现]
│   │   ├── log returns - 对数回报 [已实现]
│   │   └── EOM 指标 [已实现]
│   ├── A股过滤器 (filter) [已实现]
│   │   ├── StockFilter - 股票过滤器 [已实现]
│   │   ├── ST股票过滤 [已实现]
│   │   ├── 涨跌停过滤 [已实现]
│   │   ├── 停牌过滤 [已实现]
│   │   └── 新股过滤 [已实现]
│   ├── 检测服务 (detector) [已实现]
│   │   ├── AnomalyDetector - 异常检测器 [已实现]
│   │   └── AnomalyResult - 检测结果 [已实现]
│   ├── 东方财富数据源 (eastmoney_source) [已实现]
│   │   └── EastMoneyAnomalySource - 东财数据源 [已实现]
│   └── 配置管理 (config) [已实现]
│       ├── AnomalyConfig - 检测配置 [已实现]
│       ├── ForestConfig - 森林配置 [已实现]
│       └── FilterConfig - 过滤配置 [已实现]
│
├── 🖥️ 监控服务 (monitor/) [已实现]
│   ├── 监控配置 (config) [已实现]
│   │   └── MonitorConfig - 监控配置 [已实现]
│   ├── 监控模型 (models) [已实现]
│   │   ├── PriceAlert - 价格告警 [已实现]
│   │   ├── MonitorEventRow - 监控事件 [已实现]
│   │   └── TriggeredAlert - 触发告警 [已实现]
│   ├── 监控运行器 (runner) [已实现]
│   │   └── MonitorRunner - 监控运行器 [已实现]
│   ├── 监控服务 (service) [已实现]
│   │   └── MonitorService - 监控服务 [已实现]
│   ├── 服务配置 (service_config) [已实现]
│   │   └── MonitorServiceConfig - 服务配置 [已实现]
│   ├── 监控存储 (storage) [已实现]
│   │   └── SqliteMonitorAlertStore - SQLite存储 [已实现]
│   └── Systemd 服务 (systemd) [已实现]
│       └── MonitorUserServiceInstaller - 用户服务安装 [已实现]
│
├── 🌉 Windows Bridge (bridge/) [已实现]
│   ├── HTTP 客户端 (client) [已实现]
│   │   └── Bridge HTTP 客户端 [已实现]
│   ├── 数据模型 (models) [已实现]
│   │   └── Bridge 请求/响应模型 [已实现]
│   └── 错误处理 (error) [已实现]
│       └── Bridge 错误类型 [已实现]
│
├── 💾 数据库层 (db/) [已实现]
│   ├── ClickHouse (clickhouse) [已实现]
│   │   ├── ClickHouseClient - 客户端 [已实现]
│   │   ├── KlineDataCH - K线数据 [已实现]
│   │   ├── StockInfoCH - 股票信息 [已实现]
│   │   ├── StockQuoteCH - 实时行情 [已实现]
│   │   └── LimitUpEventCH - 涨停事件 [已实现]
│   ├── PostgreSQL (postgresql) [已实现]
│   │   ├── PostgresClient - 客户端 [已实现]
│   │   ├── KlineDaily - 日线数据 [已实现]
│   │   └── StockInfo - 股票信息 [已实现]
│   └── TDengine (tdengine) [已实现]
│       ├── TDengineClient - 客户端 [已实现]
│       └── MinuteKline - 分钟K线 [已实现]
│
├── 📥 数据导入导出 (io/) [已实现]
│   ├── 导出器 (exporter) [已实现]
│   │   ├── DataExporter - 数据导出器 [已实现]
│   │   ├── CSV/JSON/Parquet 格式 [已实现]
│   │   └── ExportResult - 导出结果 [已实现]
│   ├── 导入器 (importer) [已实现]
│   │   ├── DataImporter - 数据导入器 [已实现]
│   │   └── 多格式数据导入 [已实现]
│   ├── 数据验证 (validation) [已实现]
│   │   ├── DataValidator - 数据验证器 [已实现]
│   │   └── ValidationResult - 验证结果 [已实现]
│   └── 批处理 (batch) [已实现]
│       ├── BatchProcessor - 批量处理器 [已实现]
│       └── BatchProgress - 处理进度 [已实现]
│
├── 🔄 数据同步 (sync/) [部分实现]
│   └── ETL 同步 (etl) [部分实现]
│       ├── DataSync - ClickHouse 写入与调度循环 [部分实现]
│       ├── SyncConfig - 同步配置 [已实现]
│       ├── SyncStats - 同步统计 [已实现]
│       ├── 市场基础面快照写入 [已实现]
│       └── PostgreSQL / 分钟线来源拉取 [未实现]
│
├── ⏰ 任务调度 (tasks/) [已实现]
│   ├── 任务调度器 (scheduler) [已实现]
│   │   ├── TaskScheduler - 任务调度器 [已实现]
│   │   ├── ScheduledTask - 调度任务 [已实现]
│   │   └── TaskTemplates - 任务模板 [已实现]
│   ├── 采集调度 (collect_scheduler) [已实现]
│   │   ├── CollectScheduler - 采集调度器 [已实现]
│   │   └── SchedulerConfig - 调度配置 [已实现]
│   └── Cron 表达式 (cron) [已实现]
│       └── CronExpression - Cron解析 [已实现]
│
├── 🤖 AI决策层 (ai/) [部分实现]
│   ├── LLM适配器 (llm_adapter) [已实现]
│   │   └── OpenAI-compatible 统一适配 [已实现]
│   ├── 多模型支持 (providers) [部分实现]
│   │   ├── OpenAI - GPT-4o系列 [已实现]
│   │   ├── DeepSeek - DeepSeek-Chat/Reasoner [已实现]
│   │   ├── Ollama - 本地模型 [已实现]
│   │   ├── Gemini - 环境配置/模型枚举 [部分实现]
│   │   └── Anthropic - 环境配置/模型枚举 [部分实现]
│   ├── Prompt模板 (prompt_templates) [已实现]
│   │   └── Tera模板引擎 [已实现]
│   ├── 决策引擎 (decision_engine) [已实现]
│   │   └── 决策分析/问答能力 [已实现]
│   ├── 对话管理 (conversation) [未实现]
│   │   └── 多轮对话上下文 [未实现]
│   └── 技能注册 (skill_registry) [未实现]
│       └── 策略技能包管理 [未实现]
│
├── 📰 新闻搜索层 (news/) [部分实现]
│   ├── 新闻提供者 (provider) [已实现]
│   │   └── NewsProvider trait [已实现]
│   ├── 多源支持 (providers) [部分实现]
│   │   ├── Tavily - 高质量AI友好 [已实现]
│   │   ├── SerpAPI - 全渠道搜索 [已实现]
│   │   ├── 博查搜索 - 中文优化 [已实现]
│   │   ├── Brave - 隐私优先 [未实现]
│   │   └── SearXNG - 自建实例 [未实现]
│   ├── 新闻聚合 (aggregator) [已实现]
│   │   └── 多源fallback机制 [已实现]
│   └── 新闻缓存 (cache) [已实现]
│       └── 本地缓存存储 [已实现]
│
├── 📊 基本面分析 (fundamental/) [部分实现]
│   ├── 基本面提供者 (provider) [已实现]
│   │   └── FundamentalProvider trait [已实现]
│   ├── EastMoney数据源 (eastmoney) [部分实现]
│   │   ├── EastMoneyFundamentalProvider [部分实现]
│   │   ├── valuation / latest earnings / institution [已实现]
│   │   ├── earnings history（当前退化为最新一季） [部分实现]
│   │   ├── dragon_tiger（仅榜单汇总，营业部明细未填） [部分实现]
│   │   └── dividend / capital_flow [未实现]
│   ├── 估值指标 (valuation) [已实现]
│   │   └── PE/PB/PS/市值/ROE/EPS [已实现]
│   ├── 财报数据 (earnings) [部分实现]
│   │   ├── 最新一季营收/净利润/毛利率 [已实现]
│   │   └── 多年历史财报（当前退化为单条最新一季） [部分实现]
│   ├── 机构持仓 (institution) [已实现]
│   │   └── 基金/机构持仓 + 类型映射 [已实现]
│   ├── 资金流向 (capital_flow) [未实现]
│   │   └── 主力资金追踪 [未实现]
│   ├── 龙虎榜 (dragon_tiger) [部分实现]
│   │   ├── 今日/区间龙虎榜汇总 [已实现]
│   │   └── 买卖前5营业部明细 [待实现]
│   └── 分红信息 (dividend) [未实现]
│       └── 历史分红记录 [未实现]
│
├── 📥 智能导入 (import/) [部分实现] [可选 - Phase 5]
│   ├── 图片提取 (image_extractor) [已实现]
│   │   └── LLM Vision识别 [已实现]
│   ├── CSV解析 (csv_parser) [已实现]
│   ├── Excel解析 (excel_parser) [未实现]
│   ├── 剪贴板解析 (clipboard) [已实现]
│   ├── 文本解析 (text_parser) [已实现]
│   └── 代码解析器 (code_resolver) [已实现]
│
├── 🖼️ TUI 界面 (tui/) [部分实现]
│   └── 应用 (app) [部分实现]
│       └── run_menu - 静态文本菜单占位，ratatui未接入 [部分实现]
│
└── 📟 CLI 命令 (cli/) [部分实现]
    └── 命令处理器 (handlers) [部分实现]
        ├── init - 初始化配置 [已实现]
        ├── menu - dialoguer 简易交互菜单（部分子菜单仍占位） [部分实现]
        ├── data - 数据查询/导出（导出当前仅 CSV 可用） [部分实现]
        ├── strategy - 策略管理/运行（CLI 当前仅闭环 ma_cross） [部分实现]
        ├── task - 任务调度（Foundation P0 仅预置模板 / 前台执行） [部分实现]
        ├── analyze - 技术分析/筛选 [已实现]
        ├── backtest - 回测运行/报告 [部分实现]
        ├── performance - 绩效对比 [已实现]
        ├── monitor - 自选监控/告警 [已实现]
        ├── stop - 止盈止损管理 [已实现]
        ├── watchlist - 自选池管理 [已实现]
        ├── market - 市场分析 [已实现]
        ├── trade - 模拟交易 [已实现]
        ├── risk - 风控管理 [部分实现]
        ├── execution - 执行守护进程 / bridge / qmt（live 需经 qmt_live + bridge） [已实现]
        ├── anomaly - 异常检测 [已实现]
        ├── algo - 算法交易（当前仅 TWAP/VWAP，且任务为单进程内存态） [部分实现]
        ├── account - 账户管理 [已实现]
        ├── ai - AI 决策 [部分实现]
        ├── news - 新闻搜索 [部分实现]
        ├── fundamental - 基本面分析（分红占位，资金流向 CLI 未暴露） [部分实现]
        ├── sentiment - 舆情分析 [部分实现]
        ├── notify - 多渠道通知 [部分实现]
        ├── import - 智能导入 [部分实现]
        └── status - 系统状态 [已实现]
```

## CLI 命令树

``` 
quantix
├── init                    # 初始化配置和数据库 [已实现]
├── menu                    # 交互式菜单（simple menu 已接线，部分子菜单仍占位） [部分实现]
│   └── --tui               # TUI 界面模式（当前仅占位提示） [部分实现]
├── status                  # 系统状态 [已实现]
│   └── --health            # 健康检查 [已实现]
│
├── data                    # 数据命令 [部分实现]
│   ├── source              # 数据源管理 [已实现]
│   │   ├── list            # 列出数据源 [已实现]
│   │   ├── add             # 新增/更新数据源 [已实现]
│   │   ├── set-default     # 设置默认数据源 [已实现]
│   │   └── test            # 测试数据源连通性 [已实现]
│   ├── import-fundamentals # 导入市场基础面快照 [已实现]
│   ├── query               # 查询历史数据 [已实现]
│   └── export              # 导出数据 [部分实现，仅 CSV 可用；默认 parquet 路径仍占位]
│
├── strategy                # 策略命令（CLI 当前仅闭环 ma_cross） [部分实现]
│   ├── create              # 创建策略实例（当前仅支持 ma_cross） [已实现]
│   ├── update              # 更新策略实例（策略名当前仅支持 ma_cross） [已实现]
│   ├── delete              # 删除策略实例 [已实现]
│   ├── run                 # 运行策略（当前仅 ma_cross，且仅 paper/mock_live 直跑） [部分实现]
│   ├── list                # 列出策略 [已实现]
│   ├── show                # 显示详情 [已实现]
│   ├── config              # 配置管理 [已实现]
│   ├── daemon              # 守护进程（当前仅支持 bootstrap_policy=latest_only，且要求恰好一个 enabled 股票） [部分实现]
│   ├── signal              # 信号管理 [部分实现]
│   │   ├── list            # 列出信号 [已实现]
│   │   ├── approve         # 批准信号（支持 paper/mock_live/qmt_live；live 不支持） [部分实现]
│   │   └── reject          # 拒绝信号 [已实现]
│   ├── request             # 执行请求 [部分实现]
│   │   ├── list            # 列出请求 [已实现]
│   │   │   ├── --status    # 按状态过滤 [已实现]
│   │   │   ├── --target-mode # 按执行模式过滤 [已实现]
│   │   │   ├── --target-account # 按目标账户过滤 [已实现]
│   │   │   ├── --stats     # 统计汇总视图 [已实现]
│   │   │   └── --verbose   # 详细输出 [已实现]
│   │   ├── show            # 查看请求详情 [已实现]
│   │   │   ├── --request-id # 请求ID [已实现]
│   │   │   └── --verbose   # 故障排查信息 [已实现]
│   │   ├── execute         # 执行请求（仅 paper/mock_live；qmt_live 需走 execution bridge qmt-live） [部分实现]
│   │   └── cancel          # 取消请求 [已实现]
│   ├── service             # systemd 服务（所有动作都依赖已配置 service-config） [已实现]
│   └── service-config      # systemd 服务配置（show 未配置时输出引导提示） [已实现]
│
├── task                    # 任务命令（Foundation P0 仅预置模板 / 前台执行） [部分实现]
│   ├── add                 # 添加定时任务 [部分实现，命令保留但 Foundation P0 未开放]
│   ├── list                # 列出任务模板 [已实现]
│   ├── start               # 启动调度器 [部分实现，仅支持前台预置模板]
│   ├── stop                # 停止调度器 [已实现，提示前台 Ctrl+C]
│   └── status              # 查看任务能力状态 [已实现]
│
├── analyze                 # 分析命令 [已实现]
│   ├── indicators          # 计算技术指标 [已实现]
│   ├── backtest            # 查看既有回测报告 [已实现]
│   ├── candle-pattern      # K线形态识别（输入源必须三选一、参考价必须二选一；TDX day-file/tdx-root 当前仅支持 1d，且 --tdx-root 需配合 --code） [已实现]
│   └── screener            # 选股筛选 [已实现]
│       ├── preset-list     # 预设条件列表 [已实现]
│       └── run             # 运行筛选（必须且只能指定 --codes/--watchlist 之一；--group 仅可配合 --watchlist；且至少一个 --preset；sort_by 当前仅支持 code/score） [已实现]
│
├── backtest                # 回测命令（当前仅 ma_cross 单标的日线回测闭环） [部分实现]
│   ├── run                 # 运行回测（当前仅支持 ma_cross） [部分实现]
│   ├── report              # 查看回测报告 [已实现]
│   ├── list                # 列出回测报告 [已实现]
│   └── compare             # 对比多个回测报告 [已实现]
│
├── performance             # 绩效命令 [已实现]
│   ├── report              # 查看绩效详情 [已实现]
│   ├── list                # 列出可分析报告 [已实现]
│   └── compare             # 对比绩效指标 [已实现]
│
├── monitor                 # 监控命令 [已实现]
│   ├── watchlist           # 自选监控（--once/--repeat 必须二选一） [已实现]
│   │   ├── --once          # 执行一次监控 [已实现]
│   │   └── --repeat        # 持续重复监控 [已实现]
│   ├── alert               # 价格告警 [已实现]
│   │   ├── add             # 添加告警（--above/--below 必须二选一） [已实现]
│   │   ├── list            # 列出告警 [已实现]
│   │   └── remove          # 删除告警 [已实现]
│   ├── config              # 监控配置 [已实现]
│   │   ├── show            # 显示当前监控配置 [已实现]
│   │   ├── set             # 修改监控配置（interval_seconds/group/persist_events/notify 必须四选一） [已实现]
│   │   └── clear-group     # 清除分组限制 [已实现]
│   ├── daemon              # 守护进程 [已实现]
│   │   └── run             # 运行监控守护进程 [已实现]
│   ├── service             # systemd 服务（除 status 外依赖已配置 service-config） [已实现]
│   │   ├── install         # 安装用户服务 [已实现]
│   │   ├── uninstall       # 卸载用户服务 [已实现]
│   │   ├── start           # 启动用户服务 [已实现]
│   │   ├── stop            # 停止用户服务 [已实现]
│   │   ├── status          # 查看用户服务状态 [已实现]
│   │   ├── enable          # 启用开机自启 [已实现]
│   │   └── disable         # 禁用开机自启 [已实现]
│   ├── service-config      # 服务配置 [已实现]
│   │   ├── show            # 显示当前服务配置；未配置时输出引导提示 [已实现]
│   │   └── set             # 设置 quantix 二进制路径 [已实现]
│   └── event               # 事件历史 [已实现]
│       └── list            # 查看监控事件历史（type 当前仅支持 price-alert/stop-loss/stop-profit/trailing-stop） [已实现]
│
├── stop                    # 止盈止损 [已实现]
│   ├── set                 # 设置规则（股票需先在自选池；至少一个阈值；loss/loss-pct/trailing 互斥，profit/profit-pct 互斥） [已实现]
│   ├── update              # 更新规则（股票需先在自选池；修改或 clear 项至少一项；loss/loss-pct/trailing 互斥，profit/profit-pct 互斥） [已实现]
│   ├── list                # 列出规则 [已实现]
│   ├── status              # 查看状态 [已实现]
│   ├── history             # 历史记录（type 当前仅支持 set/update/remove/trigger） [已实现]
│   └── remove              # 删除规则 [已实现]
│
├── watchlist               # 自选池 [已实现]
│   ├── add                 # 添加股票 [已实现]
│   ├── remove              # 移除股票 [已实现]
│   ├── list                # 列出自选 [已实现]
│   ├── move                # 移动分组 [已实现]
│   ├── group               # 分组管理 [已实现]
│   │   ├── create          # 创建分组 [已实现]
│   │   └── list            # 列出分组 [已实现]
│   ├── tag                 # 标签管理 [已实现]
│   │   ├── add             # 添加标签 [已实现]
│   │   ├── remove          # 删除标签 [已实现]
│   │   └── list            # 列出标签 [已实现]
│   └── history             # 历史记录 [已实现]
│
├── market                  # 市场分析 [已实现]
│   ├── foundation          # 市场基础数据摘要 [已实现]
│   ├── sector              # 行业板块（sort_by 当前仅支持 change/change_pct） [已实现]
│   ├── concept             # 概念板块（sort_by 当前仅支持 change/change_pct） [已实现]
│   ├── north               # 北向资金 [已实现]
│   ├── sentiment           # 市场情绪 [已实现]
│   ├── leader              # 龙头股（必须且只能指定 --sector/--concept/--all 之一） [已实现]
│   ├── overview            # 综合概览 [已实现]
│   ├── strength            # 强弱板块分析 [已实现]
│   └── strength-stocks     # 强势板块个股排行 [已实现]
│
├── trade                   # 模拟交易 [已实现]
│   ├── init                # 初始化账户 [已实现]
│   ├── reset               # 重置账户 [已实现]
│   ├── buy                 # 买入 [已实现]
│   ├── sell                # 卖出 [已实现]
│   ├── history             # 成交历史 [已实现]
│   ├── fees                # 费用明细 [已实现]
│   ├── overview            # 账户概览 [已实现]
│   │   └── --current       # 现价视图（依赖行情查询） [已实现]
│   ├── position            # 当前持仓 [已实现]
│   │   └── --current       # 现价视图（依赖行情查询） [已实现]
│   └── cash                # 现金快照 [已实现]
│
├── risk                    # 风控管理 [部分实现]
│   ├── import              # 导入流水 [已实现]
│   │   └── live-trades     # 导入标准化实盘流水（按扩展名仅支持 csv/json） [已实现]
│   ├── sync                # 同步引用数据（分类标准仍受限） [部分实现]
│   │   └── industry        # 同步行业分类（当前仅支持 shenwan） [部分实现]
│   ├── rebuild             # 重建账户 [已实现]
│   │   └── live-account    # 重建实盘镜像账户 [已实现]
│   ├── rule                # 规则管理 [已实现]
│   │   ├── set             # 设置规则 [已实现]
│   │   ├── list            # 列出规则 [已实现]
│   │   ├── enable          # 启用规则 [已实现]
│   │   └── disable         # 禁用规则 [已实现]
│   ├── log                 # 风控日志 [已实现]
│   ├── lock                # 买入锁管理 [已实现]
│   │   └── release         # 释放买入锁 [已实现]
│   ├── status              # 风控状态 [已实现]
│   ├── pnl                 # 盈亏快照 [已实现]
│   └── position            # 持仓风险 [已实现]
│
├── execution               # 执行管理（live 需走 qmt_live + bridge） [已实现]
│   ├── config              # 执行配置 [已实现]
│   │   ├── init            # 初始化执行配置 [已实现]
│   │   └── show            # 显示执行配置 [已实现]
│   ├── daemon              # 执行守护进程（消费 paper/mock_live request） [已实现]
│   │   └── run             # 运行执行守护进程（qmt_live 需改走 bridge/qmt） [已实现]
│   ├── bridge              # Bridge 诊断 [已实现]
│       ├── status          # 状态检查 [已实现]
│       │   └── --checklist # 追加 QMT promotion checklist [已实现]
│       ├── qmt-preview     # QMT 预览（仅 qmt_live request） [已实现]
│       ├── qmt-live        # QMT 真实提交（仅 qmt_live request） [已实现]
│       │   └── --yes       # 跳过确认提示 [已实现]
│       ├── qmt-query       # 查询订单状态 [已实现]
│       ├── qmt-cancel      # 撤销订单 [已实现]
│       ├── qmt-account     # 查询账户状态 [已实现]
│       ├── qmt-positions   # 查询持仓 [已实现]
│       └── qmt-asset       # 查询资产 [已实现]
│   └── qmt                 # QMT 兼容入口 [已实现]
│       ├── status          # 状态检查 [已实现]
│       │   └── --checklist # 追加 QMT promotion checklist [已实现]
│       ├── preview         # QMT 预览（仅 qmt_live request） [已实现]
│       ├── live            # QMT 真实提交（仅 qmt_live request） [已实现]
│       │   └── --yes       # 跳过确认提示 [已实现]
│       ├── query           # 查询订单状态 [已实现]
│       ├── cancel          # 撤销订单 [已实现]
│       ├── account         # 查询账户状态 [已实现]
│       ├── positions       # 查询持仓 [已实现]
│       └── asset           # 查询资产 [已实现]
│
├── anomaly                 # 异常检测 [已实现]
│   └── run                 # 运行检测 [已实现]
│       ├── --top-n         # 显示数量 [已实现]
│       ├── --period        # K线周期 [已实现]
│       ├── --output        # 输出格式 [已实现]
│       └── --mock          # 模拟数据 [已实现]
│
├── account                 # 账户管理 [已实现]
│   ├── register            # 注册新账户（account_type: paper/mock_live/qmt_live；live 为兼容别名） [已实现]
│   ├── list                # 列出所有账户（可按 account_type 过滤） [已实现]
│   ├── show                # 查看账户详情 [已实现]
│   ├── update              # 更新账户配置 [已实现]
│   ├── remove              # 删除账户 [已实现]
│   ├── default             # 设置默认账户 [已实现]
│   ├── summary             # 资金聚合视图（仅聚合启用账户） [已实现]
│   ├── split               # 订单拆分预览（target_type: single/group） [已实现]
│   └── group               # 账户组管理 [已实现]
│       ├── create          # 创建账户组（weighted 当前退化为 equal） [已实现]
│       ├── list            # 列出账户组 [已实现]
│       ├── show            # 查看组详情 [已实现]
│       ├── remove          # 删除账户组 [已实现]
│       ├── add-account     # 添加账户到组 [已实现]
│       ├── remove-account  # 从组移除账户 [已实现]
│       └── set-strategy    # 设置分配策略（weighted 当前退化为 equal；primary_first 需 --primary-account） [已实现]
│
├── algo                    # 算法交易（当前仅 TWAP/VWAP，且任务为单进程内存态） [部分实现]
│   ├── create              # 创建算法任务（当前仅支持 TWAP/VWAP） [部分实现]
│   ├── start               # 启动算法任务（当前仅支持 TWAP/VWAP 内存态任务） [部分实现]
│   ├── pause               # 暂停算法任务（当前仅支持 TWAP/VWAP 内存态任务） [部分实现]
│   ├── resume              # 恢复算法任务（当前仅支持 TWAP/VWAP 内存态任务） [部分实现]
│   ├── cancel              # 取消算法任务（当前仅支持 TWAP/VWAP 内存态任务） [部分实现]
│   ├── status              # 查看算法状态（当前仅支持 TWAP/VWAP 内存态任务） [部分实现]
│   ├── list                # 列出活跃算法（仅当前进程内存态任务） [部分实现]
│   └── plan                # 预览切片计划（当前仅支持 TWAP/VWAP） [已实现]
│
├── ai                      # AI决策分析 [部分实现]
│   ├── analyze             # AI 分析股票（使用模拟分析上下文） [部分实现]
│   ├── decide              # AI 交易决策（基于模拟分析上下文） [部分实现]
│   ├── ask                 # 对话式分析（provider 自动选择） [部分实现]
│   ├── market              # 市场整体分析（固定 prompt） [部分实现]
│   └── config              # AI配置管理 [部分实现]
│       ├── --show          # 显示当前配置 [已实现]
│       └── --test          # 测试连通性（当前仅配置探测） [部分实现]
│
├── news                    # 新闻搜索 [部分实现]
│   ├── search              # 搜索股票新闻（当前仅provider检查/占位输出） [部分实现]
│   │   ├── --code         # 按股票代码 [部分实现]
│   │   ├── --query        # 按关键词 [部分实现]
│   │   ├── --provider     # 指定数据源 [部分实现]
│   │   └── --days         # 时间范围 [部分实现]
│   ├── code                # 按股票代码搜索（复用占位搜索流程） [部分实现]
│   ├── trend               # 市场热点趋势（占位提示） [部分实现]
│   └── providers           # 提供商状态 [已实现]
│
├── fundamental             # 基本面分析 [部分实现]
│   ├── show                # 综合基本面 (EastMoney数据源) [已实现]
│   ├── valuation           # 估值指标 PE/PB/PS/市值/ROE/EPS [已实现]
│   ├── earnings            # 财报数据（最新一季已实现，history 退化） [部分实现]
│   │   └── --years         # 历史年数（>1 时仍退化为单条最新一季） [部分实现]
│   ├── institution         # 机构持仓 + 类型映射 + 变动方向 [已实现]
│   ├── capital-flow        # 资金流向（待接入 CLI 子命令） [待实现]
│   ├── dragon-tiger        # 龙虎榜汇总（营业部明细未填） [部分实现]
│   └── dividend            # 分红信息（当前仅开发中提示） [未实现]
│       └── --years         # 历史年数 [未实现]
│
├── sentiment               # 舆情分析 [部分实现]
│   ├── show                # 查看舆情情绪（默认无provider） [部分实现]
│   ├── history             # 查看历史趋势（默认无provider） [部分实现]
│   └── mentions            # 查看社交媒体提及（默认无provider） [部分实现]
│
├── notify                  # 多渠道通知（仅部分渠道真正接线） [部分实现]
│   ├── send                # 发送通知（已接线渠道可用） [部分实现]
│   │   ├── --channel      # 指定渠道（部分枚举仍为预留入口） [部分实现]
│   │   ├── --title        # 标题 [已实现]
│   │   ├── --message      # 消息内容 [已实现]
│   │   └── --level        # 消息级别 [已实现]
│   ├── test                # 测试通知渠道（当前未按 channel 精确过滤） [部分实现]
│   ├── list                # 列出渠道（包含待实现渠道） [部分实现]
│   └── check               # 测试渠道连通性（仅部分渠道可真实发送） [部分实现]
│
└── import                  # 智能导入 [部分实现] [可选 - Phase 5]
    ├── from-image          # 图片识别导入 [已实现]
    ├── from-csv            # CSV文件导入 [已实现]
    ├── from-excel          # Excel文件导入 [未实现]
    ├── from-clipboard      # 剪贴板导入 [已实现]
    ├── from-text           # 文本导入 [已实现]
    └── resolve             # 代码/名称解析 [已实现]
```

## 模块依赖关系

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                            │
│                      (cli/handlers)                          │
└──────────────────────────┬──────────────────────────────────┘
                           │
       ┌───────────────────┼───────────────────┬───────────────────┐
       │                   │                   │                   │
       ▼                   ▼                   ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   策略层     │    │   执行层     │    │   风控层     │    │ 新增: AI层  │
│  strategy   │    │  execution  │    │    risk     │    │     ai      │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                   │                   │
       └──────────────────┼───────────────────┘                   │
                          │                                       │
       ┌──────────────────┼───────────────────┬───────────────────┘
       │                  │                   │
       ▼                  ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   数据源     │    │   分析层     │    │   监控层     │
│  sources    │    │  analysis   │    │ monitoring  │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                   │
       └──────────────────┼───────────────────┘
                          │
       ┌──────────────────┼───────────────────┐
       │                  │                   │
       ▼                  ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ 新增: 新闻   │    │新增: 基本面  │    │扩展: 通知   │
│    news     │    │fundamental  │    │notification │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                   │
       └──────────────────┼───────────────────┘
                          │
                          ▼
               ┌─────────────────────┐
               │      数据层          │
               │   data / db / io    │
               └──────────┬──────────┘
                          │
                          ▼
               ┌─────────────────────┐
               │      核心层          │
               │        core         │
               └─────────────────────┘
```

### 新增模块依赖说明

| 模块 | 依赖 | 说明 |
|------|------|------|
| ai | core, monitoring/notification | LLM适配需要通知能力 |
| news | core, sources | 新闻获取依赖数据源 |
| fundamental | core, sources, market | 基本面依赖市场数据 |
| notification扩展 | core | 扩展现有通知模块 |

## 更新日志

- **2026-05-05**: 继续按当前主线源码细化 `FUNCTION_TREE.md`，补充 CLI 摘要层限制说明，修正 `fundamental dividend` 为开发中占位，标明 `fundamental capital-flow` 尚未接入 CLI 子命令，补充 `strategy daemon`/`algo`/`market`/`screener`/`monitor`/`trade`/`account` 的参数与运行边界，并同步 Graphiti MCP 的稳定 `group_id` 命名
- **2026-05-03**: 按当前主线源码核对 `FUNCTION_TREE.md`，统一补充 `已实现 / 部分实现 / 未实现 / 非目标 / 待实现` 状态标记，并修正文档与实际命令/模块不一致项
- **2026-05-02**: 根目录 `FUNCTION_TREE.md` 升级为唯一 canonical 功能树文档，并通过 PR #62 并入 `master`，当前主线基线为 `origin/master@562fe84`
- **2026-03-27 (续2)**: 添加 Graphiti MCP 集成、多账户管理、算法交易执行器
- **2026-03-27**: 添加 daily_stock_analysis 迁移计划（AI/News/Fundamental/Notification扩展）
- **2026-03-27**: 合并 FUNCTION_MAP.md，增加设计边界和运营视图
- **2026-03-27**: 初版创建，记录当前项目功能结构

---

## 外部集成

### Graphiti MCP 语义记忆层

Graphiti MCP 提供语义记忆能力，用于设计决策、代码审查、调试、交接和文档。

| Group ID | 用途 |
|----------|------|
| `quantix_rust_main` | 主设计决策和架构记录 |
| `quantix_rust_review` | 代码审查记录 |
| `quantix_rust_debug` | 调试会话记录 |
| `quantix_rust_handoff` | 交接文档 |
| `quantix_rust_docs` | 技术文档 |

**MCP 配置**:
```json
{
  "graphiti-memory": {
    "type": "sse",
    "url": "http://192.168.123.104:8011/mcp",
    "description": "Graphiti semantic memory layer"
  }
}
```

---

## 附录 A: 架构设计边界

### A.1 核心架构原则

Quantix-Rust 围绕五个稳定中心设计：

1. **数据采集与存储** - 数据层面的核心
2. **策略生成与执行编排** - 策略层面的核心
3. **交易、风控、监控、止损运营工作流** - 运营层面的核心
4. **市场/选股/分析决策支持** - 分析层面的核心
5. **Windows Bridge v1 跨平台集成** - 桥接层面的核心

### A.2 关键边界

- `quantix-rust` 在 WSL2 中拥有运行时状态、执行请求、执行内核编排、paper/mock-live 执行和本地审计存储
- Windows 端 bridge 工作是外部能力边界，不是第二个运行时状态机
- Bridge 路径: `/mnt/d/mystocks/quantix/quantix_bridge`

### A.3 已完成的设计决策

| 决策 | 说明 |
|------|------|
| `runtime.db` | 执行审计存储 |
| `execution_request` | 审批与执行之间的持久化传递对象 |
| 冻结快照 | 防止请求意图漂移 |
| `paper` + `mock_live` + guarded `qmt_live` | 当前已实现的执行目标 |
| `live` | 通用 live 语义故意保持不完整，不等于 `qmt_live` |
| Bridge 不拥有执行状态 | Windows 端无状态 |
| `QMT` | Bridge v1 已支持受能力门控的 `qmt_live` 真实提交通道（task receipt/result + task-id based cancel routing 语义），同时保留 preview 路径 |
| `TDX bridge source` | 第一个真正的 bridge 能力 |

---

## 附录 B: 当前非目标

以下明确不在已完成的功能设计范围内：

- [非目标] 真实 live broker 执行
- [非目标] Windows 端拥有运行时状态
- [非目标] Wind / Choice bridge 集成
- [非目标] Bridge 端 WebSocket / gRPC 栈
- [非目标] 分布式 worker 或多进程执行守护进程协调

---

## 附录 C: 运营视图

### C.1 当前可用功能

```
当前可用
├── [已实现] 本地数据/研究/选股
├── [已实现] 本地 paper 交易
├── [部分实现] 本地风控/监控/止损工作流
├── [已实现] 策略 paper + mock_live + 请求生命周期
├── [已实现] 执行守护进程消费
├── [已实现] 多账户管理 (账户组、资金聚合、智能拆单)
└── [已实现] Windows Bridge v1
    ├── [已实现] TDX bridge 数据源
    ├── [已实现] QMT 预览
    └── [已实现] guarded `qmt_live` 真实提交 + task receipt/result query 路径
```

### C.2 故意推迟的功能

```
故意推迟
├── [非目标] 真实 live 适配器
├── [非目标] Wind / Choice bridge 支持
├── [非目标] Bridge 拥有的订单生命周期
└── [非目标] 更广泛的分布式运行时问题
```

---

## 附录 D: Windows Bridge v1 功能树

```
Windows Bridge v1 [已实现]
├── Rust 端 [已实现]
│   ├── src/bridge/client.rs [已实现]
│   ├── src/bridge/models.rs [已实现]
│   ├── src/sources/bridge_tdx.rs [已实现]
│   ├── src/watchlist/resolver.rs (BridgeTdxWatchlistQuoteLookup) [已实现]
│   ├── src/execution/qmt_bridge.rs [已实现]
│   ├── src/execution/qmt_live_gate.rs [已实现]
│   ├── src/execution/qmt_task_submit_service.rs [已实现]
│   ├── src/execution/qmt_live_adapter.rs [已实现]
│   ├── src/execution/request_diagnostics.rs [已实现]
│   └── quantix execution bridge ... [已实现]
│
└── Windows 端 [已实现]
    └── /mnt/d/mystocks/quantix/quantix_bridge [已实现]
        ├── /health [已实现]
        ├── /api/v1/capabilities [已实现]
        ├── /api/v1/data/tdx/quotes [已实现]
        ├── /api/v1/data/tdx/kline/{symbol} [已实现]
        ├── /api/v1/task/execute [已实现]
        ├── /api/v1/task/result/{task_id} [已实现]
        ├── /api/v1/broker/qmt/account/status [已实现]
        ├── /api/v1/broker/qmt/account/asset [已实现]
        ├── /api/v1/broker/qmt/positions [已实现]
        ├── /api/v1/broker/qmt/orders/preview [已实现]
        ├── /api/v1/broker/qmt/orders [已实现]   # Bridge 真实提交端点；Rust 侧仍受 `qmt_live` gate 约束
        └── /api/v1/broker/qmt/orders/{order_id} [已实现]   # 查询 / 撤单
```
