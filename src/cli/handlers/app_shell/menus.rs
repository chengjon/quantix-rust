use super::*;

pub async fn run_simple_menu() -> Result<()> {
    loop {
        println!("\n=== Quantix CLI 交互菜单 ===\n");

        let items = vec![
            "📊 数据同步   — 查询、导出K线数据，管理数据源配置",
            "📈 策略运行   — 查看、创建和管理交易策略",
            "🔙 回测分析   — 用历史数据验证策略表现",
            "⏰ 任务管理   — 查看、启动预置定时任务",
            "💹 技术分析   — 计算MA/RSI/MACD等技术指标",
            "📤 数据导出   — 导出K线数据为CSV或Parquet格式",
            "❌ 退出",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact()
            .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

        match selection {
            0 => run_data_sync_menu().await?,
            1 => strategy_handler::run_strategy_menu().await?,
            2 => run_backtest_menu().await?,
            3 => run_task_menu().await?,
            4 => run_analysis_menu().await?,
            5 => run_export_menu().await?,
            6 => {
                println!("👋 再见！");
                break;
            }
            _ => {}
        }
    }
    Ok(())
}

pub async fn run_tui_menu() -> Result<()> {
    #[cfg(feature = "tui")]
    {
        match crate::tui::run_menu()? {
            crate::tui::TuiMenuAction::DataSync => run_data_sync_menu().await?,
            crate::tui::TuiMenuAction::StrategyRun => strategy_handler::run_strategy_menu().await?,
            crate::tui::TuiMenuAction::Backtest => run_backtest_menu().await?,
            crate::tui::TuiMenuAction::TaskManagement => run_task_menu().await?,
            crate::tui::TuiMenuAction::TechnicalAnalysis => run_analysis_menu().await?,
            crate::tui::TuiMenuAction::DataExport => run_export_menu().await?,
            crate::tui::TuiMenuAction::Exit => {
                println!("👋 再见！");
            }
        }
    }

    #[cfg(not(feature = "tui"))]
    {
        println!("🎨 TUI 菜单需要启用 tui feature");
        println!("💡 使用: cargo run --features tui -- menu --tui");
        println!("💡 或使用 'quantix menu' 进入简单菜单");
    }

    Ok(())
}

async fn run_task_menu() -> Result<()> {
    let items = vec![
        "查看任务列表   — 显示预置的定时任务模板",
        "启动调度器     — 前台运行盘前/竞价/开盘/收盘/盘后任务",
        "返回",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => list_tasks().await?,
        1 => {
            println!("  💡 调度器将在前台运行，按 Ctrl+C 可停止");
            start_task_scheduler(false).await?;
        }
        2 => {}
        _ => {}
    }

    Ok(())
}

async fn run_analysis_menu() -> Result<()> {
    let items = vec![
        "计算技术指标   — 输入股票代码和指标名称，计算MA/RSI/MACD等",
        "返回",
    ];

    println!("\n  💡 提示: 技术分析需要该股票的K线数据已存在于数据库中");
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => {
            println!("  💡 常用指标: ma5,ma10,ma20,ma60,rsi14,macd,kdj,boll");
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            let indicators = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("指标列表 (逗号分隔)")
                .default("ma5,ma20,rsi14".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            calculate_indicators(code, indicators).await?;
        }
        1 => {}
        _ => {}
    }

    Ok(())
}

async fn run_backtest_menu() -> Result<()> {
    let items = vec![
        "新建回测       — 输入股票代码，使用均线交叉策略回测",
        "查看历史回测   — 浏览已完成的回测报告",
        "返回",
    ];

    println!("\n  💡 提示: 回测需要该股票的K线数据已存在于数据库中");
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => {
            println!("  💡 请确保该股票的K线数据已同步 (quantix data query 可验证)");
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            run_ma_cross_backtest(Some(code)).await?;
        }
        1 => run_backtest_command(backtest_history_menu_command()).await?,
        2 => {}
        _ => {}
    }

    Ok(())
}

async fn run_export_menu() -> Result<()> {
    let items = vec![
        "导出为 CSV     — 导出K线数据为CSV格式，可用Excel打开",
        "导出为 Parquet — 导出为Parquet列式格式，适合大数据分析",
        "返回",
    ];

    println!("\n  💡 提示: 导出前请确认该股票的K线数据已同步到数据库");
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => {
            println!("  💡 CSV文件将保存到 ./data/ 目录，可用Excel/WPS直接打开");
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            export_data(code, "csv".to_string(), "./data".to_string()).await?;
        }
        1 => {
            println!("  💡 Parquet格式体积更小、读取更快，适合Python/Pandas分析");
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            export_data(code, "parquet".to_string(), "./data".to_string()).await?;
        }
        2 => {}
        _ => {}
    }

    Ok(())
}
