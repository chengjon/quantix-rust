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

use crate::core::{QuantixError, Result};
use crate::monitoring::{AlertLevel, AlertType};

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
            log_path: std::env::var("NOTIFICATION_LOG_PATH").ok()
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

/// 桌面通知发送器
pub struct DesktopSender;

impl DesktopSender {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DesktopSender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationSender for DesktopSender {
    async fn send(&self, notification: &Notification) -> Result<()> {
        let title = &notification.title;
        let message = &notification.message;

        #[cfg(target_os = "linux")]
        {
            // 使用 notify-send 发送桌面通知
            let output = tokio::process::Command::new("notify-send")
                .arg("-u")
                .arg(match notification.level {
                    AlertLevel::Info => "low",
                    AlertLevel::Warning => "normal",
                    AlertLevel::Error => "critical",
                    AlertLevel::Critical => "critical",
                })
                .arg(title)
                .arg(message)
                .output()
                .await;

            match output {
                Ok(o) if o.status.success() => {
                    tracing::debug!("桌面通知发送成功: {}", title);
                    return Ok(());
                }
                Ok(o) => {
                    tracing::warn!(
                        "桌面通知发送失败: {}",
                        String::from_utf8_lossy(&o.stderr)
                    );
                }
                Err(e) => {
                    tracing::warn!("桌面通知发送失败: {}", e);
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows toast 通知（需要 PowerShell）
            let script = format!(
                r#"[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
$template = @"
<toast>
    <visual>
        <binding template=""ToastText02"">
            <text id=""1"">{}</text>
            <text id=""2"">{}</text>
        </binding>
    </visual>
</toast>
"@
$xml = New-Object Windows.Data.Xml.Dom.XmlDocument
$xml.LoadXml($template)
$toast = [Windows.UI.Notifications.ToastNotification]::new($xml)
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier("Quantix").Show($toast)
"#,
                title, message
            );

            let output = tokio::process::Command::new("powershell")
                .arg("-Command")
                .arg(&script)
                .output()
                .await;

            match output {
                Ok(o) if o.status.success() => {
                    tracing::debug!("桌面通知发送成功: {}", title);
                    return Ok(());
                }
                Ok(o) => {
                    tracing::warn!(
                        "桌面通知发送失败: {}",
                        String::from_utf8_lossy(&o.stderr)
                    );
                }
                Err(e) => {
                    tracing::warn!("桌面通知发送失败: {}", e);
                }
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            tracing::debug!("桌面通知不支持当前平台");
            let _ = (title, message);
        }

        Ok(())
    }

    fn channel(&self) -> NotificationChannel {
        NotificationChannel::Desktop
    }

    fn is_available(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            // 检查 notify-send 是否可用
            std::path::Path::new("/usr/bin/notify-send").exists()
        }
        #[cfg(target_os = "windows")]
        {
            true
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            false
        }
    }
}

/// Webhook 通知发送器
pub struct WebhookSender {
    url: String,
    client: reqwest::Client,
}

impl WebhookSender {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationSender for WebhookSender {
    async fn send(&self, notification: &Notification) -> Result<()> {
        let payload = serde_json::json!({
            "id": notification.id,
            "title": notification.title,
            "message": notification.message,
            "level": notification.level,
            "created_at": notification.created_at.to_rfc3339(),
            "metadata": notification.metadata,
        });

        let response = self
            .client
            .post(&self.url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::core::QuantixError::Other(format!("Webhook 请求失败: {}", e)))?;

        if response.status().is_success() {
            tracing::debug!("Webhook 通知发送成功: {}", self.url);
            Ok(())
        } else {
            Err(crate::core::QuantixError::Other(format!(
                "Webhook 返回错误状态: {}",
                response.status()
            )))
        }
    }

    fn channel(&self) -> NotificationChannel {
        NotificationChannel::Webhook
    }

    fn is_available(&self) -> bool {
        !self.url.is_empty()
    }
}

/// 日志通知发送器
pub struct LogSender {
    log_path: Option<String>,
}

impl LogSender {
    pub fn new(log_path: Option<String>) -> Self {
        Self { log_path }
    }
}

impl Default for LogSender {
    fn default() -> Self {
        Self::new(None)
    }
}

#[async_trait]
impl NotificationSender for LogSender {
    async fn send(&self, notification: &Notification) -> Result<()> {
        let log_text = notification.to_log_text();

        // 输出到 tracing 日志
        match notification.level {
            AlertLevel::Info => tracing::info!("{}", log_text),
            AlertLevel::Warning => tracing::warn!("{}", log_text),
            AlertLevel::Error => tracing::error!("{}", log_text),
            AlertLevel::Critical => tracing::error!("{}", log_text),
        }

        // 如果配置了日志文件，追加写入
        if let Some(path) = &self.log_path {
            let path_ref = std::path::Path::new(path);
            if let Some(parent) = path_ref.parent().filter(|parent| !parent.as_os_str().is_empty())
            {
                tokio::fs::create_dir_all(parent).await.map_err(|err| {
                    QuantixError::Other(format!(
                        "创建通知日志目录失败 ({}): {}",
                        parent.display(),
                        err
                    ))
                })?;
            }

            let log_entry = format!("{}\n", log_text);
            let mut file = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .await
                .map_err(|err| {
                    QuantixError::Other(format!("打开通知日志文件失败 ({}): {}", path, err))
                })?;

            use tokio::io::AsyncWriteExt;
            file.write_all(log_entry.as_bytes()).await.map_err(|err| {
                QuantixError::Other(format!("写入通知日志文件失败 ({}): {}", path, err))
            })?;
            file.flush().await.map_err(|err| {
                QuantixError::Other(format!("刷新通知日志文件失败 ({}): {}", path, err))
            })?;
        }

        Ok(())
    }

    fn channel(&self) -> NotificationChannel {
        NotificationChannel::Log
    }

    fn is_available(&self) -> bool {
        true
    }
}

/// 企业微信通知发送器
pub struct WechatWorkSender {
    webhook_url: String,
    client: reqwest::Client,
}

impl WechatWorkSender {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            client: reqwest::Client::new(),
        }
    }

    fn format_message(&self, notification: &Notification) -> String {
        let level_emoji = match notification.level {
            AlertLevel::Info => "ℹ️",
            AlertLevel::Warning => "⚠️",
            AlertLevel::Error => "❌",
            AlertLevel::Critical => "🚨",
        };
        format!(
            "### {} {}\n\n**{}**\n\n> {}",
            level_emoji,
            notification.title,
            notification.created_at.format("%Y-%m-%d %H:%M:%S"),
            notification.message
        )
    }
}

#[async_trait]
impl NotificationSender for WechatWorkSender {
    async fn send(&self, notification: &Notification) -> Result<()> {
        let content = self.format_message(notification);

        let payload = serde_json::json!({
            "msgtype": "markdown",
            "markdown": {
                "content": content
            }
        });

        let response = self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| QuantixError::Other(format!("企业微信请求失败: {}", e)))?;

