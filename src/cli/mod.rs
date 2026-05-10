/// CLI 交互层
///
/// 处理命令行参数解析和交互式菜单
pub mod commands;
pub mod handlers;

// Re-export public API
pub use commands::{
    AccountCommands, AccountGroupCommands, AiCommands, AlgoCommands, AnalyzeCommands,
    AnomalyCommands, BacktestCommands, Cli, Commands, DataCommands, DataSourceCommands,
    DataSourceKind, ExecutionBridgeCommands, ExecutionCommands, ExecutionConfigCommands,
    ExecutionDaemonCommands, ExecutionQmtCommands, FactorCommands, FactorOutputFormat,
    FundamentalCommands, ImportCommands, MarketCommands, MonitorAlertCommands, MonitorCommands,
    MonitorConfigCommands, MonitorDaemonCommands, MonitorEventCommands, MonitorServiceCommands,
    MonitorServiceConfigCommands, NewsCommands, NotifyCommands, PerformanceCommands, RiskCommands,
    RiskImportCommands, RiskLockCommands, RiskRebuildCommands, RiskRuleCommands, RiskSyncCommands,
    ScreenerCommands, SentimentCommands, StopCommands, StrategyCommands, StrategyConfigCommands,
    StrategyDaemonCommands, StrategyRequestCommands, StrategyServiceCommands,
    StrategyServiceConfigCommands, StrategySignalCommands, StrengthStockMetric, TaskCommands,
    TradeCommands, WatchlistCommands, WatchlistGroupCommands, WatchlistTagCommands,
};

#[cfg(test)]
mod tests;
