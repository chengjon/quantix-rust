/// 网格交易策略
///
/// 适合震荡市场的网格交易策略
/// 在价格区间内分批买入卖出，赚取波段利润
use async_trait::async_trait;

use crate::analysis::indicators::atr;
use crate::core::signal::Signal;
use crate::data::models::Kline;
use crate::strategy::trait_def::Strategy;

/// 网格交易配置
#[derive(Debug, Clone)]
pub struct GridConfig {
    /// 网格数量（将价格区间分成多少格）
    pub grid_count: usize,
    /// ATR 周期（用于计算价格区间）
    pub atr_period: usize,
    /// 价格区间倍数（中心价格 ± ATR * 倍数）
    pub range_multiplier: rust_decimal::Decimal,
    /// 每格的资金比例（总资金 / 网格数）
    pub position_size_pct: rust_decimal::Decimal,
    /// 是否动态调整网格
    pub dynamic_adjustment: bool,
    /// 动态调整周期（多少根 K 线重新计算一次）
    pub adjustment_period: usize,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            grid_count: 10,
            atr_period: 14,
            range_multiplier: rust_decimal::Decimal::from(20), // 2倍 ATR
            position_size_pct: rust_decimal::Decimal::from(10), // 每格 10%
            dynamic_adjustment: true,
            adjustment_period: 100,
        }
    }
}

/// 网格订单
#[derive(Debug, Clone)]
struct GridOrder {
    price: rust_decimal::Decimal,
    quantity: rust_decimal::Decimal,
    is_buy: bool, // true = 买单, false = 卖单
    filled: bool,
}

/// 网格交易策略
pub struct GridStrategy {
    config: GridConfig,
    name: String,
    // 历史数据
    close_history: Vec<rust_decimal::Decimal>,
    high_history: Vec<rust_decimal::Decimal>,
    low_history: Vec<rust_decimal::Decimal>,
    // 网格参数
    center_price: Option<rust_decimal::Decimal>,
    upper_bound: Option<rust_decimal::Decimal>,
    lower_bound: Option<rust_decimal::Decimal>,
    grid_spacing: Option<rust_decimal::Decimal>,
    // 网格订单
    grid_orders: Vec<GridOrder>,
    // 当前持仓
    current_position: rust_decimal::Decimal,
    // 最后调整的 K 线计数
    last_adjustment_bars: usize,
}

impl GridStrategy {
    pub fn new(config: GridConfig) -> Self {
        Self {
            name: format!(
                "Grid_G{}_ATR{}_R{}",
                config.grid_count, config.atr_period, config.range_multiplier
            ),
            config,
            close_history: Vec::new(),
            high_history: Vec::new(),
            low_history: Vec::new(),
            center_price: None,
            upper_bound: None,
            lower_bound: None,
            grid_spacing: None,
            grid_orders: Vec::new(),
            current_position: rust_decimal::Decimal::ZERO,
            last_adjustment_bars: 0,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(GridConfig::default())
    }

    /// 初始化网格
    fn initialize_grid(
        &mut self,
        current_price: rust_decimal::Decimal,
        current_atr: rust_decimal::Decimal,
    ) {
        // 计算中心价格为当前价格
        self.center_price = Some(current_price);

        // 计算上下边界
        let half_range =
            current_atr * self.config.range_multiplier / rust_decimal::Decimal::from(10);
        self.upper_bound = Some(current_price + half_range);
        self.lower_bound = Some(current_price - half_range);

        // 计算网格间距
        let range = self.upper_bound.unwrap() - self.lower_bound.unwrap();
        self.grid_spacing =
            Some(range / rust_decimal::Decimal::from(self.config.grid_count as u64));

        // 生成网格订单
        self.generate_grid_orders();
    }

    /// 生成网格订单
    fn generate_grid_orders(&mut self) {
        self.grid_orders.clear();

        let spacing = match self.grid_spacing {
            Some(s) => s,
            None => return,
        };

        let lower = match self.lower_bound {
            Some(l) => l,
            None => return,
        };

        // 在当前价格下方生成买单，上方生成卖单
        let current_price = match self.center_price {
            Some(p) => p,
            None => return,
        };

        for i in 0..=self.config.grid_count {
            let price = lower + spacing * rust_decimal::Decimal::from(i as u64);

            if price < current_price {
                // 低于当前价格，挂买单
                self.grid_orders.push(GridOrder {
                    price,
                    quantity: rust_decimal::Decimal::from(100), // 默认数量
                    is_buy: true,
                    filled: false,
                });
            } else if price > current_price {
                // 高于当前价格，挂卖单
                self.grid_orders.push(GridOrder {
                    price,
                    quantity: rust_decimal::Decimal::from(100),
                    is_buy: false,
                    filled: false,
                });
            }
        }
    }

    /// 检查是否触发网格订单
    fn check_grid_triggers(&mut self, bar: &Kline) -> Option<Signal> {
        for order in &mut self.grid_orders {
            if order.filled {
                continue;
            }

            if order.is_buy {
                // 检查买单触发（价格跌到网格线）
                if bar.close <= order.price {
                    order.filled = true;
                    self.current_position += order.quantity;
                    // 同时在上方对应位置挂卖单
                    return Some(Signal::Buy);
                }
            } else {
                // 检查卖单触发（价格涨到网格线）
                if bar.close >= order.price {
                    order.filled = true;
                    if self.current_position >= order.quantity {
                        self.current_position -= order.quantity;
                    } else {
                        self.current_position = rust_decimal::Decimal::ZERO;
                    }
                    return Some(Signal::Sell);
                }
            }
        }

        None
    }
}

#[async_trait]
impl Strategy for GridStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    async fn on_bar(&mut self, bar: &Kline) -> Result<Signal, Box<dyn std::error::Error>> {
        // 添加价格数据
        self.close_history.push(bar.close);
        self.high_history.push(bar.high);
        self.low_history.push(bar.low);

