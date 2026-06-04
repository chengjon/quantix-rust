use super::*;

use crate::core::{CliRuntime, QuantixError, Result};
use crate::tasks::{TaskScheduler, TaskTemplates};
use dialoguer::{Input, Select, theme::ColorfulTheme};
use std::path::Path;

pub async fn run_init(config_path: String) -> Result<()> {
    println!("🚀 初始化 Quantix CLI v{}", env!("CARGO_PKG_VERSION"));
    println!();

    // ── 1. 配置目录 ──
    let path = Path::new(&config_path);
    if path.exists() {
        println!("  ✅ 配置目录已存在: {}", config_path);
    } else {
        print!("  📁 创建配置目录: {} ... ", config_path);
        std::fs::create_dir_all(path).map_err(|e| {
            println!("❌");
            QuantixError::Other(format!("创建配置目录失败 ({}): {}", config_path, e))
        })?;
        println!("✅");
    }

    // ── 2. 加载运行时配置 ──
    println!("\n  ⚙️  加载运行时配置...");
    let rt = CliRuntime::load();
    println!(
        "    ClickHouse: {} db={}",
        rt.clickhouse.url, rt.clickhouse.database
    );
    println!(
        "    MySQL:      {} db={}",
        rt.upstream_mysql.url, rt.upstream_mysql.database
    );
    println!("    Bridge:     {}", rt.bridge.base_url);

    // ── 3. 创建数据目录 ──
    println!("\n  📂 准备数据目录...");
    let data_paths = [
        ("Watchlist", &rt.watchlist_path),
        ("Trade", &rt.trade_path),
        ("Risk", &rt.risk_path),
        ("Monitor DB", &rt.monitor_db_path),
        ("Monitor 配置", &rt.monitor_config_path),
        ("Strategy 配置", &rt.strategy_config_path),
        ("Strategy 运行时", &rt.strategy_runtime_db_path),
        ("Execution 配置", &rt.execution_config_path),
    ];

    let mut created = 0u32;
    let mut existing = 0u32;
    let mut failed = 0u32;
    for (label, file_path) in &data_paths {
        let parent = file_path.parent().unwrap_or(Path::new("."));
        if parent.exists() {
            existing += 1;
            println!("    ✅ {} -> {}", label, file_path.display());
        } else {
            match std::fs::create_dir_all(parent) {
                Ok(()) => {
                    created += 1;
                    println!("    🆕 {} -> {} (已创建)", label, file_path.display());
                }
                Err(e) => {
                    failed += 1;
                    println!("    ❌ {} -> {} (创建失败: {})", label, parent.display(), e);
                }
            }
        }
    }
    print!("    共 {} 个已存在, {} 个新建", existing, created);
    if failed > 0 {
        println!(", {} 个失败", failed);
        println!("    ⚠️  部分目录创建失败，后续操作可能出错");
    } else {
        println!();
    }

    // ── 4. 检查已有数据文件 ──
    println!("\n  📊 检查已有数据...");
    let mut has_data = false;
    for (label, file_path) in &data_paths {
        if file_path.exists() {
            has_data = true;
            let size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
            println!(
                "    📄 {} ({}) - {} bytes",
                label,
                file_path.display(),
                size
            );
        }
    }
    if !has_data {
        println!("    ℹ️  暂无数据文件 (使用各子命令初始化时自动创建)");
    }

    // ── 5. 初始化 Polars ──
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    print!("\n  🔧 初始化 Polars 计算引擎... ");
    crate::analysis::polars_adapter::init_polars()?;
    println!("✅ ({} 线程)", cpu_count);

    // ── 6. 环境检查 ──
    println!("\n  🔍 环境检查...");
    if Path::new(".env").exists() {
        println!("    ✅ .env 文件已找到");
    } else {
        println!("    ℹ️  未找到 .env 文件 (使用默认配置)");
    }

    let home = std::env::var_os("HOME")
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|| "(未设置)".to_string());
    println!("    HOME: {}", home);
    println!("    CPU:  {} 核心", cpu_count);

    // ── 7. 数据库连通性探测 ──
    println!("\n  🌐 数据库连通性探测...");

    let (ch_ok, mysql_ok, bridge_ok) = tokio::join!(
        probe_tcp_async(&rt.clickhouse.url),
        probe_tcp_async(&rt.upstream_mysql.url),
        probe_tcp_async(&rt.bridge.base_url),
    );

    if ch_ok {
        println!(
            "    ✅ ClickHouse ({}) - 可达 (db: {})",
            rt.clickhouse.url, rt.clickhouse.database
        );
    } else {
        println!(
            "    ⚠️  ClickHouse ({}) - 不可达 (后续查询操作将失败, db: {})",
            rt.clickhouse.url, rt.clickhouse.database
        );
    }

    if mysql_ok {
        println!(
            "    ✅ MySQL ({}) - 可达 (db: {})",
            rt.upstream_mysql.url, rt.upstream_mysql.database
        );
    } else {
        println!(
            "    ⚠️  MySQL ({}) - 不可达 (db: {})",
            rt.upstream_mysql.url, rt.upstream_mysql.database
        );
    }

    if bridge_ok {
        println!("    ✅ Bridge ({}) - 可达", rt.bridge.base_url);
    } else {
        println!(
            "    ⚠️  Bridge ({}) - 不可达 (行情数据源不可用)",
            rt.bridge.base_url
        );
    }

    // ── 汇总 ──
    println!();
    println!("✅ 初始化完成！");
    println!();
    println!("  📝 可用命令:");
    println!("    quantix status --health   查看系统健康状态");
    println!("    quantix data query        查询K线数据");
    println!("    quantix strategy list     查看策略列表");
    println!("    quantix trade init        初始化模拟交易账户");
    println!("    quantix task start        启动任务调度器");
    println!("    quantix menu              进入交互菜单");

    Ok(())
}

