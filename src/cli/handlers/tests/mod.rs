use super::*;
use crate::cli::{MonitorAlertCommands, MonitorCommands, StopCommands, TradeCommands};
use crate::core::QuantixError;
use crate::core::config::CLICKHOUSE_DB_ENV;
use crate::data::models::{AdjustType, Kline};
use crate::market::{
    BoardRankRow, BoardSortBy, BoardType, LeaderFilter, LeaderRow, MarketDataReader,
    MarketSentimentSnapshot, NorthFlowSnapshot,
};
use crate::monitor::{
    MonitorAlertStore, MonitorQuoteReader, MonitorQuoteRow, MonitorService,
    MonitorWatchlistReader, PriceAlert, PriceAlertKind, TriggeredAlert,
};
use crate::screener::DailyKlineLoader;
use crate::stop::{StopRule, StopRuleStore, StopService, StopTriggerKind};
use crate::trade::{PaperTradeState, PaperTradeStore, TradeService, TradeSide};
use crate::watchlist::WatchlistListItem;
use async_trait::async_trait;
use chrono::{NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

mod market;
mod monitor;
mod screener;
mod stop;
mod support;
mod trade;

use support::{ClickHouseDbEnvGuard, env_lock};

#[tokio::test]
async fn test_create_clickhouse_client_uses_runtime_settings() {
    let _lock = env_lock();
    let _guard = ClickHouseDbEnvGuard::capture();
    unsafe {
        std::env::set_var(CLICKHOUSE_DB_ENV, "quantix_runtime_test");
    }

    let client = create_clickhouse_client().await.unwrap();
    assert_eq!(client.database(), "quantix_runtime_test");
}

#[test]
fn test_task_add_is_explicitly_unsupported() {
    let err = ensure_task_command_supported_for_p0(&TaskCommands::Add {
        name: "demo".to_string(),
        cron: "0 * * * *".to_string(),
        command: "echo demo".to_string(),
    })
    .unwrap_err();

    assert!(matches!(err, QuantixError::Unsupported(_)));
}

#[test]
fn test_task_start_daemon_is_explicitly_unsupported() {
    let err = ensure_task_command_supported_for_p0(&TaskCommands::Start { daemon: true })
        .unwrap_err();

    assert!(matches!(err, QuantixError::Unsupported(_)));
}

#[test]
fn test_foundation_p0_task_templates_match_scheduler_templates() {
    let templates = foundation_p0_task_template_descriptions();

    assert_eq!(
        templates,
        vec![
            (
                "pre_market_check".to_string(),
                "检查盘前数据".to_string(),
                "0 8 * * 1-5".to_string()
            ),
            (
                "auction_collection".to_string(),
                "竞价数据采集".to_string(),
                "30,0 9 * * 1-5".to_string()
            ),
            (
                "market_open".to_string(),
                "开盘检查".to_string(),
                "30 9 * * 1-5".to_string()
            ),
            (
                "market_close".to_string(),
                "收盘检查".to_string(),
                "0 15 * * 1-5".to_string()
            ),
            (
                "post_market_process".to_string(),
                "盘后数据处理".to_string(),
                "30 15 * * 1-5".to_string()
            ),
            (
                "data_sync".to_string(),
                "数据同步".to_string(),
                "0 16 * * *".to_string()
            ),
        ]
    );
}
