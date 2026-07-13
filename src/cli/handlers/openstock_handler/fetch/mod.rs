//! OpenStock live-fetch 子命令实现。
//!
//! 从原生 `openstock_handler/fetch.rs` 拆出，承载实时拉取类 OpenStock
//! 子命令（股票列表、日历、指数/K 线、分钟线、分时）的实现，分组到
//! [`catalog`]（/data/fetch 信封记录类）和 [`bars`]（/data/bars 直读类）。

mod bars;
mod catalog;

pub(crate) use bars::*;
pub(crate) use catalog::*;
