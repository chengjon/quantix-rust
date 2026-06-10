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

pub use senders::{DesktopSender, FeishuSender, LogSender, WebhookSender, WechatWorkSender};

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

    pub fn with_alert_type(mut self, alert_type: AlertType) -> Self {
        self.alert_type = Some(alert_type);
        self
    }

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

/// 通知服务
pub struct NotificationService {
    config: NotificationConfig,
    senders: Vec<Box<dyn NotificationSender>>,
    notification_history: Vec<Notification>,
    max_history: usize,
}

impl NotificationService {
    pub fn new(config: NotificationConfig) -> Self {
        let mut senders: Vec<Box<dyn NotificationSender>> = Vec::new();

        for channel in &config.enabled_channels {
            match channel {
                NotificationChannel::Desktop => {
                    senders.push(Box::new(DesktopSender::new()));
                }
                NotificationChannel::Webhook => {
                    if let Some(url) = &config.webhook_url {
                        senders.push(Box::new(WebhookSender::new(url.clone())));
                    }
                }
                NotificationChannel::Log => {
                    senders.push(Box::new(LogSender::new(config.log_path.clone())));
                }
                NotificationChannel::Email => {
                    // 预留，暂不实现
                }
                NotificationChannel::WechatWork => {
                    if let Some(url) = &config.wechat_work_webhook {
                        senders.push(Box::new(WechatWorkSender::new(url.clone())));
                    }
                }
                NotificationChannel::Feishu => {
                    if let Some(url) = &config.feishu_webhook {
                        senders.push(Box::new(FeishuSender::new(url.clone())));
                    }
                }
                NotificationChannel::Telegram
                | NotificationChannel::Discord
                | NotificationChannel::Slack
                | NotificationChannel::Dingtalk
                | NotificationChannel::Pushplus => {
                    // 预留，暂不实现
                }
            }
        }

        Self {
            config,
            senders,
            notification_history: Vec::new(),
            max_history: 100,
        }
    }

    /// 从默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(NotificationConfig::default())
    }

    /// 发送通知
    pub async fn notify(&mut self, mut notification: Notification) -> Result<()> {
        // 检查级别
        if notification.level < self.config.min_level {
            tracing::debug!(
                "通知级别 {} 低于最低级别 {}，跳过",
                notification.level,
                self.config.min_level
            );
            return Ok(());
        }

        // 检查静默时段
        if let Some(quiet_hours) = &self.config.quiet_hours
            && quiet_hours.is_quiet_now()
        {
            tracing::debug!("当前处于静默时段，跳过通知");
            return Ok(());
        }

        // 发送到所有可用渠道
        let mut any_success = false;
        for sender in &self.senders {
            if sender.is_available() {
                match sender.send(&notification).await {
                    Ok(()) => {
                        any_success = true;
                        tracing::debug!("通知通过 {} 渠道发送成功", sender.channel());
                    }
                    Err(e) => {
                        tracing::warn!("通知通过 {} 渠道发送失败: {}", sender.channel(), e);
                    }
                }
            }
        }

        notification.sent = any_success;
        if !any_success {
            notification.error = Some("所有通知渠道发送失败".to_string());
        }

        // 记录历史
        self.notification_history.push(notification.clone());
        if self.notification_history.len() > self.max_history {
            self.notification_history.remove(0);
        }

        if any_success {
            Ok(())
        } else {
            Err(crate::core::QuantixError::Other(
                "所有通知渠道发送失败".to_string(),
            ))
        }
    }

    /// 发送简单通知
    pub async fn send_notification(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        level: AlertLevel,
    ) -> Result<()> {
        let notification = Notification::new(title, message, level);
        self.notify(notification).await
    }

    /// 获取通知历史
    pub fn get_history(&self) -> &[Notification] {
        &self.notification_history
    }

    /// 清空通知历史
    pub fn clear_history(&mut self) {
        self.notification_history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_notification_creation() {
        let notification = Notification::new("测试标题", "测试消息", AlertLevel::Warning)
            .with_metadata("code", "000001");

        assert_eq!(notification.title, "测试标题");
        assert_eq!(notification.message, "测试消息");
        assert_eq!(notification.level, AlertLevel::Warning);
        assert!(!notification.sent);
        assert_eq!(
            notification.metadata.get("code"),
            Some(&"000001".to_string())
        );
    }

    #[test]
    fn test_notification_format() {
        let notification = Notification::new("测试", "消息", AlertLevel::Error);
        let desktop_text = notification.to_desktop_text();
        assert!(desktop_text.contains("❌"));
        assert!(desktop_text.contains("测试"));

        let log_text = notification.to_log_text();
        assert!(log_text.contains("ERROR"));
    }

    #[test]
    fn test_quiet_hours() {
        // 非跨午夜场景
        let quiet = QuietHours {
            start: "22:00".to_string(),
            end: "06:00".to_string(),
        };
        // 只测试格式是否正确，实际时间判断需要 mock
        assert!(quiet.start != quiet.end);
    }

    #[test]
    fn test_log_sender() {
        let sender = LogSender::new(None);
        assert!(sender.is_available());
        assert_eq!(sender.channel(), NotificationChannel::Log);
    }

    #[tokio::test]
    async fn test_log_sender_creates_parent_dirs_and_writes_file() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("nested").join("notifications.log");
        let sender = LogSender::new(Some(log_path.to_string_lossy().into_owned()));
        let notification = Notification::new("测试通知", "写入文件", AlertLevel::Warning);

        sender.send(&notification).await.unwrap();

        let contents = std::fs::read_to_string(&log_path).unwrap();
        assert!(contents.contains("测试通知"));
        assert!(contents.contains("写入文件"));
    }

    #[tokio::test]
    async fn test_log_sender_returns_error_when_log_path_is_directory() {
        let dir = tempdir().unwrap();
        let sender = LogSender::new(Some(dir.path().to_string_lossy().into_owned()));
        let notification = Notification::new("测试通知", "写入目录应失败", AlertLevel::Error);

        let err = sender.send(&notification).await.unwrap_err();

        assert!(err.to_string().contains("Is a directory"));
    }

    #[test]
    fn test_notification_config_default() {
        let config = NotificationConfig::default();
        assert!(config.enabled_channels.contains(&NotificationChannel::Log));
        assert_eq!(config.min_level, AlertLevel::Warning);
    }

    #[tokio::test]
    async fn test_notification_service() {
        let config = NotificationConfig {
            enabled_channels: vec![NotificationChannel::Log],
            min_level: AlertLevel::Info,
            ..Default::default()
        };
        let mut service = NotificationService::new(config);

        let result = service
            .send_notification("测试通知", "这是一条测试消息", AlertLevel::Warning)
            .await;
        assert!(result.is_ok());

        let history = service.get_history();
        assert_eq!(history.len(), 1);
        assert!(history[0].sent);
    }
}
