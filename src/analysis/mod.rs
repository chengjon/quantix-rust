/// 分析模块
///
/// 技术指标计算、回测引擎、竞价分析、投资组合管理、性能计算

pub mod indicators;
pub mod backtest;
pub mod auction;
pub mod portfolio;
pub mod performance;

pub use indicators::*;
pub use backtest::*;
pub use auction::{
    AuctionAnalyzer, AuctionAnalysis, SectorStats, StrengthLevel,
    calculate_matched_ratio,
};
pub use portfolio::{Portfolio, Position, Order, OrderType, OrderStatus};
pub use performance::{PerformanceReport, PerformanceCalculator, TradeRecord, TradeSide};
