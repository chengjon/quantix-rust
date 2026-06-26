# quantix-rust

A 股量化交易 CLI 工具 - Rust 实现

与 Python quantix 项目共享数据源和数据库，提供高性能的量化分析能力。

## Foundation P0 工作约束

- 仓库内本地 worktree 放在 `.worktrees/`，全文检索和批量扫描应排除该目录，避免重复命中。
- 本地分析产物和工具目录如 `.gitnexus/`、`target/` 应视为噪音目录，并通过 `.ignore` 排除。
- Foundation P0 当前支持前台 CLI、单机 daemon 与 `systemd --user` 用户服务，不假设多 worker、分布式调度或真实 broker 已可用。
- `quantix task` CLI 当前只支持查看预置任务模板、以前台模式启动它们，以及输出能力边界说明；`task add` 与 `task start --daemon` 仍未开放。

## 功能注册表

功能状态、能力边界、已设计/待实现项都以单一注册表为准：

- 功能全景图与状态注册表：[FUNCTION_TREE.md](FUNCTION_TREE.md)

本文不是功能状态注册表；任何功能状态、证据和边界判断都以 `FUNCTION_TREE.md` 的状态注册表行为准。

不再维护独立规划文档；新增计划或设计项先登记到 `FUNCTION_TREE.md`，并显式标状态、证据和边界。

README 只保留项目入口、使用示例和历史概览；当示例或概览与注册表不一致时，以 `FUNCTION_TREE.md` 为准。

变更记录见 [CHANGELOG.md](CHANGELOG.md)。

## 开发前置检查

开始修改代码前，先确认：

- `cargo --version` 可用；若当前环境没有 Rust toolchain，只适合做结构/文档审阅，不适合宣称已完成构建验证。
- 默认以 WSL/Linux 路径和 CLI 作为执行基线；Cursor 可以作为主编辑器，但不应是唯一执行路径。
- 只在需要相关能力时再配置数据库、Bridge 或其他可选服务环境变量。
- 与 `WSL + Cursor` 并存开发相关的最小 checklist 见 [docs/QUICKSTART.md](docs/QUICKSTART.md)。

当前建议的推进顺序：

1. 先完成仍直接影响执行边界可信度的残余语义加固，优先处理 `request completed` 语义、gate 原因可视化、以及半接线 `live` 分支核查。
2. 紧接着推进交易主线稳态化，优先收口交易日历 / T+1、审计日志、回撤控制、promotion checklist、关键链路监控与最小 kill switch。
3. 在主线稳态化后，再推进 real live / broker execution 收口。
4. 主线稳定后，再处理研究、组合、扩展风控与运维能力；TUI 首屏菜单和 data CSV/Parquet 导出已接线，batch/metrics 等工程占坑继续作为次级队列处理。

## 构建产物大小管控

Rust debug 构建默认 `debug = 2`（完整 DWARF 调试信息），会导致 `target/debug/deps` 膨胀至 50GB+。项目通过三道防线控制：

### 1. Profile 配置（根因层，Cargo.toml）

```toml
[profile.dev]
debug = 1                        # 仅保留行号表，backtrace 仍可用

[profile.dev.package."*"]
debug = 0                        # 依赖库不生成 debug info

[profile.test]
debug = 1

[profile.test.package."*"]
debug = 0
```

效果：`target/debug/` 从 ~50GB 降至 ~2-3GB（cargo build）或 ~5-7GB（cargo test）。

### 2. 构建后自动检查（cb.sh wrapper）

每次 `cargo build` / `cargo test` 后自动运行大小检查：

```bash
# 替代直接调用 cargo
scripts/dev/cb.sh build          # = cargo build + size guard
scripts/dev/cb.sh test           # = cargo test + size guard

# 推荐 shell alias（加入 ~/.bashrc 或 ~/.zshrc）
alias cb='/opt/claude/quantix-rust/scripts/dev/cb.sh build'
alias ct='/opt/claude/quantix-rust/scripts/dev/cb.sh test'
```

也可手动运行监控脚本：

```bash
scripts/dev/guard_target_size.sh --status   # 查看状态
scripts/dev/guard_target_size.sh --clean    # 超阈值自动清理
scripts/dev/guard_target_size.sh            # 仅检查，超阈值 exit 1
```

### 3. 异常排查

如果 `target/` 超过 8GB，常见原因：

- 执行了 `cargo build` 而非 `cargo build --release`，且修改了 profile 设置
- 依赖版本冲突导致同一 crate 被编译多次（检查 `cargo tree --duplicates`）
- 长期未清理的陈旧构建产物（脚本会标记 >7 天的 stale 文件）

## 当前完成状态

截至 2026-06-26，当前已经完成并落地的任务可概括为：

- 策略执行主线已经闭环到 `paper` / `mock_live` / `execution_request` / `execution daemon` 这一层，`runtime.db`、frozen snapshot 和 `ExecutionKernel` 边界已经稳定。
- operator 工作流已经覆盖 `watchlist`、`screener`、`market`、`monitor`、`stop`、`trade`、`risk` 这几条主线，并且都已有 README / USER_MANUAL 级别说明。
- `screener` preset 解析和运行时边界已补齐异常输入防线：空参数段、重复参数、零窗口/周期、非有限阈值和 RSI lookback 溢出都会返回显式错误，而不是静默覆盖、回绕或 panic。
- `screener run --sort-by` 仅支持 `code` 或 `score`；未知字段会在读取 ClickHouse 日线数据或输出筛选表格前返回显式 `Unsupported`，错误包含 `不支持的 sort_by`。
- `account register --account-type` 仅支持 `paper`、`mock_live`、`qmt_live`（兼容 `live` 别名）；未知账户类型会在写入本地账户注册表前返回显式 `Unsupported`，错误包含 `无效的账户类型`。
- `account split --target-type` 仅支持 `single` 或 `group`；未知目标类型会在输出订单拆分预览前返回显式 `Unsupported`，错误包含 `无效的目标类型` 和支持列表 `single, group`。
- GitNexus MCP 日常使用建议已沉淀为 `docs/guides/GITNEXUS_MCP_DAILY_WORKFLOW_RECOMMENDATIONS.md`，用于配合 `FUNCTION_TREE.md`、GitNexus impact/detect gate 和 Graphiti 记忆流程进行日常开发。
- Windows Bridge v1 已完成首版设计与实现集成：
  - `TDX bridge source` 已接入 Rust 侧 bridge client 和 watchlist quote lookup
  - `QMT` 已接入 execution bridge CLI：`qmt-preview` 用于基于 frozen request 做 broker payload 预览，guarded `qmt_live` 用于真实提交
  - canonical Windows-side 路径固定为 `/mnt/d/mystocks/quantix/quantix_bridge`
- qmt_live P0.4g reconciliation query refinement 已闭合：完整本地 `task_identity` 存在时使用 `task_id + client_order_id + local_submission_id` 查询并复用任务服务身份校验；legacy/partial identity 保留 task-id-only recovery；身份不匹配转人工介入；不改动 gate/diagnostics、bridge 协议、响应 shape、存储 schema、`OrderStatus`、`ExecutionAdapter` 或 submit/query/cancel 主流程。
- qmt_live P0.5 operational safety 已完成面向 operator 的只读闭合：`quantix execution qmt audit` 提供证据视图，`quantix execution qmt manual-interventions list/show` 提供未解决人工介入报表；它们覆盖 identity mismatch、broker unknown state、missing external order id、preserved local state、bridge failure 等持久化信号，但都不修改 runtime/broker state，也不触发提交、撤单或回写。
- qmt_live P0.6 runtime readiness 已完成阶段性归档，最终决策为 `blocked_by_environment`：本地安全约束、脱敏证据包和 fail-closed 边界已记录，但缺少 operator 选定的 miniQMT Windows Bridge 运行环境、账户标签和真实只读 smoke 证据；当前禁止启动 qmt_live canary，P0.6 仅保留维护归档，开发重心转向 ExecutionCapabilities 后续抽象与 OpenStock 数据消费适配。
- ExecutionCapabilities P0.7 显示层语义同步已闭合：P0.7a 将静态 `ExecutionCapabilities` 映射到稳定通道语义、风险提示和 storage namespace helper；P0.7b/P0.7c 分别在 qmt_live promotion checklist 与 human-readable preflight report 中输出 `risk_notice` 和 `storage_namespace`；JSON payload、bridge 协议、runtime storage、`ExecutionAdapter`、`OrderStatus`、submit/query/cancel 主流程和 qmt_live runtime probing 均未改动。`request_diagnostics.rs` 后续接线因 GitNexus impact 为 HIGH，需单独专项审批。
- OpenStock 数据消费 P0.8 已建立 OpenSpec 规划：该主线将作为 broker-independent 行情消费方向推进，先做现有模型/来源/消费者 inventory，再做 fixture-owned provider parser 和只读 CLI/本地 artifact 验证；本规划片不改生产 Rust 代码、不做 live OpenStock CI 请求、不写 ClickHouse，也不改变 qmt_live、miniQMT market-manifest、`tdx_api` 或其他既有数据源行为。P0.8 closeout Graphiti episode `fb126253-d46e-41eb-98fd-924083015af3` 未达到 `completed`，已按项目规则记录本地 backfill 报告。
- OpenStock 数据消费 P0.8a inventory 已完成：已映射当前 `Kline` / `StockQuote` / `StockInfo` 形状、`tdx_api` / `bridge_tdx` / `eastmoney` / miniQMT manifest 边界、ClickHouse kline/quote 路径和 backtest 消费入口；建议 P0.8b 从 committed fixture 的 daily-kline parser/normalizer 开始，先输出 `Vec<Kline>`，仍不做 live network、ClickHouse 写入或既有数据源路由替换。
- **股票异常检测模块**已完成 Isolation Forest 算法迁移与东方财富 API 集成：
  - 基于 Surpriver 项目的 Isolation Forest 算法
  - 支持真实东方财富 API 数据源 (`EastMoneyAnomalySource`)
  - A股特有过滤器（ST、涨跌停、停牌、新股）
  - 特征提取：成交量回报、对数回报、EOM指标
