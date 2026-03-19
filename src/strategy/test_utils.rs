/// 策略测试工具模块
///
/// 提供可配置的测试数据生成器
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

use crate::data::models::{AdjustType, Kline};

/// 测试数据生成配置
#[derive(Debug, Clone)]
pub struct TestDataConfig {
    /// 价格买卖价差（用于生成 high/low）
    pub spread: Decimal,
    /// 基础成交量
    pub base_volume: i64,
    /// 是否使用金额字段
    pub include_amount: bool,
}

impl Default for TestDataConfig {
    fn default() -> Self {
        Self {
            spread: dec!(1.0),      // 默认 1 元价差
            base_volume: 1_000_000, // 默认 100 万股
            include_amount: true,
        }
    }
}

impl TestDataConfig {
    /// 创建紧凑价差配置（用于高频测试）
    pub fn tight_spread() -> Self {
        Self {
            spread: dec!(0.1),
            ..Default::default()
        }
    }

    /// 创建宽价差配置（用于波动性测试）
    pub fn wide_spread() -> Self {
        Self {
            spread: dec!(5.0),
            ..Default::default()
        }
    }

    /// 创建自定义成交量配置
    pub fn with_volume(volume: i64) -> Self {
        Self {
            base_volume: volume,
            ..Default::default()
        }
    }
}

/// K线测试数据生成器
pub struct KlineBuilder {
    code: String,
    date: u32,
    config: TestDataConfig,
}

impl KlineBuilder {
    /// 创建新的 K线生成器
    pub fn new(code: &str, date: u32) -> Self {
        Self {
            code: code.to_string(),
            date,
            config: TestDataConfig::default(),
        }
    }

    /// 设置测试数据配置
    pub fn with_config(mut self, config: TestDataConfig) -> Self {
        self.config = config;
        self
    }

    /// 从收盘价生成 K线（使用默认价差）
    pub fn from_close(&self, close: f64) -> Kline {
        self.from_close_with_spread(close, self.config.spread)
    }

    /// 从收盘价生成 K线（自定义价差）
    pub fn from_close_with_spread(&self, close: f64, spread: Decimal) -> Kline {
        let close_dec = Decimal::from_str(close.to_string().as_str()).unwrap();
        let spread_dec = spread;

        Kline {
            code: self.code.clone(),
            date: NaiveDate::from_num_days_from_ce_opt(self.date as i32).unwrap(),
            open: close_dec,
            high: close_dec + spread_dec,
            low: close_dec - spread_dec,
            close: close_dec,
            volume: self.config.base_volume,
            amount: if self.config.include_amount {
                Some(close_dec * Decimal::from(self.config.base_volume))
            } else {
                None
            },
            adjust_type: AdjustType::None,
        }
    }

    /// 生成完整的 OHLCV K线
    pub fn from_ohlcv(&self, open: f64, high: f64, low: f64, close: f64, volume: i64) -> Kline {
        let close_dec = Decimal::from_str(close.to_string().as_str()).unwrap();

        Kline {
            code: self.code.clone(),
            date: NaiveDate::from_num_days_from_ce_opt(self.date as i32).unwrap(),
            open: Decimal::from_str(open.to_string().as_str()).unwrap(),
            high: Decimal::from_str(high.to_string().as_str()).unwrap(),
            low: Decimal::from_str(low.to_string().as_str()).unwrap(),
            close: close_dec,
            volume,
            amount: if self.config.include_amount {
                Some(close_dec * Decimal::from(volume))
            } else {
                None
            },
            adjust_type: AdjustType::None,
        }
    }

    /// 批量生成价格序列
    pub fn generate_price_series(
        &self,
        start_price: f64,
        count: usize,
        trend: PriceTrend,
    ) -> Vec<Kline> {
        let mut klines = Vec::new();
        let mut price = start_price;

        for i in 0..count {
            // 根据趋势调整价格
            match trend {
                PriceTrend::Up => price += 0.5,
                PriceTrend::Down => price -= 0.5,
                PriceTrend::Sideways => {
                    if i % 2 == 0 {
                        price += 0.3
                    } else {
                        price -= 0.3
                    }
                }
                PriceTrend::Volatile => price += (i % 5) as f64 - 2.0,
            }

            // 确保价格为正
            if price < 1.0 {
                price = 1.0;
            }

            let kline = self.from_close(price);
            klines.push(kline);
        }

        klines
    }
}

/// 价格趋势类型
#[derive(Debug, Clone, Copy)]
pub enum PriceTrend {
    /// 上涨趋势
    Up,
    /// 下跌趋势
    Down,
    /// 横盘震荡
    Sideways,
    /// 高波动
    Volatile,
}

/// 便捷函数：创建测试 K线
pub fn create_test_kline(date: u32, close: f64) -> Kline {
    KlineBuilder::new("000001", date).from_close(close)
}

/// 便捷函数：使用配置创建测试 K线
pub fn create_test_kline_with_config(date: u32, close: f64, config: &TestDataConfig) -> Kline {
    KlineBuilder::new("000001", date)
        .with_config(config.clone())
        .from_close(close)
}

/// 便捷函数：创建完整的 OHLCV 测试 K线
pub fn create_test_ohlcv(
    date: u32,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: i64,
) -> Kline {
    KlineBuilder::new("000001", date).from_ohlcv(open, high, low, close, volume)
}

/// 便捷函数：生成价格序列
pub fn generate_price_series(start_price: f64, count: usize, trend: PriceTrend) -> Vec<Kline> {
    KlineBuilder::new("000001", 0).generate_price_series(start_price, count, trend)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kline_builder_default() {
        let kline = create_test_kline(1, 100.0);

        assert_eq!(kline.code, "000001");
        assert_eq!(kline.close, dec!(100));
        assert_eq!(kline.high, dec!(101)); // 100 + 1.0
        assert_eq!(kline.low, dec!(99)); // 100 - 1.0
        assert_eq!(kline.volume, 1_000_000);
    }

    #[test]
    fn test_kline_builder_custom_config() {
        let config = TestDataConfig::tight_spread();
        let kline = create_test_kline_with_config(1, 100.0, &config);

        assert_eq!(kline.high, dec!(100.1)); // 100 + 0.1
        assert_eq!(kline.low, dec!(99.9)); // 100 - 0.1
    }

    #[test]
    fn test_generate_price_series_up() {
        let series = generate_price_series(100.0, 10, PriceTrend::Up);

        assert_eq!(series.len(), 10);
        // 价格应该上涨
        assert!(series[0].close < series[9].close);
    }

    #[test]
    fn test_generate_price_series_down() {
        let series = generate_price_series(100.0, 10, PriceTrend::Down);

        assert_eq!(series.len(), 10);
        // 价格应该下跌
        assert!(series[0].close > series[9].close);
    }
}
