//! 基本面子命令 fail-closed 集成测试。
//!
//! 从原生 `fundamental.rs` 拆出。验证尚未接入真实数据源的命令返回
//! [`Unsupported`] 错误，而非静默成功。

use crate::cli::command_types::FundamentalCommands;
use crate::cli::handlers::fundamental::commands::{
    run_fundamental_capital_flow, run_fundamental_dividend,
};
use crate::core::{QuantixError, Result};

#[tokio::test]
async fn fundamental_dividend_returns_unsupported_until_provider_is_wired() {
    let error = run_fundamental_dividend("600519", 3)
        .await
        .expect_err("dividend must fail closed until real dividend data is wired");

    assert!(matches!(
        error,
        QuantixError::Unsupported(message)
            if message.contains("fundamental dividend")
                && message.contains("尚未接入真实分红数据源")
    ));
}

#[tokio::test]
async fn fundamental_capital_flow_returns_unsupported_until_provider_is_wired() {
    let error = run_fundamental_capital_flow("600519", 5)
        .await
        .expect_err("capital-flow must fail closed until real capital-flow data is wired");

    assert!(matches!(
        error,
        QuantixError::Unsupported(message)
            if message.contains("fundamental capital-flow")
                && message.contains("尚未接入真实资金流向数据源")
    ));
}
