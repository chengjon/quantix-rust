/// CLI 交互层
///
/// 处理命令行参数解析和交互式菜单
pub mod handlers;

use crate::core::Result;
use clap::{Parser, Subcommand};

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

    /// 自选池命令
    #[command(subcommand)]
    Watchlist(WatchlistCommands),

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
            Commands::Watchlist(cmd) => {
                handlers::run_watchlist_command(cmd).await?;
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
        let cli = Cli::try_parse_from([
            "quantix",
            "watchlist",
            "group",
            "create",
            "--name",
            "core",
        ])
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
}
