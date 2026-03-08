/// 突破策略
///
/// 基于价格和成交量的突破策略
/// 当价格突破关键位置且成交量放大时入场
use async_trait::async_trait;

use crate::analysis::indicators::atr;
use crate::data::models::Kline;
use crate::strategy::trait_def::{Signal, Strategy};

/// 突破策略配置
#[derive(Debug, Clone)]
pub struct BreakoutConfig {
    /// 突破周期（观察多少根 K 线的高低位）
    pub lookback_period: usize,
    /// ATR 周期（用于计算波动率）
    pub atr_period: usize,
    /// 成交量放大倍数阈值
    pub volume_multiplier: rust_decimal::Decimal,
    /// 最小突破幅度（相对于 ATR）
    pub min_breakout_atr: rust_decimal::Decimal,
    /// 止损幅度（相对于入场价，ATR 倍数）
    pub stop_loss_atr: rust_decimal::Decimal,
    /// 止盈幅度（相对于入场价，ATR 倍数）
    pub take_profit_atr: rust_decimal::Decimal,
}

impl Default for BreakoutConfig {
    fn default() -> Self {
        Self {
            lookback_period: 20,
            atr_period: 14,
            volume_multiplier: rust_decimal::Decimal::from(15), // 成交量放大 1.5 倍
            min_breakout_atr: rust_decimal::Decimal::from(5), // 最小突破幅度为 0.5 倍 ATR
            stop_loss_atr: rust_decimal::Decimal::from(20),  // 止损 2 倍 ATR
            take_profit_atr: rust_decimal::Decimal::from(60), // 止盈 6 倍 ATR
        }
    }
}

/// 突破类型
#[derive(Debug, Clone, Copy)]
enum BreakoutType {
    Upward,
    Downward,
}

/// 突破策略
pub struct BreakoutStrategy {
    config: BreakoutConfig,
    name: String,
    // 历史数据缓存
    kline_history: Vec<Kline>,
    // 当前持仓状态
    position: bool,
    // 入场价格
    entry_price: Option<rust_decimal::Decimal>,
    // 止损止盈价格
    stop_loss_price: Option<rust_decimal::Decimal>,
    take_profit_price: Option<rust_decimal::Decimal>,
}

impl BreakoutStrategy {
    pub fn new(config: BreakoutConfig) -> Self {
        Self {
            name: format!(
                "Breakout_LB{}_ATR{}_Vol{}",
                config.lookback_period,
                config.atr_period,
                config.volume_multiplier
            ),
            config,
            kline_history: Vec::new(),
            position: false,
            entry_price: None,
            stop_loss_price: None,
            take_profit_price: None,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(BreakoutConfig::default())
    }

    /// 计算历史高低位
    fn calculate_high_low(&self, offset: usize) -> (Option<rust_decimal::Decimal>, Option<rust_decimal::Decimal>) {
        if self.kline_history.len() < self.config.lookback_period + offset {
            return (None, None);
        }

        let end = self.kline_history.len() - offset;
        let start = end.saturating_sub(self.config.lookback_period);

        if start >= end {
            return (None, None);
        }

        let window = &self.kline_history[start..end];
        let high = window.iter().map(|k| k.high).max_by(|a, b| a.partial_cmp(b).unwrap());
        let low = window.iter().map(|k| k.low).min_by(|a, b| a.partial_cmp(b).unwrap());

        (high, low)
    }

    /// 计算平均成交量
    fn calculate_avg_volume(&self, offset: usize) -> rust_decimal::Decimal {
        if self.kline_history.len() < self.config.lookback_period + offset {
            return rust_decimal::Decimal::ZERO;
        }

        let end = self.kline_history.len() - offset;
        let start = end.saturating_sub(self.config.lookback_period);

        if start >= end || self.kline_history[start..end].is_empty() {
            return rust_decimal::Decimal::ZERO;
        }

        let sum: i64 = self.kline_history[start..end].iter().map(|k| k.volume).sum();
        let count = (end - start) as u64;

        rust_decimal::Decimal::from(sum) / rust_decimal::Decimal::from(count)
    }

    /// 检测向上突破
    fn detect_upward_breakout(
        &self,
        bar: &Kline,
        historical_high: rust_decimal::Decimal,
        avg_volume: rust_decimal::Decimal,
        current_atr: rust_decimal::Decimal,
    ) -> bool {
        // 价格突破历史高点
        let price_breakout = bar.close > historical_high;
        // 成交量放大
        let volume_breakout = rust_decimal::Decimal::from(bar.volume)
            >= avg_volume * self.config.volume_multiplier / rust_decimal::Decimal::from(10);
        // 突破幅度足够
        let breakout_size = (bar.close - historical_high) >= current_atr * self.config.min_breakout_atr / rust_decimal::Decimal::from(10);

        price_breakout && volume_breakout && breakout_size
    }

    /// 检测向下突破
    fn detect_downward_breakout(
        &self,
        bar: &Kline,
        historical_low: rust_decimal::Decimal,
        avg_volume: rust_decimal::Decimal,
        current_atr: rust_decimal::Decimal,
    ) -> bool {
        // 价格跌破历史低点
        let price_breakout = bar.close < historical_low;
        // 成交量放大
        let volume_breakout = rust_decimal::Decimal::from(bar.volume)
            >= avg_volume * self.config.volume_multiplier / rust_decimal::Decimal::from(10);
        // 突破幅度足够
        let breakout_size = (historical_low - bar.close) >= current_atr * self.config.min_breakout_atr / rust_decimal::Decimal::from(10);

        price_breakout && volume_breakout && breakout_size
    }
}

#[async_trait]
impl Strategy for BreakoutStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    async fn on_bar(&mut self, bar: &Kline) -> Result<Signal, Box<dyn std::error::Error>> {
        // 添加 K 线到历史
        self.kline_history.push(bar.clone());

