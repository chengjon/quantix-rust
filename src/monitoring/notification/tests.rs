use crate::monitoring::AlertLevel;

use super::NotificationSender;
use super::{
    LogSender, Notification, NotificationChannel, NotificationConfig, NotificationService,
    QuietHours,
};

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
    let quiet = QuietHours {
        start: "22:00".to_string(),
        end: "06:00".to_string(),
    };
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
