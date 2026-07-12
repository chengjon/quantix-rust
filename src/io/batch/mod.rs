//! 批处理模块。
//!
//! 大数据量导入导出优化处理。三个子模块各司其职：
//! - [`types`] 配置与进度类型，
//! - [`processor`] 批处理器及其全部方法，
//! - [`tests`] 行为测试（仅测试构建）。

#[cfg(test)]
mod tests;

mod processor;
mod types;

pub use processor::BatchProcessor;
pub use types::{BatchConfig, BatchProgress};
