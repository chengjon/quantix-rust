use super::strategy_helpers::{FakeLoader, fixed_ts, make_kline};
use super::*;
use crate::core::{CliRuntime, QuantixError, Result};
use crate::risk::ResolvedIndustry;
use crate::risk::service::RiskIndustryResolver;
use crate::risk::{ClassificationStandard, IndustryClassificationLevel, IndustrySourceTier};
use crate::trade::{
    CashSnapshot, InitAccountRequest, JsonPaperTradeStore, PaperTradeAccount, PaperTradeState,
    PaperTradeStore, TradeFeeRow, TradeHistoryRow, TradeOrderRequest, TradeOverview, TradePosition,
    TradePositionCurrentRow, TradeQuoteStatus, TradeRecord, TradeReportingService, TradeService,
};
use crate::watchlist::{
    PostgresWatchlistNameLookup, TdxWatchlistQuoteLookup, WatchlistDisplayRow,
    WatchlistHistoryEvent, WatchlistListItem, WatchlistQuoteLookup, WatchlistService,
    WatchlistStorage, WatchlistStore,
};
use async_trait::async_trait;
use rust_decimal_macros::dec;
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Default)]
pub(super) struct FakePaperTradeStore {
    pub(super) state: Arc<Mutex<Option<PaperTradeState>>>,
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

pub(super) fn trade_service() -> (TradeService<FakePaperTradeStore>, FakePaperTradeStore) {
    let store = FakePaperTradeStore::default();
    (TradeService::new(store.clone()), store)
}

#[derive(Debug, Clone, Default)]
struct FakeRiskIndustryResolver {
    industries: HashMap<String, String>,
}

impl FakeRiskIndustryResolver {
    fn with_rows(rows: &[(&str, &str)]) -> Self {
        Self {
            industries: rows
                .iter()
                .map(|(code, industry)| ((*code).to_string(), (*industry).to_string()))
                .collect(),
        }
    }
}

#[async_trait]
impl RiskIndustryResolver for FakeRiskIndustryResolver {
    async fn resolve(
        &self,
        code: &str,
        _trade_date: NaiveDate,
        _captured_at: chrono::DateTime<Utc>,
    ) -> Result<ResolvedIndustry> {
        let industry_name = self
            .industries
            .get(code)
            .cloned()
            .ok_or_else(|| QuantixError::Other(format!("resolver miss: {code}")))?;

        Ok(ResolvedIndustry {
            code: code.to_string(),
            industry_name,
            standard: ClassificationStandard::Shenwan,
            level: IndustryClassificationLevel::FirstLevel,
            source_tier: IndustrySourceTier::Historical,
            query_month: "2026-03".to_string(),
        })
    }
}

#[derive(Clone, Default)]
pub(super) struct FakeTradeQuoteLookup {
    pub(super) quotes: HashMap<String, WatchlistQuoteSnapshot>,
    pub(super) fail: bool,
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
async fn test_execute_trade_buy_rejects_when_industry_limit_exceeds_threshold() {
    let (service, store) = trade_service();
    let dir = tempdir().unwrap();
    let risk_service = RiskService::with_industry_resolver(
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        FakeRiskIndustryResolver::with_rows(&[("000001", "银行"), ("600000", "银行")]),
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

    execute_trade_command_with_risk(
        TradeCommands::Buy {
            code: "600000".to_string(),
            price: 10.0,
            volume: 20000,
        },
        &service,
        &store,
        &risk_service,
    )
    .await
    .unwrap();

    risk_service
        .set_rule("industry-limit", "30%", fixed_ts())
        .await
        .unwrap();

    let err = execute_trade_command_with_risk(
        TradeCommands::Buy {
            code: "000001".to_string(),
            price: 10.0,
            volume: 15000,
        },
        &service,
        &store,
        &risk_service,
    )
    .await
    .unwrap_err();

    assert!(err.to_string().contains("industry-limit"));
    assert!(err.to_string().contains("银行"));
    assert!(err.to_string().contains("projected_ratio="));

    let state = store.snapshot().unwrap();
    let account = state.account.unwrap();
    assert_eq!(account.positions.len(), 1);
    assert_eq!(account.positions.get("600000").unwrap().code, "600000");
    assert_eq!(state.trade_records.len(), 1);
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

    let output =
        execute_trade_command_with_service(TradeCommands::Position { current: false }, &service)
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

    let output =
        execute_trade_command_with_service(TradeCommands::Overview { current: false }, &service)
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

    let err =
        execute_trade_command_with_service(TradeCommands::Overview { current: false }, &service)
            .await
            .unwrap_err();

    assert!(matches!(err, QuantixError::Other(_)));
    assert!(err.to_string().contains("尚未初始化"));
}
