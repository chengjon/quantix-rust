//! Shared test infrastructure for `cli::handlers::tests::*`.
//!
//! Contains environment-variable guards (`ClickHouseDbEnvGuard`,
//! `RuntimeEnvGuard`, `NotificationEnvGuard`) and small path helpers
//! (`repo_root`, `restore_optional_env`) used by every sibling test module.
//! Re-exported by `mod.rs` via `pub(super) use self::support::*` so
//! `use super::*` in sibling test files keeps resolving these names.

use crate::core::config::{
    CLICKHOUSE_DB_ENV, CLICKHOUSE_PASSWORD_ENV, CLICKHOUSE_URL_ENV, CLICKHOUSE_USER_ENV,
};
use crate::core::runtime::{BRIDGE_API_KEY_ENV, BRIDGE_BASE_URL_ENV, STRATEGY_RUNTIME_DB_PATH_ENV};

pub(crate) struct ClickHouseDbEnvGuard {
    url: Option<String>,
    database: Option<String>,
    user: Option<String>,
    password: Option<String>,
}

impl ClickHouseDbEnvGuard {
    pub(crate) fn capture() -> Self {
        Self {
            url: std::env::var(CLICKHOUSE_URL_ENV).ok(),
            database: std::env::var(CLICKHOUSE_DB_ENV).ok(),
            user: std::env::var(CLICKHOUSE_USER_ENV).ok(),
            password: std::env::var(CLICKHOUSE_PASSWORD_ENV).ok(),
        }
    }
}

impl Drop for ClickHouseDbEnvGuard {
    fn drop(&mut self) {
        match &self.url {
            Some(value) => unsafe { std::env::set_var(CLICKHOUSE_URL_ENV, value) },
            None => unsafe { std::env::remove_var(CLICKHOUSE_URL_ENV) },
        }

        match &self.database {
            Some(value) => unsafe { std::env::set_var(CLICKHOUSE_DB_ENV, value) },
            None => unsafe { std::env::remove_var(CLICKHOUSE_DB_ENV) },
        }

        match &self.user {
            Some(value) => unsafe { std::env::set_var(CLICKHOUSE_USER_ENV, value) },
            None => unsafe { std::env::remove_var(CLICKHOUSE_USER_ENV) },
        }

        match &self.password {
            Some(value) => unsafe { std::env::set_var(CLICKHOUSE_PASSWORD_ENV, value) },
            None => unsafe { std::env::remove_var(CLICKHOUSE_PASSWORD_ENV) },
        }
    }
}

pub(crate) struct RuntimeEnvGuard {
    strategy_runtime_db_path: Option<String>,
    bridge_base_url: Option<String>,
    bridge_api_key: Option<String>,
}

impl RuntimeEnvGuard {
    pub(crate) fn capture() -> Self {
        Self {
            strategy_runtime_db_path: std::env::var(STRATEGY_RUNTIME_DB_PATH_ENV).ok(),
            bridge_base_url: std::env::var(BRIDGE_BASE_URL_ENV).ok(),
            bridge_api_key: std::env::var(BRIDGE_API_KEY_ENV).ok(),
        }
    }
}

impl Drop for RuntimeEnvGuard {
    fn drop(&mut self) {
        match &self.strategy_runtime_db_path {
            Some(value) => unsafe { std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, value) },
            None => unsafe { std::env::remove_var(STRATEGY_RUNTIME_DB_PATH_ENV) },
        }

        match &self.bridge_base_url {
            Some(value) => unsafe { std::env::set_var(BRIDGE_BASE_URL_ENV, value) },
            None => unsafe { std::env::remove_var(BRIDGE_BASE_URL_ENV) },
        }

        match &self.bridge_api_key {
            Some(value) => unsafe { std::env::set_var(BRIDGE_API_KEY_ENV, value) },
            None => unsafe { std::env::remove_var(BRIDGE_API_KEY_ENV) },
        }
    }
}

pub(crate) struct NotificationEnvGuard {
    monitor_notify: Option<String>,
    notification_log_path: Option<String>,
    notification_min_level: Option<String>,
    webhook_url: Option<String>,
    wechat_work_webhook_url: Option<String>,
    feishu_webhook_url: Option<String>,
    telegram_bot_token: Option<String>,
    telegram_chat_id: Option<String>,
    discord_webhook_url: Option<String>,
    slack_webhook_url: Option<String>,
    dingtalk_webhook_url: Option<String>,
    pushplus_token: Option<String>,
}

impl NotificationEnvGuard {
    pub(crate) fn capture() -> Self {
        Self {
            monitor_notify: std::env::var("QUANTIX_MONITOR_NOTIFY").ok(),
            notification_log_path: std::env::var("NOTIFICATION_LOG_PATH").ok(),
            notification_min_level: std::env::var("NOTIFICATION_MIN_LEVEL").ok(),
            webhook_url: std::env::var("WEBHOOK_URL").ok(),
            wechat_work_webhook_url: std::env::var("WECHAT_WORK_WEBHOOK_URL").ok(),
            feishu_webhook_url: std::env::var("FEISHU_WEBHOOK_URL").ok(),
            telegram_bot_token: std::env::var("TELEGRAM_BOT_TOKEN").ok(),
            telegram_chat_id: std::env::var("TELEGRAM_CHAT_ID").ok(),
            discord_webhook_url: std::env::var("DISCORD_WEBHOOK_URL").ok(),
            slack_webhook_url: std::env::var("SLACK_WEBHOOK_URL").ok(),
            dingtalk_webhook_url: std::env::var("DINGTALK_WEBHOOK_URL").ok(),
            pushplus_token: std::env::var("PUSHPLUS_TOKEN").ok(),
        }
    }
}

impl Drop for NotificationEnvGuard {
    fn drop(&mut self) {
        restore_optional_env("QUANTIX_MONITOR_NOTIFY", &self.monitor_notify);
        restore_optional_env("NOTIFICATION_LOG_PATH", &self.notification_log_path);
        restore_optional_env("NOTIFICATION_MIN_LEVEL", &self.notification_min_level);
        restore_optional_env("WEBHOOK_URL", &self.webhook_url);
        restore_optional_env("WECHAT_WORK_WEBHOOK_URL", &self.wechat_work_webhook_url);
        restore_optional_env("FEISHU_WEBHOOK_URL", &self.feishu_webhook_url);
        restore_optional_env("TELEGRAM_BOT_TOKEN", &self.telegram_bot_token);
        restore_optional_env("TELEGRAM_CHAT_ID", &self.telegram_chat_id);
        restore_optional_env("DISCORD_WEBHOOK_URL", &self.discord_webhook_url);
        restore_optional_env("SLACK_WEBHOOK_URL", &self.slack_webhook_url);
        restore_optional_env("DINGTALK_WEBHOOK_URL", &self.dingtalk_webhook_url);
        restore_optional_env("PUSHPLUS_TOKEN", &self.pushplus_token);
    }
}

pub(crate) fn restore_optional_env(key: &str, value: &Option<String>) {
    match value {
        Some(value) => unsafe { std::env::set_var(key, value) },
        None => unsafe { std::env::remove_var(key) },
    }
}

pub(crate) fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
