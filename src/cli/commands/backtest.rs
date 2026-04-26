use clap::Subcommand;

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
pub enum BacktestCommands {
    /// 运行一次回测并保存结果
    Run {
        /// 策略名称，当前优先支持 ma_cross
        #[arg(long, default_value = "ma_cross")]
        strategy: String,

        /// 股票代码
        #[arg(long)]
        code: String,

        /// 开始日期 (YYYYMMDD)
        #[arg(long)]
        start: Option<String>,

        /// 结束日期 (YYYYMMDD)
        #[arg(long)]
        end: Option<String>,

        /// 初始资金
        #[arg(long, default_value = "100000")]
        capital: String,

        /// 手续费率
        #[arg(long = "commission-rate", default_value = "0.0003")]
        commission_rate: String,

        /// 滑点，单位 bps
        #[arg(long = "slippage-bps", default_value_t = 10)]
        slippage_bps: u32,

        /// MA 短周期
        #[arg(long = "short", default_value_t = 5)]
        short_period: usize,

        /// MA 长周期
        #[arg(long = "long", default_value_t = 20)]
        long_period: usize,

        /// 最大持仓数量
        #[arg(long = "max-positions", default_value_t = 5)]
        max_positions: usize,

        /// 单股最大持仓比例
        #[arg(long = "max-position-ratio", default_value = "0.2")]
        max_position_ratio: String,

        /// 无风险利率（年化）
        #[arg(long = "risk-free-rate", default_value = "0.03")]
        risk_free_rate: String,

        /// 限制加载 K 线条数
        #[arg(long, default_value_t = 10000)]
        limit: usize,
    },

    /// 查看已保存的回测报告
    Report {
        /// 回测报告 ID
        #[arg(long)]
        id: String,
    },

    /// 列出已保存的回测报告
    List,

    /// 对比多个已保存的回测报告
    Compare {
        /// 回测报告 ID，可重复传入
        #[arg(long = "id", required = true)]
        ids: Vec<String>,
    },
}
