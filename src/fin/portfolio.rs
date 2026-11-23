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
    /// 当前账目重放路径不消费时间戳（上游 apply_fill 不读），字段仅为 Fill 构造完备性保留。
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

/// 门面层守卫：quantix A股语义无做空机制，超额卖出与零/负数量视为输入数据错误。
/// 上游本身允许超额静默转空头、允许 0 数量，quantix 不予放行。
fn validate_fills(fills: &[FillSpec]) -> Result<()> {
    let mut positions: HashMap<&str, Decimal> = HashMap::new();
    for spec in fills {
        if spec.quantity <= 0 {
            return Err(QuantixError::Config(format!(
                "无效数量 symbol={} qty={}",
                spec.symbol, spec.quantity
            )));
        }
        let qty = Decimal::from(spec.quantity);
        let pos = positions.entry(spec.symbol.as_str()).or_default();
        match spec.side {
            Signal::Buy => *pos += qty,
            Signal::Sell => {
                if qty > *pos {
                    return Err(QuantixError::Config(format!(
                        "超额卖出: {} 卖出 {} 超过持仓 {}",
                        spec.symbol, qty, *pos
                    )));
                }
                *pos -= qty;
            }
            Signal::Hold => {} // 由 to_fill 拒绝（Config），此处不跟踪
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerSummary {
    pub realized_pnl: Decimal,
    pub equity: Decimal,
    pub net_liquidation: Decimal,
}

/// 重放成交序列并按 `mark_prices` 估值。`mark_prices`: symbol → 市价。
/// 门面层拒绝超额卖出与零/负数量；上游本身允许超额转空头，quantix A股语义不予放行。
/// 口径注记（上游已核实）：`equity` = cash + 未实现盈亏（非市值）；`net_liquidation` =
/// cash + Σ(持仓×mark)（市值口径）。
pub fn ledger_summary(
    initial_cash: Decimal,
    fills: &[FillSpec],
    mark_prices: &HashMap<String, Decimal>,
) -> Result<LedgerSummary> {
    validate_fills(fills)?;
    let mut ledger = PositionLedger::new(initial_cash);
    for (i, spec) in fills.iter().enumerate() {
        ledger.apply_fill(to_fill(spec)?).map_err(|e| {
            QuantixError::DataParse(format!("成交重放失败 #{i} symbol={}: {e}", spec.symbol))
        })?;
    }
    let mut marks = HashMap::with_capacity(mark_prices.len());
    for (code, price) in mark_prices {
        marks.insert(
            code.clone(),
            Price::new(*price).map_err(|e| {
                QuantixError::DataParse(format!("mark 价格无效 symbol={code}: {e}"))
            })?,
        );
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

    #[test]
    fn commission_reduces_cash_but_not_realized_pnl() {
        let mut marks = HashMap::new();
        marks.insert("600000".to_string(), Decimal::from(10));
        let mut fee_fill = fill(Signal::Buy, 100, 10);
        fee_fill.commission = Some(Decimal::from(5));
        let summary = ledger_summary(Decimal::from(10000), &[fee_fill], &marks).unwrap();
        // 上游语义（已核实 position/mod.rs apply_fill/realized_pnl）：佣金进 cash 也进 realized
        assert_eq!(summary.realized_pnl, Decimal::from(-5));
        assert_eq!(summary.equity, Decimal::from(8995)); // 零费 9000 - 佣金 5
        assert_eq!(summary.net_liquidation, Decimal::from(9995)); // cash 8995 + 持仓市值 1000
    }

    #[test]
    fn insufficient_cash_is_reported_with_replay_index() {
        let marks = HashMap::new();
        let err =
            ledger_summary(Decimal::from(500), &[fill(Signal::Buy, 100, 10)], &marks).unwrap_err();
        assert!(
            matches!(err, QuantixError::DataParse(_)),
            "应收敛为 DataParse: {err:?}"
        );
        let text = err.to_string();
        assert!(text.contains("成交重放失败 #0"), "应含重放下标: {text}");
    }

    #[test]
    fn oversell_is_rejected_as_config_error() {
        let marks = HashMap::new();
        let err = ledger_summary(
            Decimal::from(10000),
            &[fill(Signal::Buy, 50, 10), fill(Signal::Sell, 100, 10)],
            &marks,
        )
        .unwrap_err();
        assert!(matches!(err, QuantixError::Config(_)));
        assert!(err.to_string().contains("超额卖出"));
    }

    #[test]
    fn zero_quantity_is_rejected() {
        let marks = HashMap::new();
        let err =
            ledger_summary(Decimal::from(100), &[fill(Signal::Buy, 0, 10)], &marks).unwrap_err();
        assert!(matches!(err, QuantixError::Config(_)));
        assert!(err.to_string().contains("无效数量"));
    }

    #[test]
    fn multi_symbol_positions_are_independent() {
        let mut marks = HashMap::new();
        marks.insert("600000".to_string(), Decimal::from(10));
        marks.insert("000001".to_string(), Decimal::from(12));
        let mut buy_b = fill(Signal::Buy, 100, 10);
        buy_b.symbol = "000001".into();
        let mut sell_b = fill(Signal::Sell, 100, 12);
        sell_b.symbol = "000001".into();
        let summary = ledger_summary(
            Decimal::from(10000),
            &[fill(Signal::Buy, 100, 10), buy_b, sell_b],
            &marks,
        )
        .unwrap();
        assert_eq!(summary.realized_pnl, Decimal::from(200)); // 仅 B 平仓盈利
        // 上游 equity 口径 = cash + 未实现盈亏（非市值）：cash 9200 + A 未实现 0
        assert_eq!(summary.equity, Decimal::from(9200));
        // 市值口径走 net_liquidation：cash 9200 + A 持仓市值 100×10
        assert_eq!(summary.net_liquidation, Decimal::from(10200));
    }
}