- **AI 决策模块**已完成基础实现 (Phase 2)：
  - `LLMAdapter` - OpenAI 协议统一适配器
  - 运行时已接线 provider：OpenAI、DeepSeek、Ollama；Gemini、Anthropic 目前仅有配置/模型枚举，`ai analyze` / `ai decide` / `ai ask` / `ai market` 在未配置任一已接线 provider 或只配置 Gemini/Anthropic 等未接线 provider 时会 fail-closed，不会静默回退到 Ollama；`ai config` 仅查看配置状态
  - `DecisionEngine` - 决策仪表盘生成
  - `ConversationManager` - 多轮对话上下文管理
  - `ai analyze` / `ai decide` / `ai ask` / `ai market` 会在运行时显式提示模拟价格/指标、模拟技术面分析、问答参数或固定 prompt 边界；这些入口适合验证 LLM 接线，不等同于实时投研或实仓交易决策
  - `ai config --test` 是配置状态检查，运行时标题为“检查 LLM 配置状态”，不会发起真实 API 连通性请求
- **新闻搜索模块**已完成基础实现 (Phase 3)：
  - `NewsProvider` trait - 新闻提供者接口
  - 已接线 provider：Tavily、SerpAPI、博查搜索；Brave、SearXNG 仍是已设计/待实现
  - `news search` / `code` / `trend` 在未配置任一已接线 provider 时会返回显式 `Unsupported`；`news providers` 仅查看配置状态
  - `NewsAggregator` - 多源 fallback 聚合
  - `NewsCache` - 本地缓存存储
- **基本面 CLI 边界**已同步到当前实现：
  - `fundamental show` / `valuation` / `earnings` / `institution` / `dragon-tiger` 是当前主要可用入口
  - `fundamental capital-flow` 与 `fundamental dividend` 已暴露命令壳，但真实资金流向/分红数据源未接线前会返回显式 `Unsupported`
- **舆情 CLI 边界**已同步到当前实现：
  - `sentiment show` / `history` / `mentions` 已暴露命令壳，但默认 provider 与趋势计算尚未接线，真实舆情数据源可用前会返回显式 `Unsupported`
- **智能导入 CLI 边界**已同步到当前实现：
  - `import from-image` / `from-csv` / `from-excel` / `from-clipboard` / `from-text` / `resolve` / `market-manifest` 是当前已接线入口
  - `import from-image --model deepseek|openai` 会先校验图片扩展名，只支持 `png, jpg, jpeg, gif, webp`；不支持的格式会在 Vision provider 配置校验或请求前返回显式 `Unsupported`，错误包含 `image format 不支持`；不支持的 Vision provider 会在 provider 配置校验或请求前返回显式 `Unsupported`，错误包含 `Vision provider 不支持` 和支持列表 `deepseek, openai`；缺少所选 provider 的 API key 时返回显式 `Unsupported`，错误包含 `Vision provider 尚未配置`
  - `import from-excel` 可读取首个或指定 worksheet 中的 watchlist 代码/名称行；复杂 Excel schema 与持久化导入闭环仍不是当前能力
- **data export CLI 边界**已同步到当前实现：
  - `data export --format` 仅支持 `csv` 和 `parquet`；未知格式会在任何 stdout 导出提示、ClickHouse 读取或输出目录创建之前返回显式 `Unsupported`，错误包含 `data export format 不支持`
- **P0.2 执行请求生命周期增强**已完成：
  - `strategy request show` - 查看请求详情
  - `strategy request list --stats` - 统计汇总视图
  - 多维度过滤（状态、模式、账户）
- **多账户管理系统**已完成设计与实现：
  - `AccountConfig` - 账户配置模型 (Paper/Live/MockLive)
  - `AccountGroup` - 账户组配置，支持资金分配策略
  - `AllocationStrategy` - Equal/Proportional/Weighted/PrimaryFirst
  - `AccountRouter` - 智能订单路由，按策略拆分订单
  - 完整 CLI 命令支持 (`quantix account *`)
  - `account register --account-type` 对未知账户类型 fail-closed：写入 `~/.quantix/accounts/registry.json` 前返回显式 `Unsupported`，支持列表固定为 `paper, mock_live, qmt_live`（兼容 `live` 别名）
  - `account split --target-type` 对未知目标类型 fail-closed：输出 `订单拆分预览` 前返回显式 `Unsupported`，支持列表固定为 `single, group`
- **算法交易执行器**已完成基础实现：
  - TWAP (时间加权平均价格) 执行器
  - VWAP (成交量加权平均价格) 执行器
  - `algo create --algo-type` / `algo plan --algo-type` 仅支持 `twap, vwap`；未知算法类型会在初始化算法上下文或输出切片预览前返回显式 `Unsupported`，错误包含 `不支持的算法类型`
  - `algo create` / `algo plan` 会对方向、切片数、切片间隔和 `plan --output` 格式等参数 fail-closed；POV / Iceberg 仍未接线
- **Graphiti MCP 集成**已完成：
  - 语义记忆层用于设计决策、代码审查、调试、交接和文档
  - Group IDs: `quantix_rust_main`, `_review`, `_debug`, `_handoff`, `_docs`
  - 当 Graphiti ingest 超时失败或长期停留 `processing` 且无法验证 `completed` 时，项目按规则在 `docs/reports/*_GRAPHITI_BACKFILL_*.md` 留下 `Graphiti backfill required` 本地回填记录
- **因子研究首切片**已有本地 CLI 闭环：
  - `factor list` - 查看已登记因子
  - `factor compute --input CSV` - 基于本地 CSV 计算因子并导出 table/csv/json/parquet
  - `factor score --input CSV` - 在最新因子日期上按多个因子等权评分并导出 table/csv/json/parquet
  - `factor evaluate --input CSV` - 计算 IC/IR、相关性等评估结果并导出 table/csv/json/parquet
  - 能力边界和最新状态以 [FUNCTION_TREE.md](FUNCTION_TREE.md) 的 `factor/` 与 `quantix factor` 注册行为准
- 当前明确仍未完成的是真实 `live` broker execution，以及 `Wind` / `Choice` bridge 支持。

## 功能特性

### 已完成模块

#### Phase 1: 数据采集基础 ✅
- **数据源适配器**
  - TDX (通达信) - 实时行情数据
  - AkShare - 财务数据、历史数据
  - Quote Collector - 多股票实时采集
  - Auction Collector - 竞价数据采集（生产构造依赖 live TDX TCP；默认单元测试不再连接外部 TDX，使用 deterministic watchlist 断言）

#### Phase 2: 竞价分析模块 ✅
- **竞价分析器** (`src/analysis/auction.rs`)
  - 抢筹强度评分 (涨幅40% + 买盘占比30% + 成交量30%)
  - 封单金额计算
  - 板块统计 (上海主板/科创板/深圳主板/创业板)
  - 推荐买入筛选

#### Phase 3: K线数据管理与同步 ✅
- **K线聚合器** (`src/sources/kline_aggregator.rs`)
  - 实时聚合 Tick 数据为 1m/5m/15m/30m/60m/1d K线
  - 使用 tokio channels 替代 Redis Stream
  - 自动窗口对齐和过期清理
- **数据同步模块** (`src/sync/etl.rs`)
  - PostgreSQL/TDengine → ClickHouse 数据桥接
  - 支持日线和分钟线同步
  - 定时同步任务调度
- **ClickHouse 数据库** (`src/db/clickhouse.rs`)
  - MergeTree 引擎，月度分区
  - 4张核心表: stock_info, stock_realtime_quotes, kline_data, limit_up_events

#### Phase 4: 回测引擎 ✅
- **投资组合管理** (`src/analysis/portfolio.rs`)
  - 持仓管理、买入卖出逻辑
  - 手续费计算、滑点模拟
- **性能指标计算** (`src/analysis/performance.rs`)
  - 夏普比率、索提诺比率、最大回撤
  - 胜率、盈亏比、卡玛比率
