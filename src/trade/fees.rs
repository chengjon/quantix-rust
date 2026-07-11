use rust_decimal::Decimal;

use crate::trade::{FeeBreakdown, FeeConfig, TradeSide};

/// 按成交金额、买卖方向与代码所属交易所拆解手续费。
///
/// 包含：佣金（取 `commission_rate * amount` 与 `commission_min` 的较大值）、
/// 印花税（仅卖出，`stamp_duty_rate * amount`）、过户费（仅沪市 `60`/`68` 开头代码）。
pub fn calculate_fee_breakdown(
    side: TradeSide,
    code: &str,
    amount: Decimal,
    fee_config: &FeeConfig,
) -> FeeBreakdown {
    let commission = (amount * fee_config.commission_rate).max(fee_config.commission_min);
    let stamp_duty = match side {
        TradeSide::Buy => Decimal::ZERO,
        TradeSide::Sell => amount * fee_config.stamp_duty_rate,
    };
    let transfer_fee = if is_shanghai_code(code) {
        amount * fee_config.transfer_fee_rate
    } else {
        Decimal::ZERO
    };
    let total_fee = commission + stamp_duty + transfer_fee;

    FeeBreakdown {
        commission,
        stamp_duty,
        transfer_fee,
        total_fee,
    }
}

fn is_shanghai_code(code: &str) -> bool {
    code.starts_with("60") || code.starts_with("68")
}
