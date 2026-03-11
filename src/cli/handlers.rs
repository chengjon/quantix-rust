use super::{
    AnalyzeCommands, DataCommands, MarketCommands, MonitorAlertCommands, MonitorCommands,
    ScreenerCommands, StopCommands, StrategyCommands, TaskCommands, WatchlistCommands,
    WatchlistGroupCommands, WatchlistTagCommands,
};
use crate::analysis::backtest::{BacktestConfig, BacktestEngine};
use crate::analysis::polars_adapter::{PolarsCalculator, from_kline_vec};
/// CLI 命令处理器
///
/// 实现各个子命令的处理逻辑
use crate::core::{CliRuntime, QuantixError, Result};
use crate::db::clickhouse::ClickHouseClient;
use crate::market::{
    BoardRankRow, BoardSortBy, BoardType, LeaderFilter, LeaderRow, MarketDataReader,
    MarketOverview, MarketSentimentSnapshot, MarketService, NorthFlowSnapshot,
};
use crate::monitor::storage::SqliteMonitorAlertStore;
use crate::monitor::{
    MonitorAlertStore, MonitorQuoteReader, MonitorQuoteRow, MonitorService, MonitorWatchlistReader,
    MonitorWatchlistSnapshot, PriceAlert, PriceAlertKind,
};
use crate::screener::{
    DailyKlineLoader, PresetInvocation, RuleMatchDetail, ScreenRow, ScreenRunOptions, ScreenSortBy,
    ScreenUniverse, ScreenerService, parse_preset_invocation,
};
use crate::stop::{SqliteStopRuleStore, StopRule, StopRuleStore, StopService};
use crate::tasks::{TaskScheduler, TaskTemplates};
use crate::watchlist::{
    PostgresWatchlistNameLookup, TdxWatchlistQuoteLookup, WatchlistDisplayRow,
    WatchlistHistoryEvent, WatchlistListItem, WatchlistQuoteLookup, WatchlistService,
    WatchlistStorage, WatchlistStore,
};
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use dialoguer::{Input, Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use rust_decimal::prelude::ToPrimitive;
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
        AnalyzeCommands::Screener(cmd) => {
            run_screener_command(cmd).await?;
        }
    }
    Ok(())
}

pub async fn run_monitor_command(cmd: MonitorCommands) -> Result<()> {
    let watchlist_reader = ConfiguredMonitorWatchlistReader::new(create_watchlist_storage());
    let quote_reader = TdxMonitorQuoteReader;
    let alert_store = create_monitor_alert_store().await?;
    let service = MonitorService::new(watchlist_reader, quote_reader, alert_store.clone());
    let output = execute_monitor_command_with_service(cmd, &service).await?;

    if let MonitorCommandOutput::Watchlist(snapshot) = &output {
        persist_triggered_monitor_alerts(&alert_store, snapshot, Utc::now()).await?;
    }

    print_monitor_command_output(&output);
    Ok(())
}

