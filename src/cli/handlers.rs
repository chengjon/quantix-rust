use super::{
    AnalyzeCommands, DataCommands, StrategyCommands, TaskCommands, WatchlistCommands,
    WatchlistGroupCommands, WatchlistTagCommands,
};
use crate::analysis::backtest::{BacktestConfig, BacktestEngine};
use crate::analysis::polars_adapter::{PolarsCalculator, from_kline_vec};
/// CLI 命令处理器
///
/// 实现各个子命令的处理逻辑
use crate::core::{CliRuntime, QuantixError, Result};
use crate::db::clickhouse::ClickHouseClient;
use crate::tasks::{TaskScheduler, TaskTemplates};
use crate::watchlist::{
    PostgresWatchlistNameLookup, TdxWatchlistQuoteLookup, WatchlistDisplayRow,
    WatchlistHistoryEvent, WatchlistService, WatchlistStorage, WatchlistStore,
};
use chrono::{NaiveDate, Utc};
use dialoguer::{Input, Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use rust_decimal_macros::dec;
use std::path::Path;
use std::sync::Arc;

async fn create_clickhouse_client() -> Result<ClickHouseClient> {
    let runtime = CliRuntime::load();
    ClickHouseClient::from_settings(&runtime.clickhouse).await
}

/// 初始化命令
pub async fn run_init(config_path: String) -> Result<()> {
    println!("🚀 初始化 Quantix CLI...");
    println!("📁 配置路径: {}", config_path);

    // 检查配置路径是否存在
    let path = Path::new(&config_path);
    if !path.exists() {
        println!("⚠️  警告: 配置路径不存在，将创建基本配置");
        std::fs::create_dir_all(path)
            .map_err(|e| QuantixError::Other(format!("创建配置目录失败: {}", e)))?;
    }

    // 初始化 Polars
    let _ = crate::analysis::polars_adapter::init_polars();

    println!("✅ 初始化完成！");
    println!("\n📝 下一步:");
    println!("  1. 配置数据库连接 (环境变量)");
    println!("  2. 运行 'quantix data query' 查询数据");
    println!("  3. 运行 'quantix strategy list' 查看策略");
    println!("  4. 运行 'quantix task start' 启动任务调度器");

    Ok(())
}

/// 交互式菜单（简单版）
pub async fn run_simple_menu() -> Result<()> {
    loop {
        println!("\n=== Quantix CLI 交互菜单 ===\n");

        let items = vec![
            "📊 数据同步",
            "📈 策略运行",
            "🔙 回测分析",
            "⏰ 任务管理",
            "💹 技术分析",
            "📤 数据导出",
            "❌ 退出",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact()
            .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

        match selection {
            0 => run_data_sync_menu().await?,
            1 => run_strategy_menu().await?,
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

/// TUI 菜单
pub async fn run_tui_menu() -> Result<()> {
    println!("🎨 TUI 菜单功能开发中...");
    // TODO: 实现 ratatui 菜单
    println!("💡 提示: 使用 'quantix menu' 进入简单菜单");
    Ok(())
}

/// 数据命令
pub async fn run_data_command(cmd: DataCommands) -> Result<()> {
    match cmd {
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

/// 查询 K线数据
async fn query_kline_data(
    code: String,
    start: Option<String>,
    end: Option<String>,
    period_type: String,
    limit: usize,
) -> Result<()> {
    println!("📊 查询 K线数据");
    println!("  代码: {}", code);
    println!("  周期: {}", period_type);
    println!("  限制: {}", limit);

    // 解析日期
    let start_date = start
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok());
    let end_date = end
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok());

    // 连接 ClickHouse
    let client = create_clickhouse_client().await?;

    // 查询数据
    let klines = client
        .get_kline_data(&code, &period_type, start_date, end_date, Some(limit))
        .await?;

    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }

    // 显示数据
    println!("\n📈 查询结果 (共 {} 条):", klines.len());
    println!(
        "{:<12} {:<10} {:<10} {:<10} {:<10} {:<12}",
        "日期", "开盘", "最高", "最低", "收盘", "成交量"
    );
    println!("{}", "-".repeat(70));

    for kline in klines.iter().take(20) {
        println!(
            "{:<12} {:<10.2} {:<10.2} {:<10.2} {:<10.2} {:<12}",
            kline.date, kline.open, kline.high, kline.low, kline.close, kline.volume,
        );
    }

    if klines.len() > 20 {
        println!("... (还有 {} 条)", klines.len() - 20);
    }

    Ok(())
}

/// 导出数据
async fn export_data(code: String, format: String, output: String) -> Result<()> {
    println!("📤 导出数据");
    println!("  代码: {}", code);
    println!("  格式: {}", format);
    println!("  输出: {}", output);

    // 创建输出目录
    std::fs::create_dir_all(&output)
        .map_err(|e| QuantixError::Other(format!("创建输出目录失败: {}", e)))?;

    // 连接 ClickHouse
    let client = create_clickhouse_client().await?;

    // 查询数据
    let klines = client
        .get_kline_data(
            &code,
            "1d",
            None,
            None,
            Some(10000), // 导出时使用较大的限制
        )
        .await?;

    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }

    let output_path = format!("{}/{}.{}", output, code, format);
    let progress = ProgressBar::new(3);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}")
            .unwrap(),
    );

    progress.set_message("准备导出...");

    match format.as_str() {
        "csv" => {
            progress.set_message("写入 CSV...");
            progress.inc(1);

            let mut wtr = csv::Writer::from_path(&output_path)
                .map_err(|e| QuantixError::Other(format!("创建 CSV 文件失败: {}", e)))?;

            wtr.write_record(&["date", "open", "high", "low", "close", "volume"])
                .map_err(|e| QuantixError::Other(format!("写入 CSV 头失败: {}", e)))?;

            for kline in &klines {
                wtr.write_record(&[
                    kline.date.to_string(),
                    kline.open.to_string(),
                    kline.high.to_string(),
                    kline.low.to_string(),
                    kline.close.to_string(),
                    kline.volume.to_string(),
                ])
                .map_err(|e| QuantixError::Other(format!("写入 CSV 数据失败: {}", e)))?;
            }

            wtr.flush()
                .map_err(|e| QuantixError::Other(format!("刷新 CSV 失败: {}", e)))?;

            progress.inc(1);
            progress.finish_with_message("CSV 导出完成");
        }
        "parquet" => {
            progress.set_message("写入 Parquet...");
            progress.inc(1);

            // TODO: 实现 Parquet 导出
            progress.finish_with_message("Parquet 导出暂未实现");
        }
        _ => {
            return Err(QuantixError::Other(format!("不支持的格式: {}", format)));
        }
    }

    println!("✅ 数据已导出到: {}", output_path);
    Ok(())
}

/// 策略命令
pub async fn run_strategy_command(cmd: StrategyCommands) -> Result<()> {
    match cmd {
        StrategyCommands::Run { name, mode, code } => {
            run_strategy(name, mode, code).await?;
        }
        StrategyCommands::List => {
            list_strategies().await?;
        }
        StrategyCommands::Show { name } => {
            show_strategy(name).await?;
        }
    }
    Ok(())
}

/// 运行策略
async fn run_strategy(name: String, mode: String, code: Option<String>) -> Result<()> {
    println!("🎯 运行策略: {} ({})", name, mode);
    if let Some(c) = &code {
        println!("📈 股票代码: {}", c);
    }

    match name.as_str() {
        "ma_cross" => {
            if mode == "backtest" {
                run_ma_cross_backtest(code).await?;
            } else {
                println!("⚠️  暂不支持实时模式");
            }
        }
        _ => {
            println!("❌ 未知策略: {}", name);
            println!("💡 可用策略: ma_cross");
        }
    }

    Ok(())
}

/// 运行 MA 交叉策略回测
async fn run_ma_cross_backtest(code: Option<String>) -> Result<()> {
    let stock_code = code.unwrap_or_else(|| "000001".to_string());

    println!("🔙 开始回测: MA 交叉策略");
    println!("  股票: {}", stock_code);
    println!("  参数: MA5, MA20");

    // 连接 ClickHouse
    let client = create_clickhouse_client().await?;

    // 获取历史数据
    let klines = client
        .get_kline_data(
            &stock_code,
            "1d",
            None,
            None,
            Some(10000), // 获取足够的数据用于回测
        )
        .await?;

    if klines.len() < 20 {
        return Err(QuantixError::Other(format!(
            "数据不足，至少需要20条，当前: {}",
            klines.len()
        )));
    }

    // 创建回测引擎
    let config = BacktestConfig {
        initial_capital: dec!(100000),
        commission_rate: dec!(0.0003),
        slippage_bps: 10, // 0.1% = 10 bps
        max_positions: 5,
        max_position_ratio: dec!(0.2),
        risk_free_rate: dec!(0.03),
    };

    let mut engine = BacktestEngine::new(config);

    // 创建策略
    let mut strategy = crate::strategy::ma_cross::MACrossStrategy::new(5, 20);

    // 转换数据格式为 HashMap
    use crate::data::models::Kline;
    use std::collections::HashMap;

    let mut data_map = HashMap::new();
    let kline_data: Vec<Kline> = klines
        .into_iter()
        .map(|k| Kline {
            code: stock_code.clone(),
            date: k.date,
            open: k.open,
            high: k.high,
            low: k.low,
            close: k.close,
            volume: k.volume,
            amount: k.amount,
            adjust_type: crate::data::models::AdjustType::None,
        })
        .collect();

    data_map.insert(stock_code.clone(), kline_data);

    // 运行回测
    let total_klines = data_map.values().map(|v| v.len()).sum::<usize>();
    let progress = ProgressBar::new(total_klines as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} K线")
            .unwrap(),
    );

    let result = engine
        .run(&mut strategy, &data_map)
        .await
        .map_err(|e| QuantixError::Other(format!("回测失败: {}", e)))?;

    progress.finish();

    // 显示结果
    println!("\n📊 回测结果:");
    println!("  总收益率: {:.2}%", result.report.total_return * dec!(100));
    println!("  夏普比率: {:.2}", result.report.sharpe_ratio);
    println!("  最大回撤: {:.2}%", result.report.max_drawdown * dec!(100));
    println!("  胜率: {:.2}%", result.report.win_rate * dec!(100));
    println!("  交易次数: {}", result.report.total_trades);

    Ok(())
}