async fn probe_tcp_async(url: &str) -> bool {
    let stripped = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .or_else(|| url.strip_prefix("mysql://"))
        .or_else(|| url.strip_prefix("mysqlx://"))
        .or_else(|| url.strip_prefix("tcp://"))
        .unwrap_or(url);

    let host_port = stripped.split('/').next().unwrap_or(stripped);
    let (host, port) = if let Some(colon) = host_port.rfind(':') {
        (
            &host_port[..colon],
            host_port[colon + 1..].parse::<u16>().unwrap_or(0),
        )
    } else if url.starts_with("https://") {
        (host_port, 443)
    } else {
        (host_port, 80)
    };

    if host.is_empty() || port == 0 {
        return false;
    }

    tokio::time::timeout(
        std::time::Duration::from_secs(3),
        tokio::net::TcpStream::connect((host, port)),
    )
    .await
    .map(|r| r.is_ok())
    .unwrap_or(false)
}

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

pub async fn run_data_command(cmd: DataCommands) -> Result<()> {
    match cmd {
        DataCommands::Source(subcommand) => match subcommand {
            DataSourceCommands::List { config_dir } => {
                list_data_sources(&config_dir)?;
            }
            DataSourceCommands::Add {
                config_dir,
                source_type,
                hosts,
                port,
                timeout,
                base_url,
                rate_limit,
            } => {
                add_data_source(
                    &config_dir,
                    source_type,
                    hosts,
                    port,
                    timeout,
                    base_url,
                    rate_limit,
                )?;
            }
            DataSourceCommands::SetDefault { config_dir, name } => {
                set_default_data_source(&config_dir, name)?;
            }
            DataSourceCommands::Test { config_dir, name } => {
                test_data_source(&config_dir, name).await?;
            }
        },
        DataCommands::TdxApi(subcommand) => {
            super::tdx_api_handler::run_tdx_api_command(subcommand).await?;
        }
        DataCommands::ImportFundamentals { input } => {
            import_market_fundamentals(input).await?;
        }
        DataCommands::Query {
            code,
            start,
            end,
            r#type,
            limit,
        } => {
            query_kline_data(code, start, end, r#type, limit).await?;
        }
        DataCommands::Export {
            code,
            format,
            output,
        } => {
            export_data(code, format, output).await?;
        }
    }
    Ok(())
}

pub async fn run_task_command(cmd: TaskCommands) -> Result<()> {
    ensure_task_command_supported_for_p0(&cmd)?;

    match cmd {
        TaskCommands::Add {
            name,
            cron,
            command,
        } => {
            add_task(name, cron, command).await?;
        }
        TaskCommands::List => {
            list_tasks().await?;
        }
        TaskCommands::Start { daemon } => {
            start_task_scheduler(daemon).await?;
        }
        TaskCommands::Stop => {
            stop_task_scheduler().await?;
        }
        TaskCommands::Status => {
            show_task_status().await?;
        }
    }
    Ok(())
}

pub(crate) fn ensure_task_command_supported_for_p0(cmd: &TaskCommands) -> Result<()> {
    match cmd {
        TaskCommands::Add { .. } => Err(QuantixError::Unsupported(
            "Foundation P0 仅支持预置任务模板；`task add` 暂不开放".to_string(),
        )),
        TaskCommands::Start { daemon: true } => Err(QuantixError::Unsupported(
            "Foundation P0 仅支持前台直接执行；`task start --daemon` 暂不支持".to_string(),
        )),
        _ => Ok(()),
    }
}

async fn add_task(name: String, cron: String, command: String) -> Result<()> {
    println!("⏰ 添加任务: {}", name);
    println!("  Cron: {}", cron);
    println!("  命令: {}", command);

    Err(QuantixError::Unsupported(
        "Foundation P0 仅支持预置任务模板；请使用 `quantix task list` 查看可运行任务".to_string(),
    ))
}

pub(crate) fn foundation_p0_task_template_descriptions() -> Vec<(String, String, String)> {
    [
        TaskTemplates::pre_market_check(),
        TaskTemplates::auction_collection(),
        TaskTemplates::market_open(),
        TaskTemplates::market_close(),
        TaskTemplates::post_market_process(),
        TaskTemplates::data_sync(),
    ]
    .into_iter()
    .map(|task| (task.name, task.command, task.cron_expr))
    .collect()
}

async fn list_tasks() -> Result<()> {
    println!("📋 Foundation P0 预置任务模板:");
    println!();
    println!("  预定义任务模板 (名称 | 描述 | Cron):");
    for (name, command, cron) in foundation_p0_task_template_descriptions() {
        println!("    - {} | {} | {}", name, command, cron);
    }
    println!();
    println!("💡 Foundation P0 只支持前台启动预置模板: `quantix task start`");

    Ok(())
}

async fn start_task_scheduler(daemon: bool) -> Result<()> {
    if daemon {
        return Err(QuantixError::Unsupported(
            "Foundation P0 仅支持前台直接执行；后台守护模式暂不支持".to_string(),
        ));
    }

    println!("⏰ 启动任务调度器...");

    let scheduler = TaskScheduler::new()
        .await
        .map_err(|e| QuantixError::Other(format!("创建调度器失败: {}", e)))?;

    if let Err(e) = scheduler.add_task(TaskTemplates::pre_market_check()).await {
        println!("⚠️  添加盘前任务失败: {}", e);
    }
    if let Err(e) = scheduler
        .add_task(TaskTemplates::auction_collection())
        .await
    {
        println!("⚠️  添加竞价任务失败: {}", e);
    }
    if let Err(e) = scheduler.add_task(TaskTemplates::market_open()).await {
        println!("⚠️  添加开盘任务失败: {}", e);
    }
    if let Err(e) = scheduler.add_task(TaskTemplates::market_close()).await {
        println!("⚠️  添加收盘任务失败: {}", e);
    }
    if let Err(e) = scheduler
        .add_task(TaskTemplates::post_market_process())
        .await
    {
        println!("⚠️  添加盘后任务失败: {}", e);
    }
    if let Err(e) = scheduler.add_task(TaskTemplates::data_sync()).await {
        println!("⚠️  添加同步任务失败: {}", e);
    }

    println!("✅ 已添加预设任务");

    scheduler
        .start()
        .await
        .map_err(|e| QuantixError::Other(format!("启动调度器失败: {}", e)))?;

    println!("✅ 任务调度器已启动");
    println!("\n按 Ctrl+C 停止调度器");

    tokio::signal::ctrl_c().await?;
    println!("\n🛑 停止调度器...");
    scheduler
        .stop()
        .await
        .map_err(|e| QuantixError::Other(format!("停止调度器失败: {}", e)))?;
    println!("✅ 调度器已停止");

    Ok(())
}

async fn stop_task_scheduler() -> Result<()> {
    println!("🛑 停止任务调度器...");
    println!("💡 提示: 在运行中的调度器按 Ctrl+C 停止");
    Ok(())
}

async fn show_task_status() -> Result<()> {
    println!("📊 Foundation P0 任务状态:");
    println!();
    println!("  状态: 仅支持当前进程内调度器");
    println!("  持久化: 暂不支持");
    println!("  后台守护: 暂不支持");
    println!();
    println!("💡 使用 `quantix task start` 以前台模式运行预置任务模板");

    Ok(())
}

pub async fn run_analyze_command(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::Indicators { code, indicators } => {
            calculate_indicators(code, indicators).await?;
        }
        AnalyzeCommands::Backtest { id } => {
            show_backtest_report(id)?;
        }
        AnalyzeCommands::CandlePattern {
            candle,
            code,
            tdx_root,
            market,
            day_file,
            start,
            end,
            r#type,
            limit,
            reference,
            previous_close,
        } => {
            analyze_candle_patterns(
                candle,
                code,
                tdx_root,
                market,
                day_file,
                start,
                end,
                r#type,
                limit,
                reference,
                previous_close,
            )
            .await?;
        }
        AnalyzeCommands::Screener(cmd) => {
            run_screener_command(cmd).await?;
        }
    }
    Ok(())
}

