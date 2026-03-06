/// 投资组合管理
///
/// 从短线侠项目迁移 - 持仓管理、资金计算

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 持仓信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// 股票代码
    pub code: String,
    /// 持仓数量（股）
    pub quantity: i64,
    /// 平均成本价
    pub avg_cost: Decimal,
    /// 当前市值
    pub market_value: Decimal,
    /// 浮动盈亏
    pub pnl: Decimal,
    /// 浮动盈亏比例
    pub pnl_percent: Decimal,
    /// 开仓日期
    pub open_date: NaiveDate,
}

impl Position {
    /// 创建新持仓
    pub fn new(code: String, quantity: i64, price: Decimal, date: NaiveDate) -> Self {
        let market_value = Decimal::from(quantity) * price;
        Self {
            code,
            quantity,
            avg_cost: price,
            market_value,
            pnl: Decimal::ZERO,
            pnl_percent: Decimal::ZERO,
            open_date: date,
        }
    }

    /// 更新持仓价格
    pub fn update_price(&mut self, price: Decimal) {
        self.market_value = Decimal::from(self.quantity) * price;
        self.pnl = self.market_value - (Decimal::from(self.quantity) * self.avg_cost);
        if self.avg_cost > Decimal::ZERO {
            self.pnl_percent = (self.pnl / (Decimal::from(self.quantity) * self.avg_cost)) * Decimal::from(100);
        }
    }

    /// 加仓
    pub fn add(&mut self, quantity: i64, price: Decimal) {
        let total_cost = (Decimal::from(self.quantity) * self.avg_cost) + (Decimal::from(quantity) * price);
        self.quantity += quantity;
        self.avg_cost = total_cost / Decimal::from(self.quantity);
        self.update_price(price);
    }

    /// 减仓
    pub fn reduce(&mut self, quantity: i64, price: Decimal) -> Decimal {
        let realized_pnl = (price - self.avg_cost) * Decimal::from(quantity);
        self.quantity -= quantity;
        if self.quantity > 0 {
            self.update_price(price);
        }
        realized_pnl
    }
}

/// 订单类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Buy,
    Sell,
}

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Rejected,
}

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// 订单ID
    pub id: String,
    /// 股票代码
    pub code: String,
    /// 订单类型
    pub order_type: OrderType,
    /// 数量
    pub quantity: i64,
    /// 价格
    pub price: Decimal,
    /// 订单时间
    pub timestamp: NaiveDateTime,
    /// 状态
    pub status: OrderStatus,
    /// 成交价格
    pub filled_price: Option<Decimal>,
    /// 成交数量
    pub filled_quantity: i64,
    /// 手续费
    pub commission: Decimal,
}

impl Order {
    /// 创建新订单
    pub fn new(
        id: String,
        code: String,
        order_type: OrderType,
        quantity: i64,
        price: Decimal,
        timestamp: NaiveDateTime,
    ) -> Self {
        Self {
            id,
            code,
            order_type,
            quantity,
            price,
            timestamp,
            status: OrderStatus::Pending,
            filled_price: None,
            filled_quantity: 0,
            commission: Decimal::ZERO,
        }
    }

    /// 计算手续费
    pub fn calculate_commission(&mut self, rate: Decimal) {
        let amount = Decimal::from(self.quantity) * self.price;
        self.commission = amount * rate;
    }

    /// 成交
    pub fn fill(&mut self, price: Decimal, quantity: i64) {
        self.status = OrderStatus::Filled;
        self.filled_price = Some(price);
        self.filled_quantity = quantity;
    }
}

/// 投资组合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    /// 初始资金
    pub initial_capital: Decimal,
    /// 可用资金
    pub cash: Decimal,
    /// 持仓
    pub positions: HashMap<String, Position>,
    /// 总资产
    pub total_value: Decimal,
    /// 总盈亏
    pub total_pnl: Decimal,
    /// 手续费率
    pub commission_rate: Decimal,
}

impl Portfolio {
    /// 创建新投资组合
    pub fn new(initial_capital: Decimal, commission_rate: Decimal) -> Self {
        Self {
            initial_capital,
            cash: initial_capital,
            positions: HashMap::new(),
            total_value: initial_capital,
            total_pnl: Decimal::ZERO,
            commission_rate,
        }
    }

