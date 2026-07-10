use async_trait::async_trait;

use crate::core::{QuantixError, Result};
use crate::monitoring::AlertLevel;
use crate::monitoring::notification::{Notification, NotificationChannel, NotificationSender};

/// 飞书通知发送器
pub struct FeishuSender {
    webhook_url: String,
    client: reqwest::Client,
}

impl FeishuSender {
    /// 创建飞书机器人发送器：webhook_url 为飞书 incoming webhook 完整 URL，内部初始化共享 reqwest::Client。
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            client: reqwest::Client::new(),
        }
    }

    fn format_message(&self, notification: &Notification) -> serde_json::Value {
        let _level_color = match notification.level {
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

        let response = self
            .client
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
