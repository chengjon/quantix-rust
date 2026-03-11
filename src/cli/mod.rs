/// CLI 交互层
///
/// 处理命令行参数解析和交互式菜单
pub mod handlers;

use crate::core::Result;
use clap::{ArgGroup, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "quantix")]
#[command(about = "A股量化交易 CLI 工具", long_about = None)]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 初始化配置和数据库
    Init {
        /// 配置文件路径（指向原 quantix 项目）
        #[arg(short, long, default_value = "../config")]
        config_path: String,
    },

    /// 交互式菜单
    Menu {
        /// 启用 TUI 界面
        #[arg(long)]
        tui: bool,
    },

    /// 数据命令
    #[command(subcommand)]
    Data(DataCommands),

    /// 策略命令
    #[command(subcommand)]
    Strategy(StrategyCommands),

    /// 任务命令（实验性，Foundation P0）
    #[command(subcommand)]
    Task(TaskCommands),

    /// 分析命令
    #[command(subcommand)]
    Analyze(AnalyzeCommands),

    /// 监控命令
    #[command(subcommand)]
    Monitor(MonitorCommands),

    /// 止盈止损命令
    #[command(subcommand)]
    Stop(StopCommands),

    /// 自选池命令
    #[command(subcommand)]
    Watchlist(WatchlistCommands),

    /// 市场分析命令
    #[command(subcommand)]
    Market(MarketCommands),

    /// 系统状态
    Status {
        /// 检查数据库连接
        #[arg(long)]
        health: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum DataCommands {
    /// 查询历史数据
    Query {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 开始日期 (YYYYMMDD)
        #[arg(short, long)]
        start: Option<String>,

        /// 结束日期 (YYYYMMDD)
        #[arg(short, long)]
        end: Option<String>,

        /// 数据类型
        #[arg(long, default_value = "daily")]
        r#type: String,

        /// 限制返回条数
        #[arg(short, long, default_value = "100")]
        limit: usize,
    },

    /// 导出数据到文件
    Export {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 输出格式
        #[arg(long, default_value = "parquet")]
        format: String,

        /// 输出目录
        #[arg(short, long, default_value = "./data")]
        output: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum StrategyCommands {
    /// 运行策略
    Run {
        /// 策略名称
        #[arg(short, long)]
        name: String,

        /// 运行模式
        #[arg(long, default_value = "backtest")]
        mode: String,

        /// 股票代码
        #[arg(short, long)]
        code: Option<String>,
    },

    /// 列出所有策略
    List,

    /// 显示策略详情
    Show {
        /// 策略名称
        #[arg(short, long)]
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    /// 添加定时任务（Foundation P0 不支持）
    Add {
        /// 任务名称
        #[arg(short, long)]
        name: String,

        /// Cron 表达式
        #[arg(long)]
        cron: String,

        /// 执行命令
        #[arg(short, long)]
        command: String,
    },

    /// 列出预置任务模板
    List,

    /// 启动任务调度器（仅支持前台模式）
    Start {
        /// 后台运行（Foundation P0 不支持）
        #[arg(long)]
        daemon: bool,
    },

    /// 停止当前前台调度器
    Stop,

    /// 查看实验性任务能力状态
    Status,
}

#[derive(Subcommand, Debug)]
pub enum AnalyzeCommands {
    /// 计算技术指标
    Indicators {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 指标列表 (逗号分隔)
        #[arg(short, long)]
        indicators: String,
    },

    /// 回测报告
    Backtest {
        /// 回测 ID
        #[arg(short, long)]
        id: String,
    },

    /// 选股筛选
    #[command(subcommand)]
    Screener(ScreenerCommands),
}

#[derive(Subcommand, Debug)]
pub enum ScreenerCommands {
    /// 列出内置筛选条件模板
    PresetList,

    /// 运行选股筛选
    Run {
        /// 显式股票代码列表，逗号分隔
        #[arg(long)]
        codes: Option<String>,

        /// 使用自选池作为筛选股票池
        #[arg(long)]
        watchlist: bool,

        /// 自选池分组，仅在 --watchlist 时生效
        #[arg(long)]
        group: Option<String>,

        /// 筛选条件模板，可重复传入
        #[arg(long = "preset")]
        preset: Vec<String>,

        /// 限制返回条数
        #[arg(long)]
        limit: Option<usize>,

        /// 排序字段
        #[arg(long)]
        sort_by: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum MonitorCommands {
    /// 运行自选池监控
    Watchlist {
        /// 执行一次监控
        #[arg(long, required = true)]
        once: bool,
    },

    /// 价格告警管理
    #[command(subcommand)]
    Alert(MonitorAlertCommands),
}

#[derive(Subcommand, Debug)]
pub enum MonitorAlertCommands {
    /// 添加价格告警
    #[command(group(
        ArgGroup::new("monitor_alert_threshold")
            .args(["above", "below"])
            .required(true)
            .multiple(false)
    ))]
    Add {
        /// 股票代码
        code: String,

        /// 高于阈值时告警
        #[arg(long)]
        above: Option<f64>,

        /// 低于阈值时告警
        #[arg(long)]
        below: Option<f64>,
    },

    /// 列出价格告警
    List,

    /// 删除价格告警
    Remove {
        /// 告警 ID
        id: u64,
    },
}

#[derive(Subcommand, Debug)]
pub enum StopCommands {
    /// 设置止盈止损规则
    #[command(group(
        ArgGroup::new("stop_rule_threshold")
            .args(["loss", "profit", "trailing"])
            .required(true)
            .multiple(true)
    ))]
    Set {
        /// 股票代码
        code: String,

        /// 固定止损价
        #[arg(long, conflicts_with = "trailing")]
        loss: Option<f64>,

        /// 固定止盈价
        #[arg(long)]
        profit: Option<f64>,

        /// 跟踪止损百分比
        #[arg(long, conflicts_with = "loss")]
        trailing: Option<f64>,
    },

    /// 列出止盈止损规则
    List,

    /// 删除止盈止损规则
    Remove {
        /// 股票代码
        code: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum MarketCommands {
    /// 行业板块排名
    Sector {
        /// 限制返回条数
        #[arg(long)]
        top: Option<usize>,

        /// 指定交易日期
        #[arg(long)]
        date: Option<String>,

        /// 排序字段
        #[arg(long)]
        sort_by: Option<String>,
    },

    /// 概念板块排名
    Concept {
        /// 限制返回条数
        #[arg(long)]
        top: Option<usize>,

        /// 指定交易日期
        #[arg(long)]
        date: Option<String>,

        /// 排序字段
        #[arg(long)]
        sort_by: Option<String>,
    },

    /// 北向资金概览
    North {
        /// 指定交易日期
        #[arg(long)]
        date: Option<String>,
    },

    /// 市场情绪概览
    Sentiment {
        /// 指定交易日期
        #[arg(long)]
        date: Option<String>,
    },

    /// 龙头股识别
    #[command(group(
        ArgGroup::new("leader_filter")
            .args(["sector", "concept", "all"])
            .required(true)
            .multiple(false)
    ))]
    Leader {
        /// 行业名称
        #[arg(long)]
        sector: Option<String>,

        /// 概念名称
        #[arg(long)]
        concept: Option<String>,

        /// 全市场龙头
        #[arg(long)]
        all: bool,

        /// 限制返回条数
        #[arg(long)]
        limit: Option<usize>,

        /// 指定交易日期
        #[arg(long)]
        date: Option<String>,
    },

    /// 市场综合概览
    Overview {
        /// 板块列表条数
        #[arg(long)]
        top: Option<usize>,

        /// 指定交易日期
        #[arg(long)]
        date: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum WatchlistCommands {
    /// 添加股票到自选池
    Add {
        /// 股票代码
        #[arg(long)]
        code: String,

        /// 分组名称
        #[arg(long)]
        group: Option<String>,
    },

    /// 从自选池移除股票
    Remove {
        /// 股票代码
        #[arg(long)]
        code: String,
    },

    /// 列出自选池
    List {
        /// 分组过滤
        #[arg(long)]
        group: Option<String>,

        /// 标签过滤
        #[arg(long)]
        tag: Option<String>,

        /// 展示最佳努力价格
        #[arg(long)]
        with_price: bool,
    },

    /// 移动股票到目标分组
    Move {
        /// 股票代码
        #[arg(long)]
        code: String,

        /// 目标分组
        #[arg(long)]
        group: String,
    },

    /// 分组管理
    #[command(subcommand)]
    Group(WatchlistGroupCommands),

    /// 标签管理
    #[command(subcommand)]
    Tag(WatchlistTagCommands),

    /// 查看历史
    History {
        /// 股票代码
        #[arg(long)]
        code: Option<String>,

        /// 返回条数
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
pub enum WatchlistGroupCommands {
    /// 创建分组
    Create {
        /// 分组名称
        #[arg(long)]
        name: String,
    },

    /// 列出分组
    List,
}

#[derive(Subcommand, Debug)]
pub enum WatchlistTagCommands {
    /// 添加标签
    Add {
        /// 股票代码
        #[arg(long)]
        code: String,

        /// 标签
        #[arg(long)]
        tag: String,
    },

    /// 删除标签
    Remove {
        /// 股票代码
        #[arg(long)]
        code: String,

        /// 标签
        #[arg(long)]
        tag: String,
    },

    /// 列出股票标签
    List {
        /// 股票代码
        #[arg(long)]
        code: String,
    },
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::Init { config_path } => {
                handlers::run_init(config_path).await?;
            }
            Commands::Menu { tui } => {
                if tui {
                    handlers::run_tui_menu().await?;
                } else {
                    handlers::run_simple_menu().await?;
                }
            }
            Commands::Data(cmd) => {
                handlers::run_data_command(cmd).await?;
            }
            Commands::Strategy(cmd) => {
                handlers::run_strategy_command(cmd).await?;
            }
            Commands::Task(cmd) => {
                handlers::run_task_command(cmd).await?;
            }
            Commands::Analyze(cmd) => {
                handlers::run_analyze_command(cmd).await?;
            }
            Commands::Monitor(cmd) => {
                handlers::run_monitor_command(cmd).await?;
            }
            Commands::Stop(_cmd) => {
                return Err(crate::core::QuantixError::Unsupported(
                    "Phase 25A 仅包含 stop CLI parser，handler 尚未实现".to_string(),
                ));
            }
            Commands::Watchlist(cmd) => {
                handlers::run_watchlist_command(cmd).await?;
            }
            Commands::Market(cmd) => {
                handlers::run_market_command(cmd).await?;
            }
            Commands::Status { health } => {
                handlers::run_status(health).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use clap::error::ErrorKind;

    #[test]
    fn parses_watchlist_add_command() {
        let cli = Cli::try_parse_from([
            "quantix",
            "watchlist",
            "add",
            "--code",
            "000001",
            "--group",
            "core",
        ])
        .unwrap();

        match cli.command {
            Commands::Watchlist(WatchlistCommands::Add { code, group }) => {
                assert_eq!(code, "000001");
                assert_eq!(group.as_deref(), Some("core"));
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_watchlist_list_command_with_filters_and_price_flag() {
        let cli = Cli::try_parse_from([
            "quantix",
            "watchlist",
            "list",
            "--group",
            "core",
            "--tag",
            "bank",
            "--with-price",
        ])
        .unwrap();

        match cli.command {
            Commands::Watchlist(WatchlistCommands::List {
                group,
                tag,
                with_price,
            }) => {
                assert_eq!(group.as_deref(), Some("core"));
                assert_eq!(tag.as_deref(), Some("bank"));
                assert!(with_price);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_watchlist_group_create_command() {
        let cli =
            Cli::try_parse_from(["quantix", "watchlist", "group", "create", "--name", "core"])
                .unwrap();

        match cli.command {
            Commands::Watchlist(WatchlistCommands::Group(WatchlistGroupCommands::Create {
                name,
            })) => {
                assert_eq!(name, "core");
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_watchlist_history_command_with_limit() {
        let cli = Cli::try_parse_from([
            "quantix",
            "watchlist",
            "history",
            "--code",
            "000001",
            "--limit",
            "20",
        ])
        .unwrap();

        match cli.command {
            Commands::Watchlist(WatchlistCommands::History { code, limit }) => {
                assert_eq!(code.as_deref(), Some("000001"));
                assert_eq!(limit, 20);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_screener_preset_list_command() {
        let cli = Cli::try_parse_from(["quantix", "analyze", "screener", "preset-list"]).unwrap();

        match cli.command {
            Commands::Analyze(AnalyzeCommands::Screener(ScreenerCommands::PresetList)) => {}
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_screener_run_command_with_codes_and_preset() {
        let cli = Cli::try_parse_from([
            "quantix",
            "analyze",
            "screener",
            "run",
            "--codes",
            "000001,600519",
            "--preset",
            "close_above_ma:period=20",
        ])
        .unwrap();

        match cli.command {
            Commands::Analyze(AnalyzeCommands::Screener(ScreenerCommands::Run {
                codes,
                watchlist,
                group,
                preset,
                limit,
                sort_by,
            })) => {
                assert_eq!(codes.as_deref(), Some("000001,600519"));
                assert!(!watchlist);
                assert_eq!(group, None);
                assert_eq!(preset, vec!["close_above_ma:period=20"]);
                assert_eq!(limit, None);
                assert_eq!(sort_by.as_deref(), None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_screener_run_command_with_watchlist_group_and_multiple_presets() {
        let cli = Cli::try_parse_from([
            "quantix",
            "analyze",
            "screener",
            "run",
            "--watchlist",
            "--group",
            "core",
            "--preset",
            "close_above_ma:period=20",
            "--preset",
            "rsi_gte:period=14,value=55",
        ])
        .unwrap();

        match cli.command {
            Commands::Analyze(AnalyzeCommands::Screener(ScreenerCommands::Run {
                codes,
                watchlist,
                group,
                preset,
                limit,
                sort_by,
            })) => {
                assert_eq!(codes, None);
                assert!(watchlist);
                assert_eq!(group.as_deref(), Some("core"));
                assert_eq!(
                    preset,
                    vec![
                        "close_above_ma:period=20".to_string(),
                        "rsi_gte:period=14,value=55".to_string()
                    ]
                );
                assert_eq!(limit, None);
                assert_eq!(sort_by.as_deref(), None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_monitor_watchlist_command_with_once() {
        let cli = Cli::try_parse_from(["quantix", "monitor", "watchlist", "--once"]).unwrap();

        match cli.command {
            Commands::Monitor(MonitorCommands::Watchlist { once }) => {
                assert!(once);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_monitor_watchlist_rejects_missing_once() {
        let err = Cli::try_parse_from(["quantix", "monitor", "watchlist"]).unwrap_err();

        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
        assert!(err.to_string().contains("--once"));
    }

    #[test]
    fn parses_monitor_alert_add_command_with_above() {
        let cli = Cli::try_parse_from([
            "quantix", "monitor", "alert", "add", "000001", "--above", "16.0",
        ])
        .unwrap();

        match cli.command {
            Commands::Monitor(MonitorCommands::Alert(MonitorAlertCommands::Add {
                code,
                above,
                below,
            })) => {
                assert_eq!(code, "000001");
                assert_eq!(above, Some(16.0));
                assert_eq!(below, None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_monitor_alert_add_command_with_below() {
        let cli = Cli::try_parse_from([
            "quantix", "monitor", "alert", "add", "000001", "--below", "15.0",
        ])
        .unwrap();

        match cli.command {
            Commands::Monitor(MonitorCommands::Alert(MonitorAlertCommands::Add {
                code,
                above,
                below,
            })) => {
                assert_eq!(code, "000001");
                assert_eq!(above, None);
                assert_eq!(below, Some(15.0));
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_monitor_alert_list_command() {
        let cli = Cli::try_parse_from(["quantix", "monitor", "alert", "list"]).unwrap();

        match cli.command {
            Commands::Monitor(MonitorCommands::Alert(MonitorAlertCommands::List)) => {}
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_monitor_alert_remove_command() {
        let cli = Cli::try_parse_from(["quantix", "monitor", "alert", "remove", "12"]).unwrap();

        match cli.command {
            Commands::Monitor(MonitorCommands::Alert(MonitorAlertCommands::Remove { id })) => {
                assert_eq!(id, 12);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_monitor_alert_add_rejects_missing_threshold() {
        let err =
            Cli::try_parse_from(["quantix", "monitor", "alert", "add", "000001"]).unwrap_err();

        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
        assert!(err.to_string().contains("--above"));
        assert!(err.to_string().contains("--below"));
    }

    #[test]
    fn parses_monitor_alert_add_rejects_both_thresholds() {
        let err = Cli::try_parse_from([
            "quantix", "monitor", "alert", "add", "000001", "--above", "16.0", "--below", "15.0",
        ])
        .unwrap_err();

        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
        assert!(err.to_string().contains("--above"));
        assert!(err.to_string().contains("--below"));
    }

    #[test]
    fn parses_monitor_alert_add_rejects_non_numeric_threshold() {
        let err = Cli::try_parse_from([
            "quantix",
            "monitor",
            "alert",
            "add",
            "000001",
            "--above",
            "not-a-number",
        ])
        .unwrap_err();

        assert_eq!(err.kind(), ErrorKind::ValueValidation);
        assert!(err.to_string().contains("not-a-number"));
    }

    #[test]
    fn parses_stop_set_command_with_loss() {
        let cli = Cli::try_parse_from(["quantix", "stop", "set", "000001", "--loss", "14.5"])
            .unwrap();

        match cli.command {
            Commands::Stop(StopCommands::Set {
                code,
                loss,
                profit,
                trailing,
            }) => {
                assert_eq!(code, "000001");
                assert_eq!(loss, Some(14.5));
                assert_eq!(profit, None);
                assert_eq!(trailing, None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_stop_set_command_with_profit() {
        let cli = Cli::try_parse_from(["quantix", "stop", "set", "000001", "--profit", "18.0"])
            .unwrap();

        match cli.command {
            Commands::Stop(StopCommands::Set {
                code,
                loss,
                profit,
                trailing,
            }) => {
                assert_eq!(code, "000001");
                assert_eq!(loss, None);
                assert_eq!(profit, Some(18.0));
                assert_eq!(trailing, None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_stop_set_command_with_loss_and_profit() {
        let cli = Cli::try_parse_from([
            "quantix", "stop", "set", "000001", "--loss", "14.5", "--profit", "18.0",
        ])
        .unwrap();

        match cli.command {
            Commands::Stop(StopCommands::Set {
                code,
                loss,
                profit,
                trailing,
            }) => {
                assert_eq!(code, "000001");
                assert_eq!(loss, Some(14.5));
                assert_eq!(profit, Some(18.0));
                assert_eq!(trailing, None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_stop_set_command_with_trailing() {
        let cli =
            Cli::try_parse_from(["quantix", "stop", "set", "000001", "--trailing", "5"]).unwrap();

        match cli.command {
            Commands::Stop(StopCommands::Set {
                code,
                loss,
                profit,
                trailing,
            }) => {
                assert_eq!(code, "000001");
                assert_eq!(loss, None);
                assert_eq!(profit, None);
                assert_eq!(trailing, Some(5.0));
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_stop_list_command() {
        let cli = Cli::try_parse_from(["quantix", "stop", "list"]).unwrap();

        match cli.command {
            Commands::Stop(StopCommands::List) => {}
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_stop_remove_command() {
        let cli = Cli::try_parse_from(["quantix", "stop", "remove", "000001"]).unwrap();

        match cli.command {
            Commands::Stop(StopCommands::Remove { code }) => {
                assert_eq!(code, "000001");
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_stop_set_rejects_missing_thresholds() {
        let err = Cli::try_parse_from(["quantix", "stop", "set", "000001"]).unwrap_err();

        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
        assert!(err.to_string().contains("--loss"));
        assert!(err.to_string().contains("--profit"));
        assert!(err.to_string().contains("--trailing"));
    }

    #[test]
    fn parses_stop_set_rejects_loss_and_trailing_together() {
        let err = Cli::try_parse_from([
            "quantix", "stop", "set", "000001", "--loss", "14.5", "--trailing", "5",
        ])
        .unwrap_err();

        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
        assert!(err.to_string().contains("--loss"));
        assert!(err.to_string().contains("--trailing"));
    }

    #[test]
    fn parses_market_sector_command_with_top() {
        let cli = Cli::try_parse_from(["quantix", "market", "sector", "--top", "10"]).unwrap();

        match cli.command {
            Commands::Market(MarketCommands::Sector { top, date, sort_by }) => {
                assert_eq!(top, Some(10));
                assert_eq!(date, None);
                assert_eq!(sort_by, None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_market_concept_command_with_date() {
        let cli =
            Cli::try_parse_from(["quantix", "market", "concept", "--date", "2026-03-09"]).unwrap();

        match cli.command {
            Commands::Market(MarketCommands::Concept { top, date, sort_by }) => {
                assert_eq!(top, None);
                assert_eq!(date.as_deref(), Some("2026-03-09"));
                assert_eq!(sort_by, None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_market_north_command() {
        let cli = Cli::try_parse_from(["quantix", "market", "north"]).unwrap();

        match cli.command {
            Commands::Market(MarketCommands::North { date }) => {
                assert_eq!(date, None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_market_sentiment_command() {
        let cli = Cli::try_parse_from(["quantix", "market", "sentiment"]).unwrap();

        match cli.command {
            Commands::Market(MarketCommands::Sentiment { date }) => {
                assert_eq!(date, None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_market_leader_command_with_sector_and_limit() {
        let cli = Cli::try_parse_from([
            "quantix", "market", "leader", "--sector", "银行", "--limit", "5",
        ])
        .unwrap();

        match cli.command {
            Commands::Market(MarketCommands::Leader {
                sector,
                concept,
                all,
                limit,
                date,
            }) => {
                assert_eq!(sector.as_deref(), Some("银行"));
                assert_eq!(concept, None);
                assert!(!all);
                assert_eq!(limit, Some(5));
                assert_eq!(date, None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parses_market_overview_command() {
        let cli = Cli::try_parse_from(["quantix", "market", "overview"]).unwrap();

        match cli.command {
            Commands::Market(MarketCommands::Overview { top, date }) => {
                assert_eq!(top, None);
                assert_eq!(date, None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn rejects_market_leader_with_sector_and_concept_together() {
        let result = Cli::try_parse_from([
            "quantix",
            "market",
            "leader",
            "--sector",
            "银行",
            "--concept",
            "人工智能",
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_market_leader_without_any_filter() {
        let result = Cli::try_parse_from(["quantix", "market", "leader"]);

        assert!(result.is_err());
    }
}
