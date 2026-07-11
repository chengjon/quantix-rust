use clap::Subcommand;

/// account 命令族 clap 枚举（容器）：Register/Remove/Update 账户管理、List/Show 查询、Snapshot 资产快照、Group 分组管理。
#[derive(Subcommand, Debug)]
pub enum AccountCommands {
    /// 注册新账户
    Register {
        /// 账户 ID
        #[arg(long)]
        id: String,

        /// 账户类型: paper | mock_live | qmt_live（兼容 live 别名）
        #[arg(long, default_value = "paper")]
        account_type: String,

        /// 初始资金
        #[arg(long, default_value = "1000000")]
        capital: f64,

        /// 适配器名称
        #[arg(long, default_value = "paper")]
        adapter: String,
    },

    /// 列出所有账户
    List {
        /// 按类型过滤
        #[arg(long)]
        account_type: Option<String>,

        /// 仅显示启用的账户
        #[arg(long)]
        enabled_only: bool,
    },

    /// 查看账户详情
    Show {
        /// 账户 ID
        #[arg(long)]
        id: String,
    },

    /// 更新账户配置
    Update {
        /// 账户 ID
        #[arg(long)]
        id: String,

        /// 启用账户
        #[arg(long)]
        enable: bool,

        /// 禁用账户
        #[arg(long)]
        disable: bool,

        /// 设置初始资金
        #[arg(long)]
        capital: Option<f64>,

        /// 设置适配器名称
        #[arg(long)]
        adapter: Option<String>,
    },

    /// 删除账户
    Remove {
        /// 账户 ID
        #[arg(long)]
        id: String,
    },

    /// 设置默认账户
    Default {
        /// 账户 ID
        #[arg(long)]
        id: String,
    },

    /// 账户组管理
    #[command(subcommand)]
    Group(AccountGroupCommands),

    /// 资金聚合视图
    Summary,

    /// 订单拆分预览
    Split {
        /// 股票代码
        #[arg(long)]
        code: String,

        /// 买卖方向
        #[arg(long)]
        side: String,

        /// 总数量
        #[arg(long)]
        quantity: i64,

        /// 目标类型: single | group
        #[arg(long, default_value = "single")]
        target_type: String,

        /// 目标 ID (账户 ID 或账户组 ID)
        #[arg(long)]
        target_id: String,

        /// 价格 (可选)
        #[arg(long)]
        price: Option<f64>,
    },
}

/// account group 子命令枚举：Create 创建分组、Add/Remove 成员维护、List 列出。
#[derive(Subcommand, Debug)]
pub enum AccountGroupCommands {
    /// 创建账户组
    Create {
        /// 组 ID
        #[arg(long)]
        id: String,

        /// 组名称
        #[arg(long)]
        name: String,

        /// 分配策略: equal | proportional | weighted | primary_first
        #[arg(long, default_value = "equal")]
        strategy: String,
    },

    /// 列出账户组
    List,

    /// 查看账户组详情
    Show {
        /// 组 ID
        #[arg(long)]
        id: String,
    },

    /// 删除账户组
    Remove {
        /// 组 ID
        #[arg(long)]
        id: String,
    },

    /// 向账户组添加账户
    AddAccount {
        /// 组 ID
        #[arg(long = "group-id")]
        group_id: String,

        /// 账户 ID
        #[arg(long = "account-id")]
        account_id: String,
    },

    /// 从账户组移除账户
    RemoveAccount {
        /// 组 ID
        #[arg(long = "group-id")]
        group_id: String,

        /// 账户 ID
        #[arg(long = "account-id")]
        account_id: String,
    },

    /// 设置分配策略
    SetStrategy {
        /// 组 ID
        #[arg(long = "group-id")]
        group_id: String,

        /// 策略: equal | proportional | weighted | primary_first
        #[arg(long)]
        strategy: String,

        /// 主账户 ID (仅 primary_first 策略需要)
        #[arg(long = "primary-account")]
        primary_account: Option<String>,
    },
}
