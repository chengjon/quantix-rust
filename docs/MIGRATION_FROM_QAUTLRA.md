# qautlra-rs 功能迁移分析

> 分析日期: 2026-03-26
> 目标: 将 qautlra-rs 的核心功能移植到 quantix-rust

## 一、项目对比

### qautlra-rs 架构

```
qautlra-rs/
├── qamd-rs/           # 核心数据结构 (MDSnapshot, Tick, DailyBar, MinuteBar)
├── ctp-common/        # CTP API 通用绑定 (GB18030编码, FFI)
├── ctp-md/            # CTP 行情 API (链接 thostmduserapi_se)
├── ctp-md-qq/         # QQ财经行情源
├── ctp-md-sina/       # Sina财经行情源
├── ctp-trader/        # CTP 交易 API
├── openctp-trader/    # OpenCTP 交易 API
├── qamdgateway/       # 市场数据网关 (Actix Actor 模型)
│   ├── actors/        # MarketDataActor, MarketDataDistributor
│   ├── ws_server.rs   # WebSocket 服务器
│   └── converter.rs   # 行情格式转换
└── xtp-rs/            # XTP 交易所 API
```

### quantix-rust 架构

```
quantix-rust/
├── src/
│   ├── sources/       # 数据源 (TDX, AkShare, EastMoney, WebSocket)
│   ├── data/          # 数据模型 (Kline, Tick, StockInfo)
│   ├── db/            # 数据库 (PostgreSQL, ClickHouse, TDengine)
│   ├── strategy/      # 策略模块
│   ├── risk/          # 风控模块
│   ├── execution/     # 执行模块
│   └── ...
```

## 二、可迁移功能

### 优先级 1: 高价值 - 核心数据结构

| 模块 | 来源 | 目标位置 | 价值 |
|------|------|----------|------|
| **MDSnapshot** | qamd-rs/snapshot.rs | src/data/ | 10档行情深度，现有 Tick 仅有基础字段 |
| **OptionalF64** | qamd-rs/types.rs | src/data/ | 处理中国数据源 "-" 字符串 |
| **DailyBar/MinuteBar** | qamd-rs/daily.rs, minute.rs | src/data/ | 标准化K线结构，复权支持 |

**迁移代码示例** (qamd-rs/snapshot.rs):
```rust
// 10档行情快照 - quantix-rst 缺少
pub struct MDSnapshot {
    pub instrument_id: String,
    pub bid_price1: f64,  pub ask_price1: f64,
    pub bid_price2: Option<f64>, pub ask_price2: Option<f64>,
    // ... 10档买卖价
    pub bid_volume1: i64, pub ask_volume1: i64,
    // ... 10档买卖量
    pub open_interest: OptionalF64,  // 期货持仓量
    pub iopv: OptionalF64,           // ETF净值
}
```

### 优先级 2: 中等价值 - CTP 直连能力

| 模块 | 来源 | 目标位置 | 价值 |
|------|------|----------|------|
| **ctp-common** | ctp-common/src/ | src/sources/ctp/ | CTP API FFI 绑定 |
| **GB18030编码** | ctp-common/src/lib.rs | src/sources/ctp/ | 中文编码转换 |
| **CTP 行情** | ctp-md/ | src/sources/ctp/md/ | 期货/期权实时行情 |
| **CTP 交易** | ctp-trader/ | src/execution/ctp/ | 期货/期权下单 |

**依赖**: 需要链接 `thostmduserapi_se` (CTP官方库)

### 优先级 3: 可选 - Actor 模型网关

| 模块 | 来源 | 目标位置 | 价值 |
|------|------|----------|------|
| **Actor 架构** | qamdgateway/src/actors/ | src/gateway/ | 高并发行情分发 |
| **WebSocket 服务** | qamdgateway/src/ws_server.rs | src/gateway/ | 统一行情推送 |
| **格式转换器** | qamdgateway/src/converter.rs | src/gateway/ | TradingView 格式 |

