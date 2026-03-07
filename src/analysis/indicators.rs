/// 技术指标计算
///
/// MA, EMA, MACD, RSI, KDJ, BOLL, ATR, OBV, CCI, WR 等

use rust_decimal::Decimal;
use rust_decimal::prelude::*;

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
    for i in 0..period {
        sum += data[i];
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
    for i in 0..period {
        sum += data[i];
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

    for i in (period - 1)..data.len() {
        let mut weighted_sum = Decimal::ZERO;
        for j in 0..period {
            let weight = Decimal::from((j + 1) as i64);
            let idx = i + 1 - period + j;
            weighted_sum += data[idx] * weight;
        }
        result[i] = Some(weighted_sum / weight_sum);
    }

    result
}

/// 移动平均线 (SMA 别名)
pub fn ma(data: &[Decimal], period: usize) -> Vec<Option<Decimal>> {
    sma(data, period)
}

/// RSI 相对强弱指标
pub fn rsi(data: &[Decimal], period: usize) -> Vec<Option<Decimal>> {
    if data.is_empty() || period < 2 {
        return vec![None; data.len()];
    }

    let mut result = vec![None; data.len()];

    if data.len() < period + 1 {
        return result;
    }

    // 计算价格变化
    let mut gains = vec![Decimal::ZERO; data.len()];
    let mut losses = vec![Decimal::ZERO; data.len()];

    for i in 1..data.len() {
        let change = data[i] - data[i - 1];
        if change > Decimal::ZERO {
            gains[i] = change;
            losses[i] = Decimal::ZERO;
        } else {
            gains[i] = Decimal::ZERO;
            losses[i] = -change;
        }
    }

    // 初始平均涨跌
    let mut avg_gain = gains[1..=period].iter().sum::<Decimal>() / Decimal::from(period as i64);
    let mut avg_loss = losses[1..=period].iter().sum::<Decimal>() / Decimal::from(period as i64);

    if avg_loss == Decimal::ZERO {
        result[period] = Some(Decimal::from(100));
    } else {
        let rs = avg_gain / avg_loss;
        result[period] = Some(Decimal::from(100) - (Decimal::from(100) / (Decimal::ONE + rs)));
    }

    // 平滑计算后续 RSI
    for i in (period + 1)..data.len() {
        avg_gain = (avg_gain * Decimal::from(period as i64 - 1) + gains[i]) / Decimal::from(period as i64);
        avg_loss = (avg_loss * Decimal::from(period as i64 - 1) + losses[i]) / Decimal::from(period as i64);

        if avg_loss == Decimal::ZERO {
            result[i] = Some(Decimal::from(100));
        } else {
            let rs = avg_gain / avg_loss;
            result[i] = Some(Decimal::from(100) - (Decimal::from(100) / (Decimal::ONE + rs)));
        }
    }

    result
}

/// MACD 指标
#[derive(Debug, Clone)]
pub struct Macd {
    /// DIF 线 (快线)
    pub dif: Decimal,
    /// DEA 线 (慢线/信号线)
    pub dea: Decimal,
    /// MACD 柱状图
    pub macd: Decimal,
}

/// MACD 指标
pub fn macd(
    data: &[Decimal],
    fast: usize,
    slow: usize,
    signal: usize,
) -> Vec<Option<Macd>> {
    let mut result = vec![None; data.len()];

    if data.len() < slow {
        return result;
    }

    // 计算 EMA
    let ema_fast = ema(data, fast);
    let ema_slow = ema(data, slow);

    // 计算 DIF
    let mut dif_values: Vec<Decimal> = Vec::new();
    for i in 0..data.len() {
        if let (Some(f), Some(s)) = (ema_fast[i], ema_slow[i]) {
            dif_values.push(f - s);
        } else {
            dif_values.push(Decimal::ZERO);
        }
    }

    // 计算 DEA (DIF 的 EMA)
    let ema_dif = ema(&dif_values, signal);

    // 计算 MACD
    for i in 0..data.len() {
        if let (Some(dif), Some(dea)) = (ema_fast[i].and_then(|f| ema_slow[i].map(|s| f - s)), ema_dif[i]) {
            result[i] = Some(Macd {
                dif,
                dea,
                macd: (dif - dea) * Decimal::from(2),
            });
        }
    }

    result
}

/// KDJ 随机指标
#[derive(Debug, Clone, Copy)]
pub struct Kdj {
    /// K 值
    pub k: Decimal,
    /// D 值
    pub d: Decimal,
    /// J 值
    pub j: Decimal,
}

/// KDJ 指标
pub fn kdj(
    high: &[Decimal],
    low: &[Decimal],
    close: &[Decimal],
    n: usize,
    m1: usize,
    m2: usize,
) -> Vec<Option<Kdj>> {
    if high.len() != low.len() || high.len() != close.len() {
        return vec![None; high.len()];
    }

    let mut result = vec![None; high.len()];

    if high.len() < n {
        return result;
    }

    let mut prev_k = Decimal::from(50);
    let mut prev_d = Decimal::from(50);

    for i in (n - 1)..high.len() {
        // 找出 n 周期内的最高价和最低价
        let start = i + 1 - n;
        let end = i + 1;
        let window_high = &high[start..end];
        let window_low = &low[start..end];

        let highest = *window_high.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let lowest = *window_low.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        let rsv = if highest != lowest {
            (close[i] - lowest) / (highest - lowest) * Decimal::from(100)
        } else {
            Decimal::from(50)
        };

        // K = (2/3) * 前一日K + (1/3) * 当日RSV
        let k = (rsv + Decimal::from(2) * prev_k) / Decimal::from(3);
        // D = (2/3) * 前一日D + (1/3) * 当日K
        let d = (k + Decimal::from(2) * prev_d) / Decimal::from(3);
        // J = 3K - 2D
        let j = Decimal::from(3) * k - Decimal::from(2) * d;

        result[i] = Some(Kdj { k, d, j });

        prev_k = k;
        prev_d = d;
    }

    result
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

    for i in (period - 1)..data.len() {
        let start = i + 1 - period;
        let end = i + 1;
        let window = &data[start..end];

        // 计算平均值
        let sum: Decimal = window.iter().sum();
        let mean = sum / Decimal::from(period as i64);

        // 计算标准差
        let variance = window.iter()
            .map(|x| (*x - mean) * (*x - mean))
            .sum::<Decimal>() / Decimal::from(period as i64);

        // 使用 sqrt 近似 (实际项目中应该用精确的 sqrt)
        let std = sqrt_approx(variance);

        result[i] = Some(BollingerBands {
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

    if high.len() < period + 1 {
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
    for i in 1..=period {
        atr_sum += tr_values[i];
    }
    let mut atr_val = atr_sum / Decimal::from(period as i64);
    result[period] = Some(atr_val);

    for i in (period + 1)..high.len() {
        atr_val = (atr_val * Decimal::from(period as i64 - 1) + tr_values[i]) / Decimal::from(period as i64);
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
            obv_value += volume[i];
        } else if close[i] < close[i - 1] {
            obv_value -= volume[i];
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

    if high.len() < period {
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

    if high.len() < period {
        return result;
    }

    for i in (period - 1)..high.len() {
        let start = i + 1 - period;
        let end = i + 1;
        let window_high = &high[start..end];
        let window_low = &low[start..end];

        let highest = *window_high.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let lowest = *window_low.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_sma() {
        let data = vec![dec!(1), dec!(2), dec!(3), dec!(4), dec!(5)];
        let result = sma(&data, 3);

        assert_eq!(result[0], None);
        assert_eq!(result[1], None);
        assert_eq!(result[2], Some(dec!(2)));
        assert_eq!(result[3], Some(dec!(3)));
        assert_eq!(result[4], Some(dec!(4)));
    }

    #[test]
    fn test_ema() {
        let data = vec![dec!(1), dec!(2), dec!(3), dec!(4), dec!(5)];
        let result = ema(&data, 3);

        assert_eq!(result[0], None);
        assert_eq!(result[1], None);
        assert!(result[2].is_some());
        assert!(result[3].is_some());
        assert!(result[4].is_some());
    }

    #[test]
    fn test_rsi() {
        let data = vec![
            dec!(10), dec!(12), dec!(11), dec!(13), dec!(15),
            dec!(14), dec!(16), dec!(15), dec!(17), dec!(19),
            dec!(18), dec!(20), dec!(19), dec!(21),
        ];
        let result = rsi(&data, 6);

        // RSI 应该在 0-100 之间
        for i in 6..data.len() {
            if let Some(val) = result[i] {
                assert!(val >= Decimal::ZERO);
                assert!(val <= Decimal::from(100));
            }
        }
    }

    #[test]
    fn test_macd() {
        let data: Vec<Decimal> = (1..=50).map(|x| Decimal::from(x)).collect();
        let result = macd(&data, 12, 26, 9);

        // 后期应该有值
        assert!(result[40].is_some());
        assert!(result[49].is_some());
    }

    #[test]
    fn test_kdj() {
        let high: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(2)).collect();
        let low: Vec<Decimal> = (10..30).map(|x| Decimal::from(x)).collect();
        let close: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(1)).collect();

        let result = kdj(&high, &low, &close, 9, 3, 3);

        // KDJ 应该有值
        assert!(result[9].is_some());
        if let Some(kdj) = result[9] {
            assert!(kdj.k >= Decimal::ZERO && kdj.k <= Decimal::from(100));
        }
    }

    #[test]
    fn test_bollinger_bands() {
        let data: Vec<Decimal> = (1..=30).map(|x| Decimal::from(x)).collect();
        let result = bollinger_bands(&data, 20, 2);

        // 后期应该有值
        assert!(result[19].is_some());
        if let Some(boll) = result[19] {
            assert!(boll.upper > boll.middle);
            assert!(boll.lower < boll.middle);
        }
    }

    #[test]
    fn test_atr() {
        let high: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(2)).collect();
        let low: Vec<Decimal> = (10..30).map(|x| Decimal::from(x)).collect();
        let close: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(1)).collect();

        let result = atr(&high, &low, &close, 14);

        // ATR 应该是正值
        assert!(result[14].is_some());
        if let Some(atr_val) = result[14] {
            assert!(atr_val > Decimal::ZERO);
        }
    }

    #[test]
    fn test_obv() {
        let close = vec![dec!(10), dec!(11), dec!(10), dec!(12), dec!(11)];
        let volume = vec![1000, 2000, 1500, 3000, 2500];

        let result = obv(&close, &volume);

        assert_eq!(result[0], Some(1000));  // 初始值
        assert_eq!(result[1], Some(3000));  // 10→11 上涨: 1000 + 2000 = 3000
        assert_eq!(result[2], Some(1500));  // 11→10 下跌: 3000 - 1500 = 1500
        assert_eq!(result[3], Some(4500));  // 10→12 上涨: 1500 + 3000 = 4500
        assert_eq!(result[4], Some(2000));  // 12→11 下跌: 4500 - 2500 = 2000
    }

    #[test]
    fn test_cci() {
        let high: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(2)).collect();
        let low: Vec<Decimal> = (10..30).map(|x| Decimal::from(x)).collect();
        let close: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(1)).collect();

        let result = cci(&high, &low, &close, 14);

        // CCI 应该有值
        assert!(result[13].is_some());
    }

    #[test]
    fn test_williams_r() {
        let high: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(2)).collect();
        let low: Vec<Decimal> = (10..30).map(|x| Decimal::from(x)).collect();
        let close: Vec<Decimal> = (10..30).map(|x| Decimal::from(x) + dec!(1)).collect();

        let result = williams_r(&high, &low, &close, 9);

        // WR 应该在 -100 到 0 之间
        assert!(result[9].is_some());
        if let Some(wr_val) = result[9] {
            assert!(wr_val >= Decimal::from(-100));
            assert!(wr_val <= Decimal::ZERO);
        }
    }

    #[test]
    fn test_wma() {
        let data = vec![dec!(1), dec!(2), dec!(3), dec!(4), dec!(5)];
        let result = wma(&data, 3);

        assert_eq!(result[0], None);
        assert_eq!(result[1], None);
        assert!(result[2].is_some());
        // WMA 应该比 SMA 大（给近期数据更高权重）
        let sma_result = sma(&data, 3);
        if let (Some(w), Some(s)) = (result[4], sma_result[4]) {
            assert!(w >= s);
        }
    }
}
