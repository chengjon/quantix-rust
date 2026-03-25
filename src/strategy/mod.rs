/// 策略模块
///
/// 策略 trait 定义和实现
pub mod trait_def;

// 策略实现
pub mod ma_cross;
pub mod mean_reversion;
pub mod momentum;
pub mod breakout;
pub mod grid;
pub mod runtime;
pub mod config;
pub mod service_config;

// 测试工具（仅测试时编译）
#[cfg(test)]
pub mod test_utils;

pub use trait_def::Strategy;

// 导出具体策略
pub use ma_cross::MACrossStrategy;
pub use mean_reversion::{MeanReversionStrategy, MeanReversionConfig};
pub use momentum::{MomentumStrategy, MomentumConfig};
pub use breakout::{BreakoutStrategy, BreakoutConfig};
pub use grid::{GridStrategy, GridConfig};
pub use config::{
    BootstrapPolicy, ConfiguredStock, ConfiguredStrategyInstance, JsonStrategyConfigStore,
    StrategyDaemonConfig,
};
pub use runtime::{StrategyBarLoader, StrategyRuntime};
pub use service_config::{JsonStrategyServiceConfigStore, StrategyServiceConfig};
