//! CLI command-tree root. This module only re-exports the subcommand
//! types from [`crate::cli::command_types`] and the dispatch root
//! ([`Cli`] / [`Commands`]) defined in [`cli`].

mod cli;

pub use self::cli::{Cli, Commands};

pub use crate::cli::command_types::{
    AccountCommands, AccountGroupCommands, AiCommands, AlgoCommands, AnalyzeCommands,
    AnomalyCommands, BacktestCommands, DataCommands, DataSourceCommands, DataSourceKind,
    ExecutionBridgeCommands, ExecutionCommands, ExecutionConfigCommands, ExecutionDaemonCommands,
    ExecutionQmtCommands, FactorCommands, FactorOutputFormat, FundamentalCommands, ImportCommands,
    MarketCommands, MonitorAlertCommands, MonitorCommands, MonitorConfigCommands,
    MonitorDaemonCommands, MonitorEventCommands, MonitorServiceCommands,
    MonitorServiceConfigCommands, NewsCommands, NotifyCommands, OpenStockCommands,
    PerformanceCommands, RiskCommands, RiskImportCommands, RiskLockCommands, RiskRebuildCommands,
    RiskRuleCommands, RiskSyncCommands, SafetyCommands, SafetyKillSwitchCommands, ScreenerCommands,
    SentimentCommands, StopCommands, StrategyCommands, StrategyConfigCommands,
    StrategyDaemonCommands, StrategyRequestCommands, StrategyServiceCommands,
    StrategyServiceConfigCommands, StrategySignalCommands, StrengthStockMetric, TaskCommands,
    TradeCommands, WatchlistCommands, WatchlistGroupCommands, WatchlistTagCommands,
};
