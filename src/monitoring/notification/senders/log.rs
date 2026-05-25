use async_trait::async_trait;

use crate::core::Result;
use crate::monitoring::AlertLevel;
use crate::monitoring::notification::{Notification, NotificationChannel, NotificationSender};

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
            let path = std::path::Path::new(path);
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    tokio::fs::create_dir_all(parent).await?;
                }
            }

            use tokio::io::AsyncWriteExt;
            let mut file = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .await?;
            file.write_all(log_entry.as_bytes()).await?;
            file.flush().await?;
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
