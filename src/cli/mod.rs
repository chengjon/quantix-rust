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

    /// 模拟交易命令
    #[command(subcommand)]
    Trade(TradeCommands),

    /// 风险管理命令
    #[command(subcommand)]
    Risk(RiskCommands),

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
    #[command(group(
        ArgGroup::new("monitor_watchlist_mode")
            .args(["once", "repeat"])
            .required(true)
            .multiple(false)
    ))]
    Watchlist {
        /// 执行一次监控
        #[arg(long)]
        once: bool,

        /// 持续重复监控
        #[arg(long)]
        repeat: bool,
    },

    /// 价格告警管理
    #[command(subcommand)]
    Alert(MonitorAlertCommands),

    /// 监控配置管理
    #[command(subcommand)]
    Config(MonitorConfigCommands),

    /// 监控守护进程
    #[command(subcommand)]
    Daemon(MonitorDaemonCommands),

    /// systemd 用户服务管理
    #[command(subcommand)]
    Service(MonitorServiceCommands),

    /// 监控服务配置
    #[command(subcommand)]
    ServiceConfig(MonitorServiceConfigCommands),

    /// 监控事件历史
    #[command(subcommand)]
    Event(MonitorEventCommands),
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
pub enum MonitorConfigCommands {
    /// 显示当前监控配置
    Show,

    /// 修改监控配置
    #[command(group(
        ArgGroup::new("monitor_config_mutation")
            .args(["interval_seconds", "group", "persist_events"])
            .required(true)
            .multiple(false)
    ))]
    Set {
        /// 轮询间隔，单位秒
        #[arg(long)]
        interval_seconds: Option<u64>,

        /// 自选池分组
        #[arg(long)]
        group: Option<String>,

        /// 是否持久化业务事件
        #[arg(long)]
        persist_events: Option<bool>,
    },

    /// 清除分组限制
    ClearGroup,
}

#[derive(Subcommand, Debug)]
pub enum MonitorDaemonCommands {
    /// 运行监控守护进程
    Run,
}

#[derive(Subcommand, Debug)]
pub enum MonitorServiceCommands {
    /// 安装 systemd 用户服务
    Install,
    /// 卸载 systemd 用户服务
    Uninstall,
    /// 启动 systemd 用户服务
    Start,
    /// 停止 systemd 用户服务
    Stop,
    /// 查看 systemd 用户服务状态
    Status,
    /// 启用开机自启
    Enable,
    /// 禁用开机自启
    Disable,
}

#[derive(Subcommand, Debug)]
pub enum MonitorServiceConfigCommands {
    /// 显示当前服务配置
    Show,

    /// 设置 quantix 可执行文件路径
    Set {
        /// quantix 二进制绝对路径
        #[arg(long = "quantix-bin")]
        quantix_bin: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum MonitorEventCommands {
    /// 查看监控事件历史
    List {
        /// 限制返回条数
        #[arg(long, default_value = "20")]
        limit: usize,

        /// 按股票代码过滤
        #[arg(long)]
        code: Option<String>,

        /// 按事件类型过滤
        #[arg(long = "type")]
        event_type: Option<String>,
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
pub enum RiskCommands {
    /// 风控规则管理
    #[command(subcommand)]
    Rule(RiskRuleCommands),

    /// 查看最近风控事件
    Log {
        /// 按事件写入日过滤 (YYYY-MM-DD)
        #[arg(long)]
        date: Option<String>,
        /// 按事件类型过滤
        #[arg(long = "type")]
        event_type: Option<String>,
        /// 限制返回条数
        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// 风控买入锁管理
    #[command(subcommand)]
    Lock(RiskLockCommands),

    /// 查看当前风控状态
    Status,

    /// 查看当前当日盈亏快照
    Pnl,

    /// 查看当前持仓风险分布
    Position,
}

#[derive(Subcommand, Debug)]
pub enum RiskLockCommands {
    /// 手动释放当日买入锁
    Release,
}

#[derive(Subcommand, Debug)]
pub enum RiskRuleCommands {
    /// 设置风控规则
    Set {
        /// 规则类型
        #[arg(long = "type")]
        rule_type: String,

        /// 规则值
        #[arg(long)]
        value: String,
    },

    /// 列出所有风控规则
    List,

    /// 启用风控规则
    Enable {
        /// 规则类型
        #[arg(long = "type")]
        rule_type: String,
    },

    /// 禁用风控规则
    Disable {
        /// 规则类型
        #[arg(long = "type")]
        rule_type: String,
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

#[derive(Subcommand, Debug)]
pub enum TradeCommands {
    /// 初始化默认模拟账户
    Init {
        #[arg(long)]
        capital: Option<f64>,
        #[arg(long)]
        commission_rate: Option<f64>,
        #[arg(long)]
        commission_min: Option<f64>,
        #[arg(long)]
        stamp_duty_rate: Option<f64>,
        #[arg(long)]
        transfer_fee_rate: Option<f64>,
    },

    /// 重置默认模拟账户
    Reset {
        #[arg(long)]
        capital: Option<f64>,
        #[arg(long)]
        commission_rate: Option<f64>,
        #[arg(long)]
        commission_min: Option<f64>,
        #[arg(long)]
        stamp_duty_rate: Option<f64>,
        #[arg(long)]
        transfer_fee_rate: Option<f64>,
    },

    /// 立即成交的限价买入
    Buy {
        code: String,
        #[arg(long)]
        price: f64,
        #[arg(long)]
        volume: i64,
    },

    /// 立即成交的限价卖出
    Sell {
        code: String,
        #[arg(long)]
        price: f64,
        #[arg(long)]
        volume: i64,
    },

    /// 查看成交历史
    History {
        #[arg(long)]
        code: Option<String>,
        #[arg(long)]
        limit: Option<usize>,
    },

    /// 查看费用明细
    Fees {
        #[arg(long)]
        code: Option<String>,
        #[arg(long)]
        limit: Option<usize>,
    },

    /// 查看账户概览
    Overview {
        #[arg(long)]
        current: bool,
    },

    /// 查看当前持仓
    Position {
        #[arg(long)]
        current: bool,
    },

    /// 查看当前现金快照
    Cash,
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
            Commands::Stop(cmd) => {
                handlers::run_stop_command(cmd).await?;
            }
            Commands::Watchlist(cmd) => {
                handlers::run_watchlist_command(cmd).await?;
            }
            Commands::Market(cmd) => {
                handlers::run_market_command(cmd).await?;
            }
            Commands::Trade(cmd) => {
                handlers::run_trade_command(cmd).await?;
            }
            Commands::Risk(cmd) => {
                handlers::run_risk_command(cmd).await?;
            }
            Commands::Status { health } => {
                handlers::run_status(health).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
