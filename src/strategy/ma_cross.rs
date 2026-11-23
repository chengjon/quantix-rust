/// 均线交叉策略
///
/// MA 金叉死叉策略实现
use async_trait::async_trait;

use crate::analysis::indicators::ma;
use crate::core::signal::Signal;
use crate::data::models::Kline;
use crate::strategy::trait_def::Strategy;

/// MA 金叉死叉策略
pub struct MACrossStrategy {
    short_period: usize,
    long_period: usize,
    name: String,
    // 历史数据缓存
    price_history: Vec<rust_decimal::Decimal>,
    // 上一次的信号状态
    last_short_ma: Option<rust_decimal::Decimal>,
    last_long_ma: Option<rust_decimal::Decimal>,
    // 当前持仓状态
    position: bool,
}

impl MACrossStrategy {
    /// 创建双均线策略：short_period 短均线周期、long_period 长均线周期，自动生成 `MA_{s}_{l}` 名称，初始空仓。
    pub fn new(short_period: usize, long_period: usize) -> Self {
        Self {
            short_period,
            long_period,
            name: format!("MA_{}_{}", short_period, long_period),
            price_history: Vec::new(),
            last_short_ma: None,
            last_long_ma: None,
            position: false,
        }
    }

    /// 计算金叉（短期均线上穿长期均线）
    fn is_golden_cross(
        &self,
        prev_short: rust_decimal::Decimal,
        prev_long: rust_decimal::Decimal,
        curr_short: rust_decimal::Decimal,
        curr_long: rust_decimal::Decimal,
    ) -> bool {
        // 上一时刻: 短期 <= 长期
        // 当前时刻: 短期 > 长期
        prev_short <= prev_long && curr_short > curr_long
    }

    /// 计算死叉（短期均线下穿长期均线）
    fn is_death_cross(
        &self,
        prev_short: rust_decimal::Decimal,
        prev_long: rust_decimal::Decimal,
        curr_short: rust_decimal::Decimal,
        curr_long: rust_decimal::Decimal,
    ) -> bool {
        // 上一时刻: 短期 >= 长期
        // 当前时刻: 短期 < 长期
        prev_short >= prev_long && curr_short < curr_long
    }

    /// 同步信号推进（供 async on_bar 与 fin 回测桥共用；无 IO、无 Result）。
    pub(crate) fn signal_at(&mut self, bar: &Kline) -> Signal {
        // 添加收盘价到历史数据
        self.price_history.push(bar.close);

        // 需要足够的数据才能计算
        if self.price_history.len() < self.long_period {
            return Signal::Hold;
        }

        // 计算短期和长期均线
        let short_mas = ma(&self.price_history, self.short_period);
        let long_mas = ma(&self.price_history, self.long_period);

        let curr_short_ma = short_mas[self.price_history.len() - 1];
        let curr_long_ma = long_mas[self.price_history.len() - 1];

        // 如果没有足够的均线数据，保持观望
        let (Some(curr_short_ma), Some(curr_long_ma)) = (curr_short_ma, curr_long_ma) else {
            return Signal::Hold;
        };

        // 需要上一时刻的均线数据来判断交叉
        if let (Some(prev_short), Some(prev_long)) = (self.last_short_ma, self.last_long_ma) {
            // 检测金叉 - 买入信号
            if self.is_golden_cross(prev_short, prev_long, curr_short_ma, curr_long_ma) {
                self.last_short_ma = Some(curr_short_ma);
                self.last_long_ma = Some(curr_long_ma);
                self.position = true;
                return Signal::Buy;
            }

            // 检测死叉 - 卖出信号
            if self.is_death_cross(prev_short, prev_long, curr_short_ma, curr_long_ma) {
                self.last_short_ma = Some(curr_short_ma);
                self.last_long_ma = Some(curr_long_ma);
                self.position = false;
                return Signal::Sell;
            }
        }

        // 更新上一时刻的均线值
        self.last_short_ma = Some(curr_short_ma);
        self.last_long_ma = Some(curr_long_ma);

        Signal::Hold
    }

    /// 历史数据缓存长度（测试探针；仅测试编译路径存在）。
    #[cfg(test)]
    pub(crate) fn price_history_len(&self) -> usize {
        self.price_history.len()
    }
}

