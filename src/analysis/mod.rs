pub mod auction;
pub mod backtest;
pub mod candle_patterns;
pub mod indicator_cache;
pub mod indicator_config;
pub mod indicator_registry;
/// 分析模块
///
/// 技术指标计算、回测引擎、竞价分析、投资组合管理、性能计算
pub mod indicators;
pub mod indicators_benches;
pub mod performance;
pub mod pipeline;
pub mod polars_adapter;
pub mod portfolio;
pub use indicator_cache::{IndicatorCache, IndicatorCacheKey};
pub use indicator_config::{IndicatorInstanceId, IndicatorPipelineConfig, IndicatorSpec};
pub use indicator_registry::{
    IndicatorDescriptor, IndicatorInput, IndicatorMeta, IndicatorRegistry, IndicatorSeries,
    IndicatorSeriesKind,
};
pub use pipeline::{IndicatorOutputMap, IndicatorPipeline};

pub use auction::{
    AuctionAnalysis, AuctionAnalyzer, SectorStats, StrengthLevel, calculate_matched_ratio,
};
pub use backtest::*;
pub use candle_patterns::*;
pub use indicators::*;
pub use indicators_benches::*;
pub use performance::{PerformanceCalculator, PerformanceReport, TradeRecord, TradeSide};
pub use polars_adapter::{
    BatchKlineData, MultiStockData, PolarsCalculator, from_kline_vec, init_polars,
};
pub use portfolio::{Order, OrderStatus, OrderType, Portfolio, Position};
