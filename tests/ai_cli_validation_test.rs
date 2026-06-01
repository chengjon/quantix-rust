use std::process::Command;

const AI_PROVIDER_ENV_KEYS: &[&str] = &[
    "DEEPSEEK_API_KEY",
    "DEEPSEEK_BASE_URL",
    "OPENAI_API_KEY",
    "OPENAI_BASE_URL",
    "ANTHROPIC_API_KEY",
    "GEMINI_API_KEY",
    "OLLAMA_API_BASE",
    "OLLAMA_HOST",
];

fn run_quantix_without_ai_provider(args: &[&str]) -> (String, String, bool) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_quantix"));
    command.args(args);
    for key in AI_PROVIDER_ENV_KEYS {
        command.env_remove(key);
    }

    let output = command.output().expect("should run quantix binary");

    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.success(),
    )
}

fn run_quantix_with_unsupported_ai_provider(args: &[&str]) -> (String, String, bool) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_quantix"));
    command.args(args);
    for key in AI_PROVIDER_ENV_KEYS {
        command.env_remove(key);
    }
    let output = command
        .env("GEMINI_API_KEY", "test-key")
        .output()
        .expect("should run quantix binary");

    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.success(),
    )
}

#[test]
fn ai_analyze_fails_closed_without_configured_provider() {
    let (stdout, stderr, success) =
        run_quantix_without_ai_provider(&["ai", "analyze", "--code", "600519.SH"]);

    assert!(
        !success,
        "expected ai analyze to fail without a configured provider, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no placeholder ai analyze output before provider failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("AI provider 尚未配置"),
        "expected provider boundary in stderr, stderr={stderr}"
    );
}

#[test]
fn ai_decide_fails_closed_without_configured_provider() {
    let (stdout, stderr, success) =
        run_quantix_without_ai_provider(&["ai", "decide", "--code", "600519.SH"]);

    assert!(
        !success,
        "expected ai decide to fail without a configured provider, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no placeholder ai decide output before provider failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("AI provider 尚未配置"),
        "expected provider boundary in stderr, stderr={stderr}"
    );
}

#[test]
fn ai_ask_fails_closed_without_configured_provider() {
    let (stdout, stderr, success) =
        run_quantix_without_ai_provider(&["ai", "ask", "--question", "怎么看半导体"]);

    assert!(
        !success,
        "expected ai ask to fail without a configured provider, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no placeholder ai ask output before provider failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("AI provider 尚未配置"),
        "expected provider boundary in stderr, stderr={stderr}"
    );
}

#[test]
fn ai_market_fails_closed_without_configured_provider() {
    let (stdout, stderr, success) = run_quantix_without_ai_provider(&["ai", "market"]);

    assert!(
        !success,
        "expected ai market to fail without a configured provider, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no placeholder ai market output before provider failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("AI provider 尚未配置"),
        "expected provider boundary in stderr, stderr={stderr}"
    );
}

#[test]
fn ai_analyze_rejects_configured_but_unwired_provider() {
    let (stdout, stderr, success) =
        run_quantix_with_unsupported_ai_provider(&["ai", "analyze", "--code", "600519.SH"]);

    assert!(
        !success,
        "expected ai analyze to fail for configured but unwired provider, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stderr.contains(
            "AI 运行时仅支持已接线 provider: deepseek, openai, ollama；当前已配置但未接线: gemini"
        ),
        "expected unsupported provider guidance in stderr, stderr={stderr}"
    );
}