**注意**: quantix-rust 已有 WebSocket 数据源，需评估是否需要替换

## 三、迁移策略

### 阶段 1: 数据结构对齐 (1-2天)

1. 将 `OptionalF64` 类型添加到 `src/data/types.rs`
2. 扩展现有 `Tick` 为 `MDSnapshot` (或新建)
3. 引入 `DailyBar`/`MinuteBar` 结构

```rust
// src/data/types.rs - 新增
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptionalF64 {
    Value(f64),
    String(String),  // 处理 "-"
    Null,
}

// src/data/market_snapshot.rs - 新建
pub struct MarketSnapshot {
    pub instrument_id: String,
    pub bid_price: [Option<f64>; 10],
    pub ask_price: [Option<f64>; 10],
    pub bid_volume: [Option<i64>; 10],
    pub ask_volume: [Option<i64>; 10],
    // ... 其他字段
}
```

### 阶段 2: CTP 集成 (3-5天)

1. 创建 `src/sources/ctp/` 目录
2. 移植 `ctp-common` 的 FFI 绑定
3. 移植 `ctp-md` 的行情订阅
4. 实现 `TdxSource` trait 接口

```
src/sources/ctp/
├── mod.rs
├── binding.rs      # FFI 绑定
├── encoding.rs     # GB18030 转换
├── md_api.rs       # 行情 API
└── trader_api.rs   # 交易 API (可选)
```

### 阶段 3: 网关集成 (可选)

如果需要 Actor 模型的高并发分发:
1. 添加 `actix` 依赖
2. 移植 `MarketDataActor` 和 `MarketDataDistributor`
3. 与现有 WebSocket 源整合

## 四、依赖对比

| 依赖 | qautlra-rs | quantix-rust | 迁移影响 |
|------|------------|--------------|----------|
| actix | ✅ | ❌ | 需新增 (如果用Actor) |
| actix-web | ✅ | ❌ | 需新增 |
| crossbeam-channel | ✅ | ❌ | 需新增 |
| hashbrown | ✅ | ❌ | 可选优化 |
| arrow2 | ✅ | arrow | 版本不同 |
| chrono | ✅ | ✅ | 兼容 |
| serde | ✅ | ✅ | 兼容 |

## 五、风险与注意事项

1. **CTP 库依赖**: CTP 官方库 `thostmduserapi_se` 需要单独安装
2. **编码问题**: CTP 使用 GB18030，需要编码转换模块
3. **API 差异**: qautlra-rs 的 Actor 模型与 quantix-rust 的 async/await 风格不同
4. **测试覆盖**: 迁移后需要补充单元测试

## 六、建议执行顺序

```
[ ] 1. 创建 src/data/market_snapshot.rs - 10档行情
[ ] 2. 创建 src/data/types.rs - OptionalF64 等工具类型
[ ] 3. 创建 src/sources/ctp/ - CTP 数据源
[ ] 4. (可选) 创建 src/gateway/ - Actor 网关
[ ] 5. 更新 src/sources/mod.rs - 导出新模块
[ ] 6. 补充测试用例
```

## 七、文件映射表

| qautlra-rs 文件 | quantix-rust 目标 |
|-----------------|-------------------|
| qamd-rs/src/snapshot.rs | src/data/market_snapshot.rs |
| qamd-rs/src/types.rs | src/data/types.rs |
| qamd-rs/src/daily.rs | src/data/daily_bar.rs |
| qamd-rs/src/minute.rs | src/data/minute_bar.rs |
| ctp-common/src/lib.rs | src/sources/ctp/encoding.rs |
| ctp-common/src/binding.rs | src/sources/ctp/binding.rs |
| ctp-md/src/*.rs | src/sources/ctp/md_*.rs |
| qamdgateway/src/converter.rs | src/sources/ctp/converter.rs |
