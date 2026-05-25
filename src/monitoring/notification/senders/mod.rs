mod desktop;
mod feishu;
mod log;
mod webhook;
mod wechat_work;

pub use desktop::DesktopSender;
pub use feishu::FeishuSender;
pub use log::LogSender;
pub use webhook::WebhookSender;
pub use wechat_work::WechatWorkSender;
