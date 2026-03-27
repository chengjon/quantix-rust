//! Algorithmic Trading Module
//!
//! 提供算法交易实现，包括 TWAP、VWAP 等常见算法

mod context;
mod executor;
mod state;
mod twap;
mod vwap;

// Re-export from state
pub use state::{AlgoState, AlgoStatus, ChildOrder, ChildOrderStatus};

// Re-export from context
pub use context::{AlgoContext, AlgoParams};

// Re-export from executor
pub use executor::{AlgorithmExecutor, AlgoError, AlgoResult, Slice, SlicePlan};

// Re-export algorithm executors
pub use twap::TwapExecutor;
pub use vwap::VwapExecutor;

/// 算法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AlgoType {
    /// 时间加权平均价格
    TWAP,
    /// 成交量加权平均价格
    VWAP,
    /// 参与率 (Percentage of Volume)
    POV,
    /// 冰山订单
    Iceberg,
}

impl std::fmt::Display for AlgoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlgoType::TWAP => write!(f, "TWAP"),
            AlgoType::VWAP => write!(f, "VWAP"),
            AlgoType::POV => write!(f, "POV"),
            AlgoType::Iceberg => write!(f, "Iceberg"),
        }
    }
}

impl std::str::FromStr for AlgoType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TWAP" => Ok(AlgoType::TWAP),
            "VWAP" => Ok(AlgoType::VWAP),
            "POV" => Ok(AlgoType::POV),
            "ICEBERG" => Ok(AlgoType::Iceberg),
            _ => Err(format!("Unknown algorithm type: {}", s)),
        }
    }
}
