//! 系统通知模块
//!
//! 提供多渠道通知能力：
//! - 桌面通知（notify-send / Windows toast）
//! - Webhook 通知（HTTP POST）
//! - 日志通知（文件记录）
//!
//! 与告警系统集成，在告警触发时发送通知

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::Result;
use crate::monitoring::{AlertLevel, AlertType};

mod senders;
mod service;

pub use senders::{DesktopSender, FeishuSender, LogSender, WebhookSender, WechatWorkSender};
pub use service::NotificationService;

/// 通知渠道类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    /// 桌面通知（notify-send / Windows toast）
    Desktop,
    /// Webhook HTTP POST
    Webhook,
    /// 日志文件
    Log,
    /// 邮件（预留）
    Email,
    /// Telegram Bot
    Telegram,
    /// 企业微信
    WechatWork,
    /// 飞书
    Feishu,
    /// Discord Webhook
    Discord,
    /// Slack Webhook
    Slack,
    /// 钉钉
    Dingtalk,
    /// PushPlus
    Pushplus,
}

impl std::fmt::Display for NotificationChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Desktop => write!(f, "desktop"),
            Self::Webhook => write!(f, "webhook"),
            Self::Log => write!(f, "log"),
            Self::Email => write!(f, "email"),
            Self::Telegram => write!(f, "telegram"),
            Self::WechatWork => write!(f, "wechat_work"),
            Self::Feishu => write!(f, "feishu"),
            Self::Discord => write!(f, "discord"),
            Self::Slack => write!(f, "slack"),
            Self::Dingtalk => write!(f, "dingtalk"),
            Self::Pushplus => write!(f, "pushplus"),
        }
    }
}

/// 通知配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// 启用的通知渠道
    pub enabled_channels: Vec<NotificationChannel>,
    /// Webhook URL（如果启用）
    pub webhook_url: Option<String>,
    /// 日志文件路径（如果启用）
    pub log_path: Option<String>,
    /// 最低通知级别（低于此级别不通知）
    pub min_level: AlertLevel,
    /// 静默时段（不发送通知）
    pub quiet_hours: Option<QuietHours>,
    /// 通知模板
    pub templates: HashMap<String, String>,
    /// 企业微信 Webhook URL
    pub wechat_work_webhook: Option<String>,
    /// 飞书 Webhook URL
    pub feishu_webhook: Option<String>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled_channels: vec![NotificationChannel::Log],
            webhook_url: None,
            log_path: Some("logs/notifications.log".to_string()),
            min_level: AlertLevel::Warning,
            quiet_hours: None,
            templates: HashMap::new(),
            wechat_work_webhook: None,
            feishu_webhook: None,
        }
    }
}

impl NotificationConfig {
    /// 从环境变量加载配置
    /// 从环境变量构造通知配置：依据 TELEGRAM_BOT_TOKEN / DISCORD_WEBHOOK / SLACK_WEBHOOK / DINGTALK_WEBHOOK / PUSHPLUS_TOKEN / WECHAT_WORK_WEBHOOK / FEISHU_WEBHOOK / WEBHOOK_URL 等存在与否填充 enabled_channels，并解析静默时段与 min_level。缺省项使用默认值。
    pub fn from_env() -> Self {
        let mut enabled_channels = Vec::new();

        // 检查各渠道的环境变量配置
        let has_telegram = std::env::var("TELEGRAM_BOT_TOKEN").is_ok()
            && std::env::var("TELEGRAM_CHAT_ID").is_ok();
        let has_wechat_work = std::env::var("WECHAT_WORK_WEBHOOK_URL").is_ok();
        let has_feishu = std::env::var("FEISHU_WEBHOOK_URL").is_ok();
        let has_discord = std::env::var("DISCORD_WEBHOOK_URL").is_ok();
        let has_slack = std::env::var("SLACK_WEBHOOK_URL").is_ok();
        let has_dingtalk = std::env::var("DINGTALK_WEBHOOK_URL").is_ok();
        let has_pushplus = std::env::var("PUSHPLUS_TOKEN").is_ok();
        let has_webhook = std::env::var("WEBHOOK_URL").is_ok();

        if has_telegram {
            enabled_channels.push(NotificationChannel::Telegram);
        }
        if has_wechat_work {
            enabled_channels.push(NotificationChannel::WechatWork);
        }
        if has_feishu {
            enabled_channels.push(NotificationChannel::Feishu);
        }
        if has_discord {
            enabled_channels.push(NotificationChannel::Discord);
        }
        if has_slack {
            enabled_channels.push(NotificationChannel::Slack);
        }
        if has_dingtalk {
            enabled_channels.push(NotificationChannel::Dingtalk);
        }
        if has_pushplus {
            enabled_channels.push(NotificationChannel::Pushplus);
        }
        if has_webhook {
            enabled_channels.push(NotificationChannel::Webhook);
        }

        // 默认总是启用日志渠道
        enabled_channels.push(NotificationChannel::Log);

        Self {
            enabled_channels,
            webhook_url: std::env::var("WEBHOOK_URL").ok(),
            log_path: std::env::var("NOTIFICATION_LOG_PATH")
                .ok()
                .or(Some("logs/notifications.log".to_string())),
            min_level: std::env::var("NOTIFICATION_MIN_LEVEL")
                .ok()
                .and_then(|s| match s.to_lowercase().as_str() {
                    "info" => Some(AlertLevel::Info),
                    "warning" => Some(AlertLevel::Warning),
                    "error" => Some(AlertLevel::Error),
                    "critical" => Some(AlertLevel::Critical),
                    _ => None,
                })
                .unwrap_or(AlertLevel::Warning),
            quiet_hours: None,
            templates: HashMap::new(),
            wechat_work_webhook: std::env::var("WECHAT_WORK_WEBHOOK_URL").ok(),
            feishu_webhook: std::env::var("FEISHU_WEBHOOK_URL").ok(),
        }
    }
}

