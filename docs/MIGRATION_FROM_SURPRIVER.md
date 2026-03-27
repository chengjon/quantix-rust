# Surpriver 迁移方案 - 异常检测模块

> **目标**: 将 Surpriver 的 Isolation Forest 异常检测功能迁移到 quantix-rust
> **创建日期**: 2026-03-26
> **状态**: 规划中

---

## 一、项目概述

### 1.1 核心功能

Surpriver 是一个基于 Isolation Forest（隔离森林）算法的股票异常检测工具。

**核心思路**:
```
不预测涨跌方向 → 通过异常检测找"不正常"的股票 → 异常股票未来变动是正常股票的2倍+
```

### 1.2 工作流程

```
股票列表 → 获取OHLCV数据 → 计算技术指标特征 → Isolation Forest训练 → 输出异常股票排名
```

### 1.3 迁移策略

采用**模块化集成**方式，将 Surpriver 功能作为新模块添加到 quantix-rust：

| 模块 | 源位置 | 目标位置 | 复用程度 |
|------|--------|----------|----------|
| Isolation Forest | `surpriver-core/isolation_forest.rs` | `src/anomaly/forest.rs` | 100% |
| Statistics | `surpriver-core/statistics.rs` | `src/anomaly/statistics.rs` | 100% |
| EOM Indicator | `surpriver-indicators/eom.rs` | `src/analysis/indicators.rs` | 合并 |
| Slope Calculator | `surpriver-indicators/slope.rs` | `src/analysis/regression.rs` | 100% |
| Feature Extractor | `surpriver-data/feature_extractor.rs` | `src/anomaly/features.rs` | 90% |
| Stock Filter | `surpriver-data/filters.rs` | `src/anomaly/filter.rs` | 80% |
| A-Share Data | `surpriver-data/ashare.rs` | `src/sources/eastmoney.rs` | 增强 |

---

## 二、目标架构

### 2.1 新增模块结构

```
src/
├── anomaly/                    # ★ 新增：异常检测模块
│   ├── mod.rs                  # 模块入口
│   ├── forest.rs               # Isolation Forest 算法
│   ├── statistics.rs           # 统计函数
│   ├── features.rs             # 特征提取
│   ├── filter.rs               # 股票过滤器
│   ├── detector.rs             # 异常检测服务
│   └── config.rs               # 配置
│
├── analysis/
│   ├── indicators.rs           # 扩展：添加 EOM 等指标
│   └── regression.rs           # ★ 新增：线性回归
│
├── sources/
│   └── eastmoney.rs            # 增强：完整东方财富 API
│
└── cli/
    └── commands/
        └── anomaly.rs          # ★ 新增：CLI 命令
```

### 2.2 数据流

```
┌─────────────────┐
│  EastMoney API  │
│  (股票列表/K线)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   StockFilter   │  ← ST/涨跌停/停牌/新股过滤
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ FeatureExtractor│  ← 成交量收益/对数收益/EOM斜率
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ IsolationForest │  ← 训练 + 异常评分
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  AnomalyResult  │  ← 输出异常股票排名
└─────────────────┘
```

---

## 三、详细实现计划

### Phase 1: 核心算法迁移 (1 天)

#### Task 1.1: 创建 anomaly 模块

**文件**: `src/anomaly/mod.rs`

```rust
//! 异常检测模块
//!
//! 基于 Isolation Forest 算法的股票异常检测

pub mod forest;
pub mod statistics;
pub mod features;
pub mod filter;
pub mod detector;
pub mod config;

pub use forest::{IsolationForest, AnomalyScore};
pub use features::{FeatureExtractor, FeatureConfig};
pub use filter::{StockFilter, FilterConfig};
pub use detector::AnomalyDetector;
pub use config::AnomalyConfig;
```

#### Task 1.2: 迁移 Isolation Forest

**源文件**: `/opt/claude/surpriver/RUST/crates/surpriver-core/src/isolation_forest.rs`
**目标**: `src/anomaly/forest.rs`

**关键修改**:
- 错误类型改为使用 `crate::core::QuantixError`
- 添加 `Serialize/Deserialize` 支持

#### Task 1.3: 迁移统计函数

**源文件**: `/opt/claude/surpriver/RUST/crates/surpriver-core/src/statistics.rs`
**目标**: `src/anomaly/statistics.rs`

---

### Phase 2: 特征提取 (1 天)

#### Task 2.1: 适配 OHLCV 数据模型

当前 quantix-rust 使用 `Kline` 模型，surpriver 使用 `OHLCVSeries`。

**解决方案**: 创建适配器

```rust
// src/anomaly/features.rs

use crate::data::models::Kline;

/// 从 Kline 转换为特征向量
impl From<&[Kline]> for FeatureMatrix {
    fn from(klines: &[Kline]) -> Self {
        // 提取 open, high, low, close, volume
        // 计算 volume_returns, log_returns, EOM
    }
}
```

#### Task 2.2: 特征提取器

