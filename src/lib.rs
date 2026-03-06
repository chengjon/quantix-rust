/// quantix-cli - A股量化交易 CLI 工具
///
/// 与 Python quantix 项目共享数据源和数据库
///
/// ## 功能模块
/// - `analysis`: 回测引擎、性能分析、技术指标
/// - `sources`: 数据源适配器 (TDX, AkShare, 文件解析)
/// - `db`: 数据库客户端 (PostgreSQL, TDengine, ClickHouse)
/// - `data`: 数据模型
/// - `tasks`: 任务调度

pub mod cli;
pub mod core;
pub mod db;
pub mod data;
pub mod sources;
pub mod analysis;
pub mod sync;
pub mod strategy;
pub mod tasks;
pub mod tui;

// 重新导出常用类型
pub use core::{Result, QuantixError};
pub use data::models::*;
pub use sources::*;
