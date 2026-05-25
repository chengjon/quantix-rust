use async_trait::async_trait;

use crate::core::{QuantixError, Result};
use crate::monitoring::AlertLevel;
use crate::monitoring::notification::{Notification, NotificationChannel, NotificationSender};

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

        let response = self
            .client
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
