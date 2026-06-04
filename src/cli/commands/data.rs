use clap::{Subcommand, ValueEnum};

#[derive(Subcommand, Debug)]
pub enum DataCommands {
    /// 数据源管理
    #[command(subcommand)]
    Source(DataSourceCommands),

    /// tdx-api Docker 服务查询
    #[command(subcommand)]
    TdxApi(TdxApiCommands),

    /// 导入本地市场基础面快照到 ClickHouse
    ImportFundamentals {
        /// 输入 JSON 文件路径，内容为 MarketFundamentalSyncRecord 数组
        #[arg(long)]
        input: String,
    },

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
pub enum TdxApiCommands {
    /// 测试 tdx-api 连通性
    Health,

    /// 获取实时行情
    Quote {
        /// 股票代码 (如 600000 或 sh600000)
        #[arg(short, long)]
        code: String,
    },

    /// 获取 K 线数据
    Kline {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// K线周期: minute1, minute5, minute15, minute30, hour, day, week, month
        #[arg(short, long, default_value = "day")]
        r#type: String,

        /// 限制返回条数
        #[arg(short, long)]
        limit: Option<u32>,
    },

    /// 获取完整历史K线 (同花顺前复权)
    KlineThs {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// K线周期: day, week, month
        #[arg(short, long, default_value = "day")]
        r#type: String,
    },

    /// 获取分时数据
    Minute {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 日期 (YYYYMMDD), 默认今天
        #[arg(short, long)]
        date: Option<String>,
    },

    /// 搜索股票代码/名称
    Search {
        /// 搜索关键词
        #[arg(short, long)]
        keyword: String,
    },

    /// 查询交易日
    Workday {
        /// 日期 (YYYYMMDD)
        #[arg(short, long)]
        date: Option<String>,

        /// 前后查询数量
        #[arg(short, long, default_value = "5")]
        count: u32,
    },

    /// 查询交易日范围
    WorkdayRange {
        /// 开始日期 (YYYYMMDD)
        #[arg(long)]
        start: String,

        /// 结束日期 (YYYYMMDD)
        #[arg(long)]
        end: String,
    },

    /// N日收益计算
    Income {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 起始日期 (YYYYMMDD)
        #[arg(long)]
        start_date: String,

        /// 计算天数列表 (如 5,10,20)
        #[arg(short, long, value_delimiter = ',', default_value = "5,10,20,60,120")]
        days: Vec<i32>,
    },

    /// 获取市场涨跌统计
    MarketStats,

    /// 列出异步任务
    Tasks,

    /// 查看任务详情
    TaskInfo {
        /// 任务 ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum DataSourceCommands {
    /// 列出当前已配置的数据源
    List {
        /// 配置目录；会读取 <config-dir>/data_sources.toml 作为覆盖层
        #[arg(long, default_value = "config")]
        config_dir: String,
    },

    /// 新增或更新一个内置数据源配置
    Add {
        /// 配置目录；会写入 <config-dir>/data_sources.toml
        #[arg(long, default_value = "config")]
        config_dir: String,

        /// 数据源类型
        #[arg(long = "type", value_enum)]
        source_type: DataSourceKind,

        /// TDX 主机列表，逗号分隔
        #[arg(long, value_delimiter = ',')]
        hosts: Vec<String>,

        /// TDX 端口
        #[arg(long)]
        port: Option<u16>,

        /// TDX 超时（毫秒）
        #[arg(long)]
        timeout: Option<u64>,

        /// AkShare 基础地址
        #[arg(long)]
        base_url: Option<String>,

        /// AkShare 限流配置
        #[arg(long)]
        rate_limit: Option<u32>,
    },

    /// 设置默认数据源
    SetDefault {
        /// 配置目录；会读取 <config-dir>/data_sources.toml
        #[arg(long, default_value = "config")]
        config_dir: String,

        /// 数据源名称，目前支持 tdx / tdx-api / akshare
        #[arg(long, value_enum)]
        name: DataSourceKind,
    },

    /// 测试数据源连通性
    Test {
        /// 配置目录；会读取 <config-dir>/data_sources.toml
        #[arg(long, default_value = "config")]
        config_dir: String,

        /// 数据源名称，目前支持 tdx / tdx-api / akshare
        #[arg(long, value_enum)]
        name: DataSourceKind,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum DataSourceKind {
    Tdx,
    TdxApi,
    Akshare,
}

impl DataSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tdx => "tdx",
            Self::TdxApi => "tdx_api",
            Self::Akshare => "akshare",
        }
    }
}