**特征组成** (默认配置):

| 特征类型 | 计算方法 | 维度 |
|----------|----------|------|
| `volume_returns` | `V[t] / V[t-1]` | 7 原始值 + slope + R² + p-value |
| `log_returns` | `ln(C[t] / C[t-1])` | 7 原始值 |
| `eom_5` | EOM (period=5) | slope + R² + p-value |
| `eom_10` | EOM (period=10) | slope + R² + p-value |
| `eom_20` | EOM (period=20) | slope + R² + p-value |

**总维度**: 7 + 3 + 7 + 3*3 = 27 维

---

### Phase 3: 股票过滤器 (0.5 天)

#### Task 3.1: A股特通过滤器

**文件**: `src/anomaly/filter.rs`

```rust
/// 过滤配置
pub struct FilterConfig {
    /// 最小平均成交量（手）
    pub min_volume: f64,
    /// 最小波动率
    pub min_volatility: f64,
    /// 排除ST股票
    pub exclude_st: bool,
    /// 排除涨跌停
    pub exclude_limit: bool,
    /// 最小上市天数
    pub min_listing_days: usize,
    /// 最小K线数量
    pub min_candles: usize,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            min_volume: 10000.0,
            min_volatility: 0.03,
            exclude_st: true,
            exclude_limit: true,
            min_listing_days: 60,
            min_candles: 50,
        }
    }
}
```

**过滤规则**:

