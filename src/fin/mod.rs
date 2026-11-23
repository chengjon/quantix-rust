//! fin-primitives 研究库集成层（设计见 docs/superpowers/specs/2026-11-23-fin-primitives-migration-design.md）。
//!
//! 边界纪律：fin-primitives 类型（BarInput/OhlcvBar/Signal/BacktestResult 等）只在本模块内部出现；
//! 对外仅暴露 quantix 类型（Decimal/Kline/core::Result）。唯一例外：FinError 以 `QuantixError::Fin`
//! 载荷形式越界（见 error_bridge），其余 fin-primitives 类型不越出本模块。

pub mod adapter;
pub mod error_bridge;
