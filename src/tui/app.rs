/// TUI 应用
///
/// 交互式菜单界面

use crate::core::Result;

/// 运行交互式菜单
pub fn run_menu() -> Result<()> {
    // TODO: 实现 ratatui 菜单
    println!("=== Quantix CLI ===");
    println!("1. 数据同步");
    println!("2. 策略运行");
    println!("3. 回测分析");
    println!("4. 任务管理");
    println!("0. 退出");
    Ok(())
}
