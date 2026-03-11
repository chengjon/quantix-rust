use rust_decimal::Decimal;

use crate::trade::{FeeBreakdown, FeeConfig, TradeSide};

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