- **回测引擎** (`src/analysis/backtest.rs`)
  - 事件驱动回测框架
  - 策略 trait 接口集成
  - 完整的回测报告生成

#### Phase 5: 任务调度 ✅
- **Cron 解析器** (`src/tasks/cron.rs`)
  - 支持 */N 步长语法
  - 支持 1-5 范围语法
  - 支持列表语法 1,2,3
- **任务调度器** (`src/tasks/scheduler.rs`)
  - 集成 tokio-cron-scheduler
  - 动态添加/删除任务
  - 预定义任务模板 (盘前/竞价/开盘/收盘/盘后/数据同步)
  - CLI 层当前只开放预置模板查看与前台启动；`task add` / `task start --daemon` 仍是保留入口

#### Phase 6: TDX 文件解析与复权 ✅
- **Day 文件解析器** (`src/sources/tdx_file.rs`)
  - 通达信 day 文件解析 (32字节/记录)
  - GBBQ 股本变迁文件解析 (29字节/记录)
  - 复权因子计算算法 (基于涨跌幅连续计算)
  - 前复权/后复权应用
  - 批量导入器

#### Phase 7: GBBQ 数据存储 ✅
- **数据模型** (`src/data/models.rs`)
  - GbbqEvent: 除权除息事件
  - CapitalChange: 股本变更摘要
- **ClickHouse 存储** (`src/db/clickhouse.rs`)
  - gbbq_events 表 (按月分区)
  - 事件插入/查询接口
  - 最新除权事件查询

#### Phase 8: 多周期 K线查询 ✅
- **K线查询接口** (`src/db/clickhouse.rs`)
  - get_kline_data(): 查询指定周期 K线
  - insert_kline_data(): 插入 K线数据
  - insert_kline_data_batch(): 批量插入
  - get_daily_from_minute(): 分钟线聚合日线
- **支持的周期**: 1m, 5m, 15m, 30m, 60m, 1d

#### Phase 9: 东方财富数据采集 ✅
- **EastMoney 数据源** (`src/sources/eastmoney.rs`)
  - 股票列表获取 (支持板块分类: HS300, ZZ500, SZ50, KCB50, BZ50)
  - 实时行情查询
  - 资金流向数据
  - 财务数据获取
  - HTTP 客户端集成

#### Phase 10: ClickHouse 批量导入优化 ✅
- **批量插入优化** (`src/db/clickhouse.rs`)
  - 使用 clickhouse crate 的 insert API
  - 可配置批次大小 (默认 1000)
  - 支持 async_insert 选项提升性能
  - 新增 insert_stock_quotes_batch 方法
  - 优化的 insert_gbbq_events_batch 和 insert_kline_data_batch

#### Phase 11: WebSocket 实时行情订阅 ✅
- **WebSocket 客户端** (`src/sources/websocket.rs`)
  - tokio-tungstenite WebSocket 连接
  - 连接状态管理 (Disconnected/Connecting/Connected/Reconnecting)
  - 订阅/取消订阅方法
  - 心跳保活机制 (可配置间隔)
  - 自动重连机制 (可配置最大次数)
  - 使用 tokio::select! 实现并发消息处理

#### Phase 12: 技术指标增强 ✅
- **技术指标** (`src/analysis/indicators.rs`)
  - SMA/EMA/WMA (简单/指数/加权移动平均)
  - RSI (相对强弱指标)
  - MACD (指数平滑异同移动平均线)
  - KDJ (随机指标)
  - Bollinger Bands (布林带)
  - ATR (平均真实波幅)
  - OBV (能量潮)
  - CCI (顺势指标)
  - Williams %R (威廉指标)

#### Phase 13: Polars 统一数据管理层 ✅
- **Polars 适配器** (`src/analysis/polars_adapter.rs`)
  - 全局线程配置 (`init_polars`)
  - 批量K线数据结构 (`BatchKlineData`)
  - Polars 指标计算器 (`PolarsCalculator`)
    - 移动平均线 (MA) 使用 `RollingOptionsFixedWindow`
    - 批量计算多个指标 (`calculate_batch`)
  - 多股票批量处理 (`MultiStockData`)
  - K线数据转换 (`from_kline_vec`)
- **Polars 0.43 API 适配**
  - `PlSmallStr` 用于 Series 名称
  - `rolling_mean()` 使用固定窗口选项
  - `cast()` 使用 `&DataType` 参数
  - Series 迭代使用 `.iter().map(|av| av.extract::<f64>())`

#### Phase 14: CLI 命令实现 ✅
- **完整命令处理器** (`src/cli/handlers.rs`)
  - `init` - 初始化配置和数据库
  - `menu` - 交互式菜单 (简单版 + TUI 规划)
  - `data query` - 查询历史K线数据
  - `data export` - 导出数据为 CSV/Parquet，未知 `--format` 会失败关闭
  - `strategy run` - 运行策略回测
  - `strategy list/show` - 策略管理
  - `task list/start/stop/status` - 实验性 Foundation P0 预置任务模板入口
  - `analyze indicators` - 技术指标计算
  - `status --health` - 数据库健康检查
- **交互式菜单系统**
  - 数据同步菜单
  - 策略运行菜单
  - 回测分析菜单
  - 任务管理菜单
  - 技术分析菜单
  - 数据导出菜单
- **CSV 数据导出** - 使用 csv crate 实现
- **进度条支持** - indicatif 进度显示

#### Phase 21: 自选池管理 ✅
- **自选池命令** (`src/cli/mod.rs`, `src/cli/handlers.rs`)
  - `watchlist add/remove/list/move` - 基础自选池维护
  - `watchlist group create/list` - 分组管理
  - `watchlist tag add/remove/list` - 标签管理
  - `watchlist history` - 本地操作历史
  - `watchlist list --with-price` - 最佳努力价格展示
- **JSON 持久化** (`src/watchlist/storage.rs`)
  - 默认路径 `~/.quantix/watchlist/watchlist.json`
  - 可通过 `QUANTIX_WATCHLIST_PATH` 覆盖
- **P0 约束**
  - 价格展示使用前台最佳努力查询
  - 行情不可用时降级为空价格，不影响 `watchlist list` 返回

#### Phase 22: 选股筛选 ✅
- **选股命令** (`src/cli/mod.rs`, `src/cli/handlers.rs`, `src/screener/*`)
  - `analyze screener preset-list` - 查看内置单指标 preset
  - `analyze screener run --codes ...` - 对显式代码列表执行筛选
  - `analyze screener run --watchlist [--group ...]` - 对自选池/分组执行筛选
- **P0 约束**
  - 仅支持日线筛选
  - `preset` 只表达单一指标条件，但支持重复 `--preset` 做 `AND` 组合
  - 条件完全参数化，例如 `close_above_ma:period=20`
  - preset 参数解析拒绝空参数段、重复 key、零周期/窗口、非有限数值和 lookback 溢出
  - 不支持全市场扫描、DSL、实时筛选、OR 逻辑

#### Phase 23: 市场分析 ✅
- **市场分析命令** (`src/cli/mod.rs`, `src/cli/handlers.rs`, `src/market/*`)
  - `quantix market foundation` - 全市场 A 股与行业分类基础摘要
  - `quantix market sector` - 行业板块排名
  - `quantix market concept` - 概念板块排名
  - `quantix market north` - 北向资金概览
  - `quantix market sentiment` - 市场情绪快照
  - `quantix market leader` - 龙头股识别
  - `quantix market overview` - 综合概览
  - `quantix market strength` - 强弱板块与强势板块个股 Top10
  - `quantix market strength-stocks` - 强势板块个股按市值/利润直接排行，支持 `--sector`
- **P0 约束**
  - 仅覆盖日度快照和只读查询
  - `leader` 只支持 `--sector`、`--concept`、`--all` 三选一
  - `market sector|concept --sort-by` 仅支持 `change` 或 `change_pct`（均按涨跌幅排序）；未知字段会在读取 ClickHouse 或输出板块表格前返回显式 `Unsupported`，错误包含 `不支持的 sort_by`
  - `foundation` / `strength` 依赖已同步的申万一级行业 SQLite 引用表
  - 历史/详情/实时功能延后到后续 Phase

#### Phase 24: 实时监控 ✅
- **监控命令** (`src/cli/mod.rs`, `src/cli/handlers.rs`, `src/monitor/*`)
  - `quantix monitor watchlist --once` - 扫描当前自选池并输出终端监控快照
  - `quantix monitor watchlist --repeat` - 在前台持续轮询自选池并输出监控快照
  - `quantix monitor alert add 000001 --above 16.0` - 添加向上突破价格告警
  - `quantix monitor alert add 000001 --below 15.0` - 添加向下跌破价格告警
  - `quantix monitor alert list` - 查看当前有效价格告警
  - `quantix monitor alert remove 1` - 删除指定价格告警
  - `quantix monitor config show` - 查看当前监控配置
  - `quantix monitor daemon run` - 运行 monitor 守护进程
  - `quantix monitor service install` - 安装 `systemd --user` 监控服务
  - `quantix monitor service-config show` - 查看 monitor service 二进制配置
  - `quantix monitor service-config set --quantix-bin /abs/path/to/quantix` - 设置稳定的 service 二进制路径
  - `quantix monitor event list` - 查看最近监控业务事件