pub async fn run_status(health: bool) -> Result<()> {
    if health {
        println!("🏥 检查数据库连接...");

        match create_clickhouse_client().await {
            Ok(_) => println!("  ✅ ClickHouse: 连接正常"),
            Err(e) => println!("  ❌ ClickHouse: 连接失败 - {}", e),
        }
    } else {
        println!("📊 Quantix CLI 状态:");
        println!();
        println!("  版本: 0.1.0");
        println!("  模式: 共享数据库模式");
        println!("  Phase: 14/14 (CLI 命令实现)");
        println!();
        println!("  📦 已完成模块:");
        println!("    ✅ 数据采集基础 (Phase 1)");
        println!("    ✅ 竞价分析 (Phase 2)");
        println!("    ✅ K线管理 (Phase 3)");
        println!("    ✅ 回测引擎 (Phase 4)");
        println!("    ✅ 任务调度 (Phase 5)");
        println!("    ✅ TDX解析 (Phase 6)");
        println!("    ✅ GBBQ存储 (Phase 7)");
        println!("    ✅ 多周期查询 (Phase 8)");
        println!("    ✅ 东方财富 (Phase 9)");
        println!("    ✅ 批量优化 (Phase 10)");
        println!("    ✅ WebSocket (Phase 11)");
        println!("    ✅ 技术指标 (Phase 12)");
        println!("    ✅ Polars适配 (Phase 13)");
        println!("    ✅ CLI命令 (Phase 14)");
        println!();
    }

    Ok(())
}

