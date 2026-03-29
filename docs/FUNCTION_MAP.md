# Quantix Rust 功能树

本文档以目录树形式展示 Quantix 量化交易系统的功能层次结构。

```
quantix-rust/
├── 📦 核心层 (core/)
│   ├── 配置管理 (config)
│   │   ├── QuantixConfig - 全局配置结构
│   │   └── 运行时配置加载
│   ├── 错误处理 (error)
│   │   ├── QuantixError - 统一错误类型
│   │   └── Result<T> - 结果类型别名
│   ├── 运行时环境 (runtime)
│   │   └── RuntimeContext - 异步运行时上下文
│   ├── 交易日历 (trading_calendar)
│   │   ├── TradingCalendar - A股交易日历
│   │   ├── 交易时段判断 (Morning/Afternoon/Auction/Closed)
│   │   ├── 节假日加载 (JSON配置)
│   │   └── 调休工作日支持
│   └── 交易时间工具 (trading_time)
│       └── 交易时段计算
│
├── 📊 数据层 (data/)
│   ├── 数据获取 (fetcher)
│   │   └── 行情数据获取接口
│   ├── 数据模型 (models)
│   │   └── OHLCV、K线等数据结构
│   └── 数据存储 (storage)
│       └── 本地数据持久化
│
├── 🔌 数据源 (sources/)
│   ├── AKShare 数据源 (akshare)
│   │   └── Python AKShare 库封装
│   ├── 通达信数据源 (tdx)
│   │   └── TDX 本地数据读取
│   ├── 通达信文件读取 (tdx_file)
│   │   └── day/lc1/lc5 文件解析
│   ├── Bridge TDX 数据源 (bridge_tdx)
│   │   └── 通过 Windows Bridge 获取 TDX 数据
│   ├── 东方财富数据源 (eastmoney)
│   │   └── 东财 API 行情获取
│   ├── WebSocket 行情 (websocket)
│   │   └── 实时行情推送
│   ├── 行情采集器
│   │   ├── QuoteCollector - 通用行情采集
│   │   ├── AuctionCollector - 竞价数据采集
│   │   └── KlineAggregator - K线聚合
│   └── 缠论数据适配 (待实现)
│
├── 📈 策略层 (strategy/)
│   ├── 策略定义 (trait_def)
│   │   ├── Strategy trait - 策略接口
│   │   └── Signal 类型定义
│   ├── 内置策略 (strategies/)
│   │   ├── 突破策略 (breakout)
│   │   ├── 网格策略 (grid)
│   │   ├── 均线交叉 (ma_cross)
│   │   ├── 均值回归 (mean_reversion)
│   │   └── 动量策略 (momentum)
│   ├── 策略注册 (registry)
│   │   └── 策略注册表管理
│   ├── 策略运行时 (runtime)
│   │   ├── 策略实例管理
│   │   └── 信号生成与存储
│   ├── 策略守护进程 (daemon)
│   │   └── 定时策略执行
│   └── 策略服务 (systemd)
│       ├── 服务安装/卸载
│       ├── 启动/停止
│       └── 开机自启
│
├── ⚡ 执行层 (execution/)
│   ├── 执行核心 (kernel)
│   │   └── ExecutionKernel - 执行决策核心
│   ├── 执行请求 (models)
│   │   ├── ExecutionRequest - 执行请求结构
│   │   └── FrozenExecutionSnapshot - 冻结快照
│   ├── Paper 交易 (paper)
│   │   └── 模拟撮合引擎
│   ├── Mock Live 模式 (mock_live)
│   │   └── 伪实盘模式（手动确认）
│   ├── 运行时存储 (runtime_store)
│   │   └── SQLite 运行状态持久化
│   ├── 订单对账 (reconciliation)
│   │   ├── OpenOrderScanner - 未完成订单扫描
│   │   ├── ReconciliationService - 对账服务
│   │   ├── Unknown 状态自动恢复
│   │   └── 超时订单自动标记失败
│   ├── QMT Bridge (qmt_bridge)
│   │   └── QMT 预览请求
│   ├── 执行适配器 (adapter)
│   │   └── 多 broker 适配接口
│   ├── 执行守护进程 (daemon)
│   │   └── 执行服务后台运行
│   └── 执行配置 (config)
│       └── 执行参数配置
│
├── 👥 账户管理 (account/)
│   ├── 账户模型 (models)
│   │   ├── AccountConfig - 账户配置
│   │   ├── AccountType - 账户类型 (Paper/Live/MockLive)
│   │   ├── AccountGroup - 账户组
│   │   ├── AllocationStrategy - 分配策略
│   │   │   ├── Equal - 平均分配
│   │   │   ├── Proportional - 按资金比例
│   │   │   ├── Weighted - 自定义权重
│   │   │   └── PrimaryFirst - 主账户优先
│   │   ├── OrderSplitRequest - 订单拆分请求
│   │   ├── SplitTarget - 拆分目标 (Single/Group)
│   │   └── OrderSplitResult - 拆分结果
│   ├── 账户注册表 (registry)
│   │   ├── AccountRegistry - 账户注册表
│   │   ├── 账户 CRUD 操作
│   │   ├── 账户组 CRUD 操作
│   │   └── 默认账户管理
│   ├── 智能路由 (router)
│   │   ├── AccountRouter - 账户路由器
│   │   ├── 订单拆分逻辑
│   │   └── 多账户分配
│   └── 账户存储 (storage)
│       ├── JsonAccountRegistryStore - JSON存储
│       └── ~/.quantix/accounts/registry.json
│
├── 🛡️ 风控层 (risk/)
│   ├── 风控模型 (models)
│   │   ├── RiskRule - 风控规则
│   │   ├── RiskRuleType - 规则类型
│   │   │   ├── PositionLimit - 持仓限制
│   │   │   ├── DailyLossLimit - 日内亏损限制
│   │   │   ├── VolatilityLimit - 波动率限制
│   │   │   ├── IndustryLimit - 行业集中度限制
│   │   │   └── AutoReduce - 自动减仓
│   │   ├── RiskState - 风控状态
│   │   └── RiskAccountSnapshot - 账户快照
│   ├── 风控服务 (service)
│   │   ├── RiskService - 风控服务核心
│   │   ├── 买入前风控检查
│   │   ├── 行业集中度检查
│   │   └── 自动减仓触发检测
│   ├── 风控存储 (storage)
│   │   └── SQLite 风控数据持久化
│   ├── 实盘流水导入 (importer)
│   │   └── 标准化流水导入
│   ├── 导入存储 (import_store)
│   │   └── 导入数据缓存
│   ├── 账户重建 (rebuild)
│   │   └── 实盘镜像账户重建
│   └── 波动率计算 (volatility)
│       └── 历史波动率计算
│
├── 📉 监控层 (monitoring/)
│   ├── 告警系统 (alert)
│   │   ├── AlertManager - 告警管理器
│   │   ├── AlertThreshold - 阈值配置
│   │   ├── AlertLevel (Info/Warning/Error/Critical)
│   │   └── AlertType (Signal/Position/Performance/Risk/System)
│   ├── 健康检查 (health)
│   │   ├── HealthRegistry - 健康检查注册表
│   │   ├── ComponentHealth - 组件健康状态
│   │   └── SystemHealth - 系统整体健康报告
│   ├── 指标收集 (metrics)
│   │   ├── MetricsCollector - 指标收集器
│   │   ├── Counter/Gauge/Histogram 类型
│   │   └── MetricsExporter - 指标导出 (Prometheus/JSON)
│   ├── 通知系统 (notification)
│   │   ├── NotificationService - 通知服务
│   │   ├── DesktopSender - 桌面通知 (Linux/Windows)
│   │   ├── WebhookSender - HTTP POST 通知
│   │   ├── LogSender - 日志文件通知
│   │   └── QuietHours - 静默时段配置
│   ├── 信号监控 (signal_monitor)
│   │   └── 策略信号实时追踪
│   ├── 持仓监控 (position_monitor)
│   │   ├── PositionMonitor - 持仓状态监控
│   │   └── PositionSnapshot - 持仓快照
│   └── 性能监控 (performance_monitor)
│       ├── PerformanceMonitor - 实时性能监控
│       └── RealtimeMetrics - 实时指标计算
│
├── 📋 自选池 (watchlist/)
│   ├── 自选模型 (models)
│   │   ├── WatchlistEntry - 自选条目
│   │   └── WatchlistHistoryEvent - 历史事件
│   ├── 行情解析 (resolver)
│   │   ├── WatchlistResolver - 行情数据解析
│   │   ├── TdxWatchlistQuoteLookup - TDX行情查询
│   │   ├── BridgeTdxWatchlistQuoteLookup - Bridge行情查询
│   │   └── PostgresWatchlistNameLookup - 名称查询
│   ├── 自选服务 (service)
│   │   └── WatchlistService - 自选池管理
│   └── 自选存储 (storage)
│       └── WatchlistStorage - 持久化存储
│
├── 🛑 止盈止损 (stop/)
│   ├── 止损模型 (models)
│   │   ├── StopRule - 止损规则
│   │   ├── StopTriggerKind - 触发类型
│   │   │   ├── Fixed - 固定价格
│   │   │   ├── Percentage - 百分比
│   │   │   └── Trailing - 跟踪止损
│   │   └── StopHistoryEvent - 历史事件
│   ├── 止损服务 (service)
│   │   ├── StopService - 止损服务
│   │   └── 实时止损评估
│   └── 止损存储 (storage)
│       └── SqliteStopRuleStore - SQLite存储
│
├── 💰 模拟交易 (trade/)
│   ├── 交易模型 (models)
│   │   ├── PaperTradeAccount - 模拟账户
│   │   ├── TradeRecord - 交易记录
│   │   ├── TradePosition - 持仓
│   │   └── TradeSide - 买卖方向
│   ├── 费用计算 (fees)
│   │   ├── FeeConfig - 费用配置
│   │   ├── FeeBreakdown - 费用明细
│   │   └── calculate_fee_breakdown - 费用计算
│   ├── 交易服务 (service)
│   │   ├── TradeService - 交易服务
│   │   └── PaperTradeStore - 模拟账户存储
│   ├── 报告服务 (reporting)
│   │   └── TradeReportingService - 交易报告
│   └── 交易存储 (storage)
│       └── JsonPaperTradeStore - JSON存储
│
├── 🔍 市场分析 (market/)
│   ├── 市场模型 (models)
│   │   ├── MarketOverview - 市场概览
│   │   ├── BoardRankRow - 板块排名
│   │   ├── LeaderRow - 龙头股
│   │   ├── MarketSentimentSnapshot - 市场情绪
│   │   └── NorthFlowSnapshot - 北向资金
│   └── 市场服务 (service)
│       ├── MarketService - 市场服务
│       ├── MarketDataReader - 数据读取
│       ├── 行业板块分析
│       ├── 概念板块分析
│       ├── 龙头股识别
│       └── 北向资金分析
│
├── 🎯 选股器 (screener/)
│   ├── 选股模型 (models)
│   │   ├── ScreenRow - 筛选结果
│   │   ├── ScreenRunOptions - 运行选项
│   │   ├── PresetInvocation - 预设调用
│   │   └── ScreenUniverse - 股票池
│   ├── 条件解析 (parser)
│   │   └── parse_preset_invocation - 条件解析
│   ├── 条件评估 (evaluator)
│   │   ├── evaluate_preset - 条件评估
│   │   └── required_lookback - 回溯计算
│   └── 选股服务 (service)
│       ├── ScreenerService - 选股服务
│       └── DailyKlineLoader - 日线加载
│
├── 📐 技术分析 (analysis/)
│   ├── 技术指标 (indicators)
│   │   ├── MA/EMA/SMA 均线
│   │   ├── MACD
│   │   ├── RSI
│   │   ├── BOLL 布林带
│   │   ├── KDJ
│   │   └── 更多指标...
│   ├── K线形态 (candle_patterns)
│   │   └── K线形态识别
│   ├── 回测引擎 (backtest)
│   │   ├── BacktestEngine - 回测引擎
│   │   └── BacktestResult - 回测结果
│   ├── 竞价分析 (auction)
│   │   ├── AuctionAnalyzer - 竞价分析器
│   │   ├── SectorStats - 板块统计
│   │   └── StrengthLevel - 强度等级
│   ├── 性能计算 (performance)
│   │   ├── PerformanceCalculator - 性能计算
│   │   └── PerformanceReport - 性能报告
│   ├── 投资组合 (portfolio)
│   │   ├── Portfolio - 投资组合
│   │   ├── Position - 持仓
│   │   └── Order - 订单
│   └── Polars 适配 (polars_adapter)
│       ├── PolarsCalculator - Polars计算
│       └── 批量K线数据处理
│
├── 🎲 异常检测 (anomaly/)
│   ├── Isolation Forest (forest)
│   │   └── IsolationForest - 隔离森林算法
│   ├── 统计函数 (statistics)
│   │   └── 平均路径长度计算
│   ├── 特征提取 (features)
│   │   ├── FeatureExtractor - 特征提取器
│   │   ├── volume returns - 成交量回报
│   │   ├── log returns - 对数回报
│   │   └── EOM 指标
│   ├── A股过滤器 (filter)
│   │   ├── StockFilter - 股票过滤器
│   │   ├── ST股票过滤
│   │   ├── 涨跌停过滤
│   │   ├── 停牌过滤
│   │   └── 新股过滤
│   ├── 检测服务 (detector)
│   │   ├── AnomalyDetector - 异常检测器
│   │   └── AnomalyResult - 检测结果
│   ├── 东方财富数据源 (eastmoney_source)
│   │   └── EastMoneyAnomalySource - 东财数据源
│   └── 配置管理 (config)
│       ├── AnomalyConfig - 检测配置
│       ├── ForestConfig - 森林配置
│       └── FilterConfig - 过滤配置
│
├── 🖥️ 监控服务 (monitor/)
│   ├── 监控配置 (config)
│   │   └── MonitorConfig - 监控配置
│   ├── 监控模型 (models)
│   │   ├── PriceAlert - 价格告警
│   │   ├── MonitorEventRow - 监控事件
│   │   └── TriggeredAlert - 触发告警
│   ├── 监控运行器 (runner)
│   │   └── MonitorRunner - 监控运行器
│   ├── 监控服务 (service)
│   │   └── MonitorService - 监控服务
│   ├── 服务配置 (service_config)
│   │   └── MonitorServiceConfig - 服务配置
│   ├── 监控存储 (storage)
│   │   └── SqliteMonitorAlertStore - SQLite存储
│   └── Systemd 服务 (systemd)
│       └── MonitorUserServiceInstaller - 用户服务安装
│
├── 🌉 Windows Bridge (bridge/)
│   ├── HTTP 客户端 (client)
│   │   └── Bridge HTTP 客户端
│   ├── 数据模型 (models)
│   │   └── Bridge 请求/响应模型
│   └── 错误处理 (error)
│       └── Bridge 错误类型
│
├── 💾 数据库层 (db/)
│   ├── ClickHouse (clickhouse)
│   │   ├── ClickHouseClient - 客户端
│   │   ├── KlineDataCH - K线数据
│   │   ├── StockInfoCH - 股票信息
│   │   ├── StockQuoteCH - 实时行情
│   │   └── LimitUpEventCH - 涨停事件
│   ├── PostgreSQL (postgresql)
│   │   ├── PostgresClient - 客户端
│   │   ├── KlineDaily - 日线数据
│   │   └── StockInfo - 股票信息
│   └── TDengine (tdengine)
│       ├── TDengineClient - 客户端
│       └── MinuteKline - 分钟K线
│
├── 📥 数据导入导出 (io/)
│   ├── 导出器 (exporter)
│   │   ├── DataExporter - 数据导出器
│   │   ├── CSV/JSON/Parquet 格式
│   │   └── ExportResult - 导出结果
│   ├── 导入器 (importer)
│   │   ├── DataImporter - 数据导入器
│   │   └── 多格式数据导入
│   ├── 数据验证 (validation)
│   │   ├── DataValidator - 数据验证器
│   │   └── ValidationResult - 验证结果
│   └── 批处理 (batch)
│       ├── BatchProcessor - 批量处理器
│       └── BatchProgress - 处理进度
│
├── 🔄 数据同步 (sync/)
│   └── ETL 同步 (etl)
│       ├── DataSync - 数据同步
│       ├── SyncConfig - 同步配置
│       └── SyncStats - 同步统计
│
├── ⏰ 任务调度 (tasks/)
│   ├── 任务调度器 (scheduler)
│   │   ├── TaskScheduler - 任务调度器
│   │   ├── ScheduledTask - 调度任务
│   │   └── TaskTemplates - 任务模板
│   ├── 采集调度 (collect_scheduler)
│   │   ├── CollectScheduler - 采集调度器
│   │   └── SchedulerConfig - 调度配置
│   └── Cron 表达式 (cron)
│       └── CronExpression - Cron解析
│
├── 🤖 AI决策层 (ai/) [✅ Phase 2 已完成]
│   ├── LLM适配器 (llm_adapter)
│   │   └── OpenAI协议统一适配
│   ├── 多模型支持 (providers)
│   │   ├── OpenAI - GPT-4o系列
│   │   ├── DeepSeek - DeepSeek-Chat/Reasoner
│   │   ├── Gemini - Google Gemini 2.5
│   │   ├── Anthropic - Claude 3.5
│   │   └── Ollama - 本地模型
│   ├── Prompt模板 (prompt_templates)
│   │   └── Tera模板引擎
│   ├── 决策引擎 (decision_engine)
│   │   └── 决策仪表盘生成
│   ├── 对话管理 (conversation)
│   │   └── 多轮对话上下文
│   └── 技能注册 (skill_registry)
│       └── 策略技能包管理
│
├── 📰 新闻搜索层 (news/) [✅ Phase 3 已完成]
│   ├── 新闻提供者 (provider)
│   │   └── NewsProvider trait
│   ├── 多源支持 (providers)
│   │   ├── Tavily - 高质量AI友好
│   │   ├── SerpAPI - 全渠道搜索
│   │   ├── 博查搜索 - 中文优化
│   │   ├── Brave - 隐私优先
│   │   └── SearXNG - 自建实例
│   ├── 新闻聚合 (aggregator)
│   │   └── 多源fallback机制
│   └── 新闻缓存 (cache)
│       └── 本地缓存存储
│
├── 📊 基本面分析 (fundamental/) [✅ API解析已实现 - Phase 4]
│   ├── 基本面提供者 (provider)
│   │   └── FundamentalProvider trait
│   ├── EastMoney数据源 (eastmoney)
│   │   └── EastMoneyFundamentalProvider
│   ├── 估值指标 (valuation) ✅ EastMoney push2 API
│   │   └── PE/PB/PS/市值/ROE/EPS
│   ├── 财报数据 (earnings) ✅ EastMoney push2 API
│   │   └── 营收/净利润/毛利率
│   ├── 机构持仓 (institution) ✅ EastMoney stockholder API
│   │   └── 基金/机构持仓 + 类型映射
│   ├── 资金流向 (capital_flow)
│   │   └── 主力资金追踪
│   ├── 龙虎榜 (dragon_tiger) ✅ EastMoney DataCenter API
│   │   └── 游资/机构买卖
│   └── 分红信息 (dividend)
│       └── 历史分红记录
│
├── 📥 智能导入 (import/) [📋 可选 - Phase 5]
│   ├── 图片提取 (image_extractor)
│   │   └── LLM Vision识别
│   ├── CSV解析 (csv_parser)
│   ├── Excel解析 (excel_parser)
│   ├── 剪贴板解析 (clipboard)
│   └── 代码解析器 (code_resolver)
│
├── 🖼️ TUI 界面 (tui/)
│   └── 应用 (app)
│       └── run_menu - 交互式菜单
│
└── 📟 CLI 命令 (cli/)
    └── 命令处理器 (handlers)
        ├── init - 初始化配置
        ├── menu - 交互式菜单
        ├── data - 数据查询/导出
        ├── strategy - 策略管理/运行
        ├── task - 任务调度
        ├── analyze - 技术分析/回测
        ├── monitor - 自选监控/告警
        ├── stop - 止盈止损管理
        ├── watchlist - 自选池管理
        ├── market - 市场分析
        ├── trade - 模拟交易
        ├── risk - 风控管理
        ├── execution - 执行守护进程
        ├── anomaly - 异常检测
        └── status - 系统状态
```

