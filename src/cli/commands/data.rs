use clap::{Subcommand, ValueEnum};

#[derive(Subcommand, Debug)]
pub enum DataCommands {
    /// 数据源管理
    #[command(subcommand)]
    Source(DataSourceCommands),

    /// OpenStock 本地只读校验
    #[command(name = "openstock", subcommand)]
    OpenStock(OpenStockCommands),

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

    /// 导入逐笔成交数据到 TDengine (OpenStock)
    ImportTicks {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 日期 (YYYYMMDD), 默认今天
        #[arg(short, long)]
        date: Option<String>,

        /// 实际写入 TDengine (默认 dry-run; 需配合 QUANTIX_OPENSTOCK_TICK_APPLY=yes)
        #[arg(long)]
        apply: bool,
    },

    /// 导入 K 线到 ClickHouse `kline_data` (OpenStock)
    ImportKlines {
        /// 股票代码 (sh./sz./cn. 前缀走 INDEX_KLINES, 其余走 HISTORICAL_KLINES)
        #[arg(short, long)]
        code: String,

        /// K线周期: day, week, month
        #[arg(short, long, default_value = "day")]
        r#type: String,

        /// 起始日期 (YYYY-MM-DD)
        #[arg(long)]
        start: Option<String>,

        /// 结束日期 (YYYY-MM-DD)
        #[arg(long)]
        end: Option<String>,

        /// 实际写入 ClickHouse 主表 (默认 dry-run; 需配合 QUANTIX_OPENSTOCK_KLINE_APPLY=yes)
        #[arg(long)]
        apply: bool,
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

        /// 数据源名称，目前支持 tdx / akshare
        #[arg(long, value_enum)]
        name: DataSourceKind,
    },

    /// 测试数据源连通性
    Test {
        /// 配置目录；会读取 <config-dir>/data_sources.toml
        #[arg(long, default_value = "config")]
        config_dir: String,

        /// 数据源名称，目前支持 tdx / akshare
        #[arg(long, value_enum)]
        name: DataSourceKind,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum DataSourceKind {
    Tdx,
    Akshare,
}

impl DataSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tdx => "tdx",
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

    /// 校验 STOCK_CODES / ALL_STOCKS 类目捕获载荷（dry-run，不联网，不写库）
    ValidateCodes {
        /// 已捕获的响应 JSON 文件路径；使用 `-` 从 stdin 读取
        #[arg(long)]
        payload: String,

        /// 类目：`codes` 或 `all_stocks`，缺省按 `codes` 处理
        #[arg(long)]
        kind: Option<String>,
    },

    /// 校验 TRADE_DATES / WORKDAYS 类目捕获载荷（dry-run，不联网，不写库）
    ValidateCalendar {
        /// 已捕获的响应 JSON 文件路径；使用 `-` 从 stdin 读取
        #[arg(long)]
        payload: String,

        /// 类目：`trade_dates` 或 `workdays`
        #[arg(long)]
        kind: String,
    },

    /// 校验 INDEX_KLINES 类目捕获载荷（dry-run，不联网，不写库）
    ValidateIndex {
        /// 已捕获的响应 JSON 文件路径；使用 `-` 从 stdin 读取
        #[arg(long)]
        payload: String,

        /// 请求时使用的指数代码（如 sh000001）
        #[arg(long)]
        symbol: String,

        /// 请求时使用的开始日期 (YYYY-MM-DD)
        #[arg(long)]
        start: Option<String>,

        /// 请求时使用的结束日期 (YYYY-MM-DD)
        #[arg(long)]
        end: Option<String>,
    },

    /// 实时拉取 STOCK_CODES 类目（联网，只读，不写库）
    FetchCodes,

    /// 实时拉取 TRADE_DATES 类目（联网，只读，不写库）
    ///
    /// runtime 实际接受 `start_date`/`end_date`（YYYY-MM-DD），忽略
    /// `year`。为便于使用，`--year` 仍保留并自动展开为当年 1-1..12-31
    /// 范围；与 `--start/--end` 二选一。
    FetchCalendar {
        /// 年份（如 2026），展开为 `<year>-01-01`..`<year>-12-31`
        #[arg(long, group = "range")]
        year: Option<u32>,

        /// 起始日期 (YYYY-MM-DD)，与 --end 同时使用
        #[arg(long, group = "range")]
        start: Option<String>,

        /// 结束日期 (YYYY-MM-DD)，与 --start 同时使用
        #[arg(long, group = "range")]
        end: Option<String>,
    },

    /// 实时拉取 INDEX_KLINES 类目（联网，只读，不写库）
    FetchIndex {
        /// 指数代码（如 sh000001）
        #[arg(long)]
        symbol: String,

        /// 开始日期 (YYYY-MM-DD)
        #[arg(long)]
        start: Option<String>,

        /// 结束日期 (YYYY-MM-DD)
        #[arg(long)]
        end: Option<String>,
    },

    /// 实时拉取 ALL_STOCKS 类目（baostock 全市场快照，联网，只读，不写库）
    FetchAllStocks {
        /// 可选日期 (YYYY-MM-DD)；缺省时服务端回退到最近交易日
        #[arg(long)]
        day: Option<String>,
    },

    /// 实时拉取 WORKDAYS 类目（eltdx action 驱动，联网，只读，不写库）
    FetchWorkdays {
        /// 操作类型：today / today_is_workday / is_workday / range / next_workday / previous_workday
        #[arg(long, default_value = "today")]
        action: String,

        /// YYYY-MM-DD，用于 is_workday / next_workday / previous_workday
        #[arg(long)]
        date: Option<String>,

        /// YYYY-MM-DD，range 起始（含），与 --end 同时使用
        #[arg(long)]
        start: Option<String>,

        /// YYYY-MM-DD，range 结束（含），与 --start 同时使用
        #[arg(long)]
        end: Option<String>,
    },

    /// 实时拉取多周期 K 线（/data/bars，支持 day/week/month + none/qfq/hfq，联网，只读，不写库）
    FetchKlines {
        /// 标的代码（例如 600000、sh600000）
        #[arg(long)]
        symbol: String,

        /// K 线周期：day | week | month（大小写不敏感，拒绝 daily/weekly/monthly 别名）
        #[arg(long, default_value = "day")]
        period: String,

        /// 复权类型：none | qfq | hfq（大小写不敏感）
        #[arg(long, default_value = "none")]
        adjust: String,

        /// YYYY-MM-DD，起始日期（含，可选）
        #[arg(long)]
        start: Option<String>,

        /// YYYY-MM-DD，结束日期（含，可选）
        #[arg(long)]
        end: Option<String>,
    },

    /// 拉取分钟级 K 线蜡烛 (P0.13b-1, OpenStock /data/bars period=1m|5m|15m|30m|60m)
    FetchMinuteKlines {
        #[arg(long)]
        symbol: String,

        #[arg(long, default_value = "1m")]
        period: String,

        #[arg(long)]
        date: String,

        #[arg(long, default_value = "none")]
        adjust: String,
    },
}
