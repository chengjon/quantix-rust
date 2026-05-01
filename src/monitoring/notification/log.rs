use async_trait::async_trait;

use crate::core::Result;
use crate::monitoring::AlertLevel;

use super::{Notification, NotificationChannel, NotificationSender};

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

        match notification.level {
            AlertLevel::Info => tracing::info!("{}", log_text),
            AlertLevel::Warning => tracing::warn!("{}", log_text),
            AlertLevel::Error => tracing::error!("{}", log_text),
            AlertLevel::Critical => tracing::error!("{}", log_text),
        }

        if let Some(path) = &self.log_path {
            let log_entry = format!("{}\n", log_text);
            if let Ok(mut file) = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .await
            {
                use tokio::io::AsyncWriteExt;
                let _ = file.write_all(log_entry.as_bytes()).await;
            }
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
