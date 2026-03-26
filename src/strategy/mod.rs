/// 策略模块
///
/// 策略 trait 定义和实现
pub mod trait_def;

// 策略实现
pub mod breakout;
pub mod grid;
pub mod ma_cross;
pub mod mean_reversion;
pub mod momentum;

// 测试工具（仅测试时编译）
#[cfg(test)]
pub mod test_utils;

pub mod config;
pub mod daemon;
pub mod fallback_loader;
pub mod registry;
pub mod runtime;
pub mod service_config;
pub mod systemd;

pub use config::{
    BootstrapPolicy, ConfiguredStock, ConfiguredStrategyInstance, JsonStrategyConfigStore,
    StrategyDaemonConfig,
};
pub use daemon::StrategySignalDaemon;
pub use fallback_loader::{
    FallbackStrategyBarLoader, LEGACY_TDX_MARKET_ENV, LEGACY_TDX_ROOT_ENV, STRATEGY_TDX_MARKET_ENV,
    STRATEGY_TDX_ROOT_ENV, StrategyBarLoadSource,
};
pub use registry::{ConfiguredStrategyEvaluator, StrategyRegistry};
pub use service_config::{JsonStrategyServiceConfigStore, StrategyServiceConfig};
pub use systemd::{StrategyServiceStatusSummary, StrategyUserServiceInstaller};
pub use trait_def::Strategy;

// 导出具体策略
pub use breakout::{BreakoutConfig, BreakoutStrategy};
pub use grid::{GridConfig, GridStrategy};
pub use ma_cross::MACrossStrategy;
pub use mean_reversion::{MeanReversionConfig, MeanReversionStrategy};
pub use momentum::{MomentumConfig, MomentumStrategy};
