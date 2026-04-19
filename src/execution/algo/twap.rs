#![allow(clippy::collapsible_if)]

//! TWAP (Time-Weighted Average Price) Algorithm
//!
//! 时间加权平均价格算法实现

use async_trait::async_trait;
use chrono::{Duration, Utc};
use rand::Rng;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::executor::{AlgoError, AlgorithmExecutor, Slice, SlicePlan};
use super::state::AlgoStatus;
use super::{AlgoContext, AlgoParams, AlgoState, AlgoType, ChildOrder, ChildOrderStatus};
use crate::core::Result;
use crate::execution::adapter::ExecutionAdapter;

/// TWAP 算法执行器
pub struct TwapExecutor {
    /// 算法上下文
    contexts: Arc<RwLock<HashMap<String, AlgoContext>>>,
    /// 切片计划缓存
    slice_plans: Arc<RwLock<HashMap<String, Vec<Slice>>>>,
    /// 子订单记录
    child_orders: Arc<RwLock<HashMap<String, Vec<ChildOrder>>>>,
}

impl TwapExecutor {
    /// 创建新的 TWAP 执行器
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            slice_plans: Arc::new(RwLock::new(HashMap::new())),
            child_orders: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 生成 TWAP 切片计划
    fn generate_slices(&self, params: &AlgoParams) -> Vec<Slice> {
        let total_seconds = (params.end_time - params.start_time).num_seconds() as u64;
        let interval = params.interval_seconds.unwrap_or(60);
        let slice_count = params
            .slice_count
            .unwrap_or_else(|| (total_seconds / interval).max(1) as u32);

        let base_quantity = params.total_quantity / slice_count as i64;
        let remainder = params.total_quantity % slice_count as i64;

        let interval_duration = Duration::seconds(interval as i64);
        let mut slices = Vec::with_capacity(slice_count as usize);

        for i in 0..slice_count {
            // 计算计划时间
            let scheduled_time = params.start_time + interval_duration * i as i32;

            // 分配余数到前面的切片
            let quantity = if i < remainder as u32 {
                base_quantity + 1
            } else {
                base_quantity
            };

            // 可选随机化
            let (final_quantity, final_time) =
                if params.randomize_quantity || params.randomize_timing {
                    let mut rng = rand::thread_rng();
                    let qty = if params.randomize_quantity && quantity > 0 {
                        let jitter = rng.gen_range(-10..10) as i64;
                        (quantity + jitter)
                            .max(params.min_slice_quantity)
                            .min(params.max_slice_quantity)
                    } else {
                        quantity
                    };
                    let time = if params.randomize_timing {
                        let jitter = rng.gen_range(0..(interval / 2) as i64);
                        scheduled_time + Duration::seconds(jitter)
                    } else {
                        scheduled_time
                    };
                    (qty, time)
                } else {
                    (quantity, scheduled_time)
                };

            if final_quantity > 0 {
                slices.push(Slice {
                    index: i,
                    scheduled_time: final_time,
                    quantity: final_quantity,
                    price: params.price_limit,
                    volume_weight: None,
                });
            }
        }

        slices
    }
}

impl Default for TwapExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AlgorithmExecutor for TwapExecutor {
    fn algo_type(&self) -> AlgoType {
        AlgoType::TWAP
    }

    async fn initialize(&mut self, params: AlgoParams) -> Result<String> {
        // 验证参数
        params.validate().map_err(|e| {
            crate::core::QuantixError::Algo(AlgoError::InvalidParams(e).to_string())
        })?;

        // 生成算法ID
        let algo_id = format!("TWAP-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));

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
            "TWAP algorithm initialized"
        );

        Ok(algo_id)
    }

    async fn start(&mut self, algo_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts
            .get_mut(algo_id)
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

        tracing::info!(algo_id = %algo_id, "TWAP algorithm started");
        Ok(())
    }

    async fn pause(&mut self, algo_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts
            .get_mut(algo_id)
            .ok_or_else(|| AlgoError::NotFound(algo_id.to_string()))?;

        context.state.status = AlgoStatus::Paused;

        tracing::info!(algo_id = %algo_id, "TWAP algorithm paused");
        Ok(())
    }

    async fn resume(&mut self, algo_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts
            .get_mut(algo_id)
            .ok_or_else(|| AlgoError::NotFound(algo_id.to_string()))?;

        context.state.status = AlgoStatus::Running;

        tracing::info!(algo_id = %algo_id, "TWAP algorithm resumed");
        Ok(())
    }

    async fn cancel(&mut self, algo_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts
            .get_mut(algo_id)
            .ok_or_else(|| AlgoError::NotFound(algo_id.to_string()))?;

        context.state.status = AlgoStatus::Cancelled;
        context.state.completed_at = Some(Utc::now());

        tracing::info!(algo_id = %algo_id, "TWAP algorithm cancelled");
        Ok(())
    }

    async fn get_state(&self, algo_id: &str) -> Result<AlgoState> {
        let contexts = self.contexts.read().await;
        let context = contexts
            .get(algo_id)
            .ok_or_else(|| AlgoError::NotFound(algo_id.to_string()))?;
        Ok(context.state.clone())
    }

    async fn step(
        &mut self,
        algo_id: &str,
        _adapter: &dyn ExecutionAdapter,
    ) -> Result<Option<ChildOrder>> {
        let mut contexts = self.contexts.write().await;
        let context = contexts
            .get_mut(algo_id)
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
                    context.next_order_time =
                        Some(slices[context.current_slice as usize].scheduled_time);
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
                    orders
                        .entry(algo_id.to_string())
                        .or_insert_with(Vec::new)
                        .push(child_order.clone());
                }

                tracing::debug!(
                    algo_id = %algo_id,
                    slice = context.current_slice,
                    quantity = slice.quantity,
                    "TWAP slice scheduled"
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
        contexts
            .iter()
            .filter(|(_, ctx)| !ctx.state.is_finished())
            .map(|(id, _)| id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_twap_initialize() {
        let mut executor = TwapExecutor::new();
        let params = AlgoParams::twap("600519.SH".to_string(), "buy".to_string(), 1000, 30);

        let result = executor.initialize(params).await;
        assert!(result.is_ok());

        let algo_id = result.unwrap();
        assert!(algo_id.starts_with("TWAP-"));
    }

    #[test]
    fn test_twap_slice_plan() {
        let executor = TwapExecutor::new();
        let params = AlgoParams::twap("600519.SH".to_string(), "buy".to_string(), 1000, 10)
            .with_slice_count(10)
            .no_randomize();

        let plan = executor.get_slice_plan(&params).unwrap();
        assert_eq!(plan.slices.len(), 10);
        assert_eq!(plan.total_quantity, 1000);
    }
}
