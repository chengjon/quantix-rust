//! 基本面数据命令处理器。
//!
//! 处理估值、财报、机构持仓、龙虎榜等基本面命令。两个子模块：
//! - [`commands`] 生命周期子命令与格式化辅助，包含入口
//!   [`run_fundamental_command`]，
//! - [`tests`] fail-closed 集成测试（待数据源接入的命令）。

mod commands;
#[cfg(test)]
mod tests;

pub use commands::run_fundamental_command;
