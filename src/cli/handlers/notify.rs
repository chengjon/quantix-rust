use super::*;

use crate::core::{CliRuntime, QuantixError, Result};
use crate::monitoring::{NotificationChannel, NotificationConfig};

const NOTIFY_SUPPORTED_CHANNELS: &str =
    "telegram, wechat_work, feishu, discord, slack, dingtalk, pushplus, desktop, webhook, log";

/// 处理通知命令
pub async fn run_notify_command(cmd: NotifyCommands) -> Result<()> {
    match cmd {
        NotifyCommands::Test { channel, message } => run_notify_test(channel, message).await,
        NotifyCommands::Send {
            title,
            message,
            level,
            channel,
        } => run_notify_send(title, message, level, channel).await,
        NotifyCommands::List => run_notify_list().await,
        NotifyCommands::Check { channel } => run_notify_check(channel).await,
    }
}

async fn run_notify_test(channel: String, message: Option<String>) -> Result<()> {
    use crate::monitoring::{AlertLevel, Notification, NotificationService};

    let test_message = message.unwrap_or_else(|| "这是一条测试通知".to_string());
    let mut config = NotificationConfig::from_env();

    if !channel.eq_ignore_ascii_case("all") {
        let target_channel = parse_notify_channel(&channel)?;
        if !is_notify_channel_configured(&target_channel, &config) {
            return Err(notify_channel_missing_config_error(
                "notify test",
                &channel,
                &target_channel,
            ));
        }
        config.enabled_channels = vec![target_channel];
    }

    println!("📤 发送测试通知...");
    println!("   渠道: {}", channel);
    println!("   消息: {}", test_message);

    let mut service = NotificationService::new(config);

    let notification = Notification::new("测试通知", &test_message, AlertLevel::Info);

    match service.notify(notification).await {
        Ok(()) => {
            println!("✅ 测试通知发送成功");
            Ok(())
        }
        Err(e) => {
            println!("❌ 测试通知发送失败: {}", e);
            Err(e)
        }
    }
}

async fn run_notify_send(
    title: String,
    message: String,
    level: String,
    channel: Option<String>,
) -> Result<()> {
    use crate::monitoring::{AlertLevel, Notification, NotificationConfig, NotificationService};

    let alert_level = match level.to_lowercase().as_str() {
        "info" => AlertLevel::Info,
        "warning" => AlertLevel::Warning,
        "error" => AlertLevel::Error,
        "critical" => AlertLevel::Critical,
        _ => {
            return Err(QuantixError::Unsupported(format!(
                "无效的通知级别: {}，支持: info, warning, error, critical",
                level
            )));
        }
    };

    let target_channel = channel.as_deref().map(parse_notify_channel).transpose()?;

    let mut config = NotificationConfig::from_env();

    if let (Some(ch), Some(target_channel)) = (channel.as_deref(), target_channel.as_ref()) {
        if !is_notify_channel_configured(target_channel, &config) {
            return Err(notify_channel_missing_config_error(
                "notify send",
                ch,
                target_channel,
            ));
        }
    }

    println!("📤 发送通知...");
    println!("   标题: {}", title);
    println!("   级别: {:?}", alert_level);
    if let Some(ref ch) = channel {
        println!("   渠道: {}", ch);
    }

    if let Some(target_channel) = target_channel {
        config.enabled_channels = vec![target_channel];
    }

    let mut service = NotificationService::new(config);
    let notification = Notification::new(&title, &message, alert_level);

    match service.notify(notification).await {
        Ok(()) => {
            println!("✅ 通知发送成功");
            Ok(())
        }
        Err(e) => {
            println!("❌ 通知发送失败: {}", e);
            Err(e)
        }
    }
}

async fn run_notify_list() -> Result<()> {
    println!("📋 可用通知渠道:");
    println!();

    let channels = [
        (
            "telegram",
            "Telegram Bot",
            "需要配置 TELEGRAM_BOT_TOKEN 和 TELEGRAM_CHAT_ID",
        ),
        (
            "wechat_work",
            "企业微信",
            "需要配置 WECHAT_WORK_WEBHOOK_URL",
        ),
        ("feishu", "飞书", "需要配置 FEISHU_WEBHOOK_URL"),
        ("discord", "Discord", "需要配置 DISCORD_WEBHOOK_URL"),
        ("slack", "Slack", "需要配置 SLACK_WEBHOOK_URL"),
        (
            "dingtalk",
            "钉钉",
            "需要配置 DINGTALK_WEBHOOK_URL 和 DINGTALK_SECRET",
        ),
        ("pushplus", "PushPlus", "需要配置 PUSHPLUS_TOKEN"),
        ("desktop", "桌面通知", "系统原生桌面通知"),
        ("webhook", "自定义 Webhook", "需要配置 WEBHOOK_URL"),
        ("log", "日志", "输出到日志文件"),
    ];

    for (name, display, desc) in &channels {
        println!("  • {} ({})", display, name);
        println!("    {}", desc);
        println!();
    }

    println!("💡 提示: 使用 'quantix notify check --channel <渠道名>' 测试连通性");

    Ok(())
}

