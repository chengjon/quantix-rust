//! `data` 命令族 clap 枚举与数据源类型枚举。
//!
//! 从原生 `commands/data.rs` 拆出。承载容器 [`DataCommands`]、输出格式
//! [`OutputFormat`]，并重导出三个子模块枚举 [`DataSourceCommands`]、
//! [`OpenStockCommands`]、[`DataSourceKind`]。

mod data_source_commands;
mod data_source_kind;
mod openstock_commands;

use clap::Subcommand;

/// Output format for `import-minute-all` / `import-status` summaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

/// data 命令族 clap 枚举（容器）：Source 数据源管理、OpenStock 本地校验、ImportFundamentals 基础面导入、Query 查询、Export 导出、ImportTicks 逐笔导入、ImportKlines K 线导入。
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

// Re-exported from the child modules below for use within this module
// and to preserve the crate::cli::command_types::data::* namespace.
pub use self::data_source_commands::DataSourceCommands;
pub use self::data_source_kind::DataSourceKind;
pub use self::openstock_commands::OpenStockCommands;
