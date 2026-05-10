use clap::{Subcommand, ValueEnum};

#[derive(Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum FactorOutputFormat {
    Table,
    Csv,
    Json,
    Parquet,
}

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
}
