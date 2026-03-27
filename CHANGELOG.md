# Changelog

All notable changes to this project are documented here.

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
