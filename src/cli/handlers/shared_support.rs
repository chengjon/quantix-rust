use super::*;

use crate::core::{CliRuntime, QuantixError, Result};
use crate::db::clickhouse::ClickHouseClient;
use crate::monitor::{
    JsonMonitorConfigStore, JsonMonitorServiceConfigStore, MonitorAlertStore, MonitorConfig,
    MonitorEventFilter, MonitorEventRow, MonitorEventType, MonitorIterationOutput,
    MonitorQuoteReader, MonitorQuoteRow, MonitorRunMode, MonitorRunner, MonitorService,
    MonitorServiceConfig, MonitorServiceStatusSummary, MonitorUserServiceInstaller,
    MonitorWatchlistReader, MonitorWatchlistSnapshot, PriceAlert, PriceAlertKind,
};
use crate::risk::{JsonRiskStore, RiskAccountSnapshot, RiskService};
use crate::stop::{StopHistoryEventType, StopRule, StopRuleStore, StopService, StopStatusRow};
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
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::collections::{BTreeMap, HashMap};

pub(crate) async fn create_clickhouse_client() -> Result<ClickHouseClient> {
    let runtime = CliRuntime::load();
    ClickHouseClient::from_settings(&runtime.clickhouse).await
}

pub(crate) fn build_trade_init_request(
    command_name: &str,
    capital: Option<f64>,
    commission_rate: Option<f64>,
    commission_min: Option<f64>,
    stamp_duty_rate: Option<f64>,
    transfer_fee_rate: Option<f64>,
) -> Result<InitAccountRequest> {
    InitAccountRequest::new(
        capital,
        commission_rate,
        commission_min,
        stamp_duty_rate,
        transfer_fee_rate,
    )
    .map_err(|err| remap_trade_request_error(err, command_name))
}

pub(crate) fn build_trade_order_request(
    command_name: &str,
    code: String,
    price: f64,
    volume: i64,
) -> Result<TradeOrderRequest> {
    TradeOrderRequest::new(code, price, volume)
        .map_err(|err| remap_trade_request_error(err, command_name))
}

pub(crate) fn decimal_to_f64(value: Decimal, command_name: &str) -> Result<f64> {
    value
        .to_f64()
        .ok_or_else(|| QuantixError::Other(format!("{command_name} 无法将价格 {value} 转换为 f64")))
}

pub(crate) fn remap_trade_request_error(err: QuantixError, command_name: &str) -> QuantixError {
    match err {
        QuantixError::Other(message) => QuantixError::Other(
            message
                .replace("trade init", command_name)
                .replace("trade order", command_name),
        ),
        other => other,
    }
}

pub(crate) fn patch_value(value: Option<f64>, clear: bool) -> Option<Option<f64>> {
    if clear { Some(None) } else { value.map(Some) }
}

pub(crate) fn parse_stop_history_event_type(value: &str) -> Result<StopHistoryEventType> {
    StopHistoryEventType::from_str(value)
        .ok_or_else(|| QuantixError::Unsupported(format!("未知 stop history event_type: {value}")))
}

pub(crate) fn parse_stop_history_date(value: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| QuantixError::Other(format!("stop history --date 无效: {value}")))
}

pub(crate) fn filter_stop_rules(rules: Vec<StopRule>, code: Option<&str>) -> Vec<StopRule> {
    match code {
        Some(code) => rules.into_iter().filter(|rule| rule.code == code).collect(),
        None => rules,
    }
}

pub(crate) async fn build_avg_cost_map_from_trade_store<Store>(
    trade_store: &Store,
) -> Result<HashMap<String, f64>>
where
    Store: PaperTradeStore,
{
    let Some(state) = trade_store.load_state().await? else {
        return Ok(HashMap::new());
    };
    let Some(account) = state.account else {
        return Ok(HashMap::new());
    };

    Ok(account
        .positions
        .into_iter()
        .filter_map(|(code, position)| position.avg_cost.to_f64().map(|avg_cost| (code, avg_cost)))
        .collect())
}

pub(crate) async fn resolve_stop_reference_price<Q, TS>(
    code: &str,
    quote_lookup: &Q,
    trade_store: &TS,
) -> Result<f64>
where
    Q: WatchlistQuoteLookup,
    TS: PaperTradeStore,
{
    let quote_price = quote_lookup
        .lookup_quotes(&[code.to_string()])
        .await
        .ok()
        .and_then(|quotes| {
            quotes
                .get(code)
                .and_then(|snapshot| snapshot.latest_price.to_f64())
        });
    if let Some(price) = quote_price {
        return Ok(price);
    }

    let avg_cost_by_code = build_avg_cost_map_from_trade_store(trade_store).await?;
    if let Some(avg_cost) = avg_cost_by_code.get(code).copied() {
        return Ok(avg_cost);
    }

    Err(QuantixError::Other(format!(
        "stop percent 规则缺少参考价，且当前无法从行情或持仓解析 {} 的 reference_price",
        code
    )))
}

