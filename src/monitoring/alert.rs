/// 告警系统模块
///
/// 提供阈值告警和通知功能
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 告警级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for AlertLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// 告警类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertType {
    /// 信号告警
    Signal { strategy: String, code: String },
    /// 持仓告警
    Position { code: String, reason: String },
    /// 性能告警
    Performance { metric: String, value: Decimal },
    /// 风险告警
    Risk { reason: String },
    /// 系统告警
    System { message: String },
}

/// 告警阈值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThreshold {
    /// 告警类型标识
    pub id: String,
    /// 名称
    pub name: String,
    /// 阈值
    pub threshold: Decimal,
    /// 告警级别
    pub level: AlertLevel,
    /// 是否启用
    pub enabled: bool,
    /// 冷却时间（秒）
    pub cooldown_secs: u64,
    /// 最后告警时间
    pub last_alert_time: Option<DateTime<Utc>>,
}

impl AlertThreshold {
    /// 创建新的告警阈值
    pub fn new(id: String, name: String, threshold: Decimal, level: AlertLevel) -> Self {
        Self {
            id,
            name,
            threshold,
            level,
            enabled: true,
            cooldown_secs: 300, // 默认5分钟
            last_alert_time: None,
        }
    }

    /// 检查是否应该告警（考虑冷却时间）
    pub fn should_alert(&self, current_value: Decimal) -> bool {
        if !self.enabled {
            return false;
        }

        // 检查阈值
        let threshold_exceeded = match self.level {
            AlertLevel::Info => current_value >= self.threshold,
            AlertLevel::Warning => current_value >= self.threshold,
            AlertLevel::Error => current_value >= self.threshold,
            AlertLevel::Critical => current_value >= self.threshold,
        };

        if !threshold_exceeded {
            return false;
        }

        // 检查冷却时间
        if let Some(last_time) = self.last_alert_time {
            let elapsed = (Utc::now() - last_time).num_seconds() as u64;
            elapsed >= self.cooldown_secs
        } else {
            true
        }
    }

    /// 更新最后告警时间
    pub fn update_last_alert(&mut self) {
        self.last_alert_time = Some(Utc::now());
    }

    /// 启用告警
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// 禁用告警
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

/// 告警配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// 启用告警系统
    pub enabled: bool,
    /// 默认冷却时间（秒）
    pub default_cooldown_secs: u64,
    /// 启用控制台输出
    pub enable_console_output: bool,
    /// 启用日志输出
    pub enable_log_output: bool,
    /// 最大告警历史数量
    pub max_alert_history: usize,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_cooldown_secs: 300,
            enable_console_output: true,
            enable_log_output: true,
            max_alert_history: 1000,
        }
    }
}

/// 告警
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// 告警ID
    pub id: String,
    /// 告警级别
    pub level: AlertLevel,
    /// 告警类型
    pub alert_type: AlertType,
    /// 消息
    pub message: String,
    /// 当前值
    pub current_value: Option<Decimal>,
    /// 阈值
    pub threshold: Option<Decimal>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 是否已确认
    pub acknowledged: bool,
    /// 额外信息
    pub metadata: HashMap<String, String>,
}

impl Alert {
    /// 创建新告警
    pub fn new(level: AlertLevel, alert_type: AlertType, message: String) -> Self {
        Self {
            id: format!("alert_{}", Utc::now().timestamp_millis()),
            level,
            alert_type,
            message,
            current_value: None,
            threshold: None,
            timestamp: Utc::now(),
            acknowledged: false,
            metadata: HashMap::new(),
        }
    }

    /// 设置当前值
    pub fn with_current_value(mut self, value: Decimal) -> Self {
        self.current_value = Some(value);
        self
    }

    /// 设置阈值
    pub fn with_threshold(mut self, threshold: Decimal) -> Self {
        self.threshold = Some(threshold);
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// 确认告警
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }

