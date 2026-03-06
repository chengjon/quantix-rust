/// 核心模块
///
/// 配置管理、错误处理、时间处理等核心功能

pub mod config;
pub mod error;
pub mod trading_time;
pub mod trading_calendar;

pub use error::{QuantixError, Result};
