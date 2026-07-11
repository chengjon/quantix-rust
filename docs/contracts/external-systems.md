# 外部系统合同文档

> **文档类型**: Reference — 三层合同（系统层 / 服务层 / 合约层）
> **审核日期**: 2026-07-11
> **主语言**: 中文

---

## 目录

1. [第一层：系统总览与分类](#第一层系统总览与分类)
2. [第二层：各系统合约详情](#第二层各系统合约详情)
3. [第三层：共享合约与原语](#第三层共享合约与原语)
4. [附录 A：环境变量索引](#附录-a环境变量索引)
5. [附录 B：Feature Flag 映射](#附录-bfeature-flag-映射)

---

## 第一层：系统总览与分类

quantix-rust 共集成 **27 个外部系统**，分为 7 大类：

| 分类 | 数量 | 系统列表 |
|------|------|----------|
| 数据存储 (Storage) | 4 | ClickHouse, PostgreSQL, TDengine, SQLite |
| 市场行情 (Market Data) | 7 | OpenStock, TDX TCP, Bridge TDX, EastMoney HTTP, AkShare, WebSocket, Kline Aggregator |
| 交易执行 (Execution) | 4 | Windows Bridge, QMT Live, QMT Preview, MiniQMT |
| AI/LLM | 3 | DeepSeek, OpenAI, Ollama (统一 OpenAI Compatible) |
| 新闻搜索 (News) | 3 | Tavily, SerpAPI, Bocha (博查) |
| 基本面 (Fundamental) | 1 | EastMoney Fundamental (含估值/财报/机构/龙虎榜) |
| 通知推送 (Notification) | 5 | 飞书, 企业微信, 桌面通知, Webhook, 日志 |

### 系统间依赖关系

```
用户 CLI / TUI
    │
    ├── Data Storage ──────────► ClickHouse (主存储)
    │                          ├── PostgreSQL (只读, 与 Python 共享)
    │                          ├── TDengine (高频分钟K线)
    │                          └── SQLite (本地, feature-gated)
    │
    ├── Market Data ───────────► OpenStock API ◄── 内网 HTTP
    │                          ├── TDX TCP (rustdx-complete) ◄── 通达信服务器
    │                          ├── Bridge TDX ◄── Windows Bridge HTTP
    │                          ├── EastMoney HTTP ◄── 东方财富
    │                          ├── AkShare HTTP (骨架/未接入)
    │                          └── WebSocket (实时推送)
    │
    ├── Trading/Execution ─────► Windows Bridge HTTP ◄── WSL → Windows
    │                          │   ├── QMT Preview (dry-run)
    │                          │   ├── QMT Live (实盘)
    │                          │   └── Task Contract (版本协商 + 轮询)
    │                          └── MiniQMT (独立行情模块)
    │
    ├── AI/LLM ────────────────► DeepSeek API
    │                          ├── OpenAI API
    │                          └── Ollama (本地)
    │
    ├── News ──────────────────► Tavily API
    │                          ├── SerpAPI
    │                          └── Bocha API
    │
    ├── Fundamental ───────────► EastMoney HTTP (估值/财报/机构/龙虎榜)
    │
    └── Notification ──────────► 飞书 Webhook
                               ├── 企业微信 Webhook
                               ├── Desktop (本地通知)
                               ├── Webhook (通用)
                               └── Log (本地日志)
```

---

## 第二层：各系统合约详情

### 2.1 ClickHouse（主存储）

#### 连接参数

| 参数 | 环境变量 | 默认值 | 说明 |
|------|----------|--------|------|
| URL | `QUANTIX_CLICKHOUSE_URL` | `http://localhost:8123` | HTTP 协议地址 |
| Database | `QUANTIX_CLICKHOUSE_DB` | `quantix` | 数据库名 |
| User | `QUANTIX_CLICKHOUSE_USER` | `default` | 用户名 |
| Password | `QUANTIX_CLICKHOUSE_PASSWORD` | `""` | 密码 |

**协议**: HTTP (clickhouse-rs 0.12 crate, `lz4` + `time` features)
**批处理**: RowBinary 格式，批次大小 1000 行

#### 核心表结构

| 表名 (ClickHouse) | 用途 | 排序键 | 引擎 |
|-------------------|------|--------|------|
| `stock_info` | 股票基本信息 | (market, code) | ReplacingMergeTree |
| `stock_realtime_quotes` | 实时行情快照 | (date, code, timestamp) | MergeTree |
| `kline_data` | K线数据 | (code, trade_date) | ReplacingMergeTree |
| `limit_up_events` | 涨停事件 | (trade_date, code) | MergeTree |
| `gbbq_events` | 股本变迁事件 | - | - |
| `market_fundamental_snapshot` | 市场基本面快照 | - | - |
| `minute_klines` | 分钟K线 | - | - |
| `minute_shares` | 分钟成交额 | - | - |

**源码**: `src/db/clickhouse/mod.rs`, `src/db/clickhouse/schema.rs`, `src/db/clickhouse/models.rs`

#### 数据模型

| Rust 结构体 | ClickHouse 表 | 文件 |
|------------|---------------|------|
| `StockInfoCH` | `stock_info` | `models.rs` |
| `StockQuoteCH` | `stock_realtime_quotes` | `models.rs` |
| `KlineDataCH` | `kline_data` | `models.rs` |
| `LimitUpEventCH` | `limit_up_events` | `models.rs` |
| `GbbqEventCH` | `gbbq_events` | `models.rs` |
| `MarketFundamentalSnapshotCH` | `market_fundamental_snapshot` | `models.rs` |
| `MarketSentimentDailyCH` | `market_sentiment_daily` | `models.rs` |
| `NorthFlowDailyCH` | `north_flow_daily` | `models.rs` |
| `SectorDailyCH` | `sector_daily` | `models.rs` |
| `MinuteKlineCH` | `minute_klines` | `models.rs` (pub use from minute.rs) |
| `MinuteShareCH` | `minute_shares` | `models.rs` (pub use from minute.rs) |

#### 依赖

```toml
clickhouse = { version = "0.12", features = ["lz4", "time"] }
```

---

### 2.2 PostgreSQL（只读，与 Python 项目共享）

#### 连接参数

| 参数 | 环境变量 | 默认值 | 说明 |
|------|----------|--------|------|
| Database URL | `QUANTIX_POSTGRES_URL` | - | 完整连接字符串 |
| Max Connections | - | 10 | 连接池上限 |

**协议**: PostgreSQL (sqlx 0.8, `postgres` feature)
**访问模式**: 只读（读取 Python quantix 项目写入的存量数据）

#### 数据模型

| Rust 结构体 | 数据库表 | 说明 |
|------------|---------|------|
| `KlineDaily` | `kline_daily` | 日K线（前复权/后复权/不复权） |
| `StockInfo` | `stock_info` | 股票名称、市场、上市/退市日期 |

**源码**: `src/db/postgresql.rs`

#### 依赖

```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "chrono", "rust_decimal", "json", "sqlite"] }
```

---

### 2.3 TDengine（高频分钟K线）

#### 连接参数

| 参数 | 环境变量 | 默认值 | 说明 |
|------|----------|--------|------|
| Base URL | `QUANTIX_TDENGINE_URL` | - | TDengine REST API 地址 |
| Token | `QUANTIX_TDENGINE_TOKEN` | - | 认证令牌 |
| Database | (可选) | - | 指定数据库 |

**协议**: REST API（`tdengine-rest` feature，默认启用）
**备选协议**: WebSocket（`tdengine-ws` feature，未启用）
**传输**: HTTP POST, `application/x-www-form-urlencoded` 格式 SQL

#### 数据模型

| Rust 结构体 | 说明 |
|------------|------|
| `TDengineRestResponse` | REST API 响应（status + data row array） |
| `TdengineRow` | 单行分钟K线（ts, code, open/high/low/close/volume） |
| `TDengineClient` | 客户端封装 |
| `MinuteKline` | 标准化分钟K线（ts, code, OHLCV） |

**源码**: `src/db/tdengine.rs`

---

### 2.4 SQLite（本地存储，可选）

**协议**: sqlx SQLite 驱动
**Feature Gate**: `sqlite`（`sqlx/sqlite`）
**用途**: 本地离线数据缓存
**状态**: feature-gated，默认不启用

---

### 2.5 OpenStock API

#### 连接参数

| 参数 | 环境变量 | 默认值 | 说明 |
|------|----------|--------|------|
| Base URL | - | (配置提供) | HTTP API 基地址 |
| API Key | - | (配置提供) | X-API-Key 认证 |
| Timeout | - | 30s | 请求超时 |
| Max Retries | - | 3 | 最大重试次数（指数退避） |
| Circuit Break | - | 5次/30s冷却 | 熔断阈值 |

**协议**: HTTP/HTTPS REST
**认证**: `X-API-Key` HTTP Header
**端点**: `/data/fetch`（统一数据取接口）
**重试策略**: 指数退避, base 500ms, 3次重试
**熔断**: 共享 circuit breaker, 连续5次失败触发, 30s 冷却

#### 数据模型

| Rust 结构体 | 源码 |
|------------|------|
| `OpenStockClient` | `src/sources/openstock_client.rs` |
| `OpenStockEnvelope` | `src/sources/openstock_envelope.rs` |
| `OpenStockErrorEnvelope` | `src/sources/openstock_envelope.rs` |

#### 子模块

| 功能 | 文件 | 说明 |
|------|------|------|
| K线数据 | `src/sources/openstock.rs` | 日线/分钟线获取 |
| 日历 | `src/sources/openstock_calendar.rs` | 交易日历 |
| 股票代码 | `src/sources/openstock_codes.rs` | 代码列表 |
| 指数 | `src/sources/openstock_index.rs` | 指数行情 |
| 市场数据 | `src/sources/openstock_market.rs` | 大盘数据 |
| 快照影子数据 | `src/sources/openstock_shadow.rs` | 影子库同步 |
| Tick数据 | `src/sources/openstock_ticks.rs` | 逐笔成交 |
| 信封响应 | `src/sources/openstock_envelope.rs` | 通用响应封装 |

#### 测试夹具（Fixture Contract）

`tests/fixtures/openstock/*.json` 是 OpenStock 响应的快照样本，分为两类形态：

**形态 A — `OpenStockEnvelope<T>` 包络形态**（生产响应路径，由
`OpenStockResponse::from_envelope` 反序列化）：

| 夹具文件 | `data_category` | 用途 |
|---------|-----------------|------|
| `codes.json` | `STOCK_CODES` | 单标的基础信息（含 envelope metadata） |
| `codes_empty.json` | (省略) | 空响应，验证 `data: []` + 仅 `source` 的退化形态 |
| `all_stocks.json` | `ALL_STOCKS` | 全市场股票清单（含 market / listing_date） |
| `trade_dates.json` | `TRADE_DATES` | 交易日历（`calendar_date` + `is_trading_day`） |
| `trade_dates_empty.json` | (省略) | 空响应 |
| `workdays.json` | `WORKDAYS` | 工作日 action-driven 形态 |
| `index_klines.json` | `INDEX_KLINES` | 指数 K 线（symbol + time + OHLCV 字符串） |
| `index_klines_empty.json` | (省略) | 空响应 |

**形态 B — 旧版直记录形态**（仅 `parse_daily_kline_json` 解析器使用，
非生产 envelope 路径）：

| 夹具文件 | 形态 | 用途 |
|---------|------|------|
| `daily_kline.json` | `{provider, period, adjust_type, records:[...]}` | 单条 K 线日数据，字段均为字符串 |
| `daily_kline_30d.json` | 同上 | 30 天 K 线序列 |

> 形态 B 不符合 `OpenStockEnvelope` 契约，属于独立 parser 的专用 fixture。
> 任何将形态 B 重构进 envelope 路径的 PR，必须先迁移这两个夹具或显式标注双轨期。

<!-- L2:FIXTURE_INVENTORY -->
```
codes.json codes_empty.json all_stocks.json trade_dates.json trade_dates_empty.json workdays.json index_klines.json index_klines_empty.json daily_kline.json daily_kline_30d.json
```
<!-- /L2 -->

---

### 2.6 TDX TCP（通达信直连）

**协议**: TCP (rustdx-complete crate)
**用途**: 实时行情采集
**状态**: 活跃（从短线侠项目迁移而来）

| Rust 结构体 | 文件 |
|------------|------|
| `StockQuote` | `src/sources/tdx.rs` |
| `Tdx` / `Tcp` | 来自 `rustdx_complete::tcp` |

**依赖**: `rustdx-complete`（外部 crate）

---

### 2.7 Bridge TDX（Windows Bridge 行情代理）

**协议**: HTTP REST → Windows Bridge
**端点**:

| 方法 | URL | 说明 |
|------|-----|------|
| `GET` | `/api/v1/capabilities` | 能力探测（含 TDX + QMT 两段） |
| `POST` | `/api/v1/data/tdx/quotes` | 批量报价查询 |
| `GET` | `/api/v1/data/tdx/kline/{symbol}` | K线查询（period/start/end） |

**认证**: `X-Quantix-Api-Key` Header（legacy JSON 路径）
**源码**: `src/bridge/client.rs`（方法 `capabilities`, `fetch_tdx_quotes`, `fetch_tdx_kline`）

#### 数据模型

| Rust 结构体 | 说明 |
|------------|------|
| `BridgeCapabilitiesResponse` | 能力探测响应（tdx + qmt） |
| `BridgeCapabilitySection` | 单段能力描述（enabled + supports） |
| `BridgeQmtCapabilitySection` | QMT 能力段（含 mode: paper/live） |
| `BridgeQuotesResponse` | 报价响应（quotes 列表） |
| `BridgeQuotePayload` | 单条报价（symbol/name/last/bid/ask/OHLCV） |
| `BridgeKlineResponse` | K线响应（symbol/period/bars/source） |
| `BridgeKlineBarPayload` | 单根K线（datetime/OHLC/volume/turnover） |

---

### 2.8 EastMoney HTTP（实时行情 + 基本面）

**协议**: HTTP REST
**用途**: 实时行情快照、资金流向

| Rust 结构体 | 文件 |
|------------|------|
| `EastMoneySource` | `src/sources/eastmoney.rs` |

> 基本面数据见 2.14 EastMoney Fundamental。

---

### 2.9 AkShare（骨架/未接入）

**协议**: HTTP REST（reqwest）
**状态**: **骨架状态** — 所有方法返回 `Unsupported` 错误
**用途**: 预留数据源
**文件**: `src/sources/akshare.rs`

---

### 2.10 WebSocket

**协议**: WebSocket
**用途**: 实时数据推送
**文件**: `src/sources/websocket.rs`

---

### 2.11 Kline Aggregator

**用途**: K线聚合器，合并多种数据源
**文件**: `src/sources/kline_aggregator.rs`

---

### 2.12 Windows Bridge（桥梁服务）

#### 连接参数

| 参数 | 环境变量 | 默认值 | 说明 |
|------|----------|--------|------|
| Base URL | `QUANTIX_BRIDGE_URL` | `http://127.0.0.1:17580` | Bridge HTTP 地址 |
| API Key | `QUANTIX_BRIDGE_API_KEY` | None | Legacy 认证 |
| Bearer Token | `QUANTIX_BRIDGE_BEARER_TOKEN` | None | Task Contract 认证 |
| Contract Version | `QUANTIX_BRIDGE_CONTRACT_VERSION` | `miniqmt.v1` | 协议版本 |
| Timeout | `QUANTIX_BRIDGE_TIMEOUT_MS` | 30000 | 超时 (ms) |
| Poll Interval | `QUANTIX_BRIDGE_POLL_INTERVAL_MS` | 1000 | 轮询间隔 (ms) |
| Poll Timeout | `QUANTIX_BRIDGE_POLL_TIMEOUT_MS` | 30000 | 轮询总超时 (ms) |

**物理路径** (canonical): `/mnt/d/mystocks/quantix/quantix_bridge`（Windows 侧）

#### 双认证路径

| 路径 | Header | 使用场景 |
|------|--------|----------|
| Legacy JSON | `X-Quantix-Api-Key` | TDX 行情、QMT preview、订单查询 |
| Task Contract | `X-Bridge-Contract-Version` + Bearer/API Key | QMT live 任务提交与结果查询 |

#### 任务合约端点

| 方法 | URL | 说明 |
|------|-----|------|
| `POST` | `/api/v1/task/execute` | 提交任务（submit_order） |
| `GET` | `/api/v1/task/result` | 查询任务结果 |

#### QMT 实盘端点

| 方法 | URL | 说明 |
|------|-----|------|
| `POST` | `/api/v1/broker/qmt/orders/preview` | 订单预览（dry-run） |
| `POST` | `/api/v1/broker/qmt/orders` | 提交实盘订单 |
| `GET` | `/api/v1/broker/qmt/orders/{order_id}` | 查询订单状态 |
| `DELETE` | `/api/v1/broker/qmt/orders/{order_id}` | 撤销订单 |
| `GET` | `/api/v1/broker/qmt/account/status` | 查询账户状态 |
| `GET` | `/api/v1/broker/qmt/assets` | 查询资产 |
| `GET` | `/api/v1/broker/qmt/positions` | 查询持仓 |

#### 运行时契约 (Behavioral Contract)

结构对齐不足以保证生产安全，下列运行时规则是与 Bridge server 的强约束。
任一规则被破坏视同 contract version Major bump。

**幂等键 (Idempotency Keys)** — 三元组构成下单请求的全局唯一幂等键：

| 字段 | 含义 | 违反后果 |
|------|------|----------|
| `request_id` | 内部生成 UUID，单次请求主标识 | 重复下单 |
| `client_order_id` | 策略侧业务单号 | 重复下单 |
| `local_submission_id` | 本地提交追踪 ID | 重复下单 |

> Bridge server 必须基于此三元组做去重；任一字段变化视同新单。
> 客户端重试必须复用全部三个字段，不可重新生成。

**可安全重试的错误码** — 仅下列 `BridgeFailureCode` 变体触发客户端自动重试：

| 错误码 | 重试策略 | 落地行为 |
|--------|----------|----------|
| `LiveBridgeTimeout` | 指数退避，base 500ms，最多 3 次 | 重试需复用幂等键三元组 |
| `LiveBridgeUnavailable` | 同上 | 同上 |
| `LiveBridgeAuthFailed` | **不重试** | 立即告警，疑似凭据丢失 |
| `LiveBridgeUnsupportedContractVersion` | **不重试** | 降级到只读 capabilities 端点 |
| `LiveBridgeUnsupportedMethod` | **不重试** | 调用方法本身不被服务端识别 |
| `LiveBridgeInvalidResult` | **不重试** | 服务端返回结构不合规，疑似契约漂移 |
| `LiveBridgeIdentityMismatch` | **不重试** | 单据身份不匹配，需人工介入 |

**超时与熔断**：

| 参数 | 默认值 | 越界行为 |
|------|--------|----------|
| 单请求超时 | 30s (`QUANTIX_BRIDGE_TIMEOUT_MS`) | 超时视为 `LiveBridgeTimeout`，可重试 |
| 轮询间隔 | 1s (`QUANTIX_BRIDGE_POLL_INTERVAL_MS`) | <100ms 视为 aggressive，触发 bridge 限流 |
| 轮询总超时 | 30s (`QUANTIX_BRIDGE_POLL_TIMEOUT_MS`) | 超时放弃 + 标记 task 为 Failed |
| 任务结果轮询次数上限 | `POLL_TIMEOUT / POLL_INTERVAL` | 超限 → `BridgePollTimeout` 错误 |

**限流 (Rate Limiting)** — Bridge server 侧实施；客户端约定：

- 下单类写操作：≤ 5 req/s（每秒 5 单），超出由 server 返回 429
- 查询类读操作：≤ 20 req/s
- 客户端收到 429 时，必须指数退避（base 1s，最多 3 次重试），不得固定 sleep

**版本协商**：

- 客户端请求带 `X-Bridge-Contract-Version: miniqmt.v1`
- 服务端拒绝时返回 `LiveBridgeUnsupportedContractVersion`，客户端必须**降级**：只调用 `/api/v1/capabilities` 端点，不发起任何下单请求
- 服务端 capabilities 响应中的 `contract_version` 字段为该实例支持的最高版本

**源码**: `src/bridge/client.rs`, `src/bridge/models.rs`, `src/bridge/error.rs`

---

### 2.13 QMT 执行适配器

#### QMT Preview（订单预览）

| 组件 | 文件 |
|------|------|
| `QmtBridgePreviewAdapter` | `src/execution/qmt_bridge.rs` |
| `QmtBridgePreviewResponse` | `src/execution/qmt_bridge.rs` |

**流程**: `ExecutionRequestRecord` → `BridgeQmtPreviewRequest` → Bridge preview → `QmtBridgePreviewResponse`

#### QMT Live（实盘执行）

| 组件 | 文件 |
|------|------|
| `QmtLiveExecutionAdapter` | `src/execution/qmt_live_adapter.rs` |
| `QmtTaskSubmitService` | `src/execution/qmt_task_submit_service.rs` |
| `QmtLiveGate` | `src/execution/qmt_live_gate.rs` |

**安全**: 只在 `BRIDGE_QMT_MODE=live` 时可用
**提交流**: `AdapterOrderRequest` → `BridgeTaskExecuteRequest` → Bridge → `QmtTaskSubmitReceipt`

#### 任务生命周期

```
Client → Bridge:   POST /task/execute  (BridgeTaskExecuteRequest)
Bridge → Client:   BridgeTaskAccepted  (task_id + contract_version)
Client → Bridge:   GET /task/result   (轮询直到终态)
```

| 状态 | 说明 |
|------|------|
| `Pending` | 已派发未完成 |
| `Completed` | 成功完成 |
| `Failed` | 失败 |
| `BridgeTaskAccepted` | Bridge已受理但未派发 |

**源码**: `src/execution/qmt_bridge.rs`, `src/execution/qmt_live_adapter.rs`, `src/execution/qmt_task_submit_service.rs`, `src/execution/qmt_live_gate.rs`

---

### 2.14 MiniQMT（独立行情）

**用途**: 通过 MiniQMT 终端获取实时行情和 K 线数据
**文件**: `src/miniqmt_market/`（含 `selection.rs`）
**源码**: `src/miniqmt_market.rs`（lib 层入口）

---

### 2.15 AI/LLM: OpenAI Compatible

**协议**: HTTP REST, OpenAI-compatible Chat Completions API
**支持的提供商**:

| 提供商 | base_url |
|--------|----------|
| DeepSeek | `https://api.deepseek.com/v1` |
| OpenAI | `https://api.openai.com/v1` |
| Ollama (本地) | `http://localhost:11434/v1` |

#### 适配器

| Rust 结构体 | 文件 |
|------------|------|
| `OpenAICompatAdapter` | `src/ai/providers/openai_compat.rs` |
| `LlmAdapter` trait | `src/ai/adapter.rs` |

#### 消息格式

遵循 OpenAI Chat Completions schemas（message, tool_call, token_usage 等）

**源码**: `src/ai/providers/openai_compat.rs`, `src/ai/types.rs`

---

### 2.16 新闻搜索提供商

#### 2.16.1 Tavily

| 属性 | 值 |
|------|-----|
| URL | `https://api.tavily.com` |
| 协议 | HTTP REST |
| 认证 | API Key (NewsApiKey) |
| 格式 | JSON |

**请求参数**: query, search_depth, max_results, include_answer, include_raw_content, include_images, topic, include_domains, exclude_domains
**响应结构**: `results` (title, url, content, raw_content, score, published_date) + `answer`

**文件**: `src/news/providers/tavily.rs`

#### 2.16.2 SerpAPI

| 属性 | 值 |
|------|-----|
| 协议 | HTTP REST |
| 认证 | API Key |
| 文件 | `src/news/providers/serpapi.rs` |

#### 2.16.3 Bocha (博查)

| 属性 | 值 |
|------|-----|
| 协议 | HTTP REST |
| 认证 | API Key |
| 文件 | `src/news/providers/bocha.rs` |

#### 公共接口

| trait/struct | 文件 |
|-------------|------|
| `NewsProvider` trait | `src/news/provider.rs` |
| `NewsSearchRequest` | `src/news/types.rs` |
| `NewsSearchResult` | `src/news/types.rs` |
| `NewsArticle` | `src/news/types.rs` |
| `NewsProviderConfig` | `src/news/types.rs` |

---

### 2.17 EastMoney 基本面

| 功能 | Rust 组件 | 文件 |
|------|----------|------|
| 估值指标 (PE/PB/PS) | `ValuationFetcher` | `src/fundamental/valuation.rs` |
| 财报 (营收/利润) | `EarningsFetcher` | `src/fundamental/earnings.rs` |
| 机构持仓 | `InstitutionFetcher` | `src/fundamental/institution.rs` |
| 龙虎榜 | `DragonTigerFetcher` | `src/fundamental/dragon_tiger.rs` |

| Rust 结构体 | 说明 |
|------------|------|
| `EastMoneyFundamentalProvider` | 统一入口，实现 `FundamentalProvider` trait |
| `ValuationMetrics` | 估值指标（PE, PB, PS, 股息率等） |
| `EarningsReport` | 财报数据 |
| `InstitutionHolding` | 机构持仓 |
| `DragonTigerItem` | 龙虎榜条目 |
| `CapitalFlow` | 资金流向 |
| `DividendInfo` | 分红信息 |

**协议**: HTTP REST（reqwest），从东方财富公开 API 抓取
**文件**: `src/fundamental/eastmoney.rs`, `src/fundamental/provider.rs`

---

### 2.18 通知推送 (Notification)

#### 发送器

| 发送器 | 文件 |
|--------|------|
| `FeishuSender` | `src/monitoring/notification/senders/feishu.rs` |
| `WechatWorkSender` | `src/monitoring/notification/senders/wechat_work.rs` |
| `DesktopSender` | `src/monitoring/notification/senders/desktop.rs` |
| `WebhookSender` | `src/monitoring/notification/senders/webhook.rs` |
| `LogSender` | `src/monitoring/notification/senders/log.rs` |

**协议**: 各系统对应的 HTTP Webhook / 本地通知
**文件**: `src/monitoring/notification/service.rs`, `src/monitoring/notification/senders/mod.rs`

---

## 第三层：共享合约与原语

### 3.1 Bridge 合约版本 (Contract Version)

| 字段 | 值 |
|------|-----|
| 默认版本 | `miniqmt.v1` |
| 传输 | `X-Bridge-Contract-Version` HTTP Header |
| 校验 | 空字符串过滤 → 回退默认 |
| 协商 | Bridge 服务端校验，不匹配返回 `LiveBridgeUnsupportedContractVersion` |

### 3.2 Bridge 错误码

| 枚举变体 | HTTP 触发条件 | 说明 |
|---------|--------------|------|
| `LiveBridgeTimeout` | `reqwest::Error::is_timeout()` | 请求超时 |
| `LiveBridgeUnavailable` | `is_connect()` / 5xx | 服务不可达 |
| `LiveBridgeAuthFailed` | 401/403 | 鉴权失败 |
| `LiveBridgeUnsupportedContractVersion` | bridge 返回 | 合约版本不支持 |
| `LiveBridgeUnsupportedMethod` | bridge 返回 | 方法不支持 |
| `LiveBridgeInvalidResult` | bridge 返回 | 结果无效 |
| `LiveBridgeIdentityMismatch` | bridge 返回 | 身份不匹配 |

### 3.3 Broker 事件类型

| 枚举变体 | 含义 |
|---------|------|
| `Acknowledgement` | 受理确认 → `OrderStatus::Accepted` |
| `Reject` | 拒单 → `OrderStatus::Rejected` |
| `Execution` | 成交回报 → `OrderStatus::Filled` |

### 3.4 执行订单状态映射

| Bridge 事件 | 内部 OrderStatus |
|-------------|-----------------|
| `Acknowledgement` | `Accepted` |
| `Reject` | `Rejected` |
| `Execution` | `Filled` |

### 3.5 接入模式 (Execution Mode)

| 模式 | 适配器 | Broker |
|------|--------|--------|
| `paper` | `PaperExecutionAdapter` | 本地内存模拟 |
| `mock_live` | `MockLiveExecutionAdapter` | 桥接模拟 |
| `qmt_live` | `QmtLiveExecutionAdapter` | 真实 QMT |

---

## 附录 A：环境变量索引

| 环境变量 | 所属系统 | 默认值 | 章节 |
|----------|---------|--------|------|
| `QUANTIX_CLICKHOUSE_URL` | ClickHouse | `http://localhost:8123` | 2.1 |
| `QUANTIX_CLICKHOUSE_DB` | ClickHouse | `quantix` | 2.1 |
| `QUANTIX_CLICKHOUSE_USER` | ClickHouse | `default` | 2.1 |
| `QUANTIX_CLICKHOUSE_PASSWORD` | ClickHouse | `""` | 2.1 |
| `QUANTIX_POSTGRES_URL` | PostgreSQL | - | 2.2 |
| `QUANTIX_TDENGINE_URL` | TDengine | - | 2.3 |
| `QUANTIX_TDENGINE_TOKEN` | TDengine | - | 2.3 |
| `QUANTIX_BRIDGE_URL` | Bridge | `http://127.0.0.1:17580` | 2.12 |
| `QUANTIX_BRIDGE_API_KEY` | Bridge | None | 2.12 |
| `QUANTIX_BRIDGE_BEARER_TOKEN` | Bridge | None | 2.12 |
| `QUANTIX_BRIDGE_CONTRACT_VERSION` | Bridge | `miniqmt.v1` | 2.12 |
| `QUANTIX_BRIDGE_TIMEOUT_MS` | Bridge | 30000 | 2.12 |
| `QUANTIX_BRIDGE_POLL_INTERVAL_MS` | Bridge | 1000 | 2.12 |
| `QUANTIX_BRIDGE_POLL_TIMEOUT_MS` | Bridge | 30000 | 2.12 |

---

## 附录 B：Feature Flag 映射

| Cargo Feature | 激活的系统 | 默认 |
|--------------|-----------|------|
| `postgresql` | PostgreSQL (sqlx/postgres) | 是 |
| `sqlite` | SQLite (sqlx/sqlite) | 否 |
| `tdengine-rest` | TDengine REST API | 是 |
| `tdengine-ws` | TDengine WebSocket | 否 |
| `tui` | 终端 UI 模式（ratatui/crossterm） | 否 |

---

## 附录 C：字段级 Spot-Check 参考（L2 Hard Sync）

> **本节由 `tests/contract_doc_field_sync_test.rs` 严格核对**：每条 `<!-- L2:TAG -->` 标记的代码块字段集合必须与源码无序相等。修改任一侧必须同步另一侧。

### C.1 ClickHouse 表列名

<!-- L2:CLICKHOUSE_TABLE name=kline_data -->
```
timestamp code name period open high low close volume amount trade_count source date
Time String String String Float Float Float Float Float Float Int String Date
```
<!-- /L2 -->

<!-- L2:CLICKHOUSE_TABLE name=minute_klines -->
```
timestamp code period adjust open high low close volume amount date
Time String String String Float Float Float Float Float Float Date
```
<!-- /L2 -->

<!-- L2:CLICKHOUSE_TABLE name=minute_shares -->
```
timestamp code price volume amount avg_price date
Time String Float Float Float Float Date
```
<!-- /L2 -->

<!-- L2:CLICKHOUSE_TABLE name=import_state -->
```
code trade_date kind status reason batch_id imported_at
String Date String String String String Time
```
<!-- /L2 -->

### C.2 Bridge 请求/响应结构体字段

<!-- L2:BRIDGE_STRUCT name=BridgeQmtOrderRequest -->
```
request_id client_order_id symbol side quantity price order_type strategy_name order_remark snapshot_metadata
String String String String Int String String String String Custom
```
<!-- /L2 -->

<!-- L2:BRIDGE_STRUCT name=BridgeTaskExecuteRequest -->
```
provider method params
String String Custom
```
<!-- /L2 -->

<!-- L2:BRIDGE_STRUCT name=BridgeKlineBarPayload -->
```
datetime open high low close volume turnover
String Float Float Float Float Int Float
```
<!-- /L2 -->

### C.3 OpenStock 数据结构字段

<!-- L2:OPENSTOCK_STRUCT name=OpenStockEnvelope -->
```
data source data_category request_id route_decision_id quality_flags cache_state circuit_state latency_ms received_at
```
<!-- /L2 -->

<!-- L2:OPENSTOCK_STRUCT name=Kline -->
```
code date open high low close volume amount adjust_type
```
<!-- /L2 -->

---

## 附录 D：PR Checklist

修改外部系统对接代码时按下列清单自检。每行右侧命名对应的同步测试；任一未通过即 PR 阻塞合并。

| 修改类型 | 操作 | 对应同步测试 |
|---------|------|------------|
| 新增/删除外部系统 | §2 系统全景图 + §x 详细章节 + 附录 A env | `l1_full_inventory_all_paths_exist` / `l1_doc_file_exists` |
| 新增/删除端点（OpenStock data_category） | §2.5 数据类别表 + fixture（如适用） | `l2_fixture_inventory_matches_filesystem` |
| 新增/删除 ClickHouse 表/列 | §5.2 表清单 + 附录 C.1 字段表 | `l2_*_columns_match` / `l2b_*_type_classes_match` |
| 新增/删除 Bridge 请求/响应字段 | §3.x 字段说明 + 附录 C.2 | `l2_bridge_*_fields_match` / `l2b_bridge_*_type_classes_match` |
| 新增/删除 OpenStock 结构字段 | §4.4 数据模型 + 附录 C.3 | `l2_openstock_*_fields_match` |
| 新增 fixture | tests/fixtures/openstock/ + §2.5 夹具清单 | `l2_fixture_inventory_matches_filesystem` / `l2_fixture_envelope_shapes_parse_and_category_is_valid` |
| 改字段类型大类（Decimal→Float64 等） | bump Contract Version Major + 附录 C 类型行 | `l2b_*_type_classes_match` |
| 改字段语义/单位（手↔股、秒↔毫秒） | bump Contract Version Major | 人工评审 |
| 改鉴权 / env 变量 | 附录 A 环境变量索引 | `l1_full_inventory_all_paths_exist` |
| 废弃字段/端点 | 加 `@deprecated vX.Y` 标记 + 保留 1 个 Minor 周期（≥4 周） | 人工评审 |

**强制门禁**：PR 合并前 `cargo test --test contract_doc_sync_test --test contract_doc_field_sync_test --test contract_doc_fixture_sync_test` 必须 0 失败。

**Contract Version 协商**（Bridge 侧）：客户端请求带 `contract_version: miniqmt.v1` 头；服务端返回 `BridgeFailureCode::LiveBridgeUnsupportedContractVersion` 时客户端降级到只读 capabilities 端点，停止下单。

---

> **维护者**: 本文档应与 `src/` 下各模块保持同步。LLM-assisted 更新后应运行 `tests/contract_doc_sync_test.rs`、`tests/contract_doc_field_sync_test.rs`、`tests/contract_doc_fixture_sync_test.rs` 验证一致性。硬失败条件：文档列举的系统在源码中不存在、或实际存在的外部系统在文档中未记录、或附录 C 的字段集合与源码定义不一致、或字段类型大类与源码不一致、或 fixture 清单与文件系统不一致。