async fn run_notify_check(channel: String) -> Result<()> {
    use crate::monitoring::{AlertLevel, Notification, NotificationService};

    // 检查环境变量配置
    let config = NotificationConfig::from_env();
    let target_channel = parse_notify_channel(&channel)?;

    if !is_notify_channel_configured(&target_channel, &config) {
        return Err(notify_channel_missing_config_error(
            "notify check",
            &channel,
            &target_channel,
        ));
    }

    println!("🔍 检查渠道连通性: {}", channel);
    println!("✅ 环境变量已配置");

    // 尝试发送测试通知
    let mut test_config = config;
    test_config.enabled_channels = vec![target_channel];
    let mut service = NotificationService::new(test_config);

    let notification = Notification::new("连通性测试", "这是一条连通性测试通知", AlertLevel::Info);

    match service.notify(notification).await {
        Ok(()) => {
            println!("✅ 测试通知发送成功");
            Ok(())
        }
        Err(e) => {
            println!("❌ 测试通知发送失败: {}", e);
            Err(e)
        }
    }
}

fn parse_notify_channel(channel: &str) -> Result<NotificationChannel> {
    match channel.to_lowercase().as_str() {
        "telegram" => Ok(NotificationChannel::Telegram),
        "wechat_work" | "wechat" | "企业微信" => Ok(NotificationChannel::WechatWork),
        "feishu" | "飞书" => Ok(NotificationChannel::Feishu),
        "discord" => Ok(NotificationChannel::Discord),
        "slack" => Ok(NotificationChannel::Slack),
        "dingtalk" | "钉钉" => Ok(NotificationChannel::Dingtalk),
        "pushplus" => Ok(NotificationChannel::Pushplus),
        "desktop" => Ok(NotificationChannel::Desktop),
        "webhook" => Ok(NotificationChannel::Webhook),
        "log" => Ok(NotificationChannel::Log),
        _ => Err(QuantixError::Unsupported(format!(
            "notify channel 不支持: {channel}；支持: {NOTIFY_SUPPORTED_CHANNELS}"
        ))),
    }
}

fn is_notify_channel_configured(
    channel: &NotificationChannel,
    config: &NotificationConfig,
) -> bool {
    match channel {
        NotificationChannel::Telegram => {
            std::env::var("TELEGRAM_BOT_TOKEN").is_ok() && std::env::var("TELEGRAM_CHAT_ID").is_ok()
        }
        NotificationChannel::WechatWork => config.wechat_work_webhook.is_some(),
        NotificationChannel::Feishu => config.feishu_webhook.is_some(),
        NotificationChannel::Discord => std::env::var("DISCORD_WEBHOOK_URL").is_ok(),
        NotificationChannel::Slack => std::env::var("SLACK_WEBHOOK_URL").is_ok(),
        NotificationChannel::Dingtalk => std::env::var("DINGTALK_WEBHOOK_URL").is_ok(),
        NotificationChannel::Pushplus => std::env::var("PUSHPLUS_TOKEN").is_ok(),
        NotificationChannel::Desktop | NotificationChannel::Log => true,
        NotificationChannel::Webhook => config.webhook_url.is_some(),
        NotificationChannel::Email => false,
    }
}

fn notify_channel_required_envs(channel: &NotificationChannel) -> &'static str {
    match channel {
        NotificationChannel::Telegram => "TELEGRAM_BOT_TOKEN, TELEGRAM_CHAT_ID",
        NotificationChannel::WechatWork => "WECHAT_WORK_WEBHOOK_URL",
        NotificationChannel::Feishu => "FEISHU_WEBHOOK_URL",
        NotificationChannel::Discord => "DISCORD_WEBHOOK_URL",
        NotificationChannel::Slack => "SLACK_WEBHOOK_URL",
        NotificationChannel::Dingtalk => "DINGTALK_WEBHOOK_URL",
        NotificationChannel::Pushplus => "PUSHPLUS_TOKEN",
        NotificationChannel::Webhook => "WEBHOOK_URL",
        NotificationChannel::Email => "EMAIL_*",
        NotificationChannel::Desktop | NotificationChannel::Log => "",
    }
}

fn notify_channel_missing_config_error(
    command: &str,
    channel: &str,
    target_channel: &NotificationChannel,
) -> QuantixError {
    let required_envs = notify_channel_required_envs(target_channel);
    QuantixError::Unsupported(format!(
        "notify channel 尚未配置: {channel}；请配置 {required_envs} 后再执行 {command} --channel {channel}"
    ))
}
