//! Algorithmic Trading CLI Handler。
//!
//! 处理 TWAP/VWAP 算法交易命令。三个子模块：
//! - [`store`] 全局算法存储（单例 + AlgoStore），
//! - [`commands`] 生命周期子命令与状态展示，包含入口
//!   [`run_algo_command`]，
//! - [`plan_preview`] 切片计划生成与表格/JSON 输出。

mod commands;
mod plan_preview;
mod store;

pub use commands::run_algo_command;
