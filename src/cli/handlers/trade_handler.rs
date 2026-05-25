use super::*;

use crate::core::{CliRuntime, QuantixError, Result};
use crate::risk::RiskService;
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

pub async fn run_trade_command(cmd: TradeCommands) -> Result<()> {
    let trade_store = create_trade_store();
    let service = TradeService::new(trade_store.clone());
    let risk_service = RiskService::new(create_risk_store());
    let output =
        execute_trade_command_with_risk(cmd, &service, &trade_store, &risk_service).await?;
    print_trade_command_output(&output);
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TradeCommandOutput {
    AccountInitialized(PaperTradeAccount),
    AccountReset(PaperTradeAccount),
    TradeExecuted(TradeRecord),
    HistoryRows(Vec<TradeHistoryRow>),
    FeeRows(Vec<TradeFeeRow>),
    Overview(TradeOverview),
    PositionList(Vec<TradePosition>),
    PositionCurrentList(Vec<TradePositionCurrentRow>),
    Cash(CashSnapshot),
}

pub(crate) async fn execute_trade_command_with_service<Store>(
    cmd: TradeCommands,
    service: &TradeService<Store>,
) -> Result<TradeCommandOutput>
where
    Store: PaperTradeStore,
{
    let reporting = TradeReportingService::new();
    match cmd {
        TradeCommands::Init {
            capital,
            commission_rate,
            commission_min,
            stamp_duty_rate,
            transfer_fee_rate,
        } => {
            let request = build_trade_init_request(
                "trade init",
                capital,
                commission_rate,
                commission_min,
                stamp_duty_rate,
                transfer_fee_rate,
            )?;
            Ok(TradeCommandOutput::AccountInitialized(
                service.init_account(request, Utc::now()).await?,
            ))
        }
        TradeCommands::Reset {
            capital,
            commission_rate,
            commission_min,
            stamp_duty_rate,
            transfer_fee_rate,
        } => {
            let request = build_trade_init_request(
                "trade reset",
                capital,
                commission_rate,
                commission_min,
                stamp_duty_rate,
                transfer_fee_rate,
            )?;
            Ok(TradeCommandOutput::AccountReset(
                service.reset_account(request, Utc::now()).await?,
            ))
        }
        TradeCommands::Buy {
            code,
            price,
            volume,
        } => {
            let request = build_trade_order_request("trade buy", code, price, volume)?;
            Ok(TradeCommandOutput::TradeExecuted(
                service.buy(request, Utc::now()).await?,
            ))
        }
        TradeCommands::Sell {
            code,
            price,
            volume,
        } => {
            let request = build_trade_order_request("trade sell", code, price, volume)?;
            Ok(TradeCommandOutput::TradeExecuted(
                service.sell(request, Utc::now()).await?,
            ))
        }
        TradeCommands::History { code, limit } => {
            let state = service.state_snapshot().await?;
            Ok(TradeCommandOutput::HistoryRows(reporting.history_rows(
                &state,
                code.as_deref(),
                limit,
            )))
        }
        TradeCommands::Fees { code, limit } => {
            let state = service.state_snapshot().await?;
            Ok(TradeCommandOutput::FeeRows(reporting.fee_rows(
                &state,
                code.as_deref(),
                limit,
            )))
        }
        TradeCommands::Overview { current: false } => {
            let state = service.state_snapshot().await?;
            Ok(TradeCommandOutput::Overview(reporting.overview(&state)))
        }
        TradeCommands::Overview { current: true } | TradeCommands::Position { current: true } => {
            Err(QuantixError::Unsupported(
                "trade current views require quote lookup".to_string(),
            ))
        }
        TradeCommands::Position { current: false } => {
            Ok(TradeCommandOutput::PositionList(service.positions().await?))
        }
        TradeCommands::Cash => Ok(TradeCommandOutput::Cash(service.cash_snapshot().await?)),
    }
}

pub(crate) async fn execute_trade_command_with_quote_lookup<Store, Q>(
    cmd: TradeCommands,
    service: &TradeService<Store>,
    quote_lookup: &Q,
) -> Result<TradeCommandOutput>
where
    Store: PaperTradeStore,
    Q: WatchlistQuoteLookup,
{
    let reporting = TradeReportingService::new();

    match cmd {
        TradeCommands::Overview { current: true } => {
            let state = service.state_snapshot().await?;
            let quotes = load_trade_quote_prices(&state, quote_lookup).await;
            let total_positions = state
                .account
                .as_ref()
                .map(|account| account.positions.len())
                .unwrap_or(0);
            let resolved_positions = quotes.len();

            let mut overview = reporting.overview(&state);
            overview.quote_coverage = Some((resolved_positions, total_positions));

            if total_positions == 0 {
                overview.live_position_value = Some(Decimal::ZERO);
                overview.live_total_assets = Some(overview.booked_total_assets);
            } else if resolved_positions == total_positions {
                let rows = reporting.position_rows_with_quotes(&state, &quotes);
                let live_position_value = rows
                    .iter()
                    .filter_map(|row| row.current_market_value)
                    .sum::<Decimal>();
                overview.live_position_value = Some(live_position_value);
                overview.live_total_assets = Some(overview.available_cash + live_position_value);
            }

            Ok(TradeCommandOutput::Overview(overview))
        }
        TradeCommands::Position { current: true } => {
            let state = service.state_snapshot().await?;
            let quotes = load_trade_quote_prices(&state, quote_lookup).await;
            Ok(TradeCommandOutput::PositionCurrentList(
                reporting.position_rows_with_quotes(&state, &quotes),
            ))
        }
        other => execute_trade_command_with_service(other, service).await,
    }
}

pub(crate) async fn execute_trade_command_with_risk<TradeStore, RiskStore>(
    cmd: TradeCommands,
    trade_service: &TradeService<TradeStore>,
    trade_store: &TradeStore,
    risk_service: &RiskService<RiskStore>,
) -> Result<TradeCommandOutput>
where
    TradeStore: PaperTradeStore,
    RiskStore: crate::risk::RiskStore,
{
    match cmd {
        TradeCommands::Init {
            capital,
            commission_rate,
            commission_min,
            stamp_duty_rate,
            transfer_fee_rate,
        } => {
            let request = build_trade_init_request(
                "trade init",
                capital,
                commission_rate,
                commission_min,
                stamp_duty_rate,
                transfer_fee_rate,
            )?;
            let account = trade_service.init_account(request, Utc::now()).await?;
            let snapshot = build_risk_account_snapshot(&account);
            risk_service
                .sync_after_trade_reset(&snapshot, Utc::now())
                .await?;
            Ok(TradeCommandOutput::AccountInitialized(account))
        }
        TradeCommands::Reset {
            capital,
            commission_rate,
            commission_min,
            stamp_duty_rate,
            transfer_fee_rate,
        } => {
            let request = build_trade_init_request(
                "trade reset",
                capital,
                commission_rate,
                commission_min,
                stamp_duty_rate,
                transfer_fee_rate,
            )?;
            let account = trade_service.reset_account(request, Utc::now()).await?;
            let snapshot = build_risk_account_snapshot(&account);
            risk_service
                .sync_after_trade_reset(&snapshot, Utc::now())
                .await?;
            Ok(TradeCommandOutput::AccountReset(account))
        }
        TradeCommands::Buy {
            code,
            price,
            volume,
        } => {
            let request = build_trade_order_request("trade buy", code, price, volume)?;
            let account = load_initialized_trade_account(trade_store).await?;
            let snapshot = build_risk_account_snapshot(&account);
            let projected_buy = build_projected_buy_impact(&account, &request);
            risk_service
                .check_buy(&snapshot, &projected_buy, Utc::now())
                .await?;

            let record = trade_service.buy(request, Utc::now()).await?;
            sync_risk_from_trade_store(trade_store, risk_service).await?;
            Ok(TradeCommandOutput::TradeExecuted(record))
        }
        TradeCommands::Sell {
            code,
            price,
            volume,
        } => {
            let request = build_trade_order_request("trade sell", code, price, volume)?;
            let record = trade_service.sell(request, Utc::now()).await?;
            sync_risk_from_trade_store(trade_store, risk_service).await?;
            Ok(TradeCommandOutput::TradeExecuted(record))
        }
        TradeCommands::Overview { current: true } | TradeCommands::Position { current: true } => {
            execute_trade_command_with_quote_lookup(cmd, trade_service, &TdxWatchlistQuoteLookup)
                .await
        }
        other => execute_trade_command_with_service(other, trade_service).await,
    }
}