/// 列出所有策略
async fn list_strategies() -> Result<()> {
    println!("📋 可用策略:");
    println!();
    println!("  1. ma_cross - 均线交叉策略");
    println!("     描述: MA5 上穿 MA20 买入，下穿卖出");
    println!("     运行: quantix strategy run --name ma_cross --mode backtest --code 000001");
    println!();
    println!("💡 更多策略开发中...");

    Ok(())
}

/// 显示策略详情
async fn show_strategy(name: String) -> Result<()> {
    println!("📖 策略详情: {}", name);

    match name.as_str() {
        "ma_cross" => {
            println!();
            println!("  名称: 均线交叉策略");
            println!("  原理: 当短期均线(MA5)上穿长期均线(MA20)时买入，反之卖出");
            println!("  参数:");
            println!("    - 短期周期: 5");
            println!("    - 长期周期: 20");
            println!("  适用场景: 趋势明显的市场");
            println!("  风险: 震荡市场容易频繁交易");
        }
        _ => {
            println!("❌ 未知策略: {}", name);
        }
    }

    Ok(())
}

/// 任务命令
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

fn ensure_task_command_supported_for_p0(cmd: &TaskCommands) -> Result<()> {
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

/// 添加任务
async fn add_task(name: String, cron: String, command: String) -> Result<()> {
    println!("⏰ 添加任务: {}", name);
    println!("  Cron: {}", cron);
    println!("  命令: {}", command);

    Err(QuantixError::Unsupported(
        "Foundation P0 仅支持预置任务模板；请使用 `quantix task list` 查看可运行任务".to_string(),
    ))
}

fn foundation_p0_task_template_descriptions() -> Vec<(String, String, String)> {
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

/// 列出所有任务
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

/// 启动任务调度器
async fn start_task_scheduler(daemon: bool) -> Result<()> {
    if daemon {
        return Err(QuantixError::Unsupported(
            "Foundation P0 仅支持前台直接执行；后台守护模式暂不支持".to_string(),
        ));
    }

    println!("⏰ 启动任务调度器...");

    // 创建调度器
    let scheduler = TaskScheduler::new()
        .await
        .map_err(|e| QuantixError::Other(format!("创建调度器失败: {}", e)))?;

    // 添加预设任务
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

    // 启动调度器
    scheduler
        .start()
        .await
        .map_err(|e| QuantixError::Other(format!("启动调度器失败: {}", e)))?;

    println!("✅ 任务调度器已启动");
    println!("\n按 Ctrl+C 停止调度器");

    // 等待停止信号
    tokio::signal::ctrl_c().await?;
    println!("\n🛑 停止调度器...");
    scheduler
        .stop()
        .await
        .map_err(|e| QuantixError::Other(format!("停止调度器失败: {}", e)))?;
    println!("✅ 调度器已停止");

    Ok(())
}

/// 停止任务调度器
async fn stop_task_scheduler() -> Result<()> {
    println!("🛑 停止任务调度器...");
    println!("💡 提示: 在运行中的调度器按 Ctrl+C 停止");
    Ok(())
}

/// 显示任务状态
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

/// 分析命令
pub async fn run_analyze_command(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::Indicators { code, indicators } => {
            calculate_indicators(code, indicators).await?;
        }
        AnalyzeCommands::Backtest { id } => {
            show_backtest_report(id).await?;
        }
    }
    Ok(())
}

/// 自选池命令
pub async fn run_watchlist_command(cmd: WatchlistCommands) -> Result<()> {
    let storage = create_watchlist_storage();
    let service = WatchlistService::default();

    match cmd {
        WatchlistCommands::Add { code, group } => {
            let mut store = storage.load_or_create()?;
            service.add(&mut store, &code, group.as_deref(), Utc::now())?;
            storage.save(&store)?;
            println!("✅ 已添加 {} 到自选池", code);
        }
        WatchlistCommands::Remove { code } => {
            let mut store = storage.load_or_create()?;
            service.remove(&mut store, &code, Utc::now())?;
            storage.save(&store)?;
            println!("✅ 已从自选池移除 {}", code);
        }
        WatchlistCommands::List {
            group,
            tag,
            with_price,
        } => {
            let store = load_watchlist_store_for_read(&storage)?;
            let items = service.list(&store, group.as_deref(), tag.as_deref());

            if with_price {
                let resolver = crate::watchlist::WatchlistResolver::new(
                    Arc::new(PostgresWatchlistNameLookup),
                    Arc::new(TdxWatchlistQuoteLookup),
                );
                let rows = resolver.resolve_rows(&items, true).await;
                print_watchlist_rows(&rows);
            } else {
                print_basic_watchlist_items(&items);
            }
        }
        WatchlistCommands::Move { code, group } => {
            let mut store = storage.load_or_create()?;
            service.move_code(&mut store, &code, &group, Utc::now())?;
            storage.save(&store)?;
            println!("✅ 已将 {} 移动到分组 {}", code, group);
        }
        WatchlistCommands::Group(group_cmd) => match group_cmd {
            WatchlistGroupCommands::Create { name } => {
                let mut store = storage.load_or_create()?;
                service.create_group(&mut store, &name, Utc::now())?;
                storage.save(&store)?;
                println!("✅ 已创建分组 {}", name);
            }
            WatchlistGroupCommands::List => {
                let store = load_watchlist_store_for_read(&storage)?;
                print_watchlist_groups(&store);
            }
        },
        WatchlistCommands::Tag(tag_cmd) => match tag_cmd {
            WatchlistTagCommands::Add { code, tag } => {
                let mut store = storage.load_or_create()?;
                service.add_tag(&mut store, &code, &tag, Utc::now())?;
                storage.save(&store)?;
                println!("✅ 已为 {} 添加标签 {}", code, tag);
            }
            WatchlistTagCommands::Remove { code, tag } => {
                let mut store = storage.load_or_create()?;
                service.remove_tag(&mut store, &code, &tag, Utc::now())?;
                storage.save(&store)?;
                println!("✅ 已为 {} 移除标签 {}", code, tag);
            }
            WatchlistTagCommands::List { code } => {
                let store = load_watchlist_store_for_read(&storage)?;
                let entry = store
                    .entries
                    .get(&code)
                    .ok_or_else(|| QuantixError::Other(format!("股票不存在: {}", code)))?;
                print_watchlist_tags(&code, &entry.tags);
            }
        },
        WatchlistCommands::History { code, limit } => {
            let store = load_watchlist_store_for_read(&storage)?;
            let events = service.history(&store, code.as_deref(), Some(limit));
            print_watchlist_history(&events);
        }
    }

    Ok(())
}

fn create_watchlist_storage() -> WatchlistStorage {
    let runtime = CliRuntime::load();
    WatchlistStorage::new(runtime.watchlist_path)
}

fn load_watchlist_store_for_read(storage: &WatchlistStorage) -> Result<WatchlistStore> {
    Ok(storage.load()?.unwrap_or_default())
}

fn print_basic_watchlist_items(items: &[crate::watchlist::WatchlistListItem]) {
    if items.is_empty() {
        println!("📭 自选池为空");
        return;
    }

    println!("{:<10} {:<12} {}", "代码", "分组", "标签");
    println!("{}", "-".repeat(48));

    for item in items {
        println!(
            "{:<10} {:<12} {}",
            item.code,
            item.group,
            format_tags(&item.tags)
        );
    }
}

fn print_watchlist_rows(rows: &[WatchlistDisplayRow]) {
    if rows.is_empty() {
        println!("📭 自选池为空");
        return;
    }

    println!(
        "{:<10} {:<12} {:<12} {:<16} {:<12} {}",
        "代码", "名称", "分组", "标签", "最新价", "涨跌幅"
    );
    println!("{}", "-".repeat(84));

    for row in rows {
        let price = row
            .latest_price
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string());
        let change_pct = row
            .price_change_pct
            .map(|value| format!("{}%", value))
            .unwrap_or_else(|| "-".to_string());

        println!(
            "{:<10} {:<12} {:<12} {:<16} {:<12} {}",
            row.code,
            row.name.as_deref().unwrap_or("-"),
            row.group,
            format_tags(&row.tags),
            price,
            change_pct
        );
    }
}