    /// 获取持仓
    pub fn get_position(&self, code: &str) -> Option<&Position> {
        self.positions.get(code)
    }

    /// 更新持仓价格
    pub fn update_position_price(&mut self, code: &str, price: Decimal) {
        if let Some(position) = self.positions.get_mut(code) {
            position.update_price(price);
        }
        self.recalculate_total_value();
    }

    /// 买入股票
    pub fn buy(&mut self, code: String, quantity: i64, price: Decimal, date: NaiveDate) -> Result<(String, Decimal), String> {
        let amount = Decimal::from(quantity) * price;
        let commission = amount * self.commission_rate;
        let total_cost = amount + commission;

        if total_cost > self.cash {
            return Err(format!("资金不足: 需要 {}, 可用 {}", total_cost, self.cash));
        }

        self.cash -= total_cost;

        if let Some(position) = self.positions.get_mut(&code) {
            position.add(quantity, price);
        } else {
            let position = Position::new(code.clone(), quantity, price, date);
            self.positions.insert(code, position);
        }

        self.recalculate_total_value();

        let order_id = format!("BUY_{}", uuid::Uuid::new_v4());
        Ok((order_id, commission))
    }

    /// 卖出股票
    pub fn sell(&mut self, code: &str, quantity: i64, price: Decimal) -> Result<(String, Decimal, Decimal), String> {
        if let Some(position) = self.positions.get_mut(code) {
            if position.quantity < quantity {
                return Err(format!("持仓不足: 持有 {}, 想卖 {}", position.quantity, quantity));
            }

            let amount = Decimal::from(quantity) * price;
            let commission = amount * self.commission_rate;
            let realized_pnl = position.reduce(quantity, price) - commission;

            self.cash += amount - commission;

            if position.quantity == 0 {
                self.positions.remove(code);
            }

            self.recalculate_total_value();

            let order_id = format!("SELL_{}", uuid::Uuid::new_v4());
            Ok((order_id, commission, realized_pnl))
        } else {
            Err(format!("无持仓: {}", code))
        }
    }

    /// 重新计算总资产
    fn recalculate_total_value(&mut self) {
        let positions_value: Decimal = self.positions.values()
            .map(|p| p.market_value)
            .sum();

        self.total_value = self.cash + positions_value;
        self.total_pnl = self.total_value - self.initial_capital;
    }

    /// 获取持仓数量
    pub fn position_count(&self) -> usize {
        self.positions.len()
    }

    /// 获取持仓市值
    pub fn positions_value(&self) -> Decimal {
        self.positions.values().map(|p| p.market_value).sum()
    }

    /// 获取收益率
    pub fn return_percent(&self) -> Decimal {
        if self.initial_capital > Decimal::ZERO {
            (self.total_pnl / self.initial_capital) * Decimal::from(100)
        } else {
            Decimal::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let pos = Position::new(
            "000001".to_string(),
            1000,
            dec!(10.5),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );
        assert_eq!(pos.quantity, 1000);
        assert_eq!(pos.avg_cost, dec!(10.5));
        assert_eq!(pos.market_value, dec!(10500));
    }

    #[test]
    fn test_position_update_price() {
        let mut pos = Position::new(
            "000001".to_string(),
            1000,
            dec!(10.0),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );
        pos.update_price(dec!(11.0));
        assert_eq!(pos.market_value, dec!(11000));
        assert_eq!(pos.pnl, dec!(1000));
        assert_eq!(pos.pnl_percent, dec!(10));
    }

    #[test]
    fn test_portfolio_buy() {
        let mut portfolio = Portfolio::new(dec!(100000), dec!(0.0003));

        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let result = portfolio.buy("000001".to_string(), 1000, dec!(10.0), date);

        assert!(result.is_ok());
        assert_eq!(portfolio.position_count(), 1);
        assert_eq!(portfolio.cash, dec!(100000) - dec!(10000) - dec!(3)); // 10000 + 3 commission
    }

    #[test]
    fn test_portfolio_sell() {
        let mut portfolio = Portfolio::new(dec!(100000), dec!(0.0003));

        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        portfolio.buy("000001".to_string(), 1000, dec!(10.0), date).unwrap();

        let result = portfolio.sell("000001", 500, dec!(11.0));
        assert!(result.is_ok());

        let pos = portfolio.get_position("000001").unwrap();
        assert_eq!(pos.quantity, 500);
    }
}
