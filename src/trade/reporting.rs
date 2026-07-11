use std::collections::BTreeMap;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::trade::{
    PaperTradeState, TradeFeeRow, TradeHistoryRow, TradeOverview, TradePositionCurrentRow,
    TradeQuoteStatus, TradeSide,
};

const DEFAULT_TRADE_REPORT_LIMIT: usize = 20;

/// 模拟交易账本报告生成器：将 [`PaperTradeState`] 投影为面向展示的行 / 概览。
#[derive(Debug, Clone, Default)]
pub struct TradeReportingService;

impl TradeReportingService {
    /// 构造默认实例（无内部状态）。
    pub fn new() -> Self {
        Self
    }

    /// 返回交易历史行（按时间倒序），可按代码过滤，`limit` 缺省取 20。
    pub fn history_rows(
        &self,
        state: &PaperTradeState,
        code_filter: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<TradeHistoryRow> {
        let mut rows: Vec<_> = state
            .trade_records
            .iter()
            .filter(|record| code_filter.is_none_or(|code| record.code == code))
            .map(|record| TradeHistoryRow {
                executed_at: record.executed_at,
                code: record.code.clone(),
                side: record.side,
                price: record.price,
                volume: record.volume,
                amount: record.amount,
                total_fee: record.total_fee,
                net_cash_impact: net_cash_impact(record.side, record.amount, record.total_fee),
            })
            .collect();
        rows.sort_by_key(|row| std::cmp::Reverse(row.executed_at));
        rows.truncate(limit.unwrap_or(DEFAULT_TRADE_REPORT_LIMIT));
        rows
    }

    /// 返回手续费明细行（按时间倒序），可按代码过滤，`limit` 缺省取 20。
    pub fn fee_rows(
        &self,
        state: &PaperTradeState,
        code_filter: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<TradeFeeRow> {
        let mut rows: Vec<_> = state
            .trade_records
            .iter()
            .filter(|record| code_filter.is_none_or(|code| record.code == code))
            .map(|record| TradeFeeRow {
                executed_at: record.executed_at,
                code: record.code.clone(),
                side: record.side,
                commission: record.commission,
                stamp_duty: record.stamp_duty,
                transfer_fee: record.transfer_fee,
                total_fee: record.total_fee,
            })
            .collect();
        rows.sort_by_key(|row| std::cmp::Reverse(row.executed_at));
        rows.truncate(limit.unwrap_or(DEFAULT_TRADE_REPORT_LIMIT));
        rows
    }

    /// 生成账户概览（现金、持仓市值、累计买卖金额、累计手续费）。
    ///
    /// 未初始化账户时返回零值概览。
    pub fn overview(&self, state: &PaperTradeState) -> TradeOverview {
        let Some(account) = &state.account else {
            return TradeOverview {
                initial_capital: Decimal::ZERO,
                available_cash: Decimal::ZERO,
                booked_position_value: Decimal::ZERO,
                booked_total_assets: Decimal::ZERO,
                trade_count: 0,
                holding_count: 0,
                total_buy_amount: Decimal::ZERO,
                total_sell_amount: Decimal::ZERO,
                total_fee: Decimal::ZERO,
                live_position_value: None,
                live_total_assets: None,
                quote_coverage: None,
            };
        };

        let booked_position_value = account
            .positions
            .values()
            .fold(Decimal::ZERO, |acc, position| {
                acc + Decimal::from(position.volume) * position.last_trade_price
            });

        let (total_buy_amount, total_sell_amount, total_fee) = aggregate_trade_totals(state);

        TradeOverview {
            initial_capital: account.initial_capital,
            available_cash: account.available_cash,
            booked_position_value,
            booked_total_assets: account.available_cash + booked_position_value,
            trade_count: state.trade_records.len(),
            holding_count: account.positions.len(),
            total_buy_amount,
            total_sell_amount,
            total_fee,
            live_position_value: None,
            live_total_assets: None,
            quote_coverage: None,
        }
    }

    /// 返回当前持仓行（无实时行情，`quote_status = BookOnly`）。
    pub fn position_rows(&self, state: &PaperTradeState) -> Vec<TradePositionCurrentRow> {
        let Some(account) = &state.account else {
            return Vec::new();
        };

        account
            .positions
            .values()
            .map(|position| TradePositionCurrentRow {
                code: position.code.clone(),
                volume: position.volume,
                avg_cost: position.avg_cost,
                last_trade_price: position.last_trade_price,
                current_price: None,
                current_market_value: None,
                unrealized_pnl: None,
                unrealized_pnl_pct: None,
                quote_status: TradeQuoteStatus::BookOnly,
            })
            .collect()
    }

    /// 返回带实时行情的持仓行：能匹配到报价的标 `Live`，缺失的标 `Missing`。
    ///
    /// 浮动盈亏 = 市值 - 成本基础；百分比以成本基础为零时返回 `0`。
    pub fn position_rows_with_quotes(
        &self,
        state: &PaperTradeState,
        quotes: &BTreeMap<String, Decimal>,
    ) -> Vec<TradePositionCurrentRow> {
        let Some(account) = &state.account else {
            return Vec::new();
        };

        account
            .positions
            .values()
            .map(|position| {
                let Some(current_price) = quotes.get(&position.code).copied() else {
                    return TradePositionCurrentRow {
                        code: position.code.clone(),
                        volume: position.volume,
                        avg_cost: position.avg_cost,
                        last_trade_price: position.last_trade_price,
                        current_price: None,
                        current_market_value: None,
                        unrealized_pnl: None,
                        unrealized_pnl_pct: None,
                        quote_status: TradeQuoteStatus::Missing,
                    };
                };

                let current_market_value = Decimal::from(position.volume) * current_price;
                let cost_basis = Decimal::from(position.volume) * position.avg_cost;
                let unrealized_pnl = current_market_value - cost_basis;
                let unrealized_pnl_pct = if cost_basis.is_zero() {
                    Some(Decimal::ZERO)
                } else {
                    Some(unrealized_pnl / cost_basis * dec!(100))
                };

                TradePositionCurrentRow {
                    code: position.code.clone(),
                    volume: position.volume,
                    avg_cost: position.avg_cost,
                    last_trade_price: position.last_trade_price,
                    current_price: Some(current_price),
                    current_market_value: Some(current_market_value),
                    unrealized_pnl: Some(unrealized_pnl),
                    unrealized_pnl_pct,
                    quote_status: TradeQuoteStatus::Live,
                }
            })
            .collect()
    }
}

fn net_cash_impact(side: TradeSide, amount: Decimal, total_fee: Decimal) -> Decimal {
    match side {
        TradeSide::Buy => -(amount + total_fee),
        TradeSide::Sell => amount - total_fee,
    }
}

fn aggregate_trade_totals(state: &PaperTradeState) -> (Decimal, Decimal, Decimal) {
    state.trade_records.iter().fold(
        (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO),
        |(buy, sell, fee), record| match record.side {
            TradeSide::Buy => (buy + record.amount, sell, fee + record.total_fee),
            TradeSide::Sell => (buy, sell + record.amount, fee + record.total_fee),
        },
    )
}
