//! VWAP (Volume-Weighted Average Price) Algorithm
//!
//! 成交量加权平均价格算法实现

use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration, Timelike};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rand::Rng;

use super::{AlgoType, AlgoContext, AlgoParams, AlgoState, AlgoStatus, ChildOrder, ChildOrderStatus};
use super::executor::{AlgorithmExecutor, AlgoError, SlicePlan, Slice};
use crate::execution::adapter::ExecutionAdapter;
use crate::core::Result;

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
            dec!(30), dec!(25), dec!(22), dec!(20), dec!(18),
            dec!(16), dec!(15), dec!(14), dec!(13), dec!(12),
            dec!(11), dec!(10), dec!(10), dec!(10), dec!(10),
            dec!(10), dec!(10), dec!(10), dec!(10), dec!(10),
            dec!(10), dec!(10), dec!(10), dec!(10), dec!(10),
            dec!(10), dec!(10), dec!(10), dec!(10), dec!(10),
            // 10:00 - 10:59 (平稳期)
            dec!(8), dec!(8), dec!(8), dec!(8), dec!(8),
            dec!(8), dec!(8), dec!(8), dec!(8), dec!(8),
            dec!(8), dec!(8), dec!(8), dec!(8), dec!(8),
            dec!(8), dec!(8), dec!(8), dec!(8), dec!(8),
            dec!(8), dec!(8), dec!(8), dec!(8), dec!(8),
            dec!(8), dec!(8), dec!(8), dec!(8), dec!(8),
            // 11:00 - 11:29 (上午后期)
            dec!(7), dec!(7), dec!(7), dec!(7), dec!(7),
            dec!(7), dec!(7), dec!(7), dec!(7), dec!(7),
            dec!(7), dec!(7), dec!(7), dec!(7), dec!(7),
            dec!(7), dec!(7), dec!(7), dec!(7), dec!(7),
            dec!(7), dec!(7), dec!(7), dec!(7), dec!(7),
            dec!(7), dec!(7), dec!(7), dec!(7), dec!(7),
            // 13:00 - 13:29 (下午开盘)
            dec!(12), dec!(11), dec!(10), dec!(10), dec!(10),
            dec!(10), dec!(10), dec!(10), dec!(10), dec!(10),
            dec!(10), dec!(10), dec!(10), dec!(10), dec!(10),
            dec!(10), dec!(10), dec!(10), dec!(10), dec!(10),
            dec!(10), dec!(10), dec!(10), dec!(10), dec!(10),
            dec!(10), dec!(10), dec!(10), dec!(10), dec!(10),
            // 14:00 - 14:59 (下午中段)
            dec!(9), dec!(9), dec!(9), dec!(9), dec!(9),
            dec!(9), dec!(9), dec!(9), dec!(9), dec!(9),
            dec!(9), dec!(9), dec!(9), dec!(9), dec!(9),
            dec!(9), dec!(9), dec!(9), dec!(9), dec!(9),
            dec!(9), dec!(9), dec!(9), dec!(9), dec!(9),
            dec!(9), dec!(9), dec!(9), dec!(9), dec!(9),
            // 15:00 - 15:09 (收盘，成交量放大)
            dec!(15), dec!(18), dec!(20), dec!(22), dec!(25),
            dec!(28), dec!(30), dec!(32), dec!(35), dec!(40),
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
        let slice_count = params.slice_count.unwrap_or_else(|| {
            (total_seconds / interval).max(1) as u32
        });

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
                adjusted.max(params.min_slice_quantity).min(params.max_slice_quantity)
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

impl Default for VwapExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AlgorithmExecutor for VwapExecutor {
    fn algo_type(&self) -> AlgoType {
        AlgoType::VWAP
    }

