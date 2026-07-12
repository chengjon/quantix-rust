//! Algorithmic Trading Module
//!
//! 提供算法交易实现，包括 TWAP、VWAP 等常见算法

mod algo_type;
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
pub use executor::{AlgoError, AlgoResult, AlgorithmExecutor, Slice, SlicePlan};

// Re-export algorithm executors
pub use twap::TwapExecutor;
pub use vwap::VwapExecutor;

// Re-export AlgoType enum
pub use algo_type::AlgoType;
