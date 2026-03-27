//! Algorithm Context
//!
//! 算法执行的上下文和参数定义

use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::super::adapter::AdapterOrderRequest;
use super::super::models::OrderSide;
use super::{AlgoType, AlgoState};

/// 算法参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgoParams {
    /// 算法类型
    pub algo_type: AlgoType,
    /// 股票代码
    pub symbol: String,
    /// 买卖方向 (buy/sell)
    pub side: String,
    /// 总数量
    pub total_quantity: i64,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: DateTime<Utc>,
    /// 价格限制 (可选)
    pub price_limit: Option<Decimal>,
    /// 参与率 (用于 POV/VWAP，百分比)
    pub participation_rate: Option<Decimal>,
    /// 是否随机化下单时间
    pub randomize_timing: bool,
    /// 是否随机化下单数量
    pub randomize_quantity: bool,
    /// 单笔最小数量
    pub min_slice_quantity: i64,
    /// 单笔最大数量
    pub max_slice_quantity: i64,
    /// 时间间隔 (秒)
    pub interval_seconds: Option<u64>,
    /// 切片数量
    pub slice_count: Option<u32>,
    /// 自定义参数
    pub extra: HashMap<String, String>,
}

impl Default for AlgoParams {
    fn default() -> Self {
        Self {
            algo_type: AlgoType::TWAP,
            symbol: String::new(),
            side: "buy".to_string(),
            total_quantity: 0,
            start_time: Utc::now(),
            end_time: Utc::now() + Duration::hours(1),
            price_limit: None,
            participation_rate: Some(Decimal::from(10)), // 10%
            randomize_timing: true,
            randomize_quantity: true,
            min_slice_quantity: 100,
            max_slice_quantity: 10000,
            interval_seconds: None,
            slice_count: None,
            extra: HashMap::new(),
        }
    }
}

impl AlgoParams {
    /// 创建 TWAP 参数
    pub fn twap(symbol: String, side: String, quantity: i64, duration_minutes: u32) -> Self {
        let now = Utc::now();
        Self {
            algo_type: AlgoType::TWAP,
            symbol,
            side,
            total_quantity: quantity,
            start_time: now,
            end_time: now + Duration::minutes(i64::from(duration_minutes)),
            ..Default::default()
        }
    }

    /// 创建 VWAP 参数
    pub fn vwap(symbol: String, side: String, quantity: i64, duration_minutes: u32) -> Self {
        let now = Utc::now();
        Self {
            algo_type: AlgoType::VWAP,
            symbol,
            side,
            total_quantity: quantity,
            start_time: now,
            end_time: now + Duration::minutes(i64::from(duration_minutes)),
            participation_rate: Some(Decimal::from(10)),
            ..Default::default()
        }
    }

    /// 设置价格限制
    pub fn with_price_limit(mut self, price: Decimal) -> Self {
        self.price_limit = Some(price);
        self
    }

    /// 设置参与率
    pub fn with_participation_rate(mut self, rate: Decimal) -> Self {
        self.participation_rate = Some(rate);
        self
    }

    /// 设置时间范围
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = start;
        self.end_time = end;
        self
    }

    /// 设置切片数量
    pub fn with_slice_count(mut self, count: u32) -> Self {
        self.slice_count = Some(count);
        self
    }

    /// 设置时间间隔
    pub fn with_interval(mut self, seconds: u64) -> Self {
        self.interval_seconds = Some(seconds);
        self
    }

    /// 禁用随机化
    pub fn no_randomize(mut self) -> Self {
        self.randomize_timing = false;
        self.randomize_quantity = false;
        self
    }

    /// 验证参数
    pub fn validate(&self) -> Result<(), String> {
        if self.symbol.is_empty() {
            return Err("Symbol is required".to_string());
        }
        if self.total_quantity <= 0 {
            return Err("Total quantity must be positive".to_string());
        }
        if self.end_time <= self.start_time {
            return Err("End time must be after start time".to_string());
        }
        if self.side != "buy" && self.side != "sell" {
            return Err("Side must be 'buy' or 'sell'".to_string());
        }
        if let Some(rate) = self.participation_rate {
            if rate <= Decimal::ZERO || rate > Decimal::from(100) {
                return Err("Participation rate must be between 0 and 100".to_string());
            }
        }
        Ok(())
    }
}

/// 算法执行上下文
#[derive(Debug, Clone)]
pub struct AlgoContext {
    /// 算法参数
    pub params: AlgoParams,
    /// 当前状态
    pub state: AlgoState,
    /// 当前市场价
    pub current_price: Decimal,
    /// 当前成交量 (用于 VWAP)
    pub current_volume: i64,
    /// 预期成交量 (用于 VWAP)
    pub expected_volume: i64,
    /// 已执行的切片索引
    pub current_slice: u32,
    /// 总切片数
    pub total_slices: u32,
    /// 下一次下单时间
    pub next_order_time: Option<DateTime<Utc>>,
    /// 上一次成交价
    pub last_fill_price: Decimal,
}

impl AlgoContext {
    /// 创建新的上下文
    pub fn new(params: AlgoParams, algo_id: String) -> Self {
        let symbol = params.symbol.clone();
        let side = params.side.clone();
        let total_quantity = params.total_quantity;

        // 计算切片数量
        let total_slices = params.slice_count.unwrap_or_else(|| {
            let duration_seconds = (params.end_time - params.start_time).num_seconds() as u64;
            let interval = params.interval_seconds.unwrap_or(60);
            (duration_seconds / interval).max(1) as u32
        });

        let state = AlgoState::new(algo_id, symbol, side, total_quantity);

        Self {
            params,
            state,
            current_price: Decimal::ZERO,
            current_volume: 0,
            expected_volume: 0,
            current_slice: 0,
            total_slices,
            next_order_time: None,
            last_fill_price: Decimal::ZERO,
        }
    }

    /// 获取当前切片数量
    pub fn get_slice_quantity(&self) -> i64 {
        let remaining = self.state.remaining_quantity();
        let remaining_slices = self.total_slices - self.current_slice;

        if remaining_slices == 0 {
            return remaining;
        }

        let base_quantity = remaining / remaining_slices as i64;
        let remainder = remaining % remaining_slices as i64;

        // 当前切片可能需要多承担余数
        if self.current_slice < remainder as u32 {
            base_quantity + 1
        } else {
            base_quantity
        }
    }

    /// 检查是否应该下单
    pub fn should_order_now(&self) -> bool {
        if self.state.status != super::AlgoStatus::Running {
            return false;
        }

        if self.state.remaining_quantity() <= 0 {
            return false;
        }

        if let Some(next_time) = self.next_order_time {
            return Utc::now() >= next_time;
        }

        true
    }

    /// 检查是否完成
    pub fn is_complete(&self) -> bool {
        self.state.remaining_quantity() <= 0 || self.current_slice >= self.total_slices
    }

    /// 检查是否超时
    pub fn is_timeout(&self) -> bool {
        Utc::now() > self.params.end_time
    }

    /// 生成订单请求
    pub fn create_order_request(&self, quantity: i64, price: Option<Decimal>) -> AdapterOrderRequest {
        let side = match self.params.side.as_str() {
            "buy" => OrderSide::Buy,
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        AdapterOrderRequest {
            client_order_id: format!("{}-{}-{}", self.state.algo_id, self.current_slice, Utc::now().timestamp()),
            symbol: self.params.symbol.clone(),
            side,
            quantity,
            price: price.unwrap_or(self.current_price),
        }
    }

    /// 计算进度百分比
    pub fn progress_percent(&self) -> Decimal {
        self.state.completion_percent()
    }
}
