//! 异常检测模块
//!
//! 基于 Isolation Forest 算法的股票异常检测
//!
//! # 核心功能
//!
//! - `forest`: Isolation Forest 算法实现
//! - `statistics`: 统计函数（平均路径长度等）
//! - `features`: 从 OHLCV 数据提取特征
//! - `filter`: A股特通过滤器（ST、涨跌停、停牌等）
//! - `detector`: 异常检测服务（整合所有组件）
//! - `config`: 配置管理
//!
//! # 使用示例
//!
//! ```ignore
//! use quantix_cli::anomaly::{AnomalyDetector, AnomalyConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let config = AnomalyConfig::default();
//!     let detector = AnomalyDetector::new(config);
//!
//!     // 运行异常检测
//!     let results = detector.detect().await?;
//!
//!     for anomaly in results.iter().take(10) {
//!         println!("{}: score={:.4}", anomaly.code, anomaly.score);
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod detector;
pub mod eastmoney_source;
pub mod features;
pub mod filter;
pub mod forest;
pub mod statistics;

pub use config::{
    AnomalyConfig, DataConfig, FeatureConfig, FilterConfig, ForestConfig, OutputConfig,
};
pub use detector::{AnomalyDetector, AnomalyResult, DataSource, MockDataSource};
pub use eastmoney_source::EastMoneyAnomalySource;
pub use features::{FeatureExtractor, FeatureSet, OHLCVCandle, OHLCVSeries};
pub use filter::{StockFilter, StockInfo};
pub use forest::{AnomalyScore, IsolationForest};