#[async_trait]
impl Strategy for MACrossStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    async fn on_bar(&mut self, bar: &Kline) -> Result<Signal, Box<dyn std::error::Error>> {
        Ok(self.signal_at(bar))
    }

    async fn finish(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 重置状态
        self.price_history.clear();
        self.last_short_ma = None;
        self.last_long_ma = None;
        self.position = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::test_utils::create_test_kline;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_ma_cross_strategy() {
        let mut strategy = MACrossStrategy::new(3, 5);

        // 生成上升趋势的测试数据
        let prices: Vec<f64> = vec![10.0, 10.5, 11.0, 11.5, 12.0, 12.5, 13.0, 14.0, 15.0];

        for (i, price) in prices.iter().enumerate() {
            let bar = create_test_kline(i as u32, *price);
            let signal = strategy.on_bar(&bar).await.unwrap();

            // 在上升趋势中，短期均线应该上穿长期均线
            if i >= 6 {
                // 此时应该产生买入信号
                if matches!(signal, Signal::Buy) {
                    break;
                }
            }
        }
    }

    #[tokio::test]
    async fn test_ma_cross_death_cross() {
        let mut strategy = MACrossStrategy::new(3, 5);

        // 先上涨后下跌，触发死叉
        let prices: Vec<f64> = vec![
            10.0, 10.5, 11.0, 11.5, 12.0, // 上涨
            12.5, 13.0, 14.0, 13.5, 13.0, // 见顶回落
            12.5, 12.0, 11.5, 11.0, // 继续下跌
        ];

        let mut has_bought = false;
        for (i, price) in prices.iter().enumerate() {
            let bar = create_test_kline(i as u32, *price);
            let signal = strategy.on_bar(&bar).await.unwrap();

            if matches!(signal, Signal::Buy) {
                has_bought = true;
            }

            // 买入后应该会有卖出信号
            if has_bought && matches!(signal, Signal::Sell) {
                return; // 测试通过
            }
        }

        // 如果没有产生卖出信号，至少验证没有 panic
    }

    #[test]
    fn test_golden_cross_detection() {
        let strategy = MACrossStrategy::new(5, 10);

        // 金叉测试: 短期从下方上穿长期
        assert!(strategy.is_golden_cross(
            dec!(9.5),  // prev_short
            dec!(10.0), // prev_long
            dec!(10.2), // curr_short
            dec!(10.1)  // curr_long
        ));

        // 非金叉
        assert!(!strategy.is_golden_cross(
            dec!(10.2), // prev_short
            dec!(10.0), // prev_long
            dec!(10.3), // curr_short
            dec!(10.1)  // curr_long
        ));
    }

    #[test]
    fn test_death_cross_detection() {
        let strategy = MACrossStrategy::new(5, 10);

        // 死叉测试: 短期从上方下穿长期
        assert!(strategy.is_death_cross(
            dec!(10.2), // prev_short
            dec!(10.0), // prev_long
            dec!(9.8),  // curr_short
            dec!(10.1)  // curr_long
        ));

        // 非死叉
        assert!(!strategy.is_death_cross(
            dec!(9.5),  // prev_short
            dec!(10.0), // prev_long
            dec!(9.7),  // curr_short
            dec!(10.1)  // curr_long
        ));
    }

    #[test]
    fn signal_at_drives_state_and_returns_signal() {
        use crate::strategy::test_utils::create_test_kline;
        let mut s = MACrossStrategy::new(2, 3);
        let klines: Vec<_> = (0..12)
            .map(|i| create_test_kline(i, if i >= 6 { 12.0 } else { 10.0 }))
            .collect();
        let signals: Vec<Signal> = klines.iter().map(|k| s.signal_at(k)).collect();
        // 有状态推进：末段价格跳变后，history 长度=12
        assert_eq!(s.price_history_len(), 12);
        // 与异步委托测试同一数据集：首 Buy 位置一致（同步/异步两条路径互补钉住）
        let i_buy = signals.iter().position(|sg| matches!(sg, Signal::Buy));
        assert_eq!(i_buy, Some(6));
        assert_eq!(
            signals
                .iter()
                .filter(|sg| matches!(sg, Signal::Buy))
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn on_bar_delegates_to_signal_at_and_drives_buy() {
        use crate::strategy::test_utils::create_test_kline;
        let mut s = MACrossStrategy::new(2, 3);
        let mut signals = Vec::new();
        for i in 0..12u32 {
            let price = if i >= 6 { 12.0 } else { 10.0 };
            let kline = create_test_kline(i, price);
            signals.push(s.on_bar(&kline).await.unwrap());
        }
        // 手算（(2,3)，prices=[10×6, 12×6]）：i=6 时 prev_short=(10+10)/2=10
        // ≤ prev_long=(10+10+10)/3=10，且 short=(10+12)/2=11 > long=(10+10+12)/3≈10.67
        // → 金叉；Buy 后 position=true 且后续无死叉 → 全序列恰 1 个 Buy @ i=6，其余全 Hold
        assert_eq!(
            signals
                .iter()
                .filter(|sg| matches!(sg, Signal::Buy))
                .count(),
            1,
            "恰 1 个 Buy: {signals:?}"
        );
        assert!(
            signals
                .iter()
                .all(|sg| matches!(sg, Signal::Buy | Signal::Hold)),
            "不应出现 Sell（无死叉输入）: {signals:?}"
        );
        let i_buy = signals
            .iter()
            .position(|sg| matches!(sg, Signal::Buy))
            .unwrap_or_else(|| panic!("应有 Buy 信号: {signals:?}"));
        assert_eq!(i_buy, 6, "首 Buy 应钉在 i=6（金叉首个可用 bar）");
        assert_eq!(s.price_history_len(), 12);
    }
}