- **SQLite 告警持久化** (`src/monitor/storage.rs`)
  - 默认路径 `~/.quantix/monitor/alerts.db`
  - 可通过 `QUANTIX_MONITOR_DB_PATH` 覆盖
- **JSON 配置持久化** (`src/monitor/config.rs`)
  - 默认路径 `~/.quantix/monitor/config.json`
  - 可通过 `QUANTIX_MONITOR_CONFIG_PATH` 覆盖
- **Service 配置与包装脚本** (`src/monitor/service_config.rs`, `src/monitor/systemd.rs`)
  - service 配置路径 `~/.quantix/monitor/service.json`
  - wrapper 脚本路径 `~/.local/bin/quantix-monitor-run`
- **P0 约束**
  - 支持 `watchlist --once`、`watchlist --repeat`、`daemon run` 与 `systemd --user` 用户服务
  - 复用现有自选池加载、TDX 行情查询与 stop 规则评估链路
  - 业务事件只持久化价格告警命中与 stop 触发，不持久化服务生命周期日志
- `systemd --user` 当前面向 WSL2/Linux 用户环境
- `service install` 要求先配置稳定的 `quantix` 二进制绝对路径
- `service uninstall` 必须先停服务再卸载
- 系统通知当前支持 `quantix monitor watchlist --repeat` / `quantix monitor daemon run` 对新增监控事件做自动通知桥接
- 推荐通过 `quantix monitor config set --notify true` 显式开启
- `QUANTIX_MONITOR_NOTIFY=1` 仍保留为兼容兜底开关
- 通知渠道复用 `quantix notify` 环境变量约定，最小可用路径是 `NOTIFICATION_LOG_PATH`
- `quantix notify send --level` 仅支持 `info, warning, error, critical`，未知级别会在任何发送进度 stdout 前返回显式 `Unsupported`，错误包含 `无效的通知级别`
- `quantix notify check/test/send --channel <外部渠道>` 在未知渠道或缺少必需环境变量时返回显式 `Unsupported`，错误包含 `notify channel 不支持` 或 `notify channel 尚未配置`，不会先输出检查/发送进度；`notify test --channel all` 仍按环境聚合渠道发送，`notify list` 仍只是渠道名称状态视图
  - 系统通知延后到后续 Phase

#### Phase 25: 止盈止损 ✅
- **止盈止损命令** (`src/cli/mod.rs`, `src/cli/handlers.rs`, `src/stop/*`)
  - `quantix stop set 000001 --loss 14.5` - 为自选池代码设置固定止损价
  - `quantix stop set 000001 --profit 18.0` - 为自选池代码设置固定止盈价
  - `quantix stop set 000001 --trailing 5 --profit 18.0` - 为自选池代码设置跟踪止损并可叠加止盈价
  - `quantix stop set 000001 --loss-pct 5` - 为自选池代码设置百分比止损
  - `quantix stop update 000001 --profit-pct 12 --clear-profit` - 局部更新规则并清理旧阈值
  - `quantix stop list` - 查看当前有效规则
  - `quantix stop status --code 000001` - 查看当前评估状态、锚点来源和有效阈值
  - `quantix stop history --code 000001 --limit 10` - 查看规则变更与触发审计历史
  - `quantix stop remove 000001` - 删除指定代码的规则
- **复用监控 SQLite** (`src/monitor/storage.rs`, `src/stop/storage.rs`)
  - 默认路径 `~/.quantix/monitor/alerts.db`
  - 可通过 `QUANTIX_MONITOR_DB_PATH` 覆盖
- **P0 约束**
  - 仅允许对已在本地自选池中的股票设置规则
  - 每个代码只保留一条有效规则，重复 `stop set` 会整条覆盖旧规则
  - `stop update` 采用局部 patch 语义，只改显式传入字段
  - 百分比规则优先锚定本地 paper 持仓均价
  - 无持仓时退回到规则的 reference_price
  - `stop status` 会展示 `anchor_source`、当前阈值和 `eval_state`
  - stop_history 会记录规则变更和 trigger 审计事件
  - quantix monitor watchlist --once 会在监控快照阶段继续评估止盈止损规则
  - 当前不自动下单，也不直接触发卖出执行

#### Phase 26: 模拟交易 ✅
- **模拟交易命令** (`src/cli/mod.rs`, `src/cli/handlers.rs`, `src/trade/*`)
  - `quantix trade init [--capital <AMOUNT>] [--commission-rate <RATE>] [--commission-min <AMOUNT>] [--stamp-duty-rate <RATE>] [--transfer-fee-rate <RATE>]` - 初始化默认模拟账户
  - `quantix trade reset [--capital <AMOUNT>] [--commission-rate <RATE>] [--commission-min <AMOUNT>] [--stamp-duty-rate <RATE>] [--transfer-fee-rate <RATE>]` - 重置默认模拟账户
  - `quantix trade buy <CODE> --price <PRICE> --volume <N>` - 按输入价立即成交买入
  - `quantix trade sell <CODE> --price <PRICE> --volume <N>` - 按输入价立即成交卖出
  - `quantix trade history [--code <CODE>] [--limit <N>]` - 查看成交历史
  - `quantix trade fees [--code <CODE>] [--limit <N>]` - 查看费用明细
  - `quantix trade overview [--current]` - 查看账户概览
  - `quantix trade position [--current]` - 查看当前持仓，可选实时估值
  - `quantix trade position --current` - 使用 best-effort 实时行情查看当前估值
  - `quantix trade cash` - 查看现金与资产快照
- **JSON 持久化** (`src/trade/storage.rs`)
  - 默认路径 `~/.quantix/trade/paper_trade.json`
  - 可通过 `QUANTIX_TRADE_PATH` 覆盖
- **P0 约束**
  - 仅支持单账户、本地 JSON 持久化、立即成交的限价买卖
  - 手续费参数仅通过 `trade init/reset` 配置
  - `--current` 复用 best-effort 实时行情查询；实时价格获取失败时降级为空，不让命令整体失败

#### Phase 27: 风险管理 ✅
- **风控命令** (`src/cli/mod.rs`, `src/cli/handlers.rs`, `src/risk/*`)
  - `quantix risk rule set --type position-limit --value 20%` - 设置单票仓位上限
  - `quantix risk rule set --type daily-loss-limit --value 50000` - 设置日亏损金额阈值
  - `quantix risk rule set --type daily-loss-limit --value 5%` - 设置日亏损比例阈值
  - `quantix risk rule set --type volatility-limit --value 4%` - 设置 ATR 波动率阈值
  - `quantix risk sync industry --standard shenwan` - 从上游 MySQL 刷新本地申万行业引用表
  - `quantix risk rule set --type industry-blocklist --value 银行,地产` - 设置行业黑名单买入拦截规则
  - `quantix risk import live-trades --account live-001 --input /tmp/live.csv` - 导入标准化实盘流水
  - `quantix risk rebuild live-account --account live-001` - 从导入流水全量重建实盘镜像账户
  - `quantix risk rule list` - 查看当前风控规则
  - `quantix risk rule enable --type position-limit` - 启用指定规则
  - `quantix risk rule disable --type daily-loss-limit` - 禁用指定规则
  - `quantix risk status` - 查看当前纸面账户风控状态
  - `quantix risk pnl` - 查看当前当日盈亏快照
  - `quantix risk position` - 查看当前持仓风险分布
  - `quantix risk status --source live_import --account live-001` - 查看导入镜像账户风控状态
  - `quantix risk pnl --source live_import --account live-001` - 查看导入镜像账户盈亏快照
  - `quantix risk position --source live_import --account live-001` - 查看导入镜像账户持仓风险分布
  - `quantix risk log` - 查看最近风控事件
  - `quantix risk lock release` - 手动释放当前交易日买入锁
- **JSON 持久化** (`src/risk/storage.rs`)
  - 默认路径 `~/.quantix/risk/risk_state.json`
  - 可通过 `QUANTIX_RISK_PATH` 覆盖
  - 行业引用 SQLite 默认为 `~/.quantix/risk/industry_reference.db`，与 `risk_state.json` 同目录
  - 上游同步配置使用 `QUANTIX_UPSTREAM_MYSQL_URL`、`QUANTIX_UPSTREAM_MYSQL_DB`、`QUANTIX_UPSTREAM_MYSQL_USER`、`QUANTIX_UPSTREAM_MYSQL_PASSWORD`
