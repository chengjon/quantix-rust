use async_trait::async_trait;

use crate::core::Result;

use super::types::{Notification, NotificationChannel};

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
