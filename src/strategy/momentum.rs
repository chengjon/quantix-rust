/// 动量策略
///
/// 基于 MACD 指标的动量策略
/// 追踪趋势，在趋势启动时入场
use async_trait::async_trait;

use crate::analysis::indicators::macd;
use crate::core::signal::Signal;
use crate::data::models::Kline;
use crate::strategy::trait_def::Strategy;

/// 动量策略配置
#[derive(Debug, Clone)]
pub struct MomentumConfig {
    /// MACD 快线周期
    pub fast_period: usize,
    /// MACD 慢线周期
    pub slow_period: usize,
    /// MACD 信号线周期
    pub signal_period: usize,
    /// MACD 柱状图正向阈值（多头确认）
    pub macd_positive_threshold: rust_decimal::Decimal,
    /// MACD 柱状图负向阈值（空头确认）
    pub macd_negative_threshold: rust_decimal::Decimal,
    /// 是否启用 DIF-DEA 背离检测
    pub enable_divergence: bool,
}

impl Default for MomentumConfig {
    fn default() -> Self {
        Self {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            macd_positive_threshold: rust_decimal::Decimal::from(5), // MACD > 0.05 看多
            macd_negative_threshold: rust_decimal::Decimal::from(5), // MACD < -0.05 看空
            enable_divergence: false,                                // 暂不启用背离检测
        }
    }
}

/// 动量策略
pub struct MomentumStrategy {
    config: MomentumConfig,
    name: String,
    // 历史数据缓存
    close_history: Vec<rust_decimal::Decimal>,
    // MACD 历史值（用于背离检测）
    macd_history: Vec<rust_decimal::Decimal>,
    // 上一根 K 线的 MACD 值
    prev_macd: Option<rust_decimal::Decimal>,
    // 当前持仓状态
    position: bool,
}

impl MomentumStrategy {
    pub fn new(config: MomentumConfig) -> Self {
        Self {
            name: format!(
                "Momentum_MACD_{}_{}_{}",
                config.fast_period, config.slow_period, config.signal_period
            ),
            config,
            close_history: Vec::new(),
            macd_history: Vec::new(),
            prev_macd: None,
            position: false,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(MomentumConfig::default())
    }

    /// 判断 MACD 金叉（DIF 上穿 DEA，且 MACD 柱状图由负转正）
    fn is_macd_golden_cross(
        &self,
        prev_macd_value: rust_decimal::Decimal,
        curr_macd_value: rust_decimal::Decimal,
    ) -> bool {
        prev_macd_value < rust_decimal::Decimal::ZERO
            && curr_macd_value > rust_decimal::Decimal::ZERO
    }

    /// 判断 MACD 死叉（DIF 下穿 DEA，且 MACD 柱状图由正转负）
    fn is_macd_death_cross(
        &self,
        prev_macd_value: rust_decimal::Decimal,
        curr_macd_value: rust_decimal::Decimal,
    ) -> bool {
        prev_macd_value > rust_decimal::Decimal::ZERO
            && curr_macd_value < rust_decimal::Decimal::ZERO
    }
}

#[async_trait]
impl Strategy for MomentumStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    async fn on_bar(&mut self, bar: &Kline) -> Result<Signal, Box<dyn std::error::Error>> {
        // 添加收盘价到历史
        self.close_history.push(bar.close);

        // 需要足够的数据
        let required_len = self
            .config
            .fast_period
            .max(self.config.slow_period)
            .max(self.config.signal_period)
            + self.config.slow_period;
        if self.close_history.len() < required_len {
            return Ok(Signal::Hold);
        }

        // 计算 MACD
        let macd_values = macd(
            &self.close_history,
            self.config.fast_period,
            self.config.slow_period,
            self.config.signal_period,
        );

        let current_macd = match macd_values.last().unwrap() {
            Some(m) => m.macd / rust_decimal::Decimal::from(100), // 转换为合适的范围
            None => return Ok(Signal::Hold),
        };

        self.macd_history.push(current_macd);

        // 需要上一时刻的 MACD 值来判断交叉
        if let Some(prev_macd_val) = self.prev_macd {
            if self.position {
                // 持仓中，检查是否应该平仓
                if self.is_macd_death_cross(prev_macd_val, current_macd) {
                    self.position = false;
                    return Ok(Signal::Sell);
                }
            } else {
                // 空仓中，检查是否应该开仓
                if self.is_macd_golden_cross(prev_macd_val, current_macd) {
                    self.position = true;
                    return Ok(Signal::Buy);
                }
            }
        }

        // 更新上一时刻的 MACD 值
        self.prev_macd = Some(current_macd);

        Ok(Signal::Hold)
    }

    async fn finish(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.close_history.clear();
        self.macd_history.clear();
        self.prev_macd = None;
        self.position = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::test_utils::create_test_kline;
    use rust_decimal_macros::dec;

    #[test]
    fn test_macd_golden_cross() {
        let strategy = MomentumStrategy::with_defaults();

        // MACD 从负转正
        assert!(strategy.is_macd_golden_cross(dec!(-5), dec!(3)));
        assert!(strategy.is_macd_golden_cross(dec!(-1), dec!(1)));

        // MACD 没有转正
        assert!(!strategy.is_macd_golden_cross(dec!(-5), dec!(-2)));
        assert!(!strategy.is_macd_golden_cross(dec!(3), dec!(5)));
    }

    #[test]
    fn test_macd_death_cross() {
        let strategy = MomentumStrategy::with_defaults();

        // MACD 从正转负
        assert!(strategy.is_macd_death_cross(dec!(5), dec!(-3)));
        assert!(strategy.is_macd_death_cross(dec!(1), dec!(-1)));

        // MACD 没有转负
        assert!(!strategy.is_macd_death_cross(dec!(5), dec!(2)));
        assert!(!strategy.is_macd_death_cross(dec!(-3), dec!(-5)));
    }

    #[tokio::test]
    async fn test_momentum_strategy() {
        let mut strategy = MomentumStrategy::with_defaults();

        // 生成上涨趋势的测试数据
        let mut price = 100.0;
        for i in 0..50 {
            // 先下跌再上涨
            if i < 20 {
                price -= 0.5;
            } else {
                price += 1.0;
            }

            let bar = create_test_kline(i as u32, price);
            let _signal = strategy.on_bar(&bar).await.unwrap();
        }

        assert_eq!(strategy.close_history.len(), 50);
        assert_eq!(strategy.macd_history.len(), 0);
        assert_eq!(strategy.prev_macd, None);
    }
}
