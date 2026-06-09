/// 技术指标计算
///
/// MA, EMA, MACD, RSI, KDJ, BOLL, ATR, OBV, CCI, WR 等
mod momentum;

#[cfg(test)]
mod tests;

pub use self::momentum::{Kdj, Macd, kdj, macd, rsi};

use rust_decimal::Decimal;

/// 简单移动平均线 (SMA)
pub fn sma(data: &[Decimal], period: usize) -> Vec<Option<Decimal>> {
    if data.is_empty() || period == 0 {
        return vec![None; data.len()];
    }

    let mut result = vec![None; data.len()];

    if data.len() < period {
        return result;
    }

    let mut sum = Decimal::ZERO;

    // 初始窗口
    for value in data.iter().take(period) {
        sum += *value;
    }
    result[period - 1] = Some(sum / Decimal::from(period as i64));

    // 滑动窗口
    for i in period..data.len() {
        sum = sum + data[i] - data[i - period];
        result[i] = Some(sum / Decimal::from(period as i64));
    }

    result
}

/// 指数移动平均线 (EMA)
pub fn ema(data: &[Decimal], period: usize) -> Vec<Option<Decimal>> {
    if data.is_empty() || period == 0 {
        return vec![None; data.len()];
    }

    let mut result = vec![None; data.len()];

    if data.len() < period {
        return result;
    }

    // 计算初始 SMA
    let mut sum = Decimal::ZERO;
    for value in data.iter().take(period) {
        sum += *value;
    }
    let mut ema_val = sum / Decimal::from(period as i64);
    result[period - 1] = Some(ema_val);

    // 平滑系数
    let alpha = Decimal::from(2) / (Decimal::from(period as i64) + Decimal::ONE);

    // 计算后续 EMA
    for i in period..data.len() {
        ema_val = data[i] * alpha + ema_val * (Decimal::ONE - alpha);
        result[i] = Some(ema_val);
    }

    result
}

/// 加权移动平均线 (WMA)
pub fn wma(data: &[Decimal], period: usize) -> Vec<Option<Decimal>> {
    if data.is_empty() || period == 0 {
        return vec![None; data.len()];
    }

    let mut result = vec![None; data.len()];

    if data.len() < period {
        return result;
    }

    // 权重和: 1 + 2 + ... + period = period * (period + 1) / 2
    let weight_sum = Decimal::from(period * (period + 1) / 2);

    for (i, slot) in result.iter_mut().enumerate().skip(period - 1) {
        let mut weighted_sum = Decimal::ZERO;
        for j in 0..period {
            let weight = Decimal::from((j + 1) as i64);
            let idx = i + 1 - period + j;
            weighted_sum += data[idx] * weight;
        }
        *slot = Some(weighted_sum / weight_sum);
    }

    result
}

/// 移动平均线 (SMA 别名)
pub fn ma(data: &[Decimal], period: usize) -> Vec<Option<Decimal>> {
    sma(data, period)
}

/// 布林带
#[derive(Debug, Clone, Copy)]
pub struct BollingerBands {
    /// 中轨 (MA)
    pub middle: Decimal,
    /// 上轨
    pub upper: Decimal,
    /// 下轨
    pub lower: Decimal,
}

/// 布林带 (BOLL)
pub fn bollinger_bands(
    data: &[Decimal],
    period: usize,
    std_dev: usize,
) -> Vec<Option<BollingerBands>> {
    let mut result = vec![None; data.len()];

    if data.is_empty() || period == 0 || data.len() < period {
        return result;
    }

    let std_dev_mult = Decimal::from(std_dev as i64);

    for (i, slot) in result.iter_mut().enumerate().skip(period - 1) {
        let start = i + 1 - period;
        let end = i + 1;
        let window = &data[start..end];

        // 计算平均值
        let sum: Decimal = window.iter().sum();
        let mean = sum / Decimal::from(period as i64);

        // 计算标准差
        let variance = window
            .iter()
            .map(|x| (*x - mean) * (*x - mean))
            .sum::<Decimal>()
            / Decimal::from(period as i64);

        // 使用 sqrt 近似 (实际项目中应该用精确的 sqrt)
        let std = sqrt_approx(variance);

        *slot = Some(BollingerBands {
            middle: mean,
            upper: mean + std_dev_mult * std,
            lower: mean - std_dev_mult * std,
        });
    }

    result
}

