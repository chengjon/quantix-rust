pub(crate) use super::command_types::{
    AiCommands, AlgoCommands, AnalyzeCommands, AnomalyCommands, DataCommands, DataSourceCommands,
    DataSourceKind, ExecutionBridgeCommands, ExecutionCommands, ExecutionConfigCommands,
    ExecutionDaemonCommands, ExecutionQmtCommands, FactorCommands, FactorOutputFormat,
    FundamentalCommands, ImportCommands, MarketCommands, MonitorAlertCommands, MonitorCommands,
    MonitorConfigCommands, MonitorDaemonCommands, MonitorEventCommands, MonitorServiceCommands,
    MonitorServiceConfigCommands, NewsCommands, NotifyCommands, OpenStockCommands,
    PerformanceCommands, RiskCommands, RiskLockCommands, RiskRuleCommands, ScreenerCommands,
    SentimentCommands, StopCommands, StrategyCommands, StrategyConfigCommands,
    StrategyDaemonCommands, StrategyRequestCommands, StrategyServiceCommands,
    StrategyServiceConfigCommands, StrategySignalCommands, StrengthStockMetric, TaskCommands,
    TdxApiCommands, TradeCommands, WatchlistCommands, WatchlistGroupCommands, WatchlistTagCommands,
};
use crate::core::Result;

mod account;
mod ai;
mod algo;
mod analyze_handler;
mod anomaly;
mod app_shell;
mod backtest_handler;
mod data_handler;
mod execution_handler;
mod factor;
mod fundamental;
mod import;
mod market_handler;
mod market_output;
mod monitor_handler;
mod monitor_output;
mod news;
mod notify;
mod openstock_handler;
mod performance_handler;
mod risk;
mod safety;
mod screener_handler;
mod sentiment;
mod shared_support;
mod stop_handler;
mod stop_output;
mod strategy_handler;
mod tdx_api_handler;
mod trade_handler;
mod trade_output;
mod watchlist_handler;

pub use self::account::run_account_command;
pub use self::ai::run_ai_command;
pub use self::algo::run_algo_command;
pub(crate) use self::analyze_handler::{analyze_candle_patterns, calculate_indicators};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::analyze_handler::{
    infer_tdx_code_from_day_file_path, parse_candle_spec, pattern_rows_from_day_file,
    pattern_rows_from_klines, resolve_tdx_day_file_path, sequence_references,
};
pub use self::anomaly::run_anomaly_command;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::app_shell::{
    ensure_task_command_supported_for_p0, foundation_p0_task_template_descriptions,
};
pub use self::app_shell::{
    run_analyze_command, run_data_command, run_init, run_simple_menu, run_status, run_task_command,
    run_tui_menu,
};
pub(crate) use self::backtest_handler::{
    StoredBacktestReport, read_backtest_report, read_backtest_reports,
};
pub(crate) use self::backtest_handler::{run_backtest_command, show_backtest_report};
use self::data_handler::{
    add_data_source, export_data, import_market_fundamentals, list_data_sources, query_kline_data,
    set_default_data_source, test_data_source,
};
#[allow(unused_imports)]
pub(crate) use self::execution_handler::*;
pub use self::factor::run_factor_command;
pub use self::fundamental::run_fundamental_command;
pub use self::import::run_import_command;
pub use self::import::{
    resolve_import_market_manifest_artifact, resolve_import_market_manifest_artifact_with_options,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::market_handler::MarketCommandOutput;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::market_handler::execute_market_command_with_reader;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::market_handler::execute_market_command_with_runtime;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::market_handler::execute_market_command_with_test_payloads;
pub use self::market_handler::run_market_command;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::market_handler::{
    build_leader_filter, parse_board_sort_by, parse_market_date,
};
use self::market_output::{
    print_market_board_rows, print_market_foundation_summary, print_market_leader_rows,
    print_market_overview, print_market_sentiment_snapshot, print_market_strength_report,
    print_north_flow_snapshot,
};
pub use self::monitor_handler::run_monitor_command;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::monitor_handler::{
    ConfiguredMonitorWatchlistReader, MonitorAlertAddRequest, MonitorServiceInstallerOps,
    TdxMonitorQuoteReader, build_monitor_alert_request, create_configured_monitor_runner,
    create_monitor_alert_store, evaluate_stop_rules_for_snapshot,
    execute_monitor_command_with_service, execute_monitor_command_with_stop_store,
    execute_monitor_config_command_with_store, execute_monitor_event_command_with_store,
    execute_monitor_iteration_with_runner, execute_monitor_service_command,
    execute_monitor_service_command_with_installer,
    execute_monitor_service_config_command_with_store, monitor_alert_id_to_i64,
    parse_monitor_event_type, persist_triggered_monitor_alerts, run_monitor_loop,
    validate_monitor_watchlist_command,
};
pub(crate) use self::monitor_handler::{MonitorCommandOutput, create_stop_rule_store};
use self::monitor_output::{
    build_unconfigured_monitor_service_status_summary, print_monitor_command_output,
};
pub use self::news::run_news_command;
pub use self::notify::run_notify_command;
use self::openstock_handler::{
    persist_openstock_live, shadow_rollback, shadow_verify, validate_openstock_calendar,
    validate_openstock_codes, validate_openstock_fixture, validate_openstock_index,
    validate_openstock_live,
};
pub(crate) use self::performance_handler::run_performance_command;
pub use self::risk::run_risk_command;
#[cfg(test)]
pub(crate) use self::safety::execute_safety_kill_switch_command_with_store_at;
pub(crate) use self::safety::run_safety_command;
pub(crate) use self::screener_handler::ClickHouseDailyKlineLoader;
use self::screener_handler::run_screener_command;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::screener_handler::{
    ScreenerCommandOutput, execute_screener_command_with_loader,
};
pub use self::sentiment::run_sentiment_command;
pub(crate) use self::shared_support::{
    build_avg_cost_map_from_trade_store, build_projected_buy_impact, build_risk_account_snapshot,
    build_stop_status_rows, build_trade_init_request, build_trade_order_request,
    create_clickhouse_client, create_risk_store, create_trade_store, decimal_to_f64,
    ensure_watchlist_contains_code, filter_stop_rules, format_stop_eval_state,
    load_initialized_trade_account, load_trade_quote_prices, parse_stop_history_date,
    parse_stop_history_event_type, patch_value, remap_trade_request_error,
    resolve_stop_reference_price, sync_risk_from_trade_store,
};
pub(crate) use self::stop_handler::StopCommandOutput;
pub use self::stop_handler::run_stop_command;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::stop_handler::{
    execute_stop_command_with_context, execute_stop_command_with_service,
};
use self::stop_output::print_stop_command_output;
pub(crate) use self::strategy_handler::*;
pub(crate) use self::trade_handler::TradeCommandOutput;
pub use self::trade_handler::run_trade_command;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::trade_handler::{
    execute_trade_command_with_quote_lookup, execute_trade_command_with_risk,
    execute_trade_command_with_service,
};
use self::trade_output::print_trade_command_output;
pub use self::watchlist_handler::run_watchlist_command;
pub(crate) use self::watchlist_handler::{
    create_watchlist_storage, format_tags, load_watchlist_store_for_read,
};

/// 策略命令
pub async fn run_strategy_command(cmd: StrategyCommands) -> Result<()> {
    strategy_handler::execute_strategy_command(cmd).await
}

pub async fn run_execution_command(cmd: ExecutionCommands) -> Result<()> {
    execution_handler::execute_execution_command(cmd).await
}

#[cfg(test)]
mod tests;
