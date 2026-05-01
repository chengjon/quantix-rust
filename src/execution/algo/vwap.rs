//! VWAP (Volume-Weighted Average Price) Algorithm
//!
//! 成交量加权平均价格算法实现

use async_trait::async_trait;
use chrono::{DateTime, Duration, Timelike, Utc};
use rand::Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::executor::{AlgoError, AlgorithmExecutor, Slice, SlicePlan};
use super::{
    AlgoContext, AlgoParams, AlgoState, AlgoStatus, AlgoType, ChildOrder, ChildOrderStatus,
};
use crate::core::Result;
mod runtime;

#[cfg(test)]
mod tests;

use crate::execution::adapter::ExecutionAdapter;

/// VWAP 算法执行器
pub struct VwapExecutor {
    /// 算法上下文
    contexts: Arc<RwLock<HashMap<String, AlgoContext>>>,
    /// 切片计划缓存
    slice_plans: Arc<RwLock<HashMap<String, Vec<Slice>>>>,
    /// 子订单记录
    child_orders: Arc<RwLock<HashMap<String, Vec<ChildOrder>>>>,
    /// 成交量分布曲线
    volume_profile: Vec<Decimal>,
}

impl VwapExecutor {
    /// 创建新的 VWAP 执行器
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            slice_plans: Arc::new(RwLock::new(HashMap::new())),
            child_orders: Arc::new(RwLock::new(HashMap::new())),
            volume_profile: Self::default_volume_profile(),
        }
    }

    /// 默认成交量分布曲线 (分钟级别)
    /// 表示每分钟成交量占全天的相对权重
    fn default_volume_profile() -> Vec<Decimal> {
        vec![
            // 09:30 - 09:59 (开盘30分钟，成交量较大)
            dec!(30),
            dec!(25),
            dec!(22),
            dec!(20),
            dec!(18),
            dec!(16),
            dec!(15),
            dec!(14),
            dec!(13),
            dec!(12),
            dec!(11),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            // 10:00 - 10:59 (平稳期)
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            dec!(8),
            // 11:00 - 11:29 (上午后期)
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            dec!(7),
            // 13:00 - 13:29 (下午开盘)
            dec!(12),
            dec!(11),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            dec!(10),
            // 14:00 - 14:59 (下午中段)
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            dec!(9),
            // 15:00 - 15:09 (收盘，成交量放大)
            dec!(15),
            dec!(18),
            dec!(20),
            dec!(22),
            dec!(25),
            dec!(28),
            dec!(30),
            dec!(32),
            dec!(35),
            dec!(40),
        ]
    }

    /// 使用自定义成交量分布
    pub fn with_volume_profile(mut self, profile: Vec<Decimal>) -> Self {
        self.volume_profile = profile;
        self
    }

    /// 获取某分钟的成交量权重
    fn get_volume_weight(&self, time: DateTime<Utc>) -> Decimal {
        // A股交易时间: 9:30-11:30, 13:00-15:00
        let hour = time.hour();
        let minute = time.minute();

        // 计算相对于开盘的分钟数
        let minutes_since_open = if hour < 12 {
            // 上午: 9:30 开始
            let base_minutes = (hour as i32 - 9) * 60 + minute as i32 - 30;
            base_minutes.max(0) as usize
        } else {
            // 下午: 13:00 开始
            // 上午有 120 分钟 (9:30-11:30)
            let afternoon_minutes = (hour as i32 - 13) * 60 + minute as i32;
            120 + afternoon_minutes.max(0) as usize
        };

        // 获取权重
        if minutes_since_open < self.volume_profile.len() {
            self.volume_profile[minutes_since_open]
        } else {
            Decimal::ONE
        }
    }

    /// 生成 VWAP 切片计划
    fn generate_slices(&self, params: &AlgoParams) -> Vec<Slice> {
        let total_seconds = (params.end_time - params.start_time).num_seconds() as u64;
        let interval = params.interval_seconds.unwrap_or(300); // 默认5分钟
        let slice_count = params
            .slice_count
            .unwrap_or_else(|| (total_seconds / interval).max(1) as u32);

        // 计算总成交量权重
        let mut total_weight = Decimal::ZERO;
        let mut time_weights = Vec::with_capacity(slice_count as usize);
        let interval_duration = Duration::seconds(interval as i64);

        for i in 0..slice_count {
            let scheduled_time = params.start_time + interval_duration * i as i32;
            let weight = self.get_volume_weight(scheduled_time);
            time_weights.push(weight);
            total_weight += weight;
        }

        // 根据权重分配数量
        let mut slices = Vec::with_capacity(slice_count as usize);
        let mut allocated_quantity: i64 = 0;
        let mut rng = rand::thread_rng();

        for (i, weight) in time_weights.into_iter().enumerate() {
            let scheduled_time = params.start_time + interval_duration * i as i32;

            // 根据权重计算数量
            let weight_ratio = if total_weight > Decimal::ZERO {
                weight / total_weight
            } else {
                Decimal::ONE / Decimal::from(slice_count)
            };

            let base_quantity = (Decimal::from(params.total_quantity) * weight_ratio)
                .to_string()
                .parse::<i64>()
                .unwrap_or(0);

            // 可选随机化
            let quantity = if params.randomize_quantity && base_quantity > 0 {
                let jitter_pct = rng.gen_range(-10..10) as i64;
                let adjusted = base_quantity + (base_quantity * jitter_pct / 100);
                adjusted
                    .max(params.min_slice_quantity)
                    .min(params.max_slice_quantity)
            } else {
                base_quantity
            };

            // 最后一个切片分配剩余数量
            let final_quantity = if i == slice_count as usize - 1 {
                params.total_quantity - allocated_quantity
            } else {
                quantity
            };

            if final_quantity > 0 {
                allocated_quantity += final_quantity;

                slices.push(Slice {
                    index: i as u32,
                    scheduled_time: if params.randomize_timing {
                        let jitter = rng.gen_range(0..(interval / 2) as i64);
                        scheduled_time + Duration::seconds(jitter)
                    } else {
                        scheduled_time
                    },
                    quantity: final_quantity,
                    price: params.price_limit,
                    volume_weight: Some(weight),
                });
            }
        }

        slices
    }
}