pub async fn run_stop_command(cmd: StopCommands) -> Result<()> {
    let watchlist_storage = create_watchlist_storage();
    let service = StopService::new(create_stop_rule_store().await?);
    let output = execute_stop_command_with_service(cmd, &service, &watchlist_storage).await?;
    print_stop_command_output(&output);
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum MonitorCommandOutput {
    Watchlist(MonitorWatchlistSnapshot),
    AlertAdded(PriceAlert),
    AlertList(Vec<PriceAlert>),
    AlertRemoved { id: u64, removed: bool },
}

#[derive(Debug, Clone, PartialEq)]
enum StopCommandOutput {
    RuleSet(StopRule),
    RuleList(Vec<StopRule>),
    RuleRemoved { code: String, removed: bool },
}

pub async fn run_market_command(cmd: MarketCommands) -> Result<()> {
    let output = execute_market_command_with_reader(cmd, create_clickhouse_client().await?).await?;

    match output {
        MarketCommandOutput::BoardRows(rows) => print_market_board_rows(&rows),
        MarketCommandOutput::NorthFlow(snapshot) => print_north_flow_snapshot(snapshot.as_ref()),
        MarketCommandOutput::Sentiment(snapshot) => {
            print_market_sentiment_snapshot(snapshot.as_ref())
        }
        MarketCommandOutput::Leaders(rows) => print_market_leader_rows(&rows),
        MarketCommandOutput::Overview(overview) => print_market_overview(&overview),
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum MarketCommandOutput {
    BoardRows(Vec<BoardRankRow>),
    NorthFlow(Option<NorthFlowSnapshot>),
    Sentiment(Option<MarketSentimentSnapshot>),
    Leaders(Vec<LeaderRow>),
    Overview(MarketOverview),
}

#[derive(Debug, Clone)]
struct ConfiguredMonitorWatchlistReader {
    storage: WatchlistStorage,
    service: WatchlistService,
}

impl ConfiguredMonitorWatchlistReader {
    fn new(storage: WatchlistStorage) -> Self {
        Self {
            storage,
            service: WatchlistService::default(),
        }
    }
}

#[async_trait]
impl MonitorWatchlistReader for ConfiguredMonitorWatchlistReader {
    async fn list_items(&self) -> Result<Vec<WatchlistListItem>> {
        let store = load_watchlist_store_for_read(&self.storage)?;
        Ok(self.service.list(&store, None, None))
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct TdxMonitorQuoteReader;

#[async_trait]
impl MonitorQuoteReader for TdxMonitorQuoteReader {
    async fn load_quotes(&self, codes: &[String]) -> Result<Vec<MonitorQuoteRow>> {
        let quote_map = TdxWatchlistQuoteLookup
            .lookup_quotes(codes)
            .await
            .unwrap_or_default();

        Ok(codes
            .iter()
            .filter_map(|code| {
                let snapshot = quote_map.get(code)?;
                Some(MonitorQuoteRow {
                    code: code.clone(),
                    group: String::new(),
                    tags: Vec::new(),
                    last_price: snapshot.latest_price.to_f64(),
                    change_pct: snapshot.price_change_pct.and_then(|value| value.to_f64()),
                    quote_time: None,
                    note: None,
                })
            })
            .collect())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct MonitorAlertAddRequest {
    code: String,
    kind: PriceAlertKind,
    target_price: f64,
}

async fn execute_monitor_command_with_service<RW, RQ, RS>(
    cmd: MonitorCommands,
    service: &MonitorService<RW, RQ, RS>,
) -> Result<MonitorCommandOutput>
where
    RW: MonitorWatchlistReader,
    RQ: MonitorQuoteReader,
    RS: MonitorAlertStore,
{
    match cmd {
        MonitorCommands::Watchlist { once } => {
            validate_monitor_watchlist_command(once)?;
            Ok(MonitorCommandOutput::Watchlist(
                service.load_watchlist_snapshot().await?,
            ))
        }
        MonitorCommands::Alert(alert_cmd) => match alert_cmd {
            MonitorAlertCommands::Add { code, above, below } => {
                let request = build_monitor_alert_request(code, above, below)?;
                let alert = service
                    .add_alert(
                        &request.code,
                        request.kind,
                        request.target_price,
                        Utc::now(),
                    )
                    .await?;
                Ok(MonitorCommandOutput::AlertAdded(alert))
            }
            MonitorAlertCommands::List => Ok(MonitorCommandOutput::AlertList(
                service.list_alerts().await?,
            )),
            MonitorAlertCommands::Remove { id } => {
                let removed = service.remove_alert(monitor_alert_id_to_i64(id)?).await?;
                Ok(MonitorCommandOutput::AlertRemoved { id, removed })
            }
        },
    }
}

async fn execute_stop_command_with_service<RS>(
    cmd: StopCommands,
    service: &StopService<RS>,
    watchlist_storage: &WatchlistStorage,
) -> Result<StopCommandOutput>
where
    RS: StopRuleStore,
{
    match cmd {
        StopCommands::Set {
            code,
            loss,
            profit,
            trailing,
        } => {
            ensure_watchlist_contains_code(watchlist_storage, &code)?;
            let rule = service
                .set_rule(&code, loss, profit, trailing, Utc::now())
                .await?;
            Ok(StopCommandOutput::RuleSet(rule))
        }
        StopCommands::List => Ok(StopCommandOutput::RuleList(service.list_rules().await?)),
        StopCommands::Remove { code } => {
            let removed = service.remove_rule(&code).await?;
            Ok(StopCommandOutput::RuleRemoved { code, removed })
        }
    }
}

async fn execute_market_command_with_reader<R>(
    cmd: MarketCommands,
    reader: R,
) -> Result<MarketCommandOutput>
where
    R: MarketDataReader,
{
    let service = MarketService::new(reader);

    match cmd {
        MarketCommands::Sector { top, date, sort_by } => {
            let rows = service
                .get_board_rankings(
                    BoardType::Sector,
                    parse_market_date(date.as_deref())?,
                    top,
                    parse_board_sort_by(sort_by.as_deref())?,
                )
                .await?;
            Ok(MarketCommandOutput::BoardRows(rows))
        }
        MarketCommands::Concept { top, date, sort_by } => {
            let rows = service
                .get_board_rankings(
                    BoardType::Concept,
                    parse_market_date(date.as_deref())?,
                    top,
                    parse_board_sort_by(sort_by.as_deref())?,
                )
                .await?;
            Ok(MarketCommandOutput::BoardRows(rows))
        }
        MarketCommands::North { date } => Ok(MarketCommandOutput::NorthFlow(
            service
                .get_north_flow(parse_market_date(date.as_deref())?)
                .await?,
        )),
        MarketCommands::Sentiment { date } => Ok(MarketCommandOutput::Sentiment(
            service
                .get_market_sentiment(parse_market_date(date.as_deref())?)
                .await?,
        )),
        MarketCommands::Leader {
            sector,
            concept,
            all,
            limit,
            date,
        } => {
            let filter = build_leader_filter(sector, concept, all)?;
            let rows = service
                .get_leaders(filter, limit, parse_market_date(date.as_deref())?)
                .await?;
            Ok(MarketCommandOutput::Leaders(rows))
        }
        MarketCommands::Overview { top, date } => Ok(MarketCommandOutput::Overview(
            service
                .get_overview(parse_market_date(date.as_deref())?, top)
                .await?,
        )),
    }
}

fn validate_monitor_watchlist_command(once: bool) -> Result<()> {
    if once {
        Ok(())
    } else {
        Err(QuantixError::Other(
            "monitor watchlist 当前仅支持 --once".to_string(),
        ))
    }
}

fn build_monitor_alert_request(
    code: String,
    above: Option<f64>,
    below: Option<f64>,
) -> Result<MonitorAlertAddRequest> {
    match (above, below) {
        (Some(target_price), None) => Ok(MonitorAlertAddRequest {
            code,
            kind: PriceAlertKind::Above,
            target_price,
        }),
        (None, Some(target_price)) => Ok(MonitorAlertAddRequest {
            code,
            kind: PriceAlertKind::Below,
            target_price,
        }),
        _ => Err(QuantixError::Other(
            "monitor alert add 必须且只能指定 --above 或 --below 之一".to_string(),
        )),
    }
}

fn monitor_alert_id_to_i64(id: u64) -> Result<i64> {
    i64::try_from(id).map_err(|_| QuantixError::Other(format!("告警 ID 超出支持范围: {}", id)))
}

async fn create_monitor_alert_store() -> Result<SqliteMonitorAlertStore> {
    let runtime = CliRuntime::load();
    SqliteMonitorAlertStore::new(runtime.monitor_db_path).await
}

async fn create_stop_rule_store() -> Result<SqliteStopRuleStore> {
    let runtime = CliRuntime::load();
    SqliteStopRuleStore::new(runtime.monitor_db_path).await
}

async fn persist_triggered_monitor_alerts<RS>(
    store: &RS,
    snapshot: &MonitorWatchlistSnapshot,
    observed_at: chrono::DateTime<Utc>,
) -> Result<()>
where
    RS: MonitorAlertStore,
{
    for alert in &snapshot.triggered_alerts {
        let triggered_at = alert.triggered_at.unwrap_or(observed_at);
        store.mark_triggered(alert.alert_id, triggered_at).await?;
    }
    Ok(())
}

fn ensure_watchlist_contains_code(storage: &WatchlistStorage, code: &str) -> Result<()> {
    let store = load_watchlist_store_for_read(storage)?;
    if store.entries.contains_key(code) {
        Ok(())
    } else {
        Err(QuantixError::Other(format!("股票不在自选池: {}", code)))
    }
}

fn build_leader_filter(
    sector: Option<String>,
    concept: Option<String>,
    all: bool,
) -> Result<LeaderFilter> {
    let mut filter_count = 0usize;
    if sector.is_some() {
        filter_count += 1;
    }
    if concept.is_some() {
        filter_count += 1;
    }
    if all {
        filter_count += 1;
    }

    if filter_count != 1 {
        return Err(QuantixError::Other(
            "leader 必须且只能指定 --sector、--concept 或 --all 之一".to_string(),
        ));
    }

    match (sector, concept, all) {
        (Some(name), None, false) => Ok(LeaderFilter::Sector(name)),
        (None, Some(name), false) => Ok(LeaderFilter::Concept(name)),
        (None, None, true) => Ok(LeaderFilter::All),
        _ => Err(QuantixError::Other(
            "leader 必须且只能指定 --sector、--concept 或 --all 之一".to_string(),
        )),
    }
}

fn parse_market_date(raw: Option<&str>) -> Result<Option<NaiveDate>> {
    raw.map(|value| {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map_err(|_| QuantixError::Other(format!("无效日期格式: {}，请使用 YYYY-MM-DD", value)))
    })
    .transpose()
}

fn parse_board_sort_by(raw: Option<&str>) -> Result<BoardSortBy> {
    match raw.unwrap_or("change_pct") {
        "change" | "change_pct" => Ok(BoardSortBy::ChangePct),
        other => Err(QuantixError::Other(format!(
            "不支持的 sort_by: {}，仅支持 change 或 change_pct",
            other
        ))),
    }
}

fn print_market_board_rows(rows: &[BoardRankRow]) {
    if rows.is_empty() {
        println!("📭 没有可展示的板块数据");
        return;
    }

    println!("{:<8} {:<12} {:<16} {}", "排名", "代码", "板块", "涨跌幅");
    println!("{}", "-".repeat(56));

    for row in rows {
        println!(
            "{:<8} {:<12} {:<16} {:.2}%",
            row.rank, row.board_code, row.board_name, row.change_pct
        );
    }
}

fn print_stop_command_output(output: &StopCommandOutput) {
    match output {
        StopCommandOutput::RuleSet(rule) => {
            println!("✅ 已设置 {} 的止盈止损规则", rule.code);
        }
        StopCommandOutput::RuleList(rules) => print_stop_rules(rules),
        StopCommandOutput::RuleRemoved { code, removed } => {
            if *removed {
                println!("✅ 已移除 {} 的止盈止损规则", code);
            } else {
                println!("⚠️  未找到 {} 的止盈止损规则", code);
            }
        }
    }
}

fn print_stop_rules(rules: &[StopRule]) {
    if rules.is_empty() {
        println!("📭 暂无止盈止损规则");
        return;
    }

    println!(
        "{:<10} {:<12} {:<12} {:<10} {:<12} {}",
        "代码", "止损价", "止盈价", "追踪%", "最高价", "最近触发"
    );
    println!("{}", "-".repeat(80));

    for rule in rules {
        println!(
            "{:<10} {:<12} {:<12} {:<10} {:<12} {}",
            rule.code,
            format_optional_price(rule.stop_loss_price),
            format_optional_price(rule.take_profit_price),
            format_optional_price(rule.trailing_pct),
            format_optional_price(rule.highest_price),
            format_optional_timestamp(rule.last_triggered_at),
        );
    }
}

fn format_optional_price(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:.2}", value))
        .unwrap_or_else(|| "-".to_string())
}

fn format_optional_timestamp(value: Option<chrono::DateTime<Utc>>) -> String {
    value
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "-".to_string())
}

fn print_north_flow_snapshot(snapshot: Option<&NorthFlowSnapshot>) {
    let Some(snapshot) = snapshot else {
        println!("📭 没有可展示的北向资金数据");
        return;
    };

    println!("日期: {}", snapshot.trade_date);
    println!("沪股通: {:.2}", snapshot.sh_amount);
    println!("深股通: {:.2}", snapshot.sz_amount);
    println!("合计: {:.2}", snapshot.total_amount);
    println!("余额: {:.2}", snapshot.balance);
}

fn print_market_sentiment_snapshot(snapshot: Option<&MarketSentimentSnapshot>) {
    let Some(snapshot) = snapshot else {
        println!("📭 没有可展示的市场情绪数据");
        return;
    };

    println!("日期: {}", snapshot.trade_date);
    println!("上涨: {}", snapshot.up_count);
    println!("下跌: {}", snapshot.down_count);
    println!("涨停: {}", snapshot.limit_up_count);
    println!("跌停: {}", snapshot.limit_down_count);
    println!("封板率: {:.2}", snapshot.seal_rate);
    println!("炸板率: {:.2}", snapshot.break_rate);
    println!("连板股: {}", snapshot.consecutive_board_count);
}

fn print_market_leader_rows(rows: &[LeaderRow]) {
    if rows.is_empty() {
        println!("📭 没有可展示的龙头股数据");
        return;
    }

    println!(
        "{:<10} {:<12} {:<12} {:<12} {}",
        "代码", "名称", "行业", "概念", "涨跌幅"
    );
    println!("{}", "-".repeat(72));

    for row in rows {
        println!(
            "{:<10} {:<12} {:<12} {:<12} {:.2}%",
            row.code,
            row.name,
            row.sector_name.as_deref().unwrap_or("-"),
            row.concept_name.as_deref().unwrap_or("-"),
            row.change_pct
        );
    }
}

fn print_market_overview(overview: &MarketOverview) {
    println!("== 市场概览 ==");
    println!("行业板块: {}", overview.top_sectors.len());
    println!("概念板块: {}", overview.top_concepts.len());

    match overview.north_flow.as_ref() {
        Some(snapshot) => println!("北向资金: {:.2}", snapshot.total_amount),
        None => println!("北向资金: -"),
    }

    match overview.sentiment.as_ref() {
        Some(snapshot) => println!("涨停数: {}", snapshot.limit_up_count),
        None => println!("涨停数: -"),
    }

    if !overview.top_sectors.is_empty() {
        println!();
        println!("Top 行业:");
        print_market_board_rows(&overview.top_sectors);
    }

    if !overview.top_concepts.is_empty() {
        println!();
        println!("Top 概念:");
        print_market_board_rows(&overview.top_concepts);
    }
}

fn print_monitor_command_output(output: &MonitorCommandOutput) {
    match output {
        MonitorCommandOutput::Watchlist(snapshot) => print_monitor_watchlist_snapshot(snapshot),
        MonitorCommandOutput::AlertAdded(alert) => println!(
            "✅ 已添加价格告警 #{} {} {} {:.2}",
            alert.id,
            alert.code,
            format_monitor_alert_kind(alert.kind),
            alert.target_price
        ),
        MonitorCommandOutput::AlertList(alerts) => print_monitor_alerts(alerts),
        MonitorCommandOutput::AlertRemoved { id, removed } => {
            if *removed {
                println!("✅ 已删除价格告警 #{}", id);
            } else {
                println!("⚠️  未找到价格告警 #{}", id);
            }
        }
    }
}

fn print_monitor_watchlist_snapshot(snapshot: &MonitorWatchlistSnapshot) {
    if snapshot.rows.is_empty() {
        println!("📭 自选池为空");
        return;
    }

    println!(
        "{:<10} {:<12} {:<16} {:<10} {:<10} {}",
        "代码", "分组", "标签", "最新价", "涨跌幅", "备注"
    );
    println!("{}", "-".repeat(80));

    for row in &snapshot.rows {
        println!(
            "{:<10} {:<12} {:<16} {:<10} {:<10} {}",
            row.code,
            row.group,
            format_tags(&row.tags),
            row.last_price
                .map(|value| format!("{:.2}", value))
                .unwrap_or_else(|| "-".to_string()),
            row.change_pct
                .map(|value| format!("{:.2}%", value))
                .unwrap_or_else(|| "-".to_string()),
            row.note.as_deref().unwrap_or("-")
        );
    }

    if !snapshot.triggered_alerts.is_empty() {
        println!();
        println!("== 触发告警 ==");
        for alert in &snapshot.triggered_alerts {
            println!(
                "[#{}] {} 当前价 {:.2} {} {:.2}",
                alert.alert_id,
                alert.code,
                alert.current_price,
                format_monitor_alert_kind(alert.kind),
                alert.target_price
            );
        }
    }

    if !snapshot.warnings.is_empty() {
        println!();
        println!("== 警告 ==");
        for warning in &snapshot.warnings {
            println!("{}", warning);
        }
    }
}

fn print_monitor_alerts(alerts: &[PriceAlert]) {
    if alerts.is_empty() {
        println!("📭 暂无价格告警");
        return;
    }

    println!(
        "{:<6} {:<10} {:<8} {:<12} {}",
        "ID", "代码", "类型", "目标价", "最后触发"
    );
    println!("{}", "-".repeat(64));

    for alert in alerts {
        println!(
            "{:<6} {:<10} {:<8} {:<12} {}",
            alert.id,
            alert.code,
            format_monitor_alert_kind(alert.kind),
            format!("{:.2}", alert.target_price),
            alert
                .last_triggered_at
                .map(|value| value.to_rfc3339())
                .unwrap_or_else(|| "-".to_string())
        );
    }
}

fn format_monitor_alert_kind(kind: PriceAlertKind) -> &'static str {
    match kind {
        PriceAlertKind::Above => "above",
        PriceAlertKind::Below => "below",
    }
}

async fn run_screener_command(cmd: ScreenerCommands) -> Result<()> {
    let output = match cmd {
        ScreenerCommands::PresetList => {
            execute_screener_command_with_loader(
                ScreenerCommands::PresetList,
                NullDailyKlineLoader,
                create_watchlist_storage(),
            )
            .await?
        }
        ScreenerCommands::Run { .. } => {
            let loader = ClickHouseDailyKlineLoader::new(create_clickhouse_client().await?);
            execute_screener_command_with_loader(cmd, loader, create_watchlist_storage()).await?
        }
    };

    match output {
        ScreenerCommandOutput::PresetList(presets) => print_screener_preset_list(&presets),
        ScreenerCommandOutput::Rows(rows) => print_screener_rows(&rows),
    }

    Ok(())
}

struct ClickHouseDailyKlineLoader {
    client: ClickHouseClient,
}

impl ClickHouseDailyKlineLoader {
    fn new(client: ClickHouseClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl DailyKlineLoader for ClickHouseDailyKlineLoader {
    async fn load_daily_klines(
        &self,
        code: &str,
        lookback: usize,
    ) -> Result<Vec<crate::data::models::Kline>> {
        let mut rows = self
            .client
            .get_kline_data(code, "1d", None, None, None)
            .await?;

        if rows.len() > lookback {
            rows = rows[rows.len() - lookback..].to_vec();
        }

        Ok(rows)
    }
}

struct NullDailyKlineLoader;

#[async_trait]
impl DailyKlineLoader for NullDailyKlineLoader {
    async fn load_daily_klines(
        &self,
        _code: &str,
        _lookback: usize,
    ) -> Result<Vec<crate::data::models::Kline>> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScreenerPresetSpec {
    name: &'static str,
    params: &'static str,
    description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScreenerCommandOutput {
    PresetList(Vec<ScreenerPresetSpec>),
    Rows(Vec<ScreenRow>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScreenerRunRequest {
    universe: ScreenUniverse,
    presets: Vec<PresetInvocation>,
    options: ScreenRunOptions,
}

async fn execute_screener_command_with_loader<L>(
    cmd: ScreenerCommands,
    loader: L,
    storage: WatchlistStorage,
) -> Result<ScreenerCommandOutput>
where
    L: DailyKlineLoader,
{
    match cmd {
        ScreenerCommands::PresetList => {
            Ok(ScreenerCommandOutput::PresetList(screener_preset_specs()))
        }
        ScreenerCommands::Run {
            codes,
            watchlist,
            group,
            preset,
            limit,
            sort_by,
        } => {
            let request =
                build_screener_run_request(codes, watchlist, group, preset, limit, sort_by)?;
            let service = ScreenerService::new(loader, storage);
            let rows = service
                .run(request.universe, &request.presets, request.options)
                .await?;
            Ok(ScreenerCommandOutput::Rows(rows))
        }
    }
}

fn screener_preset_specs() -> Vec<ScreenerPresetSpec> {
    vec![
        ScreenerPresetSpec {
            name: "close_above_ma",
            params: "period=<n>",
            description: "收盘价高于均线",
        },
        ScreenerPresetSpec {
            name: "close_below_ma",
            params: "period=<n>",
            description: "收盘价低于均线",
        },
        ScreenerPresetSpec {
            name: "rsi_gte",
            params: "period=<n>,value=<x>",
            description: "RSI 大于等于阈值",
        },
        ScreenerPresetSpec {
            name: "rsi_lte",
            params: "period=<n>,value=<x>",
            description: "RSI 小于等于阈值",
        },
        ScreenerPresetSpec {
            name: "volume_ratio_gte",
            params: "window=<n>,value=<x>",
            description: "量比大于等于阈值",
        },
    ]
}

fn build_screener_run_request(
    codes: Option<String>,
    watchlist: bool,
    group: Option<String>,
    preset_specs: Vec<String>,
    limit: Option<usize>,
    sort_by: Option<String>,
) -> Result<ScreenerRunRequest> {
    let universe = match (codes, watchlist) {
        (Some(_), true) => {
            return Err(QuantixError::Other(
                "--codes 与 --watchlist 不能同时使用".to_string(),
            ));
        }
        (None, false) => {
            return Err(QuantixError::Other(
                "必须指定 --codes 或 --watchlist".to_string(),
            ));
        }
        (Some(codes), false) => {
            let codes = parse_codes_csv(&codes);
            if codes.is_empty() {
                return Err(QuantixError::Other("codes 不能为空".to_string()));
            }
            if group.is_some() {
                return Err(QuantixError::Other(
                    "--group 仅可与 --watchlist 一起使用".to_string(),
                ));
            }
            ScreenUniverse::Codes(codes)
        }
        (None, true) => ScreenUniverse::Watchlist { group },
    };

    if preset_specs.is_empty() {
        return Err(QuantixError::Other("至少需要一个 --preset".to_string()));
    }

    let presets = preset_specs
        .iter()
        .map(|spec| parse_preset_invocation(spec))
        .collect::<Result<Vec<_>>>()?;

    let sort_by = match sort_by.as_deref().unwrap_or("code") {
        "code" => ScreenSortBy::Code,
        "score" => ScreenSortBy::Score,
        other => {
            return Err(QuantixError::Other(format!(
                "不支持的 sort_by: {}，仅支持 code 或 score",
                other
            )));
        }
    };

    Ok(ScreenerRunRequest {
        universe,
        presets,
        options: ScreenRunOptions { limit, sort_by },
    })
}

fn parse_codes_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| item.to_string())
        .collect()
}

fn print_screener_preset_list(presets: &[ScreenerPresetSpec]) {
    println!("{:<20} {:<24} {}", "Preset", "参数", "说明");
    println!("{}", "-".repeat(72));

    for preset in presets {
        println!(
            "{:<20} {:<24} {}",
            preset.name, preset.params, preset.description
        );
    }
}

fn print_screener_rows(rows: &[ScreenRow]) {
    if rows.is_empty() {
        println!("📭 没有可展示的筛选结果");
        return;
    }

    println!("{:<10} {:<8} {:<12} {}", "代码", "命中", "评分", "详情");
    println!("{}", "-".repeat(96));

    for row in rows {
        println!(
            "{:<10} {:<8} {:<12} {}",
            row.code,
            if row.matched { "yes" } else { "no" },
            row.score.round_dp(4),
            row.details
                .iter()
                .map(format_screener_rule_detail)
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }
}

fn format_screener_rule_detail(detail: &RuleMatchDetail) -> String {
    let status = if detail.matched { "Y" } else { "N" };

    match (
        detail.actual_value.as_ref(),
        detail.threshold_value.as_ref(),
        detail.reason.as_deref(),
    ) {
        (_, _, Some(reason)) => format!("{}:{}({})", status, detail.preset_name, reason),
        (Some(actual), Some(threshold), None) => {
            format!(
                "{}:{} {} / {}",
                status, detail.preset_name, actual, threshold
            )
        }
        _ => format!("{}:{}", status, detail.preset_name),
    }
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
    use crate::cli::{MonitorAlertCommands, MonitorCommands, StopCommands};
    use crate::core::QuantixError;
    use crate::core::config::{
        CLICKHOUSE_DB_ENV, CLICKHOUSE_PASSWORD_ENV, CLICKHOUSE_URL_ENV, CLICKHOUSE_USER_ENV,
    };
    use crate::data::models::{AdjustType, Kline};
    use crate::market::{
        BoardRankRow, BoardSortBy, BoardType, LeaderFilter, LeaderRow, MarketDataReader,
        MarketSentimentSnapshot, NorthFlowSnapshot,
    };
    use crate::monitor::{
        MonitorAlertStore, MonitorQuoteReader, MonitorQuoteRow, MonitorService,
        MonitorWatchlistReader, PriceAlert, PriceAlertKind, TriggeredAlert,
    };
    use crate::screener::DailyKlineLoader;
    use crate::stop::{StopRule, StopRuleStore, StopService};
    use crate::watchlist::WatchlistListItem;
    use async_trait::async_trait;
    use chrono::{NaiveDate, TimeZone, Utc};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    struct ClickHouseDbEnvGuard {
        url: Option<String>,
        database: Option<String>,
        user: Option<String>,
        password: Option<String>,
    }

    impl ClickHouseDbEnvGuard {
        fn capture() -> Self {
            Self {
                url: std::env::var(CLICKHOUSE_URL_ENV).ok(),
                database: std::env::var(CLICKHOUSE_DB_ENV).ok(),
                user: std::env::var(CLICKHOUSE_USER_ENV).ok(),
                password: std::env::var(CLICKHOUSE_PASSWORD_ENV).ok(),
            }
        }
    }

    impl Drop for ClickHouseDbEnvGuard {
        fn drop(&mut self) {
            match &self.url {
                Some(value) => unsafe { std::env::set_var(CLICKHOUSE_URL_ENV, value) },
                None => unsafe { std::env::remove_var(CLICKHOUSE_URL_ENV) },
            }

            match &self.database {
                Some(value) => unsafe { std::env::set_var(CLICKHOUSE_DB_ENV, value) },
                None => unsafe { std::env::remove_var(CLICKHOUSE_DB_ENV) },
            }

            match &self.user {
                Some(value) => unsafe { std::env::set_var(CLICKHOUSE_USER_ENV, value) },
                None => unsafe { std::env::remove_var(CLICKHOUSE_USER_ENV) },
            }

            match &self.password {
                Some(value) => unsafe { std::env::set_var(CLICKHOUSE_PASSWORD_ENV, value) },
                None => unsafe { std::env::remove_var(CLICKHOUSE_PASSWORD_ENV) },
            }
        }
    }

    #[tokio::test]
    async fn test_create_clickhouse_client_uses_runtime_settings() {
        let _lock = env_lock();
        let _guard = ClickHouseDbEnvGuard::capture();
        unsafe {
            std::env::set_var(CLICKHOUSE_URL_ENV, "http://runtime-host:8123");
            std::env::set_var(CLICKHOUSE_DB_ENV, "quantix_runtime_test");
            std::env::set_var(CLICKHOUSE_USER_ENV, "handler_user");
            std::env::set_var(CLICKHOUSE_PASSWORD_ENV, "handler_password");
        }

        let client = create_clickhouse_client().await.unwrap();
        assert_eq!(client.database(), "quantix_runtime_test");
        assert_eq!(client.http_auth_for_test().0, "handler_user");
        assert_eq!(client.http_auth_for_test().1, "handler_password");
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
        let err = ensure_task_command_supported_for_p0(&TaskCommands::Start { daemon: true })
            .unwrap_err();

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

    fn make_kline(code: &str, day: u32, close: Decimal, volume: i64) -> Kline {
        Kline {
            code: code.to_string(),
            date: NaiveDate::from_ymd_opt(2024, 1, day).unwrap(),
            open: close,
            high: close + dec!(1),
            low: close - dec!(1),
            close,
            volume,
            amount: None,
            adjust_type: AdjustType::None,
        }
    }

    #[derive(Clone, Default)]
    struct FakeLoader {
        data: HashMap<String, Vec<Kline>>,
    }

    #[async_trait]
    impl DailyKlineLoader for FakeLoader {
        async fn load_daily_klines(
            &self,
            code: &str,
            lookback: usize,
        ) -> crate::core::Result<Vec<Kline>> {
            let mut rows = self.data.get(code).cloned().unwrap_or_default();
            if rows.len() > lookback {
                rows = rows[rows.len() - lookback..].to_vec();
            }
            Ok(rows)
        }
    }

    #[tokio::test]
    async fn test_execute_screener_preset_list_returns_supported_presets() {
        let output = execute_screener_command_with_loader(
            ScreenerCommands::PresetList,
            FakeLoader::default(),
            WatchlistStorage::new("/tmp/unused-screener-watchlist.json"),
        )
        .await
        .unwrap();

        match output {
            ScreenerCommandOutput::PresetList(presets) => {
                let names: Vec<&str> = presets.iter().map(|item| item.name).collect();
                assert_eq!(
                    names,
                    vec![
                        "close_above_ma",
                        "close_below_ma",
                        "rsi_gte",
                        "rsi_lte",
                        "volume_ratio_gte",
                    ]
                );
            }
            ScreenerCommandOutput::Rows(_) => panic!("expected preset list output"),
        }
    }

    #[tokio::test]
    async fn test_execute_screener_run_with_codes_returns_rows() {
        let loader = FakeLoader {
            data: HashMap::from([
                (
                    "000001".to_string(),
                    vec![
                        make_kline("000001", 1, dec!(10), 100),
                        make_kline("000001", 2, dec!(10), 100),
                        make_kline("000001", 3, dec!(10), 100),
                        make_kline("000001", 4, dec!(11), 100),
                        make_kline("000001", 5, dec!(12), 100),
                    ],
                ),
                (
                    "000002".to_string(),
                    vec![
                        make_kline("000002", 1, dec!(10), 100),
                        make_kline("000002", 2, dec!(10), 100),
                        make_kline("000002", 3, dec!(10), 100),
                        make_kline("000002", 4, dec!(12), 100),
                        make_kline("000002", 5, dec!(15), 100),
                    ],
                ),
            ]),
        };

        let output = execute_screener_command_with_loader(
            ScreenerCommands::Run {
                codes: Some("000001,000002".to_string()),
                watchlist: false,
                group: None,
                preset: vec!["close_above_ma:period=3".to_string()],
                limit: Some(1),
                sort_by: Some("score".to_string()),
            },
            loader,
            WatchlistStorage::new("/tmp/unused-screener-watchlist.json"),
        )
        .await
        .unwrap();

        match output {
            ScreenerCommandOutput::Rows(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].code, "000002");
                assert!(rows[0].matched);
            }
            ScreenerCommandOutput::PresetList(_) => panic!("expected rows output"),
        }
    }

    #[tokio::test]
    async fn test_execute_screener_run_with_watchlist_group_uses_watchlist_storage() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("watchlist.json");
        let storage = WatchlistStorage::new(&path);
        let service = WatchlistService::default();
        let mut store = storage.load_or_create().unwrap();
        service
            .create_group(&mut store, "core", Utc::now())
            .unwrap();
        service
            .add(&mut store, "000001", Some("core"), Utc::now())
            .unwrap();
        service.add(&mut store, "000002", None, Utc::now()).unwrap();
        storage.save(&store).unwrap();

        let loader = FakeLoader {
            data: HashMap::from([(
                "000001".to_string(),
                vec![
                    make_kline("000001", 1, dec!(10), 100),
                    make_kline("000001", 2, dec!(10), 100),
                    make_kline("000001", 3, dec!(10), 100),
                    make_kline("000001", 4, dec!(11), 100),
                    make_kline("000001", 5, dec!(12), 100),
                ],
            )]),
        };

        let output = execute_screener_command_with_loader(
            ScreenerCommands::Run {
                codes: None,
                watchlist: true,
                group: Some("core".to_string()),
                preset: vec!["close_above_ma:period=3".to_string()],
                limit: None,
                sort_by: None,
            },
            loader,
            storage,
        )
        .await
        .unwrap();

        match output {
            ScreenerCommandOutput::Rows(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].code, "000001");
            }
            ScreenerCommandOutput::PresetList(_) => panic!("expected rows output"),
        }
    }

    #[tokio::test]
    async fn test_execute_screener_run_rejects_invalid_preset() {
        let err = execute_screener_command_with_loader(
            ScreenerCommands::Run {
                codes: Some("000001".to_string()),
                watchlist: false,
                group: None,
                preset: vec!["unknown_rule:value=1".to_string()],
                limit: None,
                sort_by: None,
            },
            FakeLoader::default(),
            WatchlistStorage::new("/tmp/unused-screener-watchlist.json"),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, QuantixError::Other(_)));
        assert!(err.to_string().contains("未知的 preset"));
    }

    fn monitor_sample_time() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 11, 10, 30, 0).unwrap()
    }

    fn monitor_watchlist_item(code: &str, group: &str, tags: &[&str]) -> WatchlistListItem {
        WatchlistListItem {
            code: code.to_string(),
            group: group.to_string(),
            tags: tags.iter().map(|tag| tag.to_string()).collect(),
        }
    }

    fn monitor_quote_row(code: &str, last_price: f64, change_pct: f64) -> MonitorQuoteRow {
        MonitorQuoteRow {
            code: code.to_string(),
            group: String::new(),
            tags: Vec::new(),
            last_price: Some(last_price),
            change_pct: Some(change_pct),
            quote_time: Some(monitor_sample_time()),
            note: None,
        }
    }

    fn monitor_alert(id: i64, code: &str, kind: PriceAlertKind, target_price: f64) -> PriceAlert {
        PriceAlert {
            id,
            code: code.to_string(),
            kind,
            target_price,
            created_at: monitor_sample_time(),
            last_triggered_at: None,
        }
    }

    #[derive(Clone, Default)]
    struct FakeMonitorWatchlistReader {
        items: Vec<WatchlistListItem>,
    }

    #[async_trait]
    impl MonitorWatchlistReader for FakeMonitorWatchlistReader {
        async fn list_items(&self) -> Result<Vec<WatchlistListItem>> {
            Ok(self.items.clone())
        }
    }

    #[derive(Clone, Default)]
    struct FakeMonitorQuoteReader {
        rows: Vec<MonitorQuoteRow>,
    }

    #[async_trait]
    impl MonitorQuoteReader for FakeMonitorQuoteReader {
        async fn load_quotes(&self, _codes: &[String]) -> Result<Vec<MonitorQuoteRow>> {
            Ok(self.rows.clone())
        }
    }

    #[derive(Debug, Clone, Default)]
    struct FakeMonitorAlertState {
        next_id: i64,
        alerts: Vec<PriceAlert>,
        removed_ids: Vec<i64>,
    }

    #[derive(Clone, Default)]
    struct FakeMonitorAlertStore {
        state: Arc<Mutex<FakeMonitorAlertState>>,
    }

    #[derive(Debug, Clone, Default)]
    struct FakeStopRuleState {
        rules: Vec<StopRule>,
        removed_codes: Vec<String>,
    }

    #[derive(Clone, Default)]
    struct FakeStopRuleStore {
        state: Arc<Mutex<FakeStopRuleState>>,
    }

    #[async_trait]
    impl StopRuleStore for FakeStopRuleStore {
        async fn upsert_rule(&self, rule: StopRule) -> Result<StopRule> {
            let mut state = self.state.lock().unwrap();
            if let Some(existing) = state
                .rules
                .iter_mut()
                .find(|existing| existing.code == rule.code)
            {
                *existing = rule.clone();
            } else {
                state.rules.push(rule.clone());
            }
            Ok(rule)
        }

        async fn list_rules(&self) -> Result<Vec<StopRule>> {
            Ok(self.state.lock().unwrap().rules.clone())
        }

        async fn remove_rule(&self, code: &str) -> Result<bool> {
            let mut state = self.state.lock().unwrap();
            let before = state.rules.len();
            state.rules.retain(|rule| rule.code != code);
            if before != state.rules.len() {
                state.removed_codes.push(code.to_string());
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    fn stop_sample_time() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 11, 12, 0, 0).unwrap()
    }

    fn stop_rule(code: &str) -> StopRule {
        StopRule {
            code: code.to_string(),
            stop_loss_price: Some(14.5),
            take_profit_price: None,
            trailing_pct: None,
            highest_price: None,
            last_triggered_at: None,
            created_at: stop_sample_time(),
            updated_at: stop_sample_time(),
        }
    }

    fn stop_watchlist_storage(codes: &[&str]) -> (tempfile::TempDir, WatchlistStorage) {
        let dir = tempfile::tempdir().unwrap();
        let storage = WatchlistStorage::new(dir.path().join("watchlist.json"));
        let service = WatchlistService::default();
        let mut store = storage.load_or_create().unwrap();
        for code in codes {
            service.add(&mut store, code, None, Utc::now()).unwrap();
        }
        storage.save(&store).unwrap();
        (dir, storage)
    }

    #[async_trait]
    impl MonitorAlertStore for FakeMonitorAlertStore {
        async fn add_alert(
            &self,
            code: &str,
            kind: PriceAlertKind,
            target_price: f64,
            now: chrono::DateTime<Utc>,
        ) -> Result<PriceAlert> {
            let mut state = self.state.lock().unwrap();
            state.next_id += 1;
            let alert = PriceAlert {
                id: state.next_id,
                code: code.to_string(),
                kind,
                target_price,
                created_at: now,
                last_triggered_at: None,
            };
            state.alerts.push(alert.clone());
            Ok(alert)
        }

        async fn list_alerts(&self) -> Result<Vec<PriceAlert>> {
            Ok(self.state.lock().unwrap().alerts.clone())
        }

        async fn remove_alert(&self, id: i64) -> Result<bool> {
            let mut state = self.state.lock().unwrap();
            let before = state.alerts.len();
            state.alerts.retain(|alert| alert.id != id);
            if before != state.alerts.len() {
                state.removed_ids.push(id);
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn mark_triggered(
            &self,
            id: i64,
            triggered_at: chrono::DateTime<Utc>,
        ) -> Result<bool> {
            let mut state = self.state.lock().unwrap();
            let Some(alert) = state.alerts.iter_mut().find(|alert| alert.id == id) else {
                return Ok(false);
            };
            alert.last_triggered_at = Some(triggered_at);
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_execute_stop_set_loss_succeeds() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let store = FakeStopRuleStore::default();
        let service = StopService::new(store.clone());

        let output = execute_stop_command_with_service(
            StopCommands::Set {
                code: "000001".to_string(),
                loss: Some(14.5),
                profit: None,
                trailing: None,
            },
            &service,
            &storage,
        )
        .await
        .unwrap();

        match output {
            StopCommandOutput::RuleSet(rule) => {
                assert_eq!(rule.code, "000001");
                assert_eq!(rule.stop_loss_price, Some(14.5));
                assert_eq!(rule.take_profit_price, None);
                assert_eq!(rule.trailing_pct, None);
            }
            other => panic!("unexpected output: {:?}", other),
        }

        assert_eq!(store.state.lock().unwrap().rules.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_stop_set_profit_succeeds() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let service = StopService::new(FakeStopRuleStore::default());

        let output = execute_stop_command_with_service(
            StopCommands::Set {
                code: "000001".to_string(),
                loss: None,
                profit: Some(18.0),
                trailing: None,
            },
            &service,
            &storage,
        )
        .await
        .unwrap();

        match output {
            StopCommandOutput::RuleSet(rule) => {
                assert_eq!(rule.take_profit_price, Some(18.0));
                assert_eq!(rule.stop_loss_price, None);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_stop_set_trailing_succeeds() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let service = StopService::new(FakeStopRuleStore::default());

        let output = execute_stop_command_with_service(
            StopCommands::Set {
                code: "000001".to_string(),
                loss: None,
                profit: Some(18.0),
                trailing: Some(5.0),
            },
            &service,
            &storage,
        )
        .await
        .unwrap();

        match output {
            StopCommandOutput::RuleSet(rule) => {
                assert_eq!(rule.trailing_pct, Some(5.0));
                assert_eq!(rule.take_profit_price, Some(18.0));
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_stop_set_rejects_invalid_condition_combinations() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let service = StopService::new(FakeStopRuleStore::default());

        let none_err = execute_stop_command_with_service(
            StopCommands::Set {
                code: "000001".to_string(),
                loss: None,
                profit: None,
                trailing: None,
            },
            &service,
            &storage,
        )
        .await
        .unwrap_err();
        assert!(matches!(none_err, QuantixError::Other(_)));
        assert!(none_err.to_string().contains("至少需要一个条件"));

        let conflict_err = execute_stop_command_with_service(
            StopCommands::Set {
                code: "000001".to_string(),
                loss: Some(14.5),
                profit: None,
                trailing: Some(5.0),
            },
            &service,
            &storage,
        )
        .await
        .unwrap_err();
        assert!(matches!(conflict_err, QuantixError::Other(_)));
        assert!(conflict_err.to_string().contains("--loss 和 --trailing"));
    }

    #[tokio::test]
    async fn test_execute_stop_set_rejects_codes_outside_watchlist() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let service = StopService::new(FakeStopRuleStore::default());

        let err = execute_stop_command_with_service(
            StopCommands::Set {
                code: "000002".to_string(),
                loss: Some(14.5),
                profit: None,
                trailing: None,
            },
            &service,
            &storage,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, QuantixError::Other(_)));
        assert!(err.to_string().contains("不在自选池"));
    }

    #[tokio::test]
    async fn test_execute_stop_list_returns_persisted_rules() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let store = FakeStopRuleStore {
            state: Arc::new(Mutex::new(FakeStopRuleState {
                rules: vec![stop_rule("000001")],
                removed_codes: Vec::new(),
            })),
        };
        let service = StopService::new(store);

        let output = execute_stop_command_with_service(StopCommands::List, &service, &storage)
            .await
            .unwrap();

        match output {
            StopCommandOutput::RuleList(rules) => {
                assert_eq!(rules.len(), 1);
                assert_eq!(rules[0].code, "000001");
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_stop_remove_succeeds() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let store = FakeStopRuleStore {
            state: Arc::new(Mutex::new(FakeStopRuleState {
                rules: vec![stop_rule("000001")],
                removed_codes: Vec::new(),
            })),
        };
        let service = StopService::new(store.clone());

        let output = execute_stop_command_with_service(
            StopCommands::Remove {
                code: "000001".to_string(),
            },
            &service,
            &storage,
        )
        .await
        .unwrap();

        match output {
            StopCommandOutput::RuleRemoved { code, removed } => {
                assert_eq!(code, "000001");
                assert!(removed);
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let state = store.state.lock().unwrap();
        assert!(state.rules.is_empty());
        assert_eq!(state.removed_codes, vec!["000001".to_string()]);
    }

    #[tokio::test]
    async fn test_execute_monitor_watchlist_once_returns_rows() {
        let service = MonitorService::new(
            FakeMonitorWatchlistReader {
                items: vec![
                    monitor_watchlist_item("000001", "core", &["bank"]),
                    monitor_watchlist_item("000002", "swing", &["tech"]),
                ],
            },
            FakeMonitorQuoteReader {
                rows: vec![
                    monitor_quote_row("000001", 16.2, 1.2),
                    monitor_quote_row("000002", 21.4, 2.6),
                ],
            },
            FakeMonitorAlertStore::default(),
        );

        let output = execute_monitor_command_with_service(
            MonitorCommands::Watchlist { once: true },
            &service,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::Watchlist(snapshot) => {
                assert_eq!(snapshot.rows.len(), 2);
                assert_eq!(snapshot.rows[0].code, "000001");
                assert_eq!(snapshot.rows[0].group, "core");
                assert_eq!(snapshot.rows[0].tags, vec!["bank".to_string()]);
                assert_eq!(snapshot.rows[0].last_price, Some(16.2));
                assert!(snapshot.triggered_alerts.is_empty());
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_monitor_watchlist_once_surfaces_triggered_alerts() {
        let store = FakeMonitorAlertStore {
            state: Arc::new(Mutex::new(FakeMonitorAlertState {
                next_id: 1,
                alerts: vec![monitor_alert(1, "000001", PriceAlertKind::Above, 16.0)],
                removed_ids: Vec::new(),
            })),
        };
        let service = MonitorService::new(
            FakeMonitorWatchlistReader {
                items: vec![monitor_watchlist_item("000001", "core", &[])],
            },
            FakeMonitorQuoteReader {
                rows: vec![monitor_quote_row("000001", 16.8, 3.2)],
            },
            store,
        );

        let output = execute_monitor_command_with_service(
            MonitorCommands::Watchlist { once: true },
            &service,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::Watchlist(snapshot) => {
                assert_eq!(snapshot.rows.len(), 1);
                assert_eq!(snapshot.triggered_alerts.len(), 1);
                assert_eq!(snapshot.triggered_alerts[0].alert_id, 1);
                assert_eq!(snapshot.triggered_alerts[0].code, "000001");
                assert_eq!(snapshot.triggered_alerts[0].current_price, 16.8);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_monitor_watchlist_requires_once() {
        let service = MonitorService::new(
            FakeMonitorWatchlistReader::default(),
            FakeMonitorQuoteReader::default(),
            FakeMonitorAlertStore::default(),
        );

        let err = execute_monitor_command_with_service(
            MonitorCommands::Watchlist { once: false },
            &service,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, QuantixError::Other(_)));
        assert!(err.to_string().contains("仅支持 --once"));
    }

    #[tokio::test]
    async fn test_execute_monitor_alert_add_above_succeeds() {
        let store = FakeMonitorAlertStore::default();
        let service = MonitorService::new(
            FakeMonitorWatchlistReader::default(),
            FakeMonitorQuoteReader::default(),
            store.clone(),
        );

        let output = execute_monitor_command_with_service(
            MonitorCommands::Alert(MonitorAlertCommands::Add {
                code: "000001".to_string(),
                above: Some(16.0),
                below: None,
            }),
            &service,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::AlertAdded(alert) => {
                assert_eq!(alert.code, "000001");
                assert_eq!(alert.kind, PriceAlertKind::Above);
                assert_eq!(alert.target_price, 16.0);
            }
            other => panic!("unexpected output: {:?}", other),
        }

        assert_eq!(store.state.lock().unwrap().alerts.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_monitor_alert_add_below_succeeds() {
        let store = FakeMonitorAlertStore::default();
        let service = MonitorService::new(
            FakeMonitorWatchlistReader::default(),
            FakeMonitorQuoteReader::default(),
            store.clone(),
        );

        let output = execute_monitor_command_with_service(
            MonitorCommands::Alert(MonitorAlertCommands::Add {
                code: "000001".to_string(),
                above: None,
                below: Some(15.0),
            }),
            &service,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::AlertAdded(alert) => {
                assert_eq!(alert.code, "000001");
                assert_eq!(alert.kind, PriceAlertKind::Below);
                assert_eq!(alert.target_price, 15.0);
            }
            other => panic!("unexpected output: {:?}", other),
        }

        assert_eq!(store.state.lock().unwrap().alerts.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_monitor_alert_list_returns_persisted_rows() {
        let service = MonitorService::new(
            FakeMonitorWatchlistReader::default(),
            FakeMonitorQuoteReader::default(),
            FakeMonitorAlertStore {
                state: Arc::new(Mutex::new(FakeMonitorAlertState {
                    next_id: 2,
                    alerts: vec![
                        monitor_alert(1, "000001", PriceAlertKind::Above, 16.0),
                        monitor_alert(2, "000002", PriceAlertKind::Below, 15.0),
                    ],
                    removed_ids: Vec::new(),
                })),
            },
        );

        let output = execute_monitor_command_with_service(
            MonitorCommands::Alert(MonitorAlertCommands::List),
            &service,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::AlertList(alerts) => {
                assert_eq!(alerts.len(), 2);
                assert_eq!(alerts[0].code, "000001");
                assert_eq!(alerts[1].kind, PriceAlertKind::Below);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_monitor_alert_remove_succeeds() {
        let store = FakeMonitorAlertStore {
            state: Arc::new(Mutex::new(FakeMonitorAlertState {
                next_id: 1,
                alerts: vec![monitor_alert(1, "000001", PriceAlertKind::Above, 16.0)],
                removed_ids: Vec::new(),
            })),
        };
        let service = MonitorService::new(
            FakeMonitorWatchlistReader::default(),
            FakeMonitorQuoteReader::default(),
            store.clone(),
        );

        let output = execute_monitor_command_with_service(
            MonitorCommands::Alert(MonitorAlertCommands::Remove { id: 1 }),
            &service,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::AlertRemoved { id, removed } => {
                assert_eq!(id, 1);
                assert!(removed);
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let state = store.state.lock().unwrap();
        assert!(state.alerts.is_empty());
        assert_eq!(state.removed_ids, vec![1]);
    }

    #[tokio::test]
    async fn test_execute_monitor_alert_add_rejects_invalid_threshold_combinations() {
        let service = MonitorService::new(
            FakeMonitorWatchlistReader::default(),
            FakeMonitorQuoteReader::default(),
            FakeMonitorAlertStore::default(),
        );

        let both_err = execute_monitor_command_with_service(
            MonitorCommands::Alert(MonitorAlertCommands::Add {
                code: "000001".to_string(),
                above: Some(16.0),
                below: Some(15.0),
            }),
            &service,
        )
        .await
        .unwrap_err();
        assert!(matches!(both_err, QuantixError::Other(_)));
        assert!(both_err.to_string().contains("必须且只能指定"));

        let none_err = execute_monitor_command_with_service(
            MonitorCommands::Alert(MonitorAlertCommands::Add {
                code: "000001".to_string(),
                above: None,
                below: None,
            }),
            &service,
        )
        .await
        .unwrap_err();
        assert!(matches!(none_err, QuantixError::Other(_)));
        assert!(none_err.to_string().contains("必须且只能指定"));
    }

    #[tokio::test]
    async fn test_execute_monitor_persist_triggered_alerts_falls_back_to_observed_time() {
        let store = FakeMonitorAlertStore {
            state: Arc::new(Mutex::new(FakeMonitorAlertState {
                next_id: 1,
                alerts: vec![monitor_alert(1, "000001", PriceAlertKind::Above, 16.0)],
                removed_ids: Vec::new(),
            })),
        };
        let observed_at = Utc.with_ymd_and_hms(2026, 3, 11, 10, 31, 0).unwrap();
        let snapshot = MonitorWatchlistSnapshot {
            rows: Vec::new(),
            triggered_alerts: vec![TriggeredAlert {
                alert_id: 1,
                code: "000001".to_string(),
                kind: PriceAlertKind::Above,
                target_price: 16.0,
                current_price: 16.8,
                triggered_at: None,
            }],
            warnings: Vec::new(),
        };

        persist_triggered_monitor_alerts(&store, &snapshot, observed_at)
            .await
            .unwrap();

        let alerts = store.state.lock().unwrap().alerts.clone();
        assert_eq!(alerts[0].last_triggered_at, Some(observed_at));
    }

    #[tokio::test]
    async fn test_execute_monitor_persist_triggered_alerts_preserves_snapshot_time() {
        let store = FakeMonitorAlertStore {
            state: Arc::new(Mutex::new(FakeMonitorAlertState {
                next_id: 1,
                alerts: vec![monitor_alert(1, "000001", PriceAlertKind::Above, 16.0)],
                removed_ids: Vec::new(),
            })),
        };
        let observed_at = Utc.with_ymd_and_hms(2026, 3, 11, 10, 31, 0).unwrap();
        let snapshot = MonitorWatchlistSnapshot {
            rows: Vec::new(),
            triggered_alerts: vec![TriggeredAlert {
                alert_id: 1,
                code: "000001".to_string(),
                kind: PriceAlertKind::Above,
                target_price: 16.0,
                current_price: 16.8,
                triggered_at: Some(monitor_sample_time()),
            }],
            warnings: Vec::new(),
        };

        persist_triggered_monitor_alerts(&store, &snapshot, observed_at)
            .await
            .unwrap();

        let alerts = store.state.lock().unwrap().alerts.clone();
        assert_eq!(alerts[0].last_triggered_at, Some(monitor_sample_time()));
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MarketBoardRequest {
        board_type: BoardType,
        date: Option<NaiveDate>,
        limit: usize,
        sort_by: BoardSortBy,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MarketLeaderRequest {
        filter: LeaderFilter,
        limit: usize,
        date: Option<NaiveDate>,
    }

    #[derive(Debug, Clone, Default)]
    struct FakeMarketState {
        board_requests: Vec<MarketBoardRequest>,
        leader_requests: Vec<MarketLeaderRequest>,
    }

    #[derive(Clone)]
    struct FakeMarketReader {
        state: Arc<Mutex<FakeMarketState>>,
    }

    impl FakeMarketReader {
        fn new() -> Self {
            Self {
                state: Arc::new(Mutex::new(FakeMarketState::default())),
            }
        }
    }

    #[async_trait]
    impl MarketDataReader for FakeMarketReader {
        async fn load_board_rankings(
            &self,
            board_type: BoardType,
            date: Option<NaiveDate>,
            limit: usize,
            sort_by: BoardSortBy,
        ) -> Result<Vec<BoardRankRow>> {
            self.state
                .lock()
                .unwrap()
                .board_requests
                .push(MarketBoardRequest {
                    board_type,
                    date,
                    limit,
                    sort_by,
                });

            let rows = match board_type {
                BoardType::Sector => vec![BoardRankRow::new("BK001", "银行", board_type, 1, 2.1)],
                BoardType::Concept => {
                    vec![BoardRankRow::new("GN001", "人工智能", board_type, 1, 4.2)]
                }
            };

            Ok(rows.into_iter().take(limit).collect())
        }

        async fn load_north_flow(
            &self,
            date: Option<NaiveDate>,
        ) -> Result<Option<NorthFlowSnapshot>> {
            Ok(Some(NorthFlowSnapshot::new(
                date.unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 3, 10).unwrap()),
                12.3,
                8.6,
                20.9,
                100.0,
            )))
        }

        async fn load_market_sentiment(
            &self,
            date: Option<NaiveDate>,
        ) -> Result<Option<MarketSentimentSnapshot>> {
            Ok(Some(MarketSentimentSnapshot::new(
                date.unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 3, 10).unwrap()),
                3210,
                1875,
                87,
                4,
                0.81,
                0.19,
                23,
            )))
        }

        async fn load_leaders(
            &self,
            filter: LeaderFilter,
            limit: usize,
            date: Option<NaiveDate>,
        ) -> Result<Vec<LeaderRow>> {
            self.state
                .lock()
                .unwrap()
                .leader_requests
                .push(MarketLeaderRequest {
                    filter: filter.clone(),
                    limit,
                    date,
                });

            let rows = match filter {
                LeaderFilter::Sector(name) => {
                    vec![LeaderRow::new("600000", "浦发银行", Some(name), None, 5.6)]
                }
                LeaderFilter::Concept(name) => {
                    vec![LeaderRow::new("300024", "机器人", None, Some(name), 7.1)]
                }
                LeaderFilter::All => vec![
                    LeaderRow::new("300024", "机器人", None, Some("人工智能".to_string()), 7.1),
                    LeaderRow::new("600000", "浦发银行", Some("银行".to_string()), None, 5.6),
                ],
            };

            Ok(rows.into_iter().take(limit).collect())
        }
    }

    #[tokio::test]
    async fn test_execute_market_sector_returns_rows() {
        let reader = FakeMarketReader::new();

        let output = execute_market_command_with_reader(
            MarketCommands::Sector {
                top: Some(1),
                date: Some("2026-03-09".to_string()),
                sort_by: Some("change".to_string()),
            },
            reader.clone(),
        )
        .await
        .unwrap();

        match output {
            MarketCommandOutput::BoardRows(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].board_name, "银行");
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let state = reader.state.lock().unwrap();
        assert_eq!(
            state.board_requests,
            vec![MarketBoardRequest {
                board_type: BoardType::Sector,
                date: Some(NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()),
                limit: 1,
                sort_by: BoardSortBy::ChangePct,
            }]
        );
    }

    #[tokio::test]
    async fn test_execute_market_concept_returns_rows() {
        let output = execute_market_command_with_reader(
            MarketCommands::Concept {
                top: Some(1),
                date: None,
                sort_by: None,
            },
            FakeMarketReader::new(),
        )
        .await
        .unwrap();

        match output {
            MarketCommandOutput::BoardRows(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].board_name, "人工智能");
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_market_north_returns_snapshot() {
        let output = execute_market_command_with_reader(
            MarketCommands::North {
                date: Some("2026-03-09".to_string()),
            },
            FakeMarketReader::new(),
        )
        .await
        .unwrap();

        match output {
            MarketCommandOutput::NorthFlow(Some(snapshot)) => {
                assert_eq!(
                    snapshot.trade_date,
                    NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()
                );
                assert_eq!(snapshot.total_amount, 20.9);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_market_sentiment_returns_snapshot() {
        let output = execute_market_command_with_reader(
            MarketCommands::Sentiment { date: None },
            FakeMarketReader::new(),
        )
        .await
        .unwrap();

        match output {
            MarketCommandOutput::Sentiment(Some(snapshot)) => {
                assert_eq!(snapshot.limit_up_count, 87);
                assert_eq!(snapshot.consecutive_board_count, 23);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_market_leader_with_sector_returns_rows() {
        let reader = FakeMarketReader::new();

        let output = execute_market_command_with_reader(
            MarketCommands::Leader {
                sector: Some("银行".to_string()),
                concept: None,
                all: false,
                limit: Some(5),
                date: Some("2026-03-09".to_string()),
            },
            reader.clone(),
        )
        .await
        .unwrap();

        match output {
            MarketCommandOutput::Leaders(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].code, "600000");
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let state = reader.state.lock().unwrap();
        assert_eq!(
            state.leader_requests,
            vec![MarketLeaderRequest {
                filter: LeaderFilter::Sector("银行".to_string()),
                limit: 5,
                date: Some(NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()),
            }]
        );
    }

    #[tokio::test]
    async fn test_execute_market_overview_returns_combined_payload() {
        let output = execute_market_command_with_reader(
            MarketCommands::Overview {
                top: Some(1),
                date: None,
            },
            FakeMarketReader::new(),
        )
        .await
        .unwrap();

        match output {
            MarketCommandOutput::Overview(overview) => {
                assert_eq!(overview.top_sectors.len(), 1);
                assert_eq!(overview.top_concepts.len(), 1);
                assert_eq!(overview.north_flow.unwrap().total_amount, 20.9);
                assert_eq!(overview.sentiment.unwrap().limit_up_count, 87);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_market_leader_rejects_invalid_filter_combination() {
        let err = execute_market_command_with_reader(
            MarketCommands::Leader {
                sector: Some("银行".to_string()),
                concept: Some("人工智能".to_string()),
                all: false,
                limit: None,
                date: None,
            },
            FakeMarketReader::new(),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, QuantixError::Other(_)));
        assert!(err.to_string().contains("必须且只能指定"));
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
