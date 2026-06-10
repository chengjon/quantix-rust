# quantix-rust 连接 tdx-api REST 方式概览

## 架构

quantix-rust 通过 **HTTP REST** 调用 tdx-api Docker 服务，无需在 Rust 端实现通达信二进制协议：

```
quantix-rust  ──── HTTP REST (:8080) ────  tdx-api (Docker/Go)  ──── TCP :7709 ──── 通达信服务器
```

核心客户端是 `src/sources/tdx_api.rs` 中的 `TdxApiClient`，基于 `reqwest` 实现，内置重试（指数退避）和本地缓存。

## 配置方式（两种）

### 1. 环境变量（优先级最高）

| 环境变量 | 默认值 | 说明 |
|---------|--------|------|
| `TDX_API_URL` | `http://tdx-api:8080` | 服务地址 |
| `TDX_API_TIMEOUT_SECS` | `30` | 请求超时（秒） |
| `TDX_API_ENABLED` | `true` | 是否启用 |
| `TDX_API_MAX_BATCH_QUOTE_SIZE` | `50` | 批量行情最大数量 |
| `TDX_API_HEALTH_TIMEOUT_SECS` | `5` | 健康检查超时（秒） |

### 2. 应用配置文件（`config/data_sources.toml`）

```toml
[data_sources.tdx_api]
base_url = "http://tdx-api:8080"
timeout_secs = 30
max_retries = 3
enabled = true
max_batch_quote_size = 50
health_timeout_secs = 5
```

优先级：**环境变量 > 配置文件字段默认值 > 代码内置默认值**。

## 关键运行时参数

| 参数 | 值 | 说明 |
|------|-----|------|
| 连接池 | `pool_max_idle_per_host: 4` | reqwest 内置 |
| 连接超时 | `5s` | 固定 |
| 重试次数 | `3` | 仅 5xx 重试，4xx 立即返回 |
| 重试间隔 | 500ms → 1s → 2s | 指数退避 |
| 缓存 TTL | `3600s (1h)` | 代码列表、交易日范围 |
| 批量行情上限 | `50` 只 | 可配置 |

## 创建客户端的三种方式

```rust
// 1. 从环境变量
let client = TdxApiClient::from_env()?;

// 2. 从应用配置文件
let client = TdxApiClient::from_app_config(&config.data_sources.tdx_api.unwrap())?;

// 3. 手动构造
let client = TdxApiClient::new(TdxApiConfig { ... })?;
```

## 验证连接

```bash
quantix data tdx-api health              # 健康检查
quantix data source test --name tdx-api  # 完整连接测试（含上游连通性）
```

## API 端点映射

| quantix-rust 方法 | tdx-api 端点 | 缓存 |
|-------------------|-------------|------|
| `get_quote()` | `GET /api/quote` | - |
| `batch_quote()` | `POST /api/batch-quote` | - |
| `get_kline_raw()` | `GET /api/kline` | - |
| `get_daily_kline()` | `GET /api/kline` | - |
| `get_kline_ths_qfq()` | `GET /api/kline-all/ths` | - |
| `get_kline_all_tdx()` | `GET /api/kline-all/tdx` | - |
| `get_kline_history()` | `GET /api/kline-history` | - |
| `get_minute()` | `GET /api/minute` | - |
| `get_trades()` | `GET /api/trade` | - |
| `search_codes()` | `GET /api/search` | - |
| `get_codes()` | `GET /api/codes` | 1h |
| `get_workday()` | `GET /api/workday` | - |
| `get_workday_range()` | `GET /api/workday/range` | 1h |
| `is_trading_day()` | `GET /api/workday` | - |
| `get_income()` | `GET /api/income` | - |
| `get_market_stats()` | `GET /api/market-stats` | - |
| `get_market_count()` | `GET /api/market-count` | - |
| `get_index_kline()` | `GET /api/index` | - |
| `health()` | `GET /api/health` | - |
| `server_status()` | `GET /api/server-status` | - |
| `create_pull_kline_task()` | `POST /api/tasks/pull-kline` | - |
| `create_pull_trade_task()` | `POST /api/tasks/pull-trade` | - |
| `list_tasks()` | `GET /api/tasks` | - |
| `get_task()` | `GET /api/tasks/{id}` | - |
| `cancel_task()` | `GET /api/tasks/{id}/cancel` | - |

## 数据类型转换

### 价格单位

tdx-api 使用 **厘**（1/1000 元）存储价格，quantix-rust 使用 `Decimal`：

```
tdx-api raw:  1248000 (厘)
              ↓ ÷ 1000
quantix-rust: 1248.000 (元, Decimal)
```

### 股票代码映射

| 输入 | 转换为 tdx-api |
|------|---------------|
| `600000` | `sh600000` |
| `000001` | `sz000001` |
| `430047` | `bj430047` |
| `510050` | `sh510050` |
| `sh600000` | `sh600000`（原样） |

### Fetcher trait

`TdxApiClient` 实现了 `Fetcher` trait，可作为统一数据源接口的 drop-in 替代：

```rust
use crate::data::fetcher::Fetcher;

let info = client.get_stock_info("600000").await?;
let klines = client.get_kline("600000", start, end).await?;
client.check_connection().await?;
```

## 关键源文件

| 文件 | 职责 |
|------|------|
| `src/sources/tdx_api.rs` | 客户端实现、API 类型、缓存、重试 |
| `src/core/config.rs` | `TdxApiConfig` 结构体、默认值、环境变量映射 |
| `src/cli/handlers/tdx_api_handler.rs` | CLI 子命令处理 |
| `src/tasks/collect_scheduler.rs` | 批量行情采集调度 |