    /// 格式化消息
    pub fn format_message(&self) -> String {
        let level_str = match self.level {
            AlertLevel::Info => "[INFO]",
            AlertLevel::Warning => "[WARNING]",
            AlertLevel::Error => "[ERROR]",
            AlertLevel::Critical => "[CRITICAL]",
        };

        let mut msg = format!(
            "{} {} - {}",
            level_str,
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.message
        );

        if let (Some(current), Some(threshold)) = (self.current_value, self.threshold) {
            msg.push_str(&format!(" (当前: {:.2}, 阈值: {:.2})", current, threshold));
        }

        msg
    }
}

/// 告警管理器
pub struct AlertManager {
    /// 配置
    config: AlertConfig,
    /// 告警阈值
    thresholds: HashMap<String, AlertThreshold>,
    /// 告警历史
    alert_history: Vec<Alert>,
    /// 活跃告警（未确认）
    active_alerts: Vec<Alert>,
    /// 告警计数器
    alert_counter: HashMap<String, usize>,
}

impl AlertManager {
    /// 创建新的告警管理器
    pub fn new(config: AlertConfig) -> Self {
        Self {
            config,
            thresholds: HashMap::new(),
            alert_history: Vec::new(),
            active_alerts: Vec::new(),
            alert_counter: HashMap::new(),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(AlertConfig::default())
    }

    /// 添加告警阈值
    pub fn add_threshold(&mut self, threshold: AlertThreshold) {
        self.thresholds.insert(threshold.id.clone(), threshold);
    }

    /// 移除告警阈值
    pub fn remove_threshold(&mut self, id: &str) {
        self.thresholds.remove(id);
    }

    /// 获取阈值
    pub fn get_threshold(&self, id: &str) -> Option<&AlertThreshold> {
        self.thresholds.get(id)
    }

    /// 获取可变阈值
    pub fn get_threshold_mut(&mut self, id: &str) -> Option<&mut AlertThreshold> {
        self.thresholds.get_mut(id)
    }

    /// 检查并发送告警
    pub fn check_and_alert(&mut self, threshold_id: &str, current_value: Decimal) -> Option<Alert> {
        if !self.config.enabled {
            return None;
        }

        // Check if threshold exists and should alert
        let (should_alert, level, name, threshold_value) = {
            let threshold = self.thresholds.get(threshold_id)?;
            (
                threshold.should_alert(current_value),
                threshold.level,
                threshold.name.clone(),
                threshold.threshold,
            )
        };

        if should_alert {
            // Update the threshold's last alert time
            if let Some(threshold) = self.thresholds.get_mut(threshold_id) {
                threshold.update_last_alert();
            }

            let alert = Alert::new(
                level,
                AlertType::Performance {
                    metric: name.clone(),
                    value: current_value,
                },
                format!("阈值告警: {} 超过阈值", name),
            )
            .with_current_value(current_value)
            .with_threshold(threshold_value);

            Some(self.send_alert(alert))
        } else {
            None
        }
    }

    /// 发送告警
    pub fn send_alert(&mut self, alert: Alert) -> Alert {
        // 输出到控制台
        if self.config.enable_console_output {
            println!("{}", alert.format_message());
        }

        // 输出到日志
        if self.config.enable_log_output {
            match alert.level {
                AlertLevel::Info => tracing::info!("{}", alert.message),
                AlertLevel::Warning => tracing::warn!("{}", alert.message),
                AlertLevel::Error => tracing::error!("{}", alert.message),
                AlertLevel::Critical => tracing::error!("{}", alert.message),
            }
        }

        // 更新计数器
        let key = format!("{:?}", alert.alert_type);
        *self.alert_counter.entry(key).or_insert(0) += 1;

        // 添加到活跃告警
        self.active_alerts.push(alert.clone());

        // 添加到历史
        self.alert_history.push(alert.clone());

        // 限制历史大小
        if self.alert_history.len() > self.config.max_alert_history {
            self.alert_history.remove(0);
        }

        alert
    }

    /// 确认告警
    pub fn acknowledge_alert(&mut self, alert_id: &str) -> bool {
        // 确认活跃告警
        for alert in &mut self.active_alerts {
            if alert.id == alert_id {
                alert.acknowledge();
                return true;
            }
        }

        // 确认历史告警
        for alert in &mut self.alert_history {
            if alert.id == alert_id {
                alert.acknowledge();
                return true;
            }
        }

        false
    }

    /// 获取活跃告警
    pub fn get_active_alerts(&self) -> &[Alert] {
        &self.active_alerts
    }

    /// 获取未确认的活跃告警
    pub fn get_unacknowledged_alerts(&self) -> Vec<&Alert> {
        self.active_alerts
            .iter()
            .filter(|a| !a.acknowledged)
            .collect()
    }

    /// 获取告警历史
    pub fn get_alert_history(&self) -> &[Alert] {
        &self.alert_history
    }

    /// 获取告警统计
    pub fn get_alert_stats(&self) -> &HashMap<String, usize> {
        &self.alert_counter
    }

    /// 清空活跃告警
    pub fn clear_active_alerts(&mut self) {
        self.active_alerts.clear();
    }

    /// 清空告警历史
    pub fn clear_history(&mut self) {
        self.alert_history.clear();
    }

    /// 重置计数器
    pub fn reset_counter(&mut self) {
        self.alert_counter.clear();
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_alert_manager_creation() {
        let manager = AlertManager::with_defaults();
        assert_eq!(manager.get_active_alerts().len(), 0);
        assert_eq!(manager.get_alert_history().len(), 0);
    }

    #[test]
    fn test_send_alert() {
        let mut manager = AlertManager::with_defaults();

        let alert = Alert::new(
            AlertLevel::Warning,
            AlertType::System {
                message: "测试告警".to_string(),
            },
            "这是一个测试告警".to_string(),
        );

        manager.send_alert(alert);

        assert_eq!(manager.get_active_alerts().len(), 1);
        assert_eq!(manager.get_alert_history().len(), 1);
    }

    #[test]
    fn test_check_and_alert() {
        let mut manager = AlertManager::with_defaults();

        let threshold = AlertThreshold::new(
            "test_threshold".to_string(),
            "测试阈值".to_string(),
            Decimal::from(10),
            AlertLevel::Warning,
        );

        manager.add_threshold(threshold);

        // 未超过阈值
        let alert = manager.check_and_alert("test_threshold", Decimal::from(5));
        assert!(alert.is_none());

        // 超过阈值
        let alert = manager.check_and_alert("test_threshold", Decimal::from(15));
        assert!(alert.is_some());
    }

    #[test]
    fn test_alert_cooldown() {
        let mut manager = AlertManager::with_defaults();

        let mut threshold = AlertThreshold::new(
            "test_threshold".to_string(),
            "测试阈值".to_string(),
            Decimal::from(10),
            AlertLevel::Warning,
        );
        threshold.cooldown_secs = 10; // 10秒冷却

        manager.add_threshold(threshold);

        // 第一次触发
        let alert1 = manager.check_and_alert("test_threshold", Decimal::from(15));
        assert!(alert1.is_some());

        // 冷却期内，不应该再次触发
        let alert2 = manager.check_and_alert("test_threshold", Decimal::from(15));
        assert!(alert2.is_none(), "Should not alert during cooldown period");
    }

    #[test]
    fn test_acknowledge_alert() {
        let mut manager = AlertManager::with_defaults();

        let alert = Alert::new(
            AlertLevel::Warning,
            AlertType::System {
                message: "测试告警".to_string(),
            },
            "这是一个测试告警".to_string(),
        );

        let alert = manager.send_alert(alert);
        let alert_id = alert.id.clone();

        assert!(!manager.get_active_alerts()[0].acknowledged);

        manager.acknowledge_alert(&alert_id);

        assert!(manager.get_active_alerts()[0].acknowledged);
    }

    #[test]
    fn test_get_unacknowledged_alerts() {
        let mut manager = AlertManager::with_defaults();

        let alert1 = Alert::new(
            AlertLevel::Warning,
            AlertType::System {
                message: "测试告警1".to_string(),
            },
            "告警1".to_string(),
        );

        let alert2 = Alert::new(
            AlertLevel::Info,
            AlertType::System {
                message: "测试告警2".to_string(),
            },
            "告警2".to_string(),
        );

        manager.send_alert(alert1);
        manager.send_alert(alert2);

        let unacknowledged = manager.get_unacknowledged_alerts();
        assert_eq!(unacknowledged.len(), 2);
    }

    #[test]
    fn test_alert_threshold_builder() {
        let drawdown_threshold = AlertThresholdBuilder::drawdown_warning(0.1);
        assert_eq!(
            drawdown_threshold.threshold,
            Decimal::from_f64_retain(0.1).unwrap()
        );
        assert_eq!(drawdown_threshold.level, AlertLevel::Warning);

        let critical_threshold = AlertThresholdBuilder::drawdown_critical(0.2);
        assert_eq!(critical_threshold.level, AlertLevel::Critical);
    }

    #[test]
    fn test_drawdown_threshold_value() {
        let threshold = AlertThresholdBuilder::drawdown_warning(0.1);
        // Test with actual decimal value
        let expected = Decimal::from_f64_retain(0.1).unwrap();
        assert_eq!(threshold.threshold, expected);
    }

    #[test]
    fn test_critical_threshold_value() {
        let threshold = AlertThresholdBuilder::drawdown_critical(0.2);
        let expected = Decimal::from_f64_retain(0.2).unwrap();
        assert_eq!(threshold.threshold, expected);
    }

    #[test]
    fn test_position_ratio_threshold_value() {
        let threshold = AlertThresholdBuilder::position_ratio(0.3);
        let expected = Decimal::from_f64_retain(0.3).unwrap();
        assert_eq!(threshold.threshold, expected);
    }

    #[test]
    fn test_signal_frequency_threshold_value() {
        let threshold = AlertThresholdBuilder::signal_frequency(5.0);
        let expected = Decimal::from_f64_retain(5.0).unwrap();
        assert_eq!(threshold.threshold, expected);
    }

    #[test]
    fn test_enable_disable_threshold() {
        let mut threshold = AlertThreshold::new(
            "test".to_string(),
            "测试".to_string(),
            Decimal::from(10),
            AlertLevel::Warning,
        );

        assert!(threshold.enabled);

        threshold.disable();
        assert!(!threshold.enabled);

        threshold.enable();
        assert!(threshold.enabled);
    }

    #[test]
    fn test_alert_format_message() {
        let alert = Alert::new(
            AlertLevel::Warning,
            AlertType::System {
                message: "测试告警".to_string(),
            },
            "这是一个测试告警".to_string(),
        )
        .with_current_value(dec!(15.5))
        .with_threshold(dec!(10.0));

        let msg = alert.format_message();
        assert!(msg.contains("[WARNING]"));
        assert!(msg.contains("15.50"));
        assert!(msg.contains("10.00"));
    }

    #[test]
    fn test_clear_active_alerts() {
        let mut manager = AlertManager::with_defaults();

        manager.send_alert(Alert::new(
            AlertLevel::Warning,
            AlertType::System {
                message: "测试".to_string(),
            },
            "测试告警".to_string(),
        ));

        assert_eq!(manager.get_active_alerts().len(), 1);

        manager.clear_active_alerts();
        assert_eq!(manager.get_active_alerts().len(), 0);
    }

    #[test]
    fn test_alert_stats() {
        let mut manager = AlertManager::with_defaults();

        manager.send_alert(Alert::new(
            AlertLevel::Warning,
            AlertType::System {
                message: "测试".to_string(),
            },
            "测试告警".to_string(),
        ));

        manager.send_alert(Alert::new(
            AlertLevel::Error,
            AlertType::System {
                message: "测试".to_string(),
            },
            "测试告警".to_string(),
        ));

        let stats = manager.get_alert_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats.values().next().unwrap(), &2);
    }
}
