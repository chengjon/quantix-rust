/// 回测引擎
///
/// 从短线侠项目迁移 - 基于历史数据的策略回测
use crate::analysis::performance::{
    PerformanceCalculator, PerformanceReport, TradeRecord, TradeSide,
};
use crate::analysis::portfolio::Portfolio;
use crate::core::Result;
use crate::core::signal::Signal;
use crate::data::models::Kline;
use crate::strategy::trait_def::Strategy;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// 回测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// 初始资金
    pub initial_capital: Decimal,
    /// 手续费率
    pub commission_rate: Decimal,
    /// 滑点（bps）
    pub slippage_bps: u32,
    /// 最大持仓数量
    pub max_positions: usize,
    /// 单股最大持仓比例
    pub max_position_ratio: Decimal,
    /// 无风险利率（年化）
    pub risk_free_rate: Decimal,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: dec!(1000000),
            commission_rate: dec!(0.0003), // 万分之三
            slippage_bps: 2,               // 2bp 滑点
            max_positions: 5,
            max_position_ratio: dec!(0.2), // 单股最多20%
            risk_free_rate: dec!(0.03),    // 3%
        }
    }
}

/// 回测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    /// 性能报告
    pub report: PerformanceReport,
    /// 最终账户价值
    pub final_equity: Decimal,
    /// 交易记录
    pub trades: Vec<TradeRecord>,
    /// 配置
    pub config: BacktestConfig,
}

/// 回测引擎
pub struct BacktestEngine {
    /// 配置
    config: BacktestConfig,
    /// 投资组合
    portfolio: Portfolio,
    /// 性能计算器
    calculator: PerformanceCalculator,
    /// 待成交订单
    pending_orders: Vec<BacktestOrder>,
    /// 持仓开仓信息（用于计算盈亏）
    position_info: HashMap<String, PositionInfo>,
    /// 当前日期
    current_date: NaiveDate,
}

/// 持仓信息
#[derive(Debug, Clone)]
struct PositionInfo {
    _code: String,
    open_date: NaiveDate,
    open_price: Decimal,
    quantity: i64,
}

/// 回测订单
#[derive(Debug, Clone)]
struct BacktestOrder {
    code: String,
    quantity: i64,
    order_type: OrderType,
}

/// 订单类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderType {
    Buy,
    Sell,
}

