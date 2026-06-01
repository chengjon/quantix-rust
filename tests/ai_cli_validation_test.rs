use std::process::Command;

fn run_quantix_with_unsupported_ai_provider(args: &[&str]) -> (String, String, bool) {
    let output = Command::new(env!("CARGO_BIN_EXE_quantix"))
        .args(args)
        .env_remove("DEEPSEEK_API_KEY")
        .env_remove("OPENAI_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("OLLAMA_API_BASE")
        .env_remove("OLLAMA_HOST")
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