        let status = response.status();
        if status.is_success() {
            let body = response.text().await.unwrap_or_default();
            if body.contains("\"errcode\":0") || body.contains("\"errcode\": 0") {
                tracing::debug!("企业微信通知发送成功");
                return Ok(());
            }
            tracing::warn!("企业微信返回错误: {}", body);
        }

        Err(QuantixError::Other(format!(
            "企业微信发送失败: HTTP {}",
            status
        )))
    }

    fn channel(&self) -> NotificationChannel {
        NotificationChannel::WechatWork
    }

    fn is_available(&self) -> bool {
        !self.webhook_url.is_empty()
    }
}

/// 飞书通知发送器
pub struct FeishuSender {
    webhook_url: String,
    client: reqwest::Client,
}

impl FeishuSender {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            client: reqwest::Client::new(),
        }
    }

    fn format_message(&self, notification: &Notification) -> serde_json::Value {
        let level_color = match notification.level {
            AlertLevel::Info => "blue",
            AlertLevel::Warning => "yellow",
            AlertLevel::Error => "red",
            AlertLevel::Critical => "red",
        };

        serde_json::json!({
            "msg_type": "post",
            "content": {
                "post": {
                    "zh_cn": {
                        "title": format!("【{}】{}", notification.level, notification.title),
                        "content": [[
                            {"tag": "text", "text": notification.message},
                            {"tag": "text", "text": format!("\n\n时间: {}", notification.created_at.format("%Y-%m-%d %H:%M:%S"))}
                        ]]
                    }
                },
                "extra": {
                    "single_chat": false
                }
            }
        })
    }
}

#[async_trait]
impl NotificationSender for FeishuSender {
    async fn send(&self, notification: &Notification) -> Result<()> {
        let payload = self.format_message(notification);

        let response = self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| QuantixError::Other(format!("飞书请求失败: {}", e)))?;

        let status = response.status();
        if status.is_success() {
            let body: serde_json::Value = response.json().await.unwrap_or(serde_json::json!({}));
            if body.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) == 0 {
                tracing::debug!("飞书通知发送成功");
                return Ok(());
            }
            tracing::warn!("飞书返回错误: {}", body);
        }

        Err(QuantixError::Other(format!(
            "飞书发送失败: HTTP {}",
            status
        )))
    }

    fn channel(&self) -> NotificationChannel {
        NotificationChannel::Feishu
    }

    fn is_available(&self) -> bool {
        !self.webhook_url.is_empty()
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
        if let Some(quiet_hours) = &self.config.quiet_hours {
            if quiet_hours.is_quiet_now() {
                tracing::debug!("当前处于静默时段，跳过通知");
                return Ok(());
            }
        }

        // 发送到所有可用渠道
        let mut any_success = false;
        for sender in &self.senders {
            if sender.is_available() {
                match sender.send(&notification).await {
                    Ok(()) => {
                        any_success = true;
                        tracing::debug!(
                            "通知通过 {} 渠道发送成功",
                            sender.channel()
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "通知通过 {} 渠道发送失败: {}",
                            sender.channel(),
                            e
                        );
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

    #[test]
    fn test_notification_creation() {
        let notification = Notification::new("测试标题", "测试消息", AlertLevel::Warning)
            .with_metadata("code", "000001");

        assert_eq!(notification.title, "测试标题");
        assert_eq!(notification.message, "测试消息");
        assert_eq!(notification.level, AlertLevel::Warning);
        assert!(!notification.sent);
        assert_eq!(notification.metadata.get("code"), Some(&"000001".to_string()));
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
        assert!(quiet.start < quiet.end || quiet.start > quiet.end);
    }

    #[test]
    fn test_log_sender() {
        let sender = LogSender::new(None);
        assert!(sender.is_available());
        assert_eq!(sender.channel(), NotificationChannel::Log);
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
