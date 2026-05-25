/// CLI 交互层
///
/// 处理命令行参数解析和交互式菜单
pub mod command_types;
pub mod commands;
pub mod handlers;

// Re-export public API
pub use command_types::{
    AccountCommands, AccountGroupCommands, AiCommands, AlgoCommands, AnalyzeCommands,
    AnomalyCommands, BacktestCommands, DataCommands, DataSourceCommands, DataSourceKind,
    ExecutionBridgeCommands, ExecutionCommands, ExecutionConfigCommands, ExecutionDaemonCommands,
    ExecutionQmtCommands, FactorCommands, FactorOutputFormat, FundamentalCommands, ImportCommands,
    MarketCommands, MonitorAlertCommands, MonitorCommands, MonitorConfigCommands,
    MonitorDaemonCommands, MonitorEventCommands, MonitorServiceCommands,
    MonitorServiceConfigCommands, NewsCommands, NotifyCommands, PerformanceCommands, RiskCommands,
    RiskImportCommands, RiskLockCommands, RiskRebuildCommands, RiskRuleCommands, RiskSyncCommands,
    ScreenerCommands, SentimentCommands, StopCommands, StrategyCommands, StrategyConfigCommands,
    StrategyDaemonCommands, StrategyRequestCommands, StrategyServiceCommands,
    StrategyServiceConfigCommands, StrategySignalCommands, StrengthStockMetric, TaskCommands,
    TradeCommands, WatchlistCommands, WatchlistGroupCommands, WatchlistTagCommands,
};
pub use commands::{Cli, Commands};

#[cfg(test)]
mod tests;
