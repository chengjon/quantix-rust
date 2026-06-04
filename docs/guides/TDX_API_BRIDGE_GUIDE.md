# TDX-API Bridge 使用指南

> quantix-rust 通过 HTTP 桥接 tdx-api Docker 服务，获取通达信行情数据。

## 架构概览

```
┌─────────────────────┐         HTTP (REST)        ┌──────────────────────┐
│   quantix-rust      │  ◄──────────────────────►  │   tdx-api (Docker)   │
│   (Rust CLI)        │         :8080               │   (Go Service)       │
│                     │                             │                      │
│  sources/tdx_api.rs │                             │  web/server.go       │
│  TdxApiClient       │                             │  30 API endpoints    │
│  Fetcher trait      │                             │                      │
└─────────────────────┘                             └──────────┬───────────┘
                                                               │ TCP :7709
                                                               ▼
                                                    ┌──────────────────────┐
                                                    │  通达信公共服务器       │
                                                    │  (30+ IP, 沪/深/京)   │
                                                    └──────────────────────┘
```

**职责分离：**
- **tdx-api** — 数据获取层：通达信二进制协议、连接池、同花顺抓取、异步任务
- **quantix-rust** — 分析交易层：因子研究、策略执行、风控、回测

## 快速开始

### 1. 启动 tdx-api Docker 服务

```bash
cd /opt/claude/tdx-api
docker-compose up -d
# 验证
curl http://localhost:8080/api/health
```

### 2. 配置 quantix-rust

```bash
# 环境变量方式（推荐）
export TDX_API_URL=http://tdx-api:8080
export TDX_API_TIMEOUT_SECS=30

# 或在 config/data_sources.toml 中配置
[data_sources.tdx_api]
base_url = "http://tdx-api:8080"
timeout_secs = 30
max_retries = 3
```

### 3. 验证连接

```bash
quantix data tdx-api health
quantix data source test --name tdx-api
```

## CLI 命令参考

所有 tdx-api 命令通过 `quantix data tdx-api <子命令>` 调用。

### 健康检查

```bash
quantix data tdx-api health
```

### 实时行情

```bash
# 单只股票
quantix data tdx-api quote -c 600000
quantix data tdx-api quote -c sh600000

# 支持的代码格式: 600000 / sh600000 / 000001 / sz000001 / 430047 / bj430047
```

### K 线数据

```bash
# 日K（默认最近100条）
quantix data tdx-api kline -c 600000

# 指定周期和数量
quantix data tdx-api kline -c 600000 -t minute5 --limit 50

# 支持的周期: minute1, minute5, minute15, minute30, hour, day, week, month
```

### 同花顺前复权 K 线

```bash
# 完整历史日K（THS源，前复权）
quantix data tdx-api kline-ths -c 600000

# 周/月K
quantix data tdx-api kline-ths -c 600000 -t week
quantix data tdx-api kline-ths -c 600000 -t month
```

### 分时数据

```bash
# 今日分时
quantix data tdx-api minute -c 600000

# 指定日期
quantix data tdx-api minute -c 600000 --date 20250605
```

### 搜索股票

```bash
quantix data tdx-api search -k 平安
quantix data tdx-api search -k 茅台
```

### 交易日查询

```bash
# 查询单个日期及其前后交易日
quantix data tdx-api workday -d 20250605 --count 5

# 查询日期范围
quantix data tdx-api workday-range --start 20250601 --end 20250630
```

### N 日收益计算

```bash
# 默认 5,10,20,60,120 日
quantix data tdx-api income -c 600000 --start-date 20250101

# 自定义天数
quantix data tdx-api income -c 600000 --start-date 20250101 -d 5,10,30
```

### 市场统计

```bash
quantix data tdx-api market-stats
```

### 异步任务管理

```bash
# 列出任务
quantix data tdx-api tasks

# 查看任务详情
quantix data tdx-api task-info --id <task-uuid>
```

## Rust API 参考

### 创建客户端

```rust
use crate::sources::tdx_api::{TdxApiClient, TdxApiConfig};

// 从环境变量创建
let client = TdxApiClient::from_env()?;

// 自定义配置
let client = TdxApiClient::new(TdxApiConfig {
    base_url: "http://tdx-api:8080".to_string(),
    timeout: std::time::Duration::from_secs(30),
    max_retries: 3,
})?;

// 从应用配置文件创建
let client = TdxApiClient::from_app_config(&config.data_sources.tdx_api.unwrap())?;
```

### Fetcher trait

`TdxApiClient` 实现了 `Fetcher` trait，可直接替代现有数据源：

```rust
use crate::data::fetcher::Fetcher;

// 统一接口：获取股票信息
let info = client.get_stock_info("600000").await?;
// → Option<StockInfo { code, name, market, .. }>

// 统一接口：获取日K线
let klines = client.get_kline("600000",
    NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
    NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()
).await?;
// → Vec<Kline> (价格使用 Decimal 类型)

// 统一接口：健康检查
client.check_connection().await?;
```

### 行情数据

```rust
// 单只行情
let quote: StockQuote = client.get_quote("600000").await?;

// 批量行情（最多50只）
let quotes: Vec<StockQuote> = client.batch_quote(&["600000", "000001"]).await?;
```

### K 线数据

```rust
use crate::sources::tdx_api::KlineType;

// 原始K线（价格单位: 厘，需手动转换）
let resp = client.get_kline_raw("600000", KlineType::Day, Some(100)).await?;
// resp.count, resp.list: Vec<KlineItem>

// 标准K线模型（自动转换为 Decimal 价格）
let klines = client.get_daily_kline("600000", start, end).await?;
// → Vec<Kline>

// TDX源完整历史
let all_klines = client.get_kline_all_tdx("600000", KlineType::Day, None).await?;

// 同花顺前复权完整历史
let ths_klines = client.get_kline_all_ths("600000", KlineType::Day).await?;

// 指数K线
let index = client.get_index_kline("000001", KlineType::Day, Some(100)).await?;
```

