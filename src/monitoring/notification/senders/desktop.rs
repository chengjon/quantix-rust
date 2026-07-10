use async_trait::async_trait;

use crate::core::Result;
use crate::monitoring::AlertLevel;
use crate::monitoring::notification::{Notification, NotificationChannel, NotificationSender};

/// 桌面通知发送器
pub struct DesktopSender;

impl DesktopSender {
    /// 创建桌面通知发送器（无状态，仅持有单例 unit）。
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
                    tracing::warn!("桌面通知发送失败: {}", String::from_utf8_lossy(&o.stderr));
                }
                Err(e) => {
                    tracing::warn!("桌面通知发送失败: {}", e);
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
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
                    tracing::warn!("桌面通知发送失败: {}", String::from_utf8_lossy(&o.stderr));
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
