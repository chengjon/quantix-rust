//! 基本面子命令入口、未接入桩命令与格式化辅助。
//!
//! 数据获取实现已拆至 [`queries`] 子模块。

use crate::cli::command_types::FundamentalCommands;
use crate::core::{QuantixError, Result};
use rust_decimal::Decimal;

mod queries;

// ============================================================
// 基本面数据命令
// ============================================================

/// 处理基本面命令
pub async fn run_fundamental_command(cmd: FundamentalCommands) -> Result<()> {
    match cmd {
        FundamentalCommands::Show { code } => queries::run_fundamental_show(&code).await,
        FundamentalCommands::Valuation { code } => queries::run_fundamental_valuation(&code).await,
        FundamentalCommands::Earnings { code, years } => {
            queries::run_fundamental_earnings(&code, years).await
        }
        FundamentalCommands::Institution { code } => {
            queries::run_fundamental_institution(&code).await
        }
        FundamentalCommands::DragonTiger { code, days } => {
            queries::run_fundamental_dragon_tiger(code.as_deref(), days).await
        }
        FundamentalCommands::CapitalFlow { code, days } => {
            run_fundamental_capital_flow(&code, days).await
        }
        FundamentalCommands::Dividend { code, years } => {
            run_fundamental_dividend(&code, years).await
        }
    }
}

pub(crate) async fn run_fundamental_capital_flow(code: &str, days: u32) -> Result<()> {
    println!("💸 资金流向");
    println!("   代码: {}", code);
    println!("   天数: {}", days);
    println!();

    println!("⚠️ 资金流向数据源尚未接入，命令将返回 Unsupported");
    println!();
    println!("💡 您可以先使用以下命令查看其他基本面数据:");
    println!("   - quantix fundamental valuation {}   # 估值指标", code);
    println!("   - quantix fundamental earnings {}    # 财报数据", code);
    println!("   - quantix fundamental institution {} # 机构持仓", code);

    Err(QuantixError::Unsupported(format!(
        "fundamental capital-flow 尚未接入真实资金流向数据源；code={code}, days={days}"
    )))
}

pub(crate) async fn run_fundamental_dividend(code: &str, years: u32) -> Result<()> {
    println!("💰 分红信息");
    println!("   代码: {}", code);
    println!("   年数: {}", years);
    println!();

    println!("⚠️ 分红数据源尚未接入，命令将返回 Unsupported");
    println!();
    println!("💡 您可以先使用以下命令查看其他基本面数据:");
    println!("   - quantix fundamental valuation {}   # 估值指标", code);
    println!("   - quantix fundamental earnings {}    # 财报数据", code);
    println!("   - quantix fundamental institution {} # 机构持仓", code);

    Err(QuantixError::Unsupported(format!(
        "fundamental dividend 尚未接入真实分红数据源；code={code}, years={years}"
    )))
}

/// Helper function to format optional Decimal values for display
fn format_decimal(value: &Option<Decimal>) -> String {
    match value {
        Some(v) => format!("{:.2}", v),
        None => "-".to_string(),
    }
}

/// Helper function to truncate a string to a maximum length
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
