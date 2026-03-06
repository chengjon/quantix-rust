/// 技术指标计算
///
/// MA, MACD, RSI, KDJ, BOLL 等

use rust_decimal::Decimal;

/// 移动平均线
pub fn ma(data: &[Decimal], period: usize) -> Vec<Option<Decimal>> {
    if data.len() < period {
        return vec![None; data.len()];
    }

    data.windows(period)
        .map(|w| {
            let sum: Decimal = w.iter().sum();
            Some(sum / Decimal::from(period as i64))
        })
        .collect()
}

/// RSI 相对强弱指标
pub fn rsi(data: &[Decimal], period: usize) -> Vec<Option<Decimal>> {
    // TODO: 实现 RSI 计算
    vec![None; data.len()]
}

/// MACD 指标
#[derive(Debug, Clone)]
pub struct Macd {
    pub dif: Decimal,
    pub dea: Decimal,
    pub macd: Decimal,
}

pub fn macd(
    data: &[Decimal],
    fast: usize,
    slow: usize,
    signal: usize,
) -> Vec<Option<Macd>> {
    // TODO: 实现 MACD 计算
    vec![None; data.len()]
}