async fn run_data_sync_menu() -> Result<()> {
    let items = vec![
        "查询K线数据     — 从ClickHouse查询指定股票的K线数据",
        "导出数据        — 导出K线数据为CSV或Parquet格式",
        "数据源管理      — 查看/添加/测试数据源配置",
        "返回",
    ];

    println!("\n  💡 数据同步功能需要：1) ClickHouse数据库运行中  2) 已配置数据源 (Bridge/TDX)");
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => {
            println!("  💡 请确保ClickHouse中已有K线数据 (可通过 quantix data query 验证)");
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            let limit = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("查询条数")
                .default("10".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            let limit: usize = limit.parse().unwrap_or(10);
            query_kline_data(code, None, None, "day".to_string(), limit).await?;
        }
        1 => {
            println!("  💡 支持CSV (Excel可打开) 和 Parquet (大数据分析) 两种格式");
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            let items = vec!["CSV (Excel兼容)", "Parquet (大数据分析)"];
            let fmt_sel = Select::with_theme(&ColorfulTheme::default())
                .items(&items)
                .default(0)
                .interact()
                .map_err(|e| QuantixError::Other(format!("选择失败: {}", e)))?;

            let fmt = if fmt_sel == 0 { "csv" } else { "parquet" };
            export_data(code, fmt.to_string(), "./data".to_string()).await?;
        }
        2 => {
            println!("\n  📋 数据源操作：");
            println!("    quantix data source list          查看已配置的数据源");
            println!("    quantix data source add            添加数据源");
            println!("    quantix data source test --name X  测试数据源连通性");
            println!("\n  💡 数据源配置文件位于 ../config/ 目录");
        }
        3 => {}
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

fn backtest_history_menu_command() -> crate::cli::command_types::BacktestCommands {
    crate::cli::command_types::BacktestCommands::List
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backtest_history_menu_uses_existing_report_list_command() {
        assert!(matches!(
            backtest_history_menu_command(),
            crate::cli::command_types::BacktestCommands::List
        ));
    }
}
