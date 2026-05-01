use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum StrategyCommands {
    /// 创建策略实例
    Create {
        /// 策略实例 ID
        #[arg(long)]
        id: String,

        /// 内置策略名称
        #[arg(short, long)]
        name: String,

        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 参数，格式 key=value，可重复指定
        #[arg(long = "param")]
        params: Vec<String>,

        /// 创建后禁用该实例
        #[arg(long)]
        disabled: bool,
    },

    /// 更新策略实例
    Update {
        /// 策略实例 ID
        #[arg(long)]
        id: String,

        /// 内置策略名称
        #[arg(short, long)]
        name: Option<String>,

        /// 股票代码
        #[arg(short, long)]
        code: Option<String>,

        /// 参数，格式 key=value，可重复指定
        #[arg(long = "param")]
        params: Vec<String>,

        /// 启用该实例
        #[arg(long, conflicts_with = "disable")]
        enable: bool,

        /// 禁用该实例
        #[arg(long, conflicts_with = "enable")]
        disable: bool,
    },

    /// 删除策略实例
    Delete {
        /// 策略实例 ID
        #[arg(long)]
        id: String,
    },

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
        /// 策略名称或策略实例 ID
        #[arg(short, long)]
        name: Option<String>,

        /// 显式按实例 ID 查询
        #[arg(long, conflicts_with = "name")]
        id: Option<String>,
    },

    /// 策略调度配置
    #[command(subcommand)]
    Config(StrategyConfigCommands),

    /// 策略守护进程
    #[command(subcommand)]
    Daemon(StrategyDaemonCommands),

    /// 策略信号
    #[command(subcommand)]
    Signal(StrategySignalCommands),

    /// 执行请求
    #[command(subcommand)]
    Request(StrategyRequestCommands),

    /// 策略服务
    #[command(subcommand)]
    Service(StrategyServiceCommands),

    /// 策略服务配置
    #[command(subcommand)]
    ServiceConfig(StrategyServiceConfigCommands),
}

#[derive(Subcommand, Debug)]
pub enum StrategyConfigCommands {
    /// 初始化策略配置
    Init,

    /// 显示策略配置
    Show,
}

#[derive(Subcommand, Debug)]
pub enum StrategyDaemonCommands {
    /// 运行策略守护进程
    Run {
        /// 仅执行一轮
        #[arg(long)]
        once: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum StrategySignalCommands {
    /// 列出信号
    List {
        /// 策略实例 ID
        #[arg(long = "strategy-instance")]
        strategy_instance: Option<String>,

        /// 策略名称
        #[arg(long = "strategy")]
        strategy: Option<String>,

        /// 股票代码
        #[arg(short = 'c', long = "code")]
        code: Option<String>,

        /// 审批状态
        #[arg(long = "approval-status")]
        approval_status: Option<String>,

        /// 信号状态
        #[arg(long = "signal-status")]
        signal_status: Option<String>,

        /// 限制返回条数
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// 批准信号
    Approve {
        /// 信号 ID
        #[arg(long = "signal-id")]
        signal_id: String,

        /// 目标执行模式: paper | mock_live | qmt_live（live 将被拒绝并提示改走 qmt_live）
        #[arg(long = "target-mode")]
        target_mode: String,

        /// 目标账户
        #[arg(long = "target-account")]
        target_account: String,
    },

    /// 拒绝信号
    Reject {
        /// 信号 ID
        #[arg(long = "signal-id")]
        signal_id: String,

        /// 拒绝原因
        #[arg(long)]
        reason: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum StrategyRequestCommands {
    /// 列出执行请求
    List {
        /// 请求状态
        #[arg(long)]
        status: Option<String>,

        /// 目标模式过滤: paper | mock_live | qmt_live | live（legacy rejected mode）
        #[arg(long = "target-mode")]
        target_mode: Option<String>,

        /// 目标账户
        #[arg(long = "target-account")]
        target_account: Option<String>,

        /// 限制返回条数
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// 显示统计摘要
        #[arg(long)]
        stats: bool,
    },

    /// 显示请求详情
    Show {
        /// 请求 ID
        #[arg(long = "request-id")]
        request_id: String,

        /// 显示完整 payload
        #[arg(long)]
        verbose: bool,
    },

    /// 执行一个待处理请求
    Execute {
        /// 请求 ID
        #[arg(long = "request-id")]
        request_id: String,
    },

    /// 取消一个待处理请求
    Cancel {
        /// 请求 ID
        #[arg(long = "request-id")]
        request_id: String,

        /// 取消原因
        #[arg(long)]
        reason: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum StrategyServiceCommands {
    Install,
    Uninstall,
    Start,
    Stop,
    Status,
    Enable,
    Disable,
}

#[derive(Subcommand, Debug)]
pub enum StrategyServiceConfigCommands {
    Show,
    Set {
        /// quantix 二进制绝对路径
        #[arg(long = "quantix-bin")]
        quantix_bin: String,

        /// 可选环境文件
        #[arg(long = "env-file")]
        env_file: Option<String>,
    },
}
