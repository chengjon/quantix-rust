use clap::Subcommand;

/// risk 命令族 clap 枚举（容器）：Import 实盘导入、Sync 同步、Rebuild 重建、Rule 规则、Log 日志、Lock 买入锁、Status/Pnl/Position 状态查询。
#[derive(Subcommand, Debug)]
pub enum RiskCommands {
    /// 导入标准化实盘流水
    #[command(subcommand)]
    Import(RiskImportCommands),

    /// 同步行业分类引用表
    #[command(subcommand)]
    Sync(RiskSyncCommands),

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

/// risk import 子命令枚举：LiveTrades 导入标准化实盘流水（account + input 文件路径）。
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

/// risk sync 子命令枚举：Industry 同步行业分类引用表（standard，目前仅支持 shenwan）。
#[derive(Subcommand, Debug)]
pub enum RiskSyncCommands {
    /// 同步行业分类引用表
    Industry {
        /// 分类标准，目前仅支持 shenwan
        #[arg(long)]
        standard: String,
    },
}

/// risk rebuild 子命令枚举：LiveAccount 重建实盘镜像账户。
#[derive(Subcommand, Debug)]
pub enum RiskRebuildCommands {
    /// 重建实盘镜像账户
    LiveAccount {
        /// 账户 ID
        #[arg(long)]
        account: String,
    },
}

/// risk lock 子命令枚举：Release 手动释放当日买入锁。
#[derive(Subcommand, Debug)]
pub enum RiskLockCommands {
    /// 手动释放当日买入锁
    Release,
}

/// risk rule 子命令枚举：Set 设置规则（type+value）、List 列出、Enable/Disable 启停。
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
