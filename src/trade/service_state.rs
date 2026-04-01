use chrono::{DateTime, Utc};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

use crate::core::{QuantixError, Result};
use crate::trade::models::{
    CashSnapshot, DEFAULT_ACCOUNT_ID, InitAccountRequest, PaperTradeAccount, PaperTradeState,
    TradePosition,
};

pub(super) fn init_account_state(
    existing_state: Option<PaperTradeState>,
    request: InitAccountRequest,
    now: DateTime<Utc>,
) -> Result<(PaperTradeAccount, PaperTradeState)> {
    if existing_state.and_then(|state| state.account).is_some() {
        return Err(QuantixError::Other(
            "trade account 已初始化，请使用 trade reset".to_string(),
        ));
    }

    let account = build_account(request, now);
    let state = PaperTradeState {
        account: Some(account.clone()),
        ..PaperTradeState::default()
    };
    Ok((account, state))
}

pub(super) fn reset_account_state(
    request: InitAccountRequest,
    now: DateTime<Utc>,
) -> (PaperTradeAccount, PaperTradeState) {
    let account = build_account(request, now);
    let state = PaperTradeState {
        account: Some(account.clone()),
        ..PaperTradeState::default()
    };
    (account, state)
}

pub(super) fn ensure_initialized_state(
    state: Option<PaperTradeState>,
) -> Result<PaperTradeState> {
    let state = state.unwrap_or_default();
    if state.account.is_none() {
        return Err(QuantixError::Other(
            "trade account 尚未初始化，请先运行 trade init".to_string(),
        ));
    }

    Ok(state)
}

pub(super) fn positions_from_state(state: PaperTradeState) -> Vec<TradePosition> {
    state
        .account
        .expect("initialized account")
        .positions
        .into_values()
        .collect()
}

pub(super) fn cash_snapshot_from_state(state: PaperTradeState) -> Result<CashSnapshot> {
    let account = state.account.expect("initialized account");
    let estimated_position_value = account.positions.values().try_fold(
        Decimal::ZERO,
        |acc, position| -> Result<Decimal> {
            Ok(acc + decimal_volume(position.volume)? * position.last_trade_price)
        },
    )?;

    Ok(CashSnapshot {
        initial_capital: account.initial_capital,
        available_cash: account.available_cash,
        estimated_position_value,
        estimated_total_assets: account.available_cash + estimated_position_value,
    })
}

fn build_account(request: InitAccountRequest, now: DateTime<Utc>) -> PaperTradeAccount {
    PaperTradeAccount {
        account_id: DEFAULT_ACCOUNT_ID.to_string(),
        initial_capital: request.capital,
        available_cash: request.capital,
        fee_config: request.fee_config,
        positions: Default::default(),
        created_at: now,
        updated_at: now,
    }
}

fn decimal_volume(volume: i64) -> Result<Decimal> {
    Decimal::from_i64(volume)
        .ok_or_else(|| QuantixError::Other(format!("trade volume {volume} 无法转换为 Decimal")))
}
