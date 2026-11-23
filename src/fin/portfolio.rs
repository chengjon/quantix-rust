//! 研究侧组合分析门面：fin PositionLedger（逐笔成交 → 已实现盈亏 / 按市价估值）。
//! 与业务侧 analysis/portfolio.rs（A股 T+1 记账）并存，不互相替代。

use std::collections::HashMap;

use fin_primitives::position::{Fill, PositionLedger};
use fin_primitives::types::Side;
use fin_primitives::types::{NanoTimestamp, Price, Quantity, Symbol};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::core::Signal;
use crate::core::{QuantixError, Result};

/// 研究侧成交描述（quantix 语义，内部转 fin 类型）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillSpec {
    pub symbol: String,
    /// quantix 信号语义：Buy→Bid、Sell→Ask
    pub side: Signal,
    pub quantity: i64,
    pub price: Decimal,
    pub timestamp_nanos: i64,
    pub commission: Option<Decimal>,
}

fn to_fill(spec: &FillSpec) -> Result<Fill> {
    let symbol = Symbol::new(spec.symbol.as_str()).map_err(QuantixError::Fin)?;
    let price = Price::new(spec.price).map_err(QuantixError::Fin)?;
    let quantity = Quantity::try_from(Decimal::from(spec.quantity)).map_err(QuantixError::Fin)?;
    let side = match spec.side {
        Signal::Buy => Side::Bid,
        Signal::Sell => Side::Ask,
        Signal::Hold => {
            return Err(QuantixError::Config("FillSpec.side 不能为 Hold".into()));
        }
    };
    Ok(Fill::with_commission(
        symbol,
        side,
        quantity,
        price,
        NanoTimestamp::new(spec.timestamp_nanos),
        spec.commission.unwrap_or(Decimal::ZERO),
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerSummary {
    pub realized_pnl: Decimal,
    pub equity: Decimal,
    pub net_liquidation: Decimal,
}

/// 重放成交序列并按 `mark_prices` 估值。`mark_prices`: symbol → 市价。
pub fn ledger_summary(
    initial_cash: Decimal,
    fills: &[FillSpec],
    mark_prices: &HashMap<String, Decimal>,
) -> Result<LedgerSummary> {
    let mut ledger = PositionLedger::new(initial_cash);
    for spec in fills {
        ledger
            .apply_fill(to_fill(spec)?)
            .map_err(QuantixError::Fin)?;
    }
    let mut marks = HashMap::with_capacity(mark_prices.len());
    for (code, price) in mark_prices {
        marks.insert(code.clone(), Price::new(*price).map_err(QuantixError::Fin)?);
    }
    Ok(LedgerSummary {
        realized_pnl: ledger.realized_pnl_total(),
        equity: ledger.equity(&marks).map_err(QuantixError::Fin)?,
        net_liquidation: ledger
            .net_liquidation_value(&marks)
            .map_err(QuantixError::Fin)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn fill(side: Signal, qty: i64, price: i64) -> FillSpec {
        FillSpec {
            symbol: "600000".into(),
            side,
            quantity: qty,
            price: Decimal::from(price),
            timestamp_nanos: 0,
            commission: None,
        }
    }

    #[test]
    fn buy_sell_roundtrip_realized_pnl_and_equity() {
        let mut marks = HashMap::new();
        marks.insert("600000".to_string(), Decimal::from(12));
        let summary = ledger_summary(
            Decimal::from(10000),
            &[fill(Signal::Buy, 100, 10), fill(Signal::Sell, 100, 12)],
            &marks,
        )
        .unwrap();
        assert_eq!(summary.realized_pnl, Decimal::from(200));
        assert_eq!(summary.equity, Decimal::from(10200));
    }

    #[test]
    fn hold_side_is_rejected() {
        let marks = HashMap::new();
        let err =
            ledger_summary(Decimal::from(100), &[fill(Signal::Hold, 1, 1)], &marks).unwrap_err();
        assert!(matches!(err, QuantixError::Config(_)));
    }
}