- **P0 约束**
  - `paper` 仍是默认数据源，`live_import` 需要显式 `--source live_import --account <ID>`
  - live_import 镜像账户与 paper_trade.json 严格隔离
  - `volatility-limit` 使用 `ATR(14) / latest_close * 100`
  - `volatility-limit` 缺少日线时会拒绝买单而不是静默跳过
  - `volatility-limit` 只拦截新的买单，不影响卖出
  - `industry-blocklist` 现已成为受支持的风险规则
  - `industry-limit` 现已成为受支持的风险规则
  - 风控 CLI 当前仍接受 `auto-reduce` rule type，并在触发时输出 recommendation-only 的人工减仓建议
  - Phase 27D v1 使用 `SW 一级行业` 作为运行时生效标准
  - `security_class_2024` / CSRC 2024 仍保留在系统中作为并行分类标准，但不是该 v1 规则的运行时生效标准
  - 运行时风控评估只读取本地 SQLite 参考/快照表
  - MySQL 仅作为上游同步源，不是运行时查询依赖
  - 最终运行时边界保持为 ClickHouse + SQLite；本规则不在运行时直接查询 MySQL
  - 启用 `industry-blocklist` 前需要先执行 `quantix risk sync industry --standard shenwan`
  - 启用 `industry-limit` 前同样需要先执行 `quantix risk sync industry --standard shenwan`
  - 如果本地 SQLite 行业引用表尚未同步完成，`industry-blocklist` 会 fail-closed 并拒绝买单
  - 如果本地 SQLite 行业引用表尚未同步完成，`industry-limit` 也会 fail-closed 并拒绝买单
  - 运行时行业解析顺序固定为：当前 SW 映射 -> 查询月份快照 -> 历史 SW 映射 -> 最新本地快照
  - 月度快照会在该月第一次成功命中生效标准时冻结
  - `industry-blocklist` 继续使用精确字符串匹配
  - `industry-blocklist` 只拦截新的买单，不影响卖出路径
  - 实盘导入当前只支持项目标准化 CSV/JSON
  - failed rebuild 会保留上一次成功镜像状态
  - `trade buy` 会执行风控预检查，`trade sell` 仍然允许成交，`trade init/reset` 会清除当日买入锁但保留已配置规则
  - 日亏损只基于本地 paper-trade 账户资产快照，不做实时行情盯市
  - `risk status` 会额外显示锁状态来源、作用交易日、触发原因、触发时间，便于区分真实锁定与同日手动释放生效
  - `risk log` 仅记录规则变更、日亏损锁触发、手动释放、以及 rollover/reset 清锁事件，不记录每次买入拒单
  - `risk lock release` 仅对当前交易日生效，当日内不再自动重新锁定；次日或 `trade init/reset` 会自动清除该手动释放标记
  - `risk log` 默认返回最近事件，当前支持按事件写入日 `--date` 与事件类型 `--type` 过滤
  - `industry-limit` 会按目标行业的买后集中度执行真实拦截；`auto-reduce` 当前仅输出人工减仓建议，不会自动卖出

#### Phase 29: 策略 Paper 执行骨架 ✅
- **策略执行命令** (`src/cli/handlers.rs`, `src/execution/*`, `src/strategy/runtime.rs`)
  - `quantix strategy run -n ma_cross --mode paper --code 000001` - 运行 `ma_cross` 的单次 paper 执行
  - `quantix strategy run -n ma_cross --mode mock_live --code 000001` - 运行 `ma_cross` 的单次 mock-live 执行
- **Runtime 审计 SQLite** (`src/execution/runtime_store.rs`)
  - 默认路径 `~/.quantix/strategy/runtime.db`
  - 可通过 `QUANTIX_STRATEGY_RUNTIME_DB_PATH` 覆盖
- **P0 约束**
  - 当前仅支持 `ma_cross`
  - 当前仅支持单代码、单次执行
  - 执行前请先运行 `quantix trade init`
  - 运行结果会写入独立的 runtime SQLite，paper 账户与 risk 状态仍分别保存在原有本地存储中
  - `paper` 是立即成交路径，`mock_live` 当前会返回非终态订单状态
  - `mock_live` 可能返回 `accepted`、`partially_filled`、`pending_cancel`、`unknown` 等生命周期状态
  - `mock_live` 当前是 live-ready hardening / reconciliation scaffolding，用于验证 delayed fill、partial fill 与 `unknown` 恢复语义，不是真实 broker live execution
  - 同一个 mock-live 订单在 partial fill 场景下可能写出多笔 `TradeRecord`
  - 项目级 MOCK 使用边界与验收口径见 `docs/standards/MOCK_USAGE_POLICY.md`
  - 这些增量成交会直接体现在 `trade history`、`trade fees`、`trade overview` 的本地视图里
  - live 模式仍在开发中；通用 `target_mode=live` 仍在开发中
  - `execution daemon` 与基础自动审批已在下文 Phase 29C 补齐；当前真实下单只开放受 `qmt.mode=live` 保护的 `qmt_live` 路径

#### Phase 29B: 策略信号守护进程 ✅
- **策略守护进程配置** (`src/strategy/config.rs`)
  - `quantix strategy config init`
  - `quantix strategy config show`
  - 默认路径 `~/.quantix/strategy/config.json`
- **策略信号守护进程** (`src/strategy/daemon.rs`, `src/strategy/registry.rs`)
  - `quantix strategy daemon run`
  - `quantix strategy daemon run --once`
  - 当前支持：单代码、多个策略实例、日线新 bar 触发
  - 优先读取已落库日线；主读取器返回空或失败时，可回退到本地 TDX `day` 文件
  - fallback 读取根目录通过 `QUANTIX_TDX_ROOT` 指定
  - 当同一代码在多个 TDX 市场目录命中时，可通过 `QUANTIX_TDX_MARKET` 指定 `sh/sz/bj/ds`
- **Signal / Execution Request** (`src/execution/runtime_store.rs`)
  - `quantix strategy signal list`
  - `quantix strategy signal approve --signal-id <ID> --target-mode paper --target-account default`
  - `quantix strategy signal reject --signal-id <ID> --reason <TEXT>`
  - `quantix strategy request list`
  - `quantix strategy request execute --request-id <ID>`
  - `quantix strategy request cancel --request-id <ID> [--reason <TEXT>]`
  - 批准 signal 只会创建 `execution_request`，不会自动交易
  - `request execute` 会手动消费一个 `pending execution_request`
  - `strategy signal list` 输出包含 `source=<SOURCE> fallback=<BOOL>`
  - `strategy signal list` 支持按 `--strategy-instance`、`--strategy`、`--code`、`--approval-status`、`--signal-status` 过滤，并在过滤后应用 `--limit`
  - `strategy signal approve` 输出包含 `target=<MODE>/<ACCOUNT> status=<STATUS>`
  - `strategy request list` 输出包含 `target=<MODE>/<ACCOUNT> status=<STATUS>`
  - request completed 但订单仍非终态时，`strategy request list` / `execution daemon run --once` 会额外输出 `semantics=request_completed_order_non_terminal`
  - `strategy request show` 会同时展示 `request_status`、`order_status`、`executed_at`、`failed_at`、`canceled_at` 等诊断字段
  - 当 payload 内存在结构化 `execution_diagnostics` 时，`strategy request show` 会新增 `Execution Diagnostics` section，展示 `code`、`summary`、`operator_action`、`hint_command` 等字段
  - `mock_live` request 即使返回 `accepted`，request 也会记为 `completed`
- **WSL2 systemd --user 服务** (`src/strategy/systemd.rs`)
  - `quantix strategy service install`
  - `quantix strategy service status`
  - `quantix strategy service-config show`
  - `quantix strategy service-config set --quantix-bin /abs/path/to/quantix --env-file /abs/path/to/service.env`
  - 默认 service 配置路径 `~/.quantix/strategy/service.json`
  - 可选环境文件 `~/.quantix/strategy/service.env`
  - wrapper 路径 `~/.local/bin/quantix-strategy-run`
- **当前边界**
  - `strategy daemon` 不自动交易
  - `strategy run --mode paper` 仍保留为直接执行路径
  - `strategy daemon run --once` 首次启动只 bootstrap 到最新 bar，可能输出 `strategy daemon 未生成新信号`
  - 在 Phase 29B 本身，这些能力延后到后续 Phase；当前已由下文 Phase 29C 补齐 execution daemon 与基础自动审批，并补齐受 `qmt.mode=live` 保护的 `qmt_live` 实盘路径；通用 `target_mode=live` 仍未实现

#### Phase 29C: 执行自动化收口 ✅
- **执行自动化命令** (`src/execution/config.rs`, `src/execution/daemon.rs`, `src/cli/handlers.rs`)
  - `quantix execution config init`
  - `quantix execution config show`
  - `quantix execution daemon run`
  - `quantix execution daemon run --once`
- **执行配置持久化** (`src/execution/config.rs`)
  - 默认路径 `~/.quantix/execution/config.json`
  - 可通过 `QUANTIX_EXECUTION_CONFIG_PATH` 覆盖
