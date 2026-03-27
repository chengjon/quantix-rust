pub mod analysis;
/// quantix-cli - A股量化交易 CLI 工具
///
/// 与 Python quantix 项目共享数据源和数据库
///
/// ## 功能模块
/// - `account`: 多账户管理
/// - `analysis`: 回测引擎、性能分析、技术指标
/// - `anomaly`: 异常检测 (Isolation Forest)
/// - `sources`: 数据源适配器 (TDX, AkShare, 文件解析)
/// - `db`: 数据库客户端 (PostgreSQL, TDengine, ClickHouse)
/// - `data`: 数据模型
/// - `monitoring`: 实时监控系统 (Phase 16)
/// - `io`: 数据导入导出 (Phase 17)
/// - `strategy`: 交易策略
/// - `tasks`: 任务调度
/// - `news`: 新闻搜索模块 (Phase 3)
pub mod account;
pub mod ai;
pub mod anomaly;
pub mod cli;
pub mod core;
pub mod data;
pub mod db;
pub mod bridge;
pub mod execution;
pub mod fundamental;
pub mod io;
pub mod market;
pub mod monitor;
pub mod monitoring;
pub mod news;
pub mod risk;
pub mod screener;
pub mod sources;
pub mod stop;
pub mod strategy;
pub mod sync;
pub mod tasks;
pub mod trade;
pub mod tui;
pub mod watchlist;

// 重新导出常用类型
pub use cli::Cli;
pub use core::{QuantixError, Result};
pub use data::models::*;
pub use sources::*;