impl BacktestEngine {
    /// 创建新的回测引擎
    pub fn new(config: BacktestConfig) -> Self {
        let portfolio = Portfolio::new(config.initial_capital, config.commission_rate);
        let calculator = PerformanceCalculator::new(config.initial_capital, config.risk_free_rate);

        Self {
            config,
            portfolio,
            calculator,
            pending_orders: Vec::new(),
            position_info: HashMap::new(),
            current_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(BacktestConfig::default())
    }

    /// 运行回测
    pub async fn run<S>(
        &mut self,
        strategy: &mut S,
        data: &HashMap<String, Vec<Kline>>,
    ) -> Result<BacktestResult>
    where
        S: Strategy + Send + Sync,
    {
        info!("开始回测...");

        // 初始化策略
        strategy
            .init()
            .await
            .map_err(|e| crate::core::QuantixError::Other(e.to_string()))?;

        // 获取所有交易日
        let all_dates = self.extract_all_dates(data);

        // 按日期遍历
        for date in all_dates {
            self.current_date = date;

            // 更新持仓价格
            self.update_positions_price(data, date);

            // 执行策略
            self.execute_strategy(strategy, data, date).await?;

            // 成交订单
            self.execute_orders(data, date)?;

            // 更新权益
            self.update_equity(date);

            debug!("日期: {}, 账户价值: {}", date, self.portfolio.total_value);
        }

        // 结束策略
        strategy
            .finish()
            .await
            .map_err(|e| crate::core::QuantixError::Other(e.to_string()))?;

        // 生成报告
        let report = self.calculator.calculate();

        info!("回测完成: 总收益率={}%", report.total_return * dec!(100));

        Ok(BacktestResult {
            report,
            final_equity: self.portfolio.total_value,
            trades: self.calculator.trades().to_vec(),
            config: self.config.clone(),
        })
    }

    /// 提取所有交易日
    fn extract_all_dates(&self, data: &HashMap<String, Vec<Kline>>) -> Vec<NaiveDate> {
        let mut dates: Vec<NaiveDate> = data
            .values()
            .flat_map(|klines| klines.iter().map(|k| k.date))
            .collect();

        dates.sort();
        dates.dedup();
        dates
    }

    /// 更新持仓价格
    fn update_positions_price(&mut self, data: &HashMap<String, Vec<Kline>>, date: NaiveDate) {
        for (code, _position) in self.portfolio.positions.clone() {
            if let Some(klines) = data.get(&code)
                && let Some(kline) = klines.iter().find(|k| k.date == date)
            {
                self.portfolio.update_position_price(&code, kline.close);
            }
        }
    }

    /// 执行策略
    async fn execute_strategy<S>(
        &mut self,
        strategy: &mut S,
        data: &HashMap<String, Vec<Kline>>,
        date: NaiveDate,
    ) -> Result<()>
    where
        S: Strategy + Send + Sync,
    {
        for (code, klines) in data {
            if let Some(kline) = klines.iter().find(|k| k.date == date) {
                match strategy.on_bar(kline).await {
                    Ok(Signal::Buy) => {
                        self.pending_orders.push(BacktestOrder {
                            code: code.clone(),
                            quantity: self.calculate_order_quantity(code, kline.close),
                            order_type: OrderType::Buy,
                        });
                    }
                    Ok(Signal::Sell) => {
                        if let Some(position) = self.portfolio.get_position(code) {
                            self.pending_orders.push(BacktestOrder {
                                code: code.clone(),
                                quantity: position.quantity,
                                order_type: OrderType::Sell,
                            });
                        }
                    }
                    Ok(Signal::Hold) => {}
                    Err(e) => {
                        warn!("策略执行错误: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// 计算订单数量
    fn calculate_order_quantity(&self, _code: &str, price: Decimal) -> i64 {
        // 计算目标金额（平均分配）
        let target_amount = self.portfolio.cash / Decimal::from(self.config.max_positions);

        // 计算数量（向下取整到100的倍数）
        let quantity = ((target_amount / price) / dec!(100)).floor() * dec!(100);

        quantity.to_i64().unwrap_or(0)
    }

    /// 成交订单
    fn execute_orders(
        &mut self,
        data: &HashMap<String, Vec<Kline>>,
        date: NaiveDate,
    ) -> Result<()> {
        let mut executed = Vec::new();

        for (i, order) in self.pending_orders.iter().enumerate() {
            let kline = data
                .get(&order.code)
                .and_then(|klines| klines.iter().find(|k| k.date == date));

            if let Some(kline) = kline {
                let price = self.apply_slippage(kline.close, order.order_type);

                match order.order_type {
                    OrderType::Buy => {
                        if let Ok((_order_id, commission)) =
                            self.portfolio
                                .buy(order.code.clone(), order.quantity, price, date)
                        {
                            // 记录开仓信息
                            self.position_info.insert(
                                order.code.clone(),
                                PositionInfo {
                                    _code: order.code.clone(),
                                    open_date: date,
                                    open_price: price,
                                    quantity: order.quantity,
                                },
                            );

                            debug!("买入: {} @ {}, 手续费: {}", order.code, price, commission);
                            executed.push(i);
                        }
                    }
                    OrderType::Sell => {
                        if let Ok((_order_id, commission, pnl)) =
                            self.portfolio.sell(&order.code, order.quantity, price)
                        {
                            // 记录交易
                            if let Some(info) = self.position_info.remove(&order.code) {
                                let trade = TradeRecord {
                                    code: order.code.clone(),
                                    side: TradeSide::Long,
                                    open_date: info.open_date,
                                    close_date: date,
                                    open_price: info.open_price,
                                    close_price: price,
                                    quantity: info.quantity,
                                    pnl,
                                    pnl_percent: if info.open_price > Decimal::ZERO {
                                        (price - info.open_price) / info.open_price * dec!(100)
                                    } else {
                                        Decimal::ZERO
                                    },
                                    commission,
                                };
                                self.calculator.add_trade(trade);
                            }

                            debug!(
                                "卖出: {} @ {}, 盈亏: {}, 手续费: {}",
                                order.code, price, pnl, commission
                            );
                            executed.push(i);
                        }
                    }
                }
            }
        }

        // 移除已成交的订单（倒序移除）
        for i in executed.into_iter().rev() {
            self.pending_orders.remove(i);
        }

        Ok(())
    }

    /// 应用滑点
    fn apply_slippage(&self, price: Decimal, order_type: OrderType) -> Decimal {
        let slippage = Decimal::from(self.config.slippage_bps) / dec!(10000);

        match order_type {
            OrderType::Buy => price * (Decimal::ONE + slippage),
            OrderType::Sell => price * (Decimal::ONE - slippage),
        }
    }

    /// 更新权益
    fn update_equity(&mut self, date: NaiveDate) {
        self.calculator
            .add_equity_point(date, self.portfolio.total_value);
    }

    /// 获取当前投资组合快照
    pub fn portfolio_snapshot(&self) -> &Portfolio {
        &self.portfolio
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_backtest_config_default() {
        let config = BacktestConfig::default();
        assert_eq!(config.initial_capital, dec!(1000000));
        assert_eq!(config.commission_rate, dec!(0.0003));
    }

    #[test]
    fn test_apply_slippage() {
        let engine = BacktestEngine::with_default_config();
        let price = dec!(100);

        let buy_price = engine.apply_slippage(price, OrderType::Buy);
        let sell_price = engine.apply_slippage(price, OrderType::Sell);

        assert!(buy_price > price);
        assert!(sell_price < price);
    }
}