        // 需要足够的数据
        if self.close_history.len() < self.config.atr_period + 1 {
            return Ok(Signal::Hold);
        }

        // 计算 ATR
        let atr_values = atr(
            &self.high_history,
            &self.low_history,
            &self.close_history,
            self.config.atr_period,
        );

        let current_atr = match atr_values.last().unwrap() {
            Some(v) => v,
            None => return Ok(Signal::Hold),
        };

        // 首次初始化或动态调整网格
        if self.center_price.is_none() {
            self.initialize_grid(bar.close, *current_atr);
        } else if self.config.dynamic_adjustment
            && self.close_history.len() - self.last_adjustment_bars > self.config.adjustment_period
        {
            // 重新计算网格
            self.initialize_grid(bar.close, *current_atr);
            self.last_adjustment_bars = self.close_history.len();
        }

        // 检查网格触发
        if let Some(signal) = self.check_grid_triggers(bar) {
            return Ok(signal);
        }

        Ok(Signal::Hold)
    }

    async fn finish(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.close_history.clear();
        self.high_history.clear();
        self.low_history.clear();
        self.center_price = None;
        self.upper_bound = None;
        self.lower_bound = None;
        self.grid_spacing = None;
        self.grid_orders.clear();
        self.current_position = rust_decimal::Decimal::ZERO;
        self.last_adjustment_bars = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::test_utils::create_test_kline;
    use rust_decimal::prelude::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_default_config() {
        let config = GridConfig::default();
        assert_eq!(config.grid_count, 10);
        assert_eq!(config.atr_period, 14);
        assert_eq!(config.dynamic_adjustment, true);
    }

    #[tokio::test]
    async fn test_grid_strategy_initialization() {
        let config = GridConfig {
            grid_count: 5,
            atr_period: 10,
            range_multiplier: dec!(20),
            position_size_pct: dec!(10),
            dynamic_adjustment: false,
            adjustment_period: 100,
        };

        let mut strategy = GridStrategy::new(config);

        // 生成足够的测试数据
        for i in 0..20 {
            let price = 100.0 + (i as f64 * 0.1);
            let bar = create_test_kline(i as u32, price);
            let _ = strategy.on_bar(&bar).await;
        }

        // 验证网格已初始化
        assert!(strategy.center_price.is_some());
        assert!(strategy.upper_bound.is_some());
        assert!(strategy.lower_bound.is_some());
        assert!(strategy.grid_spacing.is_some());
    }

    #[tokio::test]
    async fn test_grid_strategy_signals() {
        let mut strategy = GridStrategy::with_defaults();

        // 生成震荡行情测试数据
        for i in 0..50 {
            // 震荡: 95-105
            let price = if i % 2 == 0 {
                95.0 + (i % 10) as f64
            } else {
                105.0 - (i % 10) as f64
            };

            let bar = create_test_kline(i as u32, price);
            let signal = strategy.on_bar(&bar).await.unwrap();

            // 验证策略不会 panic
            match signal {
                Signal::Buy | Signal::Sell | Signal::Hold => {}
            }
        }
    }
}
