use std::process::Command;

const NOTIFY_PROVIDER_ENV_KEYS: &[&str] = &[
    "TELEGRAM_BOT_TOKEN",
    "TELEGRAM_CHAT_ID",
    "WECHAT_WORK_WEBHOOK_URL",
    "FEISHU_WEBHOOK_URL",
    "DISCORD_WEBHOOK_URL",
    "SLACK_WEBHOOK_URL",
    "DINGTALK_WEBHOOK_URL",
    "PUSHPLUS_TOKEN",
    "WEBHOOK_URL",
];

fn run_quantix_without_notify_provider(args: &[&str]) -> (String, String, bool) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_quantix"));
    command.args(args);
    for key in NOTIFY_PROVIDER_ENV_KEYS {
        command.env_remove(key);
    }

    let output = command.output().expect("should run quantix binary");

    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.success(),
    )
}

#[test]
fn notify_check_fails_closed_when_channel_env_missing() {
    let (stdout, stderr, success) =
        run_quantix_without_notify_provider(&["notify", "check", "--channel", "telegram"]);

    assert!(
        !success,
        "expected notify check to fail without channel config, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no placeholder notify check output before config failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("notify channel 尚未配置"),
        "expected notify channel config boundary in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("TELEGRAM_BOT_TOKEN") && stderr.contains("TELEGRAM_CHAT_ID"),
        "expected missing telegram env guidance in stderr, stderr={stderr}"
    );
}

#[test]
fn notify_test_fails_closed_when_requested_channel_env_missing() {
    let (stdout, stderr, success) =
        run_quantix_without_notify_provider(&["notify", "test", "--channel", "telegram"]);

    assert!(
        !success,
        "expected notify test to fail without requested channel config, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no placeholder notify test output before config failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("notify channel 尚未配置"),
        "expected notify channel config boundary in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("TELEGRAM_BOT_TOKEN") && stderr.contains("TELEGRAM_CHAT_ID"),
        "expected missing telegram env guidance in stderr, stderr={stderr}"
    );
}

#[test]
fn notify_send_rejects_unknown_channel_before_progress_output() {
    let (stdout, stderr, success) = run_quantix_without_notify_provider(&[
        "notify",
        "send",
        "--title",
        "风控告警",
        "--message",
        "测试消息",
        "--level",
        "warning",
        "--channel",
        "matrix",
    ]);

    assert!(
        !success,
        "expected notify send to fail for unsupported channel, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no notify send progress output before channel validation failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("notify channel 不支持") && stderr.contains("matrix"),
        "expected unsupported notify channel boundary in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("telegram") && stderr.contains("webhook"),
        "expected supported channel guidance in stderr, stderr={stderr}"
    );
}

#[test]
fn notify_send_rejects_unsupported_level_before_progress_output() {
    let (stdout, stderr, success) = run_quantix_without_notify_provider(&[
        "notify",
        "send",
        "--title",
        "风控告警",
        "--message",
        "测试消息",
        "--level",
        "debug",
        "--channel",
        "log",
    ]);

    assert!(
        !success,
        "expected notify send to fail for unsupported level, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no notify send progress output before level validation failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("无效的通知级别: debug，支持: info, warning, error, critical"),
        "expected notify level validation guidance in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("Unsupported"),
        "expected Unsupported error kind for unsupported notify level, stderr={stderr}"
    );
}

#[test]
fn notify_send_fails_closed_when_requested_channel_env_missing() {
    let (stdout, stderr, success) = run_quantix_without_notify_provider(&[
        "notify",
        "send",
        "--title",
        "风控告警",
        "--message",
        "测试消息",
        "--level",
        "warning",
        "--channel",
        "telegram",
    ]);

    assert!(
        !success,
        "expected notify send to fail without requested channel config, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no notify send progress output before config failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("notify channel 尚未配置"),
        "expected notify channel config boundary in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("TELEGRAM_BOT_TOKEN") && stderr.contains("TELEGRAM_CHAT_ID"),
        "expected missing telegram env guidance in stderr, stderr={stderr}"
    );
}