- **自动审批与 request claim**
  - `execution_request` 当前新增 `in_progress`
  - `strategy daemon` 仍负责生成 signal 和可选 auto-approval
  - `execution daemon` 只消费 `pending execution_request`
  - `strategy request execute` 与 `execution daemon` 复用同一条 request 消费路径
  - 自动审批当前只支持 `manual|always`
  - `manual` 保持人工 `strategy signal approve`
  - `always` 会在 signal 生成后直接创建 `pending execution_request`
- **P0 约束**
  - `execution daemon` 当前是单 worker、串行消费
  - request 进入 `completed` 只表示成功进入执行层，不代表订单已终态
  - request completed 但订单仍非终态时，紧凑输出会额外带上 `semantics=request_completed_order_non_terminal`
  - `strategy request show` 会展示 `request_status`、`order_status`、`executed_at`、`failed_at`、`canceled_at` 等诊断字段；若存在结构化 `execution_diagnostics`，还会单独展示 `Execution Diagnostics` section
  - `quantix execution daemon run --once` 与 `strategy request list` 会在紧凑输出里附带 `executed_at`、`failed_at`、`canceled_at` 等诊断字段（若存在）
  - 非 completion 类结构化诊断会在紧凑输出里追加 `diag=<code>`，例如 `bridge_qmt_mode_not_live`
  - `request_completed_order_terminal` / `request_completed_order_non_terminal` 不会重复显示为 `diag=<code>`；非终态完成仍沿用 `semantics=request_completed_order_non_terminal`
  - `mock_live` request 即使返回 `accepted`，request 也会记为 `completed`
  - `mock_live` 继续承担 live-ready hardening / reconciliation scaffolding；reconciliation 会收敛 delayed fill、partial fill 与 `unknown` 恢复语义
  - `live` adapter 仍未实现
  - 通用 `target_mode=live` 仍未实现；当前真实提交只走受 `qmt.mode=live` 保护的 `qmt_live` 路径

### Windows Bridge v1
- **TDX bridge source**
  - 通过 Windows `quantix-bridge` 暴露远端行情与 K 线读取
  - 当前 bridge 的首个真实能力是 `TDX bridge source`
- **QMT preview path**
  - 当前只支持 frozen execution request 的 broker payload 预览
  - `QMT preview-only` 不会真实发单，也不会改写 request / order lifecycle
  - 此处的 `preview-only` 仅指 `qmt-preview` 预览路径，不代表整个 QMT 能力仍然只有预览
  - 真实 QMT 提交只会在 bridge 明确回报 `qmt.enabled=true`、`qmt.mode=live` 且 `qmt.supports` 包含 `order_submit` 时放行
  - 真实 `qmt-live` 提交会把对应 `execution_request` 写回为 `completed` 或 `failed`

#### Windows 侧目录
- `/mnt/d/mystocks/quantix/quantix_bridge`

#### Bridge 环境变量
- `QUANTIX_BRIDGE_BASE_URL`
- `QUANTIX_BRIDGE_API_KEY`

#### Bridge CLI
- 推荐入口: `quantix execution qmt`
- 兼容旧入口: `quantix execution bridge`
- `quantix execution qmt status [--checklist]`
- `quantix execution qmt preview --request-id <ID>`
- `quantix execution qmt live --request-id <ID> [--yes]`
- `quantix execution qmt query --order-id <ORDER_ID>`
- `quantix execution bridge status [--checklist]`
- `quantix execution bridge qmt-preview --request-id <ID>`
- `quantix execution bridge qmt-live --request-id <ID> [--yes]`
- `quantix execution bridge qmt-query --order-id <ORDER_ID>`
- 如需真实 QMT 提交，Windows bridge 必须先满足 `qmt.enabled=true`、`qmt.mode=live`，并确保 `qmt.supports` 包含 `order_submit`

#### 最小安全流程
- 先确认 request 目标是 `qmt_live`，不要把通用 `target_mode=live` 当成实盘入口
- 建议先运行 `quantix execution qmt status --checklist`，确认 bridge 已进入 `qmt.mode=live`、具备 `order_submit` 能力，并对照 promotion checklist 再决定是否进入实盘提交
- `quantix execution qmt preview --request-id <ID>` 只做 preview，不会真实发单
- 手动执行 `quantix execution qmt live --request-id <ID>` 时，CLI 会要求输入 `YES` 确认；只有显式传入 `--yes` 才会跳过确认
- 提交成功后，按 CLI 打印的 `quantix execution qmt query --order-id <ORDER_ID>` 继续核验券商侧订单状态
- 旧路径 `quantix execution bridge ...` 仍兼容，适合已有脚本渐进迁移

#### 当前边界
- 通用 `target_mode=live` 仍延后到后续 phase；真实 QMT 提交仅支持受 `qmt.mode=live` 保护的 `qmt_live` 路径

#### Phase 30: 股票异常检测 ✅
- **异常检测模块** (`src/anomaly/*`)
  - Isolation Forest 算法实现
  - 特征提取：成交量回报、对数回报、EOM 指标
  - 线性回归统计（斜率、R²、p值）
  - ✅ 28个单元测试通过
- **东方财富数据源** (`src/anomaly/eastmoney_source.rs`)
  - 真实 A 股列表获取（沪深主板、创业板、科创板）
  - K线数据获取（支持多周期：1/5/15/30/60分钟、日线）
  - 复权类型支持（前复权、后复权、不复权）
- **A股特有过滤器** (`src/anomaly/filter.rs`)
  - ST 股票过滤
  - 涨跌停检测（主板±10%、创业板/科创板±20%、北交所±30%）
  - 停牌股票检测
  - 新股过滤
- **CLI 命令**
  - `quantix anomaly run` - 使用东方财富 API 检测异常股票
  - `quantix anomaly run --mock` - 使用模拟数据测试
  - 支持多种输出格式（table/json/csv）
- **算法来源**
  - 移植自 Surpriver 项目
  - 理论基础：异常股票未来价格波动是正常股票的 2x+

### Phase 15: 具体策略实现 ✅
- **MA Cross 策略** (`src/strategy/ma_cross.rs`)
  - 完整实现 MA 金叉死叉逻辑
  - 可配置短期和长期均线周期
  - 自动持仓状态管理
  - ✅ 4个单元测试通过
- **Mean Reversion 策略** (`src/strategy/mean_reversion.rs`)
  - 基于 RSI 和布林带的均值回归
  - 可配置超买超卖阈值
  - 双重条件确认信号
  - ✅ 4个单元测试通过
- **Momentum 策略** (`src/strategy/momentum.rs`)
  - 基于 MACD 的动量跟踪
  - 可配置快慢线和信号线周期
  - 支持背离检测（预留）
  - ✅ 3个单元测试通过
- **Breakout 策略** (`src/strategy/breakout.rs`)
  - 价格突破 + 成交量确认
  - ATR 动态止损止盈
  - 支持做多和做空
  - ✅ 1个单元测试通过
- **Grid Trading 策略** (`src/strategy/grid.rs`)
  - 震荡市场网格交易
  - ATR 动态价格区间
  - 自动网格订单管理
  - 支持动态调整
  - ✅ 3个单元测试通过
- **测试工具模块** (`src/strategy/test_utils.rs`)
  - 可配置的测试数据生成器
  - 消除所有硬编码参数
  - 支持多种价格趋势生成
  - ✅ 4个单元测试通过
- **所有策略完全可配置** - 无硬编码参数
- **统一 Strategy trait** - 标准化接口
- **测试覆盖率** - 90% (19/21个函数)

#### Phase 16: 实时监控系统 ✅
- **信号监控** (`src/monitoring/signal_monitor.rs`)
  - 实时追踪策略交易信号
  - 信号统计分析（买入/卖出/观望计数）
  - 策略级别和股票级别统计
  - 信号频率统计（每分钟）
  - ✅ 10个单元测试通过
- **持仓监控** (`src/monitoring/position_monitor.rs`)
  - 实时持仓状态追踪
  - 持仓变化检测（新增/加仓/减仓/平仓/价格更新）
  - 持仓快照和盈亏计算
  - 持仓比例告警检查
  - ✅ 11个单元测试通过
- **性能监控** (`src/monitoring/performance_monitor.rs`)
  - 权益历史追踪
  - 实时性能指标计算（收益率、回撤、夏普比率等）
  - 回撤状态分级（Normal/Caution/Warning/Critical）
  - 交易盈亏记录和统计
  - ✅ 10个单元测试通过
- **告警系统** (`src/monitoring/alert.rs`)
  - 阈值告警机制
  - 多级告警（Info/Warning/Error/Critical）
  - 冷却时间机制
  - 告警历史和确认功能
  - 预定义阈值构建器
  - ✅ 15个单元测试通过
- **所有模块完全可配置** - 零硬编码
- **测试覆盖率** - 85% (46/46个测试通过)

#### Phase 17: 数据导入导出增强 ✅
- **数据导出器** (`src/io/exporter.rs`)
  - 多格式导出支持 (CSV, JSON, Parquet)
  - 可配置输出参数（精度、表头、日期格式）
  - Arrow/Parquet 列式存储集成
  - 导出结果跟踪（文件大小、记录数、执行时间）
  - ✅ 5个单元测试通过