/// 静默时段配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    /// 开始时间（HH:MM 格式）
    pub start: String,
    /// 结束时间（HH:MM 格式）
    pub end: String,
}

/// 企业微信配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WechatWorkConfig {
    /// Webhook URL
    pub webhook_url: String,
    /// 消息类型（text/markdown）
    #[serde(default = "default_wechat_msg_type")]
    pub msg_type: String,
}

fn default_wechat_msg_type() -> String {
    "markdown".to_string()
}

/// 飞书配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuConfig {
    /// Webhook URL
    pub webhook_url: String,
    /// 消息类型（text/post/interactive）
    #[serde(default = "default_feishu_msg_type")]
    pub msg_type: String,
}

fn default_feishu_msg_type() -> String {
    "post".to_string()
}

impl QuietHours {
    /// 检查当前时间是否在静默时段内
    /// 判断当前本地时间是否处于静默时段；按字符串 HH:MM 比较，正确处理跨午夜区间（start > end 时取并集）。
    pub fn is_quiet_now(&self) -> bool {
        let now = chrono::Local::now();
        let now_time = now.format("%H:%M").to_string();

        // 处理跨午夜的情况
        if self.start <= self.end {
            now_time >= self.start && now_time <= self.end
        } else {
            now_time >= self.start || now_time <= self.end
        }
    }
}

/// 通知消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// 通知 ID
    pub id: String,
    /// 通知标题
    pub title: String,
    /// 通知内容
    pub message: String,
    /// 通知级别
    pub level: AlertLevel,
    /// 来源告警类型
    pub alert_type: Option<AlertType>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 是否已发送
    pub sent: bool,
    /// 发送失败原因
    pub error: Option<String>,
    /// 额外元数据
    pub metadata: HashMap<String, String>,
}

impl Notification {
    /// 创建一条新通知：分配 UUID、记录 created_at=Utc::now、sent=false、metadata 空；alert_type 与 metadata 需通过 with_* 链式补充。
    pub fn new(title: impl Into<String>, message: impl Into<String>, level: AlertLevel) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.into(),
            message: message.into(),
            level,
            alert_type: None,
            created_at: Utc::now(),
            sent: false,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// 链式设置 alert_type（用于路由与历史分类）；返回 self 以便链式调用。
    pub fn with_alert_type(mut self, alert_type: AlertType) -> Self {
        self.alert_type = Some(alert_type);
        self
    }

    /// 链式追加 metadata 键值（同一 key 后写覆盖先写）；返回 self 以便链式调用。
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// 格式化为桌面通知文本
    pub fn to_desktop_text(&self) -> String {
        let level_emoji = match self.level {
            AlertLevel::Info => "ℹ️",
            AlertLevel::Warning => "⚠️",
            AlertLevel::Error => "❌",
            AlertLevel::Critical => "🚨",
        };
        format!("{} {} - {}", level_emoji, self.title, self.message)
    }

    /// 格式化为日志文本
    pub fn to_log_text(&self) -> String {
        format!(
            "[{}] [{}] {} - {}",
            self.created_at.format("%Y-%m-%d %H:%M:%S"),
            match self.level {
                AlertLevel::Info => "INFO",
                AlertLevel::Warning => "WARN",
                AlertLevel::Error => "ERROR",
                AlertLevel::Critical => "CRIT",
            },
            self.title,
            self.message
        )
    }
}

/// 通知发送器 trait
#[async_trait]
pub trait NotificationSender: Send + Sync {
    /// 发送通知
    async fn send(&self, notification: &Notification) -> Result<()>;

    /// 获取渠道类型
    fn channel(&self) -> NotificationChannel;

    /// 检查渠道是否可用
    fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests;
