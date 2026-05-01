use super::*;

/// 预定义的告警阈值构建器
pub struct AlertThresholdBuilder;

impl AlertThresholdBuilder {
    /// 回撤告警
    pub fn drawdown_warning(threshold: f64) -> AlertThreshold {
        AlertThreshold::new(
            "drawdown_warning".to_string(),
            "回撤告警".to_string(),
            Decimal::from_f64_retain(threshold).unwrap_or(Decimal::ZERO),
            AlertLevel::Warning,
        )
    }

    /// 回撤严重告警
    pub fn drawdown_critical(threshold: f64) -> AlertThreshold {
        AlertThreshold::new(
            "drawdown_critical".to_string(),
            "严重回撤告警".to_string(),
            Decimal::from_f64_retain(threshold).unwrap_or(Decimal::ZERO),
            AlertLevel::Critical,
        )
    }

    /// 持仓比例告警
    pub fn position_ratio(threshold: f64) -> AlertThreshold {
        AlertThreshold::new(
            "position_ratio".to_string(),
            "持仓比例告警".to_string(),
            Decimal::from_f64_retain(threshold).unwrap_or(Decimal::ZERO),
            AlertLevel::Warning,
        )
    }

    /// 信号频率告警
    pub fn signal_frequency(threshold: f64) -> AlertThreshold {
        AlertThreshold::new(
            "signal_frequency".to_string(),
            "信号频率告警".to_string(),
            Decimal::from_f64_retain(threshold).unwrap_or(Decimal::ZERO),
            AlertLevel::Info,
        )
    }
}
