use super::*;

/// RSI 相对强弱指标
pub fn rsi(data: &[Decimal], period: usize) -> Vec<Option<Decimal>> {
    if data.is_empty() || period < 2 {
        return vec![None; data.len()];
    }

    let mut result = vec![None; data.len()];

    if data.len() <= period {
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
        avg_gain =
            (avg_gain * Decimal::from(period as i64 - 1) + gains[i]) / Decimal::from(period as i64);
        avg_loss = (avg_loss * Decimal::from(period as i64 - 1) + losses[i])
            / Decimal::from(period as i64);

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
pub fn macd(data: &[Decimal], fast: usize, slow: usize, signal: usize) -> Vec<Option<Macd>> {
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
        if let (Some(dif), Some(dea)) = (
            ema_fast[i].and_then(|f| ema_slow[i].map(|s| f - s)),
            ema_dif[i],
        ) {
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
    _m1: usize,
    _m2: usize,
) -> Vec<Option<Kdj>> {
    if high.len() != low.len() || high.len() != close.len() {
        return vec![None; high.len()];
    }

    let mut result = vec![None; high.len()];

    if n == 0 || high.len() < n {
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

        let highest = *window_high
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let lowest = *window_low
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

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
