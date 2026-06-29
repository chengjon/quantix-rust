use clap::{Subcommand, ValueEnum};

#[derive(Subcommand, Debug)]
pub enum DataCommands {
    /// 数据源管理
    #[command(subcommand)]
    Source(DataSourceCommands),

    /// OpenStock 本地只读校验
    #[command(name = "openstock", subcommand)]
    OpenStock(OpenStockCommands),

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

    /// 创建 K 线拉取异步任务
    PullKline {
        /// 股票代码 (逗号分隔, 如 600519,000001)
        #[arg(short, long, value_delimiter = ',')]
        codes: Vec<String>,

        /// 起始日期 (YYYYMMDD)
        #[arg(long)]
        start_date: Option<String>,

        /// 限制条数
        #[arg(long)]
        limit: Option<i32>,
    },

    /// 创建成交拉取异步任务
    PullTrade {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 起始年份
        #[arg(long)]
        start_year: Option<i32>,

        /// 结束年份
        #[arg(long)]
        end_year: Option<i32>,
    },

    /// 取消异步任务
    CancelTask {
        /// 任务 ID
        #[arg(short, long)]
        id: String,
    },

    /// 导入逐笔成交数据到 TDengine
    ImportTicks {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 日期 (YYYYMMDD), 默认今天
        #[arg(short, long)]
        date: Option<String>,
    },

    /// 导入 THS 前复权 K 线到 ClickHouse
    ImportKlines {
        /// 股票代码 (如 600519), 与 --all 互斥
        #[arg(short, long, group = "target")]
        code: Option<String>,

        /// 导入全部 A 股
        #[arg(long, group = "target")]
        all: bool,

        /// 交易所过滤 (sh/sz/bj), 仅 --all 时有效
        #[arg(long)]
        exchange: Option<String>,

        /// K线周期: day, week, month
        #[arg(short, long, default_value = "day")]
        r#type: String,

        /// 覆盖导入（忽略已有数据直接插入）
        #[arg(long)]
        force: bool,
    },

    /// 从 tdx-api 同步交易日历到本地 config/holidays.json
    SyncCalendar {
        /// 年份 (默认今年)
        #[arg(short, long)]
        year: Option<i32>,
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

#[derive(Subcommand, Debug)]
pub enum OpenStockCommands {
    /// 校验本地 OpenStock K 线 fixture
    ValidateFixture {
        /// 本地 fixture JSON 文件路径
        #[arg(long)]
        file: String,
    },

    /// 校验通过外部捕获的 OpenStock /data/bars 线上响应（dry-run，不联网，不写库）
    ValidateLive {
        /// 已捕获的响应 JSON 文件路径；使用 `-` 从 stdin 读取
        #[arg(long)]
        payload: String,

        /// 请求时使用的代码（symbol/code）
        #[arg(long)]
        symbol: String,

        /// 请求时使用的周期（必须为 daily）
        #[arg(long, default_value = "daily")]
        r#period: String,

        /// 请求时使用的开始日期 (YYYY-MM-DD)
        #[arg(long)]
        start: String,

        /// 请求时使用的结束日期 (YYYY-MM-DD)
        #[arg(long)]
        end: String,

        /// 请求时声明的 limit（如填写将与返回条数对比，检测服务端是否裁剪）
        #[arg(long)]
        limit: Option<u32>,
    },

    /// 将已校验的 live shadow 载荷写入 `quantix_shadow.openstock_daily_kline_shadow`。
    /// 默认为 dry-run；如需真正写入，必须同时传 `--apply` 与
    /// 环境变量 `QUANTIX_SHADOW_PERSIST_CONFIRM=yes`。
    PersistLive {
        /// 已捕获的响应 JSON 文件路径；使用 `-` 从 stdin 读取
        #[arg(long)]
        payload: String,

        /// 请求时使用的代码（symbol/code）
        #[arg(long)]
        symbol: String,

        /// 请求时使用的周期（必须为 daily）
        #[arg(long, default_value = "daily")]
        r#period: String,

        /// 请求时使用的开始日期 (YYYY-MM-DD)
        #[arg(long)]
        start: String,

        /// 请求时使用的结束日期 (YYYY-MM-DD)
        #[arg(long)]
        end: String,

        /// 请求时声明的 limit
        #[arg(long)]
        limit: Option<u32>,

        /// 真正执行写入；不传则只跑 dry-run gate
        #[arg(long)]
        apply: bool,
    },

    /// 按 `batch_id` 回滚（删除）shadow 行，幂等。
    ShadowRollback {
        /// `persist-live --apply` 返回的 batch_id
        #[arg(long)]
        batch_id: String,
    },

    /// 校验某 `batch_id` 在 shadow 表中的当前行数。
    ShadowVerify {
        /// `persist-live --apply` 返回的 batch_id
        #[arg(long)]
        batch_id: String,
    },
}
