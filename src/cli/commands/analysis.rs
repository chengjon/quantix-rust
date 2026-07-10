use clap::{ArgGroup, Subcommand};

/// task 命令族 clap 枚举：Add/List/Remove 定时任务管理（Foundation P0 阶段为占位，Add/Remove 不支持具体执行）。
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

/// analyze 命令族 clap 枚举：Indicators 技术指标、Pattern 蜡图形态、Report 综合分析、Backtest 策略回测、Pick 选股。
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

    /// K线形态识别
    #[command(group(
        ArgGroup::new("candle_pattern_source")
            .args(["candle", "code", "day_file"])
            .required(true)
            .multiple(false)
    ))]
    #[command(group(
        ArgGroup::new("candle_pattern_reference_mode")
            .args(["reference", "previous_close"])
            .required(true)
            .multiple(false)
    ))]
    CandlePattern {
        /// K线，格式为 o,h,l,c，可重复传入表示序列
        #[arg(long = "candle")]
        candle: Vec<String>,

        /// 股票代码，从已落库 K线读取一段历史数据
        #[arg(long)]
        code: Option<String>,

        /// 通达信根目录，例如 /mnt/d/ProgramData/tdx_20251231
        #[arg(long = "tdx-root")]
        tdx_root: Option<String>,

        /// 指定 TDX 市场目录，支持 sh/sz/bj/ds
        #[arg(long)]
        market: Option<String>,

        /// 通达信 day 文件路径，直接从源文件读取日线
        #[arg(long = "day-file")]
        day_file: Option<String>,

        /// 开始日期 (YYYYMMDD)
        #[arg(short, long)]
        start: Option<String>,

        /// 结束日期 (YYYYMMDD)
        #[arg(short, long)]
        end: Option<String>,

        /// K线周期
        #[arg(long, default_value = "1d")]
        r#type: String,

        /// 限制返回条数
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// 显式参考价 p
        #[arg(long)]
        reference: Option<String>,

        /// 使用前一根收盘价作为当前 K线参考价
        #[arg(long)]
        previous_close: bool,
    },

    /// 选股筛选
    #[command(subcommand)]
    Screener(ScreenerCommands),
}

/// screener 命令族 clap 枚举：PresetList 列出内置模板、Run 运行选股。
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
