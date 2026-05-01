mod desktop;
mod feishu;
mod log;
mod service;
mod traits;
mod types;
mod webhook;
mod wechat_work;

pub use self::desktop::DesktopSender;
pub use self::feishu::{FeishuConfig, FeishuSender};
pub use self::log::LogSender;
pub use self::service::NotificationService;
pub use self::traits::NotificationSender;
pub use self::types::{Notification, NotificationChannel, NotificationConfig, QuietHours};
pub use self::webhook::WebhookSender;
pub use self::wechat_work::{WechatWorkConfig, WechatWorkSender};

#[cfg(test)]
mod tests;
