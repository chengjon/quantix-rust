#[path = "commands/account.rs"]
mod account;
#[path = "commands/analysis.rs"]
mod analysis;
#[path = "commands/backtest.rs"]
mod backtest;
#[path = "commands/data.rs"]
mod data;
#[path = "commands/factor.rs"]
mod factor;
#[path = "commands/info.rs"]
mod info;
#[path = "commands/market.rs"]
mod market;
#[path = "commands/monitor.rs"]
mod monitor;
#[path = "commands/performance.rs"]
mod performance;
#[path = "commands/risk.rs"]
mod risk;
#[path = "commands/safety.rs"]
mod safety;
#[path = "commands/strategy.rs"]
mod strategy;
#[path = "commands/trade.rs"]
mod trade;

pub use account::{AccountCommands, AccountGroupCommands};
pub use analysis::{AnalyzeCommands, ScreenerCommands, TaskCommands};
pub use backtest::BacktestCommands;
pub use data::{DataCommands, DataSourceCommands, DataSourceKind, TdxApiCommands};
pub use factor::{FactorCommands, FactorOutputFormat};
pub use info::{
    AiCommands, FundamentalCommands, ImportCommands, NewsCommands, NotifyCommands,
    SentimentCommands,
};
pub use market::{
    MarketCommands, StrengthStockMetric, WatchlistCommands, WatchlistGroupCommands,
    WatchlistTagCommands,
};
pub use monitor::{
    MonitorAlertCommands, MonitorCommands, MonitorConfigCommands, MonitorDaemonCommands,
    MonitorEventCommands, MonitorServiceCommands, MonitorServiceConfigCommands, StopCommands,
};
pub use performance::PerformanceCommands;
pub use risk::{
    RiskCommands, RiskImportCommands, RiskLockCommands, RiskRebuildCommands, RiskRuleCommands,
    RiskSyncCommands,
};
pub use safety::{SafetyCommands, SafetyKillSwitchCommands};
pub use strategy::{
    StrategyCommands, StrategyConfigCommands, StrategyDaemonCommands, StrategyRequestCommands,
    StrategyServiceCommands, StrategyServiceConfigCommands, StrategySignalCommands,
};
pub use trade::{
    AlgoCommands, AnomalyCommands, ExecutionBridgeCommands, ExecutionCommands,
    ExecutionConfigCommands, ExecutionDaemonCommands, ExecutionQmtCommands, TradeCommands,
};