- **数据导入器** (`src/io/importer.rs`)
  - 多格式导入支持 (CSV, JSON, Parquet)
  - 错误处理和无效行跳过
  - 性能指标（导入时长、跳过/错误计数）
  - 可选数据验证集成
  - ✅ 4个单元测试通过
- **数据验证器** (`src/io/validation.rs`)
  - 全面数据验证（价格、成交量、日期）
  - 价格逻辑验证（high ≥ low, close in range）
  - 数据质量评分（0-100分制）
  - 批量验证支持
  - ✅ 7个单元测试通过
- **批处理处理器** (`src/io/batch.rs`)
  - 大数据集内存优化批处理
  - 信号量限制并发控制
  - 实时进度条（indicatif）
  - 流式处理支持超大文件
  - ✅ 5个单元测试通过
- **所有配置完全可定制** - 无硬编码参数
- **测试覆盖率** - 100% (21/21个io模块测试通过)

#### Phase 18: 性能测试与优化 ✅
- **基准测试框架** (`benches/bench_main.rs`)
  - Criterion 集成，42个测试用例全部通过
  - 技术指标、导入导出、验证、批处理基准
  - 多规模测试（100-1M 条记录）
  - 自动化基线对比和回归检测
  - ✅ 性能基线已建立
- **性能基线数据** (`docs/archive/reports/PHASE18_BENCHMARK_RESULTS.md`)
  - 技术指标: SMA 1.54ms, MACD 5.57ms (10K条)
  - 数据导出: CSV 679K 记录/秒, JSON 593K 记录/秒
  - 性能计算: 总收益率 18.8M 次/秒
  - 批处理: 9.80-23.4M 记录/秒
- **性能优化工具** (`src/core/performance_utils.rs`)
  - PerfTimer 性能计时器
  - MemoryTracker 内存跟踪器
  - 优化建议生成器（批次大小、并行化、预分配）
  - 性能分析器（自动优化建议）
  - ✅ 3个单元测试通过
- **基准测试脚本** (`scripts/dev/run_benchmarks.sh`)
  - 一键运行、基线保存、性能对比
  - Flamegraph/DHAT 集成支持
  - HTML 报告生成
- **性能优化指南** (`docs/guides/PERFORMANCE_OPTIMIZATION.md`)
  - 基准测试解读、性能剖析工具使用
  - 优化策略（批量、并行、内存、算法）
  - CI/CD 集成、常见问题诊断
- **所有配置完全可定制** - 无硬编码参数
- **完整文档** - 使用指南、优化策略、工具集成
- **零错误完成** - 修复所有溢出问题，42/42 测试通过

#### Phase 19: 部署与运维 ✅
- **Docker 容器化**
  - `Dockerfile` - 生产环境多阶段构建（<200MB）
  - `Dockerfile.dev` - 开发环境热重载
  - `docker-compose.yml` - 完整服务栈（7个服务）
  - `docker-compose.prod.yml` - 生产环境配置
  - 非 root 用户运行、健康检查集成
- **CI/CD 增强** (`.github/workflows/`)
  - `docker.yml` - 多架构镜像构建（amd64/arm64）
  - `cleanup.yml` - 定期清理旧镜像
  - GitHub Container Registry 集成
  - Trivy 安全扫描
  - 自动部署和 GitHub Release
- **部署脚本** (`scripts/deploy/`)
  - 多环境支持（dev/staging/production）
  - 健康检查、模拟运行模式
  - 彩色输出和日志
- **监控配置** (`monitoring/`)
  - `prometheus.yml` - 指标采集配置
  - `alerts.yml` - 20+ 告警规则
  - `loki.yml` + `promtail.yml` - 日志聚合
- **数据库初始化**
  - `scripts/init-postgres.sql` - PostgreSQL 扩展和优化
  - `scripts/init-clickhouse.sql` - ClickHouse 表结构
- **完整文档**
  - `docs/guides/DOCKER_GUIDE.md` - Docker 部署指南
  - `docs/guides/PRODUCTION_DEPLOYMENT.md` - 生产部署指南
- **服务栈**: Quantix + PostgreSQL 17 + ClickHouse + Prometheus + Grafana + Loki + Promtail + Traefik

#### Phase 20: Zellij 集成和 CLI 增强 ✅
- **Zellij 配置** (`config/zellij/`)
  - `config.kdl` - 主配置文件（Catppuccin Mocha 主题）
  - 快捷键绑定（Alt+h/j/k/l 移动焦点，Alt+s/v 分割窗格）
  - quantix 专用快捷键（Alt+m 菜单，Alt+q 状态）
- **预设布局** (`config/zellij/layouts/`)
  - `main.kdl` - 主工作区（CLI + 状态监控 + 日志）
  - `monitor.kdl` - 监控工作区（4窗格实时监控）
  - `backtest.kdl` - 回测工作区（控制台 + 结果 + 日志）
  - `dev.kdl` - 开发工作区（代码 + 构建 + Git）
- **启动脚本** (`scripts/zellij/`)
  - `start-session.sh` - 会话启动（自动检测现有会话）
  - `install.sh` - 一键安装（支持 cargo/包管理器）
  - `status-collector.sh` - 状态采集（CPU/内存/数据库/任务）
- **完整文档** (`docs/guides/ZELLIJ_GUIDE.md`)
  - 安装指南、快捷键参考、布局说明
  - 常见问题、最佳实践
- **Rust 原生终端复用器** - 与项目技术栈一致

#### Phase 31: tdx-api REST 数据源桥接 ✅
- **tdx-api REST 客户端** (`src/sources/tdx_api.rs`)
  - `TdxApiClient` - 通过 `TDX_API_URL` 环境变量连接 tdx-api Docker 服务
  - 实时行情、K 线（原始/THS 前复权）、分时、搜索、交易日历、逐笔成交
  - 市场涨跌统计、N日收益计算、异步任务管理
- **CLI 子命令** (`src/cli/handlers/tdx_api_handler.rs`)
  - 18 个 `quantix data tdx-api <subcommand>` 子命令
  - `import-klines` — 单股或 `--all` 批量导入 THS 前复权 K 线到 ClickHouse，增量跳过
  - `import-ticks` — 逐笔成交数据导入 TDengine tick_data 超级表
  - `sync-calendar` — 交易日历同步到 `config/holidays.json`
- **ClickHouse 扩展** (`src/db/clickhouse/kline.rs`)
  - `insert_kline_data_batch_with_source()` — 支持自定义 source（THS_QFQ/TDX）
  - `get_latest_kline_date()` — 增量导入检查
- **TDengine 扩展** (`src/db/tdengine.rs`)
  - `create_tick_table()` / `insert_ticks()` — 逐笔成交超级表
  - `execute_sql()` — 通用 REST SQL 执行
- **Docker Compose 集成** (`docker-compose.yml`)
  - tdx-api 服务定义，quantix 服务 `TDX_API_URL` 环境变量
- **每日更新脚本** (`scripts/daily-update.sh`)
  - `sync-calendar` + `import-klines --all` 定时同步
  - 建议定时任务: `0 18 * * 1-5 /opt/claude/quantix-rust/scripts/daily-update.sh --all`

```
quantix-rust/
├── src/
│   ├── analysis/        # 分析模块
│   │   ├── auction.rs       # 竞价分析
│   │   ├── backtest.rs      # 回测引擎
│   │   ├── indicators.rs    # 技术指标
│   │   ├── portfolio.rs     # 投资组合管理
│   │   └── performance.rs   # 性能计算
│   ├── cli/             # CLI 命令行
│   ├── core/            # 核心模块 (配置、错误、交易日历)
│   ├── data/            # 数据模型
│   ├── db/              # 数据库客户端 (PostgreSQL, TDengine, ClickHouse)
│   ├── io/              # 数据导入导出 (Phase 17)
│   ├── monitoring/      # 实时监控系统 (Phase 16)
│   ├── sources/         # 数据源 (TDX, AkShare, tdx-api REST, 行情采集, K线聚合)
│   ├── strategy/        # 交易策略
│   ├── sync/            # 数据同步 ETL
│   ├── tasks/           # 任务调度 (Cron, Scheduler)
│   └── tui/             # 终端 UI (可选)
├── Cargo.toml
└── README.md
```

## 开发指南

### 📚 开发规范

**重要**: 所有贡献者和开发者必须阅读 [开发规范指南](docs/standards/DEVELOPMENT_GUIDELINES.md)

该规范文档包含：
- ✅ 核心编码规则（所有权、错误处理、类型安全）
- ✅ 量化交易特殊注意事项（性能优化、安全性、CLI交互）
- ✅ 测试规范（单元测试、集成测试、回测验证）
- ✅ 收口阶段规则（先完成运行门禁闭环，再做后续清理）
- ✅ 性能优化指南（编译优化、基准测试）
- ✅ 安全与稳定性（依赖安全、资源管理、并发安全）
- ✅ 代码质量工具（rustfmt、clippy、CI/CD）

