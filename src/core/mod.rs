/// 核心模块
///
/// 配置管理、错误处理、时间处理等核心功能
pub mod config;
pub mod error;
pub mod performance_utils;
pub mod runtime;
pub mod trading_calendar;
pub mod trading_time;

pub use error::{QuantixError, Result};
pub use performance_utils::{
    BatchOptimizationConfig, MemoryTracker, OptimizationSuggestion, PerfTimer, analyze_performance,
};
pub use runtime::{CliRuntime, ClickHouseSettings};
