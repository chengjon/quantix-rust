use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum RiskCommands {
    /// 导入标准化实盘流水
    #[command(subcommand)]
    Import(RiskImportCommands),

    /// 重建实盘镜像账户
    #[command(subcommand)]
    Rebuild(RiskRebuildCommands),

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
    Status {
        /// 数据源: paper | live_import
        #[arg(long)]
        source: Option<String>,

        /// 账户 ID
        #[arg(long)]
        account: Option<String>,
    },

    /// 查看当前当日盈亏快照
    Pnl {
        /// 数据源: paper | live_import
        #[arg(long)]
        source: Option<String>,

        /// 账户 ID
        #[arg(long)]
        account: Option<String>,
    },

    /// 查看当前持仓风险分布
    Position {
        /// 数据源: paper | live_import
        #[arg(long)]
        source: Option<String>,

        /// 账户 ID
        #[arg(long)]
        account: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum RiskImportCommands {
    /// 导入标准化实盘流水
    LiveTrades {
        /// 账户 ID
        #[arg(long)]
        account: String,

        /// 输入文件
        #[arg(long)]
        input: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum RiskRebuildCommands {
    /// 重建实盘镜像账户
    LiveAccount {
        /// 账户 ID
        #[arg(long)]
        account: String,
    },
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
