//! Alert data types: [`AlertLevel`], [`AlertType`], [`AlertThreshold`],
//! [`AlertConfig`], [`Alert`], [`AlertThresholdBuilder`].

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
