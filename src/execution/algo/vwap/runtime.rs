#![allow(clippy::collapsible_if)]

use super::*;

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
        params.validate().map_err(|e| {
            crate::core::QuantixError::Algo(AlgoError::InvalidParams(e).to_string())
        })?;

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

        tracing::info!(algo_id = %algo_id, "VWAP algorithm started");
        Ok(())
    }

    async fn pause(&mut self, algo_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts
            .get_mut(algo_id)
            .ok_or_else(|| AlgoError::NotFound(algo_id.to_string()))?;
        context.state.status = AlgoStatus::Paused;

        tracing::info!(algo_id = %algo_id, "VWAP algorithm paused");
        Ok(())
    }

    async fn resume(&mut self, algo_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts
            .get_mut(algo_id)
            .ok_or_else(|| AlgoError::NotFound(algo_id.to_string()))?;
        context.state.status = AlgoStatus::Running;

        tracing::info!(algo_id = %algo_id, "VWAP algorithm resumed");
        Ok(())
    }

    async fn cancel(&mut self, algo_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts
            .get_mut(algo_id)
            .ok_or_else(|| AlgoError::NotFound(algo_id.to_string()))?;
        context.state.status = AlgoStatus::Cancelled;
        context.state.completed_at = Some(Utc::now());

        tracing::info!(algo_id = %algo_id, "VWAP algorithm cancelled");
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
        contexts
            .iter()
            .filter(|(_, ctx)| !ctx.state.is_finished())
            .map(|(id, _)| id.clone())
            .collect()
    }
}
