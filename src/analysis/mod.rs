/// 分析模块
///
/// 技术指标计算、回测引擎、竞价分析

pub mod indicators;
pub mod backtest;
pub mod auction;

pub use indicators::*;
pub use backtest::*;
pub use auction::{
    AuctionAnalyzer, AuctionAnalysis, SectorStats, StrengthLevel,
    calculate_matched_ratio,
};
