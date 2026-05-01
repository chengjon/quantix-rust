use async_trait::async_trait;

use crate::core::{QuantixError, Result};

use super::{Notification, NotificationChannel, NotificationSender};

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
            .map_err(|e| QuantixError::Other(format!("Webhook 请求失败: {}", e)))?;

        if response.status().is_success() {
            tracing::debug!("Webhook 通知发送成功: {}", self.url);
            Ok(())
        } else {
            Err(QuantixError::Other(format!(
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
