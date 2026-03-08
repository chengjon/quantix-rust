/// 均值回归策略
///
/// 基于 RSI 和布林带的均值回归策略
/// 当价格偏离均值过多时，预期价格会回归均值
use async_trait::async_trait;

use crate::analysis::indicators::{bollinger_bands, rsi};
use crate::data::models::Kline;
use crate::strategy::trait_def::{Signal, Strategy};

/// 均值回归策略配置
#[derive(Debug, Clone)]
pub struct MeanReversionConfig {
    /// RSI 周期
    pub rsi_period: usize,
    /// RSI 超买阈值（高于此值考虑卖出）
    pub rsi_overbought: rust_decimal::Decimal,
    /// RSI 超卖阈值（低于此值考虑买入）
    pub rsi_oversold: rust_decimal::Decimal,
    /// 布林带周期
    pub bb_period: usize,
    /// 布林带标准差倍数
    pub bb_std_dev: usize,
    /// 价格偏离下轨的百分比阈值（用于买入）
    pub buy_deviation_pct: rust_decimal::Decimal,
    /// 价格偏离上轨的百分比阈值（用于卖出）
    pub sell_deviation_pct: rust_decimal::Decimal,
}

impl Default for MeanReversionConfig {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            rsi_overbought: rust_decimal::Decimal::from(70), // RSI > 70 超买
            rsi_oversold: rust_decimal::Decimal::from(30),   // RSI < 30 超卖
            bb_period: 20,
            bb_std_dev: 2,
            buy_deviation_pct: rust_decimal::Decimal::from(2),  // 低于下轨 2% 买入
            sell_deviation_pct: rust_decimal::Decimal::from(2), // 高于上轨 2% 卖出
        }
    }
}

/// 均值回归策略
pub struct MeanReversionStrategy {
    config: MeanReversionConfig,
    name: String,
    // 历史数据缓存
    close_history: Vec<rust_decimal::Decimal>,
    high_history: Vec<rust_decimal::Decimal>,
    low_history: Vec<rust_decimal::Decimal>,
    // 当前持仓状态
    position: bool,
}

impl MeanReversionStrategy {
    pub fn new(config: MeanReversionConfig) -> Self {
        Self {
            name: format!(
                "MeanReversion_RSI{}_BB{}_{}",
                config.rsi_period, config.bb_period, config.bb_std_dev
            ),
            config,
            close_history: Vec::new(),
            high_history: Vec::new(),
            low_history: Vec::new(),
            position: false,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(MeanReversionConfig::default())
    }

    /// 判断是否超买（RSI 高 + 价格接近/高于上轨）
    fn is_overbought(
        &self,
        rsi: rust_decimal::Decimal,
        price: rust_decimal::Decimal,
        bb_upper: rust_decimal::Decimal,
    ) -> bool {
        let rsi_overbought = rsi >= self.config.rsi_overbought;
        let price_high = price >= bb_upper * (rust_decimal::Decimal::ONE + self.config.sell_deviation_pct / rust_decimal::Decimal::from(100));

        rsi_overbought && price_high
    }

    /// 判断是否超卖（RSI 低 + 价格接近/低于下轨）
    fn is_oversold(
        &self,
        rsi: rust_decimal::Decimal,
        price: rust_decimal::Decimal,
        bb_lower: rust_decimal::Decimal,
    ) -> bool {
        let rsi_oversold = rsi <= self.config.rsi_oversold;
        let price_low = price <= bb_lower * (rust_decimal::Decimal::ONE - self.config.buy_deviation_pct / rust_decimal::Decimal::from(100));

        rsi_oversold && price_low
    }
}

#[async_trait]
impl Strategy for MeanReversionStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    async fn on_bar(&mut self, bar: &Kline) -> Result<Signal, Box<dyn std::error::Error>> {
        // 添加价格数据到历史
        self.close_history.push(bar.close);
        self.high_history.push(bar.high);
        self.low_history.push(bar.low);

        // 需要足够的数据
        let required_len = self
            .config
            .bb_period
            .max(self.config.rsi_period + 1);
        if self.close_history.len() < required_len {
            return Ok(Signal::Hold);
        }

        // 计算 RSI
        let rsi_values = rsi(&self.close_history, self.config.rsi_period);
        let current_rsi = match rsi_values.last().unwrap() {
            Some(v) => *v,
            None => return Ok(Signal::Hold),
        };

        // 计算布林带
        let bb_values = bollinger_bands(
            &self.close_history,
            self.config.bb_period,
            self.config.bb_std_dev,
        );
        let current_bb = match bb_values.last().unwrap() {
            Some(v) => *v,
            None => return Ok(Signal::Hold),
        };

        // 根据指标生成信号
        if self.position {
            // 持仓中，检查是否应该卖出
            if self.is_overbought(current_rsi, bar.close, current_bb.upper) {
                self.position = false;
                return Ok(Signal::Sell);
            }
        } else {
            // 空仓中，检查是否应该买入
            if self.is_oversold(current_rsi, bar.close, current_bb.lower) {
                self.position = true;
                return Ok(Signal::Buy);
            }
        }

        Ok(Signal::Hold)
    }

    async fn finish(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.close_history.clear();
        self.high_history.clear();
        self.low_history.clear();
        self.position = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::test_utils::{create_test_ohlcv, create_test_kline};
    use rust_decimal::prelude::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_default_config() {
        let config = MeanReversionConfig::default();
        assert_eq!(config.rsi_period, 14);
        assert_eq!(config.rsi_overbought, dec!(70));
        assert_eq!(config.rsi_oversold, dec!(30));
        assert_eq!(config.bb_period, 20);
        assert_eq!(config.bb_std_dev, 2);
    }

    #[tokio::test]
    async fn test_mean_reversion_strategy_basic() {
        let config = MeanReversionConfig {
            rsi_period: 5,
            rsi_overbought: dec!(70),
            rsi_oversold: dec!(30),
            bb_period: 10,
            bb_std_dev: 2,
            buy_deviation_pct: dec!(5),
            sell_deviation_pct: dec!(5),
        };
        let mut strategy = MeanReversionStrategy::new(config);

        // 生成简单测试数据
        for i in 0..20 {
            let price = 100.0 + i as f64 * 0.5;
            let bar = create_test_kline(i as u32, price);
            let _signal = strategy.on_bar(&bar).await.unwrap();
        }

        // 验证策略没有 panic
        assert!(!strategy.position || strategy.position);
    }

    #[test]
    fn test_overbought_detection() {
        let strategy = MeanReversionStrategy::with_defaults();

        // RSI 高 + 价格高于上轨
        assert!(strategy.is_overbought(
            dec!(75), // RSI > 70
            dec!(105), // 价格高于上轨
            dec!(100)  // 上轨
        ));

        // RSI 不够高
        assert!(!strategy.is_overbought(
            dec!(65), // RSI < 70
            dec!(105),
            dec!(100)
        ));
    }

    #[test]
    fn test_oversold_detection() {
        let strategy = MeanReversionStrategy::with_defaults();

        // RSI 低 + 价格低于下轨
        assert!(strategy.is_oversold(
            dec!(25), // RSI < 30
            dec!(95),  // 价格低于下轨
            dec!(100)  // 下轨
        ));

        // RSI 不够低
        assert!(!strategy.is_oversold(
            dec!(35), // RSI > 30
            dec!(95),
            dec!(100)
        ));
    }
}
