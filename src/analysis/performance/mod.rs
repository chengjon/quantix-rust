//! Backtest performance analytics: a calculator engine that folds an equity
//! curve + trade log into a [`PerformanceReport`], plus the report/trade
//! record types and a few free-standing metric helpers.

mod calculator;
mod metrics;
mod report;

pub use calculator::PerformanceCalculator;
pub use report::{EquityPoint, PerformanceReport, TradeRecord, TradeSide};