## CLI 命令树

```
quantix
├── init                    # 初始化配置和数据库
├── menu                    # 交互式菜单
│   └── --tui               # TUI 界面模式
├── status                  # 系统状态
│   └── --health            # 健康检查
│
├── data                    # 数据命令
│   ├── query               # 查询历史数据
│   └── export              # 导出数据
│
├── strategy                # 策略命令
│   ├── run                 # 运行策略
│   ├── list                # 列出策略
│   ├── show                # 显示详情
│   ├── config              # 配置管理
│   ├── daemon              # 守护进程
│   ├── signal              # 信号管理
│   │   ├── list            # 列出信号
│   │   ├── approve         # 批准信号
│   │   └── reject          # 拒绝信号
│   ├── request             # 执行请求
│   │   ├── list            # 列出请求
│   │   │   ├── --status    # 按状态过滤
│   │   │   ├── --target-mode # 按执行模式过滤
│   │   │   ├── --target-account # 按目标账户过滤
│   │   │   ├── --stats     # 统计汇总视图
│   │   │   └── --verbose   # 详细输出
│   │   ├── show            # 查看请求详情
│   │   │   ├── --request-id # 请求ID
│   │   │   └── --verbose   # 故障排查信息
│   │   ├── execute         # 执行请求
│   │   └── cancel          # 取消请求
│   └── service             # systemd 服务
│
├── task                    # 任务命令
│   ├── add                 # 添加定时任务
│   ├── list                # 列出任务模板
│   ├── start               # 启动调度器
│   └── stop                # 停止调度器
│
├── analyze                 # 分析命令
│   ├── indicators          # 计算技术指标
│   ├── backtest            # 回测报告
│   ├── candle-pattern      # K线形态识别
│   └── screener            # 选股筛选
│       ├── preset-list     # 预设条件列表
│       └── run             # 运行筛选
│
├── monitor                 # 监控命令
│   ├── watchlist           # 自选监控
│   ├── alert               # 价格告警
│   │   ├── add             # 添加告警
│   │   ├── list            # 列出告警
│   │   └── remove          # 删除告警
│   ├── config              # 监控配置
│   ├── daemon              # 守护进程
│   ├── service             # systemd 服务
│   └── event               # 事件历史
│
├── stop                    # 止盈止损
│   ├── set                 # 设置规则
│   ├── update              # 更新规则
│   ├── list                # 列出规则
│   ├── status              # 查看状态
│   ├── history             # 历史记录
│   └── remove              # 删除规则
│
├── watchlist               # 自选池
│   ├── add                 # 添加股票
│   ├── remove              # 移除股票
│   ├── list                # 列出自选
│   ├── move                # 移动分组
│   ├── group               # 分组管理
│   ├── tag                 # 标签管理
│   └── history             # 历史记录
│
├── market                  # 市场分析
│   ├── sector              # 行业板块
│   ├── concept             # 概念板块
│   ├── north               # 北向资金
│   ├── sentiment           # 市场情绪
│   ├── leader              # 龙头股
│   └── overview            # 综合概览
│
├── trade                   # 模拟交易
│   ├── init                # 初始化账户
│   ├── reset               # 重置账户
│   ├── buy                 # 买入
│   ├── sell                # 卖出
│   ├── history             # 成交历史
│   ├── fees                # 费用明细
│   ├── overview            # 账户概览
│   ├── position            # 当前持仓
│   └── cash                # 现金快照
│
├── risk                    # 风控管理
│   ├── import              # 导入流水
│   ├── rebuild             # 重建账户
│   ├── rule                # 规则管理
│   ├── log                 # 风控日志
│   ├── lock                # 买入锁管理
│   ├── status              # 风控状态
│   ├── pnl                 # 盈亏快照
│   └── position            # 持仓风险
│
├── execution               # 执行管理
│   ├── config              # 执行配置
│   ├── daemon              # 执行守护进程
│   └── bridge              # Bridge 诊断
│       ├── status          # 状态检查
│       └── qmt-preview     # QMT 预览
│
└── anomaly                 # 异常检测
    └── run                 # 运行检测
        ├── --top-n         # 显示数量
        ├── --period        # K线周期
        ├── --output        # 输出格式
        └── --mock          # 模拟数据

├── account                 # 账户管理
│   ├── register            # 注册新账户
│   ├── list                # 列出所有账户
│   ├── show                # 查看账户详情
│   ├── update              # 更新账户配置
│   ├── remove              # 删除账户
│   ├── default             # 设置默认账户
│   ├── summary             # 资金聚合视图
│   ├── split               # 订单拆分预览
│   └── group               # 账户组管理
│       ├── create          # 创建账户组
│       ├── list            # 列出账户组
│       ├── show            # 查看组详情
│       ├── remove          # 删除账户组
│       ├── add-account     # 添加账户到组
│       ├── remove-account  # 从组移除账户
│       └── set-strategy    # 设置分配策略
│
├── ai                      # AI决策分析 [✅ Phase 2 已完成]
│   ├── ask                 # 对话式分析
│   ├── decision            # 生成决策报告
│   │   └── --code          # 指定股票代码
│   │   └── --model         # 指定LLM模型
│   └── config              # AI配置管理
│       ├── list-models     # 列出可用模型
│       └── test            # 测试LLM连接
│
├── news                    # 新闻搜索 [✅ Phase 3 已完成]
│   ├── search              # 搜索股票新闻
│   │   ├── --code         # 按股票代码
│   │   ├── --keyword      # 按关键词
│   │   ├── --provider     # 指定数据源
│   │   └── --days         # 时间范围
│   └── trend               # 市场热点趋势
│
├── fundamental             # 基本面分析 [✅ API解析已实现 - Phase 4]
│   ├── show                # 综合基本面 (EastMoney数据源)
│   ├── valuation           # 估值指标 PE/PB/PS/市值/ROE/EPS
│   ├── earnings            # 财报数据 营收/净利润/毛利率
│   ├── institution         # 机构持仓 + 类型映射 + 变动方向
│   ├── capital-flow        # 资金流向
│   ├── dragon-tiger        # 龙虎榜 + PascalCase解析
│   └── dividend            # 分红信息
│       └── --years         # 历史年数
│
├── sentiment               # 舆情分析 [✅ 已连线 - Phase 4]
│   └── show                # 查看舆情情绪
│       └── --code          # 股票代码（美股）
│
├── notify                  # 多渠道通知 [📋 计划中 - Phase 1]
│   ├── send                # 发送通知
│   │   ├── --channel      # 指定渠道(telegram/wechat/feishu...)
│   │   ├── --message      # 消息内容
│   │   └── --file         # 从文件读取
│   └── test                # 测试通知渠道
│       └── --channel       # 指定渠道(all=全部)
│
└── import                  # 智能导入 [📋 可选 - Phase 5]
    ├── from-image          # 图片识别导入
    ├── from-csv            # CSV文件导入
    ├── from-excel          # Excel文件导入
    └── from-clipboard      # 剪贴板导入
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
| `quantix_rust_main_review` | 代码审查记录 |
| `quantix_rust_main_debug` | 调试会话记录 |
| `quantix_rust_main_handoff` | 交接文档 |
| `quantix_rust_main_docs` | 技术文档 |

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
| `paper` + `mock_live` | 已实现的执行目标 |
| `live` | 故意保持不完整 |
| Bridge 不拥有执行状态 | Windows 端无状态 |
| `QMT` | Bridge v1 仅预览 |
| `TDX bridge source` | 第一个真正的 bridge 能力 |

---

## 附录 B: 当前非目标

以下明确不在已完成的功能设计范围内：

- 真实 live broker 执行
- Windows 端拥有运行时状态
- Wind / Choice bridge 集成
- Bridge 端 WebSocket / gRPC 栈
- 分布式 worker 或多进程执行守护进程协调

---

## 附录 C: 运营视图

### C.1 当前可用功能

```
当前可用
├── 本地数据/研究/选股
├── 本地 paper 交易
├── 本地风控/监控/止损工作流
├── 策略 paper + mock_live + 请求生命周期
├── 执行守护进程消费
├── 多账户管理 (账户组、资金聚合、智能拆单)
└── Windows Bridge v1
    ├── TDX bridge 数据源
    ├── QMT 预览
    └── QMT live 端点预留（当前产品仍为 preview-only）
```

### C.2 故意推迟的功能

```
故意推迟
├── 真实 live 适配器
├── Wind / Choice bridge 支持
├── Bridge 拥有的订单生命周期
└── 更广泛的分布式运行时问题
```

---

## 附录 D: Windows Bridge v1 功能树

```
Windows Bridge v1
├── Rust 端
│   ├── src/bridge/client.rs
│   ├── src/bridge/models.rs
│   ├── src/sources/bridge_tdx.rs
│   ├── src/watchlist/resolver.rs (BridgeTdxWatchlistQuoteLookup)
│   ├── src/execution/qmt_bridge.rs
│   └── quantix execution bridge ...
│
└── Windows 端
    └── /mnt/d/mystocks/quantix/quantix_bridge
        ├── /health
        ├── /api/v1/capabilities
        ├── /api/v1/data/tdx/quotes
        ├── /api/v1/data/tdx/kline/{symbol}
        ├── /api/v1/broker/qmt/account/status
        ├── /api/v1/broker/qmt/orders/preview
        └── /api/v1/broker/qmt/orders/live   # Bridge API 端点/预留能力；不代表 quantix-rust 当前已开放真实下单
```