### 分时 / 成交

```rust
// 分时数据
let minute = client.get_minute("600000", None).await?;
// minute.date, minute.list: Vec<MinuteItem>

// 逐笔成交
let trades = client.get_trades("600000", Some("20250605")).await?;
```

### 代码查询

```rust
// 搜索
let results = client.search_codes("平安").await?;
// Vec<SearchResult { code, name, exchange }>

// 全部代码列表（带1小时缓存）
let codes = client.get_codes(None).await?;
// CodesResponse { total, codes: Vec<CodeEntry> }

// 按交易所筛选
let sh_codes = client.get_codes(Some("sh")).await?;
```

### 交易日历

```rust
// 判断交易日
let is_trading = client.is_trading_day(NaiveDate::from_ymd_opt(2025, 6, 5).unwrap()).await?;

// 交易日范围（带缓存）
let dates = client.get_workday_range("20250601", "20250630").await?;
// → Vec<NaiveDate>
```

### 收益计算

```rust
let income = client.get_income("600000", "20250101", &[5, 10, 20, 60]).await?;
// IncomeResponse { list: Vec<IncomeItem { offset, rise, rise_rate, source, current }> }
```

### 异步任务

```rust
// 创建K线拉取任务
let task_id = client.create_pull_kline_task(&PullKlineRequest {
    codes: vec!["sh600000".to_string()],
    tables: vec!["day".to_string()],
    start_date: "2025-01-01".to_string(),
    ..Default::default()
}).await?;

// 创建成交拉取任务
let task_id = client.create_pull_trade_task(&PullTradeRequest {
    code: "sh600000".to_string(),
    start_year: Some(2024),
    ..Default::default()
}).await?;

// 查询任务
let tasks = client.list_tasks().await?;
let task = client.get_task(&task_id).await?;
```

### 缓存管理

```rust
// 清除本地缓存（代码列表、交易日）
client.invalidate_cache();
```

## 数据类型对照

### 价格单位转换

tdx-api 使用 **厘**（1/1000 元）存储价格，quantix-rust 使用 `Decimal`：

```
tdx-api raw:  1248000 (厘)
              ↓ ÷ 1000
quantix-rust: 1248.000 (元, Decimal)
```

### KlineType 映射

| KlineType 变体 | tdx-api 参数 |
|---------------|-------------|
| `Min1` | `minute1` |
| `Min5` | `minute5` |
| `Min15` | `minute15` |
| `Min30` | `minute30` |
| `Hour` | `hour` |
| `Day` | `day` |
| `Week` | `week` |
| `Month` | `month` |

### Symbol 格式转换

| 输入 | 转换为 tdx-api |
|------|---------------|
| `600000` | `sh600000` |
| `000001` | `sz000001` |
| `430047` | `bj430047` |
| `510050` | `sh510050` |
| `sh600000` | `sh600000`（原样） |

### Market 映射

| tdx-api exchange | quantix-rust Market |
|------------------|-------------------|
| `sh` | `Market::SH` |
| `sz` | `Market::SZ` |
| `bj` | `Market::BJ` |

## 内置特性

### 指数退避重试

```
请求失败 → 等待 500ms → 重试1
         → 等待 1000ms → 重试2
         → 等待 2000ms → 重试3
         → 返回错误
```

- 5xx 错误自动重试
- 4xx 错误立即返回（不重试）
- 业务错误（`code != 0`）立即返回

### 本地缓存

| 数据 | TTL | 用途 |
|------|-----|------|
| 代码列表 (`/api/codes`) | 1 小时 | `get_stock_info()` 依赖此缓存 |
| 交易日范围 (`/api/workday/range`) | 1 小时 | `is_trading_day()` 依赖此缓存 |

手动清除：`client.invalidate_cache()`

### 连接池

reqwest 内置连接池，默认配置：
- `pool_max_idle_per_host`: 4
- `connect_timeout`: 5s
- `timeout`: 可配置（默认 30s）

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
| `create_pull_kline_task()` | `POST /api/tasks/pull-kline` | - |
| `create_pull_trade_task()` | `POST /api/tasks/pull-trade` | - |
| `list_tasks()` | `GET /api/tasks` | - |
| `get_task()` | `GET /api/tasks/{id}` | - |
| `cancel_task()` | `GET /api/tasks/{id}/cancel` | - |

## Docker Compose 集成示例

```yaml
# docker-compose.yml
services:
  tdx-api:
    build: /opt/claude/tdx-api
    ports:
      - "8080:8080"
    restart: unless-stopped

  quantix:
    build: /opt/claude/quantix-rust
    environment:
      - TDX_API_URL=http://tdx-api:8080
    depends_on:
      - tdx-api
```

## 故障排除

| 问题 | 原因 | 解决 |
|------|------|------|
| `tdx-api 连接失败` | Docker 未启动 | `docker-compose up -d` |
| `tdx-api 业务错误 [-1]` | 通达信服务器连接断开 | 重启 tdx-api 容器 |
| `tdx-api 重试耗尽` | 网络不通或服务不可用 | 检查 `TDX_API_URL` 和网络 |
| `数据源错误: tdx-api 行情无数据` | 股票代码格式错误 | 使用 `search` 命令确认代码 |
| 缓存数据过期 | 超过 1 小时 | 自动刷新或手动 `invalidate_cache()` |
