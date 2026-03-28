use clap::{ArgGroup, Subcommand};

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
            .args(["loss", "profit", "loss_pct", "profit_pct", "trailing"])
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

        /// 止损百分比
        #[arg(long = "loss-pct", conflicts_with_all = ["loss", "trailing"])]
        loss_pct: Option<f64>,

        /// 止盈百分比
        #[arg(long = "profit-pct", conflicts_with = "profit")]
        profit_pct: Option<f64>,

        /// 跟踪止损百分比
        #[arg(long, conflicts_with_all = ["loss", "loss_pct"])]
        trailing: Option<f64>,
    },

    /// 更新止盈止损规则
    #[command(group(
        ArgGroup::new("stop_rule_update_change")
            .args([
                "loss",
                "profit",
                "loss_pct",
                "profit_pct",
                "trailing",
                "clear_loss",
                "clear_profit",
                "clear_loss_pct",
                "clear_profit_pct",
                "clear_trailing",
            ])
            .required(true)
            .multiple(true)
    ))]
    Update {
        /// 股票代码
        code: String,

        /// 固定止损价
        #[arg(long, conflicts_with_all = ["loss_pct", "trailing"])]
        loss: Option<f64>,

        /// 固定止盈价
        #[arg(long, conflicts_with = "profit_pct")]
        profit: Option<f64>,

        /// 止损百分比
        #[arg(long = "loss-pct", conflicts_with_all = ["loss", "trailing"])]
        loss_pct: Option<f64>,

        /// 止盈百分比
        #[arg(long = "profit-pct", conflicts_with = "profit")]
        profit_pct: Option<f64>,

        /// 跟踪止损百分比
        #[arg(long, conflicts_with_all = ["loss", "loss_pct"])]
        trailing: Option<f64>,

        /// 清除固定止损价
        #[arg(long = "clear-loss")]
        clear_loss: bool,

        /// 清除固定止盈价
        #[arg(long = "clear-profit")]
        clear_profit: bool,

        /// 清除止损百分比
        #[arg(long = "clear-loss-pct")]
        clear_loss_pct: bool,

        /// 清除止盈百分比
        #[arg(long = "clear-profit-pct")]
        clear_profit_pct: bool,

        /// 清除跟踪止损
        #[arg(long = "clear-trailing")]
        clear_trailing: bool,
    },

    /// 列出止盈止损规则
    List,

    /// 查看止盈止损状态
    Status {
        /// 按股票代码过滤
        #[arg(long)]
        code: Option<String>,
    },

    /// 查看止盈止损历史
    History {
        /// 按股票代码过滤
        #[arg(long)]
        code: Option<String>,

        /// 限制返回条数
        #[arg(long, default_value = "20")]
        limit: usize,

        /// 按事件日期过滤 (YYYY-MM-DD)
        #[arg(long)]
        date: Option<String>,

        /// 按事件类型过滤
        #[arg(long = "type")]
        event_type: Option<String>,
    },

    /// 删除止盈止损规则
    Remove {
        /// 股票代码
        code: String,
    },
}
