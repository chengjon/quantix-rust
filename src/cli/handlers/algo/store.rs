//! 算法管理器的全局内存存储。
//!
//! 从原生 `algo.rs` 拆出，承载进程内单例 [`ALGO_STORE`] 与
//! [`AlgoStore`] 结构体。

use crate::execution::algo::{AlgoContext, AlgorithmExecutor, TwapExecutor, VwapExecutor};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// 全局算法管理器 (简化版，单进程内存存储)
lazy_static::lazy_static! {
    pub(crate) static ref ALGO_STORE: Arc<RwLock<AlgoStore>> =
        Arc::new(RwLock::new(AlgoStore::new()));
}

/// 算法存储
pub(crate) struct AlgoStore {
    /// TWAP 执行器
    pub(crate) twap: TwapExecutor,
    /// VWAP 执行器
    pub(crate) vwap: VwapExecutor,
    /// 算法上下文 (用于状态查询)
    pub(crate) contexts: HashMap<String, AlgoContext>,
    /// 算法类型映射
    pub(crate) algo_types: HashMap<String, crate::execution::algo::AlgoType>,
}

impl AlgoStore {
    pub(crate) fn new() -> Self {
        Self {
            twap: TwapExecutor::new(),
            vwap: VwapExecutor::new(),
            contexts: HashMap::new(),
            algo_types: HashMap::new(),
        }
    }
}