/// ATR 平均真实波幅
pub fn atr(
    high: &[Decimal],
    low: &[Decimal],
    close: &[Decimal],
    period: usize,
) -> Vec<Option<Decimal>> {
    if high.len() != low.len() || high.len() != close.len() {
        return vec![None; high.len()];
    }

    let mut result = vec![None; high.len()];

    if period == 0 || high.len() <= period {
        return result;
    }

    // 计算真实波幅 TR
    let mut tr_values: Vec<Decimal> = Vec::new();
    tr_values.push(high[0] - low[0]);

    for i in 1..high.len() {
        let hl = high[i] - low[i];
        let hpc = (high[i] - close[i - 1]).abs();
        let lpc = (low[i] - close[i - 1]).abs();
        tr_values.push(hl.max(hpc).max(lpc));
    }

    // 计算 ATR (使用 EMA 方法)
    let mut atr_sum = Decimal::ZERO;
    for value in tr_values.iter().take(period + 1).skip(1) {
        atr_sum += *value;
    }
    let mut atr_val = atr_sum / Decimal::from(period as i64);
    result[period] = Some(atr_val);

    for i in (period + 1)..high.len() {
        atr_val = (atr_val * Decimal::from(period as i64 - 1) + tr_values[i])
            / Decimal::from(period as i64);
        result[i] = Some(atr_val);
    }

    result
}

/// OBV 能量潮
pub fn obv(close: &[Decimal], volume: &[i64]) -> Vec<Option<i64>> {
    if close.len() != volume.len() {
        return vec![None; close.len()];
    }

    let mut result = vec![None; close.len()];
    if close.is_empty() {
        return result;
    }

    let mut obv_value = volume[0];
    result[0] = Some(obv_value);

    for i in 1..close.len() {
        if close[i] > close[i - 1] {
            obv_value = obv_value.saturating_add(volume[i]);
        } else if close[i] < close[i - 1] {
            obv_value = obv_value.saturating_sub(volume[i]);
        }
        // 收盘价相等时 OBV 不变
        result[i] = Some(obv_value);
    }

    result
}

/// CCI 顺势指标
pub fn cci(
    high: &[Decimal],
    low: &[Decimal],
    close: &[Decimal],
    period: usize,
) -> Vec<Option<Decimal>> {
    if high.len() != low.len() || high.len() != close.len() {
        return vec![None; high.len()];
    }

    let mut result = vec![None; high.len()];

    if period == 0 || high.len() < period {
        return result;
    }

    for i in (period - 1)..high.len() {
        let start = i + 1 - period;
        let end = i + 1;
        let window_high = &high[start..end];
        let window_low = &low[start..end];
        let window_close = &close[start..end];

        // 典型价格 TP = (H + L + C) / 3
        let mut tp_sum = Decimal::ZERO;
        for j in 0..period {
            tp_sum += (window_high[j] + window_low[j] + window_close[j]) / Decimal::from(3);
        }
        let ma_tp = tp_sum / Decimal::from(period as i64);

        // 计算平均绝对偏差
        let mut mad_sum = Decimal::ZERO;
        for j in 0..period {
            let tp = (window_high[j] + window_low[j] + window_close[j]) / Decimal::from(3);
            mad_sum += (tp - ma_tp).abs();
        }
        let mad = mad_sum / Decimal::from(period as i64);

        // CCI = (TP - MA_TP) / (0.015 * MAD)
        let tp = (high[i] + low[i] + close[i]) / Decimal::from(3);

        let cci_val = if mad > Decimal::ZERO {
            (tp - ma_tp) / (mad * Decimal::from(15) / Decimal::from(1000))
        } else {
            Decimal::ZERO
        };

        result[i] = Some(cci_val);
    }

    result
}

/// WR 威廉指标
pub fn williams_r(
    high: &[Decimal],
    low: &[Decimal],
    close: &[Decimal],
    period: usize,
) -> Vec<Option<Decimal>> {
    if high.len() != low.len() || high.len() != close.len() {
        return vec![None; high.len()];
    }

    let mut result = vec![None; high.len()];

    if period == 0 || high.len() < period {
        return result;
    }

    for i in (period - 1)..high.len() {
        let start = i + 1 - period;
        let end = i + 1;
        let window_high = &high[start..end];
        let window_low = &low[start..end];

        let highest = *window_high
            .iter()
            .max_by(|a, b| {
                a.partial_cmp(b)
                    .expect("decimal comparison must be defined")
            })
            .expect("Williams %R high window must be non-empty");
        let lowest = *window_low
            .iter()
            .min_by(|a, b| {
                a.partial_cmp(b)
                    .expect("decimal comparison must be defined")
            })
            .expect("Williams %R low window must be non-empty");

        let wr_val = if highest != lowest {
            (highest - close[i]) / (highest - lowest) * Decimal::from(-100)
        } else {
            Decimal::from(-50)
        };

        result[i] = Some(wr_val);
    }

    result
}

/// 简单平方根近似
fn sqrt_approx(x: Decimal) -> Decimal {
    if x < Decimal::ZERO {
        return Decimal::ZERO;
    }

    let mut guess = x / Decimal::from(2) + Decimal::ONE;
    let epsilon = Decimal::from(1) / Decimal::from(1000000); // 精度

    for _ in 0..20 {
        let new_guess = (guess + x / guess) / Decimal::from(2);
        if (new_guess - guess).abs() < epsilon {
            return new_guess;
        }
        guess = new_guess;
    }

    guess
}