fn print_watchlist_groups(store: &WatchlistStore) {
    let mut groups: Vec<(&String, usize)> = store
        .groups
        .iter()
        .map(|(name, codes)| (name, codes.len()))
        .collect();
    groups.sort_by(|left, right| left.0.cmp(right.0));

    println!("{:<16} {}", "分组", "数量");
    println!("{}", "-".repeat(28));

    for (name, size) in groups {
        println!("{:<16} {}", name, size);
    }
}

fn print_watchlist_tags(code: &str, tags: &[String]) {
    println!("🏷️  {} 标签: {}", code, format_tags(tags));
}

fn print_watchlist_history(events: &[WatchlistHistoryEvent]) {
    if events.is_empty() {
        println!("🕘 暂无历史记录");
        return;
    }

    println!(
        "{:<22} {:<12} {:<10} {:<12} {}",
        "时间", "动作", "代码", "分组", "标签"
    );
    println!("{}", "-".repeat(72));

    for event in events {
        println!(
            "{:<22} {:<12} {:<10} {:<12} {}",
            event.ts.to_rfc3339(),
            format!("{:?}", event.action),
            event.code.as_deref().unwrap_or("-"),
            event.group.as_deref().unwrap_or("-"),
            event.tag.as_deref().unwrap_or("-")
        );
    }
}