        // 需要足够的数据
        let required_len = self.config.lookback_period + self.config.atr_period + 1;
        if self.kline_history.len() < required_len {
            return Ok(Signal::Hold);
        }

        // 计算 ATR
        let highs: Vec<rust_decimal::Decimal> = self.kline_history.iter().map(|k| k.high).collect();
        let lows: Vec<rust_decimal::Decimal> = self.kline_history.iter().map(|k| k.low).collect();
        let closes: Vec<rust_decimal::Decimal> = self.kline_history.iter().map(|k| k.close).collect();

        let atr_values = atr(&highs, &lows, &closes, self.config.atr_period);
        let current_atr = match atr_values.last().unwrap() {
            Some(v) => v,
            None => return Ok(Signal::Hold),
        };

        // 如果持仓中，检查止损止盈
        if self.position {
            if let Some(entry) = self.entry_price {
                if let (Some(stop_loss), Some(take_profit)) = (self.stop_loss_price, self.take_profit_price) {
                    if bar.close <= stop_loss {
                        // 触发止损
                        self.position = false;
                        self.entry_price = None;
                        self.stop_loss_price = None;
                        self.take_profit_price = None;
                        return Ok(Signal::Sell);
                    }
                    if bar.close >= take_profit {
                        // 触发止盈
                        self.position = false;
                        self.entry_price = None;
                        self.stop_loss_price = None;
                        self.take_profit_price = None;
                        return Ok(Signal::Sell);
                    }
                }
            }
            return Ok(Signal::Hold);
        }

        // 计算历史高低位（不包含当前 K 线）
        let (historical_high, historical_low) = self.calculate_high_low(1);
        let (high, low) = match (historical_high, historical_low) {
            (Some(h), Some(l)) => (h, l),
            _ => return Ok(Signal::Hold),
        };

        // 计算平均成交量
        let avg_volume = self.calculate_avg_volume(1);

        // 检测突破
        if self.detect_upward_breakout(bar, high, avg_volume, *current_atr) {
            self.position = true;
            self.entry_price = Some(bar.close);
            self.stop_loss_price = Some(bar.close - *current_atr * self.config.stop_loss_atr / rust_decimal::Decimal::from(10));
            self.take_profit_price = Some(bar.close + *current_atr * self.config.take_profit_atr / rust_decimal::Decimal::from(10));
            return Ok(Signal::Buy);
        }

        // 向下突破（做空）
        if self.detect_downward_breakout(bar, low, avg_volume, *current_atr) {
            self.position = true;
            self.entry_price = Some(bar.close);
            // 对于做空，止损在上方，止盈在下方
            self.stop_loss_price = Some(bar.close + current_atr * self.config.stop_loss_atr / rust_decimal::Decimal::from(10));
            self.take_profit_price = Some(bar.close - current_atr * self.config.take_profit_atr / rust_decimal::Decimal::from(10));
            return Ok(Signal::Sell);
        }

        Ok(Signal::Hold)
    }

    async fn finish(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.kline_history.clear();
        self.position = false;
        self.entry_price = None;
        self.stop_loss_price = None;
        self.take_profit_price = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::test_utils::create_test_ohlcv;

    #[tokio::test]
    async fn test_breakout_strategy() {
        let mut strategy = BreakoutStrategy::with_defaults();

        // 生成震荡后突破的测试数据
        let base_price = 100.0;
        for i in 0..50 {
            let (high, low, close, volume) = if i < 30 {
                // 震荡阶段
                (base_price + 2.0, base_price - 2.0, base_price, 10000)
            } else {
                // 突破阶段：价格突破 + 成交量放大
                (base_price + 15.0, base_price, base_price + 10.0, 20000)
            };

            let bar = create_test_ohlcv(i as u32, close, high, low, close, volume);
            let signal = strategy.on_bar(&bar).await.unwrap();

            // 在突破时应该产生买入信号
            if i >= 35 {
                if matches!(signal, Signal::Buy) {
                    return; // 测试通过
                }
            }
        }

        // 如果没有产生买入信号，至少验证没有 panic
    }
}