pub(crate) async fn build_stop_status_rows<RS, Q, TS>(
    service: &StopService<RS>,
    rules: &[StopRule],
    quote_lookup: &Q,
    trade_store: &TS,
    observed_at: DateTime<Utc>,
) -> Result<Vec<StopStatusRow>>
where
    RS: StopRuleStore,
    Q: WatchlistQuoteLookup,
    TS: PaperTradeStore,
{
    let codes: Vec<String> = rules.iter().map(|rule| rule.code.clone()).collect();
    let quote_rows = quote_lookup
        .lookup_quotes(&codes)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(code, snapshot)| MonitorQuoteRow {
            code,
            group: String::new(),
            tags: Vec::new(),
            last_price: snapshot.latest_price.to_f64(),
            change_pct: snapshot.price_change_pct.and_then(|value| value.to_f64()),
            quote_time: None,
            note: None,
        })
        .collect::<Vec<_>>();
    let avg_cost_by_code = build_avg_cost_map_from_trade_store(trade_store).await?;
    Ok(service.status_rows(rules, &quote_rows, &avg_cost_by_code, observed_at))
}

pub(crate) fn ensure_watchlist_contains_code(storage: &WatchlistStorage, code: &str) -> Result<()> {
    let store = load_watchlist_store_for_read(storage)?;
    if store.entries.contains_key(code) {
        Ok(())
    } else {
        Err(QuantixError::Other(format!("股票不在自选池: {}", code)))
    }
}

pub(crate) fn format_stop_eval_state(state: crate::stop::StopEvalState) -> &'static str {
    match state {
        crate::stop::StopEvalState::Armed => "armed",
        crate::stop::StopEvalState::Triggered => "triggered",
        crate::stop::StopEvalState::AnchorMissing => "anchor_missing",
        crate::stop::StopEvalState::QuoteMissing => "quote_missing",
    }
}

pub(crate) fn create_trade_store() -> JsonPaperTradeStore {
    let runtime = CliRuntime::load();
    JsonPaperTradeStore::new(runtime.trade_path)
}

pub(crate) fn create_risk_store() -> JsonRiskStore {
    let runtime = CliRuntime::load();
    JsonRiskStore::new(runtime.risk_path)
}

pub(crate) async fn sync_risk_from_trade_store<TradeStore, RiskStore>(
    trade_store: &TradeStore,
    risk_service: &RiskService<RiskStore>,
) -> Result<()>
where
    TradeStore: PaperTradeStore,
    RiskStore: crate::risk::RiskStore,
{
    let account = load_initialized_trade_account(trade_store).await?;
    let snapshot = build_risk_account_snapshot(&account);
    risk_service
        .sync_after_trade_snapshot(&snapshot, Utc::now())
        .await?;
    Ok(())
}

pub(crate) async fn load_initialized_trade_account<Store>(
    trade_store: &Store,
) -> Result<PaperTradeAccount>
where
    Store: PaperTradeStore,
{
    trade_store
        .load_state()
        .await?
        .and_then(|state| state.account)
        .ok_or_else(|| {
            QuantixError::Other("trade account 尚未初始化，请先运行 trade init".to_string())
        })
}

pub(crate) async fn load_trade_quote_prices<Q>(
    state: &PaperTradeState,
    quote_lookup: &Q,
) -> BTreeMap<String, Decimal>
where
    Q: WatchlistQuoteLookup,
{
    let Some(account) = &state.account else {
        return BTreeMap::new();
    };

    let codes: Vec<String> = account.positions.keys().cloned().collect();
    quote_lookup
        .lookup_quotes(&codes)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(code, snapshot)| (code, snapshot.latest_price))
        .collect()
}

pub(crate) fn build_risk_account_snapshot(account: &PaperTradeAccount) -> RiskAccountSnapshot {
    let positions: Vec<(String, rust_decimal::Decimal)> = account
        .positions
        .values()
        .map(|position| {
            (
                position.code.clone(),
                rust_decimal::Decimal::from(position.volume) * position.last_trade_price,
            )
        })
        .collect();
    let position_value = positions
        .iter()
        .fold(rust_decimal::Decimal::ZERO, |acc, (_, value)| acc + *value);

    RiskAccountSnapshot::new(
        account.account_id.clone(),
        account.available_cash + position_value,
        positions,
    )
}

pub(crate) fn build_projected_buy_impact(
    account: &PaperTradeAccount,
    request: &TradeOrderRequest,
) -> crate::risk::ProjectedBuyImpact {
    let current_position_value = account
        .positions
        .get(&request.code)
        .map(|position| rust_decimal::Decimal::from(position.volume) * position.last_trade_price)
        .unwrap_or(rust_decimal::Decimal::ZERO);

    crate::risk::ProjectedBuyImpact::new(
        request.code.clone(),
        current_position_value + request.price * rust_decimal::Decimal::from(request.volume),
        build_risk_account_snapshot(account).total_assets,
    )
}