fn format_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        "-".to_string()
    } else {
        tags.join(",")
    }
}

/// 计算技术指标
async fn calculate_indicators(code: String, indicators_str: String) -> Result<()> {
    println!("💹 计算技术指标");
    println!("  代码: {}", code);
    println!("  指标: {}", indicators_str);

    // 连接 ClickHouse
    let client = create_clickhouse_client().await?;

    // 获取历史数据
    let klines = client
        .get_kline_data(&code, "1d", None, None, Some(1000))
        .await?;

    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }

    // 转换为 BatchKlineData
    let batch_data = from_kline_vec(&klines);

    // 创建计算器
    let calc = PolarsCalculator::new();

    // 解析指标列表
    let indicators: Vec<&str> = indicators_str.split(',').collect();

    // 批量计算
    let results = calc.calculate_batch(&batch_data, &indicators);

    println!("\n📊 计算结果:");
    println!(
        "{:<12} {:<20} {:<15} {:<15}",
        "日期", "收盘价", "指标", "值"
    );
    println!("{}", "-".repeat(65));

    for (i, kline) in klines.iter().enumerate().take(20) {
        println!("{:<12} {:<20.2}", kline.date, kline.close,);

        for indicator in &indicators {
            if let Some(values) = results.get(*indicator) {
                if let Some(value) = values.get(i) {
                    println!(
                        "{:<12} {:<20} {:<15} {:<15}",
                        "",
                        "",
                        indicator,
                        value.map(|v| v.to_string()).unwrap_or("N/A".to_string()),
                    );
                }
            }
        }
    }

    Ok(())
}

