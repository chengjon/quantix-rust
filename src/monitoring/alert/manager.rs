//! Alert manager: [`AlertManager`].

use rust_decimal::Decimal;
use std::collections::HashMap;

use super::types::{Alert, AlertConfig, AlertLevel, AlertThreshold, AlertType};

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
            tracing::info!(
                target: "quantix::alert_console",
                "{}",
                alert.format_message()
            );
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
        self.alert_counter.clear()
    }
}
