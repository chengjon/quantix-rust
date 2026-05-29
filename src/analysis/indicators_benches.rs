// 技术指标基准测试辅助函数
//
// 提供基准测试所需的公共函数

use rust_decimal::Decimal;

/// 计算简单移动平均
pub fn calculate_sma(closes: &[Decimal], period: usize) -> Vec<Decimal> {
    if period == 0 {
        return Vec::new();
    }

    closes
        .windows(period)
        .map(|w| {
            let sum: Decimal = w.iter().sum();
            sum / Decimal::from(period as i64)
        })
        .collect()
}

/// 计算指数移动平均
pub fn calculate_ema(closes: &[Decimal], period: usize) -> Vec<Decimal> {
    if closes.is_empty() || period == 0 {
        return Vec::new();
    }

    let multiplier = Decimal::from(2) / (Decimal::from(period as i64) + Decimal::from(1));
    let mut ema = vec![closes[0]];

    for current in closes.iter().skip(1) {
        let prev_ema = ema.last().unwrap();
        let new_ema = (current - prev_ema) * multiplier + prev_ema;
        ema.push(new_ema);
    }

    ema
}

/// 计算相对强弱指标 (RSI)
pub fn calculate_rsi(closes: &[Decimal], period: usize) -> Vec<Decimal> {
    if closes.is_empty() || period == 0 {
        return Vec::new();
    }

    let fifty = Decimal::from(50);
    let zero = Decimal::from(0);
    let one = Decimal::from(1);
    let hundred = Decimal::from(100);

    let mut rsi = vec![fifty; period]; // 前期填充50

    for i in period..closes.len() {
        let window = &closes[i - period..i];
        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for j in 1..window.len() {
            let change = window[j] - window[j - 1];
            if change > zero {
                gains.push(change);
            } else {
                losses.push(-change);
            }
        }

        let avg_gain: Decimal = if gains.is_empty() {
            zero
        } else {
            gains.iter().sum::<Decimal>() / Decimal::from(gains.len() as i64)
        };

        let avg_loss: Decimal = if losses.is_empty() {
            zero
        } else {
            losses.iter().sum::<Decimal>() / Decimal::from(losses.len() as i64)
        };

        if avg_loss == zero {
            rsi.push(hundred);
        } else {
            let rs = avg_gain / avg_loss;
            let rsi_value = hundred - (hundred / (one + rs));
            rsi.push(rsi_value);
        }
    }

    rsi
}

/// 计算 MACD
pub fn calculate_macd(
    closes: &[Decimal],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> (Vec<Decimal>, Vec<Decimal>, Vec<Decimal>) {
    if fast_period == 0 || slow_period == 0 || signal_period == 0 {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let ema_fast = calculate_ema(closes, fast_period);
    let ema_slow = calculate_ema(closes, slow_period);

    let macd_line: Vec<Decimal> = ema_fast
        .iter()
        .zip(ema_slow.iter())
        .map(|(fast, slow)| *fast - *slow)
        .collect();

    let signal_line = calculate_ema(&macd_line, signal_period);

    let histogram: Vec<Decimal> = macd_line
        .iter()
        .zip(signal_line.iter())
        .map(|(macd, signal)| *macd - *signal)
        .collect();

    (macd_line, signal_line, histogram)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_sma_returns_empty_for_zero_period() {
        let sma = calculate_sma(&[Decimal::from(1), Decimal::from(2)], 0);

        assert!(sma.is_empty());
    }

    #[test]
    fn calculate_rsi_returns_empty_for_zero_period() {
        let rsi = calculate_rsi(&[Decimal::from(1), Decimal::from(2)], 0);

        assert!(rsi.is_empty());
    }

    #[test]
    fn calculate_rsi_returns_empty_for_empty_input() {
        let rsi = calculate_rsi(&[], 14);

        assert!(rsi.is_empty());
    }

    #[test]
    fn calculate_ema_returns_empty_for_zero_period() {
        let ema = calculate_ema(&[Decimal::from(1), Decimal::from(2)], 0);

        assert!(ema.is_empty());
    }

    #[test]
    fn calculate_macd_returns_empty_lines_for_zero_period() {
        for periods in [(0, 26, 9), (12, 0, 9), (12, 26, 0)] {
            let (macd_line, signal_line, histogram) = calculate_macd(
                &[Decimal::from(1), Decimal::from(2)],
                periods.0,
                periods.1,
                periods.2,
            );

            assert!(macd_line.is_empty());
            assert!(signal_line.is_empty());
            assert!(histogram.is_empty());
        }
    }

    #[test]
    fn calculate_ema_returns_empty_for_empty_input() {
        let ema = calculate_ema(&[], 12);

        assert!(ema.is_empty());
    }

    #[test]
    fn calculate_macd_returns_empty_lines_for_empty_input() {
        let (macd_line, signal_line, histogram) = calculate_macd(&[], 12, 26, 9);

        assert!(macd_line.is_empty());
        assert!(signal_line.is_empty());
        assert!(histogram.is_empty());
    }
}
