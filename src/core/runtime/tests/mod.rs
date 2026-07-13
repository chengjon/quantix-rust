//! 运行时（CliRuntime / ClickHouse / Bridge）测试。
//!
//! 从原生 `runtime/tests.rs` 拆出。`mod.rs` 把父模块（`settings::*` 导出
//! 的 env 常量与设置类型）重新导出，使子模块仍能通过 `use super::*` 访问；
//! 共用的 `ClickHouseEnvGuard` 放在 [`support` 子模块。

mod cli_paths;
mod settings_env;
mod support;

pub use super::*;
pub(crate) use support::ClickHouseEnvGuard;
