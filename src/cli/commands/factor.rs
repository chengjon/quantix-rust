use clap::{Subcommand, ValueEnum};

/// factor 输出格式：Table 终端表格、CSV 文本、JSON 结构化、Parquet 列式文件。
#[derive(Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum FactorOutputFormat {
    Table,
    Csv,
    Json,
    Parquet,
}

/// factor 命令族 clap 枚举：List 列出注册因子、Compute 批量计算、Show 查看因子定义、Validate 校验实现。
#[derive(Subcommand, Debug)]
pub enum FactorCommands {
    /// List registered factor definitions
    List {
        /// Filter by factor category
        #[arg(long)]
        category: Option<String>,

        /// Show full factor metadata
        #[arg(long)]
        verbose: bool,
    },

    /// Compute one or more factors for symbols and a date range
    Compute {
        /// CSV input file with date,symbol,open,high,low,close,volume columns
        #[arg(long)]
        input: String,

        /// Factor ID; repeat to compute multiple factors
        #[arg(long = "factor", required = true, num_args = 1..)]
        factors: Vec<String>,

        /// Stock symbol; repeat to compute multiple symbols
        #[arg(long = "symbol", required = true, num_args = 1..)]
        symbols: Vec<String>,

        /// Start date, YYYY-MM-DD
        #[arg(long)]
        start: String,

        /// End date, YYYY-MM-DD
        #[arg(long)]
        end: String,

        /// Output format; table is CLI-only display
        #[arg(long, value_enum, default_value_t = FactorOutputFormat::Table)]
        format: FactorOutputFormat,

        /// Output path for csv/json/parquet
        #[arg(long)]
        output: Option<String>,

        /// Skip first-slice factor input checks
        #[arg(long)]
        skip_checks: bool,
    },

    /// Score symbols on the latest factor date using one or more factors
    Score {
        /// CSV input file with date,symbol,open,high,low,close,volume columns
        #[arg(long)]
        input: String,

        /// Factor ID; repeat or pass multiple values to build an equal-weight score
        #[arg(long = "factor", required = true, num_args = 1..)]
        factors: Vec<String>,

        /// Stock symbol; repeat or pass multiple values to score multiple symbols
        #[arg(long = "symbol", required = true, num_args = 1..)]
        symbols: Vec<String>,

        /// Start date, YYYY-MM-DD
        #[arg(long)]
        start: String,

        /// End date, YYYY-MM-DD
        #[arg(long)]
        end: String,

        /// Output format; table is CLI-only display
        #[arg(long, value_enum, default_value_t = FactorOutputFormat::Table)]
        format: FactorOutputFormat,

        /// Output path for csv/json/parquet
        #[arg(long)]
        output: Option<String>,

        /// Keep only the highest scoring N symbols
        #[arg(long)]
        top: Option<usize>,

        /// Skip first-slice factor input checks
        #[arg(long)]
        skip_checks: bool,
    },

    /// Evaluate factor IC/IR against future returns
    Evaluate {
        /// CSV input file with date,symbol,open,high,low,close,volume columns
        #[arg(long)]
        input: String,

        /// Factor ID to evaluate
        #[arg(long)]
        factor: String,

        /// Stock symbol; repeat to evaluate multiple symbols
        #[arg(long = "symbol", required = true, num_args = 1..)]
        symbols: Vec<String>,

        /// Start date, YYYY-MM-DD
        #[arg(long)]
        start: String,

        /// End date, YYYY-MM-DD
        #[arg(long)]
        end: String,

        /// Forward return horizon in bars
        #[arg(long, default_value_t = 1)]
        horizon: usize,

        /// Output format; table is CLI-only display
        #[arg(long, value_enum, default_value_t = FactorOutputFormat::Table)]
        format: FactorOutputFormat,

        /// Output path for json
        #[arg(long)]
        output: Option<String>,

        /// Skip first-slice factor input checks
        #[arg(long)]
        skip_checks: bool,
    },
}
