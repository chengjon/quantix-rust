//! Account Management Module
//!
//! 多账户管理系统，支持:
//! - 多个 paper trading 账户
//! - 多个 live trading 账户
//! - 账户组管理
//! - 智能订单路由

pub mod models;
pub mod registry;
pub mod router;
pub mod storage;

pub use models::{
    AccountConfig, AccountGroup, AccountSummary, AccountType, AllocationStrategy,
    OrderSplitRequest, OrderSplitResult, SplitOrder, SplitTarget,
};
pub use registry::AccountRegistry;
pub use router::AccountRouter;
pub use storage::JsonAccountRegistryStore;
