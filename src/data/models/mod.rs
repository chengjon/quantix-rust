#![allow(clippy::should_implement_trait)]

/// 数据模型
///
/// 定义 K线、行情、交易等核心数据结构
/// 与 Python quantix 项目保持一致
pub mod types;

#[cfg(test)]
mod tests;

pub use types::{
    AdjustType, BarPeriod, CapitalChange, DateOrRange, GbbqEvent, Kline, Market, MinuteBar,
    MinutePeriod, MinuteShare, StockInfo, Tick, TradeDirection, iter_dates_inclusive,
};

// pub(crate) 函数以 pub(crate) use 保持原可见性，同时挂在父模块路径下供 crate 内消费。
pub(crate) use types::chunk_range_weekly;