    async fn initialize(&mut self, params: AlgoParams) -> Result<String> {
        // 验证参数
        params.validate().map_err(|e| crate::core::QuantixError::Algo(AlgoError::InvalidParams(e).to_string()))?;

        // 生成算法ID
        let algo_id = format!("VWAP-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));

        // 创建上下文
        let context = AlgoContext::new(params.clone(), algo_id.clone());

        // 生成切片计划
        let slices = self.generate_slices(&params);
        {
            let mut plans = self.slice_plans.write().await;
            plans.insert(algo_id.clone(), slices);
        }

        // 存储上下文
        {
            let mut contexts = self.contexts.write().await;
            contexts.insert(algo_id.clone(), context);
        }

        tracing::info!(
            algo_id = %algo_id,
            symbol = %params.symbol,
            quantity = params.total_quantity,
            "VWAP algorithm initialized"
        );

        Ok(algo_id)
    }

    async fn start(&mut self, algo_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts.get_mut(algo_id)
            .ok_or_else(|| AlgoError::NotFound(algo_id.to_string()))?;

        context.state.status = AlgoStatus::Running;
        context.state.started_at = Some(Utc::now());

        // 设置第一次下单时间
        let plans = self.slice_plans.read().await;
        if let Some(slices) = plans.get(algo_id) {
            if !slices.is_empty() {
                context.next_order_time = Some(slices[0].scheduled_time);
            }
        }

        tracing::info!(algo_id = %algo_id, "VWAP algorithm started");
        Ok(())
    }

    async fn pause(&mut self, algo_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts.get_mut(algo_id)
            .ok_or_else(|| AlgoError::NotFound(algo_id.to_string()))?;
        context.state.status = AlgoStatus::Paused;

        tracing::info!(algo_id = %algo_id, "VWAP algorithm paused");
        Ok(())
    }

    async fn resume(&mut self, algo_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts.get_mut(algo_id)
            .ok_or_else(|| AlgoError::NotFound(algo_id.to_string()))?;
        context.state.status = AlgoStatus::Running;

        tracing::info!(algo_id = %algo_id, "VWAP algorithm resumed");
        Ok(())
    }

    async fn cancel(&mut self, algo_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts.get_mut(algo_id)
            .ok_or_else(|| AlgoError::NotFound(algo_id.to_string()))?;
        context.state.status = AlgoStatus::Cancelled;
        context.state.completed_at = Some(Utc::now());

        tracing::info!(algo_id = %algo_id, "VWAP algorithm cancelled");
        Ok(())
    }

    async fn get_state(&self, algo_id: &str) -> Result<AlgoState> {
        let contexts = self.contexts.read().await;
        let context = contexts.get(algo_id)
            .ok_or_else(|| AlgoError::NotFound(algo_id.to_string()))?;
        Ok(context.state.clone())
    }

    async fn step(&mut self, algo_id: &str, _adapter: &dyn ExecutionAdapter) -> Result<Option<ChildOrder>> {
        let mut contexts = self.contexts.write().await;
        let context = contexts.get_mut(algo_id)
            .ok_or_else(|| AlgoError::NotFound(algo_id.to_string()))?;

        if !context.should_order_now() {
            return Ok(None);
        }

        // 获取切片计划
        let plans = self.slice_plans.read().await;
        let slices = plans.get(algo_id).cloned();
        drop(plans);

        if let Some(slices) = slices {
            if context.current_slice < slices.len() as u32 {
                let slice = &slices[context.current_slice as usize];

                // 创建子订单
                let child_order = ChildOrder {
                    order_id: format!("{}-{}", algo_id, context.current_slice),
                    algo_id: algo_id.to_string(),
                    scheduled_time: slice.scheduled_time,
                    scheduled_quantity: slice.quantity,
                    scheduled_price: slice.price,
                    order_quantity: slice.quantity,
                    order_price: slice.price,
                    filled_quantity: 0,
                    avg_fill_price: Decimal::ZERO,
                    status: ChildOrderStatus::Pending,
                    created_at: Utc::now(),
                };

                // 更新状态
                context.state.record_order();
                context.current_slice += 1;

                // 设置下一次下单时间
                if context.current_slice < slices.len() as u32 {
                    context.next_order_time = Some(slices[context.current_slice as usize].scheduled_time);
                } else {
                    context.next_order_time = None;
                }

                // 检查是否完成
                if context.is_complete() {
                    context.state.status = AlgoStatus::Completed;
                    context.state.completed_at = Some(Utc::now());
                }

                // 记录子订单
                {
                    let mut orders = self.child_orders.write().await;
                    orders.entry(algo_id.to_string())
                        .or_insert_with(Vec::new)
                        .push(child_order.clone());
                }

                tracing::debug!(
                    algo_id = %algo_id,
                    slice = context.current_slice,
                    quantity = slice.quantity,
                    volume_weight = ?slice.volume_weight,
                    "VWAP slice scheduled"
                );

                return Ok(Some(child_order));
            }
        }

        Ok(None)
    }

    fn get_slice_plan(&self, params: &AlgoParams) -> Result<SlicePlan> {
        let slices = self.generate_slices(params);
        let total_quantity: i64 = slices.iter().map(|s| s.quantity).sum();

        Ok(SlicePlan {
            slices,
            total_quantity,
            start_time: params.start_time,
            end_time: params.end_time,
        })
    }

    fn get_active_algos(&self) -> Vec<String> {
        // 同步获取活跃算法
        let rt = tokio::runtime::Handle::current();
        let contexts = rt.block_on(self.contexts.read());
        contexts.iter()
            .filter(|(_, ctx)| !ctx.state.is_finished())
            .map(|(id, _)| id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vwap_initialize() {
        let mut executor = VwapExecutor::new();
        let params = AlgoParams::vwap("600519.SH".to_string(), "buy".to_string(), 1000, 30);

        let result = executor.initialize(params).await;
        assert!(result.is_ok());

        let algo_id = result.unwrap();
        assert!(algo_id.starts_with("VWAP-"));
    }

    #[test]
    fn test_vwap_slice_plan() {
        let executor = VwapExecutor::new();
        let start = Utc::now();
        let end = start + Duration::hours(2);

        let params = AlgoParams {
            algo_type: AlgoType::VWAP,
            symbol: "600519.SH".to_string(),
            side: "buy".to_string(),
            total_quantity: 10000,
            start_time: start,
            end_time: end,
            interval_seconds: Some(300), // 5分钟
            randomize_timing: false,
            randomize_quantity: false,
            ..Default::default()
        };

        let plan = executor.get_slice_plan(&params).unwrap();

        // 验证总数量匹配
        let total: i64 = plan.slices.iter().map(|s| s.quantity).sum();
        assert_eq!(total, 10000);

        // 验证有权重
        for slice in &plan.slices {
            assert!(slice.volume_weight.is_some());
        }
    }

    #[test]
    fn test_volume_weight() {
        let executor = VwapExecutor::new();

        // 开盘时间 (9:35)
        let time1: DateTime<Utc> = "2026-03-27T01:35:00Z".parse().unwrap(); // UTC 09:35 Beijing
        let weight1 = executor.get_volume_weight(time1);

        // 中午时间 (10:30 Beijing = 02:30 UTC)
        let time2: DateTime<Utc> = "2026-03-27T02:30:00Z".parse().unwrap();
        let weight2 = executor.get_volume_weight(time2);

        // 验证权重为正
        assert!(weight1 > Decimal::ZERO);
        assert!(weight2 > Decimal::ZERO);
    }
}
