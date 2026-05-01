/// 告警系统模块
///
/// 提供阈值告警和通知功能
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[path = "alert_builder.rs"]
mod alert_builder;

pub use self::alert_builder::AlertThresholdBuilder;

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

#[cfg(test)]
#[path = "alert_test.rs"]
mod tests;
