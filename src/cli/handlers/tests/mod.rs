    use super::*;
    use crate::cli::{
        MonitorAlertCommands, MonitorCommands, MonitorConfigCommands, MonitorDaemonCommands,
        MonitorEventCommands, MonitorServiceCommands, MonitorServiceConfigCommands, StopCommands,
        StrategyServiceCommands, TradeCommands,
    };
    use crate::core::QuantixError;
    use crate::core::config::{
        CLICKHOUSE_DB_ENV, CLICKHOUSE_PASSWORD_ENV, CLICKHOUSE_URL_ENV, CLICKHOUSE_USER_ENV,
    };
    use crate::core::runtime::{EXECUTION_CONFIG_PATH_ENV, STRATEGY_CONFIG_PATH_ENV};
    use crate::data::models::{AdjustType, Kline};
    use crate::market::{
        BoardRankRow, BoardSortBy, BoardType, LeaderFilter, LeaderRow, MarketDataReader,
        MarketSentimentSnapshot, NorthFlowSnapshot,
    };
    use crate::monitor::{
        JsonMonitorConfigStore, JsonMonitorServiceConfigStore, MonitorAlertStore, MonitorEventType,
        MonitorQuoteReader, MonitorQuoteRow, MonitorRunMode, MonitorRunner, MonitorService,
        MonitorServiceConfig, MonitorServiceStatusSummary, MonitorWatchlistReader, PriceAlert,
        PriceAlertKind, SqliteMonitorAlertStore, TriggeredAlert,
    };
    use crate::screener::DailyKlineLoader;
    use crate::stop::{StopRule, StopRuleStore, StopService, StopTriggerKind};
    use crate::strategy::runtime::StrategyBarLoader;
    use crate::trade::{
        JsonPaperTradeStore, PaperTradeState, PaperTradeStore, TradeService, TradeSide,
    };
    use crate::watchlist::{WatchlistListItem, WatchlistQuoteLookup, WatchlistQuoteSnapshot};
    use crate::{execution::runtime_store::StrategyRuntimeStore, risk::JsonRiskStore};
    use async_trait::async_trait;
    use chrono::{NaiveDate, TimeZone, Utc};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, OnceLock};
    use tempfile::tempdir;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    struct ClickHouseDbEnvGuard {
        url: Option<String>,
        database: Option<String>,
        user: Option<String>,
        password: Option<String>,
    }

    impl ClickHouseDbEnvGuard {
        fn capture() -> Self {
            Self {
                url: std::env::var(CLICKHOUSE_URL_ENV).ok(),
                database: std::env::var(CLICKHOUSE_DB_ENV).ok(),
                user: std::env::var(CLICKHOUSE_USER_ENV).ok(),
                password: std::env::var(CLICKHOUSE_PASSWORD_ENV).ok(),
            }
        }
    }

    struct ExecutionConfigEnvGuard {
        execution_config_path: Option<String>,
    }

    impl ExecutionConfigEnvGuard {
        fn capture() -> Self {
            Self {
                execution_config_path: std::env::var(EXECUTION_CONFIG_PATH_ENV).ok(),
            }
        }
    }

    struct StrategyConfigEnvGuard {
        strategy_config_path: Option<String>,
    }

    impl StrategyConfigEnvGuard {
        fn capture() -> Self {
            Self {
                strategy_config_path: std::env::var(STRATEGY_CONFIG_PATH_ENV).ok(),
            }
        }
    }

    impl Drop for StrategyConfigEnvGuard {
        fn drop(&mut self) {
            match &self.strategy_config_path {
                Some(value) => unsafe { std::env::set_var(STRATEGY_CONFIG_PATH_ENV, value) },
                None => unsafe { std::env::remove_var(STRATEGY_CONFIG_PATH_ENV) },
            }
        }
    }

    impl Drop for ExecutionConfigEnvGuard {
        fn drop(&mut self) {
            match &self.execution_config_path {
                Some(value) => unsafe { std::env::set_var(EXECUTION_CONFIG_PATH_ENV, value) },
                None => unsafe { std::env::remove_var(EXECUTION_CONFIG_PATH_ENV) },
            }
        }
    }

    impl Drop for ClickHouseDbEnvGuard {
        fn drop(&mut self) {
            match &self.url {
                Some(value) => unsafe { std::env::set_var(CLICKHOUSE_URL_ENV, value) },
                None => unsafe { std::env::remove_var(CLICKHOUSE_URL_ENV) },
            }

            match &self.database {
                Some(value) => unsafe { std::env::set_var(CLICKHOUSE_DB_ENV, value) },
                None => unsafe { std::env::remove_var(CLICKHOUSE_DB_ENV) },
            }

            match &self.user {
                Some(value) => unsafe { std::env::set_var(CLICKHOUSE_USER_ENV, value) },
                None => unsafe { std::env::remove_var(CLICKHOUSE_USER_ENV) },
            }

            match &self.password {
                Some(value) => unsafe { std::env::set_var(CLICKHOUSE_PASSWORD_ENV, value) },
                None => unsafe { std::env::remove_var(CLICKHOUSE_PASSWORD_ENV) },
            }
        }
    }

    #[tokio::test]
    async fn test_create_clickhouse_client_uses_runtime_settings() {
        let _lock = env_lock();
        let _guard = ClickHouseDbEnvGuard::capture();
        unsafe {
            std::env::set_var(CLICKHOUSE_URL_ENV, "http://runtime-host:8123");
            std::env::set_var(CLICKHOUSE_DB_ENV, "quantix_runtime_test");
            std::env::set_var(CLICKHOUSE_USER_ENV, "handler_user");
            std::env::set_var(CLICKHOUSE_PASSWORD_ENV, "handler_password");
        }

        let client = create_clickhouse_client().await.unwrap();
        assert_eq!(client.database(), "quantix_runtime_test");
        assert_eq!(client.http_auth_for_test().0, "handler_user");
        assert_eq!(client.http_auth_for_test().1, "handler_password");
    }

    #[tokio::test]
    async fn test_run_execution_command_config_init_creates_config_file() {
        let _lock = env_lock();
        let _guard = ExecutionConfigEnvGuard::capture();
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("execution-config.json");
        unsafe {
            std::env::set_var(EXECUTION_CONFIG_PATH_ENV, &config_path);
        }

        run_execution_command(ExecutionCommands::Config(ExecutionConfigCommands::Init))
            .await
            .unwrap();

        let saved = std::fs::read_to_string(&config_path).unwrap();
        assert!(saved.contains("\"poll_interval_secs\": 10"));
        assert!(saved.contains("\"mode\": \"manual\""));
    }

    #[tokio::test]
    async fn test_run_strategy_command_config_init_creates_config_file() {
        let _lock = env_lock();
        let _guard = StrategyConfigEnvGuard::capture();
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("strategy-config.json");
        unsafe {
            std::env::set_var(STRATEGY_CONFIG_PATH_ENV, &config_path);
        }

        run_strategy_command(StrategyCommands::Config(StrategyConfigCommands::Init))
            .await
            .unwrap();

        let saved = std::fs::read_to_string(&config_path).unwrap();
        assert!(saved.contains("\"check_interval_secs\": 60"));
    }

    #[test]
    fn test_parse_candle_spec_parses_ohlc_values() {
        let candle = parse_candle_spec("10,12,8,10").unwrap();

        assert_eq!(candle.open, dec!(10));
        assert_eq!(candle.high, dec!(12));
        assert_eq!(candle.low, dec!(8));
        assert_eq!(candle.close, dec!(10));
    }

    #[test]
    fn test_pattern_rows_from_klines_preserve_dates_and_ohlc() {
        let rows = pattern_rows_from_klines(&[Kline {
            code: "000001".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 3, 17).unwrap(),
            open: dec!(10),
            high: dec!(12),
            low: dec!(8),
            close: dec!(10),
            volume: 100,
            amount: None,
            adjust_type: AdjustType::None,
        }]);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label, "2026-03-17");
        assert_eq!(rows[0].candle.open, dec!(10));
        assert_eq!(rows[0].candle.high, dec!(12));
        assert_eq!(rows[0].candle.low, dec!(8));
        assert_eq!(rows[0].candle.close, dec!(10));
    }

    #[test]
    fn test_sequence_references_uses_previous_close_values() {
        let candles = vec![
            CandleInput {
                open: dec!(10),
                high: dec!(10),
                low: dec!(10),
                close: dec!(10),
            },
            CandleInput {
                open: dec!(10),
                high: dec!(12),
                low: dec!(10),
                close: dec!(12),
            },
            CandleInput {
                open: dec!(12),
                high: dec!(12),
                low: dec!(8),
                close: dec!(10),
            },
        ];

        let refs = sequence_references(&candles, &ReferencePricePolicy::PreviousClose).unwrap();

        assert_eq!(refs, vec![dec!(10), dec!(12)]);
    }

    #[test]
    fn test_infer_tdx_code_from_day_file_path_extracts_six_digit_code() {
        assert_eq!(
            infer_tdx_code_from_day_file_path(
                "/mnt/d/ProgramData/tdx_20251231/vipdoc/sh/lday/sh000001.day"
            )
            .unwrap(),
            1
        );
        assert_eq!(
            infer_tdx_code_from_day_file_path("/tmp/sz300750.day").unwrap(),
            300750
        );
    }

    #[test]
    fn test_pattern_rows_from_day_file_reads_and_limits_rows() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("sh000001.day");
        let mut bytes = Vec::new();

        bytes.extend(build_day_record_bytes(
            20260315, 1000, 1100, 900, 1050, 1000.0, 200, 980,
        ));
        bytes.extend(build_day_record_bytes(
            20260316, 1050, 1200, 1000, 1180, 1200.0, 220, 1050,
        ));
        std::fs::write(&path, bytes).unwrap();

        let rows = pattern_rows_from_day_file(&path, None, None, 1).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label, "2026-03-16");
        assert_eq!(rows[0].candle.open, dec!(10.5));
        assert_eq!(rows[0].candle.high, dec!(12));
        assert_eq!(rows[0].candle.low, dec!(10));
        assert_eq!(rows[0].candle.close, dec!(11.8));
    }

    #[test]
    fn test_resolve_tdx_day_file_path_prefers_explicit_market() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let sh_dir = root.join("vipdoc").join("sh").join("lday");
        let sz_dir = root.join("vipdoc").join("sz").join("lday");
        std::fs::create_dir_all(&sh_dir).unwrap();
        std::fs::create_dir_all(&sz_dir).unwrap();
        std::fs::write(sh_dir.join("sh000001.day"), []).unwrap();
        std::fs::write(sz_dir.join("sz000001.day"), []).unwrap();

        let resolved = resolve_tdx_day_file_path(root, "000001", Some("sz")).unwrap();

        assert_eq!(resolved, sz_dir.join("sz000001.day"));
    }

    #[test]
    fn test_resolve_tdx_day_file_path_rejects_ambiguous_market() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let sh_dir = root.join("vipdoc").join("sh").join("lday");
        let sz_dir = root.join("vipdoc").join("sz").join("lday");
        std::fs::create_dir_all(&sh_dir).unwrap();
        std::fs::create_dir_all(&sz_dir).unwrap();
        std::fs::write(sh_dir.join("sh000001.day"), []).unwrap();
        std::fs::write(sz_dir.join("sz000001.day"), []).unwrap();

        let error = resolve_tdx_day_file_path(root, "000001", None).unwrap_err();

        assert!(error.to_string().contains("匹配到多个"));
    }

    fn build_day_record_bytes(
        date: u32,
        open: u32,
        high: u32,
        low: u32,
        close: u32,
        amount: f32,
        volume: u32,
        prev_close: u32,
    ) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);
        buf.extend(date.to_le_bytes());
        buf.extend(open.to_le_bytes());
        buf.extend(high.to_le_bytes());
        buf.extend(low.to_le_bytes());
        buf.extend(close.to_le_bytes());
        buf.extend(amount.to_le_bytes());
        buf.extend(volume.to_le_bytes());
        buf.extend(prev_close.to_le_bytes());
        buf
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

    fn make_kline(code: &str, day: u32, close: Decimal, volume: i64) -> Kline {
        Kline {
            code: code.to_string(),
            date: NaiveDate::from_ymd_opt(2024, 1, day).unwrap(),
            open: close,
            high: close + dec!(1),
            low: close - dec!(1),
            close,
            volume,
            amount: None,
            adjust_type: AdjustType::None,
        }
    }

    fn fixed_ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 17, 9, 30, 0).unwrap()
    }

    fn sample_run(
        symbol: &str,
        bar_end: DateTime<Utc>,
    ) -> crate::execution::models::StrategyRunRecord {
        crate::execution::models::StrategyRunRecord {
            run_id: uuid::Uuid::new_v4().to_string(),
            strategy_name: "ma_cross".to_string(),
            mode: "signal".to_string(),
            trigger: "daemon".to_string(),
            status: crate::execution::models::StrategyRunStatus::Running,
            symbol: symbol.to_string(),
            timeframe: "1d".to_string(),
            bar_end,
            started_at: fixed_ts(),
            finished_at: None,
            metadata_json: serde_json::json!({}),
        }
    }

    #[derive(Debug, Clone, Default)]
    struct FakeLoader {
        data: HashMap<String, Vec<Kline>>,
    }

    impl StrategyBarLoadTelemetry for FakeLoader {
        fn last_source(&self) -> Option<crate::strategy::StrategyBarLoadSource> {
            Some(crate::strategy::StrategyBarLoadSource {
                source_id: "test-primary".to_string(),
                fallback_used: false,
            })
        }
    }

    #[async_trait]
    impl DailyKlineLoader for FakeLoader {
        async fn load_daily_klines(
            &self,
            code: &str,
            lookback: usize,
        ) -> crate::core::Result<Vec<Kline>> {
            let mut rows = self.data.get(code).cloned().unwrap_or_default();
            if rows.len() > lookback {
                rows = rows[rows.len() - lookback..].to_vec();
            }
            Ok(rows)
        }
    }

    #[async_trait]
    impl StrategyBarLoader for FakeLoader {
        async fn load_daily_bars(
            &self,
            code: &str,
            limit: usize,
        ) -> crate::core::Result<Vec<Kline>> {
            let mut rows = self.data.get(code).cloned().unwrap_or_default();
            if rows.len() > limit {
                rows = rows[rows.len() - limit..].to_vec();
            }
            Ok(rows)
        }
    }

    #[async_trait]
    impl crate::risk::RiskBarLoader for FakeLoader {
        async fn load_daily_bars(
            &self,
            code: &str,
            limit: usize,
        ) -> crate::core::Result<Vec<Kline>> {
            let mut rows = self.data.get(code).cloned().unwrap_or_default();
            if rows.len() > limit {
                rows = rows[rows.len() - limit..].to_vec();
            }
            Ok(rows)
        }
    }

    #[tokio::test]
    async fn test_strategy_paper_requires_explicit_code() {
        let dir = tempdir().unwrap();
        let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
            .await
            .unwrap();

        let err = execute_strategy_run_with_components(
            "ma_cross",
            "paper",
            None,
            FakeLoader::default(),
            JsonPaperTradeStore::new(dir.path().join("paper_trade.json")),
            JsonRiskStore::new(dir.path().join("risk_state.json")),
            &runtime_store,
        )
        .await
        .unwrap_err();

        assert!(err.to_string().contains("--code"));
    }

    #[tokio::test]
    async fn test_strategy_paper_requires_initialized_account() {
        let dir = tempdir().unwrap();
        let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
            .await
            .unwrap();
        let loader = FakeLoader {
            data: HashMap::from([(
                "000001".to_string(),
                (1..=30)
                    .map(|day| make_kline("000001", day, dec!(10) + Decimal::from(day), 1000))
                    .collect(),
            )]),
        };

        let err = execute_strategy_run_with_components(
            "ma_cross",
            "paper",
            Some("000001".to_string()),
            loader,
            JsonPaperTradeStore::new(dir.path().join("paper_trade.json")),
            JsonRiskStore::new(dir.path().join("risk_state.json")),
            &runtime_store,
        )
        .await
        .unwrap_err();

        assert!(err.to_string().contains("trade init"));
    }

    #[tokio::test]
    async fn test_strategy_live_remains_unsupported() {
        let dir = tempdir().unwrap();
        let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
            .await
            .unwrap();

        let err = execute_strategy_run_with_components(
            "ma_cross",
            "live",
            Some("000001".to_string()),
            FakeLoader::default(),
            JsonPaperTradeStore::new(dir.path().join("paper_trade.json")),
            JsonRiskStore::new(dir.path().join("risk_state.json")),
            &runtime_store,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, QuantixError::Unsupported(_)));
    }

    #[tokio::test]
    async fn test_strategy_mock_live_returns_non_final_status() {
        let dir = tempdir().unwrap();
        let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
            .await
            .unwrap();
        let loader = FakeLoader {
            data: HashMap::from([(
                "000001".to_string(),
                [
                    dec!(10),
                    dec!(9),
                    dec!(8),
                    dec!(7),
                    dec!(6),
                    dec!(5),
                    dec!(4),
                    dec!(3),
                    dec!(2),
                    dec!(1),
                    dec!(1),
                    dec!(1),
                    dec!(1),
                    dec!(1),
                    dec!(12),
                ]
                .into_iter()
                .enumerate()
                .map(|(idx, close)| make_kline("000001", (idx + 1) as u32, close, 1000))
                .collect(),
            )]),
        };
        let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
        let trade_service = TradeService::new(trade_store.clone());
        trade_service
            .init_account(
                crate::trade::InitAccountRequest::new(Some(100000.0), None, None, None, None)
                    .unwrap(),
                fixed_ts(),
            )
            .await
            .unwrap();

        let summary = execute_strategy_run_with_components(
            "ma_cross",
            "mock_live",
            Some("000001".to_string()),
            loader,
            trade_store,
            JsonRiskStore::new(dir.path().join("risk_state.json")),
            &runtime_store,
        )
        .await
        .unwrap();

        assert_eq!(summary.mode, "mock_live");
        assert_eq!(summary.order_status, Some(OrderStatus::Accepted));
        assert!(summary.message.contains("order_status=accepted"));
    }

    #[tokio::test]
    async fn test_strategy_paper_risk_bridge_surfaces_volatility_limit_reason() {
        let dir = tempdir().unwrap();
        let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
            .await
            .unwrap();
        let loader = FakeLoader {
            data: HashMap::from([(
                "000001".to_string(),
                [
                    dec!(10),
                    dec!(9),
                    dec!(8),
                    dec!(7),
                    dec!(6),
                    dec!(5),
                    dec!(4),
                    dec!(3),
                    dec!(2),
                    dec!(1),
                    dec!(1),
                    dec!(1),
                    dec!(1),
                    dec!(1),
                    dec!(12),
                ]
                .into_iter()
                .enumerate()
                .map(|(idx, close)| make_kline("000001", (idx + 1) as u32, close, 1000))
                    .collect(),
            )]),
        };
        let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
        let trade_service = TradeService::new(trade_store.clone());
        trade_service
            .init_account(
                crate::trade::InitAccountRequest::new(Some(100000.0), None, None, None, None)
                    .unwrap(),
                fixed_ts(),
            )
            .await
            .unwrap();
        let risk_service = RiskService::with_bar_loader(
            JsonRiskStore::new(dir.path().join("risk_state.json")),
            loader.clone(),
        );
        risk_service
            .set_rule("volatility-limit", "4%", fixed_ts())
            .await
            .unwrap();

        let summary = execute_strategy_run_with_risk_service(
            "ma_cross",
            "paper",
            Some("000001".to_string()),
            loader,
            trade_store,
            risk_service,
            &runtime_store,
        )
        .await
        .unwrap();

        assert_eq!(summary.order_status, Some(OrderStatus::Rejected));

        let order = runtime_store
            .find_first_order_for_run(&summary.run_id)
            .await
            .unwrap()
            .unwrap();
        let events = runtime_store.list_order_events(&order.order_id).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "risk_rejected");
        assert!(
            events[0].details_json["reason"]
                .as_str()
                .unwrap()
                .contains("volatility-limit")
        );
    }

    #[tokio::test]
    async fn test_strategy_mock_live_risk_bridge_surfaces_volatility_limit_reason() {
        let dir = tempdir().unwrap();
        let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
            .await
            .unwrap();
        let loader = FakeLoader {
            data: HashMap::from([(
                "000001".to_string(),
                [
                    dec!(10),
                    dec!(9),
                    dec!(8),
                    dec!(7),
                    dec!(6),
                    dec!(5),
                    dec!(4),
                    dec!(3),
                    dec!(2),
                    dec!(1),
                    dec!(1),
                    dec!(1),
                    dec!(1),
                    dec!(1),
                    dec!(12),
                ]
                .into_iter()
                .enumerate()
                .map(|(idx, close)| make_kline("000001", (idx + 1) as u32, close, 1000))
                .collect(),
            )]),
        };
        let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
        let trade_service = TradeService::new(trade_store.clone());
        trade_service
            .init_account(
                crate::trade::InitAccountRequest::new(Some(100000.0), None, None, None, None)
                    .unwrap(),
                fixed_ts(),
            )
            .await
            .unwrap();
        let risk_service = RiskService::with_bar_loader(
            JsonRiskStore::new(dir.path().join("risk_state.json")),
            loader.clone(),
        );
        risk_service
            .set_rule("volatility-limit", "4%", fixed_ts())
            .await
            .unwrap();

        let summary = execute_strategy_run_with_risk_service(
            "ma_cross",
            "mock_live",
            Some("000001".to_string()),
            loader,
            trade_store.clone(),
            risk_service,
            &runtime_store,
        )
        .await
        .unwrap();

        assert_eq!(summary.order_status, Some(OrderStatus::Rejected));

        let order = runtime_store
            .find_first_order_for_run(&summary.run_id)
            .await
            .unwrap()
            .unwrap();
        let events = runtime_store.list_order_events(&order.order_id).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "risk_rejected");
        assert!(
            events[0].details_json["reason"]
                .as_str()
                .unwrap()
                .contains("volatility-limit")
        );

        let state = trade_store.load_state().await.unwrap().unwrap();
        assert!(state.account.unwrap().positions.is_empty());
    }

    #[test]
    fn test_execute_strategy_config_init_creates_default_file() {
        let dir = tempdir().unwrap();
        let store =
            crate::strategy::JsonStrategyConfigStore::new(dir.path().join("strategy-config.json"));

        let config = execute_strategy_config_init_to_store(&store).unwrap();

        assert_eq!(config.check_interval_secs, 60);
        assert!(dir.path().join("strategy-config.json").exists());
    }

    #[test]
    fn test_execute_strategy_config_show_returns_saved_config() {
        let dir = tempdir().unwrap();
        let store =
            crate::strategy::JsonStrategyConfigStore::new(dir.path().join("strategy-config.json"));
        let expected = store.load_or_create().unwrap();

        let shown = execute_strategy_config_show_from_store(&store).unwrap();

        assert_eq!(shown, expected);
    }

    #[test]
    fn test_execute_strategy_service_config_show_reports_not_configured_when_missing() {
        let dir = tempdir().unwrap();
        let store = crate::strategy::JsonStrategyServiceConfigStore::new(
            dir.path().join("strategy-service.json"),
        );

        let shown = execute_strategy_service_config_command_with_store(
            StrategyServiceConfigCommands::Show,
            &store,
        )
        .unwrap();

        assert!(shown.is_none());
    }

    #[test]
    fn test_execute_strategy_service_config_set_persists_values() {
        let dir = tempdir().unwrap();
        let binary_path = dir.path().join("quantix");
        std::fs::write(&binary_path, "#!/bin/sh\nexit 0\n").unwrap();
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();

        let store = crate::strategy::JsonStrategyServiceConfigStore::new(
            dir.path().join("strategy-service.json"),
        );

        let shown = execute_strategy_service_config_command_with_store(
            StrategyServiceConfigCommands::Set {
                quantix_bin: binary_path.display().to_string(),
                env_file: Some("/tmp/strategy.env".to_string()),
            },
            &store,
        )
        .unwrap()
        .unwrap();

        assert_eq!(shown.quantix_bin_path, binary_path);
        assert_eq!(
            shown.environment_file_path,
            Some(std::path::PathBuf::from("/tmp/strategy.env"))
        );

        let saved = store.load().unwrap();
        assert_eq!(saved.quantix_bin_path, binary_path);
        assert_eq!(
            saved.environment_file_path,
            Some(std::path::PathBuf::from("/tmp/strategy.env"))
        );
    }

    #[derive(Default)]
    struct FakeStrategyServiceInstaller {
        status_output: Option<String>,
    }

    impl StrategyServiceInstallerOps for FakeStrategyServiceInstaller {
        fn install(&self) -> Result<()> {
            Ok(())
        }

        fn uninstall(&self) -> Result<()> {
            Ok(())
        }

        fn start(&self) -> Result<()> {
            Ok(())
        }

        fn stop(&self) -> Result<()> {
            Ok(())
        }

        fn enable(&self) -> Result<()> {
            Ok(())
        }

        fn disable(&self) -> Result<()> {
            Ok(())
        }

        fn status(&self) -> Result<String> {
            Ok(self
                .status_output
                .clone()
                .unwrap_or_else(|| "installed: yes".to_string()))
        }

        fn status_summary(&self) -> Result<StrategyServiceStatusSummary> {
            Ok(StrategyServiceStatusSummary {
                installed: true,
                enabled: false,
                active: "inactive".to_string(),
                unit_path: std::path::PathBuf::from(
                    "~/.config/systemd/user/quantix-strategy.service",
                ),
                wrapper_path: std::path::PathBuf::from("~/.local/bin/quantix-strategy-run"),
                quantix_bin_path: std::path::PathBuf::from("/bin/echo"),
                environment_file_path: None,
                raw_status: None,
            })
        }
    }

    #[test]
    fn test_execute_strategy_service_install_returns_message() {
        let message = execute_strategy_service_command_with_installer(
            StrategyServiceCommands::Install,
            &FakeStrategyServiceInstaller::default(),
        )
        .unwrap();

        assert_eq!(message, "strategy service installed");
    }

    #[test]
    fn test_execute_strategy_service_status_returns_status_text() {
        let message = execute_strategy_service_command_with_installer(
            StrategyServiceCommands::Status,
            &FakeStrategyServiceInstaller {
                status_output: Some("installed: yes\nenabled: no".to_string()),
            },
        )
        .unwrap();

        assert!(message.contains("installed: yes"));
        assert!(message.contains("enabled: no"));
    }

    #[tokio::test]
    async fn test_execute_strategy_daemon_once_bootstraps_and_then_emits_signal() {
        let dir = tempdir().unwrap();
        let config_store =
            crate::strategy::JsonStrategyConfigStore::new(dir.path().join("strategy-config.json"));
        config_store.load_or_create().unwrap();
        let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
            .await
            .unwrap();
        let mut loader = FakeLoader::default();
        loader.data.insert(
            "000001".to_string(),
            vec![
                make_kline("000001", 1, dec!(10), 1000),
                make_kline("000001", 2, dec!(10), 1000),
                make_kline("000001", 3, dec!(10), 1000),
                make_kline("000001", 4, dec!(9), 1000),
                make_kline("000001", 5, dec!(9), 1000),
                make_kline("000001", 6, dec!(20), 1000),
            ],
        );

        let first = execute_strategy_daemon_run_once_with_components(
            loader.clone(),
            &config_store,
            &runtime_store,
        )
        .await
        .unwrap();
        assert!(first.is_none());
        assert_eq!(runtime_store.count_signals().await.unwrap(), 0);

        loader
            .data
            .get_mut("000001")
            .unwrap()
            .push(make_kline("000001", 7, dec!(21), 1000));

        let second =
            execute_strategy_daemon_run_once_with_components(loader, &config_store, &runtime_store)
                .await
                .unwrap();
        assert_eq!(
            second.map(|signal| signal.metadata_json["bar_source_id"].clone()),
            Some(json!("test-primary"))
        );
        assert_eq!(runtime_store.count_signals().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_execute_strategy_signal_list_approve_reject_and_request_list() {
        let dir = tempdir().unwrap();
        let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
            .await
            .unwrap();
        let run = sample_run("000001", fixed_ts());
        runtime_store.insert_run(&run).await.unwrap();

        let signal = crate::execution::models::StrategySignalRecord {
            signal_id: "signal-1".to_string(),
            strategy_instance_id: "ma_fast_5_slow_20".to_string(),
            strategy_name: "ma_cross".to_string(),
            symbol: "000001".to_string(),
            timeframe: "1d".to_string(),
            bar_end: fixed_ts(),
            signal_value: "buy".to_string(),
            signal_status: crate::execution::models::SignalStatus::New,
            approval_status: crate::execution::models::ApprovalStatus::Pending,
            run_id: run.run_id.clone(),
            metadata_json: json!({}),
            created_at: fixed_ts(),
            updated_at: fixed_ts(),
        };
        runtime_store.insert_signal(&signal).await.unwrap();

        let pending =
            execute_strategy_signal_list_with_store(&runtime_store, Some("pending"), None)
                .await
                .unwrap();
        assert_eq!(pending.len(), 1);

        let request = execute_strategy_signal_approve_with_store(
            &runtime_store,
            "signal-1",
            "paper",
            "default",
        )
        .await
        .unwrap();
        assert_eq!(request.signal_id, "signal-1");

        let requests = execute_strategy_request_list_with_store(&runtime_store, Some("pending"))
            .await
            .unwrap();
        assert_eq!(requests.len(), 1);

        let second = crate::execution::models::StrategySignalRecord {
            signal_id: "signal-2".to_string(),
            bar_end: fixed_ts() + chrono::Duration::days(1),
            ..signal
        };
        runtime_store.insert_signal(&second).await.unwrap();
        execute_strategy_signal_reject_with_store(&runtime_store, "signal-2", Some("manual"))
            .await
            .unwrap();

        let rejected = runtime_store.get_signal("signal-2").await.unwrap().unwrap();
        assert_eq!(
            rejected.approval_status,
            crate::execution::models::ApprovalStatus::Rejected
        );
    }

    #[tokio::test]
    async fn test_execute_strategy_request_execute_and_cancel() {
        let dir = tempdir().unwrap();
        let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
            .await
            .unwrap();
        let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
        let risk_store = JsonRiskStore::new(dir.path().join("risk_state.json"));

        let trade_service = TradeService::new(trade_store.clone());
        trade_service
            .init_account(
                crate::trade::InitAccountRequest::new(Some(100000.0), None, None, None, None)
                    .unwrap(),
                fixed_ts(),
            )
            .await
            .unwrap();

        let run = sample_run("000001", fixed_ts());
        runtime_store.insert_run(&run).await.unwrap();

        let signal = crate::execution::models::StrategySignalRecord {
            signal_id: "signal-request-exec".to_string(),
            strategy_instance_id: "ma_fast_5_slow_20".to_string(),
            strategy_name: "ma_cross".to_string(),
            symbol: "000001".to_string(),
            timeframe: "1d".to_string(),
            bar_end: fixed_ts(),
            signal_value: "buy".to_string(),
            signal_status: crate::execution::models::SignalStatus::New,
            approval_status: crate::execution::models::ApprovalStatus::Pending,
            run_id: run.run_id.clone(),
            metadata_json: json!({
                "market_price": "12.34",
                "signal_value": "buy",
                "execution_policy": {
                    "fixed_cash_per_buy": "10000",
                    "slippage_bps": 0
                },
                "bar_source_id": "test-primary",
                "bar_source_fallback": false
            }),
            created_at: fixed_ts(),
            updated_at: fixed_ts(),
        };
        runtime_store.insert_signal(&signal).await.unwrap();

        let request = execute_strategy_signal_approve_with_store(
            &runtime_store,
            "signal-request-exec",
            "mock_live",
            "default",
        )
        .await
        .unwrap();

        let completed = execute_strategy_request_execute_with_components(
            &runtime_store,
            &request.request_id,
            trade_store.clone(),
            risk_store.clone(),
        )
        .await
        .unwrap();
        assert_eq!(
            completed.request_status,
            crate::execution::models::ExecutionRequestStatus::Completed
        );
        assert_eq!(
            completed.payload_json["execution_result"]["order_status"],
            "accepted"
        );

        let second_signal = crate::execution::models::StrategySignalRecord {
            signal_id: "signal-request-cancel".to_string(),
            bar_end: fixed_ts() + chrono::Duration::days(1),
            ..signal
        };
        runtime_store.insert_signal(&second_signal).await.unwrap();

        let cancel_request = execute_strategy_signal_approve_with_store(
            &runtime_store,
            "signal-request-cancel",
            "paper",
            "default",
        )
        .await
        .unwrap();

        let canceled = execute_strategy_request_cancel_with_store(
            &runtime_store,
            &cancel_request.request_id,
            Some("manual cancel"),
        )
        .await
        .unwrap();
        assert_eq!(
            canceled.request_status,
            crate::execution::models::ExecutionRequestStatus::Canceled
        );
        assert_eq!(
            canceled.payload_json["cancellation"]["reason"],
            "manual cancel"
        );
    }

    #[test]
    fn test_format_strategy_approval_result_includes_target_and_status() {
        let row = crate::execution::models::ExecutionRequestRecord {
            request_id: "req-1".to_string(),
            signal_id: "signal-1".to_string(),
            target_mode: "paper".to_string(),
            target_account: "default".to_string(),
            request_status: crate::execution::models::ExecutionRequestStatus::Pending,
            approved_by: Some("cli".to_string()),
            created_at: fixed_ts(),
            updated_at: fixed_ts(),
            payload_json: json!({}),
        };

        let line = format_strategy_approval_result(&row);

        assert!(line.contains("req-1"));
        assert!(line.contains("signal=signal-1"));
        assert!(line.contains("target=paper/default"));
        assert!(line.contains("status=pending"));
    }

    #[test]
    fn test_format_strategy_rejection_result_includes_reason() {
        let row = crate::execution::models::StrategySignalRecord {
            signal_id: "signal-2".to_string(),
            strategy_instance_id: "ma_fast_5_slow_20".to_string(),
            strategy_name: "ma_cross".to_string(),
            symbol: "000001".to_string(),
            timeframe: "1d".to_string(),
            bar_end: fixed_ts(),
            signal_value: "sell".to_string(),
            signal_status: crate::execution::models::SignalStatus::New,
            approval_status: crate::execution::models::ApprovalStatus::Rejected,
            run_id: "run-2".to_string(),
            metadata_json: json!({"rejection_reason": "manual reject"}),
            created_at: fixed_ts(),
            updated_at: fixed_ts(),
        };

        let line = format_strategy_rejection_result(&row);

        assert!(line.contains("signal-2"));
        assert!(line.contains("signal_status=new"));
        assert!(line.contains("approval_status=rejected"));
        assert!(line.contains("reason=manual reject"));
    }

    #[test]
    fn test_format_strategy_request_row_includes_target_and_status() {
        let row = crate::execution::models::ExecutionRequestRecord {
            request_id: "req-2".to_string(),
            signal_id: "signal-9".to_string(),
            target_mode: "paper".to_string(),
            target_account: "swing".to_string(),
            request_status: crate::execution::models::ExecutionRequestStatus::Completed,
            approved_by: Some("cli".to_string()),
            created_at: fixed_ts(),
            updated_at: fixed_ts(),
            payload_json: json!({
                "execution_result": {
                    "order_status": "accepted",
                    "client_order_id": "req-2_000001_1"
                }
            }),
        };

        let line = format_strategy_request_row(&row);

        assert!(line.contains("req-2"));
        assert!(line.contains("signal=signal-9"));
        assert!(line.contains("target=paper/swing"));
        assert!(line.contains("status=completed"));
        assert!(line.contains("result=order_status=accepted client_order_id=req-2_000001_1"));
        assert!(line.contains("created_at=2026-03-17T09:30:00Z"));
    }

    #[test]
    fn test_format_strategy_request_detail_includes_snapshot_and_result_sections() {
        let row = crate::execution::models::ExecutionRequestRecord {
            request_id: "req-3".to_string(),
            signal_id: "signal-3".to_string(),
            target_mode: "paper".to_string(),
            target_account: "default".to_string(),
            request_status: crate::execution::models::ExecutionRequestStatus::Completed,
            approved_by: Some("cli".to_string()),
            created_at: fixed_ts(),
            updated_at: fixed_ts(),
            payload_json: json!({
                "execution_snapshot": {
                    "symbol": "000001",
                    "signal_value": "buy",
                    "order_intent": {
                        "side": "buy",
                        "requested_quantity": 800,
                        "requested_price": "12.34"
                    }
                },
                "execution_result": {
                    "run_id": "run-3",
                    "client_order_id": "req-3_000001_1",
                    "order_status": "accepted",
                    "executed_at": "2026-03-17T09:31:00Z"
                }
            }),
        };

        let detail = format_strategy_request_detail(&row, false);

        assert!(detail.contains("=== Execution Snapshot ==="));
        assert!(detail.contains("symbol: 000001"));
        assert!(detail.contains("signal: buy"));
        assert!(detail.contains("quantity: 800"));
        assert!(detail.contains("=== Execution Result ==="));
        assert!(detail.contains("run_id: run-3"));
        assert!(detail.contains("order_status: accepted"));
    }

    #[test]
    fn test_format_strategy_signal_row_includes_source_metadata() {
        let row = crate::execution::models::StrategySignalRecord {
            signal_id: "signal-1".to_string(),
            strategy_instance_id: "ma_fast_5_slow_20".to_string(),
            strategy_name: "ma_cross".to_string(),
            symbol: "000001".to_string(),
            timeframe: "1d".to_string(),
            bar_end: fixed_ts(),
            signal_value: "buy".to_string(),
            signal_status: crate::execution::models::SignalStatus::New,
            approval_status: crate::execution::models::ApprovalStatus::Pending,
            run_id: "run-1".to_string(),
            metadata_json: json!({
                "bar_source_id": "clickhouse-storage",
                "bar_source_fallback": false
            }),
            created_at: fixed_ts(),
            updated_at: fixed_ts(),
        };

        let line = format_strategy_signal_row(&row);

        assert!(line.contains("signal-1"));
        assert!(line.contains("bar_end=2026-03-17T09:30:00Z"));
        assert!(line.contains("source=clickhouse-storage"));
        assert!(line.contains("fallback=false"));
    }

    #[tokio::test]
    async fn test_execute_screener_preset_list_returns_supported_presets() {
        let output = execute_screener_command_with_loader(
            ScreenerCommands::PresetList,
            FakeLoader::default(),
            WatchlistStorage::new("/tmp/unused-screener-watchlist.json"),
        )
        .await
        .unwrap();

        match output {
            ScreenerCommandOutput::PresetList(presets) => {
                let names: Vec<&str> = presets.iter().map(|item| item.name).collect();
                assert_eq!(
                    names,
                    vec![
                        "close_above_ma",
                        "close_below_ma",
                        "rsi_gte",
                        "rsi_lte",
                        "volume_ratio_gte",
                    ]
                );
            }
            ScreenerCommandOutput::Rows(_) => panic!("expected preset list output"),
        }
    }

    #[tokio::test]
    async fn test_execute_screener_run_with_codes_returns_rows() {
        let loader = FakeLoader {
            data: HashMap::from([
                (
                    "000001".to_string(),
                    vec![
                        make_kline("000001", 1, dec!(10), 100),
                        make_kline("000001", 2, dec!(10), 100),
                        make_kline("000001", 3, dec!(10), 100),
                        make_kline("000001", 4, dec!(11), 100),
                        make_kline("000001", 5, dec!(12), 100),
                    ],
                ),
                (
                    "000002".to_string(),
                    vec![
                        make_kline("000002", 1, dec!(10), 100),
                        make_kline("000002", 2, dec!(10), 100),
                        make_kline("000002", 3, dec!(10), 100),
                        make_kline("000002", 4, dec!(12), 100),
                        make_kline("000002", 5, dec!(15), 100),
                    ],
                ),
            ]),
        };

        let output = execute_screener_command_with_loader(
            ScreenerCommands::Run {
                codes: Some("000001,000002".to_string()),
                watchlist: false,
                group: None,
                preset: vec!["close_above_ma:period=3".to_string()],
                limit: Some(1),
                sort_by: Some("score".to_string()),
            },
            loader,
            WatchlistStorage::new("/tmp/unused-screener-watchlist.json"),
        )
        .await
        .unwrap();

        match output {
            ScreenerCommandOutput::Rows(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].code, "000002");
                assert!(rows[0].matched);
            }
            ScreenerCommandOutput::PresetList(_) => panic!("expected rows output"),
        }
    }

    #[tokio::test]
    async fn test_execute_screener_run_with_watchlist_group_uses_watchlist_storage() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("watchlist.json");
        let storage = WatchlistStorage::new(&path);
        let service = WatchlistService::default();
        let mut store = storage.load_or_create().unwrap();
        service
            .create_group(&mut store, "core", Utc::now())
            .unwrap();
        service
            .add(&mut store, "000001", Some("core"), Utc::now())
            .unwrap();
        service.add(&mut store, "000002", None, Utc::now()).unwrap();
        storage.save(&store).unwrap();

        let loader = FakeLoader {
            data: HashMap::from([(
                "000001".to_string(),
                vec![
                    make_kline("000001", 1, dec!(10), 100),
                    make_kline("000001", 2, dec!(10), 100),
                    make_kline("000001", 3, dec!(10), 100),
                    make_kline("000001", 4, dec!(11), 100),
                    make_kline("000001", 5, dec!(12), 100),
                ],
            )]),
        };

        let output = execute_screener_command_with_loader(
            ScreenerCommands::Run {
                codes: None,
                watchlist: true,
                group: Some("core".to_string()),
                preset: vec!["close_above_ma:period=3".to_string()],
                limit: None,
                sort_by: None,
            },
            loader,
            storage,
        )
        .await
        .unwrap();

        match output {
            ScreenerCommandOutput::Rows(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].code, "000001");
            }
            ScreenerCommandOutput::PresetList(_) => panic!("expected rows output"),
        }
    }

    #[tokio::test]
    async fn test_execute_screener_run_rejects_invalid_preset() {
        let err = execute_screener_command_with_loader(
            ScreenerCommands::Run {
                codes: Some("000001".to_string()),
                watchlist: false,
                group: None,
                preset: vec!["unknown_rule:value=1".to_string()],
                limit: None,
                sort_by: None,
            },
            FakeLoader::default(),
            WatchlistStorage::new("/tmp/unused-screener-watchlist.json"),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, QuantixError::Other(_)));
        assert!(err.to_string().contains("未知的 preset"));
    }

    fn monitor_sample_time() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 11, 10, 30, 0).unwrap()
    }

    fn monitor_watchlist_item(code: &str, group: &str, tags: &[&str]) -> WatchlistListItem {
        WatchlistListItem {
            code: code.to_string(),
            group: group.to_string(),
            tags: tags.iter().map(|tag| tag.to_string()).collect(),
        }
    }

    fn monitor_quote_row(code: &str, last_price: f64, change_pct: f64) -> MonitorQuoteRow {
        MonitorQuoteRow {
            code: code.to_string(),
            group: String::new(),
            tags: Vec::new(),
            last_price: Some(last_price),
            change_pct: Some(change_pct),
            quote_time: Some(monitor_sample_time()),
            note: None,
        }
    }

    fn monitor_alert(id: i64, code: &str, kind: PriceAlertKind, target_price: f64) -> PriceAlert {
        PriceAlert {
            id,
            code: code.to_string(),
            kind,
            target_price,
            created_at: monitor_sample_time(),
            last_triggered_at: None,
        }
    }

    #[derive(Clone, Default)]
    struct FakeMonitorWatchlistReader {
        items: Vec<WatchlistListItem>,
    }

    #[async_trait]
    impl MonitorWatchlistReader for FakeMonitorWatchlistReader {
        async fn list_items(&self) -> Result<Vec<WatchlistListItem>> {
            Ok(self.items.clone())
        }
    }

    #[derive(Clone, Default)]
    struct FakeMonitorQuoteReader {
        rows: Vec<MonitorQuoteRow>,
    }

    #[async_trait]
    impl MonitorQuoteReader for FakeMonitorQuoteReader {
        async fn load_quotes(&self, _codes: &[String]) -> Result<Vec<MonitorQuoteRow>> {
            Ok(self.rows.clone())
        }
    }

    #[derive(Debug, Clone, Default)]
    struct FakeMonitorAlertState {
        next_id: i64,
        alerts: Vec<PriceAlert>,
        removed_ids: Vec<i64>,
    }

    #[derive(Clone, Default)]
    struct FakeMonitorAlertStore {
        state: Arc<Mutex<FakeMonitorAlertState>>,
    }

    #[derive(Debug, Clone, Default)]
    struct FakeStopRuleState {
        rules: Vec<StopRule>,
        history: Vec<crate::stop::StopHistoryEvent>,
        removed_codes: Vec<String>,
    }

    #[derive(Clone, Default)]
    struct FakeStopRuleStore {
        state: Arc<Mutex<FakeStopRuleState>>,
    }

    #[async_trait]
    impl StopRuleStore for FakeStopRuleStore {
        async fn upsert_rule(&self, rule: StopRule) -> Result<StopRule> {
            let mut state = self.state.lock().unwrap();
            if let Some(existing) = state
                .rules
                .iter_mut()
                .find(|existing| existing.code == rule.code)
            {
                *existing = rule.clone();
            } else {
                state.rules.push(rule.clone());
            }
            Ok(rule)
        }

        async fn list_rules(&self) -> Result<Vec<StopRule>> {
            Ok(self.state.lock().unwrap().rules.clone())
        }

        async fn get_rule(&self, code: &str) -> Result<Option<StopRule>> {
            Ok(self
                .state
                .lock()
                .unwrap()
                .rules
                .iter()
                .find(|rule| rule.code == code)
                .cloned())
        }

        async fn append_history(&self, _event: crate::stop::StopHistoryEvent) -> Result<()> {
            self.state.lock().unwrap().history.push(_event);
            Ok(())
        }

        async fn list_history(
            &self,
            _filter: crate::stop::StopHistoryFilter,
        ) -> Result<Vec<crate::stop::StopHistoryEvent>> {
            Ok(self.state.lock().unwrap().history.clone())
        }

        async fn remove_rule(&self, code: &str) -> Result<bool> {
            let mut state = self.state.lock().unwrap();
            let before = state.rules.len();
            state.rules.retain(|rule| rule.code != code);
            if before != state.rules.len() {
                state.removed_codes.push(code.to_string());
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    fn stop_sample_time() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 11, 12, 0, 0).unwrap()
    }

    fn stop_rule(code: &str) -> StopRule {
        StopRule {
            code: code.to_string(),
            stop_loss_price: Some(14.5),
            take_profit_price: None,
            stop_loss_pct: None,
            take_profit_pct: None,
            trailing_pct: None,
            highest_price: None,
            reference_price: None,
            last_triggered_at: None,
            created_at: stop_sample_time(),
            updated_at: stop_sample_time(),
        }
    }

    fn stop_watchlist_storage(codes: &[&str]) -> (tempfile::TempDir, WatchlistStorage) {
        let dir = tempfile::tempdir().unwrap();
        let storage = WatchlistStorage::new(dir.path().join("watchlist.json"));
        let service = WatchlistService::default();
        let mut store = storage.load_or_create().unwrap();
        for code in codes {
            service.add(&mut store, code, None, Utc::now()).unwrap();
        }
        storage.save(&store).unwrap();
        (dir, storage)
    }

    #[derive(Clone, Default)]
    struct FakePaperTradeStore {
        state: Arc<Mutex<Option<PaperTradeState>>>,
    }

    impl FakePaperTradeStore {
        fn snapshot(&self) -> Option<PaperTradeState> {
            self.state.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl PaperTradeStore for FakePaperTradeStore {
        async fn load_state(&self) -> Result<Option<PaperTradeState>> {
            Ok(self.snapshot())
        }

        async fn save_state(&self, state: &PaperTradeState) -> Result<()> {
            *self.state.lock().unwrap() = Some(state.clone());
            Ok(())
        }
    }

    fn trade_service() -> (TradeService<FakePaperTradeStore>, FakePaperTradeStore) {
        let store = FakePaperTradeStore::default();
        (TradeService::new(store.clone()), store)
    }

    #[derive(Clone, Default)]
    struct FakeTradeQuoteLookup {
        quotes: HashMap<String, WatchlistQuoteSnapshot>,
        fail: bool,
    }

    #[async_trait]
    impl WatchlistQuoteLookup for FakeTradeQuoteLookup {
        async fn lookup_quotes(
            &self,
            _codes: &[String],
        ) -> Result<HashMap<String, WatchlistQuoteSnapshot>> {
            if self.fail {
                Err(QuantixError::Other("quote lookup failed".to_string()))
            } else {
                Ok(self.quotes.clone())
            }
        }
    }

    #[tokio::test]
    async fn test_execute_trade_init_succeeds_and_returns_account_summary() {
        let (service, store) = trade_service();

        let output = execute_trade_command_with_service(
            TradeCommands::Init {
                capital: Some(1500000.0),
                commission_rate: Some(0.0003),
                commission_min: Some(3.0),
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();

        match output {
            TradeCommandOutput::AccountInitialized(account) => {
                assert_eq!(account.account_id, "default");
                assert_eq!(account.initial_capital, dec!(1500000));
                assert_eq!(account.available_cash, dec!(1500000));
                assert_eq!(account.fee_config.commission_rate, dec!(0.0003));
                assert_eq!(account.fee_config.commission_min, dec!(3));
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let state = store.snapshot().unwrap();
        assert!(state.account.is_some());
        assert!(state.trade_records.is_empty());
    }

    #[tokio::test]
    async fn test_execute_trade_reset_clears_previous_state() {
        let (service, store) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: None,
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 10.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap();

        let output = execute_trade_command_with_service(
            TradeCommands::Reset {
                capital: Some(500000.0),
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();

        match output {
            TradeCommandOutput::AccountReset(account) => {
                assert_eq!(account.initial_capital, dec!(500000));
                assert_eq!(account.available_cash, dec!(500000));
                assert!(account.positions.is_empty());
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let state = store.snapshot().unwrap();
        assert!(state.trade_records.is_empty());
        assert!(state.account.unwrap().positions.is_empty());
    }

    #[tokio::test]
    async fn test_execute_trade_buy_succeeds_and_returns_trade_summary() {
        let (service, _) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: None,
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();

        let output = execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 15.0,
                volume: 1000,
            },
            &service,
        )
        .await
        .unwrap();

        match output {
            TradeCommandOutput::TradeExecuted(record) => {
                assert_eq!(record.side, TradeSide::Buy);
                assert_eq!(record.code, "000001");
                assert_eq!(record.price, dec!(15));
                assert_eq!(record.volume, 1000);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_trade_buy_rejects_when_volatility_limit_exceeds_threshold() {
        let (service, store) = trade_service();
        let dir = tempdir().unwrap();
        let loader = FakeLoader {
            data: HashMap::from([(
                "000001".to_string(),
                (1..=15)
                    .map(|day| make_kline("000001", day, dec!(10), 1000))
                    .collect(),
            )]),
        };
        let risk_service = RiskService::with_bar_loader(
            JsonRiskStore::new(dir.path().join("risk_state.json")),
            loader,
        );

        execute_trade_command_with_risk(
            TradeCommands::Init {
                capital: None,
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
            &store,
            &risk_service,
        )
        .await
        .unwrap();

        risk_service
            .set_rule("volatility-limit", "4%", fixed_ts())
            .await
            .unwrap();

        let err = execute_trade_command_with_risk(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 15.0,
                volume: 1000,
            },
            &service,
            &store,
            &risk_service,
        )
        .await
        .unwrap_err();

        assert!(err.to_string().contains("volatility-limit"));

        let state = store.snapshot().unwrap();
        let account = state.account.unwrap();
        assert!(account.positions.is_empty());
        assert!(state.trade_records.is_empty());
    }

    #[tokio::test]
    async fn test_execute_trade_sell_succeeds_and_returns_trade_summary() {
        let (service, _) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: None,
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 15.0,
                volume: 1000,
            },
            &service,
        )
        .await
        .unwrap();

        let output = execute_trade_command_with_service(
            TradeCommands::Sell {
                code: "000001".to_string(),
                price: 16.0,
                volume: 400,
            },
            &service,
        )
        .await
        .unwrap();

        match output {
            TradeCommandOutput::TradeExecuted(record) => {
                assert_eq!(record.side, TradeSide::Sell);
                assert_eq!(record.code, "000001");
                assert_eq!(record.price, dec!(16));
                assert_eq!(record.volume, 400);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_trade_sell_ignores_volatility_limit() {
        let (service, store) = trade_service();
        let dir = tempdir().unwrap();
        let loader = FakeLoader {
            data: HashMap::from([(
                "000001".to_string(),
                (1..=15)
                    .map(|day| make_kline("000001", day, dec!(10), 1000))
                    .collect(),
            )]),
        };
        let risk_service = RiskService::with_bar_loader(
            JsonRiskStore::new(dir.path().join("risk_state.json")),
            loader,
        );

        execute_trade_command_with_risk(
            TradeCommands::Init {
                capital: None,
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
            &store,
            &risk_service,
        )
        .await
        .unwrap();

        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 15.0,
                volume: 1000,
            },
            &service,
        )
        .await
        .unwrap();

        risk_service
            .set_rule("volatility-limit", "4%", fixed_ts())
            .await
            .unwrap();

        let output = execute_trade_command_with_risk(
            TradeCommands::Sell {
                code: "000001".to_string(),
                price: 16.0,
                volume: 400,
            },
            &service,
            &store,
            &risk_service,
        )
        .await
        .unwrap();

        match output {
            TradeCommandOutput::TradeExecuted(record) => {
                assert_eq!(record.side, TradeSide::Sell);
                assert_eq!(record.code, "000001");
                assert_eq!(record.price, dec!(16));
                assert_eq!(record.volume, 400);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_trade_position_returns_current_positions() {
        let (service, _) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: None,
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "600000".to_string(),
                price: 10.0,
                volume: 200,
            },
            &service,
        )
        .await
        .unwrap();

        let output = execute_trade_command_with_service(
            TradeCommands::Position { current: false },
            &service,
        )
        .await
        .unwrap();

        match output {
            TradeCommandOutput::PositionList(positions) => {
                assert_eq!(positions.len(), 1);
                assert_eq!(positions[0].code, "600000");
                assert_eq!(positions[0].volume, 200);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_trade_cash_returns_current_snapshot() {
        let (service, _) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: Some(500000.0),
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 10.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap();

        let output = execute_trade_command_with_service(TradeCommands::Cash, &service)
            .await
            .unwrap();

        match output {
            TradeCommandOutput::Cash(snapshot) => {
                assert_eq!(snapshot.initial_capital, dec!(500000));
                assert_eq!(snapshot.available_cash, dec!(498995));
                assert_eq!(snapshot.estimated_position_value, dec!(1000));
                assert_eq!(snapshot.estimated_total_assets, dec!(499995));
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_trade_history_returns_newest_first_rows() {
        let (service, _) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: None,
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 10.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Sell {
                code: "000001".to_string(),
                price: 12.0,
                volume: 40,
            },
            &service,
        )
        .await
        .unwrap();

        let output = execute_trade_command_with_service(
            TradeCommands::History {
                code: None,
                limit: Some(10),
            },
            &service,
        )
        .await
        .unwrap();

        match output {
            TradeCommandOutput::HistoryRows(rows) => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].side, TradeSide::Sell);
                assert_eq!(rows[1].side, TradeSide::Buy);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_trade_fees_filters_by_code() {
        let (service, _) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: None,
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 10.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "600000".to_string(),
                price: 20.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap();

        let output = execute_trade_command_with_service(
            TradeCommands::Fees {
                code: Some("600000".to_string()),
                limit: Some(10),
            },
            &service,
        )
        .await
        .unwrap();

        match output {
            TradeCommandOutput::FeeRows(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].code, "600000");
                assert_eq!(rows[0].transfer_fee, dec!(0.02));
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_trade_overview_returns_booked_summary() {
        let (service, _) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: Some(500000.0),
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 10.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap();

        let output = execute_trade_command_with_service(
            TradeCommands::Overview { current: false },
            &service,
        )
        .await
        .unwrap();

        match output {
            TradeCommandOutput::Overview(overview) => {
                assert_eq!(overview.initial_capital, dec!(500000));
                assert_eq!(overview.trade_count, 1);
                assert_eq!(overview.holding_count, 1);
                assert_eq!(overview.total_buy_amount, dec!(1000));
                assert_eq!(overview.total_fee, dec!(5));
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_trade_overview_before_init_returns_user_facing_error() {
        let (service, _) = trade_service();

        let err = execute_trade_command_with_service(
            TradeCommands::Overview { current: false },
            &service,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, QuantixError::Other(_)));
        assert!(err.to_string().contains("尚未初始化"));
    }

    #[tokio::test]
    async fn test_execute_trade_position_current_uses_live_quotes_when_available() {
        let (service, _) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: None,
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 10.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap();

        let output = execute_trade_command_with_quote_lookup(
            TradeCommands::Position { current: true },
            &service,
            &FakeTradeQuoteLookup {
                quotes: HashMap::from([(
                    "000001".to_string(),
                    WatchlistQuoteSnapshot {
                        latest_price: dec!(12),
                        price_change_pct: Some(dec!(5)),
                    },
                )]),
                fail: false,
            },
        )
        .await
        .unwrap();

        match output {
            TradeCommandOutput::PositionCurrentList(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].current_price, Some(dec!(12)));
                assert_eq!(rows[0].quote_status, crate::trade::TradeQuoteStatus::Live);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_trade_position_current_degrades_when_quotes_are_partial() {
        let (service, _) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: None,
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 10.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "600000".to_string(),
                price: 20.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap();

        let output = execute_trade_command_with_quote_lookup(
            TradeCommands::Position { current: true },
            &service,
            &FakeTradeQuoteLookup {
                quotes: HashMap::from([(
                    "000001".to_string(),
                    WatchlistQuoteSnapshot {
                        latest_price: dec!(12),
                        price_change_pct: Some(dec!(5)),
                    },
                )]),
                fail: false,
            },
        )
        .await
        .unwrap();

        match output {
            TradeCommandOutput::PositionCurrentList(rows) => {
                let missing = rows.iter().find(|row| row.code == "600000").unwrap();
                assert_eq!(missing.current_price, None);
                assert_eq!(
                    missing.quote_status,
                    crate::trade::TradeQuoteStatus::Missing
                );
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_trade_overview_current_uses_live_totals_when_quotes_are_complete() {
        let (service, _) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: Some(500000.0),
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 10.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap();

        let output = execute_trade_command_with_quote_lookup(
            TradeCommands::Overview { current: true },
            &service,
            &FakeTradeQuoteLookup {
                quotes: HashMap::from([(
                    "000001".to_string(),
                    WatchlistQuoteSnapshot {
                        latest_price: dec!(12),
                        price_change_pct: Some(dec!(5)),
                    },
                )]),
                fail: false,
            },
        )
        .await
        .unwrap();

        match output {
            TradeCommandOutput::Overview(overview) => {
                assert_eq!(overview.live_position_value, Some(dec!(1200)));
                assert_eq!(overview.live_total_assets, Some(dec!(500195)));
                assert_eq!(overview.quote_coverage, Some((1, 1)));
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_trade_overview_current_withholds_live_totals_on_partial_quotes() {
        let (service, _) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: Some(500000.0),
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 10.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "600000".to_string(),
                price: 20.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap();

        let output = execute_trade_command_with_quote_lookup(
            TradeCommands::Overview { current: true },
            &service,
            &FakeTradeQuoteLookup {
                quotes: HashMap::from([(
                    "000001".to_string(),
                    WatchlistQuoteSnapshot {
                        latest_price: dec!(12),
                        price_change_pct: Some(dec!(5)),
                    },
                )]),
                fail: false,
            },
        )
        .await
        .unwrap();

        match output {
            TradeCommandOutput::Overview(overview) => {
                assert_eq!(overview.live_position_value, None);
                assert_eq!(overview.live_total_assets, None);
                assert_eq!(overview.quote_coverage, Some((1, 2)));
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_trade_overview_current_degrades_gracefully_on_quote_failure() {
        let (service, _) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: Some(500000.0),
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();
        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 10.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap();

        let output = execute_trade_command_with_quote_lookup(
            TradeCommands::Overview { current: true },
            &service,
            &FakeTradeQuoteLookup {
                quotes: HashMap::new(),
                fail: true,
            },
        )
        .await
        .unwrap();

        match output {
            TradeCommandOutput::Overview(overview) => {
                assert_eq!(overview.live_position_value, None);
                assert_eq!(overview.live_total_assets, None);
                assert_eq!(overview.quote_coverage, Some((0, 1)));
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_trade_buy_before_init_returns_user_facing_error() {
        let (service, _) = trade_service();

        let err = execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 15.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, QuantixError::Other(_)));
        assert!(err.to_string().contains("尚未初始化"));
    }

    #[tokio::test]
    async fn test_execute_trade_sell_before_init_returns_user_facing_error() {
        let (service, _) = trade_service();

        let err = execute_trade_command_with_service(
            TradeCommands::Sell {
                code: "000001".to_string(),
                price: 15.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, QuantixError::Other(_)));
        assert!(err.to_string().contains("尚未初始化"));
    }

    #[tokio::test]
    async fn test_execute_trade_buy_rejects_invalid_price_or_volume() {
        let (service, _) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: None,
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();

        let price_err = execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 0.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap_err();
        assert!(matches!(price_err, QuantixError::Other(_)));
        assert!(price_err.to_string().contains("--price"));

        let volume_err = execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 10.0,
                volume: 0,
            },
            &service,
        )
        .await
        .unwrap_err();
        assert!(matches!(volume_err, QuantixError::Other(_)));
        assert!(volume_err.to_string().contains("--volume"));
    }

    #[tokio::test]
    async fn test_execute_trade_sell_rejects_unheld_code_or_excess_volume() {
        let (service, _) = trade_service();

        execute_trade_command_with_service(
            TradeCommands::Init {
                capital: None,
                commission_rate: None,
                commission_min: None,
                stamp_duty_rate: None,
                transfer_fee_rate: None,
            },
            &service,
        )
        .await
        .unwrap();

        let missing_err = execute_trade_command_with_service(
            TradeCommands::Sell {
                code: "000001".to_string(),
                price: 10.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap_err();
        assert!(matches!(missing_err, QuantixError::Other(_)));
        assert!(missing_err.to_string().contains("未持有"));

        execute_trade_command_with_service(
            TradeCommands::Buy {
                code: "000001".to_string(),
                price: 10.0,
                volume: 100,
            },
            &service,
        )
        .await
        .unwrap();

        let excess_err = execute_trade_command_with_service(
            TradeCommands::Sell {
                code: "000001".to_string(),
                price: 10.0,
                volume: 200,
            },
            &service,
        )
        .await
        .unwrap_err();
        assert!(matches!(excess_err, QuantixError::Other(_)));
        assert!(excess_err.to_string().contains("可卖数量不足"));
    }

    #[async_trait]
    impl MonitorAlertStore for FakeMonitorAlertStore {
        async fn add_alert(
            &self,
            code: &str,
            kind: PriceAlertKind,
            target_price: f64,
            now: chrono::DateTime<Utc>,
        ) -> Result<PriceAlert> {
            let mut state = self.state.lock().unwrap();
            state.next_id += 1;
            let alert = PriceAlert {
                id: state.next_id,
                code: code.to_string(),
                kind,
                target_price,
                created_at: now,
                last_triggered_at: None,
            };
            state.alerts.push(alert.clone());
            Ok(alert)
        }

        async fn list_alerts(&self) -> Result<Vec<PriceAlert>> {
            Ok(self.state.lock().unwrap().alerts.clone())
        }

        async fn remove_alert(&self, id: i64) -> Result<bool> {
            let mut state = self.state.lock().unwrap();
            let before = state.alerts.len();
            state.alerts.retain(|alert| alert.id != id);
            if before != state.alerts.len() {
                state.removed_ids.push(id);
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn mark_triggered(
            &self,
            id: i64,
            triggered_at: chrono::DateTime<Utc>,
        ) -> Result<bool> {
            let mut state = self.state.lock().unwrap();
            let Some(alert) = state.alerts.iter_mut().find(|alert| alert.id == id) else {
                return Ok(false);
            };
            alert.last_triggered_at = Some(triggered_at);
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_execute_stop_set_loss_succeeds() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let store = FakeStopRuleStore::default();
        let service = StopService::new(store.clone());

        let output = execute_stop_command_with_service(
            StopCommands::Set {
                code: "000001".to_string(),
                loss: Some(14.5),
                profit: None,
                loss_pct: None,
                profit_pct: None,
                trailing: None,
            },
            &service,
            &storage,
        )
        .await
        .unwrap();

        match output {
            StopCommandOutput::RuleSet(rule) => {
                assert_eq!(rule.code, "000001");
                assert_eq!(rule.stop_loss_price, Some(14.5));
                assert_eq!(rule.take_profit_price, None);
                assert_eq!(rule.trailing_pct, None);
            }
            other => panic!("unexpected output: {:?}", other),
        }

        assert_eq!(store.state.lock().unwrap().rules.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_stop_set_profit_succeeds() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let service = StopService::new(FakeStopRuleStore::default());

        let output = execute_stop_command_with_service(
            StopCommands::Set {
                code: "000001".to_string(),
                loss: None,
                profit: Some(18.0),
                loss_pct: None,
                profit_pct: None,
                trailing: None,
            },
            &service,
            &storage,
        )
        .await
        .unwrap();

        match output {
            StopCommandOutput::RuleSet(rule) => {
                assert_eq!(rule.take_profit_price, Some(18.0));
                assert_eq!(rule.stop_loss_price, None);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_stop_set_trailing_succeeds() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let service = StopService::new(FakeStopRuleStore::default());

        let output = execute_stop_command_with_service(
            StopCommands::Set {
                code: "000001".to_string(),
                loss: None,
                profit: Some(18.0),
                loss_pct: None,
                profit_pct: None,
                trailing: Some(5.0),
            },
            &service,
            &storage,
        )
        .await
        .unwrap();

        match output {
            StopCommandOutput::RuleSet(rule) => {
                assert_eq!(rule.trailing_pct, Some(5.0));
                assert_eq!(rule.take_profit_price, Some(18.0));
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_stop_set_rejects_invalid_condition_combinations() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let service = StopService::new(FakeStopRuleStore::default());

        let none_err = execute_stop_command_with_service(
            StopCommands::Set {
                code: "000001".to_string(),
                loss: None,
                profit: None,
                loss_pct: None,
                profit_pct: None,
                trailing: None,
            },
            &service,
            &storage,
        )
        .await
        .unwrap_err();
        assert!(matches!(none_err, QuantixError::Other(_)));
        assert!(none_err.to_string().contains("至少需要一个条件"));

        let conflict_err = execute_stop_command_with_service(
            StopCommands::Set {
                code: "000001".to_string(),
                loss: Some(14.5),
                profit: None,
                loss_pct: None,
                profit_pct: None,
                trailing: Some(5.0),
            },
            &service,
            &storage,
        )
        .await
        .unwrap_err();
        assert!(matches!(conflict_err, QuantixError::Other(_)));
        assert!(
            conflict_err
                .to_string()
                .contains("--trailing 和 --loss/--loss-pct")
        );
    }

    #[tokio::test]
    async fn test_execute_stop_set_rejects_codes_outside_watchlist() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let service = StopService::new(FakeStopRuleStore::default());

        let err = execute_stop_command_with_service(
            StopCommands::Set {
                code: "000002".to_string(),
                loss: Some(14.5),
                profit: None,
                loss_pct: None,
                profit_pct: None,
                trailing: None,
            },
            &service,
            &storage,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, QuantixError::Other(_)));
        assert!(err.to_string().contains("不在自选池"));
    }

    #[tokio::test]
    async fn test_execute_stop_set_overwrites_existing_rule_shape() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let store = FakeStopRuleStore {
            state: Arc::new(Mutex::new(FakeStopRuleState {
                rules: vec![StopRule {
                    code: "000001".to_string(),
                    stop_loss_price: Some(14.5),
                    take_profit_price: Some(18.0),
                    stop_loss_pct: None,
                    take_profit_pct: None,
                    trailing_pct: None,
                    highest_price: Some(19.2),
                    reference_price: None,
                    last_triggered_at: Some(stop_sample_time()),
                    created_at: stop_sample_time(),
                    updated_at: stop_sample_time(),
                }],
                history: Vec::new(),
                removed_codes: Vec::new(),
            })),
        };
        let service = StopService::new(store.clone());

        let output = execute_stop_command_with_service(
            StopCommands::Set {
                code: "000001".to_string(),
                loss: None,
                profit: Some(21.0),
                loss_pct: None,
                profit_pct: None,
                trailing: Some(5.0),
            },
            &service,
            &storage,
        )
        .await
        .unwrap();

        match output {
            StopCommandOutput::RuleSet(rule) => {
                assert_eq!(rule.code, "000001");
                assert_eq!(rule.stop_loss_price, None);
                assert_eq!(rule.take_profit_price, Some(21.0));
                assert_eq!(rule.trailing_pct, Some(5.0));
                assert_eq!(rule.highest_price, None);
                assert_eq!(rule.last_triggered_at, None);
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let state = store.state.lock().unwrap();
        assert_eq!(state.rules.len(), 1);
        assert_eq!(state.rules[0].stop_loss_price, None);
        assert_eq!(state.rules[0].take_profit_price, Some(21.0));
        assert_eq!(state.rules[0].trailing_pct, Some(5.0));
        assert_eq!(state.rules[0].highest_price, None);
        assert_eq!(state.rules[0].last_triggered_at, None);
    }

    #[tokio::test]
    async fn test_execute_stop_list_returns_persisted_rules() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let store = FakeStopRuleStore {
            state: Arc::new(Mutex::new(FakeStopRuleState {
                rules: vec![stop_rule("000001")],
                history: Vec::new(),
                removed_codes: Vec::new(),
            })),
        };
        let service = StopService::new(store);

        let output = execute_stop_command_with_service(StopCommands::List, &service, &storage)
            .await
            .unwrap();

        match output {
            StopCommandOutput::RuleList(rules) => {
                assert_eq!(rules.len(), 1);
                assert_eq!(rules[0].code, "000001");
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_stop_remove_succeeds() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let store = FakeStopRuleStore {
            state: Arc::new(Mutex::new(FakeStopRuleState {
                rules: vec![stop_rule("000001")],
                history: Vec::new(),
                removed_codes: Vec::new(),
            })),
        };
        let service = StopService::new(store.clone());

        let output = execute_stop_command_with_service(
            StopCommands::Remove {
                code: "000001".to_string(),
            },
            &service,
            &storage,
        )
        .await
        .unwrap();

        match output {
            StopCommandOutput::RuleRemoved { code, removed } => {
                assert_eq!(code, "000001");
                assert!(removed);
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let state = store.state.lock().unwrap();
        assert!(state.rules.is_empty());
        assert_eq!(state.removed_codes, vec!["000001".to_string()]);
    }

    #[tokio::test]
    async fn test_execute_stop_set_loss_pct_resolves_reference_price_from_quote() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let store = FakeStopRuleStore::default();
        let service = StopService::new(store.clone());
        let trade_store = FakePaperTradeStore::default();
        let quote_lookup = FakeTradeQuoteLookup {
            quotes: HashMap::from([(
                "000001".to_string(),
                WatchlistQuoteSnapshot {
                    latest_price: dec!(15.2),
                    price_change_pct: None,
                },
            )]),
            fail: false,
        };

        let output = execute_stop_command_with_context(
            StopCommands::Set {
                code: "000001".to_string(),
                loss: None,
                profit: None,
                loss_pct: Some(5.0),
                profit_pct: None,
                trailing: None,
            },
            &service,
            &storage,
            &quote_lookup,
            &trade_store,
        )
        .await
        .unwrap();

        match output {
            StopCommandOutput::RuleSet(rule) => {
                assert_eq!(rule.stop_loss_pct, Some(5.0));
                assert_eq!(rule.reference_price, Some(15.2));
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_stop_update_applies_patch_and_clear_flags() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let store = FakeStopRuleStore {
            state: Arc::new(Mutex::new(FakeStopRuleState {
                rules: vec![StopRule {
                    code: "000001".to_string(),
                    stop_loss_price: Some(14.5),
                    take_profit_price: Some(18.0),
                    stop_loss_pct: None,
                    take_profit_pct: None,
                    trailing_pct: None,
                    highest_price: None,
                    reference_price: None,
                    last_triggered_at: None,
                    created_at: stop_sample_time(),
                    updated_at: stop_sample_time(),
                }],
                history: Vec::new(),
                removed_codes: Vec::new(),
            })),
        };
        let service = StopService::new(store);
        let trade_store = FakePaperTradeStore::default();
        let quote_lookup = FakeTradeQuoteLookup {
            quotes: HashMap::from([(
                "000001".to_string(),
                WatchlistQuoteSnapshot {
                    latest_price: dec!(15.2),
                    price_change_pct: None,
                },
            )]),
            fail: false,
        };

        let output = execute_stop_command_with_context(
            StopCommands::Update {
                code: "000001".to_string(),
                loss: None,
                profit: None,
                loss_pct: None,
                profit_pct: Some(12.0),
                trailing: None,
                clear_loss: true,
                clear_profit: true,
                clear_loss_pct: false,
                clear_profit_pct: false,
                clear_trailing: false,
            },
            &service,
            &storage,
            &quote_lookup,
            &trade_store,
        )
        .await
        .unwrap();

        match output {
            StopCommandOutput::RuleUpdated(rule) => {
                assert_eq!(rule.stop_loss_price, None);
                assert_eq!(rule.take_profit_price, None);
                assert_eq!(rule.take_profit_pct, Some(12.0));
                assert_eq!(rule.reference_price, Some(15.2));
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_stop_status_and_history_return_evaluated_rows() {
        let (_dir, storage) = stop_watchlist_storage(&["000001"]);
        let stop_store = FakeStopRuleStore {
            state: Arc::new(Mutex::new(FakeStopRuleState {
                rules: vec![StopRule {
                    code: "000001".to_string(),
                    stop_loss_price: None,
                    take_profit_price: None,
                    stop_loss_pct: Some(5.0),
                    take_profit_pct: None,
                    trailing_pct: None,
                    highest_price: None,
                    reference_price: Some(15.2),
                    last_triggered_at: None,
                    created_at: stop_sample_time(),
                    updated_at: stop_sample_time(),
                }],
                history: vec![crate::stop::StopHistoryEvent {
                    id: "hist-1".to_string(),
                    code: "000001".to_string(),
                    event_type: StopHistoryEventType::Set,
                    trigger_kind: None,
                    trigger_price: None,
                    anchor_price: Some(15.2),
                    anchor_source: Some("reference_price".to_string()),
                    snapshot_json: serde_json::json!({
                        "code": "000001",
                        "stop_loss_pct": 5.0
                    }),
                    created_at: stop_sample_time(),
                }],
                removed_codes: Vec::new(),
            })),
        };
        let service = StopService::new(stop_store.clone());
        let trade_store = FakePaperTradeStore {
            state: Arc::new(Mutex::new(Some(PaperTradeState {
                version: 1,
                account: Some(PaperTradeAccount {
                    account_id: "default".to_string(),
                    initial_capital: dec!(100000),
                    available_cash: dec!(80000),
                    fee_config: crate::trade::FeeConfig::default(),
                    positions: std::collections::BTreeMap::from([(
                        "000001".to_string(),
                        crate::trade::TradePosition {
                            code: "000001".to_string(),
                            volume: 1000,
                            avg_cost: dec!(20),
                            last_trade_price: dec!(20),
                            opened_at: stop_sample_time(),
                            updated_at: stop_sample_time(),
                        },
                    )]),
                    created_at: stop_sample_time(),
                    updated_at: stop_sample_time(),
                }),
                trade_records: Vec::new(),
            }))),
        };
        let quote_lookup = FakeTradeQuoteLookup {
            quotes: HashMap::from([(
                "000001".to_string(),
                WatchlistQuoteSnapshot {
                    latest_price: dec!(19),
                    price_change_pct: None,
                },
            )]),
            fail: false,
        };

        let status_output = execute_stop_command_with_context(
            StopCommands::Status {
                code: Some("000001".to_string()),
            },
            &service,
            &storage,
            &quote_lookup,
            &trade_store,
        )
        .await
        .unwrap();

        match status_output {
            StopCommandOutput::StatusRows(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].anchor_source, Some(crate::stop::StopAnchorSource::PositionCost));
                assert_eq!(rows[0].loss_threshold, Some(19.0));
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let history_output = execute_stop_command_with_context(
            StopCommands::History {
                code: Some("000001".to_string()),
                limit: 10,
                date: None,
                event_type: None,
            },
            &service,
            &storage,
            &quote_lookup,
            &trade_store,
        )
        .await
        .unwrap();

        match history_output {
            StopCommandOutput::HistoryRows(rows) => {
                assert!(!rows.is_empty());
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_monitor_watchlist_once_returns_rows() {
        let service = MonitorService::new(
            FakeMonitorWatchlistReader {
                items: vec![
                    monitor_watchlist_item("000001", "core", &["bank"]),
                    monitor_watchlist_item("000002", "swing", &["tech"]),
                ],
            },
            FakeMonitorQuoteReader {
                rows: vec![
                    monitor_quote_row("000001", 16.2, 1.2),
                    monitor_quote_row("000002", 21.4, 2.6),
                ],
            },
            FakeMonitorAlertStore::default(),
        );

        let output = execute_monitor_command_with_service(
            MonitorCommands::Watchlist {
                once: true,
                repeat: false,
            },
            &service,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::Watchlist {
                snapshot,
                triggered_stops,
            } => {
                assert_eq!(snapshot.rows.len(), 2);
                assert_eq!(snapshot.rows[0].code, "000001");
                assert_eq!(snapshot.rows[0].group, "core");
                assert_eq!(snapshot.rows[0].tags, vec!["bank".to_string()]);
                assert_eq!(snapshot.rows[0].last_price, Some(16.2));
                assert!(snapshot.triggered_alerts.is_empty());
                assert!(triggered_stops.is_empty());
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_monitor_watchlist_once_surfaces_triggered_alerts() {
        let store = FakeMonitorAlertStore {
            state: Arc::new(Mutex::new(FakeMonitorAlertState {
                next_id: 1,
                alerts: vec![monitor_alert(1, "000001", PriceAlertKind::Above, 16.0)],
                removed_ids: Vec::new(),
            })),
        };
        let service = MonitorService::new(
            FakeMonitorWatchlistReader {
                items: vec![monitor_watchlist_item("000001", "core", &[])],
            },
            FakeMonitorQuoteReader {
                rows: vec![monitor_quote_row("000001", 16.8, 3.2)],
            },
            store,
        );

        let output = execute_monitor_command_with_service(
            MonitorCommands::Watchlist {
                once: true,
                repeat: false,
            },
            &service,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::Watchlist {
                snapshot,
                triggered_stops,
            } => {
                assert_eq!(snapshot.rows.len(), 1);
                assert_eq!(snapshot.triggered_alerts.len(), 1);
                assert_eq!(snapshot.triggered_alerts[0].alert_id, 1);
                assert_eq!(snapshot.triggered_alerts[0].code, "000001");
                assert_eq!(snapshot.triggered_alerts[0].current_price, 16.8);
                assert!(triggered_stops.is_empty());
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_monitor_watchlist_requires_once() {
        let service = MonitorService::new(
            FakeMonitorWatchlistReader::default(),
            FakeMonitorQuoteReader::default(),
            FakeMonitorAlertStore::default(),
        );

        let err = execute_monitor_command_with_service(
            MonitorCommands::Watchlist {
                once: false,
                repeat: false,
            },
            &service,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, QuantixError::Other(_)));
        assert!(err.to_string().contains("--once"));
        assert!(err.to_string().contains("--repeat"));
    }

    #[tokio::test]
    async fn test_execute_monitor_stop_fixed_loss_triggers_from_snapshot_price() {
        let store = FakeStopRuleStore {
            state: Arc::new(Mutex::new(FakeStopRuleState {
                rules: vec![stop_rule("000001")],
                history: Vec::new(),
                removed_codes: Vec::new(),
            })),
        };
        let service = MonitorService::new(
            FakeMonitorWatchlistReader {
                items: vec![monitor_watchlist_item("000001", "core", &[])],
            },
            FakeMonitorQuoteReader {
                rows: vec![monitor_quote_row("000001", 14.2, -2.1)],
            },
            FakeMonitorAlertStore::default(),
        );

        let output = execute_monitor_command_with_stop_store(
            MonitorCommands::Watchlist {
                once: true,
                repeat: false,
            },
            &service,
            &store,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::Watchlist {
                snapshot,
                triggered_stops,
            } => {
                assert_eq!(snapshot.rows.len(), 1);
                assert_eq!(triggered_stops.len(), 1);
                assert_eq!(triggered_stops[0].kind, StopTriggerKind::Loss);
                assert_eq!(triggered_stops[0].code, "000001");
                assert_eq!(triggered_stops[0].current_price, 14.2);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_monitor_stop_fixed_profit_triggers_from_snapshot_price() {
        let mut rule = stop_rule("000001");
        rule.stop_loss_price = None;
        rule.take_profit_price = Some(18.0);
        let store = FakeStopRuleStore {
            state: Arc::new(Mutex::new(FakeStopRuleState {
                rules: vec![rule],
                history: Vec::new(),
                removed_codes: Vec::new(),
            })),
        };
        let service = MonitorService::new(
            FakeMonitorWatchlistReader {
                items: vec![monitor_watchlist_item("000001", "core", &[])],
            },
            FakeMonitorQuoteReader {
                rows: vec![monitor_quote_row("000001", 18.3, 4.8)],
            },
            FakeMonitorAlertStore::default(),
        );

        let output = execute_monitor_command_with_stop_store(
            MonitorCommands::Watchlist {
                once: true,
                repeat: false,
            },
            &service,
            &store,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::Watchlist {
                snapshot: _,
                triggered_stops,
            } => {
                assert_eq!(triggered_stops.len(), 1);
                assert_eq!(triggered_stops[0].kind, StopTriggerKind::Profit);
                assert_eq!(triggered_stops[0].threshold_price, 18.0);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_monitor_stop_trailing_updates_highest_price() {
        let mut rule = stop_rule("000001");
        rule.stop_loss_price = None;
        rule.trailing_pct = Some(5.0);
        rule.highest_price = Some(15.0);
        let store = FakeStopRuleStore {
            state: Arc::new(Mutex::new(FakeStopRuleState {
                rules: vec![rule],
                history: Vec::new(),
                removed_codes: Vec::new(),
            })),
        };
        let service = MonitorService::new(
            FakeMonitorWatchlistReader {
                items: vec![monitor_watchlist_item("000001", "core", &[])],
            },
            FakeMonitorQuoteReader {
                rows: vec![monitor_quote_row("000001", 16.8, 3.1)],
            },
            FakeMonitorAlertStore::default(),
        );

        let output = execute_monitor_command_with_stop_store(
            MonitorCommands::Watchlist {
                once: true,
                repeat: false,
            },
            &service,
            &store,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::Watchlist {
                snapshot: _,
                triggered_stops,
            } => {
                assert!(triggered_stops.is_empty());
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let state = store.state.lock().unwrap();
        assert_eq!(state.rules[0].highest_price, Some(16.8));
    }

    #[tokio::test]
    async fn test_execute_monitor_stop_trailing_triggers_after_drawdown() {
        let mut rule = stop_rule("000001");
        rule.stop_loss_price = None;
        rule.trailing_pct = Some(5.0);
        rule.highest_price = Some(20.0);
        let store = FakeStopRuleStore {
            state: Arc::new(Mutex::new(FakeStopRuleState {
                rules: vec![rule],
                history: Vec::new(),
                removed_codes: Vec::new(),
            })),
        };
        let service = MonitorService::new(
            FakeMonitorWatchlistReader {
                items: vec![monitor_watchlist_item("000001", "core", &[])],
            },
            FakeMonitorQuoteReader {
                rows: vec![monitor_quote_row("000001", 18.8, -3.4)],
            },
            FakeMonitorAlertStore::default(),
        );

        let output = execute_monitor_command_with_stop_store(
            MonitorCommands::Watchlist {
                once: true,
                repeat: false,
            },
            &service,
            &store,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::Watchlist {
                snapshot: _,
                triggered_stops,
            } => {
                assert_eq!(triggered_stops.len(), 1);
                assert_eq!(triggered_stops[0].kind, StopTriggerKind::TrailingLoss);
                assert_eq!(triggered_stops[0].threshold_price, 19.0);
                assert_eq!(triggered_stops[0].highest_price, Some(20.0));
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let state = store.state.lock().unwrap();
        assert_eq!(
            state.rules[0].last_triggered_at,
            Some(monitor_sample_time())
        );
    }

    #[tokio::test]
    async fn test_execute_monitor_stop_missing_prices_do_not_trigger() {
        let store = FakeStopRuleStore {
            state: Arc::new(Mutex::new(FakeStopRuleState {
                rules: vec![stop_rule("000001")],
                history: Vec::new(),
                removed_codes: Vec::new(),
            })),
        };
        let service = MonitorService::new(
            FakeMonitorWatchlistReader {
                items: vec![monitor_watchlist_item("000001", "core", &[])],
            },
            FakeMonitorQuoteReader {
                rows: vec![MonitorQuoteRow {
                    code: "000001".to_string(),
                    group: String::new(),
                    tags: Vec::new(),
                    last_price: None,
                    change_pct: None,
                    quote_time: Some(monitor_sample_time()),
                    note: Some("quote unavailable".to_string()),
                }],
            },
            FakeMonitorAlertStore::default(),
        );

        let output = execute_monitor_command_with_stop_store(
            MonitorCommands::Watchlist {
                once: true,
                repeat: false,
            },
            &service,
            &store,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::Watchlist {
                snapshot: _,
                triggered_stops,
            } => {
                assert!(triggered_stops.is_empty());
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let state = store.state.lock().unwrap();
        assert_eq!(state.rules[0].highest_price, None);
        assert_eq!(state.rules[0].last_triggered_at, None);
    }

    #[tokio::test]
    async fn test_execute_monitor_alert_add_above_succeeds() {
        let store = FakeMonitorAlertStore::default();
        let service = MonitorService::new(
            FakeMonitorWatchlistReader::default(),
            FakeMonitorQuoteReader::default(),
            store.clone(),
        );

        let output = execute_monitor_command_with_service(
            MonitorCommands::Alert(MonitorAlertCommands::Add {
                code: "000001".to_string(),
                above: Some(16.0),
                below: None,
            }),
            &service,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::AlertAdded(alert) => {
                assert_eq!(alert.code, "000001");
                assert_eq!(alert.kind, PriceAlertKind::Above);
                assert_eq!(alert.target_price, 16.0);
            }
            other => panic!("unexpected output: {:?}", other),
        }

        assert_eq!(store.state.lock().unwrap().alerts.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_monitor_alert_add_below_succeeds() {
        let store = FakeMonitorAlertStore::default();
        let service = MonitorService::new(
            FakeMonitorWatchlistReader::default(),
            FakeMonitorQuoteReader::default(),
            store.clone(),
        );

        let output = execute_monitor_command_with_service(
            MonitorCommands::Alert(MonitorAlertCommands::Add {
                code: "000001".to_string(),
                above: None,
                below: Some(15.0),
            }),
            &service,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::AlertAdded(alert) => {
                assert_eq!(alert.code, "000001");
                assert_eq!(alert.kind, PriceAlertKind::Below);
                assert_eq!(alert.target_price, 15.0);
            }
            other => panic!("unexpected output: {:?}", other),
        }

        assert_eq!(store.state.lock().unwrap().alerts.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_monitor_alert_list_returns_persisted_rows() {
        let service = MonitorService::new(
            FakeMonitorWatchlistReader::default(),
            FakeMonitorQuoteReader::default(),
            FakeMonitorAlertStore {
                state: Arc::new(Mutex::new(FakeMonitorAlertState {
                    next_id: 2,
                    alerts: vec![
                        monitor_alert(1, "000001", PriceAlertKind::Above, 16.0),
                        monitor_alert(2, "000002", PriceAlertKind::Below, 15.0),
                    ],
                    removed_ids: Vec::new(),
                })),
            },
        );

        let output = execute_monitor_command_with_service(
            MonitorCommands::Alert(MonitorAlertCommands::List),
            &service,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::AlertList(alerts) => {
                assert_eq!(alerts.len(), 2);
                assert_eq!(alerts[0].code, "000001");
                assert_eq!(alerts[1].kind, PriceAlertKind::Below);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_monitor_alert_remove_succeeds() {
        let store = FakeMonitorAlertStore {
            state: Arc::new(Mutex::new(FakeMonitorAlertState {
                next_id: 1,
                alerts: vec![monitor_alert(1, "000001", PriceAlertKind::Above, 16.0)],
                removed_ids: Vec::new(),
            })),
        };
        let service = MonitorService::new(
            FakeMonitorWatchlistReader::default(),
            FakeMonitorQuoteReader::default(),
            store.clone(),
        );

        let output = execute_monitor_command_with_service(
            MonitorCommands::Alert(MonitorAlertCommands::Remove { id: 1 }),
            &service,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::AlertRemoved { id, removed } => {
                assert_eq!(id, 1);
                assert!(removed);
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let state = store.state.lock().unwrap();
        assert!(state.alerts.is_empty());
        assert_eq!(state.removed_ids, vec![1]);
    }

    #[tokio::test]
    async fn test_execute_monitor_alert_add_rejects_invalid_threshold_combinations() {
        let service = MonitorService::new(
            FakeMonitorWatchlistReader::default(),
            FakeMonitorQuoteReader::default(),
            FakeMonitorAlertStore::default(),
        );

        let both_err = execute_monitor_command_with_service(
            MonitorCommands::Alert(MonitorAlertCommands::Add {
                code: "000001".to_string(),
                above: Some(16.0),
                below: Some(15.0),
            }),
            &service,
        )
        .await
        .unwrap_err();
        assert!(matches!(both_err, QuantixError::Other(_)));
        assert!(both_err.to_string().contains("必须且只能指定"));

        let none_err = execute_monitor_command_with_service(
            MonitorCommands::Alert(MonitorAlertCommands::Add {
                code: "000001".to_string(),
                above: None,
                below: None,
            }),
            &service,
        )
        .await
        .unwrap_err();
        assert!(matches!(none_err, QuantixError::Other(_)));
        assert!(none_err.to_string().contains("必须且只能指定"));
    }

    #[tokio::test]
    async fn test_execute_monitor_persist_triggered_alerts_falls_back_to_observed_time() {
        let store = FakeMonitorAlertStore {
            state: Arc::new(Mutex::new(FakeMonitorAlertState {
                next_id: 1,
                alerts: vec![monitor_alert(1, "000001", PriceAlertKind::Above, 16.0)],
                removed_ids: Vec::new(),
            })),
        };
        let observed_at = Utc.with_ymd_and_hms(2026, 3, 11, 10, 31, 0).unwrap();
        let snapshot = MonitorWatchlistSnapshot {
            rows: Vec::new(),
            triggered_alerts: vec![TriggeredAlert {
                alert_id: 1,
                code: "000001".to_string(),
                kind: PriceAlertKind::Above,
                target_price: 16.0,
                current_price: 16.8,
                triggered_at: None,
            }],
            warnings: Vec::new(),
        };

        persist_triggered_monitor_alerts(&store, &snapshot, observed_at)
            .await
            .unwrap();

        let alerts = store.state.lock().unwrap().alerts.clone();
        assert_eq!(alerts[0].last_triggered_at, Some(observed_at));
    }

    #[tokio::test]
    async fn test_execute_monitor_persist_triggered_alerts_preserves_snapshot_time() {
        let store = FakeMonitorAlertStore {
            state: Arc::new(Mutex::new(FakeMonitorAlertState {
                next_id: 1,
                alerts: vec![monitor_alert(1, "000001", PriceAlertKind::Above, 16.0)],
                removed_ids: Vec::new(),
            })),
        };
        let observed_at = Utc.with_ymd_and_hms(2026, 3, 11, 10, 31, 0).unwrap();
        let snapshot = MonitorWatchlistSnapshot {
            rows: Vec::new(),
            triggered_alerts: vec![TriggeredAlert {
                alert_id: 1,
                code: "000001".to_string(),
                kind: PriceAlertKind::Above,
                target_price: 16.0,
                current_price: 16.8,
                triggered_at: Some(monitor_sample_time()),
            }],
            warnings: Vec::new(),
        };

        persist_triggered_monitor_alerts(&store, &snapshot, observed_at)
            .await
            .unwrap();

        let alerts = store.state.lock().unwrap().alerts.clone();
        assert_eq!(alerts[0].last_triggered_at, Some(monitor_sample_time()));
    }

    #[test]
    fn test_execute_monitor_config_show_returns_default_config() {
        let dir = tempdir().unwrap();
        let store = JsonMonitorConfigStore::new(dir.path().join("monitor-config.json"));

        let output =
            execute_monitor_config_command_with_store(MonitorConfigCommands::Show, &store).unwrap();

        match output {
            MonitorCommandOutput::Config(config) => {
                assert_eq!(config.interval_seconds, 30);
                assert_eq!(config.watchlist_group, None);
                assert!(config.persist_events);
                assert_eq!(config.max_event_history, 1000);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[test]
    fn test_execute_monitor_config_set_updates_persisted_values() {
        let dir = tempdir().unwrap();
        let store = JsonMonitorConfigStore::new(dir.path().join("monitor-config.json"));

        let output = execute_monitor_config_command_with_store(
            MonitorConfigCommands::Set {
                interval_seconds: Some(15),
                group: None,
                persist_events: None,
            },
            &store,
        )
        .unwrap();

        match output {
            MonitorCommandOutput::Config(config) => {
                assert_eq!(config.interval_seconds, 15);
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let reloaded = store.load_or_create().unwrap();
        assert_eq!(reloaded.interval_seconds, 15);
    }

    #[tokio::test]
    async fn test_execute_monitor_event_list_returns_filtered_rows() {
        let dir = tempdir().unwrap();
        let store = SqliteMonitorAlertStore::new(dir.path().join("alerts.db"))
            .await
            .unwrap();
        store
            .record_event_edge(
                "price_alert",
                "price_alert:000001",
                true,
                Some(crate::monitor::NewMonitorEvent {
                    event_time: monitor_sample_time(),
                    event_type: MonitorEventType::PriceAlert,
                    code: "000001".to_string(),
                    price: Some(16.2),
                    message: "000001 triggered".to_string(),
                    source_type: "price_alert".to_string(),
                    source_key: "price_alert:000001".to_string(),
                    observed_at: Some(monitor_sample_time()),
                    run_mode: MonitorRunMode::Daemon,
                }),
                1000,
            )
            .await
            .unwrap();

        let output = execute_monitor_event_command_with_store(
            MonitorEventCommands::List {
                limit: 10,
                code: Some("000001".to_string()),
                event_type: Some("price-alert".to_string()),
            },
            &store,
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::EventList(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].code, "000001");
                assert_eq!(rows[0].event_type, MonitorEventType::PriceAlert);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_monitor_repeat_uses_runner_in_foreground_mode() {
        let dir = tempdir().unwrap();
        let runner = MonitorRunner::new(
            FakeMonitorWatchlistReader {
                items: vec![monitor_watchlist_item("000001", "core", &[])],
            },
            FakeMonitorQuoteReader {
                rows: vec![monitor_quote_row("000001", 16.8, 3.2)],
            },
            SqliteMonitorAlertStore::new(dir.path().join("alerts.db"))
                .await
                .unwrap(),
            FakeStopRuleStore::default(),
            FakePaperTradeStore::default(),
        );

        let output = execute_monitor_iteration_with_runner(
            MonitorCommands::Watchlist {
                once: false,
                repeat: true,
            },
            &crate::monitor::MonitorConfig::default(),
            &runner,
            monitor_sample_time(),
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::AutomationIteration { run_mode, output } => {
                assert_eq!(run_mode, MonitorRunMode::Foreground);
                assert_eq!(output.snapshot.rows.len(), 1);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_monitor_daemon_run_uses_runner_in_daemon_mode() {
        let dir = tempdir().unwrap();
        let runner = MonitorRunner::new(
            FakeMonitorWatchlistReader {
                items: vec![monitor_watchlist_item("000001", "core", &[])],
            },
            FakeMonitorQuoteReader {
                rows: vec![monitor_quote_row("000001", 16.8, 3.2)],
            },
            SqliteMonitorAlertStore::new(dir.path().join("alerts.db"))
                .await
                .unwrap(),
            FakeStopRuleStore::default(),
            FakePaperTradeStore::default(),
        );

        let output = execute_monitor_iteration_with_runner(
            MonitorCommands::Daemon(MonitorDaemonCommands::Run),
            &crate::monitor::MonitorConfig::default(),
            &runner,
            monitor_sample_time(),
        )
        .await
        .unwrap();

        match output {
            MonitorCommandOutput::AutomationIteration { run_mode, output } => {
                assert_eq!(run_mode, MonitorRunMode::Daemon);
                assert_eq!(output.snapshot.rows.len(), 1);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[test]
    fn test_execute_monitor_service_config_show_returns_saved_binary_path() {
        let dir = tempdir().unwrap();
        let store = JsonMonitorServiceConfigStore::new(dir.path().join("service.json"));
        store
            .save(&MonitorServiceConfig {
                quantix_bin_path: "/bin/echo".into(),
            })
            .unwrap();

        let output = execute_monitor_service_config_command_with_store(
            MonitorServiceConfigCommands::Show,
            &store,
        )
        .unwrap();

        match output {
            MonitorCommandOutput::ServiceConfig(config) => {
                assert_eq!(
                    config.quantix_bin_path,
                    std::path::PathBuf::from("/bin/echo")
                );
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[test]
    fn test_execute_monitor_service_config_show_reports_not_configured_when_missing() {
        let dir = tempdir().unwrap();
        let store = JsonMonitorServiceConfigStore::new(dir.path().join("service.json"));

        let output = execute_monitor_service_config_command_with_store(
            MonitorServiceConfigCommands::Show,
            &store,
        )
        .unwrap();

        match output {
            MonitorCommandOutput::ServiceMessage(message) => {
                assert!(message.contains("未配置"));
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[test]
    fn test_execute_monitor_service_config_set_persists_binary_path() {
        let dir = tempdir().unwrap();
        let store = JsonMonitorServiceConfigStore::new(dir.path().join("service.json"));

        let output = execute_monitor_service_config_command_with_store(
            MonitorServiceConfigCommands::Set {
                quantix_bin: "/bin/echo".to_string(),
            },
            &store,
        )
        .unwrap();

        match output {
            MonitorCommandOutput::ServiceConfig(config) => {
                assert_eq!(
                    config.quantix_bin_path,
                    std::path::PathBuf::from("/bin/echo")
                );
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let saved = store.load().unwrap();
        assert_eq!(
            saved.quantix_bin_path,
            std::path::PathBuf::from("/bin/echo")
        );
    }

    #[test]
    fn test_execute_monitor_service_config_set_rejects_invalid_binary_path() {
        let dir = tempdir().unwrap();
        let store = JsonMonitorServiceConfigStore::new(dir.path().join("service.json"));

        let err = execute_monitor_service_config_command_with_store(
            MonitorServiceConfigCommands::Set {
                quantix_bin: "relative/path".to_string(),
            },
            &store,
        )
        .unwrap_err();

        assert!(err.to_string().contains("绝对"));
    }

    #[derive(Debug, Clone, Default)]
    struct FakeMonitorServiceInstaller {
        status_summary: Option<MonitorServiceStatusSummary>,
        status_error: Option<String>,
        uninstall_error: Option<String>,
    }

    fn sample_service_status_summary() -> MonitorServiceStatusSummary {
        MonitorServiceStatusSummary {
            installed: true,
            enabled: false,
            active: "inactive".to_string(),
            unit_path: std::path::PathBuf::from("~/.config/systemd/user/quantix-monitor.service"),
            wrapper_path: std::path::PathBuf::from("~/.local/bin/quantix-monitor-run"),
            quantix_bin_path: std::path::PathBuf::from("/bin/echo"),
            raw_status: None,
        }
    }

    #[test]
    fn test_execute_monitor_service_status_returns_summary() {
        let installer = FakeMonitorServiceInstaller {
            status_summary: Some(sample_service_status_summary()),
            ..Default::default()
        };

        let output = execute_monitor_service_command_with_installer(
            MonitorServiceCommands::Status,
            &installer,
        )
        .unwrap();

        match output {
            MonitorCommandOutput::ServiceStatus(summary) => {
                assert!(summary.installed);
                assert_eq!(summary.active, "inactive");
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[test]
    fn test_execute_monitor_service_uninstall_surfaces_stop_first_error() {
        let installer = FakeMonitorServiceInstaller {
            uninstall_error: Some(
                "monitor service 仍在运行，请先执行 monitor service stop".to_string(),
            ),
            ..Default::default()
        };

        let err = execute_monitor_service_command_with_installer(
            MonitorServiceCommands::Uninstall,
            &installer,
        )
        .unwrap_err();

        assert!(err.to_string().contains("monitor service stop"));
    }

    #[test]
    fn test_build_unconfigured_monitor_service_status_summary_marks_unconfigured() {
        let summary = build_unconfigured_monitor_service_status_summary();

        assert!(!summary.installed);
        assert!(!summary.enabled);
        assert_eq!(summary.active, "unconfigured");
        assert_eq!(
            summary.quantix_bin_path,
            std::path::PathBuf::from("<unconfigured>")
        );
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MarketBoardRequest {
        board_type: BoardType,
        date: Option<NaiveDate>,
        limit: usize,
        sort_by: BoardSortBy,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MarketLeaderRequest {
        filter: LeaderFilter,
        limit: usize,
        date: Option<NaiveDate>,
    }

    #[derive(Debug, Clone, Default)]
    struct FakeMarketState {
        board_requests: Vec<MarketBoardRequest>,
        leader_requests: Vec<MarketLeaderRequest>,
    }

    #[derive(Clone)]
    struct FakeMarketReader {
        state: Arc<Mutex<FakeMarketState>>,
    }

    impl FakeMarketReader {
        fn new() -> Self {
            Self {
                state: Arc::new(Mutex::new(FakeMarketState::default())),
            }
        }
    }

    impl MonitorServiceInstallerOps for FakeMonitorServiceInstaller {
        fn install(&self) -> Result<()> {
            Ok(())
        }

        fn uninstall(&self) -> Result<()> {
            match &self.uninstall_error {
                Some(message) => Err(QuantixError::Other(message.clone())),
                None => Ok(()),
            }
        }

        fn start(&self) -> Result<()> {
            Ok(())
        }

        fn stop(&self) -> Result<()> {
            Ok(())
        }

        fn enable(&self) -> Result<()> {
            Ok(())
        }

        fn disable(&self) -> Result<()> {
            Ok(())
        }

        fn status(&self) -> Result<String> {
            match &self.status_error {
                Some(message) => Err(QuantixError::Other(message.clone())),
                None => Ok("status-text".to_string()),
            }
        }

        fn status_summary(&self) -> Result<MonitorServiceStatusSummary> {
            match (&self.status_summary, &self.status_error) {
                (_, Some(message)) => Err(QuantixError::Other(message.clone())),
                (Some(summary), None) => Ok(summary.clone()),
                (None, None) => Err(QuantixError::Other("missing status summary".to_string())),
            }
        }
    }

    #[async_trait]
    impl MarketDataReader for FakeMarketReader {
        async fn load_board_rankings(
            &self,
            board_type: BoardType,
            date: Option<NaiveDate>,
            limit: usize,
            sort_by: BoardSortBy,
        ) -> Result<Vec<BoardRankRow>> {
            self.state
                .lock()
                .unwrap()
                .board_requests
                .push(MarketBoardRequest {
                    board_type,
                    date,
                    limit,
                    sort_by,
                });

            let rows = match board_type {
                BoardType::Sector => vec![BoardRankRow::new("BK001", "银行", board_type, 1, 2.1)],
                BoardType::Concept => {
                    vec![BoardRankRow::new("GN001", "人工智能", board_type, 1, 4.2)]
                }
            };

            Ok(rows.into_iter().take(limit).collect())
        }

        async fn load_north_flow(
            &self,
            date: Option<NaiveDate>,
        ) -> Result<Option<NorthFlowSnapshot>> {
            Ok(Some(NorthFlowSnapshot::new(
                date.unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 3, 10).unwrap()),
                12.3,
                8.6,
                20.9,
                100.0,
            )))
        }

        async fn load_market_sentiment(
            &self,
            date: Option<NaiveDate>,
        ) -> Result<Option<MarketSentimentSnapshot>> {
            Ok(Some(MarketSentimentSnapshot::new(
                date.unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 3, 10).unwrap()),
                3210,
                1875,
                87,
                4,
                0.81,
                0.19,
                23,
            )))
        }

        async fn load_leaders(
            &self,
            filter: LeaderFilter,
            limit: usize,
            date: Option<NaiveDate>,
        ) -> Result<Vec<LeaderRow>> {
            self.state
                .lock()
                .unwrap()
                .leader_requests
                .push(MarketLeaderRequest {
                    filter: filter.clone(),
                    limit,
                    date,
                });

            let rows = match filter {
                LeaderFilter::Sector(name) => {
                    vec![LeaderRow::new("600000", "浦发银行", Some(name), None, 5.6)]
                }
                LeaderFilter::Concept(name) => {
                    vec![LeaderRow::new("300024", "机器人", None, Some(name), 7.1)]
                }
                LeaderFilter::All => vec![
                    LeaderRow::new("300024", "机器人", None, Some("人工智能".to_string()), 7.1),
                    LeaderRow::new("600000", "浦发银行", Some("银行".to_string()), None, 5.6),
                ],
            };

            Ok(rows.into_iter().take(limit).collect())
        }
    }

    #[tokio::test]
    async fn test_execute_market_sector_returns_rows() {
        let reader = FakeMarketReader::new();

        let output = execute_market_command_with_reader(
            MarketCommands::Sector {
                top: Some(1),
                date: Some("2026-03-09".to_string()),
                sort_by: Some("change".to_string()),
            },
            reader.clone(),
        )
        .await
        .unwrap();

        match output {
            MarketCommandOutput::BoardRows(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].board_name, "银行");
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let state = reader.state.lock().unwrap();
        assert_eq!(
            state.board_requests,
            vec![MarketBoardRequest {
                board_type: BoardType::Sector,
                date: Some(NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()),
                limit: 1,
                sort_by: BoardSortBy::ChangePct,
            }]
        );
    }

    #[tokio::test]
    async fn test_execute_market_concept_returns_rows() {
        let output = execute_market_command_with_reader(
            MarketCommands::Concept {
                top: Some(1),
                date: None,
                sort_by: None,
            },
            FakeMarketReader::new(),
        )
        .await
        .unwrap();

        match output {
            MarketCommandOutput::BoardRows(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].board_name, "人工智能");
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_market_north_returns_snapshot() {
        let output = execute_market_command_with_reader(
            MarketCommands::North {
                date: Some("2026-03-09".to_string()),
            },
            FakeMarketReader::new(),
        )
        .await
        .unwrap();

        match output {
            MarketCommandOutput::NorthFlow(Some(snapshot)) => {
                assert_eq!(
                    snapshot.trade_date,
                    NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()
                );
                assert_eq!(snapshot.total_amount, 20.9);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_market_sentiment_returns_snapshot() {
        let output = execute_market_command_with_reader(
            MarketCommands::Sentiment { date: None },
            FakeMarketReader::new(),
        )
        .await
        .unwrap();

        match output {
            MarketCommandOutput::Sentiment(Some(snapshot)) => {
                assert_eq!(snapshot.limit_up_count, 87);
                assert_eq!(snapshot.consecutive_board_count, 23);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_market_leader_with_sector_returns_rows() {
        let reader = FakeMarketReader::new();

        let output = execute_market_command_with_reader(
            MarketCommands::Leader {
                sector: Some("银行".to_string()),
                concept: None,
                all: false,
                limit: Some(5),
                date: Some("2026-03-09".to_string()),
            },
            reader.clone(),
        )
        .await
        .unwrap();

        match output {
            MarketCommandOutput::Leaders(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].code, "600000");
            }
            other => panic!("unexpected output: {:?}", other),
        }

        let state = reader.state.lock().unwrap();
        assert_eq!(
            state.leader_requests,
            vec![MarketLeaderRequest {
                filter: LeaderFilter::Sector("银行".to_string()),
                limit: 5,
                date: Some(NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()),
            }]
        );
    }

    #[tokio::test]
    async fn test_execute_market_overview_returns_combined_payload() {
        let output = execute_market_command_with_reader(
            MarketCommands::Overview {
                top: Some(1),
                date: None,
            },
            FakeMarketReader::new(),
        )
        .await
        .unwrap();

        match output {
            MarketCommandOutput::Overview(overview) => {
                assert_eq!(overview.top_sectors.len(), 1);
                assert_eq!(overview.top_concepts.len(), 1);
                assert_eq!(overview.north_flow.unwrap().total_amount, 20.9);
                assert_eq!(overview.sentiment.unwrap().limit_up_count, 87);
            }
            other => panic!("unexpected output: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_market_leader_rejects_invalid_filter_combination() {
        let err = execute_market_command_with_reader(
            MarketCommands::Leader {
                sector: Some("银行".to_string()),
                concept: Some("人工智能".to_string()),
                all: false,
                limit: None,
                date: None,
            },
            FakeMarketReader::new(),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, QuantixError::Other(_)));
        assert!(err.to_string().contains("必须且只能指定"));
    }
