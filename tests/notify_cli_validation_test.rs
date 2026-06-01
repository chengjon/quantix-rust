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