**收口阶段强制规则**:
- 当任务已经进入“可收口”阶段后，必须优先完成运行门禁闭环。
- 在门禁未闭环前，不得继续扩散为零散 cosmetic 微调、顺手重构或机械性清理。
- 任何继续修改都必须直接服务于失败门禁、验收阻塞或交付风险。

### 代码质量检查

```bash
# 格式化代码
cargo fmt

# 检查代码质量
cargo clippy -- -D warnings

# 运行所有测试
cargo test --all-features

# 检查依赖漏洞
cargo install cargo-audit
cargo audit
```

### 环境要求
- Rust 1.70+
- PostgreSQL 17+ (可选)
- TDengine 3.3+ (可选)
- ClickHouse (推荐用于 OLAP 分析)

### 构建运行

```bash
# 克隆项目
git clone https://github.com/chengjon/quantix-rust.git
cd quantix-rust

# 开发运行
cargo run -- --help

# 构建
cargo build --release
```

### 配置

通过环境变量配置数据库连接：

```bash
# PostgreSQL
export POSTGRES_URL="postgresql://localhost:5432/quantix"

# ClickHouse
export CLICKHOUSE_URL="http://localhost:8123"
export CLICKHOUSE_DB="quantix"

# TDX 数据源
export TDX_HOST="192.168.1.100"
export TDX_PORT=7709

# 自选池 JSON 存储路径（可选）
export QUANTIX_WATCHLIST_PATH="$HOME/.quantix/watchlist/watchlist.json"
export QUANTIX_RISK_PATH="$HOME/.quantix/risk/risk_state.json"
```

### 运行测试

```bash
# 所有测试
cargo test --all-features

# 单个模块测试
cargo test --package quantix-cli --lib analysis::backtest::tests

# 运行文档测试
cargo test --doc

# 运行基准测试
cargo bench
```

## 使用示例

### 自选池 CLI

```bash
# 创建分组
quantix watchlist group create --name core

# 添加股票并打标签
quantix watchlist add --code 000001 --group core
quantix watchlist tag add --code 000001 --tag bank

# 查看列表与历史
quantix watchlist list --group core
quantix watchlist list --with-price
quantix watchlist history --code 000001 --limit 20
```

### 选股筛选 CLI

```bash
# 查看可用 preset
quantix analyze screener preset-list

# 对显式代码列表做单条件筛选
quantix analyze screener run \
  --codes 000001,600519 \
  --preset close_above_ma:period=20

# 多个单指标 preset 做 AND 组合
quantix analyze screener run \
  --watchlist \
  --group core \
  --preset close_above_ma:period=20 \
  --preset volume_ratio_gte:window=5,value=1.5 \
  --sort-by score \
  --limit 20
```

Preset 参数必须是完整的 `key=value` 片段；尾随逗号、连续逗号、重复 key、零周期/窗口、`NaN`/`inf` 阈值和 lookback 溢出都会被解析层或运行时边界拒绝。

`--sort-by` 仅支持 `code` 或 `score`。未知排序字段会在读取 ClickHouse 日线数据或输出筛选表格前返回显式 `Unsupported`，错误包含 `不支持的 sort_by`、被拒绝字段和支持列表 `code, score`。

### 市场分析 CLI

```bash
# 查看全市场基础摘要
quantix market foundation

# 查看行业和概念板块
quantix market sector --top 10
quantix market concept --date 2026-03-09

# 查看北向资金和市场情绪
quantix market north --date 2026-03-09
quantix market sentiment

# 查看龙头股和综合概览
quantix market leader --sector 银行 --limit 5
quantix market overview --top 5

# 分析强势/弱势板块，并查看强势板块个股 Top10
quantix market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10

# 直接查看强势板块中的银行股排行
quantix market strength-stocks --date 2026-03-09 --strong-top 3 --sector 银行 --metric market-cap --top 10
quantix market strength-stocks --date 2026-03-09 --strong-top 3 --sector 银行 --metric profit --top 10
```

- 当前只文档化已经实现的 Phase 23 P0 行为。
- 运行 `foundation` / `strength` 前，先执行 `quantix risk sync industry --standard shenwan`。
- 历史/详情/实时功能延后到后续 Phase。

### 因子研究 CLI

```bash
# 查看已登记因子
quantix factor list

# 基于本地 CSV 计算因子
quantix factor compute \
  --input bars.csv \
  --factor rank_close \
  --format csv \
  --output factor_values.csv

# 多因子等权评分最新截面
quantix factor score \
  --input bars.csv \
  --factor rank_close ts_rank_close_5 \
  --format csv \
  --output factor_scores.csv

# 评估单个因子 IC/IR
quantix factor evaluate \
  --input bars.csv \
  --factor rank_close \
  --horizon 1 \
  --format json
```

### 回测示例

```rust
use quantix_cli::analysis::backtest::{BacktestEngine, BacktestConfig};
use quantix_cli::strategy::ma_cross::MACrossStrategy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = BacktestConfig::default();
    let mut engine = BacktestEngine::new(config);

    let mut strategy = MACrossStrategy::new(5, 20); // MA5, MA20

    // 加载历史数据
    let data = load_kline_data("000001").await?;

    // 运行回测
    let result = engine.run(&mut strategy, &data).await?;

    println!("总收益率: {}%", result.report.total_return * 100);
    println!("夏普比率: {}", result.report.sharpe_ratio);

    Ok(())
}
```

### 任务调度示例

```rust
use quantix_cli::tasks::{TaskScheduler, TaskTemplates};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = TaskScheduler::new().await?;

    // 添加预设任务
    scheduler.add_task(TaskTemplates::market_open()).await?;
    scheduler.add_task(TaskTemplates::market_close()).await?;

    // 启动调度器
    scheduler.start().await?;

    // 保持运行
    tokio::signal::ctrl_c().await?;
    scheduler.stop().await?;

    Ok(())
}
```

## 技术栈

| 类别 | 技术 |
|------|------|
| 语言 | Rust 2024 Edition |
| 异步运行时 | tokio 1.35 |
| 数据库 | PostgreSQL, TDengine, ClickHouse |
| DataFrame | Polars 0.43 (批量数据处理) |
| 序列化 | serde, serde_json |
| 时间处理 | chrono |
| 数值计算 | rust_decimal |
| HTTP 客户端 | reqwest |
| WebSocket | tokio-tungstenite |
| 日志 | tracing, tracing-subscriber |
| CLI | clap, dialoguer, indicatif |
| 终端复用 | Zellij (Rust 原生) |
| 容器化 | Docker, Docker Compose |
| 监控 | Prometheus, Grafana, Loki |

## 项目进度

**✅ 历史基础阶段 1-20 已完成**

后续阶段按里程碑持续演进；当前 README 已补充 `Phase 29` / `29B` / `29C` / `30` 等执行主线与扩展能力说明。

| Phase | 模块 | 状态 |
|-------|------|------|
| 1-5 | 数据采集、竞价分析、K线管理、回测引擎、任务调度 | ✅ |
| 6-10 | TDX解析、GBBQ存储、多周期查询、东财采集、ClickHouse优化 | ✅ |
| 11-15 | WebSocket、技术指标、Polars、CLI命令、策略实现 | ✅ |
| 16-18 | 实时监控、导入导出、性能测试与优化 | ✅ |
| 19-20 | 部署与运维（Docker/CI-CD）、Zellij集成 | ✅ |

## 与 Python quantix 的关系

```
┌─────────────────────────────────────────────────────────┐
│                     Python quantix                      │
│  (数据采集、存储、Web API、完整业务逻辑)                 │
└─────────────────────────────────────────────────────────┘
                          ↕ 共享数据库
┌─────────────────────────────────────────────────────────┐
│                      quantix-rust                       │
│  (高性能回测、实时分析、任务调度)                         │
└─────────────────────────────────────────────────────────┘
```

- **共享数据源**: PostgreSQL, TDengine
- **共享数据格式**: 一致的数据模型和序列化格式
- **互补定位**:
  - Python: 快速开发、完整业务功能
  - Rust: 高性能计算、低延迟分析

## 许可证

MIT License

## 作者

MyStocks Team

## 文档

- [开发规范指南](docs/standards/DEVELOPMENT_GUIDELINES.md) - 必读！
- [用户手册](docs/USER_MANUAL.md)
- [GitNexus MCP 日常使用建议](docs/guides/GITNEXUS_MCP_DAILY_WORKFLOW_RECOMMENDATIONS.md)
- [Python quantix](https://github.com/chengjon/mystocks)
- [问题反馈](https://github.com/chengjon/quantix-rust/issues)

## CI/CD

项目使用 GitHub Actions 进行持续集成：
- ✅ 代码格式检查（rustfmt）
- ✅ 代码质量检查（clippy）
- ✅ 单元测试和集成测试
- ✅ 安全审计（cargo audit）
- ✅ 依赖版本检查
- ✅ 多平台构建（Linux/macOS/Windows）
- ✅ 文档生成和部署
