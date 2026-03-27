//! Algorithm Executor Trait
//!
//! 算法执行器接口定义

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{AlgoType, AlgoContext, AlgoParams, AlgoState, ChildOrder};
use crate::execution::adapter::ExecutionAdapter;
use crate::core::Result;

/// 算法执行错误
#[derive(Debug, thiserror::Error)]
pub enum AlgoError {
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Algorithm not found: {0}")]
    NotFound(String),

    #[error("Algorithm already running: {0}")]
    AlreadyRunning(String),

    #[error("Algorithm not running: {0}")]
    NotRunning(String),

    #[error("Order failed: {0}")]
    OrderFailed(String),

    #[error("Market data unavailable: {0}")]
    MarketDataUnavailable(String),

    #[error("Timeout exceeded")]
    Timeout,

    #[error("Internal error: {0}")]
    Internal(String),
}

/// 算法执行结果
#[derive(Debug, Clone)]
pub struct AlgoResult {
    /// 算法ID
    pub algo_id: String,
    /// 最终状态
    pub final_state: AlgoState,
    /// 总成交数量
    pub total_filled: i64,
    /// 平均成交价
    pub avg_price: Decimal,
    /// 总成交金额
    pub total_amount: Decimal,
    /// 订单数量
    pub order_count: u32,
    /// 执行时长 (秒)
    pub duration_seconds: i64,
    /// 滑点 (基点)
    pub slippage_bps: Option<Decimal>,
    /// VWAP 比较 (如果适用)
    pub vwap_comparison: Option<Decimal>,
}

impl AlgoResult {
    /// 计算执行表现
    pub fn performance_summary(&self) -> String {
        format!(
            "Algorithm {} completed: filled {}/{} orders, avg price {:.4}, duration {}s",
            self.algo_id,
            self.order_count,
            self.order_count,
            self.avg_price,
            self.duration_seconds
        )
    }
}

/// 切片计划
#[derive(Debug, Clone)]
pub struct SlicePlan {
    /// 切片列表
    pub slices: Vec<Slice>,
    /// 总数量
    pub total_quantity: i64,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: DateTime<Utc>,
}

/// 单个切片
#[derive(Debug, Clone)]
pub struct Slice {
    /// 切片索引
    pub index: u32,
    /// 计划时间
    pub scheduled_time: DateTime<Utc>,
    /// 计划数量
    pub quantity: i64,
    /// 计划价格 (可选)
    pub price: Option<Decimal>,
    /// 成交量权重 (用于 VWAP)
    pub volume_weight: Option<Decimal>,
}

/// 算法执行器 Trait
#[async_trait]
pub trait AlgorithmExecutor: Send + Sync {
    /// 获取算法类型
    fn algo_type(&self) -> AlgoType;

    /// 初始化算法
    async fn initialize(&mut self, params: AlgoParams) -> Result<String>;

    /// 启动算法
    async fn start(&mut self, algo_id: &str) -> Result<()>;

    /// 暂停算法
    async fn pause(&mut self, algo_id: &str) -> Result<()>;

    /// 恢复算法
    async fn resume(&mut self, algo_id: &str) -> Result<()>;

    /// 取消算法
    async fn cancel(&mut self, algo_id: &str) -> Result<()>;

    /// 获取算法状态
    async fn get_state(&self, algo_id: &str) -> Result<AlgoState>;

    /// 执行一步算法 (由调度器调用)
    async fn step(&mut self, algo_id: &str, adapter: &dyn ExecutionAdapter) -> Result<Option<ChildOrder>>;

    /// 获取切片计划
    fn get_slice_plan(&self, params: &AlgoParams) -> Result<SlicePlan>;

    /// 获取所有活跃算法
    fn get_active_algos(&self) -> Vec<String>;
}

/// 算法管理器
pub struct AlgoManager {
    /// 活跃的算法
    algos: Arc<RwLock<std::collections::HashMap<String, AlgoContext>>>,
}

impl AlgoManager {
    /// 创建新的算法管理器
    pub fn new() -> Self {
        Self {
            algos: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// 添加算法
    pub async fn add_algo(&self, algo_id: String, context: AlgoContext) {
        let mut algos = self.algos.write().await;
        algos.insert(algo_id, context);
    }

    /// 获取算法
    pub async fn get_algo(&self, algo_id: &str) -> Option<AlgoContext> {
        let algos = self.algos.read().await;
        algos.get(algo_id).cloned()
    }

    /// 更新算法
    pub async fn update_algo(&self, algo_id: &str, context: AlgoContext) {
        let mut algos = self.algos.write().await;
        algos.insert(algo_id.to_string(), context);
    }

    /// 移除算法
    pub async fn remove_algo(&self, algo_id: &str) -> Option<AlgoContext> {
        let mut algos = self.algos.write().await;
        algos.remove(algo_id)
    }

    /// 获取所有活跃算法
    pub async fn get_active_algos(&self) -> Vec<(String, AlgoContext)> {
        let algos = self.algos.read().await;
        algos.iter()
            .filter(|(_, ctx)| !ctx.state.is_finished())
            .map(|(id, ctx)| (id.clone(), ctx.clone()))
            .collect()
    }

    /// 获取需要执行的算法
    pub async fn get_ready_algos(&self) -> Vec<String> {
        let algos = self.algos.read().await;
        algos.iter()
            .filter(|(_, ctx)| ctx.should_order_now())
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl Default for AlgoManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algo_error_display() {
        let err = AlgoError::InvalidParams("test".to_string());
        assert!(err.to_string().contains("test"));
    }
}
