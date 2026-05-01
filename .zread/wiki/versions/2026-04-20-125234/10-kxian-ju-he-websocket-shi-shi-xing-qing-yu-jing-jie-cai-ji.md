本模块是 Quantix 实时行情数据管线的核心层，由四个互补的采集组件构成：**K线实时聚合器**将逐笔行情压缩为多周期 K 线，**WebSocket 客户端**维持长连接推送行情，**竞价采集器**捕获集合竞价时段（9:15-9:25）的特殊数据，**行情采集器**则负责批量拉取全市场行情。四者均以 `StockQuote` / `RealtimeQuote` 为统一数据载体，通过 `mpsc` 通道与下游分析管线、ClickHouse 持久化层无缝衔接。

Sources: [mod.rs](src/sources/mod.rs#L1-L30)

## 整体架构

在深入每个组件之前，先理解它们之间的数据流关系。整个行情采集管线遵循 **采集 → 聚合 → 分发 → 持久化** 的四级管道模式：

```
┌─────────────────────────────────────────────────────────────────────┐
│                        行情数据采集与聚合管线                          │
│                                                                     │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────────────┐ │
│  │  TDX TCP     │   │  WebSocket   │   │  竞价采集器              │ │
│  │  (QuoteColl.) │   │  Client      │   │  (AuctionCollector)      │ │
│  │              │   │              │   │                          │ │
│  │  全市场批量   │   │  订阅式推送   │   │  9:15-9:25 时段         │ │
│  │  定时拉取     │   │  实时接收     │   │  TDX TCP 逐只采集       │ │
│  └──────┬───────┘   └──────┬───────┘   └──────────┬───────────────┘ │
│         │                  │                      │                 │
│         ▼                  ▼                      ▼                 │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    KlineAggregator (K线聚合)                  │   │
│  │                                                              │   │
│  │   StockQuote ──► KlineWindow ──► KlineData (1m/5m/30m/...)  │   │
│  │                                                              │   │
│  │   ┌─────────────────────────────────────────────┐            │   │
│  │   │  windows: HashMap<"code:period:date", Window>│            │   │
│  │   └─────────────────────────────────────────────┘            │   │
│  └──────────────────────┬───────────────────────────────────────┘   │
│                         │                                           │
│                         ▼                                           │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  mpsc::UnboundedReceiver<KlineData>                          │   │
│  │     ├──► ClickHouse 持久化 (via ETL)                         │   │
│  │     ├──► 技术指标管线 (Indicators)                            │   │
│  │     └──► 策略引擎 (Strategy)                                 │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  CollectScheduler (智能调度)                                  │   │
│  │  Active → 60s | Auction → 30s | Inactive → 300s             │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

上图中，**TDX TCP 连接池**是共享底层——`QuoteCollector` 和 `AuctionCollector` 都依赖 `rustdx_complete` 库的 TCP 协议连接通达信服务器。`CollectScheduler` 则根据交易时段动态调度 `QuoteCollector` 的采集频率。

Sources: [collect_scheduler.rs](src/tasks/collect_scheduler.rs#L1-L12), [kline_aggregator.rs](src/sources/kline_aggregator.rs#L1-L10), [websocket.rs](src/sources/websocket.rs#L1-L14)

## 核心数据模型

在理解每个组件的实现细节之前，需要先掌握贯穿整个管线的两个关键数据结构：`StockQuote`（TDX 行情快照）和 `RealtimeQuote`（WebSocket 推送行情），以及它们的聚合产物 `KlineData`。

### 数据模型对照表

| 字段 | `StockQuote` (TDX) | `RealtimeQuote` (WebSocket) | `KlineData` (聚合产物) | 说明 |
|------|-------------------|---------------------------|---------------------|------|
| 时间戳 | `timestamp: u64` (Unix 秒) | `timestamp: i64` | `timestamp: DateTime<Utc>` | 聚合后统一为 chrono 类型 |
| 代码/名称 | `code` / `name` | `code` / `name` | `code` / `name` | 三者一致 |
| OHLCV | `price` / `open` / `high` / `low` / `volume` / `amount` | `price` / `open` / `high` / `low` / `volume` / `amount` | `open` / `high` / `low` / `close` / `volume` / `amount` | 注意 `StockQuote.price` 对应当前价，而 K 线中有独立的 `close` |
| 昨收 | `preclose` | `preclose` | — | 仅实时行情需要 |
| 涨跌幅 | `change_percent` | `change_percent` | — | 由 TDX/服务端计算 |
| 买卖盘 | — | `bid1` / `ask1` (Option) | — | WebSocket 提供盘口数据 |
| 周期标识 | — | — | `period: KlinePeriod` | 聚合后新增 |
| 成交笔数 | — | — | `trade_count: u32` | 窗口内累加 |
| 数据源 | — | — | `source: String` | 标记 `"realtime"` 等 |

**设计要点**：`StockQuote` 是 TDX TCP 协议的原始快照，包含当前价 `price` 和昨收价 `preclose`；`RealtimeQuote` 增加了可选的买卖一档盘口（`bid1`/`ask1` 为 `Option<f64>`，兼容无盘口数据的场景）；`KlineData` 则是时间窗口聚合后的完整 OHLCV 记录，附带 `trade_count` 标记窗口内的成交笔数。

Sources: [tdx.rs](src/sources/tdx.rs#L22-L48), [websocket.rs](src/sources/websocket.rs#L27-L54), [kline_aggregator.rs](src/sources/kline_aggregator.rs#L68-L95)

## K线实时聚合器（KlineAggregator）

### 设计原理

**K线聚合**的本质是将离散的逐笔行情快照按时间窗口归并为标准 OHLCV 柱状数据。`KlineAggregator` 在内存中维护一个 `HashMap<String, KlineWindow>`，以 `"code:period:date"` 为键（例如 `"000001:5m:2026-01-02"`），每个窗口记录该周期内的开高低收、累计成交量和成交笔数。

Sources: [kline_aggregator.rs](src/sources/kline_aggregator.rs#L202-L208)

### 支持的 K 线周期

`KlinePeriod` 枚举定义了六种标准周期：

| 周期 | 标识符 | 分钟数 | 窗口对齐策略 |
|------|--------|--------|------------|
| 1 分钟 | `"1m"` | 1 | 截断到整分钟（秒/纳秒置零） |
| 5 分钟 | `"5m"` | 5 | 对齐到 5 的整数倍分钟 |
| 15 分钟 | `"15m"` | 15 | 对齐到 15 的整数倍分钟 |
| 30 分钟 | `"30m"` | 30 | 对齐到 30 的整数倍分钟 |
| 60 分钟 | `"60m"` | 60 | 对齐到整小时 |
| 日线 | `"1d"` | 240 | 对齐到当日 09:30 开盘时间 |

**窗口对齐**的实现在 `calculate_window_start` 方法中——以 5 分钟窗口为例，如果当前时间是 `10:33:45`，窗口起始会被对齐到 `10:30:00`。这保证了同一个 5 分钟周期内的所有行情快照落入同一个窗口。

Sources: [kline_aggregator.rs](src/sources/kline_aggregator.rs#L13-L66), [kline_aggregator.rs](src/sources/kline_aggregator.rs#L238-L287)

### 窗口生命周期

每条 `StockQuote` 进入聚合器后，`process_quote` 方法会同时更新该股票的 **1 分钟、5 分钟和 30 分钟**三个窗口（这是当前硬编码的三种实时聚合周期）。单窗口的更新流程如下：

```
StockQuote 到达
    │
    ▼
┌─ make_window_key ─────────────────────────────┐
│  key = "000001:5m:2026-01-02"                 │
└───────────────────────┬────────────────────────┘
                        │
              ┌─────────▼──────────┐
              │ 窗口是否已存在？    │
              └──┬──────────────┬──┘
                 │ Yes          │ No
                 ▼              ▼
          更新窗口数据     创建新窗口
          (OHLCV更新)     (首笔=开盘价)
                 │              │
                 ▼              │
         should_close()         │
          (时间窗口到期?)        │
           │Yes     │No         │
           ▼        ▼           ▼
     移除窗口      保留窗口   存入 HashMap
     输出KlineData  (无输出)    (无输出)
```

**关键细节**：

- **开盘价锁定**：窗口的 `open` 字段为 `Option<f64>`，仅第一笔行情写入后锁定不变；`high`/`low` 通过 `max`/`min` 持续更新；`close` 始终为最新价。
- **窗口关闭判断**：`should_close` 比较当前时间与 `start_time` 的差值是否超过周期长度。一旦到期，窗口从 HashMap 中移除并输出为 `KlineData`。
- **过期窗口清理**：后台 `tokio::spawn` 任务每 5 分钟扫描一次，清除超过 2 小时未更新的窗口（`elapsed < 7200`），防止非交易时段残留的窗口占用内存。

Sources: [kline_aggregator.rs](src/sources/kline_aggregator.rs#L126-L199), [kline_aggregator.rs](src/sources/kline_aggregator.rs#L289-L386)

### 输出与持久化

`KlineAggregator::new` 返回一个 `(Self, mpsc::UnboundedReceiver<KlineData>)` 元组——聚合器本身持有 sender，外部通过 receiver 消费完成的 K 线数据。在 ETL 模块中，`write_klines_to_clickhouse` 方法将 `KlineData` 转换为 ClickHouse 的 `KlineDataCH` 模型，按批次写入 `kline_data` 表。

Sources: [kline_aggregator.rs](src/sources/kline_aggregator.rs#L212-L229), [etl.rs](src/sync/etl.rs#L165-L216)

## WebSocket 实时行情客户端

### 连接状态机

`WebSocketClient` 实现了一个四状态有限状态机，管理从建立连接到断线重连的完整生命周期：

```
          connect()
 Disconnected ──────────► Connecting
     ▲                        │
     │                   ┌────┴────┐
     │                   │ 成功     │ 失败
     │                   ▼          ▼
     │               Connected   (重试)
     │                   │          │
     │         断线/关闭  │     Reconnecting
     │                   │          │
     └───────────────────┘     等待reconnect_interval
           disconnect()              │
                                     │ reconnect_count < max
                                     └──► Connecting (循环)
```

**状态转换规则**：

| 当前状态 | 触发条件 | 目标状态 |
|---------|---------|---------|
| `Disconnected` | 调用 `connect()` | `Connecting` |
| `Connecting` | TCP 握手成功 | `Connected` |
| `Connecting` | TCP 握手失败 | `Disconnected` → `Reconnecting` |
| `Connected` | 收到 `Close` 帧或 I/O 错误 | `Disconnected` |
| `Reconnecting` | 等待 `reconnect_interval` 秒 | `Connecting`（如果未超过 `max_reconnect`） |
| 任意状态 | 调用 `disconnect()` | `Disconnected`（停止循环） |

Sources: [websocket.rs](src/sources/websocket.rs#L17-L23), [websocket.rs](src/sources/websocket.rs#L177-L313)

### 配置参数

`WebSocketConfig` 提供以下可调参数：

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `url` | `String` | `wss://push2.eastmoney.com/api/qt/stock/klt` | 东方财富 WebSocket 端点 |
| `heartbeat_interval` | `u64` | 30 秒 | Ping 心跳发送间隔 |
| `reconnect_interval` | `u64` | 5 秒 | 断线后重连等待时间 |
| `max_reconnect` | `usize` | 10 | 最大重连次数（超过后停止） |
| `buffer_size` | `usize` | 1000 | 消息缓冲区大小 |

Sources: [websocket.rs](src/sources/websocket.rs#L68-L92)

### 订阅管理与消息处理

连接建立后，`WebSocketClient` 通过 `subscribe` / `unsubscribe` 管理订阅列表（`HashMap<String, Subscription>`），并向服务器发送 `{"cmd": "sub", "data": [...]}` 格式的 JSON 消息。消息处理循环使用 `tokio::select!` 同时监听三个事件源：

1. **WebSocket 消息接收**：`Text` 消息经过 `parse_message` 解析为 `RealtimeQuote`，通过 `mpsc::UnboundedSender` 发送给消费者；`Ping` 消息自动回复 `Pong`；`Close` 消息触发断线处理。
2. **心跳定时器**：独立 `tokio::spawn` 按配置间隔发送 Ping 帧，保持连接活跃。
3. **运行状态检查**：每 100ms 检查 `running` 标志，支持优雅退出。

**消息解析**采用宽松的 JSON 反序列化策略——使用 `serde_json::Value` 动态提取字段，缺失字段以默认值填充（`price` 默认 `0.0`，`bid1`/`ask1` 为 `None`），确保即使上游数据格式微变也不会 panic。

Sources: [websocket.rs](src/sources/websocket.rs#L139-L174), [websocket.rs](src/sources/websocket.rs#L229-L284), [websocket.rs](src/sources/websocket.rs#L349-L381)

## 集合竞价采集器（AuctionCollector）

### 竞价时段与采集逻辑

A股集合竞价发生在每个交易日的 **9:15 至 9:25**，这 10 分钟内投资者可以提交买卖委托但不能撤单（9:20 后）。`AuctionCollector` 专门在这个时段采集自选股的竞价数据，核心数据结构 `AuctionQuote` 在标准行情字段之外增加了两个关键维度：

| 专有字段 | 类型 | 说明 |
|---------|------|------|
| `sealed_amount_buy` | `f64` | 买封金额 = 买一价 × 买一量 |
| `sealed_amount_sell` | `f64` | 卖封金额 = 卖一价 × 卖一量 |
| `strength_score` | `f32` | 抢筹强度评分（0-100） |

**抢筹强度评分算法**采用三维加权模型：

$$
\text{score} = \text{price\_rise} \times 40 + \text{buy\_ratio} \times 30 + \text{volume\_ratio} \times 30
$$

- **涨幅权重 40%**：`price_rise = max(change_percent, 0)`，只奖励正涨幅
- **买盘占比权重 30%**：`buy_ratio = buy1_volume / (buy1_volume + sell1_volume)`
- **成交量权重 30%**：`volume_ratio = min(volume / 1,000,000, 1.0)`，以 100 万手封顶

最终分数被 `clamp(0.0, 100.0)` 约束到 0-100 区间。

Sources: [auction_collector.rs](src/sources/auction_collector.rs#L1-L64), [auction_collector.rs](src/sources/auction_collector.rs#L139-L189)

### 采集循环与时序控制

`AuctionCollector::run()` 实现了一个自包含的事件循环，核心逻辑是**时序门控**——每轮迭代首先调用 `is_auction_time()` 进行双重检查：

1. **交易日检查**：通过 `TradingCalendar::is_trading_day` 判断当前日期是否为交易日（排除周末和法定假日）
2. **时段检查**：本地时间的小时和分钟是否落在 `9:15-9:25` 区间内

只有两个条件同时满足，才执行 `collect_all()` 对自选股列表逐只采集。采集间隔为 1 秒，非竞价时段以 10 秒间隔空转等待。

**注意**：`AuctionCollector` 使用**同步**的 `rustdx_complete` TCP 协议（`SecurityQuotes::recv_parsed` 是阻塞调用），因此 `fetch_auction_quote` 标记为 `&mut self`，不可并发调用。

Sources: [auction_collector.rs](src/sources/auction_collector.rs#L192-L304)

### 竞价分析与推荐

采集到的 `AuctionQuote` 会送入 `analysis::auction::AuctionAnalyzer` 进行进一步分析。分析器通过三个阈值参数筛选推荐标的：

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `min_recommend_score` | 70.0 | 最低推荐评分 |
| `min_sealed_amount` | 500,000.0 (元) | 最低买封金额（50 万元） |
| `max_change_percent` | 8.0 (%) | 最大涨幅限制（过滤涨停附近） |

一个标的被标记为 `is_recommended = true` 需同时满足四个条件：评分 ≥ 70、买封 ≥ 50 万、涨幅 ≤ 8%、买封占比 > 60%。`AuctionAnalyzer` 还提供按板块统计（`analyze_by_sector`）和按强度/买封排名（`rank_by_strength` / `rank_by_sealed_amount`）功能，板块分类基于股票代码前缀的简化规则（`600`/`601`/`603` → 上海主板，`688` → 科创板，`000`/`001` → 深圳主板，`300` → 创业板）。

Sources: [auction.rs](src/analysis/auction.rs#L99-L277)

## 批量行情采集器（QuoteCollector）

### 分批采集策略

全市场 A 股约 5000 只，单次 TDX TCP 请求无法拉取全部行情。`QuoteCollector` 采用 **分批 + 超时** 的策略：将股票列表按 `batch_size`（默认 800 只/批）切片，每批独立调用 `TdxSource::fetch_quotes_batch`，并用 `tokio::time::timeout` 包装整个操作（默认 10 秒超时）。

**容错设计**：单批采集失败不会中断整体流程——`collect_all` 方法在 catch 到错误后记录日志并跳过该批次，继续处理后续批次。批次之间插入 100ms 延迟（`tokio::time::sleep`），避免请求频率过高触发 IP 封禁。

Sources: [quote_collector.rs](src/sources/quote_collector.rs#L18-L174)

### TDX 底层连接池

`TdxSource` 维护一个 `Vec<Arc<Mutex<Tcp>>>` 连接池（默认 3 个连接），通过 `AtomicUsize` 轮询选择连接。由于 `rustdx_complete` 的 `Tcp` 是**同步阻塞** API，`fetch_quotes_batch` 使用 `tokio::task::spawn_blocking` 将阻塞 I/O 移至专用的阻塞线程池，避免阻塞 tokio 异步运行时。

```
TdxSource::fetch_quotes_batch()
    │
    ▼
get_connection() ──► Arc<Mutex<Tcp>>  (轮询选择)
    │
    ▼
tokio::spawn_blocking(move || {
    tcp.lock()         // 获取互斥锁
    SecurityQuotes::new(codes)
    quotes.recv_parsed(&mut tcp)   // 阻塞调用
})
    │
    ▼
tokio::time::timeout(10s, handle)   // 超时保护
```

市场判断逻辑使用简单的代码前缀规则：`6` 开头为上海（`market = 1`），其余为深圳（`market = 0`）。

Sources: [tdx.rs](src/sources/tdx.rs#L92-L250), [quote_collector.rs](src/sources/quote_collector.rs#L54-L98)

## 智能采集调度器（CollectScheduler）

### 时段感知调度

`CollectScheduler` 将 `QuoteCollector` 的采集行为与 A 股交易时段深度绑定，通过 `TradingCalendar::get_current_status()` 获取当前时段信息（`TradingSession` 枚举），然后映射为调度器状态：

| 调度器状态 | 触发条件 | 采集间隔 | 行为 |
|-----------|---------|---------|------|
| `Active` | 上午/下午交易时段 | 60 秒 | 执行全市场采集 |
| `Active` | 竞价时段 (Auction) | 30 秒 | 执行全市场采集 |
| `PreMarket` | 交易日 9:00-9:30 | 300 秒 | 仅等待，不采集 |
| `PostMarket` | 交易日 15:00-15:30 | 300 秒 | 仅等待，不采集 |
| `Inactive` | 非交易日或非交易时段 | 300 秒 | 休眠 |

**特殊模式**：设置环境变量 `FORCE_MODE` 后，调度器忽略所有时段判断，始终以 60 秒间隔执行采集——这在开发测试和回测场景中非常有用。

Sources: [collect_scheduler.rs](src/tasks/collect_scheduler.rs#L14-L200), [collect_scheduler.rs](src/tasks/collect_scheduler.rs#L260-L318)

### 回调机制

调度器通过 `set_callback` 注册 `Arc<dyn Fn(Vec<StockQuote>)>` 类型的回调函数。每次 `collect_once` 完成后，采集到的行情数据会通过回调传递给下游消费者（如 `KlineAggregator` 的 `process_quote` 方法）。这种**观察者模式**设计使调度器与具体的数据消费者解耦。

Sources: [collect_scheduler.rs](src/tasks/collect_scheduler.rs#L71-L138), [collect_scheduler.rs](src/tasks/collect_scheduler.rs#L236-L258)

## 组件协作关系总览

下表总结了四个核心组件的职责边界和协作方式：

| 维度 | KlineAggregator | WebSocketClient | AuctionCollector | QuoteCollector |
|------|----------------|-----------------|------------------|----------------|
| **数据源** | `StockQuote` (被动接收) | 东方财富 WebSocket | TDX TCP (rustdx_complete) | TDX TCP (via TdxSource) |
| **采集模式** | 被动聚合 | 推送式订阅 | 定时轮询 (1s) | 批量拉取 (分批) |
| **活跃时段** | 交易时段 | 持续连接 | 9:15-9:25 | 由 CollectScheduler 控制 |
| **输出类型** | `KlineData` | `RealtimeQuote` | `AuctionQuote` | `StockQuote` |
| **下游消费者** | ETL → ClickHouse | 策略引擎 | AuctionAnalyzer | KlineAggregator / 回调 |
| **并发模型** | `Arc<Mutex<HashMap>>` | `Arc<RwLock<State>>` | `&mut self` (独占) | `Arc<TdxSource>` |
| **内存管理** | 5 分钟清理过期窗口 | 自动重连 | 无状态缓存 | 无状态 |

Sources: [kline_aggregator.rs](src/sources/kline_aggregator.rs#L1-L10), [websocket.rs](src/sources/websocket.rs#L1-L14), [auction_collector.rs](src/sources/auction_collector.rs#L1-L11), [quote_collector.rs](src/sources/quote_collector.rs#L1-L8)

## 延伸阅读

- 了解 TDX 连接池的底层实现和 `Fetcher` trait 抽象，参见 [多数据源适配器架构（TDX/AKShare/东方财富/Bridge）](8-duo-shu-ju-yuan-gua-pei-qi-jia-gou-tdx-akshare-dong-fang-cai-fu-bridge)
- 了解 K 线数据写入 ClickHouse 的 ETL 流程，参见 [多数据库集成（ClickHouse/PostgreSQL/TDengine）](9-duo-shu-ku-ji-cheng-clickhouse-postgresql-tdengine)
- 了解交易日历和时段判断的实现细节，参见 [A股交易日历与时段判断](7-agu-jiao-yi-ri-li-yu-shi-duan-pan-duan)
- 了解 K 线数据如何馈入技术指标计算管线，参见 [技术指标管线与注册表机制](15-ji-zhu-zhi-biao-guan-xian-yu-zhu-ce-biao-ji-zhi)