| 过滤器 | 规则 |
|--------|------|
| ST股票 | 名称包含 ST/*ST/退 |
| 涨跌停 | 主板 ±10%，创业板/科创板 ±20%，ST ±5% |
| 停牌 | 连续5根K线收盘价相同 |
| 新股 | 上市不足60天 |
| 低成交量 | 平均成交量 < 阈值 |
| 低波动率 | 标准差 < 阈值 |

---

### Phase 4: 数据源增强 (0.5 天)

#### Task 4.1: 增强东方财富数据源

**文件**: `src/sources/eastmoney.rs`

**新增方法**:

```rust
impl EastMoneySource {
    /// 获取全部A股列表（增强版）
    pub async fn get_all_ashare_list(&self) -> Result<Vec<StockInfo>>;

    /// 获取分钟K线
    pub async fn get_minute_klines(
        &self,
        code: &str,
        period: u32,  // 1, 5, 15, 30, 60
        adjust: AdjustType,
        limit: usize,
    ) -> Result<Vec<Kline>>;

    /// 批量获取K线（并行）
    pub async fn get_klines_batch(
        &self,
        codes: &[&str],
        period: u32,
        adjust: AdjustType,
        limit: usize,
    ) -> Result<HashMap<String, Vec<Kline>>>;
}
```

**API 端点**:

```
股票列表: https://push2.eastmoney.com/api/qt/clist
  参数: fs=m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23
        fields=f12,f14,f2,f3,f5,f6

K线数据: https://push2his.eastmoney.com/api/qt/stock/kline/get
  参数: secid={市场}.{代码}
        klt={周期}  (1/5/15/30/60 分钟, 101 日线)
        fqt={复权}  (0不复权, 1前复权, 2后复权)
        lmt={数量}
```

---

### Phase 5: CLI 集成 (0.5 天)

#### Task 5.1: 添加异常检测命令

**文件**: `src/cli/commands/anomaly.rs`

```rust
/// 异常检测命令
#[derive(Debug, Parser)]
pub struct AnomalyCmd {
    /// 显示的异常股票数量
    #[arg(short, long, default_value = "20")]
    top_n: usize,

    /// K线周期（分钟）: 1, 5, 15, 30, 60
    #[arg(short, long, default_value = "15")]
    period: u32,

    /// 最小成交量过滤（手）
    #[arg(long, default_value = "10000")]
    min_volume: f64,

    /// 最小波动率过滤
    #[arg(long, default_value = "0.03")]
    min_volatility: f64,

    /// 输出格式: cli, json, csv
    #[arg(short, long, default_value = "cli")]
    output: String,

    /// Isolation Forest 树数量
    #[arg(long, default_value = "100")]
    n_estimators: usize,

    /// 历史K线数量用于特征
    #[arg(long, default_value = "7")]
    history_to_use: usize,
}
```

#### Task 5.2: 实现主流程

```rust
impl AnomalyCmd {
    pub async fn execute(&self) -> Result<()> {
        // 1. 创建数据源
        let source = EastMoneySource::new();

        // 2. 获取股票列表
        let stocks = source.get_all_ashare_list().await?;

        // 3. 过滤股票
        let filter = StockFilter::new(FilterConfig {
            min_volume: self.min_volume,
            min_volatility: self.min_volatility,
            ..Default::default()
        });
        let filtered = filter.filter_stock_list(&stocks);

        // 4. 并行获取K线
        let codes: Vec<&str> = filtered.iter().map(|s| s.code.as_str()).collect();
        let klines = source.get_klines_batch(&codes, self.period, AdjustType::QFQ, 100).await?;

        // 5. 提取特征
        let extractor = FeatureExtractor::new(FeatureConfig {
            history_to_use: self.history_to_use,
            ..Default::default()
        });
        let (features, symbols) = extractor.extract_from_klines(&klines);

        // 6. 训练 Isolation Forest
        let mut forest = IsolationForest::new()
            .n_estimators(self.n_estimators)
            .random_state(42);
        forest.fit(&features)?;

        // 7. 找出异常股票
        let anomalies = forest.find_anomalies(&features, &symbols, self.top_n);

        // 8. 输出结果
        self.output_results(&anomalies, &filtered);

        Ok(())
    }
}
```

---

## 四、依赖变更

### 4.1 Cargo.toml 新增依赖

```toml
[dependencies]
# 现有依赖...

# 异常检测新增
rand = "0.8"
rand_chacha = "0.3"
rayon = "1.8"
statrs = "0.16"
```

### 4.2 功能开关

```toml
[features]
default = ["postgresql", "tdengine-rest", "anomaly"]
anomaly = ["rand", "rand_chacha", "rayon", "statrs"]
```

---

## 五、测试计划

### 5.1 单元测试

```bash
# 测试 Isolation Forest
cargo test -p quantix-cli anomaly::forest

# 测试特征提取
cargo test -p quantix-cli anomaly::features

# 测试过滤器
cargo test -p quantix-cli anomaly::filter
```

### 5.2 集成测试

```bash
# 编译 release 版本
cargo build --release

# 运行异常检测
./target/release/quantix anomaly --top_n 20 --period 15 --output cli

# JSON 输出
./target/release/quantix anomaly --top_n 10 --output json > anomalies.json
```

### 5.3 验证检查清单

- [ ] 股票列表获取正常（~5000只A股）
- [ ] K线数据获取正常（无空数据）
- [ ] ST股票被正确过滤
- [ ] 涨跌停股票被正确过滤
- [ ] 特征提取无 NaN 值
- [ ] Isolation Forest 训练成功
- [ ] 输出格式正确（CLI/JSON）
- [ ] 异常分数范围合理（约 -0.5 到 0.5）

---

## 六、输出示例

### CLI 输出

```
================================================================================
📊 TOP 20 ANOMALOUS STOCKS (Isolation Forest)
================================================================================

[1] 000001 平安银行
  异常分数: -0.0456 ⚠️ 异常
  最新时间: 2026-03-26 14:45
  今日成交量: 1234.56万
  5日均量: 876.23万
  20日均量: 945.67万
  5周期波动率: 0.0210
  20周期波动率: 0.0380

[2] 600519 贵州茅台
  异常分数: -0.0389 ⚠️ 异常
  ...

📌 说明:
  - 异常分数 < 0: 异常股票，分数越低越异常
  - 异常股票的未来价格变动通常是正常股票的2倍以上
  - 算法不预测涨跌方向，只检测异常模式
```

### JSON 输出

```json
{
  "timestamp": "2026-03-26T14:45:00Z",
  "config": {
    "period": 15,
    "n_estimators": 100,
    "history_to_use": 7
  },
  "anomalies": [
    {
      "rank": 1,
      "code": "000001",
      "name": "平安银行",
      "anomaly_score": -0.0456,
      "is_anomaly": true,
      "volume_ratio": 1.41,
      "volatility_5": 0.021,
      "volatility_20": 0.038
    }
  ]
}
```

---

## 七、文件清单

### 7.1 新建文件

| 文件 | 说明 |
|------|------|
| `src/anomaly/mod.rs` | 模块入口 |
| `src/anomaly/forest.rs` | Isolation Forest |
| `src/anomaly/statistics.rs` | 统计函数 |
| `src/anomaly/features.rs` | 特征提取 |
| `src/anomaly/filter.rs` | 股票过滤器 |
| `src/anomaly/detector.rs` | 检测服务 |
| `src/anomaly/config.rs` | 配置 |
| `src/analysis/regression.rs` | 线性回归 |
| `src/cli/commands/anomaly.rs` | CLI 命令 |

### 7.2 修改文件

| 文件 | 修改内容 |
|------|----------|
| `src/lib.rs` | 添加 `pub mod anomaly;` |
| `src/analysis/mod.rs` | 添加 `pub mod regression;` |
| `src/analysis/indicators.rs` | 添加 EOM 指标 |
| `src/sources/eastmoney.rs` | 增强API支持 |
| `src/cli/mod.rs` | 添加 anomaly 命令 |
| `Cargo.toml` | 添加依赖 |

---

## 八、风险提示

⚠️ **重要声明**:

1. 本工具只检测异常，**不预测涨跌方向**
2. 异常不等于机会，可能是下跌前兆
3. A股受政策影响大，历史规律可能失效
4. 数据源（东方财富 API）稳定性需监控
5. 仅供技术研究和学习参考，不构成投资建议

---

**文档版本**: v1.0
**最后更新**: 2026-03-26
