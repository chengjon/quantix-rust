/// 告警系统模块
///
/// 提供阈值告警和通知功能
pub mod manager;
pub mod types;

#[cfg(test)]
mod tests;

pub use manager::AlertManager;
pub use types::{Alert, AlertConfig, AlertLevel, AlertThreshold, AlertThresholdBuilder, AlertType};
