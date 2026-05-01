use clap::{Subcommand, ValueEnum};

#[derive(Subcommand, Debug)]
pub enum DataCommands {
    /// 数据源管理
    #[command(subcommand)]
    Source(DataSourceCommands),

    /// 导入本地市场基础面快照到 ClickHouse
    ImportFundamentals {
        /// 输入 JSON 文件路径，内容为 MarketFundamentalSyncRecord 数组
        #[arg(long)]
        input: String,
    },

    /// 校验本地市场基础面 JSON 文件，不写入 ClickHouse
    ValidateFundamentals {
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
        /// 配置目录；会写入 <config-dir>/data_sources.toml
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