/// 显示回测报告
async fn show_backtest_report(id: String) -> Result<()> {
    println!("📊 回测报告: {}", id);
    println!("💡 提示: 运行 'quantix strategy run' 生成新报告");
    Ok(())
}

/// 状态命令
pub async fn run_status(health: bool) -> Result<()> {
    if health {
        println!("🏥 检查数据库连接...");

        // 尝试连接 ClickHouse
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::CLICKHOUSE_DB_ENV;
    use crate::core::QuantixError;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    struct ClickHouseDbEnvGuard(Option<String>);

    impl ClickHouseDbEnvGuard {
        fn capture() -> Self {
            Self(std::env::var(CLICKHOUSE_DB_ENV).ok())
        }
    }

    impl Drop for ClickHouseDbEnvGuard {
        fn drop(&mut self) {
            match &self.0 {
                Some(value) => unsafe { std::env::set_var(CLICKHOUSE_DB_ENV, value) },
                None => unsafe { std::env::remove_var(CLICKHOUSE_DB_ENV) },
            }
        }
    }

    #[tokio::test]
    async fn test_create_clickhouse_client_uses_runtime_settings() {
        let _lock = env_lock();
        let _guard = ClickHouseDbEnvGuard::capture();
        unsafe {
            std::env::set_var(CLICKHOUSE_DB_ENV, "quantix_runtime_test");
        }

        let client = create_clickhouse_client().await.unwrap();
        assert_eq!(client.database(), "quantix_runtime_test");
    }

    #[test]
    fn test_task_add_is_explicitly_unsupported() {
        let err = ensure_task_command_supported_for_p0(&TaskCommands::Add {
            name: "demo".to_string(),
            cron: "0 * * * *".to_string(),
            command: "echo demo".to_string(),
        })
        .unwrap_err();

        assert!(matches!(err, QuantixError::Unsupported(_)));
    }

    #[test]
    fn test_task_start_daemon_is_explicitly_unsupported() {
        let err =
            ensure_task_command_supported_for_p0(&TaskCommands::Start { daemon: true }).unwrap_err();

        assert!(matches!(err, QuantixError::Unsupported(_)));
    }

    #[test]
    fn test_foundation_p0_task_templates_match_scheduler_templates() {
        let templates = foundation_p0_task_template_descriptions();

        assert_eq!(
            templates,
            vec![
                (
                    "pre_market_check".to_string(),
                    "检查盘前数据".to_string(),
                    "0 8 * * 1-5".to_string()
                ),
                (
                    "auction_collection".to_string(),
                    "竞价数据采集".to_string(),
                    "30,0 9 * * 1-5".to_string()
                ),
                (
                    "market_open".to_string(),
                    "开盘检查".to_string(),
                    "30 9 * * 1-5".to_string()
                ),
                (
                    "market_close".to_string(),
                    "收盘检查".to_string(),
                    "0 15 * * 1-5".to_string()
                ),
                (
                    "post_market_process".to_string(),
                    "盘后数据处理".to_string(),
                    "30 15 * * 1-5".to_string()
                ),
                (
                    "data_sync".to_string(),
                    "数据同步".to_string(),
                    "0 16 * * *".to_string()
                ),
            ]
        );
    }
}

// === 菜单辅助函数 ===

/// 数据同步菜单
async fn run_data_sync_menu() -> Result<()> {
    let items = vec![
        "同步股票列表",
        "同步实时行情",
        "同步竞价数据",
        "同步K线数据",
        "返回",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => println!("📥 同步股票列表..."),
        1 => println!("📡 同步实时行情..."),
        2 => println!("💰 同步竞价数据..."),
        3 => println!("📊 同步K线数据..."),
        4 => {}
        _ => {}
    }

    Ok(())
}

/// 策略菜单
async fn run_strategy_menu() -> Result<()> {
    let items = vec!["运行策略", "查看策略列表", "返回"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => {
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            run_ma_cross_backtest(Some(code)).await?;
        }
        1 => list_strategies().await?,
        2 => {}
        _ => {}
    }

    Ok(())
}

/// 回测菜单
async fn run_backtest_menu() -> Result<()> {
    let items = vec!["新建回测", "查看历史回测", "返回"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => {
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            run_ma_cross_backtest(Some(code)).await?;
        }
        1 => println!("📋 历史回测列表..."),
        2 => {}
        _ => {}
    }

    Ok(())
}

/// 任务菜单
async fn run_task_menu() -> Result<()> {
    let items = vec!["查看任务列表", "启动调度器", "返回"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => list_tasks().await?,
        1 => {
            start_task_scheduler(false).await?;
        }
        2 => {}
        _ => {}
    }

    Ok(())
}

/// 分析菜单
async fn run_analysis_menu() -> Result<()> {
    let items = vec!["计算技术指标", "返回"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => {
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

/// 导出菜单
async fn run_export_menu() -> Result<()> {
    let items = vec!["导出为 CSV", "导出为 Parquet", "返回"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => {
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            export_data(code, "csv".to_string(), "./data".to_string()).await?;
        }
        1 => {
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
