//! A股交易时间日历。
//!
//! 从短线侠项目迁移，支持交易时段检测和节假日判断。三个子模块：
//! - [`types`] 交易时段/状态/节假日数据类型，
//! - [`calendar`] `TradingCalendar` 管理器及其全部方法，
//! - [`tests`] 行为测试（仅测试构建）。

#[cfg(test)]
mod tests;

mod calendar;
mod types;

pub use calendar::TradingCalendar;
pub use types::{HolidayData, TradingSession, TradingStatus};
