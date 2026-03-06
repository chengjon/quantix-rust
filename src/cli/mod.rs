/// CLI 交互层
///
/// 处理命令行参数解析和交互式菜单

pub mod handlers;

use clap::{Parser, Subcommand};
use crate::core::Result;

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

    /// 任务命令
    #[command(subcommand)]
    Task(TaskCommands),

    /// 分析命令
    #[command(subcommand)]
    Analyze(AnalyzeCommands),

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
    /// 添加定时任务
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

    /// 列出所有任务
    List,

    /// 启动任务调度器
    Start {
        /// 后台运行
        #[arg(long)]
        daemon: bool,
    },

    /// 停止任务调度器
    Stop,

    /// 查看任务状态
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
            Commands::Status { health } => {
                handlers::run_status(health).await?;
            }
        }
        Ok(())
    }
}
