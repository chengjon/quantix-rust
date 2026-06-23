use clap::Subcommand;

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

#[derive(Subcommand, Debug)]
pub enum ExecutionCommands {
    /// 执行守护进程配置
    #[command(subcommand)]
    Config(ExecutionConfigCommands),

    /// 执行守护进程
    #[command(subcommand)]
    Daemon(ExecutionDaemonCommands),

    /// Bridge 诊断与预览命令
    #[command(subcommand)]
    Bridge(ExecutionBridgeCommands),

    /// QMT 执行兼容入口
    #[command(subcommand)]
    Qmt(ExecutionQmtCommands),
}

#[derive(Subcommand, Debug)]
pub enum ExecutionConfigCommands {
    /// 初始化执行配置
    Init,

    /// 显示执行配置
    Show,
}

#[derive(Subcommand, Debug)]
pub enum ExecutionDaemonCommands {
    /// 运行执行守护进程
    Run {
        /// 仅执行一轮
        #[arg(long)]
        once: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ExecutionBridgeCommands {
    /// 查看 bridge 能力状态
    Status {
        /// 追加 QMT promotion checklist
        #[arg(long)]
        checklist: bool,
    },

    /// 使用 frozen execution request 预览 QMT payload
    QmtPreview {
        /// 请求 ID
        #[arg(long = "request-id")]
        request_id: String,
    },

    /// 提交真实订单 (需要确认)
    QmtLive {
        /// 请求 ID (frozen execution request)
        #[arg(long = "request-id")]
        request_id: String,

        /// 跳过确认提示 (危险!)
        #[arg(long)]
        yes: bool,
    },

    /// 查询订单状态或 qmt_live task 结果
    QmtQuery {
        /// 订单 ID，或 qmt_live task_id
        #[arg(long = "order-id")]
        order_id: String,
    },

    /// 从本地运行时记录构建 qmt_live 审计证据
    QmtAudit {
        /// frozen execution request ID
        #[arg(long = "request-id")]
        request_id: Option<String>,

        /// qmt_live task_id
        #[arg(long = "task-id")]
        task_id: Option<String>,

        /// qmt_live local_submission_id
        #[arg(long = "local-submission-id")]
        local_submission_id: Option<String>,
    },

    /// 撤销订单（支持直接 order_id，或 qmt_live task_id 自动解析）
    QmtCancel {
        /// 订单 ID，或 qmt_live task_id
        #[arg(long = "order-id")]
        order_id: String,
    },

    /// 查询账户状态
    QmtAccount,

    /// 查询持仓
    QmtPositions,

    /// 查询资产
    QmtAsset,
}

#[derive(Subcommand, Debug)]
pub enum ExecutionQmtCommands {
    /// 查看 QMT bridge 能力状态
    Status {
        /// 追加 QMT promotion checklist
        #[arg(long)]
        checklist: bool,
    },

    /// 预览待执行 request 对应的 QMT payload
    Preview {
        /// 请求 ID
        #[arg(long = "request-id")]
        request_id: String,
    },

    /// 提交真实订单 (需要确认)
    Live {
        /// 请求 ID (frozen execution request)
        #[arg(long = "request-id")]
        request_id: String,

        /// 跳过确认提示 (危险!)
        #[arg(long)]
        yes: bool,
    },

    /// 查询订单状态或 qmt_live task 结果
    Query {
        /// 订单 ID，或 qmt_live task_id
        #[arg(long = "order-id")]
        order_id: String,
    },

    /// 从本地运行时记录构建 qmt_live 审计证据
    Audit {
        /// frozen execution request ID
        #[arg(long = "request-id")]
        request_id: Option<String>,

        /// qmt_live task_id
        #[arg(long = "task-id")]
        task_id: Option<String>,

        /// qmt_live local_submission_id
        #[arg(long = "local-submission-id")]
        local_submission_id: Option<String>,
    },

    /// 撤销订单（支持直接 order_id，或 qmt_live task_id 自动解析）
    Cancel {
        /// 订单 ID，或 qmt_live task_id
        #[arg(long = "order-id")]
        order_id: String,
    },

    /// 查询账户状态
    Account,

    /// 查询持仓
    Positions,

    /// 查询资产
    Asset,
}

#[derive(Subcommand, Debug)]
pub enum AnomalyCommands {
    /// 运行异常检测
    Run {
        /// 显示的异常股票数量
        #[arg(short, long, default_value = "20")]
        top_n: usize,

        /// K线周期（分钟）: 1, 5, 15, 30, 60
        #[arg(short, long, default_value = "15")]
        period: u32,

        /// 最小成交量过滤（手）
        #[arg(long, default_value = "10000")]
        min_volume: f64,

        /// 最小波动率过滤
        #[arg(long, default_value = "0.03")]
        min_volatility: f64,

        /// 输出格式: cli, json, csv
        #[arg(short, long, default_value = "cli")]
        output: String,

        /// Isolation Forest 树数量
        #[arg(long, default_value = "100")]
        n_estimators: usize,

        /// 历史K线数量用于特征
        #[arg(long, default_value = "7")]
        history: usize,

        /// 使用模拟数据（测试用）
        #[arg(long)]
        mock: bool,

        /// 模拟数据股票数量（仅与 --mock 一起使用）
        #[arg(long, default_value = "100")]
        mock_count: usize,
    },
}

#[derive(Subcommand, Debug)]
pub enum AlgoCommands {
    /// 创建算法任务
    Create {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 买卖方向: buy | sell
        #[arg(short, long)]
        side: String,

        /// 总数量 (股)
        #[arg(short = 'n', long)]
        quantity: i64,

        /// 算法类型: twap | vwap
        #[arg(short = 't', long, default_value = "twap")]
        algo_type: String,

        /// 执行时长 (分钟)
        #[arg(short, long, default_value = "30")]
        duration: u32,

        /// 价格限制
        #[arg(short = 'p', long)]
        price: Option<f64>,

        /// 切片数量
        #[arg(long)]
        slices: Option<u32>,

        /// 切片间隔 (秒)
        #[arg(long)]
        interval: Option<u64>,

        /// 禁用随机化
        #[arg(long)]
        no_randomize: bool,
    },

    /// 启动算法任务
    Start {
        /// 算法 ID
        #[arg(long = "algo-id")]
        algo_id: String,
    },

    /// 暂停算法任务
    Pause {
        /// 算法 ID
        #[arg(long = "algo-id")]
        algo_id: String,
    },

    /// 恢复算法任务
    Resume {
        /// 算法 ID
        #[arg(long = "algo-id")]
        algo_id: String,
    },

    /// 取消算法任务
    Cancel {
        /// 算法 ID
        #[arg(long = "algo-id")]
        algo_id: String,
    },

    /// 查看算法状态
    Status {
        /// 算法 ID
        #[arg(long = "algo-id")]
        algo_id: String,
    },

    /// 列出活跃算法
    List,

    /// 预览切片计划 (不执行)
    Plan {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 买卖方向: buy | sell
        #[arg(short, long)]
        side: String,

        /// 总数量 (股)
        #[arg(short = 'n', long)]
        quantity: i64,

        /// 算法类型: twap | vwap
        #[arg(short = 't', long, default_value = "twap")]
        algo_type: String,

        /// 执行时长 (分钟)
        #[arg(short, long, default_value = "30")]
        duration: u32,

        /// 切片数量
        #[arg(long)]
        slices: Option<u32>,

        /// 切片间隔 (秒)
        #[arg(long)]
        interval: Option<u64>,

        /// 输出格式: table | json
        #[arg(long, default_value = "table")]
        output: String,
    },
}
