/// CLI 交互层
///
/// 处理命令行参数解析和交互式菜单
pub mod commands;
pub mod handlers;

// Re-export public API
pub use commands::{
    Cli, Commands,
    AccountCommands, AccountGroupCommands,
    AiCommands, AlgoCommands, AnalyzeCommands, AnomalyCommands,
    DataCommands,
    ExecutionBridgeCommands, ExecutionCommands, ExecutionConfigCommands, ExecutionDaemonCommands,
    FundamentalCommands,
    ImportCommands,
    MarketCommands, MonitorAlertCommands, MonitorCommands, MonitorConfigCommands,
    MonitorDaemonCommands, MonitorEventCommands, MonitorServiceCommands, MonitorServiceConfigCommands,
    NewsCommands, NotifyCommands,
    RiskCommands, RiskImportCommands, RiskLockCommands, RiskRebuildCommands, RiskRuleCommands,
    RiskSyncCommands,
    ScreenerCommands, SentimentCommands, StopCommands,
    StrategyCommands, StrategyConfigCommands, StrategyDaemonCommands, StrategyRequestCommands,
    StrategyServiceCommands, StrategyServiceConfigCommands, StrategySignalCommands,
    TaskCommands, TradeCommands,
    WatchlistCommands, WatchlistGroupCommands, WatchlistTagCommands,
};
