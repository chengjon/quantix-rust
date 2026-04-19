#![allow(clippy::collapsible_if)]

use crate::core::{QuantixError, Result};

use super::{
    DesktopSender, FeishuSender, LogSender, Notification, NotificationChannel, NotificationConfig,
    NotificationSender, WebhookSender, WechatWorkSender,
};

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
                NotificationChannel::Email => {}
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
                | NotificationChannel::Pushplus => {}
            }
        }

        Self {
            config,
            senders,
            notification_history: Vec::new(),
            max_history: 100,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(NotificationConfig::default())
    }

    pub async fn notify(&mut self, mut notification: Notification) -> Result<()> {
        if notification.level < self.config.min_level {
            tracing::debug!(
                "通知级别 {} 低于最低级别 {}，跳过",
                notification.level,
                self.config.min_level
            );
            return Ok(());
        }

        if let Some(quiet_hours) = &self.config.quiet_hours {
            if quiet_hours.is_quiet_now() {
                tracing::debug!("当前处于静默时段，跳过通知");
                return Ok(());
            }
        }

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

        self.notification_history.push(notification.clone());
        if self.notification_history.len() > self.max_history {
            self.notification_history.remove(0);
        }

        if any_success {
            Ok(())
        } else {
            Err(QuantixError::Other("所有通知渠道发送失败".to_string()))
        }
    }

    pub async fn send_notification(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        level: crate::monitoring::AlertLevel,
    ) -> Result<()> {
        let notification = Notification::new(title, message, level);
        self.notify(notification).await
    }

    pub fn get_history(&self) -> &[Notification] {
        &self.notification_history
    }

    pub fn clear_history(&mut self) {
        self.notification_history.clear();
    }
}
