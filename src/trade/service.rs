use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::core::{QuantixError, Result};
use crate::trade::fees::calculate_fee_breakdown;
use crate::trade::models::{
    CashSnapshot, DEFAULT_ACCOUNT_ID, InitAccountRequest, PaperTradeAccount, PaperTradeState,
    TradeOrderRequest, TradePosition, TradeRecord, TradeSide,
};

/// 模拟交易账本持久化抽象。
#[async_trait]
pub trait PaperTradeStore: Send + Sync {
    /// 加载已持久化状态；文件不存在返回 `None`。
    async fn load_state(&self) -> Result<Option<PaperTradeState>>;

    /// 保存状态（实现应保证写入原子性）。
    async fn save_state(&self, state: &PaperTradeState) -> Result<()>;
}

/// 模拟交易核心服务：封装初始化 / 重置 / 买卖 / 查询。
#[derive(Debug, Clone)]
pub struct TradeService<Store> {
    store: Store,
}

impl<Store> TradeService<Store>
where
    Store: PaperTradeStore,
{
    /// 用指定存储后端构造服务。
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// 初始化账户。若账户已存在则返回错误（用 [`Self::reset_account`] 覆盖）。
    pub async fn init_account(
        &self,
        request: InitAccountRequest,
        now: DateTime<Utc>,
    ) -> Result<PaperTradeAccount> {
        let state = self.store.load_state().await?;
        if state.and_then(|state| state.account).is_some() {
            return Err(QuantixError::Other(
                "trade account 已初始化，请使用 trade reset".to_string(),
            ));
        }

        let account = build_account(request, now);
        let state = PaperTradeState {
            account: Some(account.clone()),
            ..PaperTradeState::default()
        };

        self.store.save_state(&state).await?;
        Ok(account)
    }

    /// 重置账户：清空持仓与交易记录，按新请求重新初始化。
    pub async fn reset_account(
        &self,
        request: InitAccountRequest,
        now: DateTime<Utc>,
    ) -> Result<PaperTradeAccount> {
        let account = build_account(request, now);
        let state = PaperTradeState {
            account: Some(account.clone()),
            ..PaperTradeState::default()
        };

        self.store.save_state(&state).await?;
        Ok(account)
    }

    /// 买入：校验资金充足，更新现金 / 持仓均价 / 最新价，落地交易记录。
    pub async fn buy(&self, request: TradeOrderRequest, now: DateTime<Utc>) -> Result<TradeRecord> {
        let mut state = self.load_initialized_state().await?;
        let account = state
            .account
            .as_mut()
            .ok_or_else(uninitialized_account_error)?;
        let amount = request.price * decimal_volume(request.volume)?;
        let fees =
            calculate_fee_breakdown(TradeSide::Buy, &request.code, amount, &account.fee_config);
        let total_cost = amount + fees.total_fee;

        if total_cost > account.available_cash {
            return Err(QuantixError::Other(format!(
                "trade buy 可用资金不足，所需 {total_cost}，当前 {}",
                account.available_cash
            )));
        }

        account.available_cash -= total_cost;
        account.updated_at = now;

        match account.positions.get_mut(&request.code) {
            Some(position) => {
                let existing_cost = position.avg_cost * decimal_volume(position.volume)?;
                let new_volume = position.volume + request.volume;
                let new_cost = existing_cost + total_cost;
                position.volume = new_volume;
                position.avg_cost = new_cost / decimal_volume(new_volume)?;
                position.last_trade_price = request.price;
                position.updated_at = now;
            }
            None => {
                account.positions.insert(
                    request.code.clone(),
                    TradePosition {
                        code: request.code.clone(),
                        volume: request.volume,
                        avg_cost: total_cost / decimal_volume(request.volume)?,
                        last_trade_price: request.price,
                        opened_at: now,
                        updated_at: now,
                    },
                );
            }
        }

        let record = build_record(request, TradeSide::Buy, amount, fees, now);
        state.trade_records.push(record.clone());
        self.store.save_state(&state).await?;

        Ok(record)
    }

    /// 卖出：校验持仓数量，更新现金 / 持仓（清零时移除），落地交易记录。
    pub async fn sell(
        &self,
        request: TradeOrderRequest,
        now: DateTime<Utc>,
    ) -> Result<TradeRecord> {
        let mut state = self.load_initialized_state().await?;
        let account = state
            .account
            .as_mut()
            .ok_or_else(uninitialized_account_error)?;
        let position = account
            .positions
            .get_mut(&request.code)
            .ok_or_else(|| QuantixError::Other(format!("trade sell 未持有 {}", request.code)))?;

        if request.volume > position.volume {
            return Err(QuantixError::Other(format!(
                "trade sell 可卖数量不足，{} 当前持仓 {}，请求卖出 {}",
                request.code, position.volume, request.volume
            )));
        }

        let amount = request.price * decimal_volume(request.volume)?;
        let fees =
            calculate_fee_breakdown(TradeSide::Sell, &request.code, amount, &account.fee_config);
        account.available_cash += amount - fees.total_fee;
        account.updated_at = now;

        if request.volume == position.volume {
            account.positions.remove(&request.code);
        } else {
            position.volume -= request.volume;
            position.last_trade_price = request.price;
            position.updated_at = now;
        }

        let record = build_record(request, TradeSide::Sell, amount, fees, now);
        state.trade_records.push(record.clone());
        self.store.save_state(&state).await?;

        Ok(record)
    }

    /// 返回当前持仓列表（账户未初始化时返回错误）。
    pub async fn positions(&self) -> Result<Vec<TradePosition>> {
        let state = self.load_initialized_state().await?;
        let account = state.account.ok_or_else(uninitialized_account_error)?;
        Ok(account.positions.into_values().collect())
    }

    /// 返回现金快照（含按 `last_trade_price` 估算的持仓市值）。
    pub async fn cash_snapshot(&self) -> Result<CashSnapshot> {
        let state = self.load_initialized_state().await?;
        let account = state.account.ok_or_else(uninitialized_account_error)?;
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

    /// 返回完整账本状态快照（账户未初始化时返回错误）。
    pub async fn state_snapshot(&self) -> Result<PaperTradeState> {
        self.load_initialized_state().await
    }

    async fn load_initialized_state(&self) -> Result<PaperTradeState> {
        let state = self.store.load_state().await?.unwrap_or_default();
        if state.account.is_none() {
            return Err(uninitialized_account_error());
        }

        Ok(state)
    }
}

fn uninitialized_account_error() -> QuantixError {
    QuantixError::Other("trade account 尚未初始化，请先运行 trade init".to_string())
}

fn build_account(request: InitAccountRequest, now: DateTime<Utc>) -> PaperTradeAccount {
    PaperTradeAccount {
        account_id: DEFAULT_ACCOUNT_ID.to_string(),
        initial_capital: request.capital,
        available_cash: request.capital,
        fee_config: request.fee_config,
        positions: BTreeMap::new(),
        created_at: now,
        updated_at: now,
    }
}

fn build_record(
    request: TradeOrderRequest,
    side: TradeSide,
    amount: Decimal,
    fees: crate::trade::FeeBreakdown,
    now: DateTime<Utc>,
) -> TradeRecord {
    TradeRecord {
        id: Uuid::new_v4().to_string(),
        code: request.code,
        side,
        price: request.price,
        volume: request.volume,
        amount,
        commission: fees.commission,
        stamp_duty: fees.stamp_duty,
        transfer_fee: fees.transfer_fee,
        total_fee: fees.total_fee,
        executed_at: now,
    }
}

fn decimal_volume(volume: i64) -> Result<Decimal> {
    Decimal::from_i64(volume)
        .ok_or_else(|| QuantixError::Other(format!("trade volume {volume} 无法转换为 Decimal")))
}
