//! Account Command Handlers
//!
//! 多账户管理 CLI 命令处理器

mod commands;

#[cfg(test)]
mod tests;

pub use commands::run_account_command;
