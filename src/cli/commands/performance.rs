use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum PerformanceCommands {
    /// 查看已保存回测报告的绩效详情
    Report {
        /// 回测报告 ID
        #[arg(long)]
        id: String,
    },

    /// 列出可用于绩效分析的已保存回测报告
    List,

    /// 对比多个已保存回测报告的绩效指标
    Compare {
        /// 回测报告 ID，可重复传入
        #[arg(long = "id", required = true)]
        ids: Vec<String>,
    },
}
