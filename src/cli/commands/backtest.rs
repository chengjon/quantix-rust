use clap::Subcommand;

#[allow(clippy::large_enum_variant)]
/// backtest 命令族 clap 枚举：Run 运行回测并保存结果、List 列出历史回测、Show 查看详情、Compare 对比、Delete 删除。
#[derive(Subcommand, Debug)]
pub enum BacktestCommands {
    /// 研究侧 walk-forward 回测（fin 引擎，ma_cross 策略）
    Walkforward {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 开始日期（YYYYMMDD，与 Run 变体同口径）
        #[arg(long)]
        start: Option<String>,

        /// 结束日期（YYYYMMDD，与 Run 变体同口径）
        #[arg(long)]
        end: Option<String>,

        /// short_period 网格 min
        #[arg(long = "short-start", default_value_t = 2)]
        short_start: usize,
        /// short_period 网格 max
        #[arg(long = "short-end", default_value_t = 5)]
        short_end: usize,
        /// short_period 网格步长
        #[arg(long = "short-step", default_value_t = 1)]
        short_step: usize,

        /// long_period 网格 min
        #[arg(long = "long-start", default_value_t = 5)]
        long_start: usize,
        /// long_period 网格 max
        #[arg(long = "long-end", default_value_t = 20)]
        long_end: usize,
        /// long_period 网格步长
        #[arg(long = "long-step", default_value_t = 1)]
        long_step: usize,

        /// 训练（in-sample）窗口 bar 数
        #[arg(long = "train-window", default_value_t = 60)]
        train_window: usize,
        /// 测试（out-of-sample）窗口 bar 数
        #[arg(long = "test-window", default_value_t = 20)]
        test_window: usize,
        /// 窗口推进步长 bar 数
        #[arg(long, default_value_t = 20)]
        step: usize,

        /// 初始资金
        #[arg(long, default_value = "100000")]
        capital: String,
        /// 手续费率
        #[arg(long = "commission-rate", default_value = "0.0003")]
        commission_rate: String,
        /// 每次信号下单数量
        #[arg(long = "position-size", default_value = "100")]
        position_size: String,
        /// 稳健性判定的稳定性阈值（is_robust：stability ≥ 阈值 且 OOS Sharpe 均值 > 0）
        #[arg(long = "min-stability", default_value_t = 0.5)]
        min_stability: f64,
    },

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